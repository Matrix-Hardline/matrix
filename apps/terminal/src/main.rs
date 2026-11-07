//! Matrix Terminal — app #28 (R61), the own command line.
//!
//! Architecture like the big ones: `alacritty_terminal` is the
//! Maschinenhaus (PTY, Escape-Parser, Zellen-Gitter, Scrollback) —
//! a LIBRARY, not a foreign window. MatrixKit renders the grid
//! itself: frame with lights, living palette, the system mono from the
//! image. The ANSI base colors are tuned to the Matrix dark world;
//! foreground/background follow the palette live.
//!
//! Deliberate deviations from the kit standard:
//! * NO single instance — terminals come in packs.
//! * Ctrl+W belongs to the shell (kill-word), not to the window —
//!   the frame runs with `abo_mit(false)`.

use matrixkit_widgets::iced;

mod licht;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex, OnceLock};

use alacritty_terminal::event::{Event as TermEvent, EventListener, Notify, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, Msg as LoopsMsg, Notifier};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Point as GridPoint, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{viewport_to_point, Config as TermConfig, Term, TermMode};
use alacritty_terminal::tty;
use alacritty_terminal::vte::ansi::{Color as AnsiColor, NamedColor};
use matrixkit_widgets::iced::widget::{column, container};
use matrixkit_widgets::iced::{Element, Length, Subscription, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use matrixkit_widgets::color;

const APP_ID: &str = "matrix-terminal";

/// Adwaita Mono at 13 px: advance 0.6 em (measured: 600/1000 upm,
/// same as Maple before it), line fixed 18 px — the
/// cell measures from which PTY columns and rows are computed.
const FONT: f32 = 13.0;
const CELL_B: f32 = FONT * 0.6;
const CELL_H: f32 = 18.0;
/// Inner air of the grid + room of the frame head (lights/title).
const MARGIN_X: f32 = 28.0;
const MARGIN_Y: f32 = 68.0;


fn main() -> iced::Result {
    // Camp 95: the DEFAULT SWITCH. The window house is now the
    // NORMAL PATH of the terminal — the first app in which the
    // shadow doctrine has reached its goal. iced stays reachable as a
    // fallback (`--iced`) and as a safety net: if the light cannot
    // START, the launch falls back to iced instead of leaving
    // The owner without a terminal. `--erstlicht` stays
    // accepted (scripts and habits do not break).
    if !std::env::args().any(|a| a == "--iced") {
        match licht::run() {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("[terminal] Erstes Licht scheitert ({e}) — Rueckfall auf iced");
            }
        }
    }
    // The terminal deliberately runs multiple instances and speaks mono.
    mkw::app::window_with::<App>(
        APP_ID,
        840.0,
        560.0,
        mkw::app::Options { single_instance: false, mono_font: true, ..Default::default() },
    )
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
        self.title.clone()
    }
    fn subscription(&self) -> iced::Subscription<Msg> {
        App::subscription(self)
    }
}

// ------------------------------------------------------ Maschinenhaus

/// Grid size for the backend.
#[derive(Clone, Copy)]
pub(crate) struct Mass {
    pub(crate) columns: usize,
    pub(crate) lines: usize,
}

impl Dimensions for Mass {
    fn total_lines(&self) -> usize {
        self.lines
    }
    fn screen_lines(&self) -> usize {
        self.lines
    }
    fn columns(&self) -> usize {
        self.columns
    }
}

/// Terminal events travel into the iced world via a channel.
#[derive(Clone)]
pub(crate) struct Courier(pub(crate) mpsc::Sender<TermEvent>);

impl EventListener for Courier {
    fn send_event(&self, event: TermEvent) {
        let _ = self.0.send(event);
    }
}

/// The receiver for the iced subscription (plugged once).
pub(crate) static EVENTS: OnceLock<Mutex<mpsc::Receiver<TermEvent>>> = OnceLock::new();

fn event_subscription() -> Subscription<TermEvent> {
    Subscription::run(|| {
        let (tx, rx) = iced::futures::channel::mpsc::unbounded();
        std::thread::spawn(move || {
            let Some(source) = EVENTS.get() else { return };
            let source = source.lock().unwrap_or_else(|e| e.into_inner());
            while let Ok(e) = source.recv() {
                if tx.unbounded_send(e).is_err() {
                    return;
                }
            }
        });
        rx
    })
}

pub(crate) struct Machine {
    pub(crate) term: Arc<FairMutex<Term<Courier>>>,
    writer: Notifier,
    pub(crate) mass: Mass,
}

impl Machine {
    pub(crate) fn start(mass: Mass) -> Self {
        let (tx, rx) = mpsc::channel();
        let _ = EVENTS.set(Mutex::new(rx));
        let bote = Courier(tx);

        let mut environment = HashMap::new();
        // Universal terminfo — the alacritty entry is missing on the image.
        environment.insert(String::from("TERM"), String::from("xterm-256color"));
        environment.insert(String::from("COLORTERM"), String::from("truecolor"));

        let window = WindowSize {
            num_cols: mass.columns as u16,
            num_lines: mass.lines as u16,
            cell_width: CELL_B as u16,
            cell_height: CELL_H as u16,
        };
        // R95: Matrix greeting BEFORE the shell — like macOS' "Last login"
        // it comes through the same PTY stream and scrolls away naturally.
        // The wrapper replaces itself with the login shell via exec; if
        // matrix-greeting is missing (dev env), the shell starts silently.
        let start = String::from(
            "command -v matrix-greeting >/dev/null && matrix-greeting; \
             exec \"${SHELL:-$(getent passwd \"$(id -un)\" | cut -d: -f7)}\" -l",
        );
        let options = tty::Options {
            shell: Some(tty::Shell::new(
                String::from("/bin/sh"),
                vec![String::from("-c"), start],
            )),
            working_directory: None,
            hold: false,
            env: environment,
        };
        let pty = tty::new(&options, window, 0).expect("matrix-terminal: PTY");
        // Generous history: assistant sessions produce long transcripts,
        // and scrolling back through them is the whole point (R201).
        let config = TermConfig { scrolling_history: 50_000, ..TermConfig::default() };
        let term = Arc::new(FairMutex::new(Term::new(config, &mass, bote.clone())));
        let run_loop = EventLoop::new(term.clone(), bote, pty, false, false)
            .expect("matrix-terminal: Ereignisschleife");
        let writer = Notifier(run_loop.channel());
        let _ = run_loop.spawn();
        Self { term, writer, mass }
    }

    pub(crate) fn inject(&self, bytes: Vec<u8>) {
        if !bytes.is_empty() {
            self.writer.notify(bytes);
        }
    }

    pub(crate) fn measure(&mut self, width: f32, height: f32) {
        let columns = ((width - MARGIN_X) / CELL_B).floor().max(20.0) as usize;
        let lines = ((height - MARGIN_Y) / CELL_H).floor().max(5.0) as usize;
        if columns == self.mass.columns && lines == self.mass.lines {
            return;
        }
        self.mass = Mass { columns, lines };
        let window = WindowSize {
            num_cols: columns as u16,
            num_lines: lines as u16,
            cell_width: CELL_B as u16,
            cell_height: CELL_H as u16,
        };
        let _ = self.writer.0.send(LoopsMsg::Resize(window));
        self.term.lock().resize(self.mass);
    }
}

// ------------------------------------------------------------ Farben

/// The 16 ANSI base colors in the Matrix dark world — muted but
/// distinguishable tones (Material kinship of the kit palette).
pub(crate) const ANSI: [(u8, u8, u8); 16] = [
    (0x2a, 0x2c, 0x33), // schwarz
    (0xf2, 0x8b, 0x82), // rot
    (0x81, 0xc9, 0x95), // green
    (0xfd, 0xd6, 0x63), // gelb
    (0x8a, 0xb4, 0xf8), // blau
    (0xd7, 0xae, 0xfb), // magenta
    (0x78, 0xd9, 0xec), // cyan
    (0xdd, 0xe1, 0xe6), // white
    (0x48, 0x4b, 0x55), // hell-schwarz
    (0xf6, 0xae, 0xa9),
    (0xa8, 0xda, 0xb5),
    (0xfd, 0xe2, 0x93),
    (0xae, 0xcb, 0xfa),
    (0xe4, 0xc7, 0xfc),
    (0xa1, 0xe4, 0xf2),
    (0xf8, 0xf9, 0xfa), // bright white
];

fn index_color(i: u8) -> iced::Color {
    match i {
        0..=15 => {
            let (r, g, b) = ANSI[i as usize];
            // family-exception: ANSI-Terminalpalette, bewusst palettenfrei
            iced::Color::from_rgb8(r, g, b)
        }
        16..=231 => {
            // 6×6×6 cube
            let i = i - 16;
            let level = |n: u8| if n == 0 { 0 } else { 55 + n * 40 };
            // family-exception: ANSI-256 color cube, deliberately palette-free
            iced::Color::from_rgb8(level(i / 36), level((i / 6) % 6), level(i % 6))
        }
        _ => {
            let g = 8 + (i - 232) * 10;
            // family-exception: ANSI-Graustufenrampe, bewusst palettenfrei
            iced::Color::from_rgb8(g, g, g)
        }
    }
}

/// The same mapping in mk::Rgba — the First Light path does not know
/// iced's colour type. The rules live here once so that the
/// two paths do not drift apart (camp 86).
pub(crate) fn ansi_rgba(ansi: AnsiColor, p: mk::Palette, foreground: bool) -> Option<mk::Rgba> {
    let acht = |r: u8, g: u8, b: u8| mk::Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    };
    match ansi {
        AnsiColor::Spec(rgb) => Some(acht(rgb.r, rgb.g, rgb.b)),
        AnsiColor::Indexed(i) => {
            let c = index_color(i);
            Some(mk::Rgba { r: c.r, g: c.g, b: c.b, a: c.a })
        }
        AnsiColor::Named(n) => match n {
            NamedColor::Foreground => Some(p.on_surface),
            NamedColor::Background => foreground.then_some(p.surface),
            NamedColor::Cursor => Some(p.primary),
            NamedColor::BrightForeground => Some(p.on_surface),
            NamedColor::DimForeground => Some(p.on_surface_variant),
            _ => {
                let i = n as usize;
                (i < 16).then(|| {
                    let (r, g, b) = ANSI[i];
                    acht(r, g, b)
                })
            }
        },
    }
}

fn ansi_color(ansi: AnsiColor, p: mk::Palette, foreground: bool) -> Option<iced::Color> {
    match ansi {
        // family-exception: program-chosen true color, deliberately palette-free
        AnsiColor::Spec(rgb) => Some(iced::Color::from_rgb8(rgb.r, rgb.g, rgb.b)),
        AnsiColor::Indexed(i) => Some(index_color(i)),
        AnsiColor::Named(n) => match n {
            NamedColor::Foreground => Some(color(p.on_surface)),
            NamedColor::Background => {
                // Default background = window surface → no span ground needed.
                if foreground { Some(color(p.surface)) } else { None }
            }
            NamedColor::Cursor => Some(color(p.primary)),
            NamedColor::BrightForeground => Some(color(p.on_surface)),
            NamedColor::DimForeground => Some(color(p.on_surface_variant)),
            _ => {
                let i = n as usize;
                (i < 16).then(|| {
                    let (r, g, b) = ANSI[i];
                    // family-exception: ANSI-Terminalpalette, bewusst palettenfrei
                    iced::Color::from_rgb8(r, g, b)
                })
            }
        },
    }
}

// ------------------------------------------------------- Tasten → PTY

/// Translate a pressed key into the bytes an xterm-like application
/// expects. `text` is iced's finished translation (Shift, AltGr,
/// umlauts) — control combinations take precedence.
fn key_to_bytes(
    key: &iced::keyboard::Key,
    mods: iced::keyboard::Modifiers,
    text: Option<&str>,
) -> Option<Vec<u8>> {
    use matrixkit_widgets::iced::keyboard::key::Named;
    use matrixkit_widgets::iced::keyboard::Key;

    // Control glyph: Ctrl+a..z → 0x01..0x1a (Ctrl+Shift stays free
    // for future copy/paste).
    if mods.control() && !mods.shift() {
        if let Key::Character(c) = key {
            let mut glyph = c.chars();
            if let (Some(z), None) = (glyph.next(), glyph.next()) {
                let small = z.to_ascii_lowercase();
                if small.is_ascii_lowercase() {
                    return Some(vec![(small as u8) & 0x1f]);
                }
                match small {
                    ' ' | '@' => return Some(vec![0x00]),
                    '[' => return Some(vec![0x1b]),
                    '\\' => return Some(vec![0x1c]),
                    ']' => return Some(vec![0x1d]),
                    _ => {}
                }
            }
        }
    }

    let esc = |s: &str| Some(format!("\x1b{s}").into_bytes());
    if let Key::Named(n) = key {
        return match n {
            Named::Enter => Some(b"\r".to_vec()),
            Named::Backspace => Some(vec![0x7f]),
            Named::Tab if mods.shift() => esc("[Z"),
            Named::Tab => Some(b"\t".to_vec()),
            Named::Escape => Some(vec![0x1b]),
            Named::ArrowUp => esc("[A"),
            Named::ArrowDown => esc("[B"),
            Named::ArrowRight => esc("[C"),
            Named::ArrowLeft => esc("[D"),
            Named::Home => esc("[H"),
            Named::End => esc("[F"),
            Named::PageUp => esc("[5~"),
            Named::PageDown => esc("[6~"),
            Named::Delete => esc("[3~"),
            Named::Insert => esc("[2~"),
            Named::Space => Some(b" ".to_vec()),
            _ => None,
        };
    }

    // Normal text — Alt sends the ESC prefix ahead (meta).
    let t = text?;
    if t.is_empty() || t.chars().next().is_some_and(|c| (c as u32) < 0x20) {
        return None;
    }
    let mut bytes = Vec::new();
    if mods.alt() {
        bytes.push(0x1b);
    }
    bytes.extend_from_slice(t.as_bytes());
    Some(bytes)
}

// --------------------------------------------------------------- App

#[derive(Debug, Clone)]
enum Msg {
    Frame(mkw::FrameMsg),
    Term(std::sync::Arc<TermEvent>),
    Key(iced::keyboard::Key, iced::keyboard::Modifiers, Option<String>),
    Wheel(f32),
    Inserted(Option<String>),
    Mods(iced::keyboard::Modifiers),
    /// Mouse over the grid — coordinates relative to the grid's
    /// top-left cell, courtesy of mouse_area (no window math).
    GridMoved(iced::Point),
    GridPressed,
    GridReleased,
}

/// One mouse report the application asked for (DECSET 1000/1002/1003)
/// — SGR encoding when announced, legacy X10 bytes otherwise.
pub(crate) fn mouse_report(
    mode: TermMode,
    button: u8,
    pressed: bool,
    line: usize,
    column: usize,
) -> Vec<u8> {
    if mode.contains(TermMode::SGR_MOUSE) {
        let end = if pressed { 'M' } else { 'm' };
        format!("\x1b[<{};{};{}{end}", button, column + 1, line + 1).into_bytes()
    } else {
        let code = 32 + if pressed { button } else { 3 };
        let cx = 32 + (column + 1).min(222) as u8;
        let cy = 32 + (line + 1).min(222) as u8;
        vec![0x1b, b'[', b'M', code, cx, cy]
    }
}

struct App {
    frame: mkw::Frame,
    machine: Machine,
    title: String,
    /// Window inner size — for the grid measurement.
    area: (f32, f32),
    /// Fractional wheel remainder — touchpads deliver part-lines,
    /// and truncating them made scrolling feel dead (R201).
    scroll_rest: f32,
    /// Cell + side under the mouse, kept for press/drag.
    hover: Option<(usize, usize, Side)>,
    /// A selection drag is in progress.
    selecting: bool,
    /// Live modifier state — Shift bypasses mouse reporting so
    /// selection stays reachable inside htop & friends.
    mods: iced::keyboard::Modifiers,
    /// A press was reported to the application (its release must be).
    reported_press: bool,
    /// The shell has left — the window follows it.
    ended: bool,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let mass = Mass {
            columns: ((840.0 - MARGIN_X) / CELL_B) as usize,
            lines: ((560.0 - MARGIN_Y) / CELL_H) as usize,
        };
        (
            App {
                frame: mkw::Frame::new(APP_ID, &[]),
                machine: Machine::start(mass),
                title: String::from("Matrix Terminal"),
                area: (840.0, 560.0),
                scroll_rest: 0.0,
                hover: None,
                selecting: false,
                mods: iced::keyboard::Modifiers::default(),
                reported_press: false,
                ended: false,
            },
            Task::none(),
        )
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Frame(r) => {
                if let mkw::FrameMsg::Size(g) = &r {
                    self.area = (g.width, g.height);
                    self.machine.measure(g.width, g.height);
                }
                self.frame.update(r).map(Msg::Frame)
            }
            Msg::Term(event) => {
                match &*event {
                    TermEvent::PtyWrite(text) => {
                        self.machine.inject(text.clone().into_bytes())
                    }
                    TermEvent::Title(t) => self.title = format!("{t} — Matrix Terminal"),
                    TermEvent::Bell => {
                        let _ = mk::feedback::now("terminal", "04-benachrichtigung-leise.wav");
                    }
                    TermEvent::Exit => {
                        self.ended = true;
                        return iced::window::latest().and_then(iced::window::close);
                    }
                    // OSC 52: programs put text on the clipboard — a CLI
                    // assistant handing over a command shouldn't need a mouse.
                    TermEvent::ClipboardStore(_, text) => {
                        return iced::clipboard::write(text.clone());
                    }
                    _ => {} // Wakeup & co: redrawing happens anyway
                }
                Task::none()
            }
            Msg::Key(key, mods, text) => {
                if self.frame.root.is_open() {
                    return Task::none(); // the root layer catches the keyboard
                }
                // Copy/Paste: Ctrl+Shift+C/V — the terminal convention.
                if mods.control() && mods.shift() {
                    if let iced::keyboard::Key::Character(c) = &key {
                        if c.as_str().eq_ignore_ascii_case("v") {
                            return iced::clipboard::read().map(Msg::Inserted);
                        }
                        if c.as_str().eq_ignore_ascii_case("c") {
                            if let Some(text) = self.machine.term.lock().selection_to_string() {
                                if !text.is_empty() {
                                    return iced::clipboard::write(text);
                                }
                            }
                            return Task::none();
                        }
                    }
                }
                // Shift + navigation scrolls the history instead of the
                // application — the way every terminal has it.
                if mods.shift() {
                    use matrixkit_widgets::iced::keyboard::key::Named;
                    if let iced::keyboard::Key::Named(n) = &key {
                        let jump = match n {
                            Named::PageUp => Some(Scroll::PageUp),
                            Named::PageDown => Some(Scroll::PageDown),
                            Named::Home => Some(Scroll::Top),
                            Named::End => Some(Scroll::Bottom),
                            _ => None,
                        };
                        if let Some(jump) = jump {
                            self.machine.term.lock().scroll_display(jump);
                            return Task::none();
                        }
                    }
                }
                if let Some(bytes) = key_to_bytes(&key, mods, text.as_deref()) {
                    // Input jumps to the end of the history (terminal custom).
                    self.machine.term.lock().scroll_display(Scroll::Bottom);
                    self.machine.inject(bytes);
                }
                Task::none()
            }
            Msg::Wheel(lines) => {
                // Accumulate — touchpads send part-lines, and Delta(0)
                // was the "cannot scroll" of R201.
                self.scroll_rest += lines;
                let whole = self.scroll_rest.trunc() as i32;
                if whole == 0 {
                    return Task::none();
                }
                self.scroll_rest -= whole as f32;
                let mode = *self.machine.term.lock().mode();
                // The application asked for the mouse (htop, tmux …):
                // the wheel is ITS event. Shift keeps the terminal's own
                // scrolling reachable regardless.
                if mode.intersects(TermMode::MOUSE_MODE) && !self.mods.shift() {
                    if let Some((line, column, _)) = self.hover {
                        let button = if whole > 0 { 64 } else { 65 };
                        let mut bytes = Vec::new();
                        for _ in 0..whole.unsigned_abs() {
                            bytes.extend_from_slice(&mouse_report(mode, button, true, line, column));
                        }
                        self.machine.inject(bytes);
                    }
                } else if mode.contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL)
                    && !self.mods.shift()
                {
                    // Full-screen without mouse interest (less, vim):
                    // the wheel becomes arrow keys — xterm custom.
                    let code: &[u8] = if whole > 0 { b"\x1b[A" } else { b"\x1b[B" };
                    let mut bytes = Vec::new();
                    for _ in 0..whole.unsigned_abs() {
                        bytes.extend_from_slice(code);
                    }
                    self.machine.inject(bytes);
                } else {
                    self.machine.term.lock().scroll_display(Scroll::Delta(whole));
                }
                Task::none()
            }
            Msg::GridMoved(at) => {
                let columns = self.machine.mass.columns;
                let lines = self.machine.mass.lines;
                let column = ((at.x / CELL_B) as usize).min(columns.saturating_sub(1));
                let line = ((at.y / CELL_H) as usize).min(lines.saturating_sub(1));
                let side = if at.x - (column as f32) * CELL_B > CELL_B / 2.0 {
                    Side::Right
                } else {
                    Side::Left
                };
                self.hover = Some((line, column, side));
                if self.reported_press {
                    let mode = *self.machine.term.lock().mode();
                    if mode.intersects(TermMode::MOUSE_DRAG | TermMode::MOUSE_MOTION) {
                        self.machine.inject(mouse_report(mode, 32, true, line, column));
                    }
                } else if self.selecting {
                    let mut term = self.machine.term.lock();
                    let point = viewport_to_point(
                        term.grid().display_offset(),
                        GridPoint::new(line, Column(column)),
                    );
                    if let Some(sel) = term.selection.as_mut() {
                        sel.update(point, side);
                    }
                }
                Task::none()
            }
            Msg::GridPressed => {
                if let Some((line, column, side)) = self.hover {
                    let mode = *self.machine.term.lock().mode();
                    if mode.intersects(TermMode::MOUSE_MODE) && !self.mods.shift() {
                        self.machine.inject(mouse_report(mode, 0, true, line, column));
                        self.reported_press = true;
                    } else {
                        let mut term = self.machine.term.lock();
                        let point = viewport_to_point(
                            term.grid().display_offset(),
                            GridPoint::new(line, Column(column)),
                        );
                        term.selection = Some(Selection::new(SelectionType::Simple, point, side));
                        self.selecting = true;
                    }
                }
                Task::none()
            }
            Msg::GridReleased => {
                if self.reported_press {
                    self.reported_press = false;
                    if let Some((line, column, _)) = self.hover {
                        let mode = *self.machine.term.lock().mode();
                        self.machine.inject(mouse_report(mode, 0, false, line, column));
                    }
                }
                self.selecting = false;
                Task::none()
            }
            Msg::Inserted(content) => {
                if let Some(t) = content {
                    // Bracketed paste — without the markers a multi-line
                    // paste runs line by line the moment it lands.
                    let bracketed =
                        self.machine.term.lock().mode().contains(TermMode::BRACKETED_PASTE);
                    let mut bytes = Vec::new();
                    if bracketed {
                        bytes.extend_from_slice(b"\x1b[200~");
                    }
                    bytes.extend_from_slice(t.as_bytes());
                    if bracketed {
                        bytes.extend_from_slice(b"\x1b[201~");
                    }
                    self.machine.inject(bytes);
                }
                Task::none()
            }
            Msg::Mods(m) => {
                self.mods = m;
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Msg> {
        if self.ended {
            return Subscription::none();
        }
        let keys = iced::event::listen_with(|event, _status, _window| {
            use matrixkit_widgets::iced::keyboard::Event as K;
            use matrixkit_widgets::iced::mouse::Event as M;
            match event {
                iced::Event::Keyboard(K::KeyPressed { key, modifiers, text, .. }) => Some(
                    Msg::Key(key, modifiers, text.map(|t| t.to_string())),
                ),
                iced::Event::Keyboard(K::ModifiersChanged(m)) => Some(Msg::Mods(m)),
                iced::Event::Mouse(M::WheelScrolled { delta }) => {
                    // Every delta counts — the accumulator in update()
                    // turns part-lines into whole ones (R201).
                    let lines = match delta {
                        iced::mouse::ScrollDelta::Lines { y, .. } => y * 3.0,
                        iced::mouse::ScrollDelta::Pixels { y, .. } => y / CELL_H,
                    };
                    (lines != 0.0).then(|| Msg::Wheel(lines))
                }
                _ => None,
            }
        });
        Subscription::batch([
            // Ctrl+W belongs to the shell — frame without window keys.
            self.frame.subscription_with(false).map(Msg::Frame),
            event_subscription().map(|e| Msg::Term(std::sync::Arc::new(e))),
            keys,
        ])
    }

    /// The grid as rich_text rows: runs of the same color become ONE
    /// span — 24 rows × few runs instead of 1920 single cells.
    fn grid(&self) -> Element<'_, Msg> {
        use alacritty_terminal::term::cell::Flags;
        use matrixkit_widgets::iced::widget::span;
        let p = self.frame.palette;
        let term = self.machine.term.lock();
        let content = term.renderable_content();
        let cursor = content.cursor.point;
        let chosen = content.selection;
        let lines_total = self.machine.mass.lines;
        let columns = self.machine.mass.columns;

        // Zellen in Zeilenpuffer einsammeln.
        let mut grid: Vec<Vec<(char, Option<iced::Color>, Option<iced::Color>)>> =
            vec![vec![(' ', None, None); columns]; lines_total];
        for indexed in content.display_iter {
            let line = indexed.point.line.0;
            let column = indexed.point.column.0;
            if line < 0 || line as usize >= lines_total || column >= columns {
                continue;
            }
            let cell = &indexed;
            if cell.flags.contains(Flags::HIDDEN) {
                continue;
            }
            let mut fg = ansi_color(cell.fg, p, true);
            let mut bg = ansi_color(cell.bg, p, false);
            if cell.flags.contains(Flags::INVERSE) {
                let old_fg = fg;
                fg = Some(bg.unwrap_or(color(p.surface)));
                bg = Some(old_fg.unwrap_or(color(p.on_surface)));
            }
            // Selection: container ground under the chosen run — the
            // kit's marking color, distinct from the primary cursor.
            if chosen.is_some_and(|s| s.contains(indexed.point)) {
                fg = Some(color(p.on_primary_container));
                bg = Some(color(p.primary_container));
            }
            if cell.flags.contains(Flags::DIM) {
                if let Some(f) = fg.as_mut() {
                    f.a *= 0.6;
                }
            }
            // Cursor: the block in primary — the one kit accent spot.
            if line == cursor.line.0 && column == cursor.column.0 {
                fg = Some(color(p.on_primary));
                bg = Some(color(p.primary));
            }
            grid[line as usize][column] = (cell.c, fg, bg);
        }
        drop(term);

        let mut column_layout = column![].spacing(0.0);
        for line in &grid {
            let mut spans = Vec::new();
            let mut run = String::new();
            let mut run_style: (Option<iced::Color>, Option<iced::Color>) = (None, None);
            for (z, fg, bg) in line {
                let style = (*fg, *bg);
                if style != run_style && !run.is_empty() {
                    spans.push(span_build(std::mem::take(&mut run), run_style, p));
                }
                run_style = style;
                run.push(*z);
            }
            if !run.is_empty() {
                spans.push(span_build(run, run_style, p));
            }
            column_layout = column_layout.push(
                iced::widget::rich_text(spans)
                    .font(mkw::mono())
                    .size(FONT)
                    .line_height(iced::widget::text::LineHeight::Absolute(CELL_H.into()))
                    .wrapping(iced::widget::text::Wrapping::None),
            );
        }

        fn span_build<'a>(
            text: String,
            (fg, bg): (Option<iced::Color>, Option<iced::Color>),
            p: mk::Palette,
        ) -> iced::widget::text::Span<'a, Msg> {
            let mut s = span(text).color(fg.unwrap_or(color(p.on_surface)));
            if let Some(b) = bg {
                s = s.background(b);
            }
            s
        }

        // mouse_area speaks grid-relative coordinates — selection needs
        // no window geometry, only the cell measures.
        let grid_area = iced::widget::mouse_area(column_layout)
            .on_move(Msg::GridMoved)
            .on_press(Msg::GridPressed)
            .on_release(Msg::GridReleased);

        container(grid_area)
            .padding(iced::Padding { top: 6.0, right: 10.0, bottom: 10.0, left: 12.0 })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view(&self) -> Element<'_, Msg> {
        self.frame.window(
            "Matrix Terminal",
            "1.0",
            "Die Kommandozeile in der MatrixKit-Grammatik — \
             alacritty-Maschinenhaus, Matrix-Karosserie.",
            self.grid(),
            Msg::Frame,
        )
    }
}
