//! R119 Now Playing (MediaRemote extract, empirically from the living
//! Control Center: NowPlayingControllerImpl asks ONE broker —
//! MRMediaRemoteGetNowPlayingClients — instead of every app). The
//! Linux broker for the same idea is MPRIS on the session bus: every
//! player registers as org.mpris.MediaPlayer2.*, the bar asks the bus.

use std::sync::OnceLock;

use zbus::blocking::{Connection, Proxy};

#[derive(Debug, Clone)]
pub struct NowPlaying {
    /// Bus name of the player (target for play/pause/next).
    pub player: String,
    pub title: String,
    pub artist: String,
    pub playing: bool,
}

fn connection() -> Option<&'static Connection> {
    static CONN: OnceLock<Option<Connection>> = OnceLock::new();
    CONN.get_or_init(|| Connection::session().ok()).as_ref()
}

fn player_proxy(name: &str) -> Option<Proxy<'static>> {
    Proxy::new(
        connection()?,
        name.to_string(),
        "/org/mpris/MediaPlayer2",
        "org.mpris.MediaPlayer2.Player",
    )
    .ok()
}

/// The current Now Playing state: the PLAYING player wins; if none
/// plays, the first paused player shows (macOS keeps the last client
/// visible too). None = no player on the bus, the widget vanishes.
pub fn read() -> Option<NowPlaying> {
    // Round 132: the house's own players (Matrix Tube, Matrix Player) speak
    // through the kit's board — first; MPRIS for everything else.
    if let Some(p) = matrixkit_theme::now_playing::read() {
        return Some(NowPlaying { player: format!("{HOUSE}{}", p.app), title: p.title, artist: p.artist, playing: p.playing });
    }
    let conn = connection()?;
    let dbus = zbus::blocking::fdo::DBusProxy::new(conn).ok()?;
    let names = dbus.list_names().ok()?;
    let mut paused: Option<NowPlaying> = None;
    for name in names.iter().map(|n| n.as_str()) {
        if !name.starts_with("org.mpris.MediaPlayer2.") {
            continue;
        }
        let Some(proxy) = player_proxy(name) else { continue };
        let status: String = proxy.get_property("PlaybackStatus").unwrap_or_default();
        let meta: std::collections::HashMap<String, zbus::zvariant::OwnedValue> =
            proxy.get_property("Metadata").unwrap_or_default();
        let title = meta
            .get("xesam:title")
            .and_then(|v| String::try_from(v.clone()).ok())
            .unwrap_or_default();
        let artist = meta
            .get("xesam:artist")
            .and_then(|v| Vec::<String>::try_from(v.clone()).ok())
            .map(|a| a.join(", "))
            .unwrap_or_default();
        if title.is_empty() && artist.is_empty() {
            continue;
        }
        let entry = NowPlaying {
            player: name.to_string(),
            title,
            artist,
            playing: status == "Playing",
        };
        if entry.playing {
            return Some(entry);
        }
        paused.get_or_insert(entry);
    }
    paused
}

/// The prefix of a house player (the kit board, not the bus).
const HOUSE: &str = "matrix:";

/// Fire a player method without blocking the UI thread.
fn call(player: String, method: &'static str) {
    if player.starts_with(HOUSE) {
        use matrixkit_theme::now_playing::{send, Command};
        send(if method == "Next" { Command::Next } else { Command::Toggle });
        return;
    }
    std::thread::spawn(move || {
        if let Some(proxy) = player_proxy(&player) {
            let _ = proxy.call_method(method, &());
        }
    });
}

pub fn play_pause(player: String) {
    call(player, "PlayPause");
}

pub fn next(player: String) {
    call(player, "Next");
}
