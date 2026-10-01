//! Talking to the iPhone over USB, through Apple's usbmuxd service
//! (on Windows: "Apple Mobile Device Service", installed with the Apple Devices app or iTunes).

use std::collections::HashSet;
use std::sync::Mutex;

use idevice::installation_proxy::InstallationProxyClient;
use idevice::lockdown::LockdownClient;
use idevice::provider::UsbmuxdProvider;
use idevice::usbmuxd::{Connection, UsbmuxdAddr, UsbmuxdConnection, UsbmuxdDevice};
use idevice::{IdeviceError, IdeviceService};
use serde::Serialize;

use crate::mcinstall::McInstallClient;

const LABEL: &str = "iconpush";

#[derive(Serialize)]
pub struct DeviceInfo {
    pub udid: String,
    pub name: String,
    #[serde(rename = "iosVersion")]
    pub ios_version: String,
    pub model: String,
}

/// Devices for which a "Trust this computer?" request is currently waiting on the phone.
static PAIRING: Mutex<Option<HashSet<String>>> = Mutex::new(None);

fn usbmuxd_missing(e: IdeviceError) -> String {
    format!(
        "Windows ne voit pas les iPhone ({e}). Installe l'app « Appareils Apple » (Microsoft Store) ou iTunes, puis relance iconpush."
    )
}

async fn usbmuxd() -> Result<UsbmuxdConnection, String> {
    UsbmuxdConnection::default().await.map_err(usbmuxd_missing)
}

async fn find(udid: &str) -> Result<UsbmuxdDevice, String> {
    let mut mux = usbmuxd().await?;
    let devices = mux.get_devices().await.map_err(|e| e.to_string())?;
    devices
        .into_iter()
        .filter(|d| d.udid == udid)
        .min_by_key(|d| d.connection_type != Connection::Usb)
        .ok_or_else(|| "iPhone débranché ?".to_string())
}

fn provider(dev: &UsbmuxdDevice) -> UsbmuxdProvider {
    dev.to_provider(UsbmuxdAddr::default(), LABEL)
}

/// Random UUID-shaped host id, without pulling a uuid crate.
fn random_host_id() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut bytes = Vec::with_capacity(16);
    for i in 0..2u64 {
        let mut h = RandomState::new().build_hasher();
        h.write_u64(i);
        h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
        bytes.extend_from_slice(&h.finish().to_be_bytes());
    }
    let hex: String = bytes.iter().map(|b| format!("{b:02X}")).collect();
    format!("{}-{}-{}-{}-{}", &hex[0..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..32])
}

/// Make sure this computer is trusted by the phone. If it isn't, start pairing in the
/// background: the phone shows "Trust this computer?" and we save the record once accepted.
async fn ensure_paired(dev: &UsbmuxdDevice) -> Result<(), String> {
    let mut mux = usbmuxd().await?;
    if mux.get_pair_record(&dev.udid).await.is_ok() {
        return Ok(());
    }

    {
        let mut pairing = PAIRING.lock().unwrap();
        let set = pairing.get_or_insert_with(HashSet::new);
        if !set.insert(dev.udid.clone()) {
            return Err("Déverrouille ton iPhone et touche « Se fier » sur l'écran.".into());
        }
    }

    let dev = dev.clone();
    tokio::spawn(async move {
        let result = async {
            let mut mux = usbmuxd().await?;
            let buid = mux.get_buid().await.map_err(|e| e.to_string())?;
            let mut lockdown = LockdownClient::connect(&provider(&dev)).await.map_err(|e| e.to_string())?;
            // Loops until the user answers the "Trust" dialog.
            let record = lockdown
                .pair(random_host_id(), buid, Some(LABEL))
                .await
                .map_err(|e| e.to_string())?;
            let bytes = record.serialize().map_err(|e| e.to_string())?;
            let mut mux = usbmuxd().await?;
            mux.save_pair_record(&dev.udid, bytes).await.map_err(|e| e.to_string())
        }
        .await;
        if let Err(e) = result {
            eprintln!("Jumelage avec {} impossible : {e}", dev.udid);
        }
        if let Some(set) = PAIRING.lock().unwrap().as_mut() {
            set.remove(&dev.udid);
        }
    });

    Err("Déverrouille ton iPhone et touche « Se fier » sur l'écran.".into())
}

/// Lockdown connection with an authenticated session.
async fn lockdown(dev: &UsbmuxdDevice) -> Result<LockdownClient, String> {
    ensure_paired(dev).await?;
    let p = provider(dev);
    let mut lockdown = LockdownClient::connect(&p).await.map_err(|e| e.to_string())?;
    let pairing = {
        let mut mux = usbmuxd().await?;
        mux.get_pair_record(&dev.udid).await.map_err(|e| e.to_string())?
    };
    lockdown.start_session(&pairing).await.map_err(|e| match e {
        IdeviceError::DeviceLocked => "Déverrouille ton iPhone.".to_string(),
        IdeviceError::InvalidHostID => {
            "Ce PC n'est plus autorisé : débranche et rebranche l'iPhone, puis touche « Se fier ».".to_string()
        }
        e => e.to_string(),
    })?;
    Ok(lockdown)
}

async fn value(lockdown: &mut LockdownClient, key: &str) -> String {
    lockdown
        .get_value(Some(key), None)
        .await
        .ok()
        .and_then(|v| v.as_string().map(str::to_owned))
        .unwrap_or_default()
}

/// Connected iPhones (USB first). Returns an error message meant for the user when
/// something blocks (service missing, phone locked, trust pending…).
pub async fn list() -> Result<Vec<DeviceInfo>, String> {
    let mut mux = usbmuxd().await?;
    let mut devices = mux.get_devices().await.map_err(|e| e.to_string())?;
    devices.sort_by_key(|d| d.connection_type != Connection::Usb);

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for dev in devices {
        if !seen.insert(dev.udid.clone()) {
            continue;
        }
        let mut lockdown = lockdown(&dev).await?;
        out.push(DeviceInfo {
            name: value(&mut lockdown, "DeviceName").await,
            ios_version: value(&mut lockdown, "ProductVersion").await,
            model: value(&mut lockdown, "ProductType").await,
            udid: dev.udid,
        });
    }
    Ok(out)
}

/// Bundle ids of every app on the phone (user and system).
pub async fn installed_apps(udid: &str) -> Result<Vec<String>, String> {
    let dev = find(udid).await?;
    ensure_paired(&dev).await?;
    let mut client = InstallationProxyClient::connect(&provider(&dev))
        .await
        .map_err(|e| e.to_string())?;
    let apps = client.get_apps(Some("Any"), None).await.map_err(|e| e.to_string())?;
    Ok(apps.into_keys().collect())
}

/// Send a .mobileconfig to the phone. It lands in Settings → "Profile Downloaded".
pub async fn push_profile(udid: &str, profile: Vec<u8>) -> Result<(), String> {
    let dev = find(udid).await?;
    ensure_paired(&dev).await?;
    let mut client = McInstallClient::connect(&provider(&dev))
        .await
        .map_err(|e| format!("connexion au service de profils : {e}"))?;
    client.hello().await?;
    client.install_profile(profile).await
}
