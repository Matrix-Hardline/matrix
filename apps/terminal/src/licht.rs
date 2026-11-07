//! Matrix Terminal, first-light version (iced replacement, Lager 86 —
//! GRABUNG-ICED-ABLOESUNG.md): the first BIG app on the window house.
//!
//! The machinery is verbatim that of the iced version — `Machine`
//! (alacritty_terminal, PTY, event loop) was free of iced from the
//! start. What is new is only WHO paints the grid: instead of `rich_text`
//! per line, the scene carries one text run per COLOUR RUN and, where
//! needed, a ground rectangle. That way the house's damage bookkeeping
//! takes hold — it paints only what has changed.
//!
//! What this move revealed about the kit: the key contract knew
//! only Shift. A terminal without Ctrl-C is none, so the house
//! grew [`shell::Mods`] and the missing keys.

use crate::{ansi_rgba, mouse_report, Machine, Mass, CELL_B, CELL_H, FONT, MARGIN_X, MARGIN_Y};
use matrixkit_shell::{self as shell, board, Key, Mods, Refresh, Scene, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{lay_out, Keep, Rect};
use matrixkit_widgets as mkw;
use matrixkit_widgets::scene as fam;

use alacritty_terminal::event::Event as TermEvent;
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::index::{Column, Point as GridPoint, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{viewport_to_point, TermMode};

const WIDTH: u32 = 840;
const HEIGHT: u32 = 560;
/// Roles from here on belong to the window frame (traffic lights, window key).
const FRAME_BASE: usize = 500;
/// Roles from here on belong to the root layer (Lager 93).
const ROOT_BASE: usize = 700;

struct Terminal {
    machine: Machine,
    palette: mk::Palette,
    /// The window skin (R210 family `fam::Chrome`): head, traffic lights, magnifier,
    /// roll up, maximise — on the Duo entirely without a head (bar menu).
    chrome: fam::Chrome,
    /// The cursor beat: the window house beats once a second.
    blink: bool,
    /// Where the grid starts (below the head) — view() learns it,
    /// pointer events need it.
    grid_top: f32,
    /// Which cell is under the pointer, and on which half?
    hover: Option<(usize, usize, Side)>,
    /// The wheel's remainder counter: trackballs deliver partial lines,
    /// and `Delta(0)` was the „cannot scroll" of R201.
    scroll_rest: f32,
    /// Is a selection drag running right now?
    selecting: bool,
    /// Did the press go to the program as a MOUSE report?
    /// Then the release belongs to it too, not to the selection.
    reported_press: bool,
    /// The modifiers of the last key — the wheel needs them, and
    /// the wheel axis carries none.
    mods: Mods,
    /// The ROOT LAYER (Lager 93): the title opens the
    /// app's dashboard.
    root: fam::RootPane,
}

impl Terminal {
    fn new() -> Self {
        // A first size, until the compositor sends its configure.
        let columns = ((WIDTH as f32 - MARGIN_X) / CELL_B).floor().max(20.0) as usize;
        let lines = ((HEIGHT as f32 - MARGIN_Y) / CELL_H).floor().max(5.0) as usize;
        Terminal {
            machine: Machine::start(Mass { columns, lines }),
            palette: mk::Palette::load().unwrap_or_default(),
            chrome: fam::Chrome::new("Matrix Terminal", FRAME_BASE),
            blink: true,
            grid_top: 0.0,
            hover: None,
            scroll_rest: 0.0,
            selecting: false,
            reported_press: false,
            mods: Mods::default(),
            root: fam::RootPane::new(crate::APP_ID, &[]),
        }
    }

    /// The pointer position as a GRID CELL — or None if it is
    /// over the head (the frame lives there, not the grid).
    fn cell_at(&self, x: f64, y: f64) -> Option<(usize, usize, Side)> {
        let left = MARGIN_X / 2.0;
        let dy = y as f32 - self.grid_top;
        let dx = x as f32 - left;
        if dy < 0.0 || dx < 0.0 {
            return None;
        }
        let columns = self.machine.mass.columns;
        let lines = self.machine.mass.lines;
        if columns == 0 || lines == 0 {
            return None;
        }
        let column = ((dx / CELL_B) as usize).min(columns - 1);
        let line = ((dy / CELL_H) as usize).min(lines - 1);
        let side = if dx - (column as f32) * CELL_B > CELL_B / 2.0 {
            Side::Right
        } else {
            Side::Left
        };
        Some((line, column, side))
    }

    /// The point in the HISTORY GRID: the cell on the screen plus
    /// the offset by which it was scrolled back.
    fn point_of(&self, line: usize, column: usize) -> GridPoint {
        let offset = self.machine.term.lock().grid().display_offset();
        viewport_to_point(offset, GridPoint::new(line, Column(column)))
    }

    /// PASTE with brackets: without the markers a
    /// multi-line text runs off line by line as soon as it lands.
    fn paste(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let bracketed = self.machine.term.lock().mode().contains(TermMode::BRACKETED_PASTE);
        let mut bytes = Vec::new();
        if bracketed {
            bytes.extend_from_slice(b"\x1b[200~");
        }
        bytes.extend_from_slice(text.as_bytes());
        if bracketed {
            bytes.extend_from_slice(b"\x1b[201~");
        }
        self.machine.term.lock().scroll_display(Scroll::Bottom);
        self.machine.inject(bytes);
    }

    /// A key into the bytes the program in the PTY expects.
    ///
    /// The rules are those of the iced version (`key_to_bytes`), just
    /// via the house's contract instead of iced's key type.
    fn bytes_for(key: &Key, mods: Mods) -> Option<Vec<u8>> {
        // Ctrl + letter: the classic control character.
        if mods.ctrl && !mods.shift {
            if let Key::Char(c) = key {
                let mut glyphs = c.chars();
                if let (Some(z), None) = (glyphs.next(), glyphs.next()) {
                    let small = z.to_ascii_lowercase();
                    if small.is_ascii_lowercase() {
                        return Some(vec![(small as u8) & 0x1f]);
                    }
                    match small {
                        ' ' | '@' => return Some(vec![0x00]),
                        '[' => return Some(vec![0x1b]),
                        '\\' => return Some(vec![0x1c]),
                        ']' => return Some(vec![0x1d]),
                        '^' => return Some(vec![0x1e]),
                        '_' | '?' => return Some(vec![0x1f]),
                        _ => {}
                    }
                }
            }
        }
        let base: Vec<u8> = match key {
            Key::Char(c) => c.as_bytes().to_vec(),
            Key::Enter => vec![b'\r'],
            Key::Backspace => vec![0x7f],
            Key::Tab => vec![b'\t'],
            Key::Escape => vec![0x1b],
            Key::Up => b"\x1b[A".to_vec(),
            Key::Down => b"\x1b[B".to_vec(),
            Key::Right => b"\x1b[C".to_vec(),
            Key::Left => b"\x1b[D".to_vec(),
            Key::Home => b"\x1b[H".to_vec(),
            Key::End => b"\x1b[F".to_vec(),
            Key::Delete => b"\x1b[3~".to_vec(),
            Key::PageUp => b"\x1b[5~".to_vec(),
            Key::PageDown => b"\x1b[6~".to_vec(),
            Key::Insert => b"\x1b[2~".to_vec(),
            Key::Function(n) => match n {
                1 => b"\x1bOP".to_vec(),
                2 => b"\x1bOQ".to_vec(),
                3 => b"\x1bOR".to_vec(),
                4 => b"\x1bOS".to_vec(),
                5 => b"\x1b[15~".to_vec(),
                6 => b"\x1b[17~".to_vec(),
                7 => b"\x1b[18~".to_vec(),
                8 => b"\x1b[19~".to_vec(),
                9 => b"\x1b[20~".to_vec(),
                10 => b"\x1b[21~".to_vec(),
                11 => b"\x1b[23~".to_vec(),
                _ => b"\x1b[24~".to_vec(),
            },
        };
        if base.is_empty() {
            return None;
        }
        // Alt sends ESC first (the terminals' meta rule).
        if mods.alt {
            let mut with_esc = vec![0x1b];
            with_esc.extend(base);
            return Some(with_esc);
        }
        Some(base)
    }
}

impl shell::Client for Terminal {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
        mkw::arm_mono(engine);
    }

    fn warp(&mut self, loom: &shell::Loom<()>) {
        // The PTY thread: what the program writes becomes a beat
        // on the loom. The events are already in place — the
        // machine set them up at start.
        let feeder = loom.clone();
        std::thread::spawn(move || {
            let Some(events) = crate::EVENTS.get() else { return };
            loop {
                let got = {
                    let guard = match events.lock() {
                        Ok(g) => g,
                        Err(e) => e.into_inner(),
                    };
                    guard.recv()
                };
                match got {
                    Ok(TermEvent::Exit) => {
                        feeder.feed(());
                        return;
                    }
                    // OSC 52: programs put things on the clipboard themselves —
                    // a helper that hands over a command should
                    // not need a mouse for it.
                    Ok(TermEvent::ClipboardStore(_, text)) => {
                        board::write(text);
                        feeder.feed(());
                    }
                    Ok(_) => feeder.feed(()),
                    Err(_) => return,
                }
            }
        });
    }

    fn woven(&mut self, _work: ()) -> Refresh {
        Refresh::Dirty
    }

    fn view(
        &mut self,
        engine: &mut TextEngine,
        _keep: &mut Keep<f32>,
        _atlas: &mut matrixkit_image::ImageAtlas,
        w: u32,
        h: u32,
    ) -> Scene {
        // The window size comes via view() — here the PTY learns
        // its columns and rows (there is no resize hook,
        // and it would be one too many anyway).
        self.machine.measure(w as f32, h as f32);
        let p = self.palette;
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mono = mkw::mono_family();

        // The wall and the head (`fam::Chrome`; headless on the Duo).
        let mut b = shell::SceneBuilder::new();
        b.quad(0.0, 0.0, w as f32, h as f32, p.surface, 0.0, None, None);
        let head = self.chrome.build(engine, &mut b, w, h, p);
        // Rolled up, the window is just its bar.
        if self.chrome.rolled {
            return b.finish();
        }
        let Scene { placed: mut scene, mut roles, mut texts, .. } = b.finish();

        // The GRID. Per row, equally coloured cells are merged into
        // one run: ONE text run instead of one per
        // character (at 80 columns that is the difference between
        // 80 and usually 3 entries).
        let top = head + 6.0;
        let left = MARGIN_X / 2.0;
        self.grid_top = top;
        let term = self.machine.term.lock();
        let content = term.renderable_content();
        let cursor = content.cursor.point;
        let chosen = content.selection;
        let columns = self.machine.mass.columns;
        let lines_total = self.machine.mass.lines;

        // When scrolled back, `display_iter` counts NEGATIVE: the visible
        // rows run from -offset to rows-1-offset. Whoever
        // throws away the negative ones paints a half-empty wall when scrolling —
        // exactly what ails the iced version too (Lager 88). The
        // offset brings them back to 0..rows-1.
        let offset = term.grid().display_offset() as i32;
        let mut grid: Vec<Vec<(char, Option<mk::Rgba>, Option<mk::Rgba>)>> =
            vec![vec![(' ', None, None); columns]; lines_total];
        for indexed in content.display_iter {
            let line = indexed.point.line.0 + offset;
            let column = indexed.point.column.0;
            if line < 0 || line as usize >= lines_total || column >= columns {
                continue;
            }
            if indexed.flags.contains(Flags::HIDDEN) {
                continue;
            }
            let mut fg = ansi_rgba(indexed.fg, p, true);
            let mut bg = ansi_rgba(indexed.bg, p, false);
            if indexed.flags.contains(Flags::INVERSE) {
                let old = fg;
                fg = Some(bg.unwrap_or(p.surface));
                bg = Some(old.unwrap_or(p.on_surface));
            }
            if chosen.is_some_and(|s| s.contains(indexed.point)) {
                fg = Some(p.on_primary_container);
                bg = Some(p.primary_container);
            }
            if indexed.flags.contains(Flags::DIM) {
                if let Some(f) = fg.as_mut() {
                    f.a *= 0.6;
                }
            }
            grid[line as usize][column] = (indexed.c, fg, bg);
        }
        drop(term);

        for (row, line) in grid.iter().enumerate() {
            let y = top + row as f32 * CELL_H;
            let mut col = 0usize;
            while col < columns {
                let (_, fg, bg) = line[col];
                let mut end = col + 1;
                while end < columns && (line[end].1, line[end].2) == (fg, bg) {
                    end += 1;
                }
                let run: String = line[col..end].iter().map(|(z, _, _)| *z).collect();
                let x = left + col as f32 * CELL_B;
                let breite = (end - col) as f32 * CELL_B;
                if let Some(b) = bg {
                    let slot = scene.len();
                    let mut placed = Vec::new();
                    lay_out(
                        &matrixkit_theme::scene::Node::Quad {
                            w: breite,
                            h: CELL_H,
                            color: b,
                            radius: 0.0,
                        },
                        x,
                        y,
                        &mut placed,
                    );
                    scene.extend(placed);
                    roles.push(None);
                    let _ = slot;
                }
                if !run.trim().is_empty() {
                    // The text run hangs on the LAST laid
                    // rectangle: compose() paints in anchor order
                    // (Lager 70).
                    let anchor = scene.len().saturating_sub(1);
                    texts.push(Text {
                        s: run,
                        px: FONT,
                        weight: mk::typo::Weight::Normal,
                        family: mono,
                        color: fg.unwrap_or(p.on_surface),
                        x,
                        y,
                        // The home is somewhat WIDER than the cell.
                        // A glyph does not stay inside its box:
                        // descenders (g, y, p) reach below the
                        // line, and over a long run the difference
                        // between the true advance width and our
                        // CELL_B adds up. Whatever is
                        // painted outside the home is not reset in the
                        // next frame and stays behind
                        // as residue (Lager 89).
                        home: Rect {
                            x: x - 1.0,
                            y: y - 2.0,
                            w: breite + 3.0,
                            h: CELL_H + 5.0,
                        },
                        anchor,
                        clip: None,
                    });
                }
                col = end;
            }
        }

        // The cursor: a bar on the write head's cell. It too
        // counts in the grid, not in the viewport — when scrolling
        // it wanders out of the picture, and then it must not be painted.
        let cursor_row = cursor.line.0 + offset;
        if self.blink && cursor_row >= 0 && (cursor_row as usize) < lines_total {
            let x = left + cursor.column.0 as f32 * CELL_B;
            let y = top + cursor_row as f32 * CELL_H;
            let mut placed = Vec::new();
            lay_out(
                &matrixkit_theme::scene::Node::Quad {
                    w: CELL_B,
                    h: CELL_H,
                    color: mk::Rgba { a: 0.65, ..p.primary },
                    radius: 1.0,
                },
                x,
                y,
                &mut placed,
            );
            scene.extend(placed);
            roles.push(None);
        }
        let _ = clear;

        // The ROOT LAYER lies ABOVE everything (Lager 93, R210 family).
        let mut b = shell::SceneBuilder::new();
        b.scene = Scene { placed: scene, roles, texts, picts: Vec::new(), curves: Vec::new() };
        let about = fam::RootAbout {
            name: "Matrix Terminal",
            version: "1.0",
            description: "Die Kommandozeile in der MatrixKit-Grammatik — \
                alacritty-Maschinenhaus, Matrix-Karosserie.",
        };
        self.root.lay(engine, &mut b, &about, w as f32, h as f32, p, ROOT_BASE);
        b.finish()
    }

    /// The window is MAXIMISED — or free again (Lager 92).
    fn zoomed(&mut self, on: bool) {
        self.chrome.set_zoomed(on);
    }

    fn extent(&mut self) -> Option<(u32, u32)> {
        self.chrome.extent()
    }

    fn key_with(&mut self, key: Key, mods: Mods) -> Refresh {
        self.mods = mods;
        // With the root layer open, the keyboard belongs to the layer —
        // as in iced (the layer catches the focus).
        if self.root.key(&key, mods.shift) {
            return Refresh::Dirty;
        }
        // Copy/paste: Ctrl+Shift+C/V — the custom of
        // terminals (Ctrl+C belongs to the program).
        if mods.ctrl && mods.shift {
            if let Key::Char(c) = &key {
                if c.eq_ignore_ascii_case("c") {
                    let picked = self.machine.term.lock().selection_to_string();
                    if let Some(text) = picked {
                        if !text.is_empty() {
                            board::write(text);
                        }
                    }
                    return Refresh::Calm;
                }
                if c.eq_ignore_ascii_case("v") {
                    if let Some(text) = board::read() {
                        self.paste(&text);
                        return Refresh::Dirty;
                    }
                    return Refresh::Calm;
                }
            }
        }
        // Shift + page keys scroll the HISTORY, not the
        // program — every terminal does it that way.
        if mods.shift {
            let jump = match key {
                Key::PageUp => Some(Scroll::PageUp),
                Key::PageDown => Some(Scroll::PageDown),
                Key::Home => Some(Scroll::Top),
                Key::End => Some(Scroll::Bottom),
                _ => None,
            };
            if let Some(jump) = jump {
                self.machine.term.lock().scroll_display(jump);
                return Refresh::Dirty;
            }
        }
        if let Some(bytes) = Self::bytes_for(&key, mods) {
            // Input jumps to the end of the history (terminal custom).
            self.machine.term.lock().scroll_display(Scroll::Bottom);
            self.machine.inject(bytes);
            return Refresh::Dirty;
        }
        Refresh::Calm
    }

    fn motion(&mut self, x: f64, y: f64) -> Refresh {
        self.hover = self.cell_at(x, y);
        // The bar: magnifier and traffic-light glyphs (`fam::Chrome`).
        self.chrome.motion(x, y)
    }

    fn depart(&mut self) -> Refresh {
        // The pointer has left the window — the bar must
        // withdraw, otherwise the magnifier stays (Lager 91).
        self.hover = None;
        self.chrome.depart()
    }

    fn animating(&self) -> bool {
        self.chrome.animating() || self.root.animating()
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        let zoom = self.chrome.glide(dt);
        let root = self.root.glide(dt);
        if zoom || root {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn strike(&mut self, role: Option<usize>, x: f64, y: f64, source: Source) -> Refresh {
        let origin = source.label();
        // Root layer open: ONLY its roles count — everything else
        // lies under the veil.
        if let Some(deed) = self.root.strike_at(role, ROOT_BASE) {
            return match deed {
                fam::RootDeed::Help => {
                    mkw::help_open("Matrix Terminal");
                    Refresh::Calm
                }
                fam::RootDeed::Calm => Refresh::Calm,
                _ => Refresh::Dirty,
            };
        }
        match self.chrome.strike(role) {
            Some(fam::ChromeDeed::Close) => {
                eprintln!("[terminal] Treffer ({origin}): Ampel × — Matrix Terminal schließt");
                return Refresh::Close;
            }
            Some(fam::ChromeDeed::Title) => {
                self.root.toggle();
                return Refresh::Dirty;
            }
            Some(fam::ChromeDeed::Answer(r)) => return r,
            None => {}
        }
        match role {
            _ => {
                // In the GRID: either the press belongs to the program
                // (htop, tmux — it ordered the mouse), or it
                // starts a selection.
                let Some((line, column, side)) = self.cell_at(x, y) else {
                    return Refresh::Calm;
                };
                self.hover = Some((line, column, side));
                let mode = *self.machine.term.lock().mode();
                if mode.intersects(TermMode::MOUSE_MODE) && !self.mods.shift {
                    self.machine.inject(mouse_report(mode, 0, true, line, column));
                    self.reported_press = true;
                    return Refresh::Calm;
                }
                let point = self.point_of(line, column);
                self.machine.term.lock().selection =
                    Some(Selection::new(SelectionType::Simple, point, side));
                self.selecting = true;
                Refresh::Dirty
            }
        }
    }

    fn travel(&mut self, x: f64, y: f64, _source: Source) -> Refresh {
        let Some((line, column, side)) = self.cell_at(x, y) else {
            return Refresh::Calm;
        };
        self.hover = Some((line, column, side));
        if self.reported_press {
            let mode = *self.machine.term.lock().mode();
            if mode.intersects(TermMode::MOUSE_DRAG | TermMode::MOUSE_MOTION) {
                self.machine.inject(mouse_report(mode, 32, true, line, column));
            }
            return Refresh::Calm;
        }
        if !self.selecting {
            return Refresh::Calm;
        }
        let point = self.point_of(line, column);
        let mut term = self.machine.term.lock();
        if let Some(sel) = term.selection.as_mut() {
            sel.update(point, side);
        }
        drop(term);
        Refresh::Dirty
    }

    fn release(&mut self, _role: Option<usize>, x: f64, y: f64, _source: Source) -> Refresh {
        if self.reported_press {
            self.reported_press = false;
            if let Some((line, column, _)) = self.cell_at(x, y).or(self.hover) {
                let mode = *self.machine.term.lock().mode();
                self.machine.inject(mouse_report(mode, 0, false, line, column));
            }
        }
        self.selecting = false;
        Refresh::Calm
    }

    fn axis(&mut self, x: f64, y: f64, value: f64, _keep: &mut Keep<f32>) -> Refresh {
        if self.root.is_open() {
            return Refresh::Calm;
        }
        // The pointer does not report anew while scrolling — update the cell
        // here, otherwise a mouse program reports the wrong one.
        if let Some(at) = self.cell_at(x, y) {
            self.hover = Some(at);
        }
        // Collect partial lines: trackballs deliver fractions, and
        // a discarded half notch is a dead wheel (R201).
        // The factor 4 is the kit's (cf. help, Lager 41).
        self.scroll_rest += -(value as f32) * 4.0 / CELL_H;
        let whole = self.scroll_rest.trunc() as i32;
        if whole == 0 {
            return Refresh::Calm;
        }
        self.scroll_rest -= whole as f32;
        let mode = *self.machine.term.lock().mode();
        if mode.intersects(TermMode::MOUSE_MODE) && !self.mods.shift {
            // The program ordered the mouse: the wheel is ITS
            // event. Shift keeps our own scrolling reachable.
            if let Some((line, column, _)) = self.hover {
                let button = if whole > 0 { 64 } else { 65 };
                let mut bytes = Vec::new();
                for _ in 0..whole.unsigned_abs() {
                    bytes.extend_from_slice(&mouse_report(mode, button, true, line, column));
                }
                self.machine.inject(bytes);
            }
            return Refresh::Calm;
        }
        if mode.contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL) && !self.mods.shift {
            // Fullscreen without mouse interest (less, vim): the wheel
            // becomes arrow keys — the xterm custom.
            let code: &[u8] = if whole > 0 { b"\x1b[A" } else { b"\x1b[B" };
            let mut bytes = Vec::new();
            for _ in 0..whole.unsigned_abs() {
                bytes.extend_from_slice(code);
            }
            self.machine.inject(bytes);
            return Refresh::Calm;
        }
        self.machine.term.lock().scroll_display(Scroll::Delta(whole));
        Refresh::Dirty
    }

    /// The head strip (logical px): a press there without a button moves
    /// the window — in a foreign desktop by the compositor, on a duo as a
    /// flick to the other panel (R207/R209).
    fn headline(&self) -> f32 {
        self.chrome.headline()
    }

    /// The bar asks for the settings (Duo grammar R210).
    fn root(&mut self) -> Refresh {
        if self.root.open() {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    /// R209: the terminal types — the house holds the on-screen keyboard.
    fn wants_keyboard(&self) -> bool {
        true
    }

    fn beat(&mut self) -> Refresh {
        self.blink = !self.blink;
        let _ = self.root.beat();
        Refresh::Dirty
    }

}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // GPU painting by default (GRABUNG-GPU-MALEREI: own finger probe passed
    // on the duo 01.11.2026 — 0.29/255 mean deviation, CPU 76 -> 10 ticks
    // per 10 s). MATRIX_CPU=1 is the emergency exit.
    if std::env::var_os("MATRIX_CPU").is_none() {
        std::env::set_var("MATRIX_GPU", "1");
    }
    eprintln!(
        "[terminal] Matrix Terminal: erste GROSSE App aufs Fensterhaus, pid={}",
        std::process::id()
    );
    fam::run(
        Terminal::new(),
        shell::Options {
            mount: shell::Mount::Window {
                app_id: String::from(crate::APP_ID),
                title: String::from("Matrix Terminal"),
                width: WIDTH,
                height: HEIGHT,
            },
            argb: false,
            tag: "terminal",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
