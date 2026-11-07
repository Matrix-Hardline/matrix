//! R216 — the wallpaper in the HOUSE (matrixkit-shell, no iced). One
//! painter per panel (`--output NAME`, the house now targets outputs),
//! a Background layer that fills its screen: the picture, decoded at
//! screen size on a thread and laid as a put picture; the cinema veil
//! above it. The 2-s pulse follows the wallpaper contract
//! (mk::wallpaper::current — the house Settings write it), cinema mode
//! and the sun. No exec restart on a change: the house draws what it is
//! given (the iced painter's lower panel stayed black, 02.11.2026).

use matrixkit_image::{Fit, ImageAtlas};
use matrixkit_shell::{self as shell, Loom, Pict, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{Keep, Rect};
use std::path::PathBuf;

/// How dark the background gets in cinema mode (as the iced painter).
const CINEMA_DARK: f32 = 0.78;

pub enum Work {
    /// (path, picture w, h, RGBA) decoded at screen size.
    Picture(String, u32, u32, Vec<u8>),
}

struct Wall {
    loom: Option<Loom<Work>>,
    /// The picture asked for and the one shown (key in the atlas).
    wanted: Option<String>,
    shown: Option<(String, u32, u32, u64)>,
    /// The picture before the last change — drawn UNDER the new one for a
    /// few frames, so the swap never shows black while the new texture
    /// uploads (measured: one black frame on the lower panel).
    previous: Option<((String, u32, u32, u64), u8)>,
    fresh: Option<(String, u32, u32, Vec<u8>)>,
    serial: u64,
    cinema: bool,
    dim: f32,
}

impl Wall {
    fn new() -> Wall {
        let cinema = mk::cinema::active();
        Wall { loom: None, wanted: None, shown: None, previous: None, fresh: None, serial: 0, cinema, dim: if cinema { 1.0 } else { 0.0 } }
    }

    /// Look at the contract; a new picture is decoded off the surface.
    fn follow(&mut self) -> bool {
        let now = mk::wallpaper::current();
        if now == self.wanted {
            return false;
        }
        self.wanted = now.clone();
        let Some(path) = now else { return true };
        eprintln!("[wand] {} neues Bild: {path}", shell::output_wish().unwrap_or_default());
        if crate::palette_claim(&path) {
            crate::palette_generate(&path);
        }
        if let Some(loom) = &self.loom {
            loom.spin(move || {
                let long = mk::device::image_long_side();
                let t0 = std::time::Instant::now();
                let got = matrixkit_image::fitted_rgba(std::path::Path::new(&path), long);
                eprintln!("[wand] dekodiert in {} ms", t0.elapsed().as_millis());
                match got {
                    Some((w, h, rgba)) => Work::Picture(path, w, h, rgba),
                    None => Work::Picture(path, 0, 0, Vec::new()),
                }
            });
        }
        true
    }
}

impl shell::Client for Wall {
    type Card = ();
    type Work = Work;

    fn warp(&mut self, loom: &Loom<Work>) {
        self.loom = Some(loom.clone());
        self.follow();
    }

    fn woven(&mut self, work: Work) -> Refresh {
        let Work::Picture(path, w, h, rgba) = work;
        if w == 0 || self.wanted.as_deref() != Some(path.as_str()) {
            return Refresh::Calm;
        }
        self.fresh = Some((path, w, h, rgba));
        Refresh::Dirty
    }

    fn view(&mut self, _engine: &mut TextEngine, _keep: &mut Keep<f32>, atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let (wf, hf) = (w as f32, h as f32);
        let mut b = SceneBuilder::new();
        let ground = b.quad(0.0, 0.0, wf, hf, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, 0.0, None, None);
        if let Some((path, pw, ph, rgba)) = self.fresh.take() {
            self.serial += 1;
            atlas.put(&PathBuf::from(format!("\u{0}wand:{}", self.serial)), pw, ph, rgba);
            if let Some(old) = self.shown.take() {
                self.previous = Some((old, 6));
            }
            self.shown = Some((path, pw, ph, self.serial));
        }
        let home = Rect { x: 0.0, y: 0.0, w: wf, h: hf };
        let lay = |b: &mut SceneBuilder, pw: u32, ph: u32, serial: u64| {
            // Cover: the picture's box keeps its aspect and overfills the
            // screen; the home (the screen) clips the overhang.
            let s = (wf / pw as f32).max(hf / ph as f32);
            let (bw, bh) = (pw as f32 * s, ph as f32 * s);
            b.pict(Pict {
                path: PathBuf::from(format!("\u{0}wand:{serial}")),
                box_w: bw as u32,
                box_h: bh as u32,
                fit: Fit::Stretch,
                x: (wf - bw) / 2.0,
                y: (hf - bh) / 2.0,
                radius: 0.0,
                home,
                anchor: ground,
                clip: Some(home),
            });
        };
        if let Some(((_, pw, ph, serial), frames)) = &mut self.previous {
            lay(&mut b, *pw, *ph, *serial);
            *frames = frames.saturating_sub(1);
        }
        if self.previous.as_ref().is_some_and(|(_, f)| *f == 0) {
            self.previous = None;
        }
        if let Some((_, pw, ph, serial)) = &self.shown {
            lay(&mut b, *pw, *ph, *serial);
        }
        if self.dim > 0.0 {
            b.quad(0.0, 0.0, wf, hf, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: self.dim * CINEMA_DARK }, 0.0, None, None);
        }
        b.finish()
    }

    fn animating(&self) -> bool {
        self.previous.is_some() || (self.dim - if self.cinema { 1.0 } else { 0.0 }).abs() > 0.001
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        let target = if self.cinema { 1.0 } else { 0.0 };
        if (self.dim - target).abs() <= 0.001 {
            return Refresh::Calm;
        }
        let step = if mk::motion_reduced() { 1.0 } else { dt / 0.6 };
        self.dim = if self.dim < target { (self.dim + step).min(target) } else { (self.dim - step).max(target) };
        Refresh::Dirty
    }

    fn strike(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        Refresh::Calm
    }

    fn pulse(&mut self) -> Refresh {
        // The sun (automatic appearance) as the iced painter did.
        if crate::wallpaper_mode_auto() {
            if let Some(goal_light) = crate::sun_top() {
                if goal_light != crate::is_light() {
                    mk::light_toggle();
                }
            }
        }
        let cinema = mk::cinema::active();
        let moved = cinema != self.cinema;
        self.cinema = cinema;
        if self.follow() || moved {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    shell::run(
        Wall::new(),
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-hintergrund"),
                layer: shell::Layer::Background,
                keyboard: shell::KeyboardWish::None,
                anchor: shell::Anchors { top: true, right: true, bottom: true, left: true },
                size: (0, 0),
                margin: (0, 0, 0, 0),
                exclusive: -1,
            },
            argb: false,
            tag: "wand",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
