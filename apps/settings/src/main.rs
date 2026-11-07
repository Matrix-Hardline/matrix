//! Matrix Einstellungen — the system settings in MatrixKit style.
//!
//! Eighth MatrixKit app and seed of roadmap stage 3 (the
//! replace the DankMaterialShell settings with our own). Also shows
//! the controls extracted in round 6 from the reference System Settings in real
//! real use: sliders, segmented choice, stepper, collapsible group.
//! Every setting writes a file under ~/.config/matrix/ that the rest
//! of the system can read — the app is only the surface above it.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, container, Space};
use matrixkit_widgets::iced::{Element, Length, Subscription, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use mkw::color;
use std::time::Duration;

mod input_devices;
mod language;
mod dev;
mod sounds;
mod rights;
mod update;
mod overview;
mod connections;
mod laptops;
mod canvas;
mod bars;
mod hardline;
/// R212: the house version (no iced) — the default since 02.11.2026.
mod licht;

const APP_ID: &str = "matrix-settings";

/// Sections of the fused settings (R41/42): overview (dashboard,
/// start page), general, canvas, wallpaper, bar & dock, sounds,
/// developer.
const AREAS: usize = 15;
/// The Hardline section (last sidebar entry); hidden on duo devices (R209).
const HARDLINE_AREA: usize = 13;

/// Title per section — the toolbar tells you where you are.
const AREA_TITLE: [&str; AREAS] = [
    "Übersicht", "Allgemein", "Softwareupdate", "Verbindungen", "Leinwand",
    "Hintergrund", "Leiste & Dock", "Ton", "Bedienung", "Entwickler",
    "Sprache & Region", "Benutzer und Rechte", "Apps und Rechte",
    "Hardline", "Anzeige",
];

/// Keywords per section for the sidebar search (reference grammar:
/// the search finds sections through their contents too).
const KEYWORDS: [&str; AREAS] = [
    "dashboard monitor system info prozessor cpu ram arbeitsspeicher speicher akku batterie netzwerk ip kernel temperatur",
    "erscheinung hell dunkel modus name rechnername uhr zeit format endung dateiendung erweitert",
    "update updates updater softwareupdate aktualisieren version image neustart quelle registry",
    "wlan wifi netz netzwerk internet funk bluetooth geraete kopfhoerer maus tastatur verbinden hotspot",
    "geist modi navigation maus scrollen klebe wisch touch",
    "wallpaper bild bilder hintergrundbild hell dunkel galerie",
    "bar dock topbar widgets anpassen wackeln pins zeile zwischenablage",
    "klaenge toene ton sound audio gong hoerprobe stumm master anmeldung fehler ausgabe eingabe mikrofon lautsprecher geraet pegel test lautstaerke hdmi kopfhoerer",
    "bedienung eingabe controller gamepad xbox playstation pad tastatur maus trackpad zeiger scrollen klicken",
    "dev developer ssh netz zugang schluessel update partner freigabe",
    "sprache language english deutsch region format zahl datum uhrzeit tastatur layout keyboard schweiz international übersetzung",
    "benutzer konto konten nutzer account uid admin administrator wheel rechte passwort",
    "apps rechte berechtigungen permissions netzwerk freigaben register erteilt",
    "hardline identitaet identität schluessel schlüssel adresse anker widerruf netz knoten petname regal",
    "anzeige bildschirm grösse skalierung schrift text helligkeit erkennen monitor display",
];

/// Section wish from outside (bar/dock/context menu): file in the
/// runtime directory, read and consumed.
/// The part the last wish asked for (usize::MAX = none), see `area_wish`.
static WISH_PART: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(usize::MAX);

/// Take the part of the last wish once.
fn wish_part() -> Option<usize> {
    let p = WISH_PART.swap(usize::MAX, std::sync::atomic::Ordering::Relaxed);
    (p != usize::MAX).then_some(p)
}

fn area_wish() -> Option<usize> {
    let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
    // R105: matrix-settings-area is the protocol; -bereich stays readable
    // for one transition (an old bar/dock may still write it mid-upgrade).
    let base = std::path::PathBuf::from(base);
    let path = base.join("matrix-settings-area");
    let legacy = base.join("matrix-settings-bereich");
    let wish = std::fs::read_to_string(&path)
        .or_else(|_| std::fs::read_to_string(&legacy))
        .ok()?;
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&legacy);
    // Round 78: "display:5" also opens a part of a group (Display › Sleep).
    let (wish, part) = match wish.trim().split_once(':') {
        Some((w, p)) => (w.to_string(), p.trim().parse().ok()),
        None => (wish.trim().to_string(), None),
    };
    WISH_PART.store(part.unwrap_or(usize::MAX), std::sync::atomic::Ordering::Relaxed);
    match wish.as_str() {
        "overview" | "uebersicht" | "dashboard" => Some(0),
        "general" | "allgemein" => Some(1),
        "update" | "updater" | "softwareupdate" => Some(2),
        "connections" | "verbindungen" | "wlan" | "bluetooth" | "netz" => Some(3),
        "canvas" | "leinwand" => Some(4),
        "wallpaper" | "hintergrund" => Some(5),
        "bars" | "customize" | "leisten" | "anpassen" => Some(6),
        "sounds" | "sound" | "klaenge" | "klänge" | "ton" => Some(7),
        "controls" | "bedienung" | "controller" | "tastatur" | "maus" | "trackpad" => Some(8),
        "dev" | "developer" | "entwickler" | "entwicklerzugang" => Some(9),
        "language" | "sprache" | "region" | "sprache-region" => Some(10),
        "users" | "benutzer" => Some(11),
        "apps" | "app-rechte" => Some(12),
        "hardline" => Some(13),
        // Round 70: the new area «Anzeige» (display).
        "display" | "anzeige" | "bildschirm" | "bildschirme" | "helligkeit" => Some(14),
        "sleep" | "ruhezustand" | "tiefschlaf" => {
            WISH_PART.store(5, std::sync::atomic::Ordering::Relaxed);
            Some(14)
        }
        _ => None,
    }
}

fn main() -> iced::Result {
    // R212 (owner 02.11.2026: «Settings without iced»): the house is
    // the default. `--iced` opens the previous version — as its own
    // instance (another app id), so a house window does not swallow it;
    // the house opens it for areas it does not carry yet.
    if std::env::args().any(|a| a == "--iced") {
        return mkw::app::window::<App>("matrix-settings-iced", 940.0, 680.0);
    }
    match licht::run() {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("[settings] Fensterhaus-Fehler ({e}) — Rückfall auf iced");
            mkw::app::window::<App>(APP_ID, 940.0, 680.0)
        }
    }
}

impl mkw::app::Program for App {
    type Msg = Msg;
    fn new() -> (Self, Task<Msg>) {
        App::new()
    }
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        App::update(self, msg)
    }
    fn view(&self) -> Element<'_, Msg> {
        App::view(self)
    }
    fn title(&self) -> String {
            match AREA_TITLE.get(self.area) {
                Some(t) if self.area != 0 => String::from(*t),
                _ => String::from("Matrix Einstellungen"),
            }
        }
    fn subscription(&self) -> iced::Subscription<Msg> {
        App::subscription(self)
    }
}

/// Icon style choice — matches ~/.config/matrix/icon-stil.
const ICON_STYLES: [&str; 2] = ["Standard", "Getönt"];
/// R102: the symbol sets — one font file each, identical codepoints.
const SYMBOL_SETS: [&str; 2] = ["Rund", "Eckig"];

/// Appearance (reference System Settings > Appearance): Automatic
/// follows the real sun position (theme-sonne timer), light/dark switch
/// firmly and pause the rhythm.
const APPEARANCES: [&str; 3] = ["Automatisch (Sonne)", "Hell", "Dunkel"];

/// Canvas mode (user canvas concept, 7.7.2026): Classic = tray
/// minimize; canvas = windows stay in place, the − light lays the
/// privacy veil over the content. Free 2D panning comes with the
/// compositor buildout (roadmap).
const CANVAS_MODES: [&str; 2] = ["Klassisch", "Unendliche Leinwand"];

/// Corner radius range: MINI (4) to beyond LARGE (24) — the scale
/// from mk::radius, in steps of 2.
const RADIUS_MIN: u8 = mk::radius::MINI as u8;
const RADIUS_MAX: u8 = 24;

/// Profile of the system — read once at start (reference "About this Mac").
struct Profile {
    os: String,
    kernel: String,
    host: String,
    memory: String,
}

impl Profile {
    fn read() -> Self {
        let os = std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("PRETTY_NAME="))
                    .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
            })
            .unwrap_or_else(|| String::from("Matrix"));
        let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let host = std::fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let memory = std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| {
                s.lines().find(|l| l.starts_with("MemTotal:")).and_then(|l| {
                    l.split_whitespace().nth(1).and_then(|kb| kb.parse::<u64>().ok())
                })
            })
            .map(|kb| mk::format::bytes_decimal(kb * 1024))
            .unwrap_or_default();
        Self { os, kernel, host, memory }
    }
}

struct App {
    frame: mkw::Frame,
    /// 0 General · 1 Canvas · 2 Wallpaper · 3 Bar & Dock.
    area: usize,
    /// Visited sections — the ◀ ▶ chevrons of the toolbar family.
    history: Vec<usize>,
    history_pos: usize,
    search: String,
    /// R207/R209 phone form (duo panel, narrow window): the section list is
    /// open — one column, no sidebar; a section opens over the whole width.
    menu_open: bool,
    overview: overview::Panel,
    connections: connections::Panel,
    canvas: canvas::Panel,
    bars: bars::Panel,
    sounds: sounds::Panel,
    /// Softwareupdate section (R89) — the fused Matrix Updater.
    software: update::Panel,
    /// Input section (R92) — controller, keyboard, mouse & trackpad.
    input_devices: input_devices::Panel,
    language: language::Panel,
    dev: dev::Panel,
    hardline: hardline::Hardline,
    profile: Profile,
    /// Appearance
    appearance: usize,
    canvas_mode_val: usize,
    icon_style: usize,
    /// Symbol set (R102): 0 = Rounded, 1 = Sharp — a font swap re-skins
    /// every glyph in the OS, applies to freshly started apps.
    symbol_set: usize,
    motion_reduced: bool,
    glass: bool,
    /// Display
    scaling: f32,
    corner_radius: u8,
    /// Index in mk::typo::SIZES (dynamicTypeSize extract).
    text_scale: usize,
    /// Collapsible "Erweitert" (advanced) group
    advanced_open: bool,
    show_debug_overlay: bool,
    /// short feedback in the footer
    saved: Option<String>,
    /// Undo stack (reference UndoManager, round 13): Ctrl+Z restores
    /// the latest slider change.
    undo: mk::undo::Stack<Change>,
}

/// What can be undone — the old value BEFORE the change.
#[derive(Debug, Clone)]
enum Change {
    Scaling(f32),
    CornerRadius(u8),
}

#[derive(Debug, Clone)]
enum Msg {
    Frame(mkw::FrameMsg),
    Tick,
    Area(usize),
    Back,
    Forward,
    /// Phone form: back from a section to the section list.
    Menu,
    Search(String),
    SearchClear,
    Overview(overview::Msg),
    Connections(connections::Msg),
    Canvas(canvas::Msg),
    Bars(bars::Msg),
    Sounds(sounds::Msg),
    Update(update::Msg),
    Controls(input_devices::Msg),
    Language(language::Msg),
    Dev(dev::Msg),
    Hardline(hardline::Msg),
    Appearance(&'static str),
    CanvasMode(&'static str),
    IconStyle(usize),
    SymbolSet(usize),
    MotionToggle(bool),
    GlassToggle(bool),
    Scaling(f32),
    RadiusMinus,
    RadiusPlus,
    TextSmaller,
    TextLarger,
    AdvancedToggle,
    DebugToggle(bool),
    Key(mkw::Key),
}

/// Settings culture: everything runs via mk::einstellung (AppStorage analog).
/// R106: the glass toggle owns ~/.config/niri/glass.kdl — rule present
/// means the compositor refracts behind the dock, empty file means off.
/// niri watches its includes, so the change lands without a re-login.
fn write_glass_rule(on: bool) {
    let home = std::env::var("HOME").unwrap_or_default();
    let path = std::path::PathBuf::from(home).join(".config/niri/glass.kdl");
    let rule = if on {
        concat!(
            "// Frost strength: 1 Kawase pass keeps the backdrop readable — motion\n",
            "// behind the glass stays visible as displacement (user wish 22.07).\n",
            "blur {\n",
            "    passes 1\n",
            "    offset 2\n",
            "}\n",
            "\n",
            "// R106/R108: liquid glass rules — the compositor derives body, shadow\n",
            "// and rim from ONE distance field per surface. All knobs live-reload.\n",
            "// This file is the toggle: Einstellungen -> Erscheinungsbild rewrites\n",
            "// it (rules present = glass on, empty file = off).\n",
            "layer-rule {\n",
            "    match namespace=\"^matrix-dock$\"\n",
            "    geometry-corner-radius 19\n",
            "    background-effect {\n",
            "        xray false\n",
            "        blur true\n",
            "        glass true\n",
            "        glass-inset-x 40\n",
            "        glass-inset-y 40\n",
            "        glass-height 120\n",
            "        glass-band 16\n",
            "        glass-bend 20\n",
            "        glass-scrim 0.35\n",
            "        glass-shadow 0\n",
            "        glass-shadow-spread 14\n",
            "    }\n",
            "}\n",
            "\n",
            "layer-rule {\n",
            "    match namespace=\"^matrix-bar$\"\n",
            "    geometry-corner-radius 16\n",
            "    background-effect {\n",
            "        xray false\n",
            "        blur true\n",
            "        glass true\n",
            "        glass-inset-x 10\n",
            "        glass-inset-y 0\n",
            "        glass-height 44\n",
            "        glass-anchor-top true\n",
            "        glass-band 12\n",
            "        glass-bend 14\n",
            "        glass-scrim 0.35\n",
            "        glass-shadow 0\n",
            "        glass-shadow-spread 12\n",
            "    }\n",
            "}\n",
            "\n",
            "// R125: every popup CARD (Zentrale, context menus) is its own\n",
            "// exactly-sized window — inset 0 and height 0 make the whole\n",
            "// surface the glass body. union 0: cards never melt into the\n",
            "// bar or dock when they open next to them.\n",
            "layer-rule {\n",
            "    match namespace=\"^matrix-card$\"\n",
            "    geometry-corner-radius 22\n",
            "    background-effect {\n",
            "        xray false\n",
            "        blur true\n",
            "        glass true\n",
            "        glass-inset-x 0\n",
            "        glass-inset-y 0\n",
            "        glass-band 14\n",
            "        glass-bend 16\n",
            "        glass-scrim 0.35\n",
            "        glass-shadow 0\n",
            "        glass-shadow-spread 12\n",
            "        glass-union 0\n",
            "    }\n",
            "}\n",
            "\n",
            "// R127: island TILES carry a transparent gutter on both sides\n",
            "// (the minus badges float there) — the body insets to the tile.\n",
            "layer-rule {\n",
            "    match namespace=\"^matrix-tile$\"\n",
            "    geometry-corner-radius 22\n",
            "    background-effect {\n",
            "        xray false\n",
            "        blur true\n",
            "        glass true\n",
            "        glass-inset-x 30\n",
            "        glass-inset-y 0\n",
            "        glass-band 14\n",
            "        glass-bend 16\n",
            "        glass-scrim 0.35\n",
            "        glass-shadow 0\n",
            "        glass-shadow-spread 12\n",
            "        glass-union 0\n",
            "    }\n",
            "}\n",
        )
    } else {
        "// Written by Matrix Einstellungen: Liquid Glass is off.\n"
    };
    // 02.11.2026 (Duo): «Liquid Glass off and on — doesn't seem to
    // work». ON wrote the built-in copy above (4 rules), which
    // erased the device's own rules (the duo's bar chips, shelf, hardtext).
    // Now: OFF keeps the device's rules aside, ON brings exactly them back;
    // the built-in copy is only the last resort.
    let keep = path.with_extension("kdl.an");
    let is_rules = |p: &std::path::Path| std::fs::read_to_string(p).is_ok_and(|t| t.contains("layer-rule"));
    if !on {
        if is_rules(&path) {
            let _ = std::fs::copy(&path, &keep);
        }
        write_atomic(&path, rule);
        return;
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [keep.clone(), std::path::PathBuf::from("/etc/skel/.config/niri/glass.kdl"), std::path::PathBuf::from(format!("{home}/.config/matrix/glass.kdl"))];
    if let Some(saved) = candidates.iter().find(|c| is_rules(c)).and_then(|c| std::fs::read_to_string(c).ok()) {
        write_atomic(&path, &saved);
        return;
    }
    if !is_rules(&path) {
        write_atomic(&path, rule);
    }
}

/// Write whole or not at all — niri live-reloads its includes and must
/// never read half a file.
fn write_atomic(path: &std::path::Path, text: &str) {
    let tmp = path.with_extension("kdl.neu");
    if std::fs::write(&tmp, text).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

fn conf_write(name: &str, value: &str) {
    mk::settings::write(name, value);
}

fn conf_read(name: &str) -> Option<String> {
    mk::settings::read(name)
}

/// Run a command quietly (theme switch, timer) — best effort.
/// Round 178: through the kit's reaped spawn — the dropped `Child` left one zombie per switch.
fn silent(cmd: &str, args: &[&str]) {
    let mut c = std::process::Command::new(cmd);
    c.args(args).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    mk::process::spawn_reaped(&mut c);
}

/// Re-renders the app icons (reads icon-stil itself) — the icon style
/// switch thus takes visible effect immediately in dock/launcher.
fn icons_rerender() {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        format!("{home}/.local/bin/matrixkit-icons"),
        String::from("/usr/local/bin/matrixkit-icons"),
        String::from("/usr/bin/matrixkit-icons"),
    ];
    for k in candidates {
        if std::path::Path::new(&k).exists() {
            let mut c = std::process::Command::new(&k);
            c.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
            mk::process::spawn_reaped(&mut c);
            return;
        }
    }
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        // Adopt existing settings from disk.
        let icon_style = match conf_read("icon-style").as_deref() {
            Some("tinted") | Some("getoent") => 1,
            _ => 0,
        };
        let symbol_set = match conf_read("symbol-set").as_deref() {
            Some("sharp") => 1,
            _ => 0,
        };
        let appearance = match conf_read("appearance").as_deref() {
            Some("light") | Some("hell") => 1,
            Some("dark") | Some("dunkel") => 2,
            _ => 0,
        };
        let motion_reduced = matches!(conf_read("motion").as_deref(), Some("reduced") | Some("reduziert"));
        let glass = conf_read("glass").as_deref() != Some("off");
        // The desktop-mode picker shows the REALITY of the session.
        let canvas_mode_val_real = if mkw::session_is_canvas() { 1 } else { 0 };
        conf_write("canvas-mode", if canvas_mode_val_real == 1 { "canvas" } else { "classic" });
        // Migration 2026-07-19: the key used to have another name — clear legacy.
        if let Ok(home) = std::env::var("HOME") {
            let _ = std::fs::remove_file(format!("{home}/.config/matrix/desktop-modus"));
        }
        let scaling = conf_read("scaling")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0)
            .clamp(0.8, 1.5);
        let text_scale = mk::settings::read("text-size")
            .and_then(|w| mk::typo::SIZES.iter().position(|s| *s == w))
            .unwrap_or(1);
        let corner_radius = conf_read("corner-radius")
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(mk::radius::NORMAL as u8)
            .clamp(RADIUS_MIN, RADIUS_MAX);
        let start_area = std::env::args()
            .nth(1)
            .and_then(|a| match a.as_str() {
                "overview" | "uebersicht" | "dashboard" => Some(0),
                "update" | "updater" | "softwareupdate" => Some(2),
                "verbindungen" | "wlan" | "bluetooth" => Some(3),
                "canvas" | "leinwand" => Some(4),
                "wallpaper" | "hintergrund" => Some(5),
                "bars" | "customize" | "leisten" | "anpassen" => Some(6),
                "klaenge" => Some(7),
                "controls" | "bedienung" | "controller" | "tastatur" | "maus" | "trackpad" => Some(8),
                "dev" | "entwickler" => Some(9),
                _ => None,
            })
            .or_else(area_wish)
            .unwrap_or(0);
        let (canvas_panel, canvas_task) = canvas::Panel::new();
        let (dev_panel, dev_task) = dev::Panel::new();
        let (connections_panel, connections_task) = connections::Panel::new();
        let (software_panel, software_task) = update::Panel::new();
        (
            Self {
                frame: mkw::Frame::new(APP_ID, &[mk::permissions::Permission::Network]),
                area: start_area.min(AREAS - 1),
                overview: overview::Panel::new(),
                connections: connections_panel,
                canvas: canvas_panel,
                bars: bars::Panel::new(),
                sounds: sounds::Panel::new(),
                software: software_panel,
                input_devices: input_devices::Panel::new(),
                language: language::Panel::new(),
                dev: dev_panel,
                hardline: hardline::Hardline::new(),
                profile: Profile::read(),
                appearance,
                canvas_mode_val: canvas_mode_val_real,
                icon_style,
                symbol_set,
                motion_reduced,
                glass,
                scaling,
                corner_radius,
                text_scale,
                advanced_open: false,
                show_debug_overlay: false,
                saved: None,
                history: vec![start_area.min(AREAS - 1)],
                history_pos: 0,
                search: String::new(),
                menu_open: true,
                undo: mk::undo::Stack::new(),
            },
            Task::batch([
                canvas_task.map(Msg::Canvas),
                dev_task.map(Msg::Dev),
                connections_task.map(Msg::Connections),
                software_task.map(Msg::Update),
            ]),
        )
    }

    /// Section switch with history (like folder changes in Dateien):
    /// everything behind the current position expires, the target is appended.
    fn go_to_area(&mut self, b: usize) {
        let b = b.min(AREAS - 1);
        if b == self.area {
            return;
        }
        self.history.truncate(self.history_pos + 1);
        self.history.push(b);
        self.history_pos = self.history.len() - 1;
        self.area = b;
        self.search.clear();
    }

    fn receipt(&mut self, text: &str) {
        self.saved = Some(format!("{text} gespeichert \u{2713}"));
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Frame(m) => return self.frame.update(m).map(Msg::Frame),
            Msg::Area(b) => {
                self.menu_open = false;
                self.go_to_area(b);
            }
            Msg::Menu => self.menu_open = true,
            Msg::Back => {
                if self.history_pos > 0 {
                    self.history_pos -= 1;
                    self.area = self.history[self.history_pos];
                }
            }
            Msg::Forward => {
                if self.history_pos + 1 < self.history.len() {
                    self.history_pos += 1;
                    self.area = self.history[self.history_pos];
                }
            }
            Msg::Search(s) => self.search = s,
            Msg::SearchClear => self.search.clear(),
            Msg::Overview(m) => return self.overview.update(m).map(Msg::Overview),
            Msg::Connections(m) => return self.connections.update(m).map(Msg::Connections),
            Msg::Canvas(m) => return self.canvas.update(m).map(Msg::Canvas),
            Msg::Bars(m) => return self.bars.update(m).map(Msg::Bars),
            Msg::Sounds(m) => return self.sounds.update(m).map(Msg::Sounds),
            Msg::Update(m) => return self.software.update(m).map(Msg::Update),
            Msg::Controls(m) => return self.input_devices.update(m).map(Msg::Controls),
            Msg::Language(m) => return self.language.update(m).map(Msg::Language),
            Msg::Dev(m) => return self.dev.update(m).map(Msg::Dev),
            Msg::Hardline(m) => self.hardline.update(m),
            Msg::Tick => {
                self.frame.palette_changed();
                self.saved = None;
                let p = self.frame.palette;
                self.overview.tick(p);
                self.connections.tick(p);
                self.canvas.tick(p);
                self.bars.tick(p);
                self.sounds.tick(p);
                self.software.tick(p);
                self.input_devices.tick(p);
                self.language.tick(p);
                self.dev.tick(p);
                // Bar/dock/context wish for a section? (wish file)
                if let Some(b) = area_wish() {
                    self.go_to_area(b);
                }
            }
            Msg::Appearance(choice) => {
                self.appearance = APPEARANCES.iter().position(|e| *e == choice).unwrap_or(0);
                match self.appearance {
                    1 => {
                        conf_write("appearance", "light");
                        // Fixed mode: pause the solar rhythm (R112
                        // names first, pre-R112 names as legacy).
                        silent("systemctl", &["--user", "stop", "theme-sun.timer", "theme-sun-light.timer", "theme-sun-dark.timer"]);
                        silent("systemctl", &["--user", "stop", "theme-sonne.timer", "theme-sonne-light.timer", "theme-sonne-dark.timer"]);
                        mk::light_set(true);
                        self.saved = Some(String::from("Erscheinung: Hell \u{2713} — Sonnen-Rhythmus pausiert"));
                    }
                    2 => {
                        conf_write("appearance", "dark");
                        silent("systemctl", &["--user", "stop", "theme-sun.timer", "theme-sun-light.timer", "theme-sun-dark.timer"]);
                        silent("systemctl", &["--user", "stop", "theme-sonne.timer", "theme-sonne-light.timer", "theme-sonne-dark.timer"]);
                        mk::light_set(false);
                        self.saved = Some(String::from("Erscheinung: Dunkel \u{2713} — Sonnen-Rhythmus pausiert"));
                    }
                    _ => {
                        conf_write("appearance", "automatisch");
                        // Re-activate the rhythm + immediately apply the correct
                        // mode for NOW (theme-sonne computes itself)
                        silent("systemctl", &["--user", "start", "theme-sun.timer"]);
                        silent("systemctl", &["--user", "start", "theme-sun.service"]);
                        silent("systemctl", &["--user", "start", "theme-sonne.timer"]);
                        silent("systemctl", &["--user", "start", "theme-sonne.service"]);
                        self.saved = Some(String::from("Erscheinung: Automatisch \u{2713} — folgt der Sonne"));
                    }
                }
            }
            Msg::CanvasMode(choice) => {
                self.canvas_mode_val = CANVAS_MODES.iter().position(|m| *m == choice).unwrap_or(0);
                conf_write("canvas-mode", if self.canvas_mode_val == 1 { "canvas" } else { "classic" });
                // Honest receipt: the desktop mode IS the session —
                // switching happens at login, not via a toggle.
                let real_canvas = mkw::session_is_canvas();
                self.saved = Some(if (self.canvas_mode_val == 1) == real_canvas {
                    if real_canvas {
                        String::from("Unendliche Leinwand ist aktiv \u{2713}")
                    } else {
                        String::from("Klassischer Modus ist aktiv \u{2713}")
                    }
                } else if self.canvas_mode_val == 1 {
                    String::from("Vorgemerkt — beim n\u{e4}chsten Anmelden im Login \u{201e}Matrix Leinwand\u{201c} w\u{e4}hlen")
                } else {
                    String::from("Vorgemerkt — beim n\u{e4}chsten Anmelden im Login \u{201e}Niri\u{201c} w\u{e4}hlen")
                });
            }
            Msg::IconStyle(i) => {
                self.icon_style = i;
                conf_write("icon-style", if i == 1 { "tinted" } else { "standard" });
                icons_rerender();
                self.saved =
                    Some(String::from("Icon-Stil gespeichert \u{2713} — Icons werden neu gezeichnet"));
            }
            Msg::SymbolSet(i) => {
                self.symbol_set = i;
                conf_write("symbol-set", if i == 1 { "sharp" } else { "rounded" });
                self.saved = Some(String::from(
                    "Symbolstil gespeichert \u{2713} — gilt für neu gestartete Apps",
                ));
            }
            Msg::MotionToggle(on) => {
                self.motion_reduced = on;
                conf_write("motion", if on { "reduced" } else { "full" });
                self.receipt("Bewegung");
            }
            Msg::GlassToggle(on) => {
                self.glass = on;
                conf_write("glass", if on { "on" } else { "off" });
                write_glass_rule(on);
                self.receipt("Liquid Glass");
            }
            Msg::Scaling(v) => {
                let new = (v * 20.0).round() / 20.0;
                if (new - self.scaling).abs() > f32::EPSILON {
                    self.undo.remember("Anzeigegröße", Change::Scaling(self.scaling));
                }
                self.scaling = new; // 0.05 grid
                conf_write("scaling", &format!("{:.2}", self.scaling));
                self.receipt("Anzeigegröße");
            }
            Msg::RadiusMinus => {
                self.undo.remember("Eckenradius", Change::CornerRadius(self.corner_radius));
                self.corner_radius = self.corner_radius.saturating_sub(2).max(RADIUS_MIN);
                conf_write("corner-radius", &self.corner_radius.to_string());
                self.receipt("Eckenradius");
            }
            Msg::RadiusPlus => {
                self.undo.remember("Eckenradius", Change::CornerRadius(self.corner_radius));
                self.corner_radius = (self.corner_radius + 2).min(RADIUS_MAX);
                conf_write("corner-radius", &self.corner_radius.to_string());
                self.receipt("Eckenradius");
            }
            Msg::TextSmaller => {
                self.text_scale = self.text_scale.saturating_sub(1);
                mk::settings::write("text-size", mk::typo::SIZES[self.text_scale]);
                self.receipt("Textgröße");
            }
            Msg::TextLarger => {
                self.text_scale = (self.text_scale + 1).min(mk::typo::SIZES.len() - 1);
                mk::settings::write("text-size", mk::typo::SIZES[self.text_scale]);
                self.receipt("Textgröße");
            }
            Msg::Key(t) => {
                if self.frame.handle_key(t) {
                    return Task::none();
                }
                if t == mkw::Key::Refresh && self.area == 2 {
                    return self.software.refresh_button().map(Msg::Update);
                }
                match t {
                    // Ctrl+Z (reference UndoManager): revert the latest change.
                    mkw::Key::Undo => match self.undo.back() {
                        Some((name, Change::Scaling(v))) => {
                            self.scaling = v;
                            conf_write("scaling", &format!("{v:.2}"));
                            self.saved =
                                Some(format!("{name} wiederhergestellt \u{21a9}"));
                        }
                        Some((name, Change::CornerRadius(v))) => {
                            self.corner_radius = v;
                            conf_write("corner-radius", &v.to_string());
                            self.saved =
                                Some(format!("{name} wiederhergestellt \u{21a9}"));
                        }
                        None => {}
                    },
                    // Ctrl+R (reference round 14): the active section reloads —
                    // dev and connections gather their status asynchronously.
                    mkw::Key::Refresh => {
                        return match self.area {
                            2 => self
                                .connections
                                .update(connections::Msg::Reload)
                                .map(Msg::Connections),
                            7 => self.dev.update(dev::Msg::Reload).map(Msg::Dev),
                            _ => Task::none(),
                        };
                    }
                    _ => {}
                }
            }
            Msg::AdvancedToggle => self.advanced_open = !self.advanced_open,
            Msg::DebugToggle(on) => {
                self.show_debug_overlay = on;
                conf_write("debug-overlay", if on { "1" } else { "0" });
                self.receipt("Debug-Overlay");
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        Subscription::batch([
            self.frame.subscription().map(Msg::Frame),
            mkw::tick("settings", Duration::from_secs(3)).map(|_| Msg::Tick),
            mkw::key_subscription(Msg::Key),
            self.bars.subscription().map(Msg::Bars),
            self.sounds.subscription().map(Msg::Sounds),
            self.software.subscription().map(Msg::Update),
        ])
    }

    fn view(&self) -> Element<'_, Msg> {
        let p = self.frame.palette;

        // ── Appearance ────────────────────────────────────────────────
        let appearance = mkw::section(
            "Erscheinungsbild",
            vec![
                mkw::row_menu(
                    "Erscheinung",
                    Some("Hell, Dunkel — oder automatisch mit dem Sonnenstand."),
                    &APPEARANCES,
                    Some(APPEARANCES[self.appearance.min(2)]),
                    Msg::Appearance,
                    p,
                ),
                mkw::row_item(
                    "Icon-Stil",
                    Some("Farbige oder auf die Palette getönte App-Symbole."),
                    None,
                    Some(container(mkw::segments(&ICON_STYLES, self.icon_style, Msg::IconStyle, p))
                        .width(Length::Fixed(180.0))
                        .into()),
                    p,
                ),
                mkw::row_item(
                    "Symbolstil",
                    Some("Rund oder eckig — die Formsprache aller Systemsymbole. Gilt für neu gestartete Apps."),
                    None,
                    Some(container(mkw::segments(&SYMBOL_SETS, self.symbol_set, Msg::SymbolSet, p))
                        .width(Length::Fixed(180.0))
                        .into()),
                    p,
                ),
                mkw::row_toggle(
                    "Bewegung reduzieren",
                    Some("Federn und Übergänge auf das Nötigste beschränken."),
                    None,
                    self.motion_reduced,
                    p,
                    Some(Msg::MotionToggle(!self.motion_reduced)),
                ),
                mkw::row_toggle(
                    "Liquid Glass",
                    Some("Dock als echtes Glas: Fenster dahinter werden live gebrochen. Wirkt sofort."),
                    None,
                    self.glass,
                    p,
                    Some(Msg::GlassToggle(!self.glass)),
                ),
            ],
            p,
        );

        // ── Display ───────────────────────────────────────────────────
        let display = mkw::section(
            "Anzeige",
            vec![
                mkw::row_item(
                    "Anzeigegröße",
                    Some("Schrift und Bedienelemente — Vormerkung für den Shell-Ausbau."),
                    None,
                    Some(
                        iced::widget::row![
                            container(mkw::slider(0.8..=1.5, self.scaling, 0.05, Msg::Scaling, p))
                                .width(Length::Fixed(150.0)),
                            mkw::txt(
                                format!("{:.0} %", self.scaling * 100.0),
                                mk::typo::CAPTION,
                                p.on_surface_variant,
                            )
                            .width(Length::Fixed(44.0))
                            .align_x(iced::alignment::Horizontal::Right),
                        ]
                        .spacing(mk::spacing::S)
                        .align_y(iced::Alignment::Center)
                        .into(),
                    ),
                    p,
                ),
                mkw::row_item(
                    "Eckenradius",
                    Some("Rundung der Fenster und Karten — Vormerkung für den Shell-Ausbau."),
                    None,
                    Some(mkw::stepper(
                        format!("{} px", self.corner_radius),
                        (self.corner_radius > RADIUS_MIN).then_some(Msg::RadiusMinus),
                        (self.corner_radius < RADIUS_MAX).then_some(Msg::RadiusPlus),
                        p,
                    )),
                    p,
                ),
                mkw::row_item(
                    "Textgröße",
                    Some("Skaliert alle Matrix-Texte und -Symbole — sofort, überall."),
                    None,
                    Some(mkw::stepper(
                        match mk::typo::SIZES[self.text_scale] {
                            "klein" => String::from("Klein"),
                            "gross" => String::from("Groß"),
                            "sehr-gross" => String::from("Sehr groß"),
                            _ => String::from("Normal"),
                        },
                        (self.text_scale > 0).then_some(Msg::TextSmaller),
                        (self.text_scale + 1 < mk::typo::SIZES.len()).then_some(Msg::TextLarger),
                        p,
                    )),
                    p,
                ),
            ],
            p,
        );

        // ── Advanced (collapsible) ────────────────────────────────────
        let advanced_content = column![mkw::row_toggle(
            "Debug-Overlay anzeigen",
            Some("Blendet Bildrate und Ebenen-Grenzen ein."),
            None,
            self.show_debug_overlay,
            p,
            Some(Msg::DebugToggle(!self.show_debug_overlay)),
        )]
        .into();
        let advanced = container(mkw::expand(
            "Erweitert",
            self.advanced_open,
            Msg::AdvancedToggle,
            advanced_content,
            p,
        ))
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(color(p.surface_container_high).into()),
            border: iced::Border { radius: mk::CORNER_RADIUS.into(), ..Default::default() },
            shadow: mkw::elevation::card(),
            ..Default::default()
        });

        // ── Canvas: user doctrine 2026-07-19 — it is NEVER called
        // "Desktop"; the name is Leinwand.
        let desk = mkw::section(
            "Leinwand",
            vec![mkw::row_menu(
                "Modus",
                Some("Leinwand: Fenster bleiben an Ort — die \u{2212}-Ampel legt den Privatschleier über den Inhalt statt abzulegen."),
                &CANVAS_MODES,
                Some(CANVAS_MODES[self.canvas_mode_val.min(1)]),
                Msg::CanvasMode,
                p,
            )],
            p,
        );

        // ── About this system (reference "About this Mac") ─────────────
        let about = mkw::section(
            "Über dieses System",
            vec![
                mkw::row_value("Betriebssystem", None, &self.profile.os, p),
                mkw::row_value("Rechner", None, &self.profile.host, p),
                mkw::row_value("Kernel", None, &self.profile.kernel, p),
                mkw::row_value("Arbeitsspeicher", None, &self.profile.memory, p),
            ],
            p,
        );

        let list = column![appearance, display, desk, about, advanced]
            .spacing(mk::spacing::L);

        // Fusion R41: the SidebarFamily anatomy — seven sections, ONE app.
        let all_items = vec![
            mkw::SidebarItem { glyph: mkw::symbol::MONITORING, title: "Übersicht", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::TUNE, title: "Allgemein", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::RESTART, title: "Softwareupdate", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::WIFI, title: "Verbindungen", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::TOUCH_APP, title: "Leinwand", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::PALETTE, title: "Hintergrund", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::WIDGETS, title: "Leiste & Dock", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::VOLUME_UP, title: "Ton", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::SPORTS_ESPORTS, title: "Bedienung", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::CODE, title: "Entwickler", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::PUBLIC, title: "Sprache & Region", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::PERSON, title: "Benutzer und Rechte", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::SHIELD, title: "Apps und Rechte", count: None },
            mkw::SidebarItem { glyph: mkw::symbol::KEY, title: "Hardline", count: None },
        ];
        // Search (toolbar family): filters the sidebar like the reference
        // system settings — title AND keywords per section.
        let s_lower = self.search.to_lowercase();
        // R209: a duo device carries no Hardline (owner, 01.11.2026).
        let shown = |i: usize| mk::device::carries_hardline() || i != HARDLINE_AREA;
        let (items, image): (Vec<_>, Vec<usize>) = if s_lower.is_empty() {
            all_items.into_iter().enumerate().filter(|(i, _)| shown(*i)).map(|(i, item)| (item, i)).unzip()
        } else {
            let mut hits = Vec::new();
            let mut hit_idx = Vec::new();
            for (i, item) in all_items.into_iter().enumerate() {
                let hit = shown(i)
                    && (item.title.to_lowercase().contains(&s_lower) || KEYWORDS[i].contains(&s_lower));
                if hit {
                    hits.push(item);
                    hit_idx.push(i);
                }
            }
            (hits, hit_idx)
        };
        let active_visible = image
            .iter()
            .position(|&b| b == self.area)
            .unwrap_or(usize::MAX);
        let choice_image = image.clone();

        // R207/R209: the form follows the width (duo panel = phone).
        let width = self.frame.size.width;
        let phone = mk::form::of(width) == mk::form::Form::Phone;
        let active_content: Element<'_, Msg> = match self.area {
            0 => self.overview.view(width).map(Msg::Overview),
            2 => self.software.view().map(Msg::Update),
            3 => self.connections.view().map(Msg::Connections),
            4 => self.canvas.canvas_view().map(Msg::Canvas),
            5 => self
                .canvas
                .wallpaper_view(self.frame.size.width)
                .map(Msg::Canvas),
            6 => self.bars.view().map(Msg::Bars),
            7 => self.sounds.view().map(Msg::Sounds),
            8 => self.input_devices.view().map(Msg::Controls),
            9 => self.dev.view().map(Msg::Dev),
            10 => self.language.view().map(Msg::Language),
            11 => rights::users_view(p),
            12 => rights::apps_view(p),
            13 => self.hardline.view(p).map(Msg::Hardline),
            _ => list.into(),
        };
        let footer_text = match self.area {
            0 => self.overview.footer_text(),
            2 => self.software.footer_text(),
            3 => self.connections.footer_text(),
            4 | 5 => self.canvas.footer_text().unwrap_or_else(|| {
                String::from("Änderungen wirken sofort in der Leinwand — gespeichert in ~/.config/matrix/")
            }),
            6 => String::from("Änderungen greifen sofort — Bar und Dock lesen ihre Belegung laufend neu."),
            7 => self.sounds.footer_text(),
            8 => self.input_devices.footer_text(),
            9 => String::from("Zugang & Netz für die Matrix-Entwicklung — Passwörter tippst immer du selbst"),
            13 => self.hardline.footer_text(),
            10 => self.language.footer_text(),
            11 => String::from("Konten liest das System aus /etc/passwd — Änderungen bleiben Handarbeit mit Passwort"),
            12 => String::from("Das bindende Rechte-Register aller MatrixKit-Apps — Vergabe geschieht in der App selbst"),
            _ => self
                .saved
                .clone()
                .unwrap_or_else(|| String::from("Jede Änderung landet sofort in ~/.config/matrix/")),
        };

        // Toolbar family (from Matrix Dateien): ◀ ▶ title … search. In the
        // phone form ◀ leads back to the section list.
        let back = mkw::ui::phone_back(width, self.menu_open, Msg::Menu, (self.history_pos > 0).then_some(Msg::Back));
        let panel = mkw::ui::toolbar(
            String::from(AREA_TITLE[self.area.min(AREAS - 1)]),
            back,
            (!phone && self.history_pos + 1 < self.history.len()).then_some(Msg::Forward),
            Vec::new(),
            Vec::new(),
            &self.search,
            Msg::Search,
            Msg::SearchClear,
            p,
        );
        let detail: Element<'_, Msg> = column![
            panel,
            Space::new().height(mk::spacing::S),
            self.frame.scroll_area(active_content, Msg::Frame),
            mkw::footer(footer_text, p),
        ]
        .spacing(0)
        .into();
        let card = mkw::ui::sidebar_family_form(
            width,
            self.menu_open,
            items,
            active_visible,
            None,
            move |i| Msg::Area(choice_image[i]),
            detail,
            p,
        );

        self.frame.window(
            "Matrix Einstellungen",
            env!("CARGO_PKG_VERSION"),
            "Die System-Einstellungen im MatrixKit-Stil — Allgemein, Leinwand, Hintergrund, Leiste & Dock. Jede Änderung sofort auf dem Datenträger.",
            card.into(),
            Msg::Frame,
        )
    }
}
