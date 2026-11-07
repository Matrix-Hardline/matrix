//! Matrix Dock — app #11, member of the MatrixKit bar family.
//!
//! Since 2.0 the pill has TWO rows:
//!
//! * **Row 1 — shortcuts**: pinned apps (`~/.config/matrix/
//!   dock-pins`, one app_id per word) plus everything running. The dot
//!   says "running", accent says "has focus". A click fetches the window
//!   (the focused app cycles) — or STARTS the pin if it rests.
//! * **Row 2 — widgets**: like the bar, only in the dock (`~/.config/
//!   matrix/dock-widgets`, default `apps`): "Apps" opens the launcher
//!   Matrix Start centered; `zentrale` and `uhr` exist too. Since R173
//!   the displays that only lived up top are available here as well —
//!   `user` (the session menu: lock, log out, restart), `battery`,
//!   `wlan`, `bluetooth` and `matrix` (the system menu). They are the
//!   way out of the session once the bar goes.
//!
//! Layer::Top deliberately (task #39); `--reserve` keeps the edge
//! free since the DMS bars sleep (`--reservieren` accepted as legacy alias).

use matrixkit_widgets::iced;
use matrixkit_widgets::iced_layershell;
use std::collections::HashMap;

use matrixkit_widgets::iced::widget::{button, column, container, image, row, Space};
use matrixkit_widgets::iced::{Color, Element, Length, Subscription, Task};
use matrixkit_widgets::iced_layershell::to_layer_message;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use matrixkit_widgets::{color, tick};

/// R216: the dock in the house (matrixkit-shell, no iced) — `--haus`.
mod haus;

/// Image edge of the app icon in the pill.
const ICON: f32 = 44.0;
/// Button surface around the icon (inner air for the hover tone).
const BUTTON: f32 = 52.0;
/// Button rounding: large floating element.
const BUTTON_RADIUS: f32 = mk::radius::LARGE;
const GAP: f32 = mk::spacing::XS;
const MARGIN: f32 = mk::spacing::S;
/// Row 1 (button + dot) + row 2 (widgets) + inner margins.
const HEIGHT: u32 = 120;
/// R172: a tile is taller than an icon button (two text lines plus
/// its rows), so the shelf gets its own height instead of squeezing
/// the tiles into the icon metric.
/// R172f (user): the shelf follows its content instead of standing
/// tall — a tile is two text lines, the widget column is two short
/// rows, and nothing needs more air than that.
const SHELF_HEIGHT: u32 = 96;
/// R168d: the folded dock — an INVISIBLE catch strip, flush with the
/// screen edge (the probe showed fingers land at the outermost pixel,
/// while the normal dock floats on a margin).
const STRIP: u32 = 16;
/// The running dot below the icon.
const DOT: f32 = 4.0;
/// How long the dock keeps its OSD face after a key press (ms).
const OSD_DWELL_TIME: u128 = 1500;
/// R173: the head island of the session menu (avatar + name) is not
/// part of the measured entry groups — its own padding, height and
/// island gap are this much.
const HEADER_ROOM: f32 = 68.0;

/// The side air the tiled shelf keeps to the screen edges — the
/// SAME 10 px the bar keeps up top, so the two system furnitures
/// share one silhouette instead of each inventing its own inset.
const SHELF_SIDE: f32 = 10.0;

fn width(tiles: usize) -> u32 {
    let n = tiles.max(3) as f32; // room for the widget row
    (n * BUTTON + (n - 1.0) * GAP + 2.0 * MARGIN) as u32
}

/// Row scroller (R49): a dock row as an invisible horizontal
/// scroller — capped to the pill inner width. If the content fits,
/// the caller centers it; if it overflows, wheel/trackpad scroll
/// sideways, and only in the row under the mouse.
fn lines_scroller<'a>(
    content: Element<'a, Msg>,
    inner_width: f32,
    id: &'static str,
) -> Element<'a, Msg> {
    let core = container(
        iced::widget::scrollable(content)
            .id(iced::advanced::widget::Id::new(id))
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::new()
                    .width(0)
                    .scroller_width(0)
                    .margin(0),
            ))
            .width(Length::Shrink),
    )
    .max_width(inner_width);
    // R62 (user finding on the PC): a VERTICAL mouse wheel should push
    // the cross row — iced only translates that with Shift held
    // (scrollable.rs: "movement = if !is_shift_pressed…"); the Surface
    // trackpad delivered real X deltas and hid the case.
    // The catcher below turns wheel-Y into the cross gesture; horizontal
    // deltas are still consumed by the scroller itself.
    iced::widget::mouse_area(core)
        .on_scroll(move |delta| {
            let y = match delta {
                iced::mouse::ScrollDelta::Lines { y, .. } => y * 48.0,
                iced::mouse::ScrollDelta::Pixels { y, .. } => y,
            };
            Msg::RowWheel(id, y)
        })
        .into()
}

fn main() -> Result<(), iced_layershell::Error> {
    // R216: `--haus` starts the dock in the house (no iced). It replaces
    // the older first-light shelf (licht.rs, Lager 33–42), so the old
    // `--erstlicht` leads there too. iced stays the default until the
    // house dock has passed the finger probe on the device.
    // R216 (02.11.2026): the HOUSE is the standard. Way back without a
    // reinstall: `--iced`, or the setting `shell-iced=on` (then kill — the
    // watchdog respawns into iced).
    let iced = std::env::args().any(|a| a == "--iced") || mk::settings::read("shell-iced").as_deref() == Some("on");
    if std::env::args().any(|a| a == "--haus" || a == "--erstlicht") || !iced {
        // Round 80: one dock — the watchdog and a manual restart once
        // started two at the same moment (doubled tiles and zones).
        if !mkw::daemon_single() {
            eprintln!("[dock] ältere Instanz lebt — Ende");
            return Ok(());
        }
        if let Err(e) = haus::run() {
            eprintln!("[dock] Fensterhaus-Fehler: {e}");
        }
        return Ok(());
    }
    matrixkit_widgets::present_mailbox();
    use matrixkit_widgets::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
    let _ = (Anchor::Bottom, KeyboardInteractivity::None, Layer::Top);
    // R174: the shelf is no longer ONE surface. Real glass is a
    // compositor property of a layer surface (rule on the namespace),
    // so a tile can only wear it by BEING a surface — the control
    // centre has done it this way since R134. The dock therefore runs
    // the daemon pattern: a spine surface for the things that need a
    // continuous strip (OSD face, grown menus, the auto-hide catch
    // edge) plus one surface per tile and per widget.
    mkw::app_canvas::run::<App>("matrix-dock", mkw::app_canvas::background())
}

impl mkw::app_canvas::Daemon for App {
    type Msg = Msg;
    fn new() -> (Self, Task<Msg>) {
        App::new()
    }
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        App::update(self, msg)
    }
    fn view(&self, window: iced::window::Id) -> Element<'_, Msg> {
        App::view(self, window)
    }
    fn subscription(&self) -> iced::Subscription<Msg> {
        App::subscription(self)
    }
}

/// The spine's own geometry — everything the old single window was.
/// In the tiled shelf it stays EMPTY (namespace without a glass rule),
/// so nothing frosts across the whole edge; it only carries the OSD
/// face, the grown menus and the auto-hide catch strip.
/// R209: on a duo the dock (spine and every chip) lives on its own panel —
/// before, it landed wherever the compositor put new surfaces.
fn dock_output() -> iced_layershell::reexport::OutputOption {
    mkw::panel::output_option("dock-output", mk::device::DUO_DOCK_PANEL)
}

fn spine_settings(reserve: bool) -> iced_layershell::reexport::NewLayerShellSettings {
    use matrixkit_widgets::iced_layershell::reexport::{
        Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings,
    };
    NewLayerShellSettings {
        // R172: a shelf across the whole edge must be anchored on
        // BOTH sides — with Bottom alone the compositor rejects a
        // zero width outright (protocol error 1, seen live on the
        // Surface). The icon dock keeps its content-sized pill.
        size: Some((
            if tiled_now() { 0 } else { width(1) },
            if tiled_now() { SHELF_HEIGHT } else { HEIGHT }
                + 2 * mkw::panel::SHADOW_MARGIN as u32,
        )),
        anchor: if tiled_now() {
            Anchor::Bottom | Anchor::Left | Anchor::Right
        } else {
            Anchor::Bottom
        },
        // Shadow room below (user finding 2026-07-15): the pill stays
        // 10 px above the edge, but the surface reaches 14 px beyond —
        // the blur fades instead of breaking hard at the surface bottom
        // (the "rectangle wings").
        margin: Some((0, 0, SPINE_MARGIN_BOTTOM, 0)),
        layer: Layer::Top,
        keyboard_interactivity: KeyboardInteractivity::None,
        exclusive_zone: Some(if reserve { reserved_strip() } else { 0 }),
        // The glass rule matches on the NAMESPACE (R106). Tiled, the
        // spine must NOT match it — otherwise the very glass slab the
        // user rejected comes back behind the free-floating tiles.
        namespace: Some(String::from(if tiled_now() {
            "matrix-shelf"
        } else {
            "matrix-dock"
        })),
        output_option: dock_output(),
        ..Default::default()
    }
}

// ------------------------------------------------------- Row 1: tiles

/// A dock tile: pin and/or running app.
#[derive(Debug, Clone)]
pub(crate) struct Tile {
    pub(crate) app_id: String,
    pub(crate) window: Vec<mkw::canvas::OpenWindow>,
    pub(crate) pinned: bool,
    pub(crate) focus: bool,
    /// NSDockTile.badgeLabel extract: a small note above the icon.
    badge: Option<String>,
}

impl Tile {
    pub(crate) fn running(&self) -> bool {
        !self.window.is_empty()
    }

    /// Click target: the focused app cycles, else the oldest window.
    pub(crate) fn target(&self) -> Option<u64> {
        if self.focus && self.window.len() > 1 {
            let idx = self.window.iter().position(|f| f.focus)?;
            return Some(self.window[(idx + 1) % self.window.len()].id);
        }
        self.window.first().map(|f| f.id)
    }
}

/// Which face the dock wears. Contract `dock-style`: "tiles"
/// (default since R172) or "icons" for the old squircles.
fn tiled_now() -> bool {
    mk::settings::read("dock-style").as_deref() != Some("icons")
}

/// The maker line. First-party apps say "Matrix"; anything else is
/// honestly labelled as coming from outside — the second line only
/// earns its place when it can say something true.
pub(crate) fn maker_of(app_id: &str) -> &'static str {
    if app_id.starts_with("matrix-") {
        "Matrix"
    } else {
        "Programm"
    }
}


/// Pins from the settings file (one app_id per word).
/// Outer-name migration (2026-07-21): pins written before the English
/// rename keep working — old ids map to the new ones on read.
fn migrate_pin(id: &str) -> String {
    match id {
        "matrix-dateien" => "matrix-files", "matrix-drives" => "matrix-disks",
        "matrix-einstellungen" => "matrix-settings", "matrix-klaenge" => "matrix-sounds",
        "matrix-tastatur" => "matrix-keyboard", "matrix-aufnahme" => "matrix-capture",
        "matrix-mitteilungen" => "matrix-notifications", "matrix-sperre" => "matrix-lock",
        "matrix-hintergrund" => "matrix-wallpaper", "matrix-farben" => "matrix-colors",
        "matrix-hilfe" => "matrix-help", "matrix-zentrale" => "matrix-central",
        "matrix-kontext" => "matrix-context", "matrix-wiederherstellung" => "matrix-recovery",
        "matrix-schluessel-app" => "matrix-keys-app",
        other => other,
    }
    .to_string()
}

fn pins_read() -> Vec<String> {
    mk::settings::read("dock-pins")
        .unwrap_or_else(|| String::from("matrix-settings"))
        .split_whitespace()
        // The rename migration was written but never called (found by
        // a dead-code warning, 02.11.2026): pins kept from before the
        // English rename stayed unknown ids.
        .map(migrate_pin)
        .collect()
}

/// Pins first (in pin order), then running apps by seniority.
pub(crate) fn tiles_read() -> Vec<Tile> {
    let pins = pins_read();
    let mut after_app: HashMap<String, Vec<mkw::canvas::OpenWindow>> = HashMap::new();
    for f in mkw::canvas::windows() {
        after_app.entry(f.app_id.clone()).or_default().push(f);
    }
    let mut tiles: Vec<Tile> = Vec::new();
    for pin in &pins {
        let mut window = after_app.remove(pin).unwrap_or_default();
        window.sort_by_key(|f| f.id);
        let focus = window.iter().any(|f| f.focus);
        let badge = mk::badge::read(pin);
        tiles.push(Tile { app_id: pin.clone(), window, pinned: true, focus, badge });
    }
    let mut running_ones: Vec<Tile> = after_app
        .into_iter()
        .map(|(app_id, mut window)| {
            window.sort_by_key(|f| f.id);
            let focus = window.iter().any(|f| f.focus);
            let badge = mk::badge::read(&app_id);
            Tile { app_id, window, pinned: false, focus, badge }
        })
        .collect();
    running_ones.sort_by_key(|k| k.window[0].id);
    tiles.extend(running_ones);
    tiles
}

/// Initial letter for apps without a Matrix icon ("org.mozilla.firefox" → F).
fn initial(app_id: &str) -> String {
    app_id
        .rsplit('.')
        .next()
        .and_then(|s| s.chars().next())
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| String::from("?"))
}

// ------------------------------------------------------- Row 2: widgets

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Widget {
    Apps,
    Central,
    Clock,
    Clipboard,
    /// R69: opens the capture panel.
    Capture,
    /// R173: the displays that moved down from the bar.
    Battery,
    Wlan,
    Bluetooth,
    /// The session menu — the way OUT of the session.
    User,
    /// The Matrix menu (Apple-menu extract, round 28).
    Matrix,
    /// R187: Hardline — one glance says whether this device is in the
    /// net at all, one click switches it COMPLETELY off.
    Hardline,
}

impl Widget {
    fn out(name: &str) -> Option<Self> {
        // R103: English values are the contract; German stays accepted
        // (existing ~/.config/matrix/dock-widgets files).
        match name {
            "apps" => Some(Self::Apps),
            "central" | "zentrale" => Some(Self::Central),
            "clock" | "uhr" => Some(Self::Clock),
            "clipboard" | "zwischenablage" => Some(Self::Clipboard),
            "capture" | "aufnahme" => Some(Self::Capture),
            // R173: the same names the bar uses for the same things —
            // one vocabulary, so a widget keeps its name when it moves.
            "battery" | "akku" => Some(Self::Battery),
            "wlan" | "wifi" => Some(Self::Wlan),
            "bluetooth" => Some(Self::Bluetooth),
            "user" | "nutzer" => Some(Self::User),
            "matrix" => Some(Self::Matrix),
            "hardline" => Some(Self::Hardline),
            _ => None,
        }
    }

    /// Does the widget's own content already carry inner air? Buttons
    /// do (family padding), a bare line of text does not — and the
    /// bare glass body would otherwise cling to the glyphs (R172/R173,
    /// the clock stuck to its edge for exactly this reason).
    fn brings_padding(self) -> bool {
        !matches!(self, Self::Clock | Self::Battery)
    }

    /// Does this widget need the network state? Only then does the
    /// dock keep a listener on nmcli (R135: no listener, no cost).
    fn needs_network(self) -> bool {
        matches!(self, Self::Wlan | Self::Bluetooth)
    }
}

// ----------------------------------------------------- R173: out of the bar

/// The entries of the session menu. Final things ask via a second click.
/// Ported verbatim from the bar — the way out of the session must read
/// the same wherever it hangs.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Lock,
    Logout,
    Standby,
    Restart,
    Shutdown,
}

impl Action {
    const ALL: [Action; 5] = [
        Action::Lock,
        Action::Logout,
        Action::Standby,
        Action::Restart,
        Action::Shutdown,
    ];

    fn glyph(self) -> char {
        match self {
            Action::Lock => mkw::symbol::LOCK,
            Action::Logout => mkw::symbol::LOGOUT,
            Action::Standby => mkw::symbol::DARK_MODE,
            Action::Restart => mkw::symbol::RESTART,
            Action::Shutdown => mkw::symbol::POWER,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Action::Lock => "Sperren",
            Action::Logout => "Abmelden …",
            Action::Standby => "Standby",
            Action::Restart => "Neustart",
            Action::Shutdown => "Herunterfahren",
        }
    }

    /// Final actions want a second, red click.
    fn permanent(self) -> bool {
        matches!(self, Action::Restart | Action::Shutdown)
    }

    fn question(self) -> &'static str {
        match self {
            Action::Restart => "Wirklich neu starten?",
            Action::Shutdown => "Wirklich ausschalten?",
            _ => "",
        }
    }

    fn execute(self) {
        match self {
            Action::Lock => {
                mkw::panel::app_start("matrix-lock");
            }
            // niri shows its own confirmation dialog — hence "…".
            // The logout sound plays SYNCHRONOUSLY to the end before the
            // session falls — otherwise poweroff would cut it off (R30).
            Action::Logout => {
                mk::feedback::now("abmeldung", "02-abmeldung.wav");
                mk::canvas::logout(true);
            }
            Action::Standby => {
                mk::command::silent("systemctl", &["suspend"]);
            }
            Action::Restart => {
                mk::feedback::now("abmeldung", "02-abmeldung.wav");
                mk::command::silent("systemctl", &["reboot"]);
            }
            Action::Shutdown => {
                mk::feedback::now("abmeldung", "02-abmeldung.wav");
                mk::command::silent("systemctl", &["poweroff"]);
            }
        }
    }
}

/// Display name of the account: GECOS from /etc/passwd, else the login name.
fn user_read() -> String {
    let login = std::env::var("USER").unwrap_or_else(|_| String::from("Konto"));
    if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
        for l in passwd.lines() {
            if l.starts_with(&format!("{login}:")) {
                if let Some(gecos) = l.split(':').nth(4) {
                    let name = gecos.split(',').next().unwrap_or("").trim();
                    if !name.is_empty() {
                        return large(name);
                    }
                }
            }
        }
    }
    large(&login)
}

/// First letter capitalized — account names stand upright in the shelf.
fn large(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_else(|| s.to_string())
}

/// Battery level (0–100, charging?) — None on devices without one (PC).
fn battery_read() -> Option<(u8, bool)> {
    let rd = std::fs::read_dir("/sys/class/power_supply").ok()?;
    for e in rd.flatten() {
        let path = e.path();
        let Ok(cap) = std::fs::read_to_string(path.join("capacity")) else {
            continue;
        };
        let Ok(percent) = cap.trim().parse::<u8>() else {
            continue;
        };
        let loading = std::fs::read_to_string(path.join("status"))
            .map(|s| s.trim() == "Charging")
            .unwrap_or(false);
        return Some((percent.min(100), loading));
    }
    None
}

/// Name of the active network connection (without loopback).
fn net_read() -> Option<String> {
    let out = std::process::Command::new("nmcli")
        .args(["-t", "-f", "NAME,TYPE", "connection", "show", "--active"])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find(|l| !l.contains("loopback"))
        .and_then(|l| l.split(':').next())
        .map(|s| s.to_string())
}

/// Is the WLAN radio on? None = no NetworkManager (the widget then
/// says nothing instead of lying).
fn wlan_read() -> Option<bool> {
    mk::command::first_line("nmcli", &["radio", "wifi"]).map(|z| z == "enabled")
}

fn bt_read() -> Option<bool> {
    let out = std::process::Command::new("bluetoothctl")
        .arg("show")
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    if !s.contains("Controller") {
        return None;
    }
    Some(s.lines().any(|l| l.trim() == "Powered: yes"))
}

/// What the dock has grown open above its shelf.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Sheet {
    /// R173: the session menu of the user widget.
    Session,
    /// R173: the Matrix menu (Apple-menu extract).
    Matrix,
}

/// R106: glass is on unless switched off — and never fights the
/// accessibility contract (reduced transparency = opaque surfaces).
fn glass_wanted() -> bool {
    mkw::glass::wanted()
}

/// Logical size of the first output (for backdrop mapping).
fn screen_size() -> (f32, f32) {
    let fallback = (1920.0, 1080.0);
    let Some(out) = mk::canvas::raw(&["msg", "-j", "outputs"]) else { return fallback };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else { return fallback };
    let outputs = match &v {
        serde_json::Value::Array(a) => a.clone(),
        serde_json::Value::Object(m) => m.values().cloned().collect(),
        _ => return fallback,
    };
    for o in outputs {
        if let Some(mode) = o.get("logical") {
            let w = mode.get("width").and_then(|x| x.as_f64());
            let h = mode.get("height").and_then(|x| x.as_f64());
            if let (Some(w), Some(h)) = (w, h) {
                return (w as f32, h as f32);
            }
        }
    }
    fallback
}

fn widgets_read() -> Vec<Widget> {
    mk::settings::read("dock-widgets")
        // R187: a fresh account gets Hardline in the shelf from the
        // start — not switched ON, but VISIBLE, so the state is never
        // a thing you have to go looking for.
        .unwrap_or_else(|| String::from("apps hardline"))
        .split_whitespace()
        .filter_map(Widget::out)
        // R209: a duo device carries no Hardline (owner, 01.11.2026).
        .filter(|w| mk::device::carries_hardline() || *w != Widget::Hardline)
        .collect()
}

// ------------------------------------------------ Clipboard history

/// History file (JSON array, newest first) — volatile in the runtime
/// directory; matrix-context reads it for the history menu.
fn zw_path() -> std::path::PathBuf {
    let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
    {
        // R107: matrix-clipboard.json is the contract; the German
        // legacy name is read if the new one is missing.
        let dir = std::path::PathBuf::from(base);
        let modern = dir.join("matrix-clipboard.json");
        if modern.exists() { modern } else {
            let legacy = dir.join("matrix-zwischenablage.json");
            if legacy.exists() { legacy } else { modern }
        }
    }
}

fn zw_read() -> Vec<String> {
    std::fs::read_to_string(zw_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Fold the current copy into the history (deduplicated, max 8,
/// giant copies stay out). Returns the newest entry.
fn zw_maintain() -> Option<String> {
    let out = std::process::Command::new("wl-paste")
        .args(["--no-newline", "--type", "text"])
        .output()
        .ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let mut history = zw_read();
    if out.status.success() && !text.trim().is_empty() && text.chars().count() <= 10_000 {
        history.retain(|e| e != &text);
        history.insert(0, text);
        history.truncate(8);
        if let Ok(json) = serde_json::to_string(&history) {
            let _ = std::fs::write(zw_path(), json);
        }
    }
    history.into_iter().next()
}

// ----------------------------------------------------------------- App

/// R209: the Apps tile on a duo — its two lines and its flash key.
const APPS_NAME: &str = "Apps";
const APPS_MAKER: &str = "Alle Programme";
const APPS_FLASH: &str = "\u{0}apps";


/// What a surface of the fleet shows (R174). The spine is not in
/// here — it is the app's own window, not a fleet pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    /// Index into `self.tiles`.
    Tile(usize),
    /// A shelf widget, plus which of the two rows it sits in.
    Widget(Widget, bool),
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Msg {
    WindowClosed(iced::window::Id),
    /// R174b: the pointer entered (true) or left (false) a shelf chip.
    Hover(iced::window::Id, bool),
    /// One frame of the glass magnification.
    GrowTick,
    FadeTick,
    Tick,
    OsdPulse,
    /// Redraw only — drives the launch bounce (60 ms, only while needed).
    BounceTick,
    Click(usize),
    Context(usize),
    /// R173: grow open one of the shelf menus (user / Matrix).
    SheetOpen(Sheet),
    MenuClose,
    /// R173: a session action — final ones ask for a second click.
    Act(Action),
    /// R173: start an app from the Matrix menu ("name area" like the bar).
    AppStart(&'static str),
    /// R135 event family: a listener fired — re-read that state.
    PowerChanged,
    NetChanged,
    /// The minute tick: the documented slow fallback for states whose
    /// monitor tool is missing (`events::watch` stays silent then).
    Minute,
    AppsOpen,
    CentralToggle,
    /// R69: open the capture panel above the dock.
    CaptureOpen,
    HardlineToggle,
    /// Open the clipboard history (external menu, the dock never grows).
    ClipboardMenu,
    /// R62: wheel-Y over a dock row → cross scroll (only that row).
    RowWheel(&'static str, f32),
    /// R85: touch swipe (Surface). Finger X pressed/moved/lifted —
    /// horizontal dragging pushes the rows sideways. Additive to the
    /// buttons: a tap stays a tap, only movement scrolls.
    Swipe(f32),
    SwipeStart(f32),
    /// R168c: raw pointer/finger with the full point — feeds both the
    /// row swipe (x) and the auto-hide edge logic (y).
    SwipeStartAt(iced::Point),
    SwipeAt(iced::Point),
    Poke,
    WakeTick,
    SwipeEnd,
    /// R210 cinema mode: a player started or paused (`mk::cinema`).
    Cinema,
    /// R210: the subtitle line changed (`mk::cinema::subtitle`).
    Subtitle,
}

struct App {
    palette: mk::Palette,
    watcher: mk::PaletteWatcher,
    /// R168c auto-hide: the dock folds to a strip; a swipe up from the
    /// bottom edge (or the mouse touching it) brings it back.
    autohide: bool,
    hidden: bool,
    reserve: bool,
    last_poke: std::time::Instant,
    edge_from: Option<f32>,
    /// R168e: the wake slide — 0 = below the edge, 1 = home.
    wake: mk::motion::Spring,
    /// R169b: throttles the home-open probe inside the fast pulse.
    pulse_count: u32,
    tiles: Vec<Tile>,
    /// R172: tile face instead of icons (contract `dock-style`).
    tiled: bool,
    /// app_id -> human name, from the ONE launcher source.
    labels: std::collections::HashMap<String, String>,
    widgets: Vec<Widget>,
    clock: String,
    /// Newest copy for the widget preview (row 2).
    zw_preview: Option<String>,
    /// R187: does the Hardline node run at all?
    hardline_on: bool,
    /// Live-rendered Matrix icons per app_id; None = foreign app (initial).
    icons: HashMap<String, Option<image::Handle>>,
    /// The Dynamic Dock face: last OSD state + key press time.
    osd: Option<(mk::osd::State, std::time::SystemTime)>,
    hard: bool,
    /// R106 liquid glass: wallpaper backdrop for the refraction shader.
    glass: bool,
    backdrop: Option<std::sync::Arc<(Vec<u8>, u32, u32)>>,
    backdrop_stamp: u64,
    /// Logical screen size (from the compositor) for backdrop mapping.
    screen: (f32, f32),
    /// R111 vibrant text: ink pair measured from the wallpaper behind
    /// the dock body (primary, secondary); None = palette tones.
    vibrant: Option<(mk::Rgba, mk::Rgba)>,
    /// launchanim extract (R38): starting pins bounce until their window
    /// arrives (app_id → click time). Reference: the icon bounces on app start.
    bounce: std::collections::HashMap<String, std::time::Instant>,
    /// R209: the answer to a tap (app_id → tap time) — the tile's edge lights
    /// up for a moment even when the app only comes to the front.
    flash: mkw::TapFlash<String>,
    /// The grown-open menu above the shelf (tile context, or since
    /// R173 the user and Matrix menus that came down from the bar).
    menu: Option<Sheet>,
    /// The menu waits for the grown surface (anti-glitch, 2026-07-08):
    /// grow first, show on the next pulse — otherwise the menu layout
    /// squeezes into the old surface for one frame.
    menu_waiting: Option<Sheet>,
    /// R173: final session action waiting for its second (red) click.
    confirmation: Option<Action>,
    /// R173: account name for the user widget and the menu header.
    user: String,
    /// R173 (R135 event family): read once at start, then only when a
    /// listener says so — never on the dock's own tick.
    battery: Option<(u8, bool)>,
    net: Option<String>,
    wlan: Option<bool>,
    bluetooth: Option<bool>,
    /// R85: last finger X position during a touch swipe.
    swipe_x: Option<f32>,
    /// Start-size reassertion (Surface finding 2026-07-19): the mass()
    /// from new() can lose the race against the first layer configuration —
    /// the dock then stuck at bootstrap width and clipped.
    /// The first ticks send the size again (idempotent).
    start_masse: u8,
    /// R174: the spine surface (OSD, menus, auto-hide edge). In the
    /// icon dock it is also the pill itself.
    spine: Option<iced::window::Id>,
    /// R210 cinema mode (owner 02.11.2026): the tiles step aside; with
    /// subtitles on, a glass strip shows them in the dock's place.
    cinema: bool,
    subtitle: Option<(usize, String)>,
    /// The strip's surface and the size it was opened with.
    strip: Option<(iced::window::Id, usize)>,
    /// One surface per tile, flowing from the left edge.
    tile_fleet: mkw::fleet::Fleet<Pane>,
    /// One surface per widget, flowing from the right edge — upper and
    /// lower row keep their own fleet, because a fleet advances along
    /// ONE axis and the two rows differ across it.
    widget_fleet: mkw::fleet::Fleet<Pane>,
    widget_fleet2: mkw::fleet::Fleet<Pane>,
    /// How far the tile row is scrolled, in tiles. With every tile its
    /// own surface there is no scrollable viewport any more — the row
    /// PAGES instead: the plan simply starts at this index (user's
    /// requirement stands: the tiles must never push the widgets off
    /// the screen).
    tile_offset: usize,
    /// R174b: which shelf surface the pointer is over, and how far its
    /// GLASS has grown (0..1). The kit magnifier scales what is DRAWN
    /// inside a surface — but here the glass body IS the surface, so
    /// it only breathes if the surface itself does (user: "the
    /// animation is missing on the glass itself, not on the text").
    hover: Option<iced::window::Id>,
    grow: mk::motion::Spring,
}

/// Height of one shelf element — the tile face plus its air. Both
/// widget rows together get the same height, so the two halves of the
/// shelf stay level (user, R172k).
const ELEMENT_HEIGHT: u32 = 52;
/// Gap between the two widget rows.
const ROW_GAP: u32 = mk::spacing::XS as u32;
/// Height one window-title line adds to a tile (R174j).
pub(crate) const LINE_H: u32 = 18;
/// The shelf floats this far above the screen edge.
const SHELF_LIFT: i32 = 10;

/// The spine's bottom margin: the shelf floats SHELF_LIFT above the
/// edge, but its surface reaches SHADOW_MARGIN further down so the
/// blur fades out instead of breaking hard at the surface bottom
/// (user finding 2026-07-15, the "rectangle wings").
const SPINE_MARGIN_BOTTOM: i32 = SHELF_LIFT - mkw::panel::SHADOW_MARGIN as i32;

/// R192 (user): a window is maximized when it keeps the SAME air to
/// the shelf that it keeps to the left and right screen edges. So the
/// reserved strip has to end exactly at the shelf's upper silhouette
/// and not one pixel lower — the compositor then adds its own `gaps`
/// on top, the same gaps it adds at the sides.
///
/// Measured on the RIGHT half: its two widget rows have a FIXED
/// height, while the left half grows upwards with the window lists of
/// an opened tile (R174j) and would drag the strip along.
///
/// Smithay ADDS the surface's bottom margin to the exclusive zone
/// (`LayerMap::arrange`), so the negative shadow room has to be
/// compensated here — otherwise the shelf reserves 30 px less than it
/// occupies and its tiles cut into the window.
fn reserved_strip() -> i32 {
    let snug = mk::device::shelf().tile_height;
    let shelf = if let (true, Some(h)) = (tiled_now(), snug) {
        // R209 (Duo, the owner): the tiles hug their content, so the reserve
        // does too — the window ends one niri gap above the tiles, the same
        // air as at every other edge.
        h as i32
    } else if tiled_now() {
        // Two widget rows plus their gap — a tile row is the same
        // height by construction (R172k).
        (2 * ELEMENT_HEIGHT + ROW_GAP) as i32
    } else {
        HEIGHT as i32
    };
    SHELF_LIFT + shelf - SPINE_MARGIN_BOTTOM
}


impl App {
    fn new() -> (Self, Task<Msg>) {
        let mut app = App {
            autohide: mk::settings::read("dock-autohide").as_deref() == Some("on"),
                reserve: std::env::args().any(|a| a == "--reserve" || a == "--reservieren"),
            hidden: false,
            last_poke: std::time::Instant::now(),
            edge_from: None,
            wake: mk::motion::Spring::new(1.0),
            pulse_count: 0,
            palette: mk::Palette::load().unwrap_or_default(),
            watcher: mk::PaletteWatcher::new(),
            tiles: Vec::new(),
            tiled: tiled_now(),
            labels: mk::apps::index()
                .into_iter()
                .map(|e| (e.app_id, e.name))
                .collect(),
            widgets: widgets_read(),
            clock: String::new(),
            zw_preview: None,
            hardline_on: hardline_running(),
            icons: HashMap::new(),
            osd: None,
            hard: mk::motion_reduced(),
            // R172d: tiled, the shelf itself must not be glass — each
            // tile carries its own now, and a pane behind them would be
            // a second material for one thing. The compositor only
            // frosts what the app asks for.
            glass: glass_wanted() && !tiled_now(),
            backdrop: mkw::glass::wallpaper_backdrop(),
            backdrop_stamp: 1,
            screen: screen_size(),
            bounce: std::collections::HashMap::new(),
            flash: mkw::TapFlash::default(),
            menu: None,
            menu_waiting: None,
            confirmation: None,
            user: user_read(),
            battery: battery_read(),
            net: None,
            wlan: None,
            bluetooth: None,
            swipe_x: None,
            start_masse: 3,
            vibrant: None,
            spine: None,
            cinema: mk::cinema::active(),
            subtitle: mk::cinema::subtitle::read(),
            strip: None,
            // The tiles start where the bar keeps its side air, so both
            // edges of the screen breathe the same (user, R172c).
            tile_fleet: mkw::fleet::Fleet::across(SHELF_SIDE as i32, GAP as i32),
            widget_fleet: mkw::fleet::Fleet::across(SHELF_SIDE as i32, GAP as i32),
            widget_fleet2: mkw::fleet::Fleet::across(SHELF_SIDE as i32, GAP as i32),
            tile_offset: 0,
            hover: None,
            grow: mk::motion::Spring::smooth(0.0),
        };
        app.measure_vibrant();
        // R135: one reading at birth, then the listeners carry it.
        app.radio_read();
        let spine = iced::window::Id::unique();
        app.spine = Some(spine);
        // The player learns from this how much a taller subtitle strip
        // takes from the picture (`mk::cinema::subtitle::picture_shrink`).
        mk::cinema::subtitle::publish_dock_zone(reserved_strip() + SPINE_MARGIN_BOTTOM);
        let reserve = app.reserve;
        let task = Task::batch([
            Task::done(Msg::NewLayerShell {
                settings: spine_settings(reserve),
                id: spine,
            }),
            app.refresh(),
        ]);
        (app, task)
    }

    /// R174: lay the shelf out as surfaces. Each tile and each widget
    /// becomes its own layer surface with the `matrix-tile` namespace —
    /// that is the ONLY way the compositor can put real glass under an
    /// individual element (a rule matches a surface, never a widget
    /// inside one).
    fn sync_shelf(&mut self) -> Task<Msg> {
        if !self.tiled {
            return Task::none();
        }
        // Cinema mode: the tiles step aside (all fleets empty).
        if self.cinema {
            let close = |id| Task::done(Msg::RemoveWindow(id));
            let none = |_: mkw::fleet::Shell, _: iced::window::Id| Task::none();
            return Task::batch([
                self.tile_fleet.sync(Vec::new(), none, close),
                self.widget_fleet.sync(Vec::new(), none, close),
                self.widget_fleet2.sync(Vec::new(), none, close),
            ]);
        }
        let bottom = SHELF_LIFT;
        // The widgets claim the right end; the tiles get what is left.
        let widgets: Vec<Widget> = self
            .widgets
            .iter()
            .copied()
            .filter(|w| self.widget_live(*w))
            .collect();
        // R209 (Duo, the owner): «Apps» leads the shelf — first on the left,
        // before the pinned apps — instead of standing alone at the right.
        let lead_apps = mk::device::shelf().apps_leads && widgets.contains(&Widget::Apps);
        let widgets: Vec<Widget> = widgets.into_iter().filter(|w| !(lead_apps && *w == Widget::Apps)).collect();
        let half = widgets.len().div_ceil(2);
        let (upper, lower) = widgets.split_at(half);

        let plan_widgets = |list: &[Widget], second: bool| -> Vec<mkw::fleet::Spec<Pane>> {
            list.iter()
                .rev() // right-anchored: the fleet advances leftwards
                .map(|w| {
                    let width = self.widget_width(*w);
                    mkw::fleet::Spec {
                        pane: Pane::Widget(*w, second),
                        width,
                        height: ELEMENT_HEIGHT,
                        cross: bottom
                            + if second {
                                0
                            } else {
                                (ELEMENT_HEIGHT + ROW_GAP) as i32
                            },
                        namespace: "matrix-chip",
                        keyboard: false,
                        sig: (if second { 3 } else { 2 }, width),
                    }
                })
                .collect()
        };

        // Two widget rows stacked = the height of one tile, so both
        // halves of the shelf are level (user, R172k).
        // R209 (Duo, the owner): the glass hugs the tile — no shelf widgets to
        // level with, so the surface is exactly the tile (padding S, title,
        // maker); the lit edge sits on the glass edge.
        let tile_height = mk::device::shelf().tile_height.unwrap_or(2 * ELEMENT_HEIGHT + ROW_GAP);
        let widget_room: u32 = [upper, lower]
            .iter()
            .map(|list| {
                list.iter()
                    .map(|w| self.widget_width(*w) + GAP as u32)
                    .sum::<u32>()
            })
            .max()
            .unwrap_or(0);
        let room_for_tiles = (self.screen.0 as u32)
            .saturating_sub(widget_room + 2 * SHELF_SIDE as u32 + mk::spacing::L as u32);

        // Paging instead of scrolling (R174 finding): separate surfaces
        // cannot be clipped by a shared viewport, so the row shows the
        // tiles that FIT starting at `tile_offset`. The user's rule
        // stands either way — tiles never push the widgets away.
        let mut used = 0u32;
        let mut tiles: Vec<mkw::fleet::Spec<Pane>> = Vec::new();
        if lead_apps {
            let width = mkw::tile_auto_width(APPS_NAME, APPS_MAKER).ceil() as u32;
            used += width + GAP as u32;
            tiles.push(mkw::fleet::Spec {
                pane: Pane::Widget(Widget::Apps, false),
                width,
                height: tile_height,
                cross: bottom,
                namespace: "matrix-chip",
                keyboard: false,
                sig: (2, width + tile_height * 8192),
            });
        }
        for (i, tile) in self.tiles.iter().enumerate().skip(self.tile_offset) {
            // R174j: width follows the widest LINE too, and the tile
            // grows upward by one caption row per open window.
            let lines = self.tile_lines(tile);
            let mut wide = mkw::tile_auto_width(&self.label_of(&tile.app_id), maker_of(&tile.app_id));
            for line in &lines {
                wide = wide.max(
                    mkw::text_advance(line, mk::typo::CAPTION.size)
                        + 2.0 * mk::spacing::M
                        + mkw::TEXT_SAFETY,
                );
            }
            let width = wide.ceil() as u32;
            let height = tile_height + lines.len() as u32 * LINE_H;
            if used + width > room_for_tiles && !tiles.is_empty() {
                break;
            }
            used += width + GAP as u32;
            tiles.push(mkw::fleet::Spec {
                pane: Pane::Tile(i),
                width,
                height,
                cross: bottom,
                namespace: "matrix-chip",
                keyboard: false,
                // Width AND height, so a new window reopens the pane.
                sig: (1, width + height * 8192),
            });
        }

        use matrixkit_widgets::iced_layershell::reexport::Anchor;
        let open = |anchor_right: bool| {
            move |shell: mkw::fleet::Shell, id: iced::window::Id| {
                let anchor = if anchor_right { Anchor::Bottom | Anchor::Right } else { Anchor::Bottom | Anchor::Left };
                Task::done(Msg::NewLayerShell { settings: shell.layer_settings(anchor, dock_output()), id })
            }
        };
        let close = |id| Task::done(Msg::RemoveWindow(id));
        // Plan first, sync second: the planners read `self`, the fleets
        // need it mutably — so nothing may still be borrowed here.
        let upper_plan = plan_widgets(upper, false);
        let lower_plan = plan_widgets(lower, true);
        Task::batch([
            self.tile_fleet.sync(tiles, open(false), close),
            self.widget_fleet.sync(upper_plan, open(true), close),
            self.widget_fleet2.sync(lower_plan, open(true), close),
        ])
    }

    /// Does this widget have anything to show right now? A machine
    /// without a battery must not leave a hole in the row.
    fn widget_live(&self, w: Widget) -> bool {
        let p = self.palette;
        self.widget_view(w, p.on_surface, p.on_surface_variant)
            .is_some()
    }

    /// How wide this widget wants its own surface (R174). Same reason
    /// as `mkw::tile_auto_width`: a layer surface states its size
    /// before anything is laid out, so the text is measured, not asked.
    /// How wide this widget wants its own surface (R174). Same reason
    /// as `mkw::tile_auto_width`: a layer surface states its size
    /// before anything is laid out, so the content is measured, not
    /// asked. Every shelf widget is a glyph, a label, or both — and
    /// the ones that carry BOTH were the ones that came out clipped
    /// on the Surface ("Ap|", "Ma").
    fn widget_width(&self, w: Widget) -> u32 {
        let (glyph, label): (bool, Option<String>) = match w {
            Widget::Apps => (true, Some(String::from("Apps"))),
            Widget::Matrix => (true, Some(String::from("Matrix"))),
            Widget::User => (true, Some(self.user.clone())),
            Widget::Battery => (true, self.battery.map(|(percent, _)| format!("{percent} %"))),
            Widget::Wlan => (
                true,
                match (self.wlan, &self.net) {
                    (Some(true), Some(name)) => Some(name.clone()),
                    (Some(true), None) => Some(String::from("WLAN")),
                    _ => None,
                },
            ),
            Widget::Clipboard => (true, self.zw_preview.clone()),
            Widget::Clock => (false, Some(self.clock.clone())),
            Widget::Central | Widget::Capture | Widget::Bluetooth => (true, None),
            Widget::Hardline => (true, Some(String::from(if self.hardline_on { "An" } else { "Aus" }))),
        };
        // Chrome air + the button's own padding, always.
        let mut width = 2.0 * mk::spacing::M + 2.0 * mk::spacing::S + mkw::TEXT_SAFETY;
        if glyph {
            width += mk::font_size::LARGE;
        }
        if let Some(label) = label {
            // Long labels (clipboard copy, network name) are elided by
            // the widget itself — measuring the raw string would blow
            // the surface across half the shelf.
            let label: String = label.chars().take(18).collect();
            if glyph {
                width += mk::spacing::S;
            }
            width += mkw::text_advance(&label, mk::typo::CALLOUT.size);
        }
        width.ceil() as u32
    }

    /// R174j (user): the caption lines a tile shows under its origin —
    /// one per open window, arrow first, so a full shelf reads like a
    /// window list (Dateien: one line per folder). Empty titles fall
    /// back to the running dot. ONE source for view and geometry.
    fn tile_lines(&self, k: &Tile) -> Vec<String> {
        // R209 (Duo, the owner): tiles do not grow — no window lines on a duo.
        if !mk::device::shelf().window_lines {
            return Vec::new();
        }
        tile_lines_of(k, &self.label_of(&k.app_id))
    }

    /// Which fleet pane lives in this surface?
    fn pane_of(&self, window: iced::window::Id) -> Option<Pane> {
        self.tile_fleet
            .pane(window)
            .or_else(|| self.widget_fleet.pane(window))
            .or_else(|| self.widget_fleet2.pane(window))
            .copied()
    }

    /// R111: measure the wallpaper luminance behind the dock body and
    /// derive the vibrant ink pair. Representative rect: the center
    /// half of the bottom strip — stable across tile-count changes.
    fn measure_vibrant(&mut self) {
        let Some(backdrop) = self.backdrop.as_deref() else {
            self.vibrant = None;
            return;
        };
        let (sw, sh) = self.screen;
        let rect = iced::Rectangle {
            x: sw * 0.25,
            y: sh - 10.0 - HEIGHT as f32,
            width: sw * 0.5,
            height: HEIGHT as f32,
        };
        let luma = mkw::glass::luma_of(backdrop, self.screen, rect);
        self.vibrant = Some(mkw::glass::vibrant_ink(luma));
    }

    /// Refresh tiles + palette; with a new column count the pill grows.
    fn refresh(&mut self) -> Task<Msg> {
        if self.watcher.changed() {
            if let Some(new) = mk::Palette::load() {
                self.palette = new;
            }
            // Re-bake icons only at the FADE END (edge), never per frame.
            if !mk::transition::active() {
                self.icons.clear();
            }
        }
        let before = self.tiles.len();
        self.tiles = tiles_read();
        {
            let tiles = &self.tiles;
            self.bounce.retain(|id, since| {
                since.elapsed().as_secs() < 8
                    && !tiles.iter().any(|k| &k.app_id == id && k.running())
            });
        }
        self.hard = mk::motion_reduced();
        self.widgets = widgets_read();
        if self.widgets.contains(&Widget::Clock) {
            if let Ok(out) = std::process::Command::new("date").arg("+%H:%M").output() {
                self.clock = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }
        // Watch the clipboard only while the widget is active —
        // no wl-paste call per tick otherwise.
        if self.widgets.contains(&Widget::Clipboard) {
            self.zw_preview = zw_maintain();
        }
        // R174f: the tile shelf shows WORDS, not icons — baking a PNG
        // per app (and re-baking the lot on every palette change) is
        // pure waste there. Measured hunt for the CPU mountain between
        // second 10 and 40 after start.
        if !self.tiled {
            for k in &self.tiles {
                let p = self.palette;
                self.icons.entry(k.app_id.clone()).or_insert_with(|| {
                    matrixkit_icons::render_png(&k.app_id, &p).map(image::Handle::from_bytes)
                });
            }
        }
        // R174h (user): both dock faces stay available and the choice
        // is live. The two faces differ down to the LAYER SURFACES
        // (one shelf-wide strip versus a chip per element), which no
        // running window can be talked into becoming — so the dock
        // starts itself afresh. Cheaper than it sounds: a restart is
        // the same work the session start does anyway.
        if tiled_now() != self.tiled {
            use std::os::unix::process::CommandExt as _;
            // EXEC, not spawn-and-exit: the dock runs inside a systemd
            // unit, and systemd tears the whole cgroup down the moment
            // the main process leaves — a spawned twin dies with it and
            // the user is left WITHOUT a dock (measured on the Surface).
            // Exec keeps the same process, so the unit never notices.
            if let Ok(exe) = matrixkit_theme::process::own_exe_result() {
                let args: Vec<String> = std::env::args().skip(1).collect();
                let refused = std::process::Command::new(exe).args(&args).exec();
                // exec only returns on failure — then the old face
                // simply stays, which is better than no dock at all.
                let _ = refused;
            }
        }
        // R174: the shelf is a fleet of surfaces now — it follows
        // every change of the tile list, the widget list or their
        // labels, exactly like the control centre's islands do.
        let shelf = self.sync_shelf();
        if self.tiles.len() != before {
            return Task::batch([self.mass(), shelf]);
        }
        shelf
    }

    /// R173/R135: re-read the radio states. Called at start and when a
    /// listener fires — never on the dock's own tick, and only while a
    /// widget actually shows them (nmcli/bluetoothctl are processes).
    fn radio_read(&mut self) {
        if !self.widgets.iter().any(|w| w.needs_network()) {
            return;
        }
        self.wlan = wlan_read();
        self.bluetooth = bt_read();
        self.net = net_read();
    }

    /// R173: how much room the open menu needs above the shelf. The
    /// shelf menus are MEASURED rather than given a fixed box, because
    /// the session menu is far taller than the Matrix menu.
    fn menu_room(&self) -> u32 {
        let head = |entries: Vec<mkw::ui::MenuItem<Msg>>, header: bool| {
            // menu_size measures the entry groups only — the header
            // rides in its own island (padding + gap), so it is added.
            let h = mkw::ui::menu_size(&entries).1 + if header { HEADER_ROOM } else { 0.0 };
            h as u32 + mk::spacing::M as u32
        };
        match self.menu.or(self.menu_waiting) {
            None => 0,
            Some(Sheet::Session) => head(self.session_entries(), true),
            Some(Sheet::Matrix) => head(self.matrix_entries(), false),
        }
    }


    /// R169b: an open matrix-home keeps the dock awake — the reference shows it on the home screen.
    fn home_open() -> bool {
        let Ok(dir) = std::fs::read_dir("/proc") else { return false };
        for e in dir.flatten() {
            if let Ok(comm) = std::fs::read_to_string(e.path().join("comm")) {
                if comm.trim() == "matrix-home" {
                    return true;
                }
            }
        }
        false
    }
    /// R168e: waking is a slide, not a pop — the dock floats up from
    /// the edge on a spring (snaps under reduced motion).
    fn wake_up(&mut self) -> Task<Msg> {
        self.hidden = false;
        if mk::motion_reduced() {
            self.wake = mk::motion::Spring::snappy(1.0);
        } else {
            // The SAME entrance spring the windows and panels ride —
            // the wake feels like the control center opening.
            self.wake = mk::motion::Spring::snappy(0.0);
            self.wake.retarget(1.0);
        }
        self.mass()
    }

    /// Surface size: base height + menu room + shadow breathing room
    /// (top and sides; the bottom edge sits at the screen edge).
    /// R168c: hidden, the dock folds to a thin catch strip — the swipe
    /// stage of the reference's auto-hide.
    fn mass(&self) -> Task<Msg> {
        let edge = mkw::panel::SHADOW_MARGIN as u32;
        // R174: in the daemon pattern every layer message names its
        // surface — all of these belong to the spine.
        let Some(id) = self.spine else {
            return Task::none();
        };
        if self.hidden {
            return Task::batch([
                Task::done(Msg::SizeChange {
                    id,
                    size: (
                        if self.tiled { 0 } else { width(self.tiles.len()) + 2 * edge },
                        STRIP,
                    ),
                }),
                Task::done(Msg::MarginChange { id, margin: (0, 0, 0, 0) }),
                Task::done(Msg::ExclusiveZoneChange { id, zone_size: 0 }),
            ]);
        }
        let height = if self.tiled { SHELF_HEIGHT } else { HEIGHT } + self.menu_room();
        Task::batch([
            Task::done(Msg::SizeChange {
                id,
                // R172: tiled, the shelf spans the whole edge — width 0
                // means "as wide as the output" in layer-shell, the same
                // trick the bar uses up top.
                size: (
                    if self.tiled { 0 } else { width(self.tiles.len()) + 2 * edge },
                    height + 2 * edge,
                ),
            }),
            Task::done(Msg::MarginChange {
                id,
                margin: (0, 0, SPINE_MARGIN_BOTTOM, 0),
            }),
            Task::done(Msg::ExclusiveZoneChange {
                id,
                // With the subtitle strip the strip reserves its own room.
                zone_size: if self.reserve && !self.autohide && !self.strip_wanted() { reserved_strip() } else { 0 },
            }),
        ])
    }

    /// Cinema mode with subtitles on: the strip stands in the dock's place.
    fn strip_wanted(&self) -> bool {
        self.cinema && self.subtitle.is_some()
    }

    /// Open, resize or close the subtitle strip (R210).
    fn sync_strip(&mut self) -> Task<Msg> {
        let size = self.subtitle.as_ref().map(|(s, _)| *s).unwrap_or(1);
        let height = mk::cinema::subtitle::strip_height(size);
        let margin = mk::cinema::subtitle::MARGIN;
        match (self.strip_wanted(), self.strip) {
            (true, None) => {
                let id = iced::window::Id::unique();
                self.strip = Some((id, size));
                Task::done(Msg::NewLayerShell { settings: mkw::glass_strip::settings(dock_output(), height, margin), id })
            }
            (true, Some((id, old))) if old != size => {
                self.strip = Some((id, size));
                Task::batch([
                    Task::done(Msg::SizeChange { id, size: (0, height) }),
                    Task::done(Msg::ExclusiveZoneChange { id, zone_size: height as i32 }),
                ])
            }
            (false, Some((id, _))) => {
                self.strip = None;
                Task::done(Msg::RemoveWindow(id))
            }
            _ => Task::none(),
        }
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Cinema => {
                let now = mk::cinema::active();
                if now == self.cinema {
                    return Task::none();
                }
                self.cinema = now;
                self.subtitle = mk::cinema::subtitle::read();
                return Task::batch([self.sync_shelf(), self.sync_strip(), self.mass()]);
            }
            Msg::Subtitle => {
                let was = self.strip_wanted();
                self.subtitle = mk::cinema::subtitle::read();
                let mut tasks = vec![self.sync_strip()];
                if was != self.strip_wanted() {
                    tasks.push(self.mass());
                }
                return Task::batch(tasks);
            }
            Msg::Hover(id, inside) if inside => {
                if self.hover == Some(id) {
                    return Task::none();
                }
                // Pointer jumped straight onto the next chip: the old
                // one snaps back at once, otherwise two stand grown.
                let left = self.hover.replace(id);
                self.grow = mk::motion::Spring::smooth(0.0);
                self.grow.retarget(1.0);
                if self.hard {
                    self.grow.value = 1.0;
                }
                let mut tasks = vec![self.shell_geometry(id, self.grow.value)];
                if let Some(left) = left {
                    tasks.push(self.shell_geometry(left, 0.0));
                }
                return Task::batch(tasks);
            }
            Msg::Hover(id, _) => {
                if self.hover != Some(id) {
                    return Task::none();
                }
                if self.hard {
                    self.hover = None;
                    self.grow.value = 0.0;
                    return self.shell_geometry(id, 0.0);
                }
                self.grow.retarget(0.0);
                return Task::none();
            }
            Msg::GrowTick => {
                let Some(id) = self.hover else {
                    return Task::none();
                };
                let moving = self.grow.tick(1.0 / 60.0);
                let task = self.shell_geometry(id, self.grow.value);
                // Settled at rest means the pointer is gone for good.
                if !moving && self.grow.target == 0.0 {
                    self.hover = None;
                }
                return task;
            }
            Msg::FadeTick => {
                if self.watcher.changed() {
                    if let Some(new) = mk::Palette::load() {
                        self.palette = new;
                    }
                    if !mk::transition::active() {
                        self.icons.clear();
                        // R106: the wallpaper likely changed with the palette.
                        if let Some(fresh) = mkw::glass::wallpaper_backdrop() {
                            self.backdrop = Some(fresh);
                            self.backdrop_stamp += 1;
                        }
                        self.glass = glass_wanted();
                        self.measure_vibrant();
                    }
                }
                return Task::none();
            }
            Msg::BounceTick => {
                self.flash.tick();
                Task::none()
            }
            Msg::Tick => {
                let wants = mk::settings::read("dock-autohide").as_deref() == Some("on");
                if wants != self.autohide {
                    self.autohide = wants;
                    self.hidden = false;
                    self.last_poke = std::time::Instant::now();
                    return Task::batch([self.mass(), self.refresh()]);
                }
                let t = self.refresh();
                if self.start_masse > 0 {
                    self.start_masse -= 1;
                    return Task::batch([t, self.mass()]);
                }
                t
            }
            Msg::OsdPulse => {
                // R169b: an open home screen wakes and holds the dock.
                self.pulse_count = self.pulse_count.wrapping_add(1);
                if self.hidden && self.pulse_count % 10 == 0 && Self::home_open() {
                    return self.wake_up();
                }
                // R168c: fold away after three still seconds (no menu
                // open, nothing touched) — the reference's auto-hide.
                if self.autohide
                    && !self.hidden
                    && self.menu.is_none()
                    && self.menu_waiting.is_none()
                    && self.last_poke.elapsed().as_secs_f32() > 3.0
                    && !Self::home_open()
                {
                    self.hidden = true;
                    // Fresh empty frames past the resize — no ghost
                    // stripe of the last full dock.
                    self.wake = mk::motion::Spring::snappy(1.0);
                    self.wake.retarget(0.0);
                    return self.mass();
                }
                if let Some(i) = self.menu_waiting.take() {
                    self.menu = Some(i);
                }
                let war = self.osd_active();
                self.osd = mk::osd::read();
                let _ = war; // the next view renders the state change
                Task::none()
            }
            Msg::Context(i) => {
                // External menu (matrix-context dock ...): the dock never
                // grows — the resize glitch is structurally impossible.
                mk::feel::hold();
                if let Some(k) = self.tiles.get(i) {
                    let width_total = width(self.tiles.len()) as f32
                        + 2.0 * mkw::panel::SHADOW_MARGIN;
                    let tile_x = mkw::panel::SHADOW_MARGIN
                        + MARGIN
                        + i as f32 * (BUTTON + GAP)
                        + BUTTON / 2.0;
                    let mut offset_x = tile_x - width_total / 2.0;
                    // R103 spacing calibration: 10 (dock gap) + 8 air — the same
                    // breath as between the bar pill and its popups.
                    let mut bottom = (HEIGHT + 18) as f32;
                    // R209: in the tile shelf every tile is its own surface at
                    // the left edge — the menu belongs over THAT surface, not
                    // over the old centred icon row (it appeared shifted).
                    if let Some(shell) = self
                        .tile_fleet
                        .ids()
                        .into_iter()
                        .find(|id| self.tile_fleet.pane(*id) == Some(&Pane::Tile(i)))
                        .and_then(|id| self.tile_fleet.shell_of(id))
                    {
                        let centre = shell.main as f32 + shell.width as f32 / 2.0;
                        offset_x = centre - self.screen.0 as f32 / 2.0;
                        bottom = (shell.cross + shell.height as i32) as f32 + 8.0;
                    }
                    let ids = if k.window.is_empty() {
                        String::from("-")
                    } else {
                        k.window
                            .iter()
                            .map(|f| f.id.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    };
                    let arg = format!(
                        "PATH=$HOME/.local/bin:$PATH exec matrix-context dock {} {} {} {} {}",
                        offset_x as i32,
                        bottom as i32,
                        k.app_id,
                        u8::from(k.pinned),
                        ids
                    );
                    let _ = std::process::Command::new("sh").args(["-c", &arg]).spawn().map(
                        |mut child| {
                            std::thread::spawn(move || {
                                let _ = child.wait();
                            })
                        },
                    );
                }
                return Task::none();
            }
            Msg::SheetOpen(s) => {
                // Only one popup in the session speaks at a time.
                mkw::panel::close_other_popups("");
                self.confirmation = None;
                // Grow FIRST, show on the next pulse — the same
                // anti-glitch the tile menu uses.
                self.menu = None;
                self.menu_waiting = Some(s);
                return self.mass();
            }
            Msg::MenuClose => {
                self.menu = None;
                self.menu_waiting = None;
                self.confirmation = None;
                return self.mass();
            }
            Msg::Act(a) => {
                if a.permanent() && self.confirmation != Some(a) {
                    // Only the red second click counts.
                    self.confirmation = Some(a);
                    return Task::none();
                }
                a.execute();
                self.menu = None;
                self.menu_waiting = None;
                self.confirmation = None;
                return self.mass();
            }
            Msg::AppStart(id) => {
                // "name area" → write the wish file, then start (R41).
                let mut parts = id.splitn(2, ' ');
                let name = parts.next().unwrap_or(id);
                if let Some(area) = parts.next() {
                    let base =
                        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
                    let _ = std::fs::write(format!("{base}/{name}-area"), area);
                }
                mkw::panel::app_start(name);
                self.menu = None;
                self.menu_waiting = None;
                self.confirmation = None;
                return self.mass();
            }
            Msg::PowerChanged => {
                self.battery = battery_read();
                return Task::none();
            }
            Msg::NetChanged => {
                self.radio_read();
                return Task::none();
            }
            Msg::Minute => {
                // Fallback only: a device without udev/nmcli monitors
                // would otherwise freeze at its birth reading.
                self.battery = battery_read();
                self.radio_read();
                return self.update(Msg::Tick);
            }
            Msg::Click(i) => {
                if let Some(k) = self.tiles.get(i) {
                    self.flash.tap(k.app_id.clone());
                    if mk::device::shelf().tap_shelves && k.focus {
                        // R209 (Duo): the app in front, tapped again, goes on
                        // the shelf — the duo's window has no shelf light.
                        mk::window::shelf();
                    } else if let Some(id) = k.target() {
                        mkw::canvas::focus_window(id);
                    } else if k.pinned {
                        // Resting pin: wake the app to life — and bounce until
                        // the window arrives (launchanim, R38).
                        mkw::panel::app_start(&k.app_id);
                        self.bounce
                            .insert(k.app_id.clone(), std::time::Instant::now());
                    }
                }
                self.refresh()
            }
            Msg::ClipboardMenu => {
                // External menu like the tile context: the dock never grows.
                // Centered above the dock (offset 0), bottom edge above the pill.
                // R103 spacing calibration: same 8 px air as everywhere.
                let bottom = (HEIGHT + 18) as i32;
                let arg = format!(
                    "PATH=$HOME/.local/bin:$PATH exec matrix-context clips 0 {bottom}"
                );
                let _ = std::process::Command::new("sh").args(["-c", &arg]).spawn().map(
                    |mut child| {
                        std::thread::spawn(move || {
                            let _ = child.wait();
                        })
                    },
                );
                Task::none()
            }
            Msg::AppsOpen if mk::device::shelf().apps_leads && !self.flash.contains(&String::from(APPS_FLASH)) => {
                // R209: the Apps tile answers like every app tile (tick + lit edge).
                self.flash.tap(String::from(APPS_FLASH));
                self.update(Msg::AppsOpen)
            }
            Msg::AppsOpen => {
                // Matrix Start toggles itself (mkw::leiste_toggle).
                mkw::panel::app_start("matrix-start");
                Task::none()
            }
            Msg::CaptureOpen => {
                mkw::panel::app_start("matrix-capture");
                Task::none()
            }
            // R187 (the owner): "Completely off really means completely
            // off. No back door, nothing!" — so OFF ends the process.
            // Not muted, not idle: gone. ON starts it again; the
            // folder in Matrix Dateien is NOT touched either way
            // (that is a separate decision, in the settings).
            Msg::HardlineToggle => {
                if self.hardline_on {
                    hardline_stop();
                } else {
                    // One string with a space is a program NAME, not a
                    // command line — Command::new finds nothing and the
                    // toggle turns ON silently into thin air (R201 bug:
                    // OFF worked, ON never did).
                    mkw::panel::app_start_with("hardline-node", &["serve"]);
                }
                self.hardline_on = !self.hardline_on;
                Task::none()
            }
            Msg::CentralToggle => {
                mkw::panel::app_start("matrix-central");
                Task::none()
            }
            Msg::RowWheel(id, y) => {
                return iced::advanced::widget::operate(
                    iced::advanced::widget::operation::scrollable::scroll_by(
                        iced::advanced::widget::Id::new(id),
                        iced::advanced::widget::operation::scrollable::AbsoluteOffset {
                            x: -y,
                            y: 0.0,
                        },
                    ),
                );
            }
            Msg::SwipeStartAt(at) => {
                self.last_poke = std::time::Instant::now();
                if self.hidden {
                    self.edge_from = Some(at.y);
                    return Task::none();
                }
                self.update(Msg::SwipeStart(at.x))
            }
            Msg::SwipeAt(at) => {
                self.last_poke = std::time::Instant::now();
                if self.hidden {
                    // Swipe UP from the bottom edge wakes the dock.
                    if let Some(from) = self.edge_from {
                        if from - at.y > 4.0 {
                            self.edge_from = None;
                            return self.wake_up();
                        }
                    }
                    return Task::none();
                }
                self.update(Msg::Swipe(at.x))
            }
            Msg::Poke => {
                self.last_poke = std::time::Instant::now();
                if self.hidden {
                    // The mouse touching the edge wakes the dock
                    // (the reference's pointer grammar).
                    return self.wake_up();
                }
                Task::none()
            }
            Msg::WakeTick => {
                self.wake.tick(1.0 / 60.0);
                Task::none()
            }
            Msg::SwipeStart(x) => {
                self.swipe_x = Some(x);
                Task::none()
            }
            Msg::SwipeEnd => {
                if self.hidden && self.edge_from.is_some() {
                    // A tap on the invisible strip wakes the dock too.
                    self.edge_from = None;
                    return self.wake_up();
                }
                self.swipe_x = None;
                Task::none()
            }
            Msg::Swipe(x) => {
                let Some(before) = self.swipe_x else { return Task::none() };
                let dx = x - before;
                self.swipe_x = Some(x);
                if dx.abs() < 0.5 {
                    return Task::none();
                }
                // Push both rows by the same distance — the non-overflowing one
                // stays put (scroll_by is a no-op there). No guessing which row
                // the finger hit.
                let shift_over = |id: &'static str| {
                    iced::advanced::widget::operate(
                        iced::advanced::widget::operation::scrollable::scroll_by(
                            iced::advanced::widget::Id::new(id),
                            iced::advanced::widget::operation::scrollable::AbsoluteOffset {
                                x: -dx,
                                y: 0.0,
                            },
                        ),
                    )
                };
                return Task::batch([shift_over("dock-zeile-1"), shift_over("dock-zeile-2")]);
            }
            // variants added by the to_layer_message macro (layer control)
            _ => Task::none(),
        }
    }

    /// Does the shelf need a frame every 150 ms right now? Only while
    /// something is genuinely in motion — an OSD counting itself out,
    /// the auto-hide watching the idle clock, a menu growing, or the
    /// dock folded away and waiting for the edge.
    fn pulse_needed(&self) -> bool {
        self.autohide
            || self.hidden
            || self.osd_active()
            || self.menu.is_some()
            || self.menu_waiting.is_some()
    }

    fn subscription(&self) -> Subscription<Msg> {
        let mut all = vec![
            // Instant reaction: the event stream replaces the 1 s polling;
            // a 5 s poll remains as fallback (clock widget, palette).
            mkw::canvas_stream().map(|_| Msg::Tick),
            mkw::tick_to_the_minute().map(|_| Msg::Minute),
            tick("dock", std::time::Duration::from_secs(5)).map(|_| Msg::Tick),
            mkw::palette_fade_subscription().map(|_| Msg::FadeTick),
            // R210 cinema mode and its subtitle strip.
            mkw::events::file_changes(
                "dock-kino",
                mk::cinema::path().to_string_lossy().into_owned(),
                std::time::Duration::from_millis(250),
            )
            .map(|_| Msg::Cinema),
            mkw::events::file_changes(
                "dock-untertitel",
                mk::cinema::subtitle::path().to_string_lossy().into_owned(),
                std::time::Duration::from_millis(100),
            )
            .map(|_| Msg::Subtitle),
            // OSD channel. R174e: the stat() itself was never the cost —
            // the MESSAGE was. Every pulse redraws every window the app
            // owns, and since the shelf split into one surface per
            // element that is seventeen redraws, six times a second
            // (59 % of a CPU, measured on the Surface). So the stat()
            // moves into a listener that speaks up only when the file
            // really changed; the 150 ms heartbeat runs on only while
            // something actually needs a frame.
            if self.pulse_needed() {
                tick("dock-osd", std::time::Duration::from_millis(150)).map(|_| Msg::OsdPulse)
            } else {
                mkw::events::file_changes(
                    "dock-osd-datei",
                    mk::osd::path(),
                    std::time::Duration::from_millis(150),
                )
                .map(|_| Msg::OsdPulse)
            },
            // R85: touch swipe (Surface). Raw finger events — the buttons
            // catch the press before the scroller sees it as a swipe;
            // hence the app listens directly and pushes the rows.
            iced::event::listen_with(|event, _status, _window| match event {
                iced::Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => {
                    Some(Msg::SwipeStartAt(position))
                }
                iced::Event::Touch(iced::touch::Event::FingerMoved { position, .. }) => {
                    Some(Msg::SwipeAt(position))
                }
                iced::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) => Some(Msg::Poke),
                iced::Event::Touch(
                    iced::touch::Event::FingerLifted { .. }
                    | iced::touch::Event::FingerLost { .. },
                ) => Some(Msg::SwipeEnd),
                _ => None,
            }),
        ];
        // R135 event family: the battery and the radios FLOW to the
        // dock — udev speaks the moment the charger moves, nmcli the
        // moment the network does. A listener costs nothing while
        // nothing happens; a poll costs on every tick forever. Both
        // are armed only while a widget shows their state.
        if self.widgets.contains(&Widget::Battery) {
            all.push(
                mkw::events::watch(
                    "dock-power",
                    "udevadm",
                    &["monitor", "--udev", "--subsystem-match=power_supply"],
                    |l| l.contains("power_supply"),
                )
                .map(|_| Msg::PowerChanged),
            );
        }
        if self.widgets.iter().any(|w| w.needs_network()) {
            all.push(
                mkw::events::watch("dock-network", "nmcli", &["monitor"], |_| true)
                    .map(|_| Msg::NetChanged),
            );
        }
        if !self.wake.is_settled() {
            all.push(tick("dock-wake", std::time::Duration::from_millis(16)).map(|_| Msg::WakeTick));
        }
        // Self-test hook, like the bar's: MATRIX_DOCK_TEST_MENU=user
        // (or =matrix) grows the menu open after 2 s. The Surface has
        // no pointer injection, so this is the only way to prove a
        // menu from outside the session (remote diagnosis).
        if self.menu.is_none() {
            // Two arms, not one: iced only accepts NON-capturing map
            // closures, so the choice has to happen before the map.
            match std::env::var("MATRIX_DOCK_TEST_MENU").as_deref() {
                Ok("matrix") => all.push(
                    tick("dock-menuetest", std::time::Duration::from_secs(2))
                        .map(|_| Msg::SheetOpen(Sheet::Matrix)),
                ),
                Ok(_) => all.push(
                    tick("dock-menuetest", std::time::Duration::from_secs(2))
                        .map(|_| Msg::SheetOpen(Sheet::Session)),
                ),
                // family-exception: no test variable set is the normal case — this hook only opens a shelf menu unattended for a screenshot
                Err(_) => {}
            }
        }
        if (!self.bounce.is_empty() || self.flash.is_active()) && !self.hard {
            all.push(
                tick("dock-huepf", std::time::Duration::from_millis(60)).map(|_| Msg::BounceTick),
            );
        }
        // R174b: the glass magnification runs ONLY while it moves —
        // the shelf is silent again the moment the chip has settled.
        if self.hover.is_some() && !self.grow.is_settled() && !self.hard {
            all.push(
                tick("dock-lupe", std::time::Duration::from_millis(16)).map(|_| Msg::GrowTick),
            );
        }
        Subscription::batch(all)
    }

    /// Is the dock currently the OSD?
    fn osd_active(&self) -> bool {
        self.osd
            .map(|(_, since)| {
                since.elapsed().map(|d| d.as_millis() < OSD_DWELL_TIME).unwrap_or(false)
            })
            .unwrap_or(false)
    }

    /// The Dynamic Dock face (user OSD design): the pill stays,
    /// row 1 shows the LEVEL as a bar, row 2 the plain text.
    fn osd_face(&self, state: mk::osd::State) -> Element<'_, Msg> {
        let p = self.palette;
        let (glyph, full) = mkw::osd_display::glyph(state);
        let symbol_color = if full { p.on_surface } else { p.on_surface_variant };
        let portion = state.percent / 100.0;

        // Row 1: symbol + level bar (kit block) at tile height.
        // Uniform inner frame (user, 2026-07-19): the good distance
        // text→bottom edge (RAND + S) applies on ALL four sides.
        let inner = MARGIN + mk::spacing::S;
        let track_width = (width(self.tiles.len().max(3)) as f32
            - 2.0 * inner
            - mk::icon_size::LARGE
            - mk::spacing::M)
            .max(80.0);
        let meter = mkw::osd_display::level_bar(p, portion, full, track_width);
        let level = container(
            row![
                mkw::symbol(glyph, mk::icon_size::LARGE, symbol_color),
                meter,
            ]
            .spacing(mk::spacing::M)
            .align_y(iced::Alignment::Center),
        )
        .center_y(Length::Fixed(BUTTON + DOT + mk::spacing::XXS - mk::spacing::S));

        // Row 2: what changed, in words — with EXACTLY the anatomy
        // of the row-2 buttons (user, 2026-07-19): same padding,
        // and two invisible glyph anchors give the row the same height
        // as the symbol buttons, symmetric = text stays centered.
        let anchor = || mkw::symbol::<Msg>(mkw::symbol::APPS, mk::font_size::LARGE, p.on_surface.with_alpha(0.0));
        let wort = container(
            row![
                anchor(),
                mkw::txt(state.text(), mk::typo::CALLOUT, p.on_surface_variant),
                anchor(),
            ]
            .spacing(mk::spacing::XS)
            .align_y(iced::Alignment::Center),
        )
        .padding([mk::spacing::S as u16, mk::spacing::M as u16]);

        container(
            container(
                column![level, container(wort).center_x(Length::Fill)]
                    .spacing(mk::spacing::XXS)
                    .align_x(iced::Alignment::Center),
            )
            .padding(iced::Padding {
                top: inner,
                right: inner,
                bottom: MARGIN,
                left: inner,
            })
            // EXACTLY the dock silhouette (user, 2026-07-08 + 2026-07-19): the
            // OSD face builds the same row heights as the resting pill
            // and grows from its content like it — no more forced fixed
            // height that made the OSD taller than the dock.
            .width(if self.tiled {
                Length::Fill
            } else {
                Length::Fixed(width(self.tiles.len()) as f32)
            })
            // R108: the OSD face is the same silhouette — under glass it
            // must not bring back the opaque pill and its old shadow.
            .style({
                let glass_on = self.glass && self.backdrop.is_some();
                // R172i (user): tiled, there is NO body here at all —
                // the strip is not one continuous dock but two floating
                // groups, tiles left and widgets right, each carrying
                // its own glass. A pane behind them welds the two back
                // into one.
                let tiled = self.tiled;
                move |_| {
                    if tiled {
                        container::Style::default()
                    } else if glass_on {
                        mkw::panel::glass_pill(p, BUTTON_RADIUS, MARGIN)
                    } else {
                        mkw::panel::pill(p, BUTTON_RADIUS, MARGIN)
                    }
                }
            }),
        )
        .center_x(Length::Fill)
        .align_bottom(Length::Fill)
        .padding(iced::Padding {
            top: mkw::panel::SHADOW_MARGIN,
            left: mkw::panel::SHADOW_MARGIN,
            right: mkw::panel::SHADOW_MARGIN,
            bottom: mkw::panel::SHADOW_MARGIN,
        })
        .into()
    }

    /// Small widget button for row 2 (symbol + word).
    fn lines_button<'a>(&self, content: Element<'a, Msg>, msg: Msg) -> Element<'a, Msg> {
        let p = self.palette;
        // R172h (user): tiled, the hover must wear the SHAPE of the
        // thing it lights up — the old small radius drew a different
        // silhouette inside the glass body, and the magnifier grew a
        // body that already has an edge. One shape, one highlight.
        let radius = if self.tiled { mk::radius::LARGE } else { mk::radius::SMALL };
        let core = button(content)
            // Bar sizing (user, 2026-07-17): the same padding as the
            // family_button widgets of the top bar.
            // Tiled, a widget is half a tile tall: two rows plus their
            // gap must add up to the height of ONE tile beside them.
            // R172k (user): the text keeps the SAME air to its edge as
            // in a tile. A widget carries one line where a tile carries
            // two, so the pair of rows still stands as tall as one tile.
            .padding([mk::spacing::S as u16, mk::spacing::M as u16])
            .style(move |_, status| mkw::panel::button_style(p, status, radius))
            .on_press(msg);
        mkw::magnifier(core)
    }

    /// R173: the bar's own button recipe, brought down unchanged — a
    /// widget that moved must not change its look on the way, and only
    /// this one knows the ACTIVE pill a menu anchor needs.
    fn family<'a>(
        &'a self,
        glyph: Option<char>,
        title: Option<&'a str>,
        active: bool,
        msg: Msg,
    ) -> Element<'a, Msg> {
        mkw::ui::family_button(glyph, title, active, msg, self.palette, mk::typo::CALLOUT)
    }

    /// One widget of row 2. None = this widget has nothing to say on
    /// this device right now (the bar's grammar: no battery, no
    /// battery widget) — it then takes no space at all.
    fn widget_view(&self, w: Widget, ink: mk::Rgba, ink2: mk::Rgba) -> Option<Element<'_, Msg>> {
        Some(match w {
            Widget::Apps => self.lines_button(
                row![
                    mkw::symbol(mkw::symbol::APPS, mk::font_size::LARGE, ink),
                    mkw::txt("Apps", mk::typo::CALLOUT, ink),
                ]
                .spacing(mk::spacing::S)
                .align_y(iced::Alignment::Center)
                .into(),
                Msg::AppsOpen,
            ),
            Widget::Central => self.lines_button(
                mkw::symbol(mkw::symbol::TUNE, mk::font_size::LARGE, ink),
                Msg::CentralToggle,
            ),
            Widget::Capture => self.lines_button(
                mkw::symbol(mkw::symbol::IMAGE, mk::font_size::LARGE, ink),
                Msg::CaptureOpen,
            ),
            Widget::Hardline => {
                // The state must be readable WITHOUT clicking: the
                // glyph is filled when the node runs, hollow when it
                // does not, and the word says it in plain German.
                let on = self.hardline_on;
                self.lines_button(
                    row![
                        mkw::symbol(
                            if on { mkw::symbol::KEY } else { mkw::symbol::LOCK },
                            mk::font_size::LARGE,
                            if on { ink } else { ink2 },
                        ),
                        mkw::txt(if on { "An" } else { "Aus" }, mk::typo::CALLOUT, if on { ink } else { ink2 }),
                    ]
                    .spacing(mk::spacing::S)
                    .align_y(iced::Alignment::Center)
                    .into(),
                    Msg::HardlineToggle,
                )
            }
            Widget::Clock => mkw::txt(&self.clock, mk::typo::CALLOUT, ink2).into(),
            Widget::Clipboard => {
                // Preview of the newest copy, single-line and short;
                // a click opens the history (matrix-context clips).
                let preview = self.zw_preview.as_deref().map(|t| {
                    let mut z: String = t.split_whitespace().collect::<Vec<_>>().join(" ");
                    if z.chars().count() > 18 {
                        z = z.chars().take(17).collect::<String>() + "…";
                    }
                    z
                });
                self.lines_button(
                    row![
                        mkw::symbol(mkw::symbol::CONTENT_COPY, mk::font_size::LARGE, ink),
                        mkw::txt(
                            preview.unwrap_or_else(|| String::from("Zwischenablage")),
                            mk::typo::CALLOUT,
                            ink2,
                        ),
                    ]
                    .spacing(mk::spacing::S)
                    .align_y(iced::Alignment::Center)
                    .into(),
                    Msg::ClipboardMenu,
                )
            }
            Widget::Battery => {
                // R173, from the bar: a level is a statement, not a
                // button — there is nowhere for a click to go.
                let (percent, loading) = self.battery?;
                let glyph = if loading {
                    mkw::symbol::BATTERY_CHARGING
                } else {
                    mkw::symbol::BATTERY_FULL
                };
                mkw::tag(glyph, format!("{percent} %"), mk::typo::CALLOUT, ink2)
            }
            Widget::Wlan => {
                let on = self.wlan?;
                // The connection NAME is the useful part; the radio
                // state only matters when there is nothing to name.
                let label = match (on, &self.net) {
                    (true, Some(n)) => n.clone(),
                    (true, None) => String::from("WLAN"),
                    (false, _) => String::from("WLAN aus"),
                };
                self.lines_button(
                    row![
                        mkw::symbol(
                            mkw::symbol::WIFI,
                            mk::font_size::LARGE,
                            if on { ink } else { ink2.with_alpha(0.5) },
                        ),
                        mkw::txt(label, mk::typo::CALLOUT, ink2),
                    ]
                    .spacing(mk::spacing::S)
                    .align_y(iced::Alignment::Center)
                    .into(),
                    // The Zentrale is where radios are switched — the
                    // widget shows, the control center acts.
                    Msg::CentralToggle,
                )
            }
            Widget::Bluetooth => {
                let on = self.bluetooth?;
                self.lines_button(
                    mkw::symbol(
                        mkw::symbol::BLUETOOTH,
                        mk::font_size::LARGE,
                        if on { ink } else { ink2.with_alpha(0.5) },
                    ),
                    Msg::CentralToggle,
                )
            }
            Widget::User => {
                let open = self.menu_is(Sheet::Session);
                self.family(
                    Some(mkw::symbol::PERSON),
                    Some(self.user.as_str()),
                    open,
                    if open { Msg::MenuClose } else { Msg::SheetOpen(Sheet::Session) },
                )
            }
            Widget::Matrix => {
                let open = self.menu_is(Sheet::Matrix);
                self.family(
                    Some(mkw::symbol::APPS),
                    Some("Matrix"),
                    open,
                    if open { Msg::MenuClose } else { Msg::SheetOpen(Sheet::Matrix) },
                )
            }
        })
    }

    /// Is that menu open (or already growing)?
    fn menu_is(&self, s: Sheet) -> bool {
        self.menu == Some(s) || self.menu_waiting == Some(s)
    }

    /// The session menu entries — the way OUT of the session, ported
    /// from the bar with its second-click grammar intact.
    fn session_entries(&self) -> Vec<mkw::ui::MenuItem<Msg>> {
        let p = self.palette;
        let mut entries: Vec<mkw::ui::MenuItem<Msg>> = Vec::new();
        for a in Action::ALL {
            if matches!(a, Action::Standby) {
                entries.push(mkw::ui::MenuItem::Separator);
            }
            let (title, color) = if self.confirmation == Some(a) {
                (a.question().to_string(), Some(p.error))
            } else {
                (a.title().to_string(), None)
            };
            entries.push(mkw::ui::MenuItem::Item {
                glyph: Some(a.glyph()),
                title,
                color,
                msg: Msg::Act(a),
            });
        }
        entries
    }

    /// The Matrix menu — the Apple-menu extract (round 28), unchanged
    /// from the bar: "…" asks or opens, the update entry carries its
    /// live badge, and the energy actions stay in the session menu.
    fn matrix_entries(&self) -> Vec<mkw::ui::MenuItem<Msg>> {
        let mut e: Vec<mkw::ui::MenuItem<Msg>> = vec![
            mkw::ui::MenuItem::Item {
                glyph: Some(mkw::symbol::INFO),
                title: String::from("Über diese Matrix"),
                color: None,
                msg: Msg::AppStart("matrix-help"),
            },
            mkw::ui::MenuItem::Separator,
            mkw::ui::MenuItem::Item {
                glyph: Some(mkw::symbol::TUNE),
                title: String::from("Systemeinstellungen …"),
                color: None,
                msg: Msg::AppStart("matrix-settings"),
            },
        ];
        match mk::badge::read("matrix-updater") {
            Some(n) => e.push(mkw::ui::MenuItem::ItemWithBadge {
                glyph: Some(mkw::symbol::RESTART),
                title: String::from("Softwareupdate …"),
                badge: if n == "1" {
                    String::from("1 Update")
                } else {
                    format!("{n} Updates")
                },
                color: None,
                msg: Msg::AppStart("matrix-updater"),
            }),
            None => e.push(mkw::ui::MenuItem::Item {
                glyph: Some(mkw::symbol::RESTART),
                title: String::from("Softwareupdate …"),
                color: None,
                msg: Msg::AppStart("matrix-updater"),
            }),
        }
        e.push(mkw::ui::MenuItem::Separator);
        e.push(mkw::ui::MenuItem::Item {
            glyph: Some(mkw::symbol::WIDGETS),
            title: String::from("Leiste & Dock anpassen …"),
            color: None,
            msg: Msg::AppStart("matrix-settings bars"),
        });
        e.push(mkw::ui::MenuItem::Separator);
        e.push(mkw::ui::MenuItem::Item {
            glyph: Some(mkw::symbol::LOCK),
            title: String::from("Bildschirm sperren"),
            color: None,
            msg: Msg::Act(Action::Lock),
        });
        e.push(mkw::ui::MenuItem::Item {
            glyph: Some(mkw::symbol::LOGOUT),
            title: format!("{} abmelden …", self.user),
            color: None,
            msg: Msg::Act(Action::Logout),
        });
        e
    }

    /// The session menu pill, with the living avatar greeting by name.
    fn session_menu(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let header: Element<'_, Msg> = match matrixkit_icons::avatar_png(&p)
            .map(image::Handle::from_bytes)
        {
            Some(h) => row![
                image(h).width(28).height(28),
                mkw::txt(&self.user, mk::typo::HEADLINE, p.on_surface),
            ]
            .spacing(mk::spacing::S)
            .align_y(iced::Alignment::Center)
            .into(),
            None => mkw::txt(&self.user, mk::typo::HEADLINE, p.on_surface).into(),
        };
        mkw::ui::menu_family(Some(header), self.session_entries(), p)
    }

    /// R106: lay the liquid-glass shader behind the dock content.
    /// The pill is the (only) glass element; its screen rect maps the
    /// wallpaper region beneath the dock into the shader.
    fn glassify<'a>(&self, content: Element<'a, Msg>) -> Element<'a, Msg> {
        // R106 phase 2: the canvas compositor refracts the REAL scene
        // behind the pill (layer-rule `glass` on this namespace) — the
        // in-app wallpaper fake painted over live windows and is retired.
        // The translucent glass_pill on top provides the palette tint.
        return content;
        #[allow(unreachable_code)]
        let (true, Some(backdrop)) = (self.glass, self.backdrop.clone()) else {
            return content;
        };
        let edge = mkw::panel::SHADOW_MARGIN;
        let surface_w = width(self.tiles.len()) as f32 + 2.0 * edge;
        // Same measured room the layout uses — a fixed box would map the
        // wallpaper region wrong for the (much taller) session menu.
        let surface_h = HEIGHT as f32 + self.menu_room() as f32 + 2.0 * edge;
        let (screen_w, screen_h) = self.screen;
        let (_, wall_w, wall_h) = *backdrop.as_ref();
        // Wallpaper covers the screen; same aspect assumed (cover-fill).
        let sx = wall_w as f32 / screen_w.max(1.0);
        let sy = wall_h as f32 / screen_h.max(1.0);
        // Anchored bottom with margin (10 - edge): surface top on screen.
        let top = screen_h - surface_h - (10.0 - edge);
        let left = (screen_w - surface_w) / 2.0;
        let liquid = mkw::glass::Liquid {
            elements: vec![mkw::glass::Element {
                x: edge,
                y: surface_h - edge - HEIGHT as f32,
                width: surface_w - 2.0 * edge,
                height: HEIGHT as f32,
                radius: BUTTON_RADIUS + MARGIN,
            }],
            spacing: 0.0,
            tint: self.palette.surface_container,
            coverage: 0.35,
            backdrop,
            screen_rect: iced::Rectangle {
                x: left * sx,
                y: top * sy,
                width: surface_w * sx,
                height: surface_h * sy,
            },
            backdrop_stamp: self.backdrop_stamp,
        };
        iced::widget::stack![
            iced::widget::shader(liquid)
                .width(Length::Fill)
                .height(Length::Fill),
            content,
        ]
        .into()
    }

    /// The name a tile shows. The launcher index is the ONE source
    /// for human names; an unknown id falls back to its own id so a
    /// tile is never nameless.
    fn label_of(&self, app_id: &str) -> String {
        let full = self.labels.get(app_id).map(String::as_str).unwrap_or(app_id);
        // Found on the Surface: with the maker line saying "Matrix",
        // a name of "Matrix Einstellungen" says it twice and wraps
        // the tile onto a second line. The two lines must not repeat
        // each other — that is the whole point of arrangement B.
        match full.strip_prefix("Matrix ") {
            Some(rest) if maker_of(app_id) == "Matrix" => rest.to_string(),
            // No launcher entry (a service, a foreign binary): make a
            // readable name out of the id instead of showing the id —
            // a tile is words, so the words must be presentable.
            _ if full == app_id => {
                let stem = full.strip_prefix("matrix-").unwrap_or(full);
                let mut chars = stem.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => full.to_string(),
                }
            }
            _ => full.to_string(),
        }
    }

    /// R209: how brightly a tile's edge answers — a short flash after a tap,
    /// a calm pulse while its app starts (until the window arrives).
    fn tile_glow(&self, app_id: &str) -> f32 {
        mk::motion::tap_glow(
            self.flash.since(&app_id.to_string()),
            mkw::TAP_FLASH_S,
            self.bounce.get(app_id).copied(),
        )
    }

    /// One tile on its own surface (R174). It wears NO painted glass:
    /// the compositor puts the real body under the `matrix-tile`
    /// namespace, exactly as it does for the control centre.
    fn tile_pane(&self, i: usize) -> Element<'_, Msg> {
        let p = self.palette;
        let Some(k) = self.tiles.get(i) else {
            return container(Space::new()).into();
        };
        let lines = self.tile_lines(k);
        let has_lines = !lines.is_empty();
        let face = mkw::app_tile_naked(
            self.label_of(&k.app_id),
            maker_of(&k.app_id),
            lines,
            Msg::Click(i),
            Some(move |_at| Msg::Context(i)),
            self.tile_glow(&k.app_id),
            p,
        );
        // Running dot: accent when focused, resting pins stay still —
        // and silent once the window lines already say "running".
        let item_color = if !k.running() || has_lines {
            Color::TRANSPARENT
        } else if k.focus {
            color(p.primary)
        } else {
            color(p.on_surface_variant.with_alpha(0.55))
        };
        let dot = container(Space::new())
            .width(Length::Fixed(DOT))
            .height(Length::Fixed(DOT))
            .style(move |_| container::Style {
                background: Some(item_color.into()),
                border: iced::border::rounded(mk::radius::capsule(DOT)),
                ..Default::default()
            });
        // R174j (user): the width is ESTIMATED, so the surplus must
        // split onto both sides — left-aligned it all fell to the
        // right and the margins looked uneven.
        // On a duo the surface hugs the tile — no room for the dot row; the
        // window lines and the lit edge say «running» there.
        let inner: Element<'_, Msg> = if mk::device::shelf().tile_height.is_some() {
            face
        } else {
            column![face, container(dot).center_x(Length::Fill)]
                .spacing(mk::spacing::XXS)
                .align_x(iced::Alignment::Center)
                .into()
        };
        let stack: Element<'_, Msg> = container(inner)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
        mkw::magnifier(stack)
    }

    /// R209 (Duo): «Apps» dressed exactly like a pinned app's tile — same
    /// lines, same glass, same press, glow and tick.
    fn apps_tile(&self) -> Element<'_, Msg> {
        let face = mkw::app_tile_naked(
            APPS_NAME,
            APPS_MAKER,
            Vec::new(),
            Msg::AppsOpen,
            None::<fn(iced::Point) -> Msg>,
            self.tile_glow(APPS_FLASH),
            self.palette,
        );
        mkw::magnifier(container(face).center_x(Length::Fill).center_y(Length::Fill))
    }

    /// One widget on its own surface — same rule as the tile.
    fn widget_pane(&self, w: Widget) -> Element<'_, Msg> {
        let p = self.palette;
        let (ink, ink2) = match (self.glass && self.backdrop.is_some(), self.vibrant) {
            (true, Some(v)) => v,
            _ => (p.on_surface, p.on_surface_variant),
        };
        let Some(el) = self.widget_view(w, ink, ink2) else {
            return container(Space::new()).into();
        };
        // R173: content that brings no air of its own gets the
        // chrome's padding — the clock clung to the glass edge without.
        let el = if w.brings_padding() {
            el
        } else {
            mkw::tile_chrome_naked(el)
        };
        container(el)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    /// Watch the pointer over one shelf surface, so its glass can grow.
    fn chip<'a>(&self, window: iced::window::Id, content: Element<'a, Msg>) -> Element<'a, Msg> {
        // R209 (Duo, the owner): no magnifier — the tiles keep their size.
        if !mk::device::shelf().magnifier {
            return content;
        }
        iced::widget::mouse_area(content)
            .on_enter(Msg::Hover(window, true))
            .on_exit(Msg::Hover(window, false))
            .into()
    }

    /// Resize ONE shelf surface to the current magnification. The
    /// bottom edge stays put, so the chip grows upwards and to both
    /// sides — the same gesture the kit magnifier draws inside an
    /// ordinary surface, only here it is the surface that moves.
    fn shell_geometry(&self, id: iced::window::Id, t: f32) -> Task<Msg> {
        let Some(shell) = self
            .tile_fleet
            .shell_of(id)
            .or_else(|| self.widget_fleet.shell_of(id))
            .or_else(|| self.widget_fleet2.shell_of(id))
        else {
            return Task::none();
        };
        // Widgets hang off the right edge, tiles off the left — the
        // margin that has to give way differs with the anchor.
        // (R209: «Apps» may lead the LEFT fleet on a duo — the fleet decides.)
        let from_right = self.tile_fleet.pane(id).is_none();
        let scale = 1.0 + (mkw::MAGNIFIER_MAX - 1.0) * t.clamp(0.0, 1.0);
        let width = (shell.width as f32 * scale).round() as u32;
        let height = (shell.height as f32 * scale).round() as u32;
        // Half the growth goes to each side, so the chip stays put.
        let side = (width as i32 - shell.width as i32) / 2;
        let main = (shell.main - side).max(0);
        Task::batch([
            Task::done(Msg::SizeChange { id, size: (width, height) }),
            Task::done(Msg::MarginChange {
                id,
                margin: if from_right {
                    (0, main, shell.cross, 0)
                } else {
                    (0, 0, shell.cross, main)
                },
            }),
        ])
    }

    fn view(&self, window: iced::window::Id) -> Element<'_, Msg> {
        // R210: the subtitle strip — the line, centred, on glass.
        if self.strip.is_some_and(|(id, _)| id == window) {
            let (size, text) = self.subtitle.clone().unwrap_or((1, String::new()));
            let p = self.palette;
            let ink = self.vibrant.map(|v| v.0).unwrap_or(p.on_surface);
            return mkw::glass_strip::face(text, mk::cinema::subtitle::px(size), ink);
        }
        // R174: a fleet surface shows exactly its one element.
        match self.pane_of(window) {
            Some(Pane::Tile(i)) => return self.chip(window, self.tile_pane(i)),
            // R209: on a duo «Apps» leads the shelf as a tile among tiles.
            Some(Pane::Widget(Widget::Apps, _)) if mk::device::shelf().apps_leads => {
                return self.chip(window, self.apps_tile());
            }
            Some(Pane::Widget(w, _)) => return self.chip(window, self.widget_pane(w)),
            None => {}
        }
        // Everything below is the SPINE: tiled it carries only the OSD
        // face, the grown menus and the auto-hide catch edge; in the
        // icon dock it is still the whole pill.
        // R168d: folded, the dock draws NOTHING — the strip is an
        // invisible catch surface at the very edge.
        if self.hidden {
            return container(Space::new().width(Length::Fill).height(Length::Fill)).into();
        }
        // R168e: waking, the whole face floats up from the edge.
        let slide = (1.0 - self.wake.value) * (HEIGHT as f32 + 24.0);
        let p = self.palette;

        // ---- The OSD face: row 1 = level, row 2 = what changed.
        // R121 island pop (Dynamic-Island extract): the face springs
        // into place when it appears or changes KIND; value steps keep
        // the standing face and only the level slides (iOS grammar).
        if self.osd_active() {
            if let Some((state, _)) = self.osd {
                let key = mkw::osd_display::glyph(state).0 as u64;
                return mkw::island(self.osd_face(state), key);
            }
        }

        // ---- Row 1: the tiles
        let mut panel = row![].spacing(GAP).align_y(iced::Alignment::Center);
        for (i, k) in self.tiles.iter().enumerate() {
            // R174: tiled, every tile is its OWN surface — the spine
            // must not draw a second copy of the row.
            if self.tiled {
                break;
            }
            let edge = ICON;
            let image: Element<'_, Msg> = match self.icons.get(&k.app_id).cloned().flatten() {
                Some(h) => image(h).width(edge).height(edge).into(),
                None => container(
                    mkw::txt(initial(&k.app_id), mk::typo::TITLE, p.on_primary_container),
                )
                .center_x(Length::Fixed(edge))
                .center_y(Length::Fixed(edge))
                .style(move |_| container::Style {
                    background: Some(color(p.primary_container).into()),
                    border: iced::border::rounded(mk::radius::inner(
                        BUTTON_RADIUS,
                        (BUTTON - ICON) / 2.0,
                    )),
                    ..Default::default()
                })
                .into(),
            };

            // launchanim (R38, two-stage since R112): the Dock extract
            // names bounceInitialDuration / bounceSecondaryDuration /
            // bounceSecondaryInterval — an eager first hop, then a
            // patient secondary rhythm (smaller hop, long rest) until
            // the window arrives. Rest position stays CENTERED (4 px
            // floor = (52−44)/2); 4 px of headroom remain for the lift.
            let hub = if self.hard {
                0.0
            } else {
                self.bounce.get(&k.app_id).map_or(0.0, |since| {
                    let t = since.elapsed().as_secs_f32();
                    if t < 0.55 {
                        // Initial bounce: full height, eager.
                        (t * std::f32::consts::PI / 0.55).sin() * 4.0
                    } else {
                        // Secondary rhythm: 0.45 s hop, 0.9 s rest.
                        let ts = (t - 0.55) % 1.35;
                        if ts < 0.45 {
                            (ts * std::f32::consts::PI / 0.45).sin() * 2.0
                        } else {
                            0.0
                        }
                    }
                })
            };
            // R172: the tile replaces the icon. Two lines carry the
            // identity (name leads, origin qualifies — decided by
            // looking at both on the Surface), and the dock keeps its
            // own narrower metric. `dock-style` is the way back: set
            // it to "icons" and the old face returns.
            let button: Element<'_, Msg> = if self.tiled {
                let name = self.label_of(&k.app_id);
                let maker = maker_of(&k.app_id);
                mkw::magnifier(mkw::app_tile_auto(
                    name,
                    maker,
                    Msg::Click(i),
                    Some(move |_at| Msg::Context(i)),
                    p,
                ))
            } else {
                let button = button(
                    container(image)
                        .center_x(Length::Fill)
                        .align_y(iced::alignment::Vertical::Bottom)
                        .padding(iced::Padding {
                            bottom: (BUTTON - ICON) / 2.0 + hub,
                            ..iced::Padding::ZERO
                        }),
                )
                .width(Length::Fixed(BUTTON))
                .height(Length::Fixed(BUTTON))
                .padding(0)
                .style(move |_theme, status| mkw::panel::button_style(p, status, BUTTON_RADIUS))
                .on_press(Msg::Click(i));
                iced::widget::mouse_area(button)
                    .on_right_press(Msg::Context(i))
                    .into()
            };
            // Badge glyph: red capsule top right (badgeLabel grammar).
            let button: Element<'_, Msg> = match &k.badge {
                Some(text) => iced::widget::stack![
                    button,
                    container(
                        container(mkw::txt(text.clone(), mk::typo::LABEL, p.on_primary))
                            .padding([0, 5])
                            .style(move |_| container::Style {
                                background: Some(color(p.error).into()),
                                border: iced::border::rounded(mk::radius::capsule(16.0)),
                                ..Default::default()
                            }),
                    )
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Length::Fill),
                ]
                .into(),
                None => button,
            };
            // Kit magnifier: the tile grows springily under the pointer.
            let button = if self.tiled { button } else { mkw::magnifier_strong(button, 1.22) };

            // Dot: running (accent when focused) — resting pins stay still.
            let item_color = if !k.running() {
                Color::TRANSPARENT
            } else if k.focus {
                color(p.primary)
            } else {
                color(p.on_surface_variant.with_alpha(0.55))
            };
            let item = container(Space::new())
                .width(Length::Fixed(DOT))
                .height(Length::Fixed(DOT))
                .style(move |_| container::Style {
                    background: Some(item_color.into()),
                    border: iced::border::rounded(mk::radius::capsule(DOT)),
                    ..Default::default()
                });

            panel = panel.push(
                column![
                    button,
                    container(item).center_x(if self.tiled {
                        Length::Fill
                    } else {
                        Length::Fixed(BUTTON)
                    }),
                ]
                .spacing(mk::spacing::XXS)
                .align_x(iced::Alignment::Center),
            );
        }

        // ---- Row 2: the widgets (like the bar, only in the dock)
        // R111 vibrant text: on glass the ink follows the measured
        // backdrop luminance instead of the fixed palette tones.
        let (ink, ink2) = match (self.glass && self.backdrop.is_some(), self.vibrant) {
            (true, Some(v)) => v,
            _ => (p.on_surface, p.on_surface_variant),
        };
        // R172j (user): the widgets keep the SAME gap the tiles keep
        // between themselves — one rhythm across the whole edge.
        let widget_gap = if self.tiled { GAP } else { mk::spacing::M };
        let mut bottom = row![]
            .spacing(widget_gap)
            .align_y(iced::Alignment::Center);
        let mut bottom2 = row![]
            .spacing(widget_gap)
            .align_y(iced::Alignment::Center);
        // Build first, split second: a silent widget (no battery on
        // this machine) must not leave a hole in one of the two rows.
        // R174: tiled, the widgets live on their own surfaces too.
        let built: Vec<(Widget, Element<'_, Msg>)> = if self.tiled {
            Vec::new()
        } else {
            self.widgets
                .iter()
                .filter_map(|w| self.widget_view(*w, ink, ink2).map(|el| (*w, el)))
                .collect()
        };
        let half = built.len().div_ceil(2);
        for (wi, (w, el)) in built.into_iter().enumerate() {
            // R172e (user): tiled, the widgets live on the RIGHT in two
            // rows while the tiles line up on the left — the dock splits
            // into two halves instead of one centred strip.
            // Same glass body as a tile: a widget is a thing of the
            // same kind, so it wears the same chrome. R173: which
            // fassung depends on the content — a button brings its own
            // air, a bare line of text does not and would otherwise
            // cling to the glass edge (the clock did exactly that).
            let el = if self.tiled {
                if w.brings_padding() {
                    mkw::tile_chrome_bare(el, p)
                } else {
                    mkw::tile_chrome(el, p)
                }
            } else {
                el
            };
            if self.tiled && wi >= half {
                bottom2 = bottom2.push(el);
            } else {
                bottom = bottom.push(el);
            }
        }

        // Row scroller (R49, user finding): if a row overflowed the
        // pill width (e.g. a long clipboard preview in
        // row 2), it used to glitch out of the dock. Now EVERY row is
        // its own horizontal scroller — invisible bar,
        // wheel/trackpad act sideways, and because the event goes to the
        // row under the mouse, exactly that one scrolls. If the content
        // fits, it stays centered as before.
        let inner_width = if self.tiled {
            // The shelf is as wide as the screen; the rows may use all
            // of it minus the side air the bar also keeps.
            f32::INFINITY
        } else {
            width(self.tiles.len()) as f32 - 2.0 * MARGIN
        };

        // R108: with compositor glass the shadow comes from the same
        // distance field as the body (shape family) — the app draws no
        // ring of its own, so there is exactly one silhouette author.
        let glass_active = self.glass && self.backdrop.is_some();
        let pill_inner = container(
                if self.tiled {
                    // R172l (user): the tile row takes only the room the
                    // widgets leave — and SCROLLS inside it. Growing past
                    // that pushed the widgets off the screen, which is
                    // exactly what a scroller exists to prevent.
                    column![row![
                        container(lines_scroller(panel.into(), inner_width, "dock-zeile-1"))
                            .width(Length::Fill)
                            .clip(true),
                        Space::new().width(mk::spacing::L),
                        column![bottom, bottom2]
                            .spacing(mk::spacing::XS)
                            .align_x(iced::Alignment::End),
                    ]
                    .align_y(iced::Alignment::Center)]
                } else {
                    column![
                        container(lines_scroller(panel.into(), inner_width, "dock-zeile-1")).center_x(Length::Fill),
                        container(lines_scroller(bottom.into(), inner_width, "dock-zeile-2")).center_x(Length::Fill),
                    ]
                }
                .spacing(mk::spacing::XXS)
                .align_x(iced::Alignment::Center),
            )
            .padding(iced::Padding::from(MARGIN))
            // R106e: the pill claims its FULL reserved rect (width x HEIGHT)
            // instead of hugging content. The compositor glass body is the
            // surface minus the uniform shadow margin — only when the pill
            // fills that same rect do the two shapes coincide pixel-exact
            // (user finding: any slack reads as a second, offset layer).
            .width(if self.tiled {
                Length::Fill
            } else {
                Length::Fixed(width(self.tiles.len()) as f32)
            })
            .height(Length::Fixed(HEIGHT as f32))
            .align_y(iced::Alignment::Center)
            .style({
                let glass_on = self.glass && self.backdrop.is_some();
                // R172i (user): tiled, there is NO body here at all —
                // the strip is not one continuous dock but two floating
                // groups, tiles left and widgets right, each carrying
                // its own glass. A pane behind them welds the two back
                // into one.
                let tiled = self.tiled;
                move |_| {
                    if tiled {
                        container::Style::default()
                    } else if glass_on {
                        mkw::panel::glass_pill(p, BUTTON_RADIUS, MARGIN)
                    } else {
                        mkw::panel::pill(p, BUTTON_RADIUS, MARGIN)
                    }
                }
            })
            .into();
        // R172d (user): the shelf itself disappears — only the tiles
        // float at the edge. Each carries its own glass now, so a pane
        // behind them would just be a second, redundant material.
        let dressed: Element<'_, Msg> = if self.tiled {
            pill_inner
        } else if glass_active {
            pill_inner
        } else {
            mkw::panel::shadow_layers(pill_inner, BUTTON_RADIUS + MARGIN)
        };
        let pill = container(dressed).center_x(Length::Fill);

        // The grown-open menu: a pill ABOVE the shelf. A click beside
        // it closes. R173: the two menus that came down from the bar —
        // tile contexts live in matrix-context and never grow the dock.
        let Some(open) = self.menu else {
            return self.glassify(
                container(pill)
                    .center_x(Length::Fill)
                    .align_bottom(Length::Fill)
                    .padding(iced::Padding {
                        top: mkw::panel::SHADOW_MARGIN,
                        // R172 (user, on the Surface): the shelf must
                        // keep the BAR's 10 px at the sides, not the
                        // 40 px shadow margin — the two furnitures
                        // share one silhouette. Vertically the shadow
                        // still needs its room.
                        left: if self.tiled { SHELF_SIDE } else { mkw::panel::SHADOW_MARGIN },
                        right: if self.tiled { SHELF_SIDE } else { mkw::panel::SHADOW_MARGIN },
                        bottom: mkw::panel::SHADOW_MARGIN,
                    })
                    .into(),
            );
        };
        let menu_pill: Element<'_, Msg> = match open {
            Sheet::Session => self.session_menu(),
            Sheet::Matrix => mkw::ui::menu_family(None, self.matrix_entries(), p),
        };
        // Both shelf menus hang under the widget column at the right
        // edge, exactly where their anchor button stands.
        let placed: Element<'_, Msg> = container(menu_pill)
            .align_x(iced::alignment::Horizontal::Right)
            .width(Length::Fill)
            .into();
        let menu_line = iced::widget::mouse_area(
            container(placed)
                .width(Length::Fill)
                .height(Length::Fixed(self.menu_room() as f32))
                // Stick right above the dock pill, not at the top edge.
                .align_y(iced::alignment::Vertical::Bottom)
                .padding(iced::Padding { bottom: mk::spacing::XS, ..iced::Padding::ZERO }),
        )
        .on_press(Msg::MenuClose);

        mkw::slide_in(
            self.glassify(
                container(column![menu_line, pill])
                    // R173: the SAME frame the resting dock wears — an
                    // open menu must not lift the shelf off its edge
                    // (the old menu path shrank the top margin away and
                    // the whole dock jumped up by the shadow margin).
                    .align_bottom(Length::Fill)
                    .padding(iced::Padding {
                        top: mkw::panel::SHADOW_MARGIN,
                        left: if self.tiled { SHELF_SIDE } else { mkw::panel::SHADOW_MARGIN },
                        right: if self.tiled { SHELF_SIDE } else { mkw::panel::SHADOW_MARGIN },
                        bottom: mkw::panel::SHADOW_MARGIN,
                    })
                    .into()
            ),
            slide,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_names_from_the_bar_are_understood() {
        // R173: a widget must keep its name when it moves down, and the
        // German legacy spellings of the bar keep working.
        assert_eq!(Widget::out("user"), Some(Widget::User));
        assert_eq!(Widget::out("nutzer"), Some(Widget::User));
        assert_eq!(Widget::out("battery"), Some(Widget::Battery));
        assert_eq!(Widget::out("akku"), Some(Widget::Battery));
        assert_eq!(Widget::out("wlan"), Some(Widget::Wlan));
        assert_eq!(Widget::out("bluetooth"), Some(Widget::Bluetooth));
        assert_eq!(Widget::out("matrix"), Some(Widget::Matrix));
        assert_eq!(Widget::out("quatsch"), None);
    }

    #[test]
    fn bare_text_widgets_ask_for_the_padded_chrome() {
        // The open R172 bug: a bare line of text in the bare glass body
        // clings to its edge — only buttons bring their own air.
        assert!(!Widget::Clock.brings_padding());
        assert!(!Widget::Battery.brings_padding());
        assert!(Widget::Apps.brings_padding());
        assert!(Widget::User.brings_padding());
    }

    #[test]
    fn only_radio_widgets_arm_the_network_listener() {
        assert!(Widget::Wlan.needs_network());
        assert!(Widget::Bluetooth.needs_network());
        assert!(!Widget::Clock.needs_network());
    }
}

// ------------------------------------------------------------ R187: Hardline

/// Does the node run? The local socket is the honest test: it exists
/// only while the service does, and STOP removes it before exiting.
fn hardline_socket() -> std::path::PathBuf {
    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => std::path::PathBuf::from(dir).join("hardline-node.sock"),
        Err(_) => {
            let user = std::env::var("USER").unwrap_or_else(|_| "user".into());
            std::path::PathBuf::from(format!("/tmp/hardline-node-{user}.sock"))
        }
    }
}

fn hardline_running() -> bool {
    std::os::unix::net::UnixStream::connect(hardline_socket()).is_ok()
}

/// Ask the node to end itself — a clean exit that also takes the
/// socket with it. No signal, no pkill: the service closes its own
/// door, which is the only way it can promise there is nothing left.
fn hardline_stop() {
    use std::io::Write;
    if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(hardline_socket()) {
        let _ = stream.write_all(b"STOP\n");
    }
}


/// The window lines of a tile (R174j). The house dock keeps its own
/// twin (`haus::window_lines`) on its iced-free window model.
pub(crate) fn tile_lines_of(k: &Tile, label: &str) -> Vec<String> {
        // A title that only repeats the app's own name says nothing —
        // "Matrix Dateien" under "Dateien / Matrix" is noise (seen on
        // the Surface). Such windows keep the plain dot instead.
        let label = label.to_string();
        let mut lines: Vec<String> = k
            .window
            .iter()
            .filter(|w| {
                let t = w.title.trim();
                !t.is_empty() && t != label && t != format!("Matrix {label}")
            })
            .take(3)
            .map(|w| {
                let short: String = w.title.chars().take(18).collect();
                format!("→ {short}")
            })
            .collect();
        if k.window.len() > 3 {
            lines.push(format!("→ … {} weitere", k.window.len() - 3));
        }
        lines
}