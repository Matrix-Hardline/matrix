//! Matrix Kontext — the context-menu SERVICE of the canvas (R120).
//!
//! macOS extract behind the redesign: reference menus feel instant
//! because they are windows of a RESIDENT server — no process launch
//! sits between the click and the pill. matrix-context therefore
//! stays alive: the first start becomes the daemon (hidden 1×1
//! surface), every further invocation hands its arguments over via
//! the request file and vanishes before any GPU init (~15 ms instead
//! of a cold app start).
//!
//! Modes (unchanged contract, callers need no changes):
//!   matrix-context leinwand <x> <y>   — canvas right click
//!   matrix-context dock <offset> <bottom> <app_id> <pinned> <ids|->
//!   matrix-context clips <offset> <bottom>
//!
//! Clicking an entry acts, clicking beside it closes — both CLOSE
//! the veil window; a second right click simply replaces the content
//! (the old toggle grammar, minus the respawn). Architecture: the
//! layershell DAEMON pattern (StartMode::Background, no standing
//! surface) — every menu is a fresh window, like the reference.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced_layershell;
use matrixkit_widgets::iced::widget::{container, mouse_area, Space};
use matrixkit_widgets::iced::{Element, Length, Subscription, Task};
use matrixkit_widgets::iced_layershell::reexport::Anchor;
use matrixkit_widgets::iced_layershell::to_layer_message;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;

mod licht;

pub(crate) fn runtime_dir() -> String {
    std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"))
}

/// Does a menu card stand on screen? Unknown counts as yes — no restart
/// on doubt (mk::canvas::layer_present, round 23).
fn card_shown() -> bool {
    mk::canvas::layer_present("matrix-card").unwrap_or(true)
}

pub(crate) fn request_path() -> String {
    format!("{}/matrix-context-request", runtime_dir())
}

fn main() -> Result<(), iced_layershell::Error> {
    use std::os::unix::net::{UnixListener, UnixStream};

    // The daemon choice is SHARED (camp 49): whoever binds the socket
    // serves — iced or First Light alike. All further callers
    // pass their arguments through and vanish before any
    // GPU init. That way the existing callers (compositor,
    // dock) automatically feed whichever one is serving.
    // Round 62: the house is the default (as for the notifications and
    // the keyboard); `--iced` is the way back. `--erstlicht` still works.
    let first = std::env::args().nth(1);
    let iced = first.as_deref() == Some("--iced");
    let flag = matches!(first.as_deref(), Some("--iced") | Some("--erstlicht"));
    let erstlicht = !iced;
    let args: Vec<String> =
        std::env::args().skip(if flag { 2 } else { 1 }).collect();
    let sock_path = format!("{}/matrixkit-matrix-context.sock", runtime_dir());

    // Daemon election: whoever binds the socket serves; everyone else
    // hands the request over and vanishes without touching the GPU.
    let listener = match UnixListener::bind(&sock_path) {
        Ok(l) => Some(l),
        Err(_) => {
            if UnixStream::connect(&sock_path).is_ok() {
                if !args.is_empty() {
                    let _ = std::fs::write(request_path(), args.join("\u{1f}"));
                }
                return Ok(());
            }
            // Crash corpse — clean up and take its place.
            let _ = std::fs::remove_file(&sock_path);
            UnixListener::bind(&sock_path).ok()
        }
    };
    if let Some(l) = listener {
        std::mem::forget(l);
    }

    // A stale request from a crash must not pop a menu at login; our
    // own arguments become the first request instead.
    let _ = std::fs::remove_file(request_path());

    if erstlicht {
        // Own arguments travel IN MEMORY into the light path — the
        // listener healing (camp 47) would consume an immediately written
        // request file as the initial state.
        if let Err(e) = licht::run(args) {
            eprintln!("[kontext] Fensterhaus-Fehler: {e}");
        }
        return Ok(());
    }
    matrixkit_widgets::present_mailbox();
    use matrixkit_widgets::iced_layershell::reexport::{KeyboardInteractivity, Layer};

    // Our menus live on compositor glass cards (R125) — thin shells.
    mkw::ui::menus_on_glass();

    if !args.is_empty() {
        let _ = std::fs::write(request_path(), args.join("\u{1f}"));
    }

    mkw::app_canvas::run::<App>(
        "matrix-context",
        iced_layershell::settings::LayerShellSettings {
            // Daemon: NO standing surface — every menu is a fresh
            // veil window, spawned on request and closed after.
            start_mode: iced_layershell::settings::StartMode::Background,
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            ..Default::default()
        },
    )
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

/// The entries of the canvas menu: (symbol, title, app).
pub(crate) const ENTRIES: [(char, &str, &str); 4] = [
    (mkw::symbol::APPS, "Apps …", "matrix-start"),
    ('\u{e894}', "Neues Browserfenster", "matrix-web"),
    (mkw::symbol::PALETTE, "Hintergrund ändern …", "matrix-settings hintergrund"),
    (mkw::symbol::TUNE, "Einstellungen …", "matrix-settings"),
];

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Msg {
    /// 25 Hz request poll — a stat on tmpfs, nothing more.
    Poll,
    StartUp(usize),
    PinToggle,
    Quit,
    /// Dock anpassen: opens matrix-settings in the "bars" section.
    Customize,
    /// Clipboard history: entry i back into the clipboard.
    Clip(usize),
    ClipsClear,
    Close,
    /// The compositor (or we) closed the veil window.
    WindowClosed(iced::window::Id),
}

/// The clipboard history the dock maintains (JSON array, newest
/// first) — volatile in the runtime directory, like the badges.
pub(crate) fn clips_path() -> std::path::PathBuf {
    let base = runtime_dir();
    // R107: matrix-clipboard.json is the contract; the German legacy
    // name is read if the new one is missing.
    let dir = std::path::PathBuf::from(base);
    let modern = dir.join("matrix-clipboard.json");
    if modern.exists() {
        modern
    } else {
        let legacy = dir.join("matrix-zwischenablage.json");
        if legacy.exists() { legacy } else { modern }
    }
}

pub(crate) fn clips_read() -> Vec<String> {
    std::fs::read_to_string(clips_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Dock mode: the tile menu as its own overlay surface — the dock
/// itself never grows anymore (the resize glitch is thereby
/// structurally impossible; user finding 2026-07-08).
struct DockTarget {
    app_id: String,
    pinned: bool,
    window: Vec<u64>,
    /// Tile offset from the screen center (the dock is centered).
    offset_x: f32,
    /// Distance of the menu bottom from the lower screen edge.
    bottom: f32,
}

/// Clips mode: the clipboard history as a menu above the dock.
struct ClipsTarget {
    offset_x: f32,
    bottom: f32,
    entries: Vec<String>,
}

/// What the service currently shows.
enum Target {
    Hidden,
    Canvas { x: f32, y: f32 },
    Dock(DockTarget),
    Clips(ClipsTarget),
}

impl Target {
    /// One request line (unit-separated argv) becomes a target.
    fn parse(args: &[String]) -> Target {
        match args.first().map(String::as_str) {
            Some("dock") => Target::Dock(DockTarget {
                offset_x: args.get(1).and_then(|a| a.parse().ok()).unwrap_or(0.0),
                bottom: args.get(2).and_then(|a| a.parse().ok()).unwrap_or(140.0),
                app_id: args.get(3).cloned().unwrap_or_default(),
                pinned: args.get(4).map(String::as_str) == Some("1"),
                window: args
                    .get(5)
                    .map(|f| f.split(',').filter_map(|t| t.parse().ok()).collect())
                    .unwrap_or_default(),
            }),
            Some("clips") => Target::Clips(ClipsTarget {
                offset_x: args.get(1).and_then(|a| a.parse().ok()).unwrap_or(0.0),
                bottom: args.get(2).and_then(|a| a.parse().ok()).unwrap_or(140.0),
                entries: clips_read(),
            }),
            _ => Target::Canvas {
                x: args.get(1).and_then(|a| a.parse().ok()).unwrap_or(200.0),
                y: args.get(2).and_then(|a| a.parse().ok()).unwrap_or(200.0),
            },
        }
    }
}

struct App {
    palette: mk::Palette,
    target: Target,
    /// The open veil window, if any (one menu at a time).
    veil: Option<iced::window::Id>,
    /// The menu CARD window (R125): exactly menu-sized, namespace
    /// matrix-card — the compositor lays its glass body under it.
    card: Option<iced::window::Id>,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        (
            App {
                palette: mk::Palette::load().unwrap_or_default(),
                target: Target::Hidden,
                veil: None,
                card: None,
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> Subscription<Msg> {
        Subscription::batch([
            // The request file is watched by the Kit's listener thread (a
            // stat every 40 ms costs nothing); the app wakes only when a
            // request really arrives — before, it woke 25 times a second.
            mkw::events::file_changes(
                "context-request",
                request_path(),
                std::time::Duration::from_millis(40),
            )
            .map(|_| Msg::Poll),
            iced::window::close_events().map(Msg::WindowClosed),
        ])
    }

    /// Close veil and card — the daemon stays warm without any
    /// surface (the reference grammar: a menu IS a window).
    fn hide(&mut self) -> Task<Msg> {
        self.target = Target::Hidden;
        let mut tasks = Vec::new();
        if let Some(id) = self.veil.take() {
            tasks.push(Task::done(Msg::RemoveWindow(id)));
        }
        if let Some(id) = self.card.take() {
            tasks.push(Task::done(Msg::RemoveWindow(id)));
        }
        Task::batch(tasks)
    }

    /// The card geometry of the current target: size from the exact
    /// menu metrics, position edge-flipped like the reference.
    fn card_geometry(&self) -> ((u32, u32), Anchor, (i32, i32, i32, i32)) {
        let (sw, sh) = mkw::glass::screen_size();
        match &self.target {
            Target::Canvas { x, y } => {
                let (w, h) = mkw::ui::menu_size(&self.canvas_entries());
                let mut cx = *x;
                let mut cy = *y;
                if cx + w > sw {
                    cx = (cx - w).max(0.0);
                }
                if cy + h > sh {
                    cy = (cy - h).max(0.0);
                }
                (
                    (w as u32, h as u32),
                    Anchor::Top | Anchor::Left,
                    (cy as i32, 0, 0, cx as i32),
                )
            }
            Target::Dock(d) => {
                let (w, h) = mkw::ui::menu_size(&self.dock_entries(d));
                let left = (sw / 2.0 + d.offset_x - w / 2.0).max(0.0);
                (
                    (w as u32, h as u32),
                    Anchor::Bottom | Anchor::Left,
                    (0, 0, d.bottom as i32, left as i32),
                )
            }
            Target::Clips(c) => {
                let (w, h) = mkw::ui::menu_size(&self.clips_entries(c));
                let left = (sw / 2.0 + c.offset_x - w / 2.0).max(0.0);
                (
                    (w as u32, h as u32),
                    Anchor::Bottom | Anchor::Left,
                    (0, 0, c.bottom as i32, left as i32),
                )
            }
            Target::Hidden => ((1, 1), Anchor::Top | Anchor::Left, (0, 0, 0, 0)),
        }
    }

    /// Open the veil (dismiss layer) and a FRESH card window sized to
    /// the menu — replacing menus recreates the card, so the island
    /// pop plays per menu like the reference.
    fn show(&mut self) -> Task<Msg> {
        use matrixkit_widgets::iced_layershell::reexport::{
            KeyboardInteractivity, Layer, NewLayerShellSettings,
        };
        let mut tasks = Vec::new();
        if let Some(old) = self.card.take() {
            tasks.push(Task::done(Msg::RemoveWindow(old)));
        }
        if self.veil.is_none() {
            let id = iced::window::Id::unique();
            self.veil = Some(id);
            tasks.push(Task::done(Msg::NewLayerShell {
                settings: NewLayerShellSettings {
                    size: Some((0, 0)),
                    anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
                    layer: Layer::Overlay,
                    keyboard_interactivity: KeyboardInteractivity::None,
                    exclusive_zone: Some(-1),
                    ..Default::default()
                },
                id,
            }));
        }
        let (size, anchor, margin) = self.card_geometry();
        let id = iced::window::Id::unique();
        self.card = Some(id);
        tasks.push(Task::done(Msg::NewLayerShell {
            settings: NewLayerShellSettings {
                size: Some(size),
                anchor,
                margin: Some(margin),
                layer: Layer::Overlay,
                keyboard_interactivity: KeyboardInteractivity::None,
                exclusive_zone: Some(-1),
                namespace: Some(String::from("matrix-card")),
                ..Default::default()
            },
            id,
        }));
        Task::batch(tasks)
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Poll => {
                if let Ok(s) = std::fs::read_to_string(request_path()) {
                    let _ = std::fs::remove_file(request_path());
                    let args: Vec<String> =
                        s.split('\u{1f}').map(String::from).collect();
                    // The palette may have wandered while we slept.
                    self.palette = mk::Palette::load().unwrap_or_default();
                    self.target = Target::parse(&args);
                    eprintln!("context: request {:?}", args.first());
                    // SELF-HEALING (02.11.2026): after a day of panel and
                    // posture changes this daemon took requests but opened
                    // no menu any more (316 MB, the desktop menu was dead).
                    // If no card stands 0.9 s after a request, it starts
                    // itself anew (mk::process::reexec); the next tap works.
                    std::thread::spawn(|| {
                        std::thread::sleep(std::time::Duration::from_millis(900));
                        if !card_shown() {
                            eprintln!("context: Menü blieb aus — Dienst startet sich neu");
                            let e = mk::process::reexec();
                            eprintln!("context: Neustart misslang: {e}");
                        }
                    });
                    return self.show();
                }
                Task::none()
            }
            Msg::StartUp(i) => {
                // "name bereich" → wish file, then start (fusion R41).
                let target = ENTRIES[i].2;
                let mut parts = target.splitn(2, ' ');
                let name = parts.next().unwrap_or(target);
                if let Some(area) = parts.next() {
                    let _ = std::fs::write(
                        format!("{}/{name}-bereich", runtime_dir()),
                        area,
                    );
                }
                mkw::panel::app_start(name);
                self.hide()
            }
            Msg::PinToggle => {
                if let Some(d) = self.target_dock() {
                    let mut pins: Vec<String> = mk::settings::read("dock-pins")
                        .unwrap_or_default()
                        .split_whitespace()
                        .map(String::from)
                        .collect();
                    if d.pinned {
                        let id = d.app_id.clone();
                        pins.retain(|p| p != &id);
                    } else {
                        pins.push(d.app_id.clone());
                    }
                    mk::settings::write("dock-pins", &pins.join(" "));
                }
                self.hide()
            }
            Msg::Quit => {
                if let Some(d) = self.target_dock() {
                    for id in d.window.clone() {
                        mk::canvas::window_close(id);
                    }
                }
                self.hide()
            }
            Msg::Customize => {
                // Fusion R41: section wish first, then start — matrix-settings
                // reads and consumes the file on startup (R105 protocol name).
                let _ = std::fs::write(
                    format!("{}/matrix-settings-area", runtime_dir()),
                    "bars",
                );
                mkw::panel::app_start("matrix-settings");
                self.hide()
            }
            Msg::Clip(i) => {
                // The entry wanders back into the clipboard — passed as an
                // argument, never through a shell (no injection).
                let text = match &self.target {
                    Target::Clips(c) => c.entries.get(i).cloned(),
                    _ => None,
                };
                if let Some(text) = text {
                    // Round 51: reaped — one zombie per copy before.
                    let _ = mk::process::spawn_reaped(std::process::Command::new("wl-copy").arg("--").arg(text));
                }
                self.hide()
            }
            Msg::ClipsClear => {
                let _ = std::fs::remove_file(clips_path());
                self.hide()
            }
            Msg::Close => self.hide(),
            Msg::WindowClosed(id) => {
                eprintln!("context: closed {id:?}");
                if self.veil == Some(id) {
                    self.veil = None;
                    self.target = Target::Hidden;
                }
                if self.card == Some(id) {
                    self.card = None;
                }
                // MEMORY CEILING (round 32, 02.11.2026): every iced menu
                // surface leaves ~18 MB behind (388 MB after 20 menus).
                // Idle and above the ceiling, the daemon starts itself anew
                // — only when no menu stands (the next tap opens normally).
                if self.veil.is_none() && self.card.is_none() && mk::process::rss_mb().is_some_and(|mb| mb > 120) {
                    std::thread::spawn(|| {
                        std::thread::sleep(std::time::Duration::from_millis(400));
                        if !card_shown() {
                            eprintln!("context: {} MB — Dienst startet sich leicht neu", mk::process::rss_mb().unwrap_or(0));
                            let e = mk::process::reexec();
                            eprintln!("context: Neustart misslang: {e}");
                        }
                    });
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn target_dock(&self) -> Option<&DockTarget> {
        match &self.target {
            Target::Dock(d) => Some(d),
            _ => None,
        }
    }

    /// The canvas menu entries — shared by size and view.
    fn canvas_entries(&self) -> Vec<mkw::ui::MenuItem<Msg>> {
        ENTRIES
            .iter()
            .enumerate()
            .map(|(i, (glyph, title, _))| {
                if i == 0 {
                    mkw::ui::MenuItem::ItemWithShortcut {
                        glyph: Some(*glyph),
                        title: String::from(*title),
                        shortcut: String::from("Super+Space"),
                        color: None,
                        msg: Msg::StartUp(i),
                    }
                } else {
                    mkw::ui::MenuItem::Item {
                        glyph: Some(*glyph),
                        title: String::from(*title),
                        color: None,
                        msg: Msg::StartUp(i),
                    }
                }
            })
            .collect()
    }

    /// Clipboard-history entries — shared by size and view.
    fn clips_entries(&self, c: &ClipsTarget) -> Vec<mkw::ui::MenuItem<Msg>> {
        let p = self.palette;
        let mut entries: Vec<mkw::ui::MenuItem<Msg>> = Vec::new();
        if c.entries.is_empty() {
            entries.push(mkw::ui::MenuItem::Item {
                glyph: Some(mkw::symbol::CONTENT_COPY),
                title: String::from("Noch nichts kopiert"),
                color: None,
                msg: Msg::Close,
            });
        }
        for (i, text) in c.entries.iter().enumerate() {
            // Single-line preview: line breaks out, cut hard.
            let mut line: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if line.chars().count() > 34 {
                line = line.chars().take(33).collect::<String>() + "…";
            }
            entries.push(mkw::ui::MenuItem::Item {
                glyph: None,
                title: line,
                color: None,
                msg: Msg::Clip(i),
            });
        }
        if !c.entries.is_empty() {
            entries.push(mkw::ui::MenuItem::Separator);
            entries.push(mkw::ui::MenuItem::Item {
                glyph: None,
                title: String::from("Verlauf leeren"),
                color: Some(p.error),
                msg: Msg::ClipsClear,
            });
        }
        entries
    }

    /// Dock-tile entries — shared by size and view.
    fn dock_entries(&self, d: &DockTarget) -> Vec<mkw::ui::MenuItem<Msg>> {
        let mut entries: Vec<mkw::ui::MenuItem<Msg>> = vec![mkw::ui::MenuItem::Item {
            glyph: None,
            title: String::from(if d.pinned { "Loslösen" } else { "Anpinnen" }),
            color: None,
            msg: Msg::PinToggle,
        }];
        if !d.window.is_empty() {
            entries.push(mkw::ui::MenuItem::Item {
                glyph: None,
                title: String::from("Beenden"),
                color: None,
                msg: Msg::Quit,
            });
        }
        // Always reachable: customize the dock (and the bar) — like "Dock
        // anpassen …" in the reference dock context menu. R120 moved the tile
        // menu out of the dock surface and this entry was left behind, so the
        // documented way into Matrix Anpassen had no entry point at all.
        entries.push(mkw::ui::MenuItem::Item {
            glyph: Some(mkw::symbol::TUNE),
            title: String::from("Anpassen …"),
            color: None,
            msg: Msg::Customize,
        });
        entries
    }

    /// Two windows per menu (R125): the veil catches the dismiss, the
    /// CARD carries the pill on its compositor glass body.
    fn view(&self, window: iced::window::Id) -> Element<'_, Msg> {
        if self.card == Some(window) {
            let p = self.palette;
            let entries = match &self.target {
                Target::Canvas { .. } => self.canvas_entries(),
                Target::Dock(d) => self.dock_entries(d),
                Target::Clips(c) => self.clips_entries(c),
                Target::Hidden => Vec::new(),
            };
            if entries.is_empty() {
                return Space::new()
                    .width(Length::Fixed(1.0))
                    .height(Length::Fixed(1.0))
                    .into();
            }
            return mkw::ui::menu_family(None, entries, p);
        }
        // The veil: invisible, clicking it closes — both buttons.
        mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Msg::Close)
        .on_right_press(Msg::Close)
        .into()
    }
}
