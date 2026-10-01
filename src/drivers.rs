//! Apple USB driver: detection, and guided install of iTunes (64-bit) or the Apple Devices app.
//! Both ship "Apple Mobile Device Service" (usbmuxd), which Windows needs to talk to an iPhone.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Official redirect to the latest iTunes for Windows (64-bit).
const ITUNES_URL: &str = "https://www.apple.com/itunes/download/win64";
/// Apple Devices in the Microsoft Store (official Apple app).
const STORE_URI: &str = "ms-windows-store://pdp/?productid=9NP83LWLPZ9K";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Is usbmuxd (Apple Mobile Device Service) answering?
pub fn driver_running() -> bool {
    let addr: SocketAddr = "127.0.0.1:27015".parse().expect("valid address");
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

#[derive(Clone, PartialEq)]
pub enum InstallState {
    Idle,
    Downloading { done: u64, total: u64 },
    Verifying,
    Launched,
    Error(String),
}

pub fn open_store() -> Result<(), String> {
    Command::new("explorer").arg(STORE_URI).spawn().map(|_| ()).map_err(|e| e.to_string())
}

fn installer_path() -> PathBuf {
    crate::library::data_dir().join("downloads").join("iTunes64Setup.exe")
}

/// Download iTunes from Apple, check Apple's digital signature, then start the installer
/// (Windows asks for administrator approval). Runs on its own thread.
pub fn install_itunes(state: Arc<Mutex<InstallState>>, ctx: eframe::egui::Context) {
    std::thread::spawn(move || {
        let set = |s: InstallState| {
            *state.lock().unwrap() = s;
            ctx.request_repaint();
        };
        let result = (|| -> Result<(), String> {
            let path = installer_path();
            download(&path, |done, total| set(InstallState::Downloading { done, total }))?;
            set(InstallState::Verifying);
            verify_apple_signature(&path)?;
            launch(&path)
        })();
        set(match result {
            Ok(()) => InstallState::Launched,
            Err(e) => InstallState::Error(e),
        });
    });
}

fn download(path: &Path, progress: impl Fn(u64, u64)) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("dossier {} : {e}", dir.display()))?;
    }
    let mut res = crate::library::agent()
        .get(ITUNES_URL)
        .call()
        .map_err(|e| format!("téléchargement impossible : {e}"))?;
    let total = res.body().content_length().unwrap_or(0);
    let mut reader = res.body_mut().as_reader();

    let partial = path.with_extension("part");
    let mut file = std::fs::File::create(&partial).map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 256 * 1024];
    let mut done = 0u64;
    let mut last = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("téléchargement interrompu : {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        done += n as u64;
        if done - last > 1024 * 1024 {
            progress(done, total);
            last = done;
        }
    }
    file.flush().map_err(|e| e.to_string())?;
    drop(file);
    if total > 0 && done != total {
        return Err(format!("fichier incomplet ({done} / {total} octets)"));
    }
    std::fs::rename(&partial, path).map_err(|e| e.to_string())?;
    progress(done, total.max(done));
    Ok(())
}

fn powershell(script: &str) -> Result<String, String> {
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("PowerShell indisponible : {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn ps_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "''"))
}

/// Refuse to run the installer unless Windows confirms a valid signature from Apple Inc.
fn verify_apple_signature(path: &Path) -> Result<(), String> {
    let out = powershell(&format!(
        "$s = Get-AuthenticodeSignature -LiteralPath {}; \"$($s.Status)|$($s.SignerCertificate.Subject)\"",
        ps_quote(path)
    ))?;
    let (status, subject) = out.split_once('|').unwrap_or((out.as_str(), ""));
    if status == "Valid" && subject.contains("O=Apple Inc.") {
        Ok(())
    } else {
        let _ = std::fs::remove_file(path);
        Err(format!("signature non valide (statut : {status}, signataire : {subject}) — fichier supprimé"))
    }
}

/// Start-Process goes through ShellExecute, so Windows shows the admin (UAC) prompt.
fn launch(path: &Path) -> Result<(), String> {
    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &format!("Start-Process -FilePath {}", ps_quote(path))])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("lancement de l'installateur impossible : {e}"))
}
