//! The system tray — status-bar reference extract (round 24, reference toolkit l. 25):
//! the bar IS the StatusNotifierWatcher + host (org.kde protocol,
//! the same zbus pattern as the notification daemon). Items register
//! via DBus; their icons (IconPixmap ARGB32 or IconName lookup)
//! appear as buttons in zone 3; click = Activate, right click =
//! SecondaryActivate. With this the LAST foreign surface of the
//! system map falls — even though no app sets an item today:
//! the tray waits ready.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TrayItem {
    pub service: String,
    pub path: String,
    pub title: String,
    /// RGBA8 raw data + edge length (from IconPixmap) — iced-ready.
    pub icon: Option<(u32, u32, Vec<u8>)>,
}

pub enum TrayCommand {
    Activate(String, String),
    Secondary(String, String),
}

#[derive(Default)]
struct WatcherStand {
    items: Vec<String>,
}

struct Watcher {
    state: Arc<Mutex<WatcherStand>>,
    changed: mpsc::Sender<()>,
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl Watcher {
    fn register_status_notifier_item(
        &self,
        service: String,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) {
        // Spec culture: some clients send only a path — then the
        // sender is the service.
        let entry = if service.starts_with('/') {
            let sender = header
                .sender()
                .map(|s| s.to_string())
                .unwrap_or_default();
            format!("{sender}{service}")
        } else if service.starts_with(':') {
            format!("{service}/StatusNotifierItem")
        } else {
            format!("{service}/StatusNotifierItem")
        };
        let mut g = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if !g.items.contains(&entry) {
            g.items.push(entry);
        }
        let _ = self.changed.send(());
    }

    fn register_status_notifier_host(&self, _service: String) {}

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).items.clone()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }
}

fn icon_load(conn: &zbus::blocking::Connection, service: &str, path: &str) -> Option<(u32, u32, Vec<u8>)> {
    let proxy = zbus::blocking::Proxy::new(
        conn,
        service.to_string(),
        path.to_string(),
        "org.kde.StatusNotifierItem",
    )
    .ok()?;
    // IconPixmap: a(iiay) — take the largest variant, turn ARGB→RGBA.
    if let Ok(pixmaps) = proxy.get_property::<Vec<(i32, i32, Vec<u8>)>>("IconPixmap") {
        if let Some((w, h, data)) = pixmaps.into_iter().max_by_key(|(w, _, _)| *w) {
            if w > 0 && h > 0 && data.len() >= (w * h * 4) as usize {
                let mut rgba = Vec::with_capacity(data.len());
                for px in data.chunks_exact(4) {
                    // Netzwerk-ARGB → RGBA
                    rgba.extend_from_slice(&[px[1], px[2], px[3], px[0]]);
                }
                return Some((w as u32, h as u32, rgba));
            }
        }
    }
    None
}

fn title_load(conn: &zbus::blocking::Connection, service: &str, path: &str) -> String {
    zbus::blocking::Proxy::new(conn, service.to_string(), path.to_string(), "org.kde.StatusNotifierItem")
        .ok()
        .and_then(|p| p.get_property::<String>("Title").ok())
        .unwrap_or_default()
}

/// Starts watcher+host; delivers the living item list and a command
/// sender (Activate). The bar polls the list in its pulse.
pub fn start() -> (Arc<Mutex<Vec<TrayItem>>>, mpsc::Sender<TrayCommand>) {
    let items: Arc<Mutex<Vec<TrayItem>>> = Arc::new(Mutex::new(Vec::new()));
    let (command_tx, command_rx) = mpsc::channel::<TrayCommand>();
    let items_thread = items.clone();

    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<()>();
        let state = Arc::new(Mutex::new(WatcherStand::default()));
        let make = || -> zbus::Result<zbus::blocking::Connection> {
            let conn = zbus::blocking::connection::Builder::session()?
                .serve_at(
                    "/StatusNotifierWatcher",
                    Watcher { state: state.clone(), changed: tx.clone() },
                )?
                .build()?;
            use zbus::fdo::RequestNameFlags;
            let dbus = zbus::blocking::fdo::DBusProxy::new(&conn)?;
            let _ = dbus.request_name(
                "org.kde.StatusNotifierWatcher".try_into().unwrap(),
                RequestNameFlags::AllowReplacement | RequestNameFlags::ReplaceExisting,
            );
            Ok(conn)
        };
        let Ok(conn) = make() else {
            eprintln!("matrix-bar: Tray-Watcher-Start scheiterte");
            return;
        };
        loop {
            // Execute commands (clicks).
            while let Ok(b) = command_rx.try_recv() {
                match b {
                    TrayCommand::Activate(d, p) => {
                        if let Ok(proxy) = zbus::blocking::Proxy::new(
                            &conn, d, p, "org.kde.StatusNotifierItem",
                        ) {
                            let _ = proxy.call_method("Activate", &(0i32, 0i32));
                        }
                    }
                    TrayCommand::Secondary(d, p) => {
                        if let Ok(proxy) = zbus::blocking::Proxy::new(
                            &conn, d, p, "org.kde.StatusNotifierItem",
                        ) {
                            let _ = proxy.call_method("SecondaryActivate", &(0i32, 0i32));
                        }
                    }
                }
            }
            // Registrierungen → Items samt Icons neu einlesen.
            if rx.try_recv().is_ok() {
                let list = state.lock().unwrap_or_else(|e| e.into_inner()).items.clone();
                let mut new = Vec::new();
                for entry in list {
                    let (service, path) = match entry.find('/') {
                        Some(i) => (entry[..i].to_string(), entry[i..].to_string()),
                        None => (entry.clone(), String::from("/StatusNotifierItem")),
                    };
                    new.push(TrayItem {
                        icon: icon_load(&conn, &service, &path),
                        title: title_load(&conn, &service, &path),
                        service,
                        path,
                    });
                }
                *items_thread.lock().unwrap_or_else(|e| e.into_inner()) = new;
            }
            std::thread::sleep(std::time::Duration::from_millis(120));
        }
    });

    (items, command_tx)
}
