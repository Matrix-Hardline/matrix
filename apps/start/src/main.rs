//! Matrix Start — the search palette (Super+Space opens it above the
//! screen center). Typing searches through EVERYTHING:
//!   • installed apps (from the .desktop files),
//!   • OPEN windows — Enter drives the canvas camera there (focus-window),
//!   • system actions (lock, toggle appearance, settings,
//!     log out, restart, power off),
//!   • a calculator (input like "12*8+4" → "= 100", Enter copies the result).
//! ↑/↓ selects, Enter triggers, Esc closes. A second call closes (toggle).

use matrixkit_widgets::iced;
use matrixkit_widgets::iced_layershell;
use matrixkit_widgets::iced::widget::{column, container, image, mouse_area, row, Space};
use matrixkit_widgets::iced::{Color, Element, Font, Length, Subscription, Task};
use matrixkit_widgets::iced_layershell::to_layer_message;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use mkw::color;

const WIDTH: u32 = 620;
const HEIGHT: u32 = 520;

const ICON: f32 = 32.0;

mod licht;

fn main() -> Result<(), iced_layershell::Error> {
    // Lager 23 of the iced replacement: --erstlicht is the window house
    // path of the palette (shadow doctrine).
    if std::env::args().nth(1).as_deref() == Some("--erstlicht") {
        // The guard applies to BOTH paths. It sat only in the
        // iced branch, so the light path lacked the open/close gesture: a second
        // key press would have opened a SECOND panel instead of closing
        // the first one
        // (finger probe 16.08.2026).
        if !mkw::panel_toggle() {
            return Ok(());
        }
        if let Err(e) = licht::run() {
            eprintln!("[start] Fensterhaus-Fehler: {e}");
        }
        return Ok(());
    }
    matrixkit_widgets::present_mailbox();
    use matrixkit_widgets::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
    use matrixkit_widgets::iced_layershell::settings::{LayerShellSettings, Settings};

    // Toggle: if the palette already ran, it is closed now — so are we.
    if !mkw::panel_toggle() {
        return Ok(());
    }

    // R209 (Duo, the owner): the app menu IS glass — the whole surface is a
    // `matrix-card` (compositor glass), stretched over the dock's panel
    // with a window's air (gap at the edges, the dock clear below).
    let glass_menu = mk::device::shelf().apps_leads;
    let gap = mk::device::WINDOW_GAP as i32;
    iced_layershell::application(
        App::new,
        move || String::from(if glass_menu && mkw::glass::wanted() { "matrix-card" } else { "matrix-start" }),
        App::update,
        App::view,
    )
    .subscription(App::subscription)
    .style(|_state, _theme| iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    })
    .settings(Settings {
        layer_settings: LayerShellSettings {
            // R103: fullscreen dismiss overlay — a click anywhere outside
            // the palette closes it (uniform popup grammar).
            size: Some((0, 0)),
            anchor: Anchor::Top | Anchor::Right | Anchor::Bottom | Anchor::Left,
            margin: if glass_menu {
                (gap, gap, mk::device::dock_clearance() as i32, gap)
            } else {
                (0, 0, 0, 0)
            },
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            exclusive_zone: -1,
            // R209 (Duo): open where the dock is (the owner).
            start_mode: mkw::panel::target("dock-output", mk::device::DUO_DOCK_PANEL),
            ..Default::default()
        },
        default_font: Font::with_name("Inter Variable"),
        fonts: mkw::load_symbol_fonts(),
        ..Default::default()
    })
    .run()
}

// ----------------------------------------------------------- App index

/// A launchable app from a .desktop file.
#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) app_id: String,
    pub(crate) name: String,
    pub(crate) exec: String,
    terminal: bool,
    /// Icon= field of the desktop entry — theme name or absolute path.
    /// Visible for the first-light path: it uses it to look for the same
    /// file as the iced gate (finger probe 16.08.2026).
    pub(crate) icon: Option<String>,
}

pub(crate) fn index_build() -> Vec<Entry> {
    let home = std::env::var("HOME").unwrap_or_default();
    // The kit's one list (lowest priority first, R215).
    let sources = matrixkit_theme::apps::data_dirs();
    let mut after_id: std::collections::HashMap<String, Entry> = std::collections::HashMap::new();
    for q in &sources {
        let Ok(rd) = std::fs::read_dir(q) else { continue };
        for e in rd.flatten() {
            let path = e.path();
            if path.extension().and_then(|x| x.to_str()) != Some("desktop") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if let Some(entry) = desktop_read(&path, stem) {
                after_id.insert(stem.to_string(), entry);
            } else {
                after_id.remove(stem);
            }
        }
    }
    // R48: ~/Programme — an AppImage in the folder = installed, like the
    // reference's /Applications. No .desktop needed: the folder IS the register.
    let prog = format!("{home}/Programme");
    if let Ok(rd) = std::fs::read_dir(&prog) {
        for e in rd.flatten() {
            let path = e.path();
            let extension_ok = path
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x.eq_ignore_ascii_case("appimage"));
            if !extension_ok {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let id = format!("programm-{}", stem.to_lowercase());
            after_id.entry(id.clone()).or_insert(Entry {
                app_id: id,
                name: stem.to_string(),
                exec: format!("\"{}\"", path.display()),
                terminal: false,
                icon: None,
            });
        }
    }
    let mut list: Vec<Entry> = after_id.into_values().collect();
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    list
}

fn desktop_read(path: &std::path::Path, stem: &str) -> Option<Entry> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut im_entry = false;
    let (mut name, mut exec, mut kind) = (None, None, None);
    let mut local: [Option<String>; 2] = [None, None];
    let mut icon = None;
    let (mut hidden, mut terminal) = (false, false);
    for line in content.lines() {
        let z = line.trim();
        if z.starts_with('[') {
            im_entry = z == "[Desktop Entry]";
            continue;
        }
        if !im_entry {
            continue;
        }
        let Some((k, v)) = z.split_once('=') else { continue };
        match k.trim() {
            "Name" => name = Some(v.trim().to_string()),
            key if mk::language::desktop_name_key(key).is_some() => {
                let slot = mk::language::desktop_name_key(key).unwrap_or(0);
                local[slot].get_or_insert_with(|| v.trim().to_string());
            }
            "Exec" => exec = Some(v.trim().to_string()),
            "Icon" => icon = Some(v.trim().to_string()),
            "Type" => kind = Some(v.trim().to_string()),
            "NoDisplay" | "Hidden" if v.trim() == "true" => hidden = true,
            "Terminal" if v.trim() == "true" => terminal = true,
            _ => {}
        }
    }
    if hidden || kind.as_deref() != Some("Application") {
        return None;
    }
    Some(Entry {
        app_id: stem.to_string(),
        name: { let [de, en] = local; mk::language::desktop_name(name, de, en)? },
        exec: exec?,
        terminal,
        icon,
    })
}

fn exec_cleanup(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|w| !w.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn app_start(e: &Entry) {
    let command = exec_cleanup(&e.exec);
    let _ = if e.terminal {
        std::process::Command::new("foot").args(["-e", "sh", "-c", &command]).spawn()
    } else {
        std::process::Command::new("sh").args(["-c", &command]).spawn()
    };
}

/// Word start beats substring, name start beats everything.
pub(crate) fn items(name: &str, search: &str) -> Option<u32> {
    let n = name.to_lowercase();
    let s = search.to_lowercase();
    if s.is_empty() {
        return Some(1);
    }
    if n.starts_with(&s) {
        return Some(100);
    }
    if n.split_whitespace().any(|w| w.starts_with(&s)) {
        return Some(50);
    }
    n.contains(&s).then_some(10)
}

// ----------------------------------------------------------- Open windows

#[derive(Debug, Clone)]
pub(crate) struct WindowInfo {
    pub(crate) id: u64,
    pub(crate) name: String, // app-friendly name (from the index) or app_id
    pub(crate) title: String,
}

/// Fetch open windows from niri (JSON). Empty list if niri is missing.
pub(crate) fn window_fetch(apps: &[Entry]) -> Vec<WindowInfo> {
    let Some(out) = mk::canvas::raw(&["msg", "-j", "windows"]) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else {
        return Vec::new();
    };
    let Some(arr) = v.as_array() else { return Vec::new() };
    arr.iter()
        .filter_map(|w| {
            let id = w["id"].as_u64()?;
            let app_id = w["app_id"].as_str().unwrap_or("");
            let title = w["title"].as_str().unwrap_or("").to_string();
            // Friendly name: ask the app index for the app_id.
            let name = apps
                .iter()
                .find(|e| e.app_id == app_id)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| {
                    if app_id.is_empty() { title.clone() } else { app_id.to_string() }
                });
            Some(WindowInfo { id, name, title })
        })
        .collect()
}

// ----------------------------------------------------------- Calculator

/// Tiny expression calculator (+ - * / x, parens, decimal). None if the
/// input does not look like a complete calculation (then no hit).
pub(crate) fn compute(input: &str) -> Option<String> {
    let s = input.trim();
    // Must contain a digit AND an operator — otherwise it is text.
    if !s.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    if !s.contains(['+', '-', '*', '/', 'x', '×', '·']) {
        return None;
    }
    let glyph: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
    let mut p = Parser { z: &glyph, i: 0 };
    let value = p.expression()?;
    if p.i != glyph.len() {
        return None; // leftover rest = no clean expression
    }
    if !value.is_finite() {
        return None;
    }
    // Pretty formatting: whole numbers without decimals, else up to 6 digits.
    let g = if (value.fract()).abs() < 1e-9 {
        format!("{}", value.round() as i64)
    } else {
        let t = format!("{value:.6}");
        t.trim_end_matches('0').trim_end_matches('.').to_string()
    };
    Some(g)
}

struct Parser<'a> {
    z: &'a [char],
    i: usize,
}
impl Parser<'_> {
    fn expression(&mut self) -> Option<f64> {
        let mut a = self.term()?;
        while let Some(&c) = self.z.get(self.i) {
            if c == '+' || c == '-' {
                self.i += 1;
                let b = self.term()?;
                a = if c == '+' { a + b } else { a - b };
            } else {
                break;
            }
        }
        Some(a)
    }
    fn term(&mut self) -> Option<f64> {
        let mut a = self.factor()?;
        while let Some(&c) = self.z.get(self.i) {
            if c == '*' || c == 'x' || c == '×' || c == '·' {
                self.i += 1;
                a *= self.factor()?;
            } else if c == '/' {
                self.i += 1;
                let b = self.factor()?;
                if b == 0.0 {
                    return None;
                }
                a /= b;
            } else {
                break;
            }
        }
        Some(a)
    }
    fn factor(&mut self) -> Option<f64> {
        match self.z.get(self.i) {
            Some('(') => {
                self.i += 1;
                let v = self.expression()?;
                if self.z.get(self.i) == Some(&')') {
                    self.i += 1;
                    Some(v)
                } else {
                    None
                }
            }
            Some('-') => {
                self.i += 1;
                Some(-self.factor()?)
            }
            Some(c) if c.is_ascii_digit() || *c == '.' => {
                let start = self.i;
                while matches!(self.z.get(self.i), Some(d) if d.is_ascii_digit() || *d == '.') {
                    self.i += 1;
                }
                let s: String = self.z[start..self.i].iter().collect();
                s.parse::<f64>().ok()
            }
            _ => None,
        }
    }
}

// ----------------------------------------------------------- Hits

/// What a hit does when triggered.
#[derive(Debug, Clone)]
pub(crate) enum Target {
    App(usize),
    /// R155: a file hit from the living index (matrix-seeker).
    File(String),
    /// R156: an intent from the registry — argv, already resolved.
    Intent(Vec<String>),
    Window(u64),
    Command { cmd: String, terminal: bool },
    Appearance,
    Logout,
    Copy(String),
}

#[derive(Debug, Clone)]
enum Icon {
    App(String),
    Symbol(char),
}

/// A resolved app icon: Matrix apps render live from the palette,
/// foreign apps come from the icon theme (PNG or SVG).
#[derive(Debug, Clone)]
enum LoadedIcon {
    Raster(image::Handle),
    Vector(iced::widget::svg::Handle),
}

/// R103: resolve a desktop `Icon=` value through the icon theme —
/// hicolor + flatpak exports + pixmaps, biggest raster size first,
/// scalable SVG as the fallback. The launcher used to show letter
/// tiles for every non-Matrix app (user finding 2026-07-21).
fn icon_locate(icon: &str) -> Option<LoadedIcon> {
    // The SEARCH has lived in matrixkit-icons since 16.08.2026 —
    // one truth for both paths (otherwise the first-light path showed
    // only the initial for every foreign app). All that is left here
    // is to wrap the found path in an iced handle.
    let path = matrixkit_icons::foreign_icon_path(icon)?;
    let s = path.to_string_lossy().to_string();
    if s.ends_with(".svg") {
        Some(LoadedIcon::Vector(iced::widget::svg::Handle::from_path(path)))
    } else {
        Some(LoadedIcon::Raster(image::Handle::from_path(path)))
    }
}

#[derive(Debug, Clone)]
struct Hit {
    name: String,
    category: &'static str,
    icon: Icon,
    items: u32,
    target: Target,
}

/// The fixed system actions (name, symbol, target).
pub(crate) fn actions() -> Vec<(&'static str, char, Target)> {
    vec![
        ("Sperren", mkw::symbol::LOCK, Target::Command { cmd: "loginctl lock-session".into(), terminal: false }),
        ("Erscheinung umschalten", mkw::symbol::DARK_MODE, Target::Appearance),
        ("Einstellungen", mkw::symbol::TUNE, Target::Command { cmd: "matrix-settings".into(), terminal: false }),
        ("Abmelden", mkw::symbol::LOGOUT, Target::Logout),
        ("Neustarten", mkw::symbol::RESTART, Target::Command { cmd: "systemctl reboot".into(), terminal: false }),
        ("Ausschalten", mkw::symbol::POWER, Target::Command { cmd: "systemctl poweroff".into(), terminal: false }),
    ]
}

/// R156 (the Siri dig): the intent registry. Apple's architecture,
/// measured in Metadata.appintents: the assistant knows no apps — it
/// reads a declarative registry the apps publish. Ours lives in
/// /usr/share/matrix/intents (the image's) and
/// ~/.local/share/matrix/intents (the user's own, Shortcuts parity).
#[derive(Debug, Clone)]
pub(crate) struct Intent {
    pub(crate) title: String,
    pub(crate) app_id: String,
    pub(crate) exec: Vec<String>,
    pub(crate) accepts_query: bool,
    pub(crate) keywords: Vec<String>,
}

fn intents_parse(raw: &str) -> Vec<Intent> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };
    let app_id = v["app_id"].as_str().unwrap_or_default().to_string();
    let Some(list) = v["intents"].as_array() else { return Vec::new() };
    list.iter()
        .filter_map(|i| {
            let exec: Vec<String> = i["exec"]
                .as_array()?
                .iter()
                .filter_map(|a| a.as_str().map(String::from))
                .collect();
            if exec.is_empty() {
                return None;
            }
            Some(Intent {
                title: i["title"].as_str()?.to_string(),
                app_id: app_id.clone(),
                exec,
                accepts_query: i["accepts_query"].as_bool().unwrap_or(false),
                keywords: i["keywords"]
                    .as_array()
                    .map(|k| k.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                    .unwrap_or_default(),
            })
        })
        .collect()
}

pub(crate) fn intents_load() -> Vec<Intent> {
    let mut out = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();
    for dir in [String::from("/usr/share/matrix/intents"), format!("{home}/.local/share/matrix/intents")] {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            if let Ok(raw) = std::fs::read_to_string(e.path()) {
                out.extend(intents_parse(&raw));
            }
        }
    }
    out
}

/// Fill {query} into the argv. Inside URLs the query travels
/// plus-joined; elsewhere verbatim.
pub(crate) fn intent_resolve(exec: &[String], query: &str) -> Vec<String> {
    exec.iter()
        .map(|a| {
            if a.contains("{query}") {
                let q = if a.contains("://") { query.replace(' ', "+") } else { query.to_string() };
                a.replace("{query}", &q)
            } else {
                a.clone()
            }
        })
        .collect()
}

/// R155 (the Spotlight dig): ask the living index. A missing daemon
/// answers with an empty palette section, never with a stall — the
/// timeout guards the keystroke latency.
pub(crate) fn seeker_find(query: &str) -> Vec<(i64, String)> {
    let run = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
    let sock = format!("{run}/matrix-seeker.sock");
    let Ok(mut s) = std::os::unix::net::UnixStream::connect(&sock) else {
        return Vec::new();
    };
    let _ = s.set_read_timeout(Some(std::time::Duration::from_millis(60)));
    use std::io::{Read as _, Write as _};
    if s.write_all(format!("find {query}\n").as_bytes()).is_err() {
        return Vec::new();
    }
    let mut body = String::new();
    let _ = s.read_to_string(&mut body);
    body.lines()
        .filter_map(|l| {
            let (score, path) = l.split_once('\t')?;
            Some((score.parse().ok()?, path.to_string()))
        })
        .collect()
}

/// Opening a hit reports back — the index learns what you actually use.
pub(crate) fn seeker_report_used(path: &str) {
    let run = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
    if let Ok(mut s) = std::os::unix::net::UnixStream::connect(format!("{run}/matrix-seeker.sock")) {
        use std::io::Write as _;
        let _ = s.write_all(format!("used {path}\n").as_bytes());
    }
}

fn trigger(z: &Target, apps: &[Entry]) {
    match z {
        Target::App(i) => {
            if let Some(e) = apps.get(*i) {
                app_start(e);
            }
        }
        Target::Window(id) => {
            // Focus also drives the camera there (in canvas mode).
            mk::canvas::window_bring(*id);
        }
        Target::Command { cmd, terminal } => {
            let _ = if *terminal {
                std::process::Command::new("foot").args(["-e", "sh", "-c", cmd]).spawn()
            } else {
                std::process::Command::new("sh").args(["-c", cmd]).spawn()
            };
        }
        Target::Intent(argv) => {
            if let Some((cmd, args)) = argv.split_first() {
                let _ = std::process::Command::new(cmd).args(args).spawn();
            }
        }
        Target::File(path) => {
            seeker_report_used(path);
            // Round 178: the kit's opener (xdg-open is missing on the Duo; the child was never reaped).
            mk::open::open(path);
        }
        Target::Appearance => mk::light_toggle(),
        Target::Logout => mk::canvas::logout(false),
        Target::Copy(s) => {
            use std::io::Write;
            if let Ok(mut child) = std::process::Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut pipe_in) = child.stdin.take() {
                    let _ = pipe_in.write_all(s.as_bytes());
                }
                let _ = child.wait();
            }
        }
    }
    std::process::exit(0);
}

// ----------------------------------------------------------------- App

#[to_layer_message]
#[derive(Debug, Clone)]
enum Msg {
    Search(String),
    Clear,
    Key(mkw::Key),
    Click(usize),
    /// R103: click on the backdrop outside the palette.
    Dismiss,
    /// R209: keep the on-screen keyboard while the menu is open.
    KeyboardBeat,
}

struct App {
    palette: mk::Palette,
    apps: Vec<Entry>,
    window: Vec<WindowInfo>,
    search: String,
    /// R156: the intent registry, loaded once at palette start.
    intents: Vec<Intent>,
    hit: Vec<Hit>,
    selected: usize,
    icons: std::collections::HashMap<String, Option<LoadedIcon>>,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        // R209 (Duo): the search wants the keyboard at once — a finger
        // opened Start (dock tile), so the physical-keyboard veto must not apply.
        // R209: the search types — summon the keyboard and claim the panel
        // Start opens on (the dock's). Elsewhere: the field's own beat.
        mkw::panel::claim_keyboard("dock-output", mk::device::DUO_DOCK_PANEL);
        mkw::keyboard::window_active(true);
        let apps = index_build();
        let window = window_fetch(&apps);
        let mut app = App {
            palette: mk::Palette::load().unwrap_or_default(),
            apps,
            window,
            search: String::new(),
            intents: intents_load(),
            hit: Vec::new(),
            selected: 0,
            icons: std::collections::HashMap::new(),
        };
        app.filter();
        (app, mkw::focus_search())
    }

    fn filter(&mut self) {
        let s = self.search.trim();
        let empty = s.is_empty();
        let mut kand: Vec<Hit> = Vec::new();

        // 1) Calculator — only if the input looks like math.
        if let Some(erg) = compute(s) {
            kand.push(Hit {
                name: format!("= {erg}"),
                category: "Rechnen",
                icon: Icon::Symbol(mkw::symbol::CODE),
                items: 100_000,
                target: Target::Copy(erg),
            });
        }

        // 2) Open windows (first on an empty search — quick overview).
        for f in &self.window {
            let p_name = items(&f.name, s);
            let p_title = items(&f.title, s);
            let Some(p) = p_name.into_iter().chain(p_title).max() else { continue };
            let display = if f.title.is_empty() || f.title == f.name {
                f.name.clone()
            } else {
                format!("{} — {}", f.name, f.title)
            };
            // Slightly favor windows (jumping is the most frequent wish).
            let bonus = if empty { 4 } else { 15 };
            kand.push(Hit {
                name: display,
                category: "Fenster",
                icon: Icon::App(
                    self.apps.iter().find(|e| e.name == f.name).map(|e| e.app_id.clone())
                        .unwrap_or_default(),
                ),
                items: p + bonus,
                target: Target::Window(f.id),
            });
        }

        // 3) Apps.
        for (i, e) in self.apps.iter().enumerate() {
            if let Some(p) = items(&e.name, s) {
                kand.push(Hit {
                    name: e.name.clone(),
                    category: "App",
                    icon: Icon::App(e.app_id.clone()),
                    items: p,
                    target: Target::App(i),
                });
            }
        }

        // 3a) Intents (R156): commands from the registry. A keyword as
        // first word plus accepts_query turns the rest into {query}
        // ("web wetter morgen") — the Siri grammar without the voice.
        if !empty {
            let (first, rest) = match s.split_once(' ') {
                Some((f, r)) => (f.to_lowercase(), r.trim().to_string()),
                None => (s.to_lowercase(), String::new()),
            };
            for i in &self.intents {
                let keyword_hit = i.keywords.iter().any(|k| *k == first);
                let p = if i.accepts_query && keyword_hit && !rest.is_empty() {
                    Some(35)
                } else {
                    items(&i.title, s)
                        .into_iter()
                        .chain(i.keywords.iter().filter_map(|k| items(k, s)))
                        .max()
                };
                let Some(p) = p else { continue };
                let query = if i.accepts_query && keyword_hit { rest.clone() } else { String::new() };
                let name = if query.is_empty() {
                    i.title.clone()
                } else {
                    format!("{} — \u{201e}{query}\u{201c}", i.title)
                };
                kand.push(Hit {
                    name,
                    category: "Befehl",
                    icon: Icon::App(i.app_id.clone()),
                    items: p + 6,
                    target: Target::Intent(intent_resolve(&i.exec, &query)),
                });
            }
        }

        // 3b) Files, from the living index (R155). Shown only with
        // input; a small malus keeps apps first at equal match quality.
        if !empty {
            let home = std::env::var("HOME").unwrap_or_default();
            for (p, path) in seeker_find(s) {
                let name = std::path::Path::new(&path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.clone());
                let place = std::path::Path::new(&path)
                    .parent()
                    .map(|d| d.to_string_lossy().replacen(&home, "~", 1))
                    .unwrap_or_default();
                kand.push(Hit {
                    name: format!("{name} — {place}"),
                    category: "Datei",
                    icon: Icon::Symbol(mkw::symbol::FILE),
                    items: (p - 3).max(1) as u32,
                    target: Target::File(path),
                });
            }
        }

        // 4) System actions — only with input (do not clutter the empty palette).
        if !empty {
            for (name, sym, target) in actions() {
                if let Some(p) = items(name, s) {
                    kand.push(Hit {
                        name: name.to_string(),
                        category: "Aktion",
                        icon: Icon::Symbol(sym),
                        items: p + 5,
                        target,
                    });
                }
            }
        }

        kand.sort_by(|a, b| b.items.cmp(&a.items).then(a.name.cmp(&b.name)));
        // R103: the palette shows EVERYTHING and scrolls (user finding:
        // "limited app list, no scrolling"). Searches stay ranked,
        // the empty palette lists every app alphabetically after the
        // open windows.
        if !empty {
            kand.truncate(40);
        }
        self.hit = kand;
        self.selected = 0;

        // Bake icons of the visible app/window hits.
        let p = self.palette;
        for t in &self.hit {
            if let Icon::App(id) = &t.icon {
                if id.is_empty() {
                    continue;
                }
                let icon_field = self
                    .apps
                    .iter()
                    .find(|e| e.app_id == *id)
                    .and_then(|e| e.icon.clone());
                self.icons.entry(id.clone()).or_insert_with(|| {
                    matrixkit_icons::render_png(id, &p)
                        .map(image::Handle::from_bytes)
                        .map(LoadedIcon::Raster)
                        .or_else(|| icon_locate(icon_field.as_deref().unwrap_or(id)))
                });
            }
        }
    }

    fn trigger_selected(&self) {
        if let Some(t) = self.hit.get(self.selected) {
            trigger(&t.target, &self.apps);
        }
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Search(s) => {
                self.search = s;
                self.filter();
            }
            Msg::Clear => {
                self.search.clear();
                self.filter();
            }
            Msg::Click(i) => {
                mk::feel::tap();
                if let Some(t) = self.hit.get(i) {
                    trigger(&t.target, &self.apps);
                }
            }
            Msg::Dismiss => std::process::exit(0),
            Msg::KeyboardBeat => mkw::keyboard::transmit(),
            Msg::Key(t) => match t {
                mkw::Key::Escape => std::process::exit(0),
                mkw::Key::Next => {
                    if !self.hit.is_empty() {
                        self.selected = (self.selected + 1) % self.hit.len();
                    }
                }
                mkw::Key::Prev => {
                    if !self.hit.is_empty() {
                        self.selected =
                            (self.selected + self.hit.len() - 1) % self.hit.len();
                    }
                }
                mkw::Key::Activate => self.trigger_selected(),
                _ => {}
            },
            _ => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        if mk::device::finger_first() {
            return Subscription::batch([
                mkw::key_subscription(Msg::Key),
                mkw::keyboard::heartbeat().map(|_| Msg::KeyboardBeat),
            ]);
        }
        mkw::key_subscription(Msg::Key)
    }

    /// R209 (Duo, the owner): the GLASS APP MENU — the surface is the glass
    /// card; search on top, then the hits as tiles in the dock's own
    /// language (name + kind, the whole tile is the button, buzz on tap,
    /// the keyboard's choice lit). Columns follow the panel width.
    fn glass_view(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let glass = mkw::glass::wanted();
        let tile_h = mk::device::SNUG_TILE_HEIGHT as f32;
        let gap = mk::spacing::S;
        let grid = iced::widget::responsive(move |size| {
            let columns = ((size.width + gap) / (170.0 + gap)).floor().max(1.0) as usize;
            let mut grid = column![].spacing(gap);
            // family-exception: the iced fallback of the launcher (the house view in licht.rs uses the kit raster); retires with it
            for (n, chunk) in self.hit.chunks(columns).enumerate() {
                let mut line = row![].spacing(gap);
                for (k, t) in chunk.iter().enumerate() {
                    let index = n * columns + k;
                    let glow = if index == self.selected && !self.search.is_empty() { 0.6 } else { 0.0 };
                    let tile = mkw::app_tile_naked(
                        // One line per name: a wrapped name pushed the kind out.
                        mk::elide(&t.name, 20),
                        t.category,
                        Vec::new(),
                        Msg::Click(index),
                        None::<fn(iced::Point) -> Msg>,
                        glow,
                        p,
                    );
                    line = line.push(
                        container(tile)
                            .width(Length::FillPortion(1))
                            .height(Length::Fixed(tile_h))
                            .style(move |_| container::Style {
                                background: Some(color(p.on_surface.with_alpha(0.06)).into()),
                                border: iced::border::rounded(mk::radius::LARGE),
                                ..Default::default()
                            }),
                    );
                }
                // Short last line: keep the tiles their column width.
                for _ in chunk.len()..columns {
                    line = line.push(Space::new().width(Length::FillPortion(1)));
                }
                grid = grid.push(line);
            }
            if self.hit.is_empty() {
                grid = grid.push(
                    container(mkw::txt("Nichts gefunden", mk::typo::CALLOUT, p.on_surface_variant))
                        .center_x(Length::Fill)
                        .padding(mk::spacing::L),
                );
            }
            mkw::scroll_region(grid.into(), p)
        });
        container(
            column![
                mkw::search_field(&self.search, "Suchen, springen oder rechnen …", Msg::Search, Msg::Clear, p),
                grid,
            ]
            .spacing(mk::spacing::M),
        )
        .padding(mk::spacing::M)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| {
            if glass {
                container::Style::default()
            } else {
                mkw::panel::pill(p, mk::radius::NORMAL, mk::spacing::S)
            }
        })
        .into()
    }

    fn view(&self) -> Element<'_, Msg> {
        if mk::device::shelf().apps_leads {
            return self.glass_view();
        }
        let p = self.palette;

        let mut content = column![].spacing(mk::spacing::S);

        for (row, t) in self.hit.iter().enumerate() {
            let image: Element<'_, Msg> = match &t.icon {
                Icon::App(id) => match self.icons.get(id).cloned().flatten() {
                    Some(LoadedIcon::Raster(h)) => image(h).width(ICON).height(ICON).into(),
                    Some(LoadedIcon::Vector(h)) => {
                        iced::widget::svg(h).width(ICON).height(ICON).into()
                    }
                    None => placeholder_icon(&t.name, p),
                },
                Icon::Symbol(sym) => container(mkw::symbol::<Msg>(*sym, ICON * 0.62, p.on_surface_variant))
                    .center_x(Length::Fixed(ICON))
                    .center_y(Length::Fixed(ICON))
                    .into(),
            };

            let active = row == self.selected;
            let line = mouse_area(
                container(
                    row![
                        image,
                        mkw::txt(&t.name, mk::typo::BODY, p.on_surface),
                        Space::new().width(Length::Fill),
                        mkw::txt(t.category, mk::typo::CAPTION, p.on_surface_variant),
                    ]
                    .spacing(mk::spacing::M)
                    .align_y(iced::Alignment::Center),
                )
                .padding([mk::spacing::XS as u16, mk::spacing::S as u16])
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: active.then(|| {
                        color(p.on_surface.over(p.surface_container, mkw::panel::HOVER)).into()
                    }),
                    border: iced::border::rounded(mk::radius::NORMAL),
                    ..Default::default()
                }),
            )
            .on_press(Msg::Click(row));
            content = content.push(line);
        }

        if self.hit.is_empty() {
            content = content.push(
                container(mkw::txt("Nichts gefunden", mk::typo::CALLOUT, p.on_surface_variant))
                    .center_x(Length::Fill)
                    .padding(mk::spacing::L),
            );
        }

        // R103 popup grammar: dismiss backdrop behind, palette floating
        // in the upper third, list scrolls inside the fixed pill.
        let pill = container(
            column![
                container(mkw::search_field(
                    &self.search,
                    "Suchen, springen oder rechnen …",
                    Msg::Search,
                    Msg::Clear,
                    p,
                ))
                .padding(mk::spacing::S),
                mkw::scroll_region(content.padding(mk::spacing::S).into(), p),
            ],
        )
        .style(move |_| mkw::panel::pill(p, mk::radius::NORMAL, mk::spacing::S));

        // R209 (Duo, the owner): the view fills the dock's panel with the SAME
        // air as a window — one window gap at the edges, the dock clear below.
        if mk::device::shelf().apps_leads {
            let gap = mk::device::WINDOW_GAP;
            return iced::widget::stack![
                mkw::panel::backdrop(Msg::Dismiss),
                container(pill.width(Length::Fill).height(Length::Fill)).padding(iced::Padding {
                    top: gap,
                    right: gap,
                    bottom: mk::device::dock_clearance(),
                    left: gap,
                }),
            ]
            .into();
        }
        let pill = pill
            .width(Length::Fixed(WIDTH as f32 - 2.0 * mk::spacing::M))
            .height(Length::Fixed(HEIGHT as f32));

        iced::widget::stack![
            mkw::panel::backdrop(Msg::Dismiss),
            container(pill)
                .center_x(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::Padding {
                    top: 140.0,
                    ..iced::Padding::ZERO
                }),
        ]
        .into()
    }
}

fn placeholder_icon<'a>(name: &str, p: mk::Palette) -> Element<'a, Msg> {
    container(mkw::txt(
        name.chars().next().unwrap_or('?').to_uppercase().to_string(),
        mk::typo::HEADLINE,
        p.on_primary_container,
    ))
    .center_x(Length::Fixed(ICON))
    .center_y(Length::Fixed(ICON))
    .style(move |_| container::Style {
        background: Some(color(p.primary_container).into()),
        border: iced::border::rounded(mk::radius::SMALL),
        ..Default::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoring_orders_correct() {
        assert!(items("Firefox", "fir").unwrap() > items("Fcitx Firefox-Helfer", "fir").unwrap());
        assert_eq!(items("Bazaar", "xyz"), None);
        assert_eq!(items("Egal", ""), Some(1));
    }

    #[test]
    fn exec_codes_fallen_out() {
        assert_eq!(exec_cleanup("firefox %u"), "firefox");
        assert_eq!(exec_cleanup("app --flag %U %f"), "app --flag");
    }

    #[test]
    fn calculator_computes_and_rejects_text() {
        assert_eq!(compute("12*8+4").as_deref(), Some("100"));
        assert_eq!(compute("(2+3)*4").as_deref(), Some("20"));
        assert_eq!(compute("10/4").as_deref(), Some("2.5"));
        assert_eq!(compute("7x6").as_deref(), Some("42"));
        assert_eq!(compute("firefox"), None); // text, no digit
        assert_eq!(compute("42"), None); // number without operator = no hit
        assert_eq!(compute("5/0"), None); // divide by zero guard
    }
}

#[cfg(test)]
mod intent_tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "app_id": "matrix-web",
        "intents": [
            { "id": "web.search", "title": "Im Web suchen",
              "exec": ["xdg-open", "https://duckduckgo.com/?q={query}"],
              "accepts_query": true, "keywords": ["web", "suche"] }
        ]
    }"#;

    #[test]
    fn a_manifest_becomes_intents() {
        let list = intents_parse(SAMPLE);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].app_id, "matrix-web");
        assert!(list[0].accepts_query);
        assert_eq!(list[0].keywords, vec!["web", "suche"]);
    }

    #[test]
    fn queries_travel_plus_joined_into_urls() {
        let list = intents_parse(SAMPLE);
        let argv = intent_resolve(&list[0].exec, "wetter morgen");
        assert_eq!(argv[1], "https://duckduckgo.com/?q=wetter+morgen");
        // Outside URLs the query stays verbatim.
        let raw = intent_resolve(&[String::from("echo"), String::from("{query}")], "a b");
        assert_eq!(raw[1], "a b");
    }

    #[test]
    fn rubbish_manifests_yield_nothing() {
        assert!(intents_parse("not json").is_empty());
        assert!(intents_parse(r#"{"app_id":"x","intents":[{"title":"t"}]}"#).is_empty());
    }
}
