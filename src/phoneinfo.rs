//! Read-only information about the connected iPhone: model, iOS, battery health, storage.
//! Nothing here changes anything on the device.

use idevice::diagnostics_relay::DiagnosticsRelayClient;
use idevice::installation_proxy::InstallationProxyClient;
use idevice::IdeviceService;
use plist::{Dictionary, Value};

#[derive(Clone, Debug, Default)]
pub struct PhoneInfo {
    pub name: String,
    pub model_id: String,
    pub model_name: String,
    pub ios: String,
    pub build: String,
    pub serial: String,
    pub color: String,
    pub region: String,

    pub battery_level: Option<i64>,
    pub charging: Option<bool>,
    pub cycle_count: Option<i64>,
    pub design_mah: Option<i64>,
    pub max_mah: Option<i64>,
    pub temperature_c: Option<f64>,

    pub disk_total: Option<u64>,
    pub disk_free: Option<u64>,

    pub user_apps: Option<usize>,
}

impl PhoneInfo {
    /// Maximum capacity as a % of the original (what Settings calls "Maximum Capacity").
    /// Computed from the raw battery data, so it can differ by a point or two from Settings.
    pub fn health_percent(&self) -> Option<f64> {
        match (self.max_mah, self.design_mah) {
            (Some(max), Some(design)) if design > 0 => Some((max as f64 / design as f64 * 100.0).min(100.0)),
            _ => None,
        }
    }
}

fn int(d: &Dictionary, key: &str) -> Option<i64> {
    match d.get(key)? {
        Value::Integer(i) => i.as_signed().or_else(|| i.as_unsigned().map(|u| u as i64)),
        Value::Real(r) => Some(*r as i64),
        _ => None,
    }
}

fn string(d: &Dictionary, key: &str) -> String {
    d.get(key).and_then(Value::as_string).unwrap_or_default().to_string()
}

/// Marketing name for an iPhone model identifier (falls back to the identifier itself).
pub fn model_name(id: &str) -> String {
    let name = match id {
        "iPhone12,1" => "iPhone 11",
        "iPhone12,3" => "iPhone 11 Pro",
        "iPhone12,5" => "iPhone 11 Pro Max",
        "iPhone12,8" => "iPhone SE (2e génération)",
        "iPhone13,1" => "iPhone 12 mini",
        "iPhone13,2" => "iPhone 12",
        "iPhone13,3" => "iPhone 12 Pro",
        "iPhone13,4" => "iPhone 12 Pro Max",
        "iPhone14,4" => "iPhone 13 mini",
        "iPhone14,5" => "iPhone 13",
        "iPhone14,2" => "iPhone 13 Pro",
        "iPhone14,3" => "iPhone 13 Pro Max",
        "iPhone14,6" => "iPhone SE (3e génération)",
        "iPhone14,7" => "iPhone 14",
        "iPhone14,8" => "iPhone 14 Plus",
        "iPhone15,2" => "iPhone 14 Pro",
        "iPhone15,3" => "iPhone 14 Pro Max",
        "iPhone15,4" => "iPhone 15",
        "iPhone15,5" => "iPhone 15 Plus",
        "iPhone16,1" => "iPhone 15 Pro",
        "iPhone16,2" => "iPhone 15 Pro Max",
        "iPhone17,3" => "iPhone 16",
        "iPhone17,4" => "iPhone 16 Plus",
        "iPhone17,1" => "iPhone 16 Pro",
        "iPhone17,2" => "iPhone 16 Pro Max",
        "iPhone17,5" => "iPhone 16e",
        "iPhone18,3" => "iPhone 17",
        "iPhone18,1" => "iPhone 17 Pro",
        "iPhone18,2" => "iPhone 17 Pro Max",
        "iPhone18,4" => "iPhone Air",
        _ => return id.to_string(),
    };
    name.to_string()
}

pub async fn read(udid: &str) -> Result<PhoneInfo, String> {
    let (mut lockdown, provider) = crate::device::session(udid).await?;
    let mut info = PhoneInfo::default();

    // General device values (no key, no domain = everything lockdown exposes).
    if let Ok(Value::Dictionary(d)) = lockdown.get_value(None, None).await {
        info.name = string(&d, "DeviceName");
        info.model_id = string(&d, "ProductType");
        info.model_name = model_name(&info.model_id);
        info.ios = string(&d, "ProductVersion");
        info.build = string(&d, "BuildVersion");
        info.serial = string(&d, "SerialNumber");
        info.region = string(&d, "RegionInfo");
        info.color = string(&d, "DeviceColor");
    }
    if let Ok(Value::Dictionary(d)) = lockdown.get_value(None, Some("com.apple.mobile.battery")).await {
        info.battery_level = int(&d, "BatteryCurrentCapacity");
        info.charging = d.get("BatteryIsCharging").and_then(Value::as_boolean);
    }
    if let Ok(Value::Dictionary(d)) = lockdown.get_value(None, Some("com.apple.disk_usage")).await {
        info.disk_total = int(&d, "TotalDiskCapacity").map(|v| v as u64);
        info.disk_free = int(&d, "AmountDataAvailable").or_else(|| int(&d, "TotalDataAvailable")).map(|v| v as u64);
    }

    // Battery health from the battery controller (IORegistry "AppleSmartBattery").
    if let Ok(mut diag) = DiagnosticsRelayClient::connect(&provider).await {
        let battery = match diag.ioregistry(None, Some("AppleSmartBattery"), None).await {
            Ok(Some(d)) => Some(d),
            _ => diag.ioregistry(None, None, Some("IOPMPowerSource")).await.ok().flatten(),
        };
        if let Some(b) = battery {
            // On recent iOS the capacity fields live in a nested "BatteryData" dictionary,
            // while CycleCount/Temperature stay at the top level. Look in both.
            let nested = b.get("BatteryData").and_then(Value::as_dictionary);
            let field = |key: &str| int(&b, key).or_else(|| nested.and_then(|n| int(n, key)));

            info.cycle_count = field("CycleCount");
            info.design_mah = field("DesignCapacity");
            info.max_mah = field("AppleRawMaxCapacity")
                .or_else(|| field("NominalChargeCapacity"))
                .or_else(|| field("MaxCapacity"));
            info.temperature_c = field("Temperature").map(|t| t as f64 / 100.0);
            if info.battery_level.is_none() {
                info.battery_level = field("CurrentCapacity");
            }
        }
        let _ = diag.goodbye().await;
    }

    if let Ok(mut proxy) = InstallationProxyClient::connect(&provider).await {
        if let Ok(apps) = proxy.get_apps(Some("User"), None).await {
            info.user_apps = Some(apps.len());
        }
    }

    Ok(info)
}
