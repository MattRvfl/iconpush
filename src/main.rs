//! iconpush desktop app: serves the web UI on 127.0.0.1 and exposes a small API that
//! talks to the iPhone over USB. The UI is the same one published on GitHub Pages.

mod device;
mod mcinstall;

use std::io::Read;

use include_dir::{include_dir, Dir};
use serde_json::json;
use tiny_http::{Header, Method, Request, Response, Server};

static WEB: Dir = include_dir!("$CARGO_MANIFEST_DIR/web");

const MAX_PROFILE_BYTES: usize = 20 * 1024 * 1024;

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => "application/octet-stream",
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}

fn json_response(status: u16, body: serde_json::Value) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_data(body.to_string().into_bytes())
        .with_status_code(status)
        .with_header(header("Content-Type", "application/json; charset=utf-8"))
        .with_header(header("Cache-Control", "no-store"))
}

/// Random secret put in the URL we open; API calls must echo it back. This stops other
/// websites open in the browser from driving the local API.
fn random_token() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    (0..2)
        .map(|i| {
            let mut h = RandomState::new().build_hasher();
            h.write_u32(i);
            format!("{:016x}", h.finish())
        })
        .collect()
}

fn query_param<'a>(url: &'a str, key: &str) -> Option<&'a str> {
    url.split_once('?')?.1.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then_some(v)
    })
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn handle_api(rt: &tokio::runtime::Runtime, mut req: Request, path: &str) {
    let url = req.url().to_string();
    let udid = query_param(&url, "udid").map(percent_decode).unwrap_or_default();

    let response = match (req.method(), path) {
        (Method::Get, "/api/status") => match rt.block_on(device::list()) {
            Ok(devices) => json_response(200, json!({ "app": "iconpush", "version": env!("CARGO_PKG_VERSION"), "devices": devices })),
            Err(e) => json_response(200, json!({ "app": "iconpush", "version": env!("CARGO_PKG_VERSION"), "devices": [], "error": e })),
        },
        (Method::Get, "/api/apps") => match rt.block_on(device::installed_apps(&udid)) {
            Ok(ids) => json_response(200, json!({ "bundleIds": ids })),
            Err(e) => json_response(500, json!({ "error": e })),
        },
        (Method::Post, "/api/push") => {
            let mut body = Vec::new();
            let read = req.as_reader().take(MAX_PROFILE_BYTES as u64 + 1).read_to_end(&mut body);
            if read.is_err() || body.len() > MAX_PROFILE_BYTES {
                json_response(413, json!({ "ok": false, "error": "profil trop gros" }))
            } else if !body.starts_with(b"<?xml") {
                json_response(400, json!({ "ok": false, "error": "ce n'est pas un profil" }))
            } else {
                match rt.block_on(device::push_profile(&udid, body)) {
                    Ok(()) => {
                        println!("✔ Profil envoyé sur l'iPhone {udid}");
                        json_response(200, json!({ "ok": true }))
                    }
                    Err(e) => {
                        eprintln!("✘ Envoi impossible : {e}");
                        json_response(500, json!({ "ok": false, "error": e }))
                    }
                }
            }
        }
        _ => json_response(404, json!({ "error": "inconnu" })),
    };
    let _ = req.respond(response);
}

fn serve_file(req: Request, path: &str) {
    let rel = path.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    let response = match WEB.get_file(rel) {
        Some(file) => Response::from_data(file.contents().to_vec())
            .with_header(header("Content-Type", content_type(rel)))
            .with_header(header("Cache-Control", "no-cache")),
        None => Response::from_string("404").with_status_code(404),
    };
    let _ = req.respond(response);
}

/// Make the Windows console print accents correctly.
#[cfg(windows)]
fn utf8_console() {
    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleOutputCP(code_page: u32) -> i32;
    }
    // SAFETY: plain Win32 call with a constant argument.
    unsafe {
        SetConsoleOutputCP(65001);
    }
}

#[cfg(not(windows))]
fn utf8_console() {}

fn main() {
    utf8_console();
    let server = match Server::http("127.0.0.1:0") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Impossible de démarrer le serveur local : {e}");
            std::process::exit(1);
        }
    };
    let port = server.server_addr().to_ip().map(|a| a.port()).unwrap_or(0);
    let host = format!("127.0.0.1:{port}");
    let token = random_token();
    let url = format!("http://{host}/?t={token}");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    println!("iconpush {} — des icônes perso pour ton iPhone", env!("CARGO_PKG_VERSION"));
    println!();
    println!("  Ouvre : {url}");
    println!("  Laisse cette fenêtre ouverte pendant que tu utilises iconpush.");
    println!("  Ferme-la pour quitter.");
    println!();
    if std::env::var_os("ICONPUSH_NO_BROWSER").is_some() {
        // Used for automated tests: don't pop a browser window.
    } else if open::that(&url).is_err() {
        println!("  (Le navigateur ne s'est pas ouvert tout seul : copie le lien ci-dessus.)");
    }

    for req in server.incoming_requests() {
        // Reject requests aimed at another host name (DNS rebinding).
        let host_ok = req
            .headers()
            .iter()
            .find(|h| h.field.equiv("Host"))
            .map(|h| h.value.as_str() == host)
            .unwrap_or(false);
        if !host_ok {
            let _ = req.respond(Response::from_string("forbidden").with_status_code(403));
            continue;
        }

        let path = req.url().split('?').next().unwrap_or("/").to_string();
        if path.starts_with("/api/") {
            let token_ok = req
                .headers()
                .iter()
                .find(|h| h.field.equiv("X-Iconpush-Token"))
                .map(|h| h.value.as_str() == token)
                .unwrap_or(false);
            if !token_ok {
                let _ = req.respond(json_response(403, json!({ "error": "jeton invalide" })));
                continue;
            }
            handle_api(&rt, req, &path);
        } else {
            serve_file(req, &path);
        }
    }
}
