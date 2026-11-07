//! Matrix Tastatur im Fensterhaus (round 33, 02.11.2026 — «ohne iced»;
//! first draft Lager 68).
//!
//! The on-screen keyboard is the one app that MAKES input instead of
//! receiving it: the injection stays as it is (`zwp_virtual_keyboard_v1`,
//! injection.rs); the surface moves into the house. Layout and look come
//! from the kit family `fam::keyboard` — the same pill the iced keyboard
//! drew, and the same the login screen draws.
//!
//! What the client must hold differently:
//!  * `KeyboardWish::None` — a keyboard that took the focus would take it
//!    from the field it serves.
//!  * Overlay, not Top (R149c) — the greeter and the lock screen are
//!    fullscreen overlays.
//!  * A key types on TOUCH-DOWN (R59, iced's instant button): `strike()`.
//!  * Finger device (duo): panel-wide on its panel, one window gap from
//!    the edges and the floor, and it RESERVES its height (R209).

use crate::{injection, open_marker, pad_socket, CHANNEL, MANUAL_CLOSE, PAD_COMMANDS};
use matrixkit_scene::scene as fam;
use matrixkit_scene::scene::keyboard::{self as keys, Act};
use matrixkit_shell::{self as shell, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{Keep, Rect};
use matrixkit_widgets as mkw;
use std::time::Instant;

/// The afterglow (R168b): a key glows 150 ms in primary — visible exactly
/// when the finger lifts.
const AFTERGLOW_MS: u128 = 150;

struct Board {
    palette: mk::Palette,
    watcher: mk::PaletteWatcher,
    shift: bool,
    page: usize,
    finger: bool,
    flash: Vec<(String, Instant)>,
    /// Role → (deed, row, column), laid anew with every picture.
    deeds: Vec<(Act, usize, usize)>,
    held: Option<(usize, usize)>,
    /// R94: the controller's focus (row, column) — None until it speaks.
    pad_focus: Option<(usize, usize)>,
}

fn inject(command: injection::Command) {
    if let Some(Some(channel)) = CHANNEL.get() {
        let _ = channel.send(command);
    }
    // Typing keeps the guard awake (the heartbeat).
    mkw::keyboard::send("auf");
}

impl Board {
    fn new() -> Self {
        Self {
            palette: mk::Palette::load().unwrap_or_default(),
            watcher: mk::PaletteWatcher::new(),
            shift: false,
            page: 0,
            finger: mk::device::finger_first(),
            flash: Vec::new(),
            deeds: Vec::new(),
            held: None,
            pad_focus: None,
        }
    }

    fn act(&mut self, act: Act, display: String) -> Refresh {
        match act {
            Act::Text(c) => {
                inject(injection::Command::Char(c));
                mk::feedback::key_press();
                self.flash.push((display, Instant::now()));
                // Shift is one letter on the letter page; on the symbol page
                // «#+=» is a level that stays until switched back.
                if self.page == 0 {
                    self.shift = false;
                }
            }
            Act::Space => {
                inject(injection::Command::Char(' '));
                mk::feedback::key_press();
                self.flash.push((String::new(), Instant::now()));
            }
            Act::Delete => {
                inject(injection::Command::Name("BackSpace"));
                mk::feedback::key_delete();
                self.flash.push((display, Instant::now()));
            }
            Act::Enter => {
                inject(injection::Command::Name("Return"));
                mk::feedback::key_modifier();
                self.flash.push((display, Instant::now()));
            }
            Act::Shift => {
                self.shift = !self.shift;
                mk::feedback::key_modifier();
            }
            Act::Page => {
                self.page = 1 - self.page;
                self.shift = false;
                mk::feedback::key_modifier();
            }
            Act::Close => {
                let _ = std::fs::remove_file(open_marker());
                // The guard reads the exit code: the person closed it — it
                // does not raise it again right away.
                std::process::exit(MANUAL_CLOSE);
            }
        }
        Refresh::Dirty
    }

    fn display_of(&self, row: usize, col: usize) -> String {
        keys::rows(self.page, self.shift, true).get(row).and_then(|r| r.get(col)).map(|k| k.display.clone()).unwrap_or_default()
    }

    /// R94: a controller command on the focus; "tap" types the focused key.
    fn pad_step(&mut self, command: &str) -> Refresh {
        let grid = keys::rows(self.page, self.shift, true);
        if command == "zu" {
            let _ = std::fs::remove_file(open_marker());
            std::process::exit(MANUAL_CLOSE);
        }
        let Some((mut r, mut c)) = self.pad_focus else {
            // First contact: the ring appears without wandering.
            self.pad_focus = Some((0, 0));
            return Refresh::Dirty;
        };
        match command {
            "left" => c = c.saturating_sub(1),
            "right" => c += 1,
            "up" => r = r.saturating_sub(1),
            "down" => r += 1,
            "tap" => {
                let r = r.min(grid.len() - 1);
                if let Some(k) = grid[r].get(c) {
                    let (act, display) = (k.act, k.display.clone());
                    return self.act(act, display);
                }
            }
            _ => {}
        }
        r = r.min(grid.len() - 1);
        c = c.min(grid[r].len() - 1);
        self.pad_focus = Some((r, c));
        Refresh::Dirty
    }
}

impl shell::Client for Board {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, _atlas: &mut matrixkit_image::ImageAtlas, w: u32, h: u32) -> Scene {
        let (wf, hf) = (w as f32, h as f32);
        let mut b = SceneBuilder::new();
        let glowing: Vec<String> = self.flash.iter().filter(|(_, t)| t.elapsed().as_millis() < AFTERGLOW_MS).map(|(d, _)| d.clone()).collect();
        let look = keys::Look { page: self.page, shift: self.shift, closable: true, glowing: &glowing, held: self.held, focus: self.pad_focus, enter: "" };
        // Finger device: the pill IS the surface. Desktop: the pill floats
        // in the middle, the surface around it carries the shadow.
        let rect = if self.finger {
            Rect { x: 0.0, y: 0.0, w: wf, h: hf }
        } else {
            Rect { x: (wf - keys::PILL_W) / 2.0, y: (hf - keys::PILL_H) / 2.0, w: keys::PILL_W, h: keys::PILL_H }
        };
        let first = b.len();
        self.deeds = keys::lay(&mut b, engine, rect, &look, 0, self.palette);
        if !self.finger {
            let mut scene = b.finish();
            fam::lift(&mut scene.placed, first, fam::floating_shadow());
            return scene;
        }
        b.finish()
    }

    /// The press IS the stroke — no waiting for the release (R59).
    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        let Some(&(act, row, col)) = role.and_then(|i| self.deeds.get(i)) else { return Refresh::Calm };
        self.held = Some((row, col));
        // ✕ only on RELEASE over ✕ (03.11.2026): it sits beside ⏎, and a
        // finger grazing it on touch-down closed the keyboard (exit 7).
        if act == Act::Close {
            return Refresh::Dirty;
        }
        let display = self.display_of(row, col);
        self.act(act, display)
    }

    fn release(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        let held = self.held.take();
        let now = role.and_then(|i| self.deeds.get(i)).copied();
        if let (Some(h), Some((Act::Close, r, c))) = (held, now) {
            if h == (r, c) {
                return self.act(Act::Close, String::new());
            }
        }
        if held.is_some() {
            return Refresh::Dirty;
        }
        Refresh::Calm
    }

    /// While something glows the frame clock runs — then it sleeps again.
    fn animating(&self) -> bool {
        !self.flash.is_empty()
    }

    fn glide(&mut self, _dt: f32) -> Refresh {
        let before = self.flash.len();
        self.flash.retain(|(_, t)| t.elapsed().as_millis() < AFTERGLOW_MS);
        if self.flash.len() == before {
            Refresh::Calm
        } else {
            Refresh::Dirty
        }
    }

    fn beat(&mut self) -> Refresh {
        // The afterglow ends here too: a layer surface's frame clock may
        // not run, and ⌫ stayed lit (03.11.2026).
        let before = self.flash.len();
        self.flash.retain(|(_, t)| t.elapsed().as_millis() < AFTERGLOW_MS);
        let glow_ended = self.flash.len() != before;
        // R94: the controller queue (filled by the pad thread).
        let commands: Vec<String> = PAD_COMMANDS.get().map(|q| q.lock().unwrap_or_else(|e| e.into_inner()).drain(..).collect()).unwrap_or_default();
        let mut r = Refresh::Calm;
        for c in commands {
            r = self.pad_step(&c);
        }
        if self.watcher.changed() {
            self.palette = mk::Palette::load().unwrap_or_default();
            return Refresh::Dirty;
        }
        if glow_ended {
            return Refresh::Dirty;
        }
        r
    }

    fn pulse(&mut self) -> Refresh {
        // The guard's «is it still open?» (the iced keyboard wrote it every 5 s).
        let _ = std::fs::write(open_marker(), b"1");
        Refresh::Calm
    }

    fn output(&self) -> Option<String> {
        // Round 83: folded to one panel, the keyboard comes to the front one.
        mk::device::is_duo().then(|| mk::device::duo_panel("keyboard-output", mk::device::DUO_KEYBOARD_PANEL))
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // The surface lives only as long as its guard (01.11.2026: an orphan
    // pill kept running the old binary) — the kernel ends us with it.
    mkw::process::die_with_parent();
    let _ = CHANNEL.set(injection::start(keys::all_chars()));
    let _ = std::fs::write(open_marker(), b"1");
    let _ = PAD_COMMANDS.get_or_init(Default::default);
    // R94: the pad bridge — a datagram socket for controller commands.
    std::thread::spawn(|| {
        use std::os::unix::net::UnixDatagram;
        let path = pad_socket();
        let _ = std::fs::remove_file(&path);
        let Ok(sock) = UnixDatagram::bind(&path) else { return };
        let mut buf = [0u8; 32];
        while let Ok(n) = sock.recv(&mut buf) {
            let command = String::from_utf8_lossy(&buf[..n]).trim().to_string();
            if let Some(q) = PAD_COMMANDS.get() {
                q.lock().unwrap_or_else(|e| e.into_inner()).push_back(command);
            }
        }
    });
    let finger = mk::device::finger_first();
    let gap = mk::device::WINDOW_GAP as i32;
    let edge = mkw::panel::SHADOW_MARGIN as u32;
    eprintln!("[tastatur] Matrix Tastatur im Haus (round 33), Finger: {finger}, pid={}", std::process::id());
    shell::run(
        Board::new(),
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-keyboard"),
                layer: shell::Layer::Overlay,
                keyboard: shell::KeyboardWish::None,
                anchor: shell::Anchors { top: false, right: finger, bottom: true, left: finger },
                size: if finger { (0, keys::PILL_H as u32) } else { (keys::PILL_W as u32 + 2 * edge, keys::PILL_H as u32 + 2 * edge) },
                margin: if finger { (0, gap, gap, gap) } else { (0, 0, 10 - mkw::panel::SHADOW_MARGIN as i32, 0) },
                // The compositor reserves the pill plus one gap below; niri's
                // own gap keeps the window one gap above it.
                exclusive: if finger { keys::PILL_H as i32 } else { 0 },
            },
            argb: true,
            tag: "tastatur",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
