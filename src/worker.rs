//! Background thread that owns all USB work, so the window never freezes.
//! It polls for iPhones every couple of seconds and runs commands sent by the UI.

use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use crate::device::{self, DeviceInfo};

const POLL_EVERY: Duration = Duration::from_secs(2);

pub enum Command {
    LoadApps(String),
    Push { udid: String, profile: Vec<u8> },
}

pub enum Event {
    Devices(Result<Vec<DeviceInfo>, String>),
    Apps { udid: String, result: Result<HashSet<String>, String> },
    Pushed(Result<(), String>),
}

pub fn spawn(ctx: eframe::egui::Context) -> (Sender<Command>, Receiver<Event>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
    let (ev_tx, ev_rx) = mpsc::channel::<Event>();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                let _ = ev_tx.send(Event::Devices(Err(format!("erreur interne : {e}"))));
                return;
            }
        };
        let send = |ev: Event| {
            let ok = ev_tx.send(ev).is_ok();
            ctx.request_repaint();
            ok
        };

        if !send(Event::Devices(rt.block_on(device::list()))) {
            return;
        }
        loop {
            let ev = match cmd_rx.recv_timeout(POLL_EVERY) {
                Ok(Command::LoadApps(udid)) => {
                    let result = rt.block_on(device::installed_apps(&udid));
                    Event::Apps { udid, result }
                }
                Ok(Command::Push { udid, profile }) => Event::Pushed(rt.block_on(device::push_profile(&udid, profile))),
                Err(RecvTimeoutError::Timeout) => Event::Devices(rt.block_on(device::list())),
                Err(RecvTimeoutError::Disconnected) => return, // window closed
            };
            if !send(ev) {
                return;
            }
        }
    });

    (cmd_tx, ev_rx)
}
