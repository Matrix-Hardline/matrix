//! Matrix Tastatur — app #27, the on-screen keyboard (R58).
//!
//! The tablet reference grammar: it APPEARS by itself when you touch
//! an input field and no physical keyboard is attached — and it
//! vanishes when the field loses focus, a keyboard gets plugged in
//! or ✕ closes it.
//!
//! Drei Gesichter eines Binaries:
//! * **Wache** (no argument, autostart): listens on the datagram
//!   socket for the `auf` heartbeats of the MatrixKit input fields
//!   (mkw::tastatur::funken), checks the keyboard situation in
//!   /proc/bus/input/devices every 2 s and starts/ends the surface.
//! * **`--flaeche`**: the layershell pill (Layer::Top, NEVER takes
//!   keyboard focus) — QWERTZ-CH, ⇧, ?123, types via the injection.
//! * **`zeigen` / `verbergen` / `--tipptest`**: handles and proof.
//!
//! Typing goes through `zwp_virtual_keyboard_v1` (injection.rs) — the
//! characters land in the FOCUSED window, which is why the same
//! keyboard works in every app and in the greeter password field.

mod injection;

use matrixkit_widgets::iced;
use matrixkit_widgets::iced_layershell;
use std::sync::mpsc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use matrixkit_widgets::iced::widget::{button, column, container, row};
use matrixkit_widgets::iced::{Color, Element, Font, Length, Subscription, Task};
use matrixkit_widgets::iced_layershell::to_layer_message;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use matrixkit_widgets::color;

/// Base key (1 unit) and row height — touch targets, not decor.
const KEY: f32 = 56.0;
const KEY_H: f32 = 48.0;
const GAP: f32 = 6.0;
const MARGIN: f32 = 12.0;
/// Widest row: 11 units (qwertzuiopü).
const PILL_B: f32 = 11.0 * KEY + 10.0 * GAP + 2.0 * MARGIN;
const PILL_H: f32 = 4.0 * KEY_H + 3.0 * GAP + 2.0 * MARGIN;

/// Exit code of the surface for "✕ pressed" — the guard then locks
/// until the next focus change instead of popping right back up.
const MANUAL_CLOSE: i32 = 7;

/// The injection channel — laid before the iced run (App::new has no
/// arguments, so it lives in the OnceLock).
static CHANNEL: OnceLock<Option<mpsc::Sender<injection::Command>>> = OnceLock::new();

/// Pad bridge (R94): matrix-pad sends navigation commands to the
/// visible surface — the focus ring wanders across the keys.
static PAD_COMMANDS: OnceLock<std::sync::Mutex<std::collections::VecDeque<String>>> =
    OnceLock::new();

fn runtime_base() -> String {
    std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"))
}

fn pad_socket() -> String {
    format!("{}/matrix-keyboard-pad.sock", runtime_base())
}

/// Visibility marker: exists + fresh = the surface lives. The guard
/// kills the surface hard (SIGKILL), so only the mtime counts —
/// the 5 s pulse keeps it fresh, matrix-pad accepts < 8 s.
fn open_marker() -> String {
    format!("{}/matrix-keyboard-offen", runtime_base())
}

mod licht;

fn main() -> Result<(), iced_layershell::Error> {
    matrixkit_widgets::present_mailbox();
    // Camp 68 of the iced replacement: --erstlicht is the
    // window-house path (shadow doctrine) — it picks the PATH,
    // not the MODE. Without this distinction
    // `matrix-keyboard --erstlicht` showed the keyboard AT ONCE, while
    // the reference without an argument only starts the watcher and
    // opens on a signal. The app is in niri's autostart — the
    // keyboard would have stuck to the screen (finger test
    // 17.08.2026).
    let light_path = std::env::args().any(|a| a == "--erstlicht");
    let arg = std::env::args().skip(1).find(|a| a != "--erstlicht");
    match arg.as_deref() {
        Some("--surface") => {
            // Round 33 (02.11.2026): the HOUSE draws the keyboard («without
            // iced»); iced only as the fallback (`--iced`, or the setting
            // `shell-iced=on`).
            let iced = std::env::args().any(|a| a == "--iced") || mk::settings::read("shell-iced").as_deref() == Some("on");
            if light_path || !iced {
                if let Err(e) = licht::run() {
                    eprintln!("[tastatur] Erstes Licht: {e}");
                }
                return Ok(());
            }
            area()
        }
        Some("--typetest") => {
            let text: String = std::env::args().skip(2).collect::<Vec<_>>().join(" ");
            injection::type_test(&text);
            Ok(())
        }
        Some("show") => {
            mkw::keyboard::send("show");
            Ok(())
        }
        Some("hide") => {
            // The guard understands «hide» (it sent «zu», which nobody read).
            mkw::keyboard::send("hide");
            Ok(())
        }
        _ => {
            guard();
            Ok(())
        }
    }
}

// ------------------------------------------------------------------ Wache

/// Is a PHYSICAL keyboard attached? Kernel truth instead of guessing:
/// a block in /proc/bus/input/devices with kbd handler, EV_KEY AND
/// EV_REP (only real keyboards repeat) — video bus and power
/// buttons fall through, virtual devices anyway.
/// Fresh controller activity (R92): matrix-pad touches the marker
/// file on every input — whoever focuses a field with the pad wants
/// the on-screen keyboard, even with a physical one attached.
fn pad_active() -> bool {
    matrixkit_theme::runtime::fresh("matrix-pad-aktiv", 8_000)
}

fn physical_keyboard() -> bool {
    let Ok(content) = std::fs::read_to_string("/proc/bus/input/devices") else {
        // When in doubt, never FORCE a keyboard on anyone.
        return true;
    };
    content.split("\n\n").any(is_real_keyboard)
}

/// The letter row Q..P lives in bits 16..=25 of the low KEY word — no
/// device without letters is a keyboard someone can type text on.
/// R149b (the greeter probe on the naked Surface): gpio-keys passes
/// the EV_KEY+EV_REP test with flying colors, yet its whole KEY map
/// is three buttons — power and volume. Buttons repeat; letters decide.
const LETTER_ROW: u64 = 0x3ff_0000;

fn is_real_keyboard(block: &str) -> bool {
    let kbd = block
        .lines()
        .any(|z| z.starts_with("H:") && z.contains("kbd"));
    let virt = block
        .lines()
        .any(|z| z.starts_with("N:") && z.to_lowercase().contains("virtual"));
    let ev = block
        .lines()
        .find_map(|z| z.trim().strip_prefix("B: EV="))
        .and_then(|h| u64::from_str_radix(h.trim(), 16).ok())
        .unwrap_or(0);
    // Low word (bits 0..63) is the LAST entry of the KEY bitmap line.
    let key_low = block
        .lines()
        .find_map(|z| z.trim().strip_prefix("B: KEY="))
        .and_then(|h| h.split_whitespace().last())
        .and_then(|w| u64::from_str_radix(w, 16).ok())
        .unwrap_or(0);
    kbd && !virt
        && (ev & 0x2 != 0)
        && (ev & 0x10_0000 != 0)
        && (key_low & LETTER_ROW != 0)
}

/// Guard log to /tmp (per user) — survives the greeter and is
/// readable by anyone after login. Deliberately small: only changes.
fn guard_log(line: &str) {
    // Round 41: a bounded trail (last 300 lines) — the file in /tmp grew
    // without end (551 KB in a RAM folder, 03.11.2026).
    mk::runtime::trail("matrix-keyboard-wache.log", line, 300);
}

fn child_quit(child: &mut Option<std::process::Child>) {
    if let Some(mut k) = child.take() {
        let _ = k.kill();
        let _ = k.wait();
    }
}

/// The daemon: collect heartbeats, check the keyboard situation, run
/// the surface. Silence > 4 s means blur (the heartbeat comes from the
/// cursor blink of focused fields, every ~700 ms).
fn guard() {
    use std::os::unix::net::UnixDatagram;
    let socket = mkw::keyboard::socket();
    let _ = std::fs::remove_file(&socket);
    // Round 178: a busy/unwritable socket path no longer panics the guard at start — retry briefly, then give up
    // quietly (the keyboard itself keeps working, only the guard is missing).
    let mut tries = 0;
    let sock = loop {
        match UnixDatagram::bind(&socket) {
            Ok(s) => break s,
            Err(e) if tries < 10 => {
                tries += 1;
                eprintln!("[keyboard] Wache: Sockel {} belegt ({e}) — neuer Versuch", socket.display());
                std::thread::sleep(Duration::from_secs(1));
                let _ = std::fs::remove_file(&socket);
            }
            Err(e) => {
                eprintln!("[keyboard] Wache: Sockel nicht verfügbar ({e}) — ohne Wache weiter");
                return;
            }
        }
    };
    let _ = sock.set_read_timeout(Some(Duration::from_millis(500)));

    let mut child: Option<std::process::Child> = None;
    let mut spawn_pace = mk::time::Pace::new(Duration::from_secs(1));
    // When the surface started; a surface that lives but never shows its
    // layer is replaced (round 23 — the context menu had that sickness).
    let mut spawned_at: Option<Instant> = None;
    let mut seen_check = mk::time::Pace::new(Duration::from_secs(2));
    let mut update_check = mk::time::Pace::new(Duration::from_secs(10));
    let mut last_open = Instant::now() - Duration::from_secs(60);
    let mut last_touch = Instant::now() - Duration::from_secs(3600);
    // After ✕ or `verbergen`: only a blur (silence) re-arms.
    let mut locked = false;
    // After `zeigen`: visible despite a physical keyboard (handle/test).
    let mut forced = false;
    let mut phys = physical_keyboard();
    // R94b: start + every change land in a FILE — the greeter
    // journals its stderr nowhere, the hunt was blind.
    guard_log(&format!("gestartet, phys={phys} pad={}", pad_active()));
    let mut last_check = Instant::now();
    let mut puffer = [0u8; 64];

    loop {
        if let Ok(n) = sock.recv(&mut puffer) {
            match String::from_utf8_lossy(&puffer[..n]).trim() {
                "open" => last_open = Instant::now(),
                // Touch word: the field was focused BY TOUCH — the
                // physical-keyboard veto does not apply (Book 2 with
                // docked keyboard, iPad doctrine).
                "open!" => {
                    last_open = Instant::now();
                    last_touch = Instant::now();
                }
                "show" => {
                    forced = true;
                    locked = false;
                }
                // A fresh finger in a typing window (house, 03.11.2026): the
                // person wants the keyboard again — even after ✕.
                "tap" => {
                    if locked {
                        guard_log("frischer Tipp ins Feld — Sperre aufgehoben");
                    }
                    locked = false;
                    last_open = Instant::now();
                    last_touch = Instant::now();
                }
                "close" | "hide" => {
                    forced = false;
                    locked = true;
                    child_quit(&mut child);
                }
                _ => {}
            }
        }
        if last_check.elapsed() >= Duration::from_secs(2) {
            phys = physical_keyboard();
            last_check = Instant::now();
        }
        if let Some(k) = child.as_mut() {
            // mk::process::watch (R215): never alive for a process that is gone.
            match mk::process::watch(k) {
                mk::process::Watch::Exited(code) => {
                    guard_log(&format!("Tastatur beendet, Code {code:?}"));
                    if code == Some(MANUAL_CLOSE) {
                        locked = true;
                        forced = false;
                    }
                    child = None;
                }
                mk::process::Watch::Vanished => {
                    guard_log("Tastatur verschwunden ohne Status — neu bereit");
                    child = None;
                }
                mk::process::Watch::Running => {}
            }
        }
        let silence = last_open.elapsed() > Duration::from_secs(4);
        if silence {
            locked = false; // Blur unlocks — the next focus calls anew
        }
        let touch_wish = last_touch.elapsed() < Duration::from_secs(8);
        let goal = forced || ((!phys || pad_active() || touch_wish) && !locked && !silence);
        // Diagnose-Herzschlag (R92, Greeter-Jagd): jeden Zustandswechsel
        // into the journal — the next keyboardless boot tells its own story.
        if goal != child.is_some() {
            guard_log(&format!(
                "soll={goal} phys={phys} pad={} gesperrt={locked} stille={silence}",
                pad_active()
            ));
        }
        // At most one start a second: a surface that dies at once (a failed
        // exec, 02.11.2026: 8700 tries in two hours) must not spin the guard.
        if goal && child.is_some() && spawned_at.is_some_and(|t| t.elapsed() >= Duration::from_secs(4)) && seen_check.ready() {
            if mk::canvas::layer_present("matrix-keyboard") == Some(false) {
                guard_log("Tastatur läuft, aber ohne Fläche — ersetzt");
                child_quit(&mut child);
            } else {
                spawned_at = None; // seen once: healthy
            }
        }
        if goal && child.is_none() && spawn_pace.ready() {
            spawned_at = Some(Instant::now());
            if let Ok(itself) = matrixkit_theme::process::own_exe_result() {
                child = {
                    // The watcher passes on the PATH it was
                    // started with itself.
                    let mut c = std::process::Command::new(itself);
                    c.arg("--surface");
                    // Its stderr into a file (03.11.2026: the surface ended
                    // after 7 s and nobody could read why).
                    if let Some(f) = mk::runtime::log_file("matrix-keyboard-surface.log", 256 * 1024) {
                        c.stderr(f);
                    }
                    if std::env::args().any(|a| a == "--erstlicht") {
                        c.arg("--erstlicht");
                    }
                    c.spawn().ok()
                };
            }
        } else if !goal {
            child_quit(&mut child);
        }
        // Updated on disk and no keyboard showing: become the new version
        // (round 22 — before, a replaced guard ran old code until login).
        if child.is_none() && !goal && update_check.ready() && mk::process::updated() {
            guard_log("neue Fassung auf der Platte — Wächter startet sich neu");
            let e = mk::process::reexec();
            guard_log(&format!("Neustart misslang: {e}"));
        }
    }
}

// ------------------------------------------------------------ Belegung

#[derive(Debug, Clone)]
enum Action {
    Text(char),
    Shift,
    Delete,
    Enter,
    Leer,
    Page,
    Close,
}

struct KeyDef {
    display: String,
    units: f32,
    action: Action,
    accent: bool,
}

fn char_key(z: char) -> KeyDef {
    KeyDef { display: z.to_string(), units: 1.0, action: Action::Text(z), accent: false }
}

fn large(z: char) -> char {
    z.to_uppercase().next().unwrap_or(z)
}

/// The layout of a page — QWERTZ-CH with umlauts, page 1 = digits
/// and symbols incl. accented letters (é è à ç).
fn assignment(page: usize, shift: bool) -> Vec<Vec<KeyDef>> {
    let line = |s: &str| -> Vec<KeyDef> {
        s.chars()
            .map(|c| char_key(if shift { large(c) } else { c }))
            .collect()
    };
    let delete = KeyDef {
        display: "\u{232b}".into(),
        units: 1.5,
        action: Action::Delete,
        accent: false,
    };
    let bottom = |removable: &str| -> Vec<KeyDef> {
        vec![
            KeyDef { display: removable.into(), units: 1.5, action: Action::Page, accent: false },
            char_key(','),
            KeyDef { display: String::new(), units: 5.0, action: Action::Leer, accent: false },
            char_key('.'),
            KeyDef { display: "\u{23ce}".into(), units: 2.0, action: Action::Enter, accent: true },
            KeyDef { display: "\u{2715}".into(), units: 1.0, action: Action::Close, accent: false },
        ]
    };
    match page {
        0 => vec![
            line("qwertzuiopü"),
            line("asdfghjklöä"),
            {
                let mut z = vec![KeyDef {
                    display: "\u{21e7}".into(),
                    units: 1.5,
                    action: Action::Shift,
                    accent: shift,
                }];
                z.extend(line("yxcvbnm"));
                z.push(delete);
                z
            },
            bottom("?123"),
        ],
        // Symbols, two levels like a phone (#+= switches): the terminal needs
        // ~ | \ $ ' ! ? < > [ ] { } ^ ` (owner, 01.11.2026: «~ fehlt»).
        // Every character of every level is in the injection keymap
        // (`all_chars` walks both pages and both shift states).
        _ => {
            let (r1, r2, r3) = if shift {
                ("[]{}#%^*+=", "_\\|~<>\u{20ac}\u{00a3}`\u{00b0}", "\u{00e9}\u{00e8}\u{00e0}\u{00e7}\u{00a7}")
            } else {
                ("1234567890", "-/:;()$&@\"", "'!?=+*%")
            };
            let chars = |s: &str| -> Vec<KeyDef> { s.chars().map(char_key).collect() };
            vec![
                chars(r1),
                chars(r2),
                {
                    let mut z = vec![KeyDef {
                        display: if shift { "123".into() } else { "#+=".into() },
                        units: 1.5,
                        action: Action::Shift,
                        accent: shift,
                    }];
                    z.extend(chars(r3));
                    z.push(delete);
                    z
                },
                bottom("ABC"),
            ]
        }
    }
}

/// All characters this keyboard can ever type — for the keymap.
fn all_chars() -> Vec<char> {
    let mut all = vec![' '];
    for page in 0..2 {
        for shift in [false, true] {
            for line in assignment(page, shift) {
                for t in line {
                    if let Action::Text(z) = t.action {
                        all.push(z);
                    }
                }
            }
        }
    }
    all
}

// ------------------------------------------------------------- surface

fn area() -> Result<(), iced_layershell::Error> {
    // The surface lives only as long as its guard: killed or replaced, the
    // guard used to leave an orphan pill behind, still running the old
    // binary (seen 01.11.2026 on the duo). The kernel ends us with it.
    mkw::process::die_with_parent();
    use matrixkit_widgets::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
    use matrixkit_widgets::iced_layershell::settings::{LayerShellSettings, Settings};
    let edge = mkw::panel::SHADOW_MARGIN as u32;
    // R209 (Duo, the owner): a panel-wide keyboard on the other panel, one
    // window gap from edges and floor, and it RESERVES its height — a window
    // on that panel ends one gap above it instead of hiding behind it.
    let finger = mk::device::finger_first();
    let gap = mk::device::WINDOW_GAP as i32;
    // R94: pad bridge — datagram socket for controller commands.
    let _ = std::fs::write(open_marker(), b"1");
    let queue = PAD_COMMANDS.get_or_init(Default::default);
    let _ = queue; // Initialization suffices — the thread fills it.
    std::thread::spawn(|| {
        use std::os::unix::net::UnixDatagram;
        let path = pad_socket();
        let _ = std::fs::remove_file(&path);
        let Ok(sock) = UnixDatagram::bind(&path) else { return };
        let mut puffer = [0u8; 32];
        while let Ok(n) = sock.recv(&mut puffer) {
            let command = String::from_utf8_lossy(&puffer[..n]).trim().to_string();
            if let Some(q) = PAD_COMMANDS.get() {
                q.lock().unwrap_or_else(|e| e.into_inner()).push_back(command);
            }
        }
    });
    iced_layershell::application(
        App::new,
        || String::from("matrix-keyboard"),
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
            // Duo: no shadow room — the pill IS the surface, so the margins
            // are exactly the window gap (with the 40 px shadow room they went
            // negative and the pill sat flush on edge and window).
            size: Some(if finger { (0, PILL_H as u32) } else { (PILL_B as u32 + 2 * edge, PILL_H as u32 + 2 * edge) }),
            anchor: if finger { Anchor::Bottom | Anchor::Left | Anchor::Right } else { Anchor::Bottom },
            // Shadow room like the dock: the pill floats 10 px above the
            // edge, the surface reaches beyond it for the blur.
            margin: if finger {
                (0, gap, gap, gap)
            } else {
                (0, 0, 10 - mkw::panel::SHADOW_MARGIN as i32, 0)
            },
            start_mode: mkw::panel::target("keyboard-output", mk::device::DUO_KEYBOARD_PANEL),
            // R149c: Overlay, not Top — the greeter and the lock screen
            // are fullscreen Overlay surfaces, and an on-screen keyboard
            // that renders BELOW the very field it serves is no keyboard
            // (iOS doctrine: the keyboard lies above everything).
            layer: Layer::Overlay,
            // NEVER take keyboard focus — else you would type into nothing.
            keyboard_interactivity: KeyboardInteractivity::None,
            // The compositor reserves zone + margin: the pill plus one gap
            // below; niri's own gap then keeps the window one gap above.
            exclusive_zone: if finger { PILL_H as i32 } else { 0 },
            ..Default::default()
        },
        default_font: Font::with_name("Inter Variable"),
        fonts: mkw::load_symbol_fonts(),
        ..Default::default()
    })
    .run()
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Msg {
    FadeTick,
    Pulse,
    PadTick,
    FlashTick,
    Pressure(char),
    Shift,
    Delete,
    Enter,
    Leer,
    Page,
    Close,
}

struct App {
    palette: mk::Palette,
    watcher: mk::PaletteWatcher,
    shift: bool,
    page: usize,
    /// R94: pad focus (row, column) — None until the controller speaks.
    pad_focus: Option<(usize, usize)>,
    /// R168b afterglow: keys that just fired (display, moment) — the
    /// flash the finger cannot cover, visible as it lifts.
    flash: Vec<(String, std::time::Instant)>,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let _ = CHANNEL.set(injection::start(all_chars()));
        (
            App {
                palette: mk::Palette::load().unwrap_or_default(),
                watcher: mk::PaletteWatcher::new(),
                shift: false,
                page: 0,
                pad_focus: None,
                flash: Vec::new(),
            },
            Task::none(),
        )
    }

    fn inject(command: injection::Command) {
        if let Some(Some(channel)) = CHANNEL.get() {
            let _ = channel.send(command);
        }
        // Typing keeps the guard awake — even if an app currently
        // suspends the cursor blink (the heartbeat).
        mkw::keyboard::send("auf");
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::FadeTick | Msg::Pulse => {
                if matches!(msg, Msg::Pulse) {
                    let _ = std::fs::write(open_marker(), b"1");
                }
                if self.watcher.changed() {
                    if let Some(new) = mk::Palette::load() {
                        self.palette = new;
                    }
                }
                Task::none()
            }
            Msg::FlashTick => {
                self.flash.retain(|(_, t)| t.elapsed().as_millis() < 150);
                Task::none()
            }
            Msg::Pressure(z) => {
                Self::inject(injection::Command::Char(z));
                // R168b: the trio click + the afterglow — feedback the
                // finger cannot cover while it is still on the key.
                mk::feedback::key_press();
                self.flash.push((z.to_string(), std::time::Instant::now()));
                // Shift is one letter on the letter page; on the symbol page
                // «#+=» is a level that stays until switched back.
                if self.page == 0 {
                    self.shift = false;
                }
                Task::none()
            }
            Msg::Leer => {
                Self::inject(injection::Command::Char(' '));
                mk::feedback::key_press();
                self.flash.push((String::new(), std::time::Instant::now()));
                Task::none()
            }
            Msg::Delete => {
                Self::inject(injection::Command::Name("BackSpace"));
                mk::feedback::key_delete();
                self.flash.push((String::from("\u{232b}"), std::time::Instant::now()));
                Task::none()
            }
            Msg::Enter => {
                Self::inject(injection::Command::Name("Return"));
                mk::feedback::key_modifier();
                self.flash.push((String::from("\u{23ce}"), std::time::Instant::now()));
                Task::none()
            }
            Msg::Shift => {
                self.shift = !self.shift;
                mk::feedback::key_modifier();
                Task::none()
            }
            Msg::Page => {
                mk::feedback::key_modifier();
                self.page = 1 - self.page;
                self.shift = false;
                Task::none()
            }
            Msg::Close => {
                let _ = std::fs::remove_file(open_marker());
                std::process::exit(MANUAL_CLOSE)
            }
            Msg::PadTick => {
                let commands: Vec<String> = PAD_COMMANDS
                    .get()
                    .map(|q| q.lock().unwrap_or_else(|e| e.into_inner()).drain(..).collect())
                    .unwrap_or_default();
                for b in commands {
                    if let Some(t) = self.pad_step(&b) {
                        return self.update(t);
                    }
                }
                Task::none()
            }
            // variants added by the to_layer_message macro
            _ => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Msg> {
        let mut subs = vec![
            mkw::palette_fade_subscription().map(|_| Msg::FadeTick),
            mkw::tick("keyboard", Duration::from_secs(5)).map(|_| Msg::Pulse),
            // The pad queue needs 30 ms only while a controller is in use.
            mkw::tick_when("keyboard-pad", pad_active(), Duration::from_millis(30), Duration::from_millis(500))
                .map(|_| Msg::PadTick),
        ];
        if !self.flash.is_empty() {
            subs.push(mkw::tick("keyboard-flash", Duration::from_millis(50)).map(|_| Msg::FlashTick));
        }
        Subscription::batch(subs)
    }

    /// R94: apply a pad command to the focus; returns the Msg to type
    /// if "tipp" hits the focused key.
    fn pad_step(&mut self, command: &str) -> Option<Msg> {
        let grid = assignment(self.page, self.shift);
        let (mut r, mut c) = self.pad_focus.unwrap_or((0, 0));
        if self.pad_focus.is_none() && command != "zu" {
            // First contact: the ring appears without wandering.
            self.pad_focus = Some((0, 0));
            return None;
        }
        match command {
            "left" => c = c.saturating_sub(1),
            "right" => c += 1,
            "up" => r = r.saturating_sub(1),
            "down" => r += 1,
            "zu" => {
                let _ = std::fs::remove_file(open_marker());
                std::process::exit(MANUAL_CLOSE);
            }
            "tap" => {
                let def = &grid[r.min(grid.len() - 1)][c];
                return Some(match def.action {
                    Action::Text(z) => Msg::Pressure(z),
                    Action::Shift => Msg::Shift,
                    Action::Delete => Msg::Delete,
                    Action::Enter => Msg::Enter,
                    Action::Leer => Msg::Leer,
                    Action::Page => Msg::Page,
                    Action::Close => Msg::Close,
                });
            }
            _ => {}
        }
        r = r.min(grid.len() - 1);
        c = c.min(grid[r].len() - 1);
        self.pad_focus = Some((r, c));
        None
    }

    fn key_element(&self, def: &KeyDef, focused: bool, unit: f32) -> Element<'_, Msg> {
        let p = self.palette;
        let width = def.units * unit + (def.units - 1.0) * GAP;
        let msg = match def.action {
            Action::Text(z) => Msg::Pressure(z),
            Action::Shift => Msg::Shift,
            Action::Delete => Msg::Delete,
            Action::Enter => Msg::Enter,
            Action::Leer => Msg::Leer,
            Action::Page => Msg::Page,
            Action::Close => Msg::Close,
        };
        let accent = def.accent;
        // R168b afterglow: this key just fired — it glows in primary
        // for 150 ms, visible exactly as the finger lifts.
        let flashing = self
            .flash
            .iter()
            .any(|(d, t)| d == &def.display && t.elapsed().as_millis() < 150);
        let font = if flashing {
            p.on_primary
        } else if accent {
            p.on_primary
        } else {
            p.on_surface
        };
        let content = container(mkw::txt(def.display.clone(), mk::typo::BODY, font))
            .center_x(Length::Fill)
            .center_y(Length::Fill);
        let button = button(content)
            .width(Length::Fixed(width))
            .height(Length::Fixed(KEY_H))
            .padding(0)
            .on_press(msg.clone())
            .style(move |_, status| {
                let ground = if flashing {
                    p.primary
                } else if accent {
                    p.primary
                } else {
                    p.on_surface.over(p.surface_container, 0.08)
                };
                let bg = match status {
                    iced::widget::button::Status::Hovered => {
                        p.on_surface.over(ground, mk::state_layer::HOVER)
                    }
                    iced::widget::button::Status::Pressed => {
                        p.on_surface.over(ground, mk::state_layer::PRESSED)
                    }
                    _ => ground,
                };
                // family-exception: keyboard keys — own tile surface like dock tiles
                iced::widget::button::Style {
                    background: Some(color(bg).into()),
                    text_color: color(font),
                    border: iced::Border {
                        radius: mk::radius::SMALL.into(),
                        // R94: the pad focus ring — the same ring language as the
                        // keyboard navigation in the apps.
                        width: if focused { 2.0 } else { 0.0 },
                        color: color(p.primary),
                    },
                    ..Default::default()
                }
            });
        // R59 (UIKeyboard extract): the key types on TOUCH — the
        // button now only fires for mouse/trackpad on release.
        mkw::instant_button(button, msg)
    }

    fn view(&self) -> Element<'_, Msg> {
        if mk::device::finger_first() {
            // R209: the keys share the panel's width (11 units per row).
            return iced::widget::responsive(move |size| {
                let inner = size.width - 2.0 * MARGIN;
                // The widest row decides (the bottom row has more than 11
                // units — with 11 its last key stuck out of the pill).
                // A row of U units is U·unit + (U−1)·GAP wide.
                let units = assignment(self.page, self.shift)
                    .iter()
                    .map(|line| line.iter().map(|k| k.units).sum::<f32>())
                    .fold(11.0_f32, f32::max);
                let unit = ((inner - (units - 1.0) * GAP) / units).clamp(20.0, KEY);
                self.keyboard(unit, true)
            })
            .into();
        }
        self.keyboard(KEY, false)
    }

    fn keyboard(&self, unit: f32, panel_wide: bool) -> Element<'_, Msg> {
        let p = self.palette;
        let mut lines = column![].spacing(GAP);
        for (ri, line) in assignment(self.page, self.shift).iter().enumerate() {
            let mut r = row![].spacing(GAP);
            for (ci, def) in line.iter().enumerate() {
                let focused = self.pad_focus == Some((ri, ci));
                r = r.push(self.key_element(def, focused, unit));
            }
            lines = lines.push(container(r).center_x(Length::Fill));
        }
        let pill = container(lines)
            .padding(MARGIN)
            .width(if panel_wide { Length::Fill } else { Length::Fixed(PILL_B) })
            .style(move |_| mkw::panel::pill(p, mk::radius::SMALL, MARGIN));
        if panel_wide {
            return pill.height(Length::Fill).into();
        }
        let floating = mkw::panel::shadow_layers(pill.into(), mk::radius::SMALL + MARGIN);
        container(floating)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .align_y(iced::alignment::Vertical::Bottom)
            .padding(iced::Padding {
                bottom: mkw::panel::SHADOW_MARGIN,
                ..iced::Padding::ZERO
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::is_real_keyboard;

    /// Captured live on the Surface Book 2 greeter (R149b): the
    /// power/volume buttons pass EV_KEY+EV_REP but have no letters.
    #[test]
    fn surface_buttons_are_not_a_keyboard() {
        let volume = "N: Name=\"gpio-keys\"\nH: Handlers=kbd event4\nB: EV=100003\nB: KEY=c000000000000 0\n";
        let power = "N: Name=\"gpio-keys\"\nH: Handlers=kbd event5\nB: EV=3\nB: KEY=10000000000000 0\n";
        assert!(!is_real_keyboard(volume));
        assert!(!is_real_keyboard(power));
    }

    #[test]
    fn letter_keyboards_are_recognized() {
        // AT keyboard: EV with KEY+REP, letters in the low KEY word.
        let at = "N: Name=\"AT Translated Set 2 keyboard\"\nH: Handlers=sysrq kbd event2\nB: EV=120013\nB: KEY=402000000 3803078f800d001 feffffdfffefffff fffffffffffffffe\n";
        assert!(is_real_keyboard(at));
    }

    #[test]
    fn the_own_pad_is_no_keyboard() {
        // Matrix Pad: kbd handler, but EV=7 (no repeat) and no letters.
        let pad = "N: Name=\"Matrix Pad\"\nH: Handlers=kbd mouse3 event15\nB: EV=7\nB: KEY=30000 0 0 168000000000 200000010008002\n";
        assert!(!is_real_keyboard(pad));
    }
}
