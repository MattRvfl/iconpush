//! Minimal client for the `com.apple.mobile.MCInstall` lockdown service, which manages
//! configuration profiles. `idevice` doesn't ship it, so we speak the protocol directly:
//! each message is a big-endian u32 length followed by an XML plist.
//!
//! On a non-supervised iPhone, `InstallProfile` hands the profile to the device, which then
//! shows "Profile Downloaded" in Settings: the user still has to tap "Install".

use std::io::Cursor;

use idevice::{Idevice, IdeviceError, IdeviceService};
use plist::{Dictionary, Value};

pub struct McInstallClient {
    idevice: Idevice,
}

impl IdeviceService for McInstallClient {
    fn service_name() -> std::borrow::Cow<'static, str> {
        "com.apple.mobile.MCInstall".into()
    }

    async fn from_stream(idevice: Idevice) -> Result<Self, IdeviceError> {
        Ok(Self { idevice })
    }
}

impl McInstallClient {
    async fn request(&mut self, req: Dictionary) -> Result<Dictionary, String> {
        let mut xml = Vec::new();
        Value::Dictionary(req)
            .to_writer_xml(&mut xml)
            .map_err(|e| format!("plist: {e}"))?;

        let mut msg = (xml.len() as u32).to_be_bytes().to_vec();
        msg.extend_from_slice(&xml);
        self.idevice.send_raw(&msg).await.map_err(|e| format!("envoi: {e}"))?;

        let len = self.idevice.read_raw(4).await.map_err(|e| format!("lecture: {e}"))?;
        let len = u32::from_be_bytes([len[0], len[1], len[2], len[3]]) as usize;
        let body = self.idevice.read_raw(len).await.map_err(|e| format!("lecture: {e}"))?;

        let res: Dictionary = plist::from_reader(Cursor::new(body)).map_err(|e| format!("réponse illisible : {e}"))?;
        match res.get("Status").and_then(Value::as_string) {
            Some("Acknowledged") => Ok(res),
            _ => Err(describe_error(&res)),
        }
    }

    pub async fn hello(&mut self) -> Result<(), String> {
        let mut req = Dictionary::new();
        req.insert("RequestType".into(), "HelloHostIdentifier".into());
        self.request(req).await.map(|_| ())
    }

    pub async fn install_profile(&mut self, profile: Vec<u8>) -> Result<(), String> {
        let mut req = Dictionary::new();
        req.insert("RequestType".into(), "InstallProfile".into());
        req.insert("Payload".into(), Value::Data(profile));
        self.request(req).await.map(|_| ())
    }
}

/// Turn an MCInstall error response into a readable message.
fn describe_error(res: &Dictionary) -> String {
    let chain = res
        .get("ErrorChain")
        .and_then(Value::as_array)
        .map(|errors| {
            errors
                .iter()
                .filter_map(|e| {
                    let e = e.as_dictionary()?;
                    e.get("LocalizedDescription")
                        .or_else(|| e.get("USEnglishDescription"))
                        .and_then(Value::as_string)
                        .map(str::to_owned)
                })
                .collect::<Vec<_>>()
                .join(" — ")
        })
        .filter(|s| !s.is_empty());

    chain.unwrap_or_else(|| {
        let status = res.get("Status").and_then(Value::as_string).unwrap_or("?");
        format!("l'iPhone a refusé le profil (statut {status})")
    })
}
