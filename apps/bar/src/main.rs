//! Matrix Bar — app #12, member of the MatrixKit bar family.
//!
//! The bar at the top edge, built from WIDGETS since 2.0: three zones
//! (left | center | right), assigned via the settings culture —
//! `~/.config/matrix/bar-widgets`, z. B. `fokus | uhr | puls zentrale
//! nutzer`. Widgets: fokus (aktive App), uhr, puls (Systemwerte),
//! zentrale (opens/closes the control center), nutzer (account name +
//! session menu: lock, log out, standby, restart, shut down —
//! final things want a second click).
//!
//! Layer::Top (bottom starves in the canvas session, task #39).
//! v1 reserves NO space (exclusive_zone 0) while the DMS bar exists
//! in parallel — `matrix-bar --reserve` arms the zone when the
//! bar replaces the old one (`--reservieren` accepted as legacy alias).

use matrixkit_widgets::iced;
use matrixkit_widgets::iced_layershell;
use matrixkit_widgets::iced::widget::{button, column, container, image, mouse_area, row, stack, Space};
use matrixkit_widgets::iced::{Element, Length, Subscription, Task};
use matrixkit_widgets::iced_layershell::to_layer_message;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;

mod haus;
mod licht;
mod media;
mod tray;
use matrixkit_widgets::tick;

pub(crate) const HEIGHT: u32 = 44;
/// R168d: the folded bar — an INVISIBLE catch strip flush with the
/// top edge (the normal bar floats 10 px below it).
const STRIP: u32 = 16;
/// Uniform inner padding of the pill bar (user, R50c): hover surfaces
/// keep the same air to the bar edge ALL AROUND — left as above.
/// It also caps the button height (44 − 2·6 = 32) against the tall
/// line boxes of the symbol font.
pub(crate) const INSET: f32 = 6.0;
/// Surface height with a menu open (R103): tall enough to cover every
/// screen — the free area is the dismiss backdrop, so a click anywhere
/// outside the menu closes it.
const MENU_HEIGHT: u32 = 2400;

/// R209: the panel that carries the bar on a duo device (the lower one in
/// laptop posture).
pub(crate) const BAR_PANEL_DUO: &str = "DSI-2";

fn main() -> Result<(), iced_layershell::Error> {
    // Camp 30 of the iced replacement: --erstlicht is the window-house
    // path (shadow doctrine).
    if std::env::args().nth(1).as_deref() == Some("--erstlicht") {
        // R174j applies to BOTH paths: with the tile shelf the top bar
        // steps down. The guard sat only in the iced branch, so the
        // light bar appeared exactly where the reference takes its
        // leave (finger test 16.08.2026).
        if mk::settings::read("dock-style").as_deref() == Some("tiles") {
            return Ok(());
        }
        if let Err(e) = licht::run() {
            eprintln!("[bar] Fensterhaus-Fehler: {e}");
        }
        return Ok(());
    }
    // R216: `--haus` is the house path (matrixkit-shell, no iced) — the
    // spine, the glass chips and the shell cards as fleet surfaces of ONE
    // process. iced stays the default until the device test.
    // R216 (02.11.2026): the HOUSE is the standard. Way back without a
    // reinstall: `--iced`, or the setting `shell-iced=on` (then kill — the
    // watchdog respawns into iced).
    let iced = std::env::args().any(|a| a == "--iced") || mk::settings::read("shell-iced").as_deref() == Some("on");
    if std::env::args().any(|a| a == "--haus") || !iced {
        if mk::settings::read("dock-style").as_deref() == Some("tiles") {
            return Ok(());
        }
        // Round 80: one bar (the dock's double start, 04.11.).
        if !matrixkit_widgets::daemon_single() {
            eprintln!("[bar] ältere Instanz lebt — Ende");
            return Ok(());
        }
        if let Err(e) = haus::run() {
            eprintln!("[bar] Haus-Fehler: {e}");
        }
        return Ok(());
    }
    matrixkit_widgets::present_mailbox();
    // R174j (user): with the tile shelf the top bar retires — the
    // session start may still spawn us, but we bow out. The settings
    // switch kills/spawns live; the watchdog skips us while tiled.
    if mk::settings::read("dock-style").as_deref() == Some("tiles") {
        return Ok(());
    }
    // R209: the bar runs the daemon pattern (dock R174) — its own strip is
    // one surface (the SPINE), and on a duo every widget is a glass chip of
    // its own beside it. Elsewhere the spine is the whole bar, as before.
    mkw::app_canvas::run::<App>("matrix-bar", mkw::app_canvas::background())
}

impl mkw::app_canvas::Daemon for App {
    type Msg = Msg;
    fn new() -> (Self, Task<Msg>) {
        App::new()
    }
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        let task = App::update(self, msg);
        Task::batch([task, self.sync_chips()])
    }
    fn view(&self, window: iced::window::Id) -> Element<'_, Msg> {
        App::view(self, window)
    }
    fn subscription(&self) -> Subscription<Msg> {
        App::subscription(self)
    }
}

/// R209 (Duo, the owner): ONE bar, on the lower panel (laptop posture:
/// DSI-2) — not on whichever panel happens to be active. `bar-output` in
/// the settings names another panel; elsewhere the bar stays put.
fn bar_output() -> iced_layershell::reexport::OutputOption {
    mkw::panel::output_option("bar-output", BAR_PANEL_DUO)
}

/// The spine's geometry — everything the old single window was. As chips
/// it is an invisible 1 px line that only reserves the strip and grows
/// for the menus (a strip-wide transparent surface would swallow the taps
/// meant for the chips); its namespace matches no glass rule.
fn spine_settings(reserve: bool) -> iced_layershell::reexport::NewLayerShellSettings {
    use matrixkit_widgets::iced_layershell::reexport::{
        Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings,
    };
    let chips = mk::device::bar_chips();
    NewLayerShellSettings {
        // width 0 = spans between the anchors (full edge).
        size: Some((0, if chips { 1 } else { HEIGHT + mkw::panel::SHADOW_MARGIN as u32 })),
        anchor: Anchor::Top | Anchor::Left | Anchor::Right,
        // R50: the bar floats like the dock — 10 px air to the edge.
        margin: Some((10, 0, 0, 0)),
        layer: Layer::Top,
        keyboard_interactivity: KeyboardInteractivity::None,
        exclusive_zone: Some(if reserve { reserved_zone() } else { 0 }),
        namespace: Some(String::from(if chips { "matrix-bar-spine" } else { "matrix-bar" })),
        output_option: bar_output(),
        ..Default::default()
    }
}

/// The strip the bar reserves. The compositor adds the 10 px top margin
/// to it, and niri's window gap comes on top — so the chips (the owner,
/// 01.11.2026) reserve just their height: one gap between chip and window,
/// as everywhere else. The whole bar keeps its old measure.
pub(crate) fn reserved_zone() -> i32 {
    if mk::device::bar_chips() {
        HEIGHT as i32
    } else {
        HEIGHT as i32 + 10
    }
}

/// What a chip surface shows (R209).
#[derive(Debug, Clone, Copy, PartialEq)]
enum Pane {
    /// One widget of the left or right zone.
    Widget(Widget),
    /// The whole center zone in one chip (the compositor centers it).
    Center,
}

/// Air between two chips and at the panel sides — the dock's measures.
const CHIP_GAP: i32 = mk::spacing::XS as i32;
const CHIP_SIDE: i32 = 10;

// ---------------------------------------------------------------- Widgets

/// A bar widget — the building blocks of the three zones.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Widget {
    Matrix,
    Tray,
    Focus,
    Clock,
    Pulse,
    Bell,
    Battery,
    Central,
    User,
    /// R70c: opens the capture panel.
    Capture,
    /// R119 Now Playing: the MPRIS broker view (title + play state).
    Media,
}

impl Widget {
    fn out(name: &str) -> Option<Self> {
        // R103: English values are the contract; German ones stay
        // accepted — existing ~/.config/matrix/bar-widgets files.
        match name {
            "matrix" => Some(Self::Matrix),
            "focus" | "fokus" => Some(Self::Focus),
            "clock" | "uhr" => Some(Self::Clock),
            "pulse" | "puls" => Some(Self::Pulse),
            "bell" | "glocke" => Some(Self::Bell),
            "tray" => Some(Self::Tray),
            "battery" | "akku" => Some(Self::Battery),
            "central" | "zentrale" => Some(Self::Central),
            "media" | "medien" => Some(Self::Media),
            "user" | "nutzer" => Some(Self::User),
            "capture" | "aufnahme" => Some(Self::Capture),
            _ => None,
        }
    }
}

impl Widget {
    /// Round 113: every widget (the wiggle mode's library).
    pub(crate) const ALL: [Widget; 11] = [
        Widget::Matrix, Widget::Focus, Widget::Clock, Widget::Pulse, Widget::Bell, Widget::Tray,
        Widget::Battery, Widget::Central, Widget::Media, Widget::User, Widget::Capture,
    ];

    /// The contract word (what `bar-widgets` holds).
    pub(crate) fn word(self) -> &'static str {
        match self {
            Widget::Matrix => "matrix",
            Widget::Focus => "focus",
            Widget::Clock => "clock",
            Widget::Pulse => "pulse",
            Widget::Bell => "bell",
            Widget::Tray => "tray",
            Widget::Battery => "battery",
            Widget::Central => "central",
            Widget::Media => "media",
            Widget::User => "user",
            Widget::Capture => "capture",
        }
    }

    /// Its name for people (the library).
    pub(crate) fn label(self) -> &'static str {
        match self {
            Widget::Matrix => "Matrix-Menü",
            Widget::Focus => "Aktives Fenster",
            Widget::Clock => "Uhrzeit",
            Widget::Pulse => "Systemlast",
            Widget::Bell => "Mitteilungen",
            Widget::Tray => "Symbolleiste",
            Widget::Battery => "Akku",
            Widget::Central => "Kontrollzentrum",
            Widget::Media => "Wiedergabe",
            Widget::User => "Benutzer",
            Widget::Capture => "Aufnahme",
        }
    }
}

/// Round 113: write the three zones back (the wiggle mode).
pub(crate) fn widgets_write(zones: &(Vec<Widget>, Vec<Widget>, Vec<Widget>)) {
    mk::settings::write("bar-widgets", &zones_text(zones));
}

fn zones_text(zones: &(Vec<Widget>, Vec<Widget>, Vec<Widget>)) -> String {
    let words = |z: &Vec<Widget>| z.iter().map(|w| w.word()).collect::<Vec<_>>().join(" ");
    format!("{} | {} | {}", words(&zones.0), words(&zones.1), words(&zones.2))
}

/// Zone assignment from `~/.config/matrix/bar-widgets` (settings culture):
/// three zones split by `|`, widgets by spaces. If the file is missing,
/// the default assignment applies.
pub(crate) fn widgets_read() -> (Vec<Widget>, Vec<Widget>, Vec<Widget>) {
    zones_parse(
        &mk::settings::read("bar-widgets").unwrap_or_else(|| String::from(mk::device::bar_default())),
    )
}

/// Pure parsing (testable): three zones split by `|`, unknowns drop out quietly.
#[cfg(test)]
mod zone_tests {
    #[test]
    fn zones_round_trip() {
        for w in super::Widget::ALL {
            assert_eq!(super::Widget::out(w.word()), Some(w));
        }
        let z = super::zones_parse("matrix focus | clock | battery central user");
        assert_eq!(super::zones_parse(&super::zones_text(&z)), z);
    }
}

fn zones_parse(conf: &str) -> (Vec<Widget>, Vec<Widget>, Vec<Widget>) {
    let mut zones = conf.splitn(3, '|').map(|z| {
        z.split_whitespace()
            .filter_map(Widget::out)
            .collect::<Vec<_>>()
    });
    (
        zones.next().unwrap_or_default(),
        zones.next().unwrap_or_default(),
        zones.next().unwrap_or_default(),
    )
}

// ------------------------------------------------------------- Sitzung

/// The entries of the session menu. Final things ask via a second click.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Action {
    Lock,
    Logout,
    Standby,
    Restart,
    Shutdown,
}

impl Action {
    pub(crate) const ALLE: [Action; 5] = [
        Action::Lock,
        Action::Logout,
        Action::Standby,
        Action::Restart,
        Action::Shutdown,
    ];

    pub(crate) fn glyph(self) -> char {
        match self {
            Action::Lock => mkw::symbol::LOCK,
            Action::Logout => mkw::symbol::LOGOUT,
            Action::Standby => mkw::symbol::DARK_MODE,
            Action::Restart => mkw::symbol::RESTART,
            Action::Shutdown => mkw::symbol::POWER,
        }
    }

    pub(crate) fn title(self) -> &'static str {
        match self {
            Action::Lock => "Sperren",
            Action::Logout => "Abmelden …",
            Action::Standby => "Standby",
            Action::Restart => "Neustart",
            Action::Shutdown => "Herunterfahren",
        }
    }

    /// Final actions want a second, red click.
    pub(crate) fn permanent(self) -> bool {
        matches!(self, Action::Logout | Action::Restart | Action::Shutdown)
    }

    /// The glass dialog's words (R209): title, note, red action.
    pub(crate) fn dialog(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Action::Logout => ("Abmelden?", "Alle Programme werden beendet.", "Abmelden"),
            Action::Restart => ("Neu starten?", "Ungesicherte Arbeit geht verloren.", "Neu starten"),
            _ => ("Ausschalten?", "Ungesicherte Arbeit geht verloren.", "Ausschalten"),
        }
    }

    fn question(self) -> &'static str {
        match self {
            Action::Logout => "Wirklich abmelden?",
            Action::Restart => "Wirklich neu starten?",
            Action::Shutdown => "Wirklich ausschalten?",
            _ => "",
        }
    }

    pub(crate) fn execute(self) {
        match self {
            Action::Lock => {
                // Own lock screen first; fallback DMS, then logind.
                mkw::panel::app_start("matrix-lock");
            }
            // Matrix asked already (glass dialog, R209) — niri must not ask
            // again with its own «Press Enter to confirm».
            // The logout sound (02) plays SYNCHRONOUSLY to the end before the
            // session falls — otherwise poweroff would cut it off (R30).
            Action::Logout => {
                mk::feedback::now("abmeldung", "02-abmeldung.wav");
                mk::canvas::logout(false);
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
pub(crate) fn user_read() -> String {
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

/// First letter capitalized — account names stand upright in the bar.
fn large(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_else(|| s.to_string())
}

/// Focused window as "App — Title" (shortened), via the kit bridge.
/// Display name from the .desktop entry (product language!) — the
/// prettified app id is only the fallback for apps without one.
fn desktop_display_name(app_id: &str) -> Option<String> {
    mk::apps::display_name(app_id)
}

pub(crate) fn focus_read() -> Option<String> {
    let f = mkw::canvas::focus()?;
    if let Some(name) = desktop_display_name(&f.app_id) {
        let title = f.title.clone();
        return Some(if title.is_empty() || title == name {
            name
        } else {
            format!("{name} — {title}")
        });
    }
    let short = f
        .app_id
        .rsplit('.')
        .next()
        .unwrap_or(&f.app_id)
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|first| first.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut s = if f.title.is_empty() || f.title == short {
        short
    } else {
        format!("{short} — {}", f.title)
    };
    if s.chars().count() > 64 {
        s = s.chars().take(63).collect::<String>() + "…";
    }
    Some(s)
}

/// Volume of the default sink: "45 %" or "Stumm".
pub(crate) fn volume_read() -> Option<String> {
    let z = mk::command::first_line("wpctl", &["get-volume", "@DEFAULT_AUDIO_SINK@"])?;
    if z.contains("MUTED") {
        return Some(String::from("Stumm"));
    }
    let value: f32 = z.split_whitespace().nth(1)?.parse().ok()?;
    Some(format!("{:.0} %", value * 100.0))
}

/// Battery level (0–100, charging?) — None on devices without one (PC).
pub(crate) fn battery_read() -> Option<(u8, bool)> {
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

/// Is DMS alive? The bell only shows if its history exists.
/// Name of the active network connection (without loopback).
pub(crate) fn net_read() -> Option<String> {
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

// ----------------------------------------------------------------- App

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Msg {
    /// R168c auto-hide: raw pointer/finger pokes from the event stream.
    EdgeTouch(iced::Point),
    EdgeMove(iced::Point),
    EdgeUp,
    Poke,
    WakeTick,
    FadeTick,
    Clock,
    Pulse,
    Focus,
    CentralToggle,
    /// R69: laufende Bildschirmaufnahme beenden.
    FilmStop,
    /// R70c: the capture widget opens the panel.
    CapturePanel,
    BellClick,
    TrayClick(String, String),
    TrayRight(String, String),
    MenuOpen,
    MenuClose,
    /// The Matrix menu (Apple-menu extract, round 28) at the far left.
    MatrixOpen,
    /// R210 duo grammar: the focused app's name chip opens ITS menu —
    /// windows carry no head there, settings and close live here.
    AppMenuOpen,
    AppSettings,
    AppClose,
    /// The app card's duo section: swap the panels / to the other panel.
    AppSwap,
    /// R211 Krabbler in the app: the switch, and one species (index into
    /// `mk::crawlers::SPECIES`).
    AppCrawlSwitch,
    AppCrawlSpecies(usize),
    AppCardClose,
    /// Start an app from the Matrix menu (closes the menu).
    App(&'static str),
    Action(Action),
    /// R119 Now Playing: click toggles, right-click skips.
    MediaPlayPause,
    MediaNext,
    /// R209: a surface got its size — the spine's width is the panel's.
    Resized(iced::window::Id, f32),
    /// R209 glass dialog: answered (true = do it) — button, Enter/Esc, veil.
    DialogAnswer(bool),
    /// A key from the dialog's veil.
    DialogKey(bool),
    /// R209 chips: a chip was tapped — answer like a dock tile, then act.
    ChipTap(Widget),
    /// Fades the chips' tap glow.
    FlashTick,
}

/// R106/R108: glass is on unless switched off — and never fights the
/// accessibility contract (reduced transparency = opaque surfaces).
fn glass_wanted() -> bool {
    mkw::glass::wanted()
}

struct App {
    /// R168c auto-hide: the bar folds to a strip; a swipe DOWN from
    /// the top edge (or the mouse touching it) brings it back.
    autohide: bool,
    hidden: bool,
    reserve: bool,
    last_poke: std::time::Instant,
    edge_from: Option<f32>,
    /// R168e: the wake slide — 0 = above the edge, 1 = home.
    wake: mk::motion::Spring,
    palette: mk::Palette,
    /// R108: compositor glass behind the bar pill — the app then draws
    /// neither fill nor shadow ring (accessibility-aware like the dock).
    glass: bool,
    /// R111 vibrant text: ink pair measured from the wallpaper behind
    /// the bar strip (primary, secondary); None = palette tones.
    vibrant: Option<(mk::Rgba, mk::Rgba)>,
    /// R119 Now Playing: the current MPRIS broker view; None hides
    /// the widget (like the tray with no items).
    now_playing: Option<media::NowPlaying>,
    /// Truncated label for the strip (owned — the view borrows it).
    now_label: String,
    watcher: mk::PaletteWatcher,
    zones: (Vec<Widget>, Vec<Widget>, Vec<Widget>),
    sys: sysinfo::System,
    clock: String,
    date_val: String,
    focus: Option<String>,
    cpu: f32,
    ram: String,
    volume: Option<String>,
    net: Option<String>,
    user: String,
    pulse_counter: u8,
    battery: Option<(u8, bool)>,
    menu_open: bool,
    /// Matrix menu (left) open — shares growth & closing with the session
    /// menu, but only one is ever open.
    matrix_open: bool,
    /// The app card is open (its chip stays lit).
    app_open: bool,
    /// R210: the focused APP's glass card (veil + card surfaces).
    app_card: Option<AppCard>,
    tray_items: std::sync::Arc<std::sync::Mutex<Vec<tray::TrayItem>>>,
    tray_command: std::sync::mpsc::Sender<tray::TrayCommand>,
    /// Final action waiting for its second (red) click.
    confirmation: Option<Action>,
    /// R69: start epoch of a running screen recording — the bar then
    /// shows the red stop dot (reference menu-bar behavior).
    film_start: Option<u64>,
    /// R209: the spine surface (the whole bar unless chips).
    spine: iced::window::Id,
    /// R209 chips (duo): one glass surface per widget.
    chips: bool,
    left: mkw::fleet::Fleet<Pane>,
    right: mkw::fleet::Fleet<Pane>,
    center: mkw::fleet::Fleet<Pane>,
    /// The panel width the spine was given (0 = not yet known).
    panel_width: f32,
    /// R209: the final action the glass dialog asks about, with its two
    /// surfaces (veil, card) while it is open.
    dialog: Option<(Action, iced::window::Id, iced::window::Id)>,
    /// Chips answering a tap: when each was tapped (glow fades out).
    flash: mkw::TapFlash<Widget>,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        // Status-bar reference extract: the bar is the tray watcher (round 24).
        let tray_items_init = tray::start();
        let mut app = App {
            palette: mk::Palette::load().unwrap_or_default(),
            glass: glass_wanted(),
            vibrant: None,
            now_playing: None,
            now_label: String::new(),
            watcher: mk::PaletteWatcher::new(),
            zones: widgets_read(),
            sys: sysinfo::System::new(),
            clock: String::new(),
            date_val: String::new(),
            focus: None,
            cpu: 0.0,
            ram: String::new(),
            volume: None,
            net: None,
            user: user_read(),
            pulse_counter: 0,
            battery: battery_read(),
            autohide: mk::settings::read("bar-autohide").as_deref() == Some("on"),
            hidden: false,
            reserve: std::env::args().any(|a| a == "--reserve" || a == "--reservieren"),
            last_poke: std::time::Instant::now(),
            edge_from: None,
            wake: mk::motion::Spring::new(1.0),
            menu_open: false,
            matrix_open: false,
            app_open: false,
            app_card: None,
            tray_items: tray_items_init.0,
            tray_command: tray_items_init.1,
            confirmation: None,
            film_start: None,
            spine: iced::window::Id::unique(),
            chips: mk::device::bar_chips(),
            left: mkw::fleet::Fleet::across(CHIP_SIDE, CHIP_GAP),
            right: mkw::fleet::Fleet::across(CHIP_SIDE, CHIP_GAP),
            center: mkw::fleet::Fleet::across(0, CHIP_GAP),
            panel_width: 0.0,
            flash: mkw::TapFlash::default(),
            dialog: None,
        };
        if app.chips {
            // Chips are not hidden by an auto-hide fold (no catch strip).
            app.autohide = false;
        }
        app.clock_read();
        app.pulse_read();
        app.measure_vibrant();
        let task = Task::batch([
            Task::done(Msg::NewLayerShell { settings: spine_settings(app.reserve), id: app.spine }),
            app.sync_chips(),
        ]);
        (app, task)
    }

    /// The spine's height task (menus grow it, a fold shrinks it).
    fn spine_size(&self, height: u32) -> Task<Msg> {
        Task::done(Msg::SizeChange { id: self.spine, size: (0, height) })
    }

    /// The spine height when nothing is open.
    fn spine_rest(&self) -> u32 {
        if self.chips {
            1
        } else {
            HEIGHT + mkw::panel::SHADOW_MARGIN as u32
        }
    }

    /// R209: lay the chips out as surfaces — left zone from the left edge,
    /// right zone from the right edge, the center zone as one chip the
    /// compositor centers. Unchanged plans keep their surfaces.
    fn sync_chips(&mut self) -> Task<Msg> {
        if !self.chips {
            return Task::none();
        }
        let plan = |list: Vec<Widget>| -> Vec<mkw::fleet::Spec<Pane>> {
            list.into_iter()
                .filter(|w| self.widget_build(*w).is_some())
                .map(|w| {
                    let width = self.chip_width(w);
                    mkw::fleet::Spec {
                        pane: Pane::Widget(w),
                        width,
                        height: HEIGHT,
                        cross: 10,
                        namespace: "matrix-chip",
                        keyboard: false,
                        sig: (1, width),
                    }
                })
                .collect()
        };
        let left_plan = plan(self.zones.0.clone());
        // Right-anchored: the fleet advances leftwards.
        let right_plan = plan(self.zones.2.iter().rev().copied().collect());
        let center_live: Vec<Widget> =
            self.zones.1.iter().copied().filter(|w| self.widget_build(*w).is_some()).collect();
        let center_plan = if center_live.is_empty() {
            Vec::new()
        } else {
            let width = center_live.iter().map(|w| self.chip_width(*w)).sum::<u32>()
                + (center_live.len() as u32 - 1) * mk::spacing::M as u32;
            vec![mkw::fleet::Spec {
                pane: Pane::Center,
                width,
                height: HEIGHT,
                cross: 10,
                namespace: "matrix-chip",
                keyboard: false,
                sig: (2, width),
            }]
        };
        use matrixkit_widgets::iced_layershell::reexport::Anchor;
        let open = |anchor: Anchor| {
            move |shell: mkw::fleet::Shell, id: iced::window::Id| {
                Task::done(Msg::NewLayerShell { settings: shell.layer_settings(anchor, bar_output()), id })
            }
        };
        let close = |id| Task::done(Msg::RemoveWindow(id));
        Task::batch([
            self.left.sync(left_plan, open(Anchor::Top | Anchor::Left), close),
            self.right.sync(right_plan, open(Anchor::Top | Anchor::Right), close),
            self.center.sync(center_plan, open(Anchor::Top), close),
        ])
    }

    /// How wide this widget's chip is. A layer surface states its size
    /// before anything is laid out, so the content is measured, not
    /// asked (dock R174). Rounded up to 8 px so a ticking clock or a new
    /// battery value rarely reopens the surface.
    fn chip_width(&self, w: Widget) -> u32 {
        let callout = mk::typo::CALLOUT.size;
        let text = |s: &str| mkw::text_advance(s, callout);
        // A family button: M padding inside, INSET air around it.
        let button = |content: f32| content + 2.0 * mk::spacing::M + 2.0 * INSET;
        // Plain text: M air on both sides.
        let plain = |content: f32| content + 2.0 * mk::spacing::M;
        let width = match w {
            Widget::Matrix | Widget::Bell | Widget::Central | Widget::Capture => {
                button(mk::font_size::LARGE)
            }
            Widget::User => button(text(&self.user)),
            Widget::Media => button(mk::font_size::LARGE + mk::spacing::S + text(&self.now_label)),
            Widget::Focus => plain(text(&self.focus_title())),
            Widget::Clock => {
                let mut t = text(&self.clock);
                if !self.date_val.is_empty() {
                    t += text(&format!("   {}", self.date_val));
                }
                plain(t)
            }
            Widget::Pulse => plain(text(&self.pulse_text())),
            Widget::Battery => {
                let percent = self.battery.map(|(p, _)| format!("{p} %")).unwrap_or_default();
                plain(callout + 3.0 + mk::spacing::XS + text(&percent))
            }
            Widget::Tray => {
                let n = self.tray_items.lock().map(|t| t.len()).unwrap_or(0) as f32;
                n * 28.0 + (n - 1.0).max(0.0) * mk::spacing::XXS + 2.0 * INSET
            }
        } + mkw::TEXT_SAFETY;
        (width / 8.0).ceil() as u32 * 8
    }

    /// The focused window's title as the bar shows it.
    fn focus_title(&self) -> String {
        let title = self.focus.clone().unwrap_or_else(|| String::from("Matrix"));
        // R209: never into the clock — a long title ends in «…».
        let max = mk::device::bar_title_chars();
        let mut short = mk::elide(&title, max);
        // As chips the title also yields to the centered chip: it may only
        // take what is left of the left half (Duo panel: 654 pt).
        if self.chips && self.panel_width > 0.0 && self.zones.0.contains(&Widget::Focus) {
            let center: f32 = self
                .zones
                .1
                .iter()
                .filter(|w| self.widget_build(**w).is_some())
                .map(|w| self.chip_width(*w) as f32)
                .sum();
            let others: f32 = self
                .zones
                .0
                .iter()
                .filter(|w| **w != Widget::Focus && self.widget_build(**w).is_some())
                .map(|w| self.chip_width(*w) as f32 + CHIP_GAP as f32)
                .sum();
            let right: f32 = self
                .zones
                .2
                .iter()
                .filter(|w| self.widget_build(**w).is_some())
                .map(|w| self.chip_width(*w) as f32 + CHIP_GAP as f32)
                .sum();
            // Where the left group must end: before the centered chip, or
            // (no center) before the right group.
            let limit = if center > 0.0 {
                self.panel_width / 2.0 - center / 2.0
            } else {
                self.panel_width - CHIP_SIDE as f32 - right
            };
            let room = limit - CHIP_SIDE as f32 - others
                - 2.0 * CHIP_GAP as f32 - 2.0 * mk::spacing::M - mkw::TEXT_SAFETY - 8.0;
            while short.chars().count() > 1
                && mkw::text_advance(&short, mk::typo::CALLOUT.size) > room
            {
                let mut cut: String = short.trim_end_matches('\u{2026}').chars().collect();
                cut.pop();
                short = format!("{}\u{2026}", cut.trim_end());
            }
        }
        short
    }

    /// The pulse line (CPU, RAM, sound, network).
    fn pulse_text(&self) -> String {
        let mut pulse: Vec<String> = vec![format!("CPU {:.0} %", self.cpu), self.ram.clone()];
        if let Some(v) = &self.volume {
            pulse.push(format!("Ton {v}"));
        }
        if let Some(n) = &self.net {
            pulse.push(n.clone());
        }
        pulse.join("  ·  ")
    }

    /// What tapping this widget's chip does (None = nothing to press).
    fn chip_action(&self, w: Widget) -> Option<Msg> {
        Some(match w {
            Widget::Matrix => if self.matrix_open { Msg::MenuClose } else { Msg::MatrixOpen },
            Widget::User => if self.menu_open { Msg::MenuClose } else { Msg::MenuOpen },
            Widget::Bell => Msg::BellClick,
            Widget::Central => Msg::CentralToggle,
            Widget::Capture => Msg::CapturePanel,
            Widget::Media => Msg::MediaPlayPause,
            // R210 duo grammar: the name chip holds the window's menu.
            Widget::Focus if self.chips && self.focus.is_some() => {
                Msg::AppMenuOpen
            }
            Widget::Focus | Widget::Clock | Widget::Pulse | Widget::Battery | Widget::Tray => return None,
        })
    }

    /// The bare face of a pressable chip — glyph and/or title, no button
    /// of its own (the chip is the button).
    fn chip_face(&self, w: Widget) -> Option<Element<'_, Msg>> {
        let p = self.palette;
        let ink = match (self.glass, self.vibrant) {
            (true, Some(v)) => v.0,
            _ => p.on_surface,
        };
        let glyph = |g: char| -> Element<'_, Msg> { mkw::symbol(g, mk::font_size::LARGE, ink) };
        Some(match w {
            Widget::Matrix => glyph(mkw::symbol::APPS),
            Widget::Bell => {
                let dnd = matches!(mk::settings::read("do-not-disturb").as_deref(), Some("on") | Some("an"));
                glyph(if dnd { mkw::symbol::VOLUME_OFF } else { mkw::symbol::NOTIFICATIONS })
            }
            Widget::Central => glyph(mkw::symbol::TUNE),
            Widget::Capture => glyph(mkw::symbol::IMAGE),
            Widget::User => mkw::txt(self.user.as_str(), mk::typo::CALLOUT, ink).into(),
            Widget::Focus => mkw::txt(self.focus_title(), mk::typo::CALLOUT, ink).into(),
            Widget::Media => {
                let np = self.now_playing.as_ref()?;
                row![
                    glyph(if np.playing { mkw::symbol::PAUSE } else { mkw::symbol::PLAY_ARROW }),
                    mkw::txt(self.now_label.as_str(), mk::typo::CALLOUT, ink),
                ]
                .spacing(mk::spacing::S)
                .align_y(iced::Alignment::Center)
                .into()
            }
            _ => return None,
        })
    }

    /// One chip surface: its widget centered in the glass body.
    fn chip_view(&self, pane: Pane) -> Element<'_, Msg> {
        let p = self.palette;
        // R209 (the owner): a pressable chip answers like a dock tile — the
        // whole chip is the button (mkw::chip_button), with buzz and glow.
        if let Pane::Widget(w) = pane {
            if let (Some(face), Some(_)) = (self.chip_face(w), self.chip_action(w)) {
                let mut glow = self.flash.glow(&w);
                // An open menu keeps its chip lit.
                if (w == Widget::User && self.menu_open) || (w == Widget::Matrix && self.matrix_open) || (w == Widget::Focus && self.app_open) {
                    glow = glow.max(0.5);
                }
                return mkw::chip_button(face, Msg::ChipTap(w), glow, p);
            }
        }
        let content: Element<'_, Msg> = match pane {
            Pane::Widget(w) => self.widget_build(w).unwrap_or_else(|| Space::new().into()),
            Pane::Center => self.zone_build(&self.zones.1),
        };
        let glass = self.glass;
        container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| {
                if glass {
                    container::Style::default()
                } else {
                    mkw::panel::pill(p, mk::CORNER_RADIUS, INSET)
                }
            })
            .into()
    }

    /// R111: measure the wallpaper luminance behind the bar strip and
    /// derive the vibrant ink pair. Decodes the wallpaper — only on
    /// init and theme/settings edges, never per pulse.
    fn measure_vibrant(&mut self) {
        if !self.glass {
            self.vibrant = None;
            return;
        }
        let (sw, _) = mkw::glass::screen_size();
        let rect = iced::Rectangle {
            x: INSET,
            y: 0.0,
            width: (sw - 2.0 * INSET).max(1.0),
            height: HEIGHT as f32,
        };
        self.vibrant = mkw::glass::region_luma(rect).map(mkw::glass::vibrant_ink);
    }

    fn clock_read(&mut self) {
        // Leitbild-Menueleisten-Uhr, empirisch (R33+R35): ShowDayOfWeek=1,
        // ShowDate=0, the reference ICUForce12HourTime=1 -> "So. 1:27 PM". German
        // locales often leave %p empty — then 24 h fallback.
        let am_pm = mk::command::first_line("date", &["+%p"])
            .map(|p| !p.trim().is_empty())
            .unwrap_or(false);
        let format = if am_pm { "+%a. %-I:%M %p" } else { "+%a. %H:%M" };
        if let Some(z) = mk::command::first_line("date", &[format]) {
            self.clock = z;
            self.date_val = String::new();
        }
    }

    fn pulse_read(&mut self) {
        if self.watcher.changed() {
            self.glass = glass_wanted();
            if let Some(new) = mk::Palette::load() {
                self.palette = new;
            }
            // R111: theme edge = the wallpaper likely changed too.
            self.measure_vibrant();
        }
        self.zones = widgets_read(); // the assignment lives — changing the file suffices
        // R119: ask the MPRIS broker (cheap property reads, one bus).
        self.now_playing = media::read();
        self.now_label = self.now_playing.as_ref().map_or_else(String::new, |np| {
            let base = if np.title.is_empty() { np.artist.clone() } else { np.title.clone() };
            mk::elide(&base, 25)
        });
        self.focus = focus_read();
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.cpu = self.sys.global_cpu_usage();
        self.ram = mk::format::bytes_decimal(self.sys.used_memory());
        self.volume = volume_read();
        // Network/battery/DMS change sluggishly — every 5th pulse (10 s) suffices.
        self.pulse_counter = self.pulse_counter.wrapping_add(1);
        if self.pulse_counter % 5 == 0 || self.net.is_none() {
            self.net = net_read();
            {
                // Charge chime (reference extract R33, PowerChime): it sounds on
                // PLUGGING IN — only on plugging in, never on unplugging.
                let new = battery_read();
                if let (Some((_, old_loading)), Some((_, new_loading))) = (self.battery, new) {
                    if !old_loading && new_loading {
                        std::thread::spawn(|| {
                            mk::feedback::now("geraete", "06-geraet-verbunden.wav");
                        });
                    }
                }
                self.battery = new;
            }
        }
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::FadeTick => {
                // Only pull the palette along — no process spawns,
                // no network/battery checks (the CPU lesson of 2026-07-08).
                if self.watcher.changed() {
                    self.glass = glass_wanted();
                    if let Some(p) = mk::Palette::load() {
                        self.palette = p;
                    }
                }
            }
            Msg::Clock => self.clock_read(),
            Msg::Focus => self.focus = focus_read(),
            Msg::EdgeTouch(at) => {
                self.last_poke = std::time::Instant::now();
                if self.hidden {
                    self.edge_from = Some(at.y);
                }
            }
            Msg::EdgeMove(at) => {
                self.last_poke = std::time::Instant::now();
                if self.hidden {
                    // Swipe DOWN from the top edge wakes the bar.
                    if let Some(from) = self.edge_from {
                        if at.y - from > 4.0 {
                            self.edge_from = None;
                            return self.wake_up();
                        }
                    }
                }
            }
            Msg::EdgeUp => {
                if self.hidden && self.edge_from.is_some() {
                    // A tap on the invisible strip wakes the bar too.
                    self.edge_from = None;
                    return self.wake_up();
                }
                self.edge_from = None;
            }
            Msg::WakeTick => {
                self.wake.tick(1.0 / 60.0);
            }
            Msg::Poke => {
                self.last_poke = std::time::Instant::now();
                if self.hidden {
                    return self.wake_up();
                }
            }
            Msg::Pulse => {
                // R168c: fold away after three still seconds.
                let wants = mk::settings::read("bar-autohide").as_deref() == Some("on");
                if wants != self.autohide {
                    self.autohide = wants;
                    self.hidden = false;
                    self.last_poke = std::time::Instant::now();
                    return self.bar_mass();
                }
                if self.hidden && Self::home_open() {
                    // The home screen appeared — the bar belongs to it.
                    return self.wake_up();
                }
                if self.autohide
                    && !self.hidden
                    && !self.menu_open
                    && !self.matrix_open
                    && self.last_poke.elapsed().as_secs_f32() > 3.0
                    && !Self::home_open()
                {
                    self.hidden = true;
                    // Keep the spring ticking past the resize: every
                    // tick paints a fresh EMPTY frame — without it the
                    // compositor keeps showing the top slice of the
                    // last full bar (the ghost stripe).
                    self.wake = mk::motion::Spring::snappy(1.0);
                    self.wake.retarget(0.0);
                    return self.bar_mass();
                }
                self.pulse_read();
                // R69: Aufnahme-Status von matrix-capture lesen.
                let status = std::env::var("XDG_RUNTIME_DIR")
                    .map(|d| format!("{d}/matrix-capture-recording"))
                    .ok()
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .and_then(|s| s.lines().nth(2).and_then(|z| z.parse::<u64>().ok()));
                self.film_start = status;
            }
            Msg::FilmStop => {
                mkw::panel::app_start_with("matrix-capture", &["recording-stop"]);
                self.film_start = None;
            }
            Msg::CapturePanel => {
                mkw::panel::close_other_popups("matrix-capture");
                mkw::panel::app_start_with("matrix-capture", &["panel"]);
                if self.menu_open || self.matrix_open {
                    return Task::done(Msg::MenuClose);
                }
            }
            Msg::CentralToggle => {
                mkw::panel::close_other_popups("matrix-central");
                // The Zentrale toggles itself (mkw::leiste_toggle).
                mkw::panel::app_start("matrix-central");
                if self.menu_open || self.matrix_open {
                    return Task::done(Msg::MenuClose);
                }
            }
            Msg::TrayClick(d, p) => {
                let _ = self.tray_command.send(tray::TrayCommand::Activate(d, p));
            }
            Msg::TrayRight(d, p) => {
                let _ = self.tray_command.send(tray::TrayCommand::Secondary(d, p));
            }
            Msg::BellClick => {
                mkw::panel::close_other_popups("matrix-notifications");
                // Toggle signal to the notification daemon (open history).
                let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
                let _ = std::fs::write(format!("{dir}/matrix-notifications-show"), "1");
                if self.menu_open || self.matrix_open {
                    return Task::done(Msg::MenuClose);
                }
            }
            Msg::MenuOpen => {
                mkw::panel::close_other_popups("");
                self.menu_open = true;
                self.matrix_open = false;
                self.app_open = false;
                self.confirmation = None;
                return self.spine_size(MENU_HEIGHT + mkw::panel::SHADOW_MARGIN as u32);
            }
            Msg::MatrixOpen => {
                mkw::panel::close_other_popups("");
                self.matrix_open = true;
                self.menu_open = false;
                self.app_open = false;
                self.confirmation = None;
                return self.spine_size(MENU_HEIGHT + mkw::panel::SHADOW_MARGIN as u32);
            }
            Msg::AppMenuOpen => {
                // R210 (the owner): the name chip opens a GLASS CARD for the
                // whole app — on the panel of its focused window.
                if self.app_card.is_some() {
                    return self.update(Msg::AppCardClose);
                }
                mkw::panel::close_other_popups("");
                let Some((window, app_id)) = mk::canvas::focused_app_window() else { return Task::none() };
                let windows = mk::canvas::app_windows(&app_id);
                let light = window.pid > 0 && mk::house::listens(window.pid);
                // Where it lies, in words (duo: the upper or lower panel).
                let place = match (mk::device::panels().is_some(), window.output.as_str()) {
                    (true, "DSI-1") => String::from("oberes Panel"),
                    (true, "DSI-2") => String::from("unteres Panel"),
                    (_, o) => o.to_string(),
                };
                let mut card = AppCard {
                    wide: false,
                    about: format!("{} · {}", if light { "Matrix-App" } else { "App" }, place),
                    crawl: light.then(|| mk::crawlers::state(window.pid)),
                    light,
                    pair: mk::canvas::spans_panels(&windows),
                    duo: mk::device::panels().is_some(),
                    title: self.focus.clone().unwrap_or_else(|| app_id.clone()),
                    window,
                    app_id,
                    veil: iced::window::Id::unique(),
                    card: iced::window::Id::unique(),
                };
                let output = iced_layershell::reexport::OutputOption::OutputName(card.window.output.clone());
                // R214 (the owner): the card has the window's size and place —
                // it lies over the window; unknown geometry: centered as before.
                // The card has no text field: the on-screen keyboard goes (it
                // lay over the card in the owner's photo, 02.11.2026).
                // An open keyboard had shrunk the window from below: close it
                // first and let the window grow back before measuring.
                if mk::runtime::path("matrix-keyboard-offen").exists() {
                    mk::onscreen_keyboard::send("close");
                    std::thread::sleep(std::time::Duration::from_millis(350));
                }
                // The bar knows what it keeps at the top: 10 px margin + its zone.
                let frame = mk::canvas::window_frame(card.window.id, Some(10 + reserved_zone()));
                // What the card was given (diagnosis, runtime file).
                mk::runtime::write("matrix-card-frame", &format!("window {} on {} → {:?}", card.window.id, card.window.output, frame));
                card.wide = frame.as_ref().is_some_and(|f| f.3 >= 560);
                let settings = match frame {
                    Some((out, x, y, w, h)) if w >= 200 && h >= 160 => {
                        mkw::glass_dialog::card_settings_at(iced_layershell::reexport::OutputOption::OutputName(out), x, y, w, h)
                    }
                    _ => {
                        let (w, h) = mkw::app_card_size(card.light, card.duo, card.crawl.is_some());
                        mkw::glass_dialog::card_settings_sized(output.clone(), w, h)
                    }
                };
                let (veil, id) = (card.veil, card.card);
                self.app_card = Some(card);
                self.app_open = true;
                return Task::batch([
                    Task::done(Msg::NewLayerShell { settings: mkw::glass_dialog::veil_settings(output.clone()), id: veil }),
                    Task::done(Msg::NewLayerShell { settings, id }),
                ]);
            }
            Msg::AppSettings => {
                if let Some(card) = &self.app_card {
                    if card.light {
                        mk::house::send(card.window.pid, "root");
                    }
                }
                return self.update(Msg::AppCardClose);
            }
            Msg::AppClose => {
                if let Some(card) = &self.app_card {
                    // Every window of the app (light ones close their way).
                    mk::canvas::close_app(&card.app_id, 0);
                }
                mk::feel::tap();
                return self.update(Msg::AppCardClose);
            }
            Msg::AppSwap => {
                if let Some(card) = &self.app_card {
                    let (app_id, window, pair) = (card.app_id.clone(), card.window.clone(), card.pair);
                    std::thread::spawn(move || {
                        if pair {
                            mk::canvas::swap_panels(&app_id);
                        } else {
                            mk::canvas::window_to_other_panel(&window);
                        }
                    });
                }
                mk::feel::tap();
                return self.update(Msg::AppCardClose);
            }
            Msg::AppCrawlSwitch | Msg::AppCrawlSpecies(_) => {
                // The card stays open: several species can be chosen.
                if let Some(card) = &mut self.app_card {
                    let Some(keys) = &mut card.crawl else { return Task::none() };
                    match msg {
                        Msg::AppCrawlSwitch if !keys.is_empty() => keys.clear(),
                        Msg::AppCrawlSwitch => *keys = mk::crawlers::SPECIES.iter().map(|(k, _, _)| k.to_string()).collect(),
                        Msg::AppCrawlSpecies(i) => {
                            let key = mk::crawlers::SPECIES[i.min(3)].0.to_string();
                            if let Some(at) = keys.iter().position(|k| *k == key) {
                                keys.remove(at);
                            } else {
                                keys.push(key);
                            }
                        }
                        _ => {}
                    }
                    mk::crawlers::ask(card.window.pid, keys);
                    mk::feel::tap();
                }
            }
            Msg::AppCardClose => {
                self.app_open = false;
                if let Some(card) = self.app_card.take() {
                    return Task::batch([Task::done(Msg::RemoveWindow(card.card)), Task::done(Msg::RemoveWindow(card.veil))]);
                }
            }
            Msg::MenuClose => {
                self.menu_open = false;
                self.matrix_open = false;
                self.app_open = false;
                self.confirmation = None;
                return self.spine_size(self.spine_rest());
            }
            Msg::App(id) => {
                // "name bereich" → write the wish file, then start (R41).
                let mut parts = id.splitn(2, ' ');
                let name = parts.next().unwrap_or(id);
                if let Some(area) = parts.next() {
                    let base = std::env::var("XDG_RUNTIME_DIR")
                        .unwrap_or_else(|_| String::from("/tmp"));
                    let _ = std::fs::write(
                        format!("{base}/{name}-area"),
                        area,
                    );
                }
                mkw::panel::app_start(name);
                self.menu_open = false;
                self.matrix_open = false;
                self.app_open = false;
                self.confirmation = None;
                return self.spine_size(self.spine_rest());
            }
            Msg::MediaPlayPause => {
                if let Some(np) = &self.now_playing {
                    media::play_pause(np.player.clone());
                }
            }
            Msg::MediaNext => {
                if let Some(np) = &self.now_playing {
                    media::next(np.player.clone());
                }
            }
            Msg::Action(a) if a.permanent() => {
                // R209: final things ask on glass, not by a red second click.
                self.menu_open = false;
                self.matrix_open = false;
                self.app_open = false;
                self.confirmation = None;
                let (veil, card) = (iced::window::Id::unique(), iced::window::Id::unique());
                self.dialog = Some((a, veil, card));
                let output = bar_output();
                return Task::batch([
                    self.spine_size(self.spine_rest()),
                    Task::done(Msg::NewLayerShell { settings: mkw::glass_dialog::veil_settings(output.clone()), id: veil }),
                    Task::done(Msg::NewLayerShell { settings: mkw::glass_dialog::card_settings(output), id: card }),
                ]);
            }
            Msg::DialogKey(yes) if self.dialog.is_some() => return self.update(Msg::DialogAnswer(yes)),
            Msg::DialogKey(false) if self.app_card.is_some() => return self.update(Msg::AppCardClose),
            Msg::DialogKey(_) => {}
            Msg::DialogAnswer(yes) => {
                if let Some((a, veil, card)) = self.dialog.take() {
                    if yes {
                        mk::feel::tap();
                        // The surfaces go first; the action may end the session.
                        return Task::batch([
                            Task::done(Msg::RemoveWindow(card)),
                            Task::done(Msg::RemoveWindow(veil)),
                        ])
                        .chain(Task::perform(async move { a.execute() }, |_| Msg::FadeTick));
                    }
                    return Task::batch([Task::done(Msg::RemoveWindow(card)), Task::done(Msg::RemoveWindow(veil))]);
                }
            }
            Msg::Action(a) => {
                if a.permanent() && self.confirmation != Some(a) {
                    self.confirmation = Some(a); // only the red second click counts
                } else {
                    a.execute();
                    self.menu_open = false;
                    self.matrix_open = false;
                self.app_open = false;
                    self.confirmation = None;
                    return self.spine_size(self.spine_rest());
                }
            }
            Msg::ChipTap(w) => {
                // The dock tile's answer (R209): a short buzz and a lit edge.
                self.flash.tap(w);
                if let Some(msg) = self.chip_action(w) {
                    return self.update(msg);
                }
            }
            Msg::FlashTick => {
                self.flash.tick();
            }
            Msg::Resized(window, width) => {
                if window == self.spine {
                    self.panel_width = width;
                }
            }
            // variants added by the to_layer_message macro
            _ => {}
        }
        Task::none()
    }


    /// R169b: the reference keeps bar and dock ON the home screen — an open matrix-home holds the fold.
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
    /// R168e: waking is a slide, not a pop — the bar floats down from
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
        self.bar_mass()
    }

    /// R168c: the bar's surface — folded to a strip when hidden,
    /// full height (menus included) when awake. An auto-hiding bar
    /// never reserves canvas space.
    fn bar_mass(&self) -> Task<Msg> {
        if self.hidden {
            return Task::batch([
                self.spine_size(STRIP),
                Task::done(Msg::MarginChange { id: self.spine, margin: (0, 0, 0, 0) }),
                Task::done(Msg::ExclusiveZoneChange { id: self.spine, zone_size: 0 }),
            ]);
        }
        let height = if self.menu_open || self.matrix_open {
            MENU_HEIGHT + mkw::panel::SHADOW_MARGIN as u32
        } else {
            self.spine_rest()
        };
        Task::batch([
            self.spine_size(height),
            Task::done(Msg::MarginChange { id: self.spine, margin: (10, 0, 0, 0) }),
            Task::done(Msg::ExclusiveZoneChange {
                id: self.spine,
                zone_size: if self.reserve && !self.autohide { reserved_zone() } else { 0 },
            }),
        ])
    }

    fn subscription(&self) -> Subscription<Msg> {
        let mut all = vec![
            // TimelineView(.everyMinute) extract: exactly on the minute boundary.
            mkw::tick_to_the_minute().map(|_| Msg::Clock),
            tick("puls", std::time::Duration::from_secs(2)).map(|_| Msg::Pulse),
            mkw::palette_fade_subscription().map(|_| Msg::FadeTick),
            // Focus changes arrive IMMEDIATELY via the event stream.
            mkw::canvas_stream().map(|_| Msg::Focus),
            // R168c: pointer/finger pokes feed the auto-hide clock and
            // the edge swipe that wakes a folded bar.
            iced::event::listen_with(|event, _status, _window| match event {
                iced::Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => {
                    Some(Msg::EdgeTouch(position))
                }
                iced::Event::Touch(iced::touch::Event::FingerMoved { position, .. }) => {
                    Some(Msg::EdgeMove(position))
                }
                iced::Event::Touch(
                    iced::touch::Event::FingerLifted { .. }
                    | iced::touch::Event::FingerLost { .. },
                ) => Some(Msg::EdgeUp),
                iced::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) => Some(Msg::Poke),
                iced::Event::Window(iced::window::Event::Resized(size)) => {
                    Some(Msg::Resized(_window, size.width))
                }
                ref other => mkw::glass_dialog::key(other).map(Msg::DialogKey),
            }),
        ];
        if self.flash.is_active() {
            all.push(tick("bar-flash", std::time::Duration::from_millis(60)).map(|_| Msg::FlashTick));
        }
        if !self.wake.is_settled() {
            all.push(tick("bar-wake", std::time::Duration::from_millis(16)).map(|_| Msg::WakeTick));
        }
        // Self-test hook: MATRIX_BAR_TEST_MENU=1 opens the menu after
        // 2 s — checks SizeChange/Stretch without a mouse (remote diagnosis).
        if !self.menu_open && std::env::var("MATRIX_BAR_TEST_MENU").is_ok() {
            all.push(tick("menuetest", std::time::Duration::from_secs(2)).map(|_| Msg::MenuOpen));
        }
        Subscription::batch(all)
    }

    /// Clickable bar element in the button tone of the bar family.
    /// Bar widgets carry the SidebarFamily look (MatrixUI, 2026-07-08).
    fn family<'a>(&'a self, glyph: Option<char>, title: Option<&'a str>, active: bool, msg: Msg) -> Element<'a, Msg> {
        // R60: ONE bar text size — HINWEIS, like clock/focus/pulse (before,
        // the user button stuck out by 1 px with FLIESS).
        mkw::ui::family_button(glyph, title, active, msg, self.palette, mk::typo::CALLOUT)
    }

    /// None = the widget has nothing to say on this device right now.
    fn widget_build(&self, w: Widget) -> Option<Element<'_, Msg>> {
        let p = self.palette;
        // R111 vibrant text: strip texts follow the measured backdrop
        // luminance while glass is on (menus keep the palette tones —
        // they sit on their own opaque surfaces).
        let (ink, ink2) = match (self.glass, self.vibrant) {
            (true, Some(v)) => v,
            _ => (p.on_surface, p.on_surface_variant),
        };
        Some(match w {
            // Apple-menu extract (round 28): the system menu at the far left —
            // always there, always the same, everything reachable from here.
            // Menu wandering (R37, 1984 grammar): if the OTHER menu is open,
            // hovering already switches — without a click.
            Widget::Matrix => {
                let button = self.family(
                    Some(mkw::symbol::APPS),
                    None,
                    self.matrix_open,
                    if self.matrix_open { Msg::MenuClose } else { Msg::MatrixOpen },
                );
                if self.menu_open {
                    mouse_area(button).on_enter(Msg::MatrixOpen).into()
                } else {
                    button
                }
            }
            Widget::Focus => mkw::txt(self.focus_title(), mk::typo::CALLOUT, ink).into(),
            Widget::Clock => {
                let mut z = row![mkw::txt(&self.clock, mk::typo::CALLOUT, ink)];
                if !self.date_val.is_empty() {
                    z = z.push(mkw::txt(
                        format!("   {}", self.date_val),
                        mk::typo::CALLOUT,
                        ink2,
                    ));
                }
                z.into()
            }
            Widget::Pulse => mkw::txt(self.pulse_text(), mk::typo::CALLOUT, ink2).into(),
            Widget::Tray => {
                let items = self.tray_items.lock().ok()?.clone();
                if items.is_empty() {
                    return None;
                }
                let mut line = row![].spacing(mk::spacing::XXS);
                for it in items {
                    let image: Element<'_, Msg> = match &it.icon {
                        Some((w, h, rgba)) => iced::widget::image(
                            iced::widget::image::Handle::from_rgba(*w, *h, rgba.clone()),
                        )
                        .width(20)
                        .height(20)
                        .into(),
                        None => mkw::txt(
                            it.title.chars().next().unwrap_or('•').to_string(),
                            mk::typo::CALLOUT,
                            ink,
                        )
                        .into(),
                    };
                    line = line.push(
                        iced::widget::mouse_area(mkw::magnifier(
                            button(image)
                                .padding(4)
                                .style(move |_, status| {
                                    mkw::panel::button_style(p, status, mk::radius::SMALL)
                                })
                                .on_press(Msg::TrayClick(it.service.clone(), it.path.clone())),
                        ))
                        .on_right_press(Msg::TrayRight(it.service.clone(), it.path.clone())),
                    );
                }
                line.into()
            }
            Widget::Bell => {
                // "Nicht stören" (mk setting) shows a muted bell.
                let dnd = matches!(mk::settings::read("do-not-disturb").as_deref(), Some("on") | Some("an"));
                let (glyph, color) = if dnd {
                    (mkw::symbol::VOLUME_OFF, p.on_surface_variant)
                } else {
                    (mkw::symbol::NOTIFICATIONS, p.on_surface)
                };
                let _ = color;
                self.family(Some(glyph), None, false, Msg::BellClick)
            }
            Widget::Battery => {
                let (percent, loading) = self.battery?;
                let glyph = if loading {
                    mkw::symbol::BATTERY_CHARGING
                } else {
                    mkw::symbol::BATTERY_FULL
                };
                mkw::tag(
                    glyph,
                    format!("{percent} %"),
                    mk::typo::CALLOUT,
                    ink2,
                )
            }
            Widget::Central => {
                self.family(Some(mkw::symbol::TUNE), None, false, Msg::CentralToggle)
            }
            Widget::Capture => {
                self.family(Some(mkw::symbol::IMAGE), None, false, Msg::CapturePanel)
            }
            Widget::Media => {
                // R119 Now Playing: hidden without a player (like the
                // tray without items); playing shows the pause glyph.
                let np = self.now_playing.as_ref()?;
                let glyph = if np.playing { mkw::symbol::PAUSE } else { mkw::symbol::PLAY_ARROW };
                let button = self.family(
                    Some(glyph),
                    if self.now_label.is_empty() { None } else { Some(self.now_label.as_str()) },
                    np.playing,
                    Msg::MediaPlayPause,
                );
                mouse_area(button).on_right_press(Msg::MediaNext).into()
            }
            Widget::User => {
                let button = self.family(
                    None,
                    Some(self.user.as_str()),
                    self.menu_open, // menu open = active pill like in the sidebar
                    if self.menu_open { Msg::MenuClose } else { Msg::MenuOpen },
                );
                // Menu wandering: crossing over from the open Matrix menu suffices.
                if self.matrix_open {
                    mouse_area(button).on_enter(Msg::MenuOpen).into()
                } else {
                    button
                }
            }
        })
    }

    fn zone_build(&self, widgets: &[Widget]) -> Element<'_, Msg> {
        let mut z = row![]
            .spacing(mk::spacing::M)
            .align_y(iced::Alignment::Center);
        for w in widgets {
            if let Some(el) = self.widget_build(*w) {
                z = z.push(el);
            }
        }
        z.into()
    }

    /// The session menu — a pill below the user widget.
    fn menu(&self) -> Element<'_, Msg> {
        let p = self.palette;
        // Head: the living avatar greets by name.
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
        // MatrixUI MenuFamily: the session menu in the ONE menu language.
        let mut entries: Vec<mkw::ui::MenuItem<Msg>> = Vec::new();
        for a in Action::ALLE {
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
                msg: Msg::Action(a),
            });
        }
        mkw::ui::menu_family(Some(header), entries, p)
    }

    /// The Matrix menu — the Apple-menu extract (round 28, read live on the
    /// reference system): groups split by separators, "…" = asks / opens
    /// something, without "…" = happens immediately, live badge on the
    /// update entry, and the logout entry carries the REAL name.
    fn matrix_menu(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let mut e: Vec<mkw::ui::MenuItem<Msg>> = vec![
            mkw::ui::MenuItem::Item {
                glyph: Some(mkw::symbol::INFO),
                title: String::from("Über diese Matrix"),
                color: None,
                msg: Msg::App("matrix-help"),
            },
            mkw::ui::MenuItem::Separator,
        ];
        // Systemeinstellungen + Softwareupdate — the badge lives (Apple
        // grammar: the menu itself carries status).
        e.push(mkw::ui::MenuItem::Item {
            glyph: Some(mkw::symbol::TUNE),
            title: String::from("Systemeinstellungen …"),
            color: None,
            msg: Msg::App("matrix-settings"),
        });
        match mk::badge::read("matrix-updater") {
            Some(n) => e.push(mkw::ui::MenuItem::ItemWithBadge {
                glyph: Some(mkw::symbol::RESTART),
                title: String::from("Softwareupdate …"),
                badge: if n == "1" { String::from("1 Update") } else { format!("{n} Updates") },
                color: None,
                msg: Msg::App("matrix-updater"),
            }),
            None => e.push(mkw::ui::MenuItem::Item {
                glyph: Some(mkw::symbol::RESTART),
                title: String::from("Softwareupdate …"),
                color: None,
                msg: Msg::App("matrix-updater"),
            }),
        }
        e.push(mkw::ui::MenuItem::Separator);
        e.push(mkw::ui::MenuItem::Item {
            glyph: Some(mkw::symbol::WIDGETS),
            title: String::from("Leiste & Dock anpassen …"),
            color: None,
            // Fusion R41: opens the settings in the Leiste & Dock section.
            msg: Msg::App("matrix-settings leisten"),
        });
        e.push(mkw::ui::MenuItem::Separator);
        // Energy actions (standby/restart/shutdown) live ONLY in the session
        // menu on the right — no duplication (user, 2026-07-12).
        e.push(mkw::ui::MenuItem::Item {
            glyph: Some(mkw::symbol::LOCK),
            title: String::from("Bildschirm sperren"),
            color: None,
            msg: Msg::Action(Action::Lock),
        });
        e.push(mkw::ui::MenuItem::Item {
            glyph: Some(mkw::symbol::LOGOUT),
            title: format!("{} abmelden …", self.user),
            color: None,
            msg: Msg::Action(Action::Logout),
        });
        mkw::ui::menu_family(None, e, p)
    }

    fn view(&self, window: iced::window::Id) -> Element<'_, Msg> {
        // R210: the app card — veil and glass card.
        if let Some(card) = &self.app_card {
            if window == card.veil {
                return mkw::glass_dialog::veil(Msg::AppCardClose);
            }
            if window == card.card {
                let duo = card.duo.then(|| mkw::AppCardDuo {
                    label: if card.pair { "Panels tauschen" } else { "Aufs andere Panel" },
                    hint: if card.pair { "Duo — die Fenster liegen auf beiden Panels" } else { "Duo — das Fenster wechselt das Panel" },
                    on_press: Msg::AppSwap,
                });
                let crawl = card.crawl.as_ref().map(|keys| mkw::AppCardCrawl {
                    on: !keys.is_empty(),
                    on_switch: Msg::AppCrawlSwitch,
                    species: mk::crawlers::SPECIES
                        .iter()
                        .enumerate()
                        .map(|(i, (key, name, diet))| (*name, *diet, mk::night_glass::NEON[i], keys.iter().any(|k| k == key), Msg::AppCrawlSpecies(i)))
                        .collect(),
                });
                let face = mkw::app_card(
                    mkw::AppCardFace { title: &card.title, about: &card.about, on_settings: card.light.then_some(Msg::AppSettings), duo, crawl, on_close: Msg::AppClose, wide: card.wide },
                    self.palette,
                );
                // The window may be lower than the card's rows: they scroll.
                let face: Element<'_, Msg> = iced::widget::scrollable(face).height(Length::Fill).into();
                return mkw::glass_dialog::card(face, self.glass, self.palette);
            }
        }
        // R209 glass dialog: veil and card.
        if let Some((a, veil, card)) = self.dialog {
            if window == veil {
                return mkw::glass_dialog::veil(Msg::DialogAnswer(false));
            }
            if window == card {
                let (title, note, action) = a.dialog();
                let face = mkw::confirmation_card(
                    title,
                    note.to_string(),
                    action,
                    Msg::DialogAnswer(true),
                    Msg::DialogAnswer(false),
                    self.palette,
                );
                return mkw::glass_dialog::card(face, self.glass, self.palette);
            }
        }
        // R209: a chip surface shows exactly its one widget.
        if let Some(pane) = self
            .left
            .pane(window)
            .or_else(|| self.right.pane(window))
            .or_else(|| self.center.pane(window))
        {
            return self.chip_view(*pane);
        }
        // As chips the spine is empty until a menu opens.
        if self.chips && !self.menu_open && !self.matrix_open {
            return container(Space::new().width(Length::Fill).height(Length::Fill)).into();
        }
        // R168d: folded, the bar draws NOTHING — the strip is an
        // invisible catch surface at the very edge.
        if self.hidden {
            return container(Space::new().width(Length::Fill).height(Length::Fill)).into();
        }
        // R168e: waking, the bar floats down from the top edge.
        let slide = (self.wake.value - 1.0) * (HEIGHT as f32 + 24.0);
        let p = self.palette;

        // R69: the red stop dot hangs FIXED on the right as soon as a
        // recording runs — independent of the widget config (like the reference).
        let mut right = row![].spacing(mk::spacing::M).align_y(iced::Alignment::Center);
        if let Some(start) = self.film_start {
            let sek = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().saturating_sub(start))
                .unwrap_or(0);
            right = right.push(mkw::magnifier(
                button(
                    row![
                        mkw::txt("\u{25cf}", mk::typo::CALLOUT, p.error),
                        mkw::txt(
                            format!("{}:{:02}", sek / 60, sek % 60),
                            mk::typo::CALLOUT,
                            p.on_surface,
                        ),
                    ]
                    .spacing(mk::spacing::XS)
                    .align_y(iced::Alignment::Center),
                )
                .padding([4, mk::spacing::S as u16])
                .on_press(Msg::FilmStop)
                .style(move |_, status| mkw::panel::button_style(p, status, mk::radius::SMALL)),
            ));
        }
        right = right.push(self.zone_build(&self.zones.2));
        let pages = row![
            self.zone_build(&self.zones.0),
            Space::new().width(Length::Fill),
            right,
        ]
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        let layer = stack![
            pages,
            container(self.zone_build(&self.zones.1))
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        ];

        // R50c (user, measured from screenshots): ONE inset all around.
        // Before, hover pills stuck to the top/bottom edge (0 px) with
        // 16 px left — now EINZUG air everywhere, button height capped
        // by it, and the bar rounding is exactly concentric:
        // CORNER_RADIUS + EINZUG. To the sides the surface keeps
        // SHADOW_MARGIN breathing room (pill lesson of 2026-07-08).
        // R108 shape family: under compositor glass the bar draws only
        // its content — body, shadow and rim come from the layer rule's
        // distance field, so there is exactly one silhouette author.
        let glass_active = self.glass;
        let bar_inner = container(layer)
            .padding(INSET)
            .width(Length::Fill)
            .height(Length::Fixed(HEIGHT as f32))
            .style(move |_| {
                if glass_active {
                    mkw::panel::glass_pill(p, mk::CORNER_RADIUS, INSET)
                } else {
                    mkw::panel::pill(p, mk::CORNER_RADIUS, INSET)
                }
            })
            .into();
        let dressed: Element<'_, Msg> = if glass_active {
            bar_inner
        } else {
            mkw::panel::shadow_layers(bar_inner, mk::CORNER_RADIUS + INSET)
        };
        let bar = container(dressed)
        // Side gap = top edge gap (10) — user wish for the same margin
        // all around. The shadow thus gets only 10 px of side air instead
        // of SHADOW_MARGIN; the blur core (offset 4) fits, only the
        // outermost gauss seam is clipped — invisible.
        .padding(iced::Padding {
            left: 10.0,
            right: 10.0,
            ..iced::Padding::ZERO
        });

        if !self.menu_open && !self.matrix_open {
            return bar.into();
        }

        // Open menu: bar on top, pill below — session menu right,
        // Matrix menu left (under its anchor, like the Apple menu).
        // Clicking the free surface closes (hence the surface grows).
        let (pill, page) = if self.matrix_open {
            (self.matrix_menu(), iced::alignment::Horizontal::Left)

        } else {
            (self.menu(), iced::alignment::Horizontal::Right)
        };
        let bottom = mouse_area(
            container(pill)
                .align_x(page)
                .padding([mk::spacing::S as u16, mk::spacing::M as u16])
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Msg::MenuClose);

        // R209: as chips the strip itself is drawn by the chips — the spine
        // only keeps its room (a tap there closes the menu).
        let top: Element<'_, Msg> = if self.chips {
            mouse_area(Space::new().width(Length::Fill).height(Length::Fixed(HEIGHT as f32)))
                .on_press(Msg::MenuClose)
                .into()
        } else {
            bar.into()
        };
        mkw::slide_in(column![top, bottom], slide)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zones_parse_standard() {
        let (l, m, r) = zones_parse("matrix fokus | uhr | akku puls tray glocke zentrale nutzer");
        assert_eq!(l, vec![Widget::Matrix, Widget::Focus]);
        assert_eq!(m, vec![Widget::Clock]);
        assert_eq!(
            r,
            vec![
                Widget::Battery,
                Widget::Pulse,
                Widget::Tray,
                Widget::Bell,
                Widget::Central,
                Widget::User
            ]
        );
    }

    #[test]
    fn zones_parse_robust() {
        // Unknowns drop out, missing zones stay empty, no panic.
        let (l, m, r) = zones_parse("quatsch fokus");
        assert_eq!(l, vec![Widget::Focus]);
        assert!(m.is_empty() && r.is_empty());
        let (l, _, _) = zones_parse("");
        assert!(l.is_empty());
    }

    #[test]
    fn large_capitalized() {
        assert_eq!(large("nutzer"), "Nutzer");
        assert_eq!(large(""), "");
    }
}


/// The top bar's app card (R210): which app, which window, what it offers.
struct AppCard {
    window: mk::canvas::AppWindow,
    app_id: String,
    title: String,
    /// «Matrix-App · oberes Panel».
    about: String,
    /// R211: what crawls in it (None: not a house app — no Krabbler).
    crawl: Option<Vec<String>>,
    /// The window is wide: the card in two columns.
    wide: bool,
    /// A light-shell window (mailbox): it has settings (root pane).
    light: bool,
    /// The app's windows lie on both panels.
    pair: bool,
    /// The device has panels (duo section shown).
    duo: bool,
    veil: iced::window::Id,
    card: iced::window::Id,
}
