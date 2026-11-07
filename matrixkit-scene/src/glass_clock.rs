//! GLASS CLOCK FAMILY (round 123): the lock screen's large clock as drawn
//! vector digits — rounded, condensed strokes that keep their round tops
//! when the clock is stretched taller (the lock's height handle), laid as
//! glass: a soft shadow, a body the wallpaper shows through, a light rim
//! on the upper edge. `glass` runs from 0 (solid, opaque ink) to 1 (clear
//! glass); `stretch` is the digit height over its width.
//!
//! ```ignore
//! let look = glass_clock::Look { digit_w: 60.0, stretch: 1.9, glass: 0.8, weight: 0.5, body: Look::BODY, ink: white };
//! let (w, h) = glass_clock::measure("8:45", &look);
//! glass_clock::lay(&mut b, "8:45", x, y, &look, home, anchor);
//! ```

use matrixkit_shell::shapes::{Cmd, Shape, Style};
use matrixkit_shell::SceneBuilder;
use matrixkit_theme::scene::Rect;
use matrixkit_theme::Rgba;
use std::f32::consts::PI;

/// How the clock looks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// The width of one digit (logical px).
    pub digit_w: f32,
    /// Digit height over width (1.5 … 3.2).
    pub stretch: f32,
    /// 0 = solid ink, 1 = clear glass.
    pub glass: f32,
    /// Round 141: the stroke's weight, 0 (hairline) … 1 (bold); `WEIGHT`
    /// is the clock as it always was.
    pub weight: f32,
    /// Round 147: how much body the glass has — 0 clear, 1 opaque white
    /// (the lens tint is body²). `BODY` = the notification cards' glass.
    pub body: f32,
    pub ink: Rgba,
}

impl Look {
    /// Round 144: wide enough that width and height stay apart — a clock
    /// pulled tall and then narrow keeps its height (3.2 cut it at the
    /// narrowest width, 05.11.2026); 6.0 ≥ 3.2 / clock_edit::SIZE_MIN.
    pub const STRETCH_MIN: f32 = 0.9;
    pub const STRETCH_MAX: f32 = 6.0;
    /// The weight the clock had before it could be chosen.
    pub const WEIGHT: f32 = 0.5;
    /// As legible as the cards' glass: tint 0.06 (= 0.245²).
    pub const BODY: f32 = 0.245;

    /// The lens tint for `body` (squared: the slider's first part is the
    /// subtle one, where glass still bends).
    pub fn tint(&self) -> f32 {
        let b = self.body.clamp(0.0, 1.0);
        b * b
    }

    pub fn stroke(&self) -> f32 {
        // Thicker as glass (the rim needs room), a little lighter when solid;
        // the weight runs 0.06 … 0.28 of a digit (0.5 = the old 0.17).
        self.digit_w * (0.06 + 0.22 * self.weight.clamp(0.0, 1.0) + 0.05 * self.glass.clamp(0.0, 1.0))
    }

    fn height(&self) -> f32 {
        self.digit_w * self.stretch.clamp(Self::STRETCH_MIN, Self::STRETCH_MAX)
    }

    fn gap(&self) -> f32 {
        self.digit_w * 0.24
    }

    fn colon_w(&self) -> f32 {
        self.stroke() * 2.2
    }
}

/// The size of `text` (digits and ':') in this look.
pub fn measure(text: &str, look: &Look) -> (f32, f32) {
    let mut w = 0.0;
    let mut n = 0;
    for c in text.chars() {
        if n > 0 {
            w += look.gap();
        }
        w += if c == ':' { look.colon_w() } else { look.digit_w };
        n += 1;
    }
    (w, look.height())
}

/// Lay `text` at (x, y) — the glyphs' top-left, the strokes' outer edge
/// included. `anchor` is a Placed of the scene the pictures hang on.
pub fn lay(b: &mut SceneBuilder, text: &str, x: f32, y: f32, look: &Look, home: Rect, anchor: usize) {
    let pad = look.stroke() * 0.9;
    let mut cx = x;
    let mut first = true;
    for c in text.chars() {
        if !first {
            cx += look.gap();
        }
        first = false;
        let cw = if c == ':' { look.colon_w() } else { look.digit_w };
        if let Some(shape) = glyph_shape(c, cw, look, pad) {
            b.pict(crate::scene::pict_of(&shape, cx - pad, y - pad, home, anchor));
        }
        cx += cw;
    }
}

/// The desktop's glass (round 124) as a picture: niri's liquid-glass
/// recipe (shaders/glass.frag, the card rule of glass.kdl) applied to ANY
/// shape — the clock's digits, the keypad's circles. The backdrop is
/// frosted like one Kawase pass, bent along the shape's distance field in
/// a rim band with the chromatic fringe (1.15 / 1.0 / 0.85), dimmed by the
/// legibility scrim on bright ground, lit along the vertical axis (top
/// stronger, bottom softer) with shaded flanks.
pub mod lens {
    use matrixkit_shell::shapes::Shape;

    /// The knobs, as in a glass.kdl layer rule (logical px).
    #[derive(Debug, Clone, Copy)]
    pub struct Glass {
        pub band: f32,
        pub bend: f32,
        pub scrim: f32,
        /// A light body (0 = clear like the cards; a key wants ~0.14 so
        /// it shows on smooth ground, where pure glass only bends).
        pub tint: f32,
        /// Extra rim light on top of the desktop's 0.22.
        pub rim: f32,
    }

    impl Glass {
        /// The cards' glass (`matrix-card`: band 14, bend 16, scrim 0.35).
        pub const CARD: Glass = Glass { band: 14.0, bend: 16.0, scrim: 0.35, tint: 0.0, rim: 0.0 };
        /// The light body and rim light of a notification card's glass —
        /// the legibility a clock's `Look::BODY` matches (round 147).
        pub const CARD_TINT: f32 = 0.06;
        pub const CARD_RIM: f32 = 0.06;

        /// The card's glass fitted to a body only `half` thick (a stroke):
        /// the rim band keeps its share of the body.
        pub fn fitted(half: f32) -> Glass {
            let band = Self::CARD.band.min(half * 0.9);
            Glass { band, bend: Self::CARD.bend * band / Self::CARD.band, ..Self::CARD }
        }

        /// A key: the card's glass (it bends what lies behind) with a
        /// little more rim light — a light body hid the bending (05.11.).
        pub fn key(half: f32) -> Glass {
            Glass { rim: 0.10, ..Self::fitted(half) }
        }
    }

    /// `shape` (logical, its opaque parts are the body) as glass placed
    /// with its box at (x, y) on the screen. Give the shape whole logical
    /// pixels: a picture is found again only at exactly box × scale. `backdrop(x, y)` = the
    /// screen's colour at a logical point. Straight RGBA, `scale`× size.
    pub fn picture(shape: &Shape, x: f32, y: f32, scale: f32, glass: Glass, backdrop: &dyn Fn(f32, f32) -> [f32; 3]) -> Option<(u32, u32, Vec<u8>)> {
        let (pw, ph) = ((shape.w * scale).round().max(1.0) as u32, (shape.h * scale).round().max(1.0) as u32);
        let (pw, ph, mask) = matrixkit_shell::shapes::raster(shape, pw, ph)?;
        let (w, h) = (pw as usize, ph as usize);
        let cov: Vec<f32> = mask.chunks_exact(4).map(|p| p[3] as f32 / 255.0).collect();
        // The signed distance (logical px, negative inside) — chamfer both ways.
        let inside: Vec<bool> = cov.iter().map(|c| *c >= 0.5).collect();
        let din = chamfer(&inside, w, h, true);
        let dout = chamfer(&inside, w, h, false);
        let d: Vec<f32> = (0..w * h).map(|k| if inside[k] { -(din[k] - 0.5) } else { dout[k] - 0.5 } / scale).collect();
        let r = (scale * 1.5).round().max(1.0) as usize;
        let ds = super::box_blur(&super::box_blur(&d, w, h, r), w, h, r);
        // One Kawase pass at offset 2 (physical): four diagonal taps.
        let o = 2.0 / scale;
        let frost = |x: f32, y: f32| {
            let mut c = [0.0f32; 3];
            for (dx, dy) in [(-o, -o), (o, -o), (-o, o), (o, o)] {
                let s = backdrop(x + dx, y + dy);
                for n in 0..3 {
                    c[n] += s[n] * 0.25;
                }
            }
            c
        };
        let mut out = vec![0u8; w * h * 4];
        for j in 0..h {
            for i in 0..w {
                let k = j * w + i;
                if cov[k] <= 0.0 {
                    continue;
                }
                let at = |ii: isize, jj: isize| ds[(jj.clamp(0, h as isize - 1) as usize) * w + ii.clamp(0, w as isize - 1) as usize];
                let (ii, jj) = (i as isize, j as isize);
                let mut gx = at(ii + 1, jj) - at(ii - 1, jj);
                let mut gy = at(ii, jj + 1) - at(ii, jj - 1);
                let len = (gx * gx + gy * gy).sqrt() + 1e-5;
                gx /= len;
                gy /= len;
                let rim = (1.0 + d[k] / glass.band).clamp(0.0, 1.0);
                let rim = rim * rim;
                let bend = glass.bend * rim;
                let (lx, ly) = (x + i as f32 / scale, y + j as f32 / scale);
                let mut c = [
                    frost(lx - gx * bend * 1.15, ly - gy * bend * 1.15)[0],
                    frost(lx - gx * bend, ly - gy * bend)[1],
                    frost(lx - gx * bend * 0.85, ly - gy * bend * 0.85)[2],
                ];
                if glass.scrim > 0.0 {
                    let lum = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
                    let t = ((lum - 0.55) / 0.40).clamp(0.0, 1.0);
                    let over = t * t * (3.0 - 2.0 * t);
                    for v in &mut c {
                        *v *= 1.0 - over * glass.scrim;
                    }
                }
                let up = -gy;
                let rim_edge = rim * rim * rim;
                let lit = up.abs() * if up >= 0.0 { 1.0 } else { 0.72 };
                // The whole rim glints a little (keys), the body lightens.
                let ring = rim_edge * glass.rim;
                for v in &mut c {
                    *v += (1.0 - *v) * glass.tint;
                    *v = (*v + rim_edge * lit * 0.22 + ring - rim_edge * gx.abs() * 0.08).clamp(0.0, 1.0);
                }
                for n in 0..3 {
                    out[k * 4 + n] = (c[n] * 255.0) as u8;
                }
                out[k * 4 + 3] = (cov[k] * 255.0) as u8;
            }
        }
        Some((pw, ph, out))
    }

    /// A rounded card of live glass (round 129): its lens map (made once
    /// per size and kept while drawn) registered over `backdrop` with this
    /// frame's veil and fade. Returns the map's path for a Stretch pict.
    #[allow(clippy::too_many_arguments)]
    pub fn card(atlas: &mut matrixkit_image::ImageAtlas, w: f32, h: f32, radius: f32, scale: f32, backdrop: &std::path::Path, veil: f32, fade: f32) -> std::path::PathBuf {
        let (w, h) = (w.floor(), h.floor());
        let path = std::path::PathBuf::from(format!("\u{0}glas-karte-{}x{}-{}-{}", w as u32, h as u32, radius as u32, scale as u32));
        let glass = Glass::fitted(h / 2.0);
        if atlas.stretched_ready(&path).is_none() {
            let shape = Shape::new(w, h).op(super::rounded(0.5, 0.5, w - 0.5, h - 0.5, radius), matrixkit_shell::shapes::Style::Fill(matrixkit_theme::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }));
            let t0 = std::time::Instant::now();
            if let Some((pw, ph, px)) = map(&shape, scale, glass.band) {
                atlas.put(&path, pw, ph, px);
            }
            eprintln!("[glas] Karte {w}×{h} gerechnet in {} ms", t0.elapsed().as_millis());
        }
        // Round 140: a card outlives frames of the other panel.
        atlas.pin(&path);
        matrixkit_shell::lens::set(&path, matrixkit_shell::lens::Lens {
            backdrop: backdrop.to_path_buf(),
            bend: glass.bend,
            scrim: glass.scrim,
            rim: Glass::CARD_RIM,
            veil,
            tint: Glass::CARD_TINT,
            opacity: fade,
        });
        path
    }

    /// LENS MAP (round 125) of `shape` for live glass on the GPU
    /// (`matrixkit_shell::lens`): rg = outward normal (0.5 + 0.5 n), b =
    /// the rim band as in niri (clamp(1 + d / band)², 0 deep inside), a =
    /// coverage. It depends on the shape only — the backdrop is bent live.
    /// Give the shape whole logical pixels.
    pub fn map(shape: &Shape, scale: f32, band: f32) -> Option<(u32, u32, Vec<u8>)> {
        let (pw, ph) = ((shape.w * scale).round().max(1.0) as u32, (shape.h * scale).round().max(1.0) as u32);
        let (pw, ph, mask) = matrixkit_shell::shapes::raster(shape, pw, ph)?;
        let (w, h) = (pw as usize, ph as usize);
        let cov: Vec<f32> = mask.chunks_exact(4).map(|p| p[3] as f32 / 255.0).collect();
        let inside: Vec<bool> = cov.iter().map(|c| *c >= 0.5).collect();
        let din = chamfer(&inside, w, h, true);
        let dout = chamfer(&inside, w, h, false);
        let d: Vec<f32> = (0..w * h).map(|k| if inside[k] { -(din[k] - 0.5) } else { dout[k] - 0.5 } / scale).collect();
        let r = (scale * 1.5).round().max(1.0) as usize;
        let ds = super::box_blur(&super::box_blur(&d, w, h, r), w, h, r);
        let mut out = vec![0u8; w * h * 4];
        for j in 0..h {
            for i in 0..w {
                let k = j * w + i;
                if cov[k] <= 0.0 {
                    continue;
                }
                let at = |ii: isize, jj: isize| ds[(jj.clamp(0, h as isize - 1) as usize) * w + ii.clamp(0, w as isize - 1) as usize];
                let (ii, jj) = (i as isize, j as isize);
                let mut gx = at(ii + 1, jj) - at(ii - 1, jj);
                let mut gy = at(ii, jj + 1) - at(ii, jj - 1);
                let len = (gx * gx + gy * gy).sqrt() + 1e-5;
                gx /= len;
                gy /= len;
                let rim = (1.0 + d[k] / band).clamp(0.0, 1.0);
                out[k * 4] = ((0.5 + 0.5 * gx) * 255.0).round() as u8;
                out[k * 4 + 1] = ((0.5 + 0.5 * gy) * 255.0).round() as u8;
                out[k * 4 + 2] = (rim * rim * 255.0).round() as u8;
                out[k * 4 + 3] = (cov[k] * 255.0).round() as u8;
            }
        }
        Some((pw, ph, out))
    }

    /// Distance (px) of every pixel to the nearest pixel of the other
    /// kind (`of_inside`: for inside pixels; 0 for the rest). 3-4 chamfer.
    fn chamfer(inside: &[bool], w: usize, h: usize, of_inside: bool) -> Vec<f32> {
        const FAR: f32 = 1.0e6;
        let mut d: Vec<f32> = inside.iter().map(|&v| if v == of_inside { FAR } else { 0.0 }).collect();
        let (a, b) = (1.0f32, 1.4142f32);
        for y in 0..h {
            for x in 0..w {
                let k = y * w + x;
                let mut v = d[k];
                if x > 0 { v = v.min(d[k - 1] + a); }
                if y > 0 {
                    v = v.min(d[k - w] + a);
                    if x > 0 { v = v.min(d[k - w - 1] + b); }
                    if x + 1 < w { v = v.min(d[k - w + 1] + b); }
                }
                d[k] = v;
            }
        }
        for y in (0..h).rev() {
            for x in (0..w).rev() {
                let k = y * w + x;
                let mut v = d[k];
                if x + 1 < w { v = v.min(d[k + 1] + a); }
                if y + 1 < h {
                    v = v.min(d[k + w] + a);
                    if x + 1 < w { v = v.min(d[k + w + 1] + b); }
                    if x > 0 { v = v.min(d[k + w - 1] + b); }
                }
                d[k] = v;
            }
        }
        d
    }
}

/// The clock as ONE glass body: its shape (whole logical pixels), the
/// offset of the glyphs' top-left inside it, and the stroke width.
pub fn glass_shape(text: &str, look: &Look) -> (Shape, f32, f32) {
    let sw = look.stroke() * 1.35;
    let pad = 2.0;
    let (tw, th) = measure(text, look);
    let (lw, lh) = ((tw + 2.0 * pad + sw * 0.4).ceil(), (th + 2.0 * pad + sw * 0.35).ceil());
    let mut shape = Shape::new(lw, lh);
    let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    let grow = sw * 0.175;
    let mut cx = pad + grow;
    for (k, c) in text.chars().enumerate() {
        if k > 0 {
            cx += look.gap();
        }
        let cw = if c == ':' { look.colon_w() } else { look.digit_w };
        if c == ':' {
            let r = sw * 0.5;
            shape = shape.op(vec![Cmd::Circle(cx + cw / 2.0, pad + grow + th * 0.34, r), Cmd::Circle(cx + cw / 2.0, pad + grow + th * 0.68, r)], Style::Fill(white));
        } else if let Some(d) = c.to_digit(10) {
            let base = look.stroke();
            for p in digit(d, cw - base, th - base) {
                shape = shape.op(shift(&p, cx + base / 2.0, pad + grow + base / 2.0), Style::Stroke { tint: white, width: sw });
            }
        }
        cx += cw;
    }
    (shape, pad + grow, sw)
}

/// The clock as a LENS MAP for live glass (round 125): the map, its
/// logical top-left for glyphs at (x, y), its logical size, the band.
#[allow(clippy::type_complexity)]
pub fn lens_map(text: &str, look: &Look, x: f32, y: f32, scale: f32) -> Option<(u32, u32, Vec<u8>, f32, f32, f32, f32, f32)> {
    let (shape, inset, sw) = glass_shape(text, look);
    let band = lens::Glass::fitted(sw / 2.0).band;
    let (pw, ph, map) = lens::map(&shape, scale, band)?;
    Some((pw, ph, map, x - inset, y - inset, shape.w, shape.h, band))
}

/// How a LIVE clock picture keeps up with a changing look (round 146).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pace {
    /// Sharp: made at the screen's scale whenever the look changes.
    Still,
    /// A finger resizes the clock: the last picture is STRETCHED to the new
    /// size (the GPU, every frame) and made anew at most every 300 ms.
    Stretch,
    /// A finger changes the strokes (the weight): a DRAFT at one pixel per
    /// logical pixel follows every frame. Measured on the Duo (05.11.2026,
    /// ~445×350 clock): draft 14.5 ms, sharp at 3× 109 ms.
    Draft,
}

/// The clock as LIVE GLASS (round 146, from the lock): its lens map made
/// once per look and kept in the atlas (pinned), registered as a lens over
/// `backdrop` every frame. One per surface stem.
///
/// ```ignore
/// let pic = self.live.picture(atlas, &time, &look, x, y, scale, Pace::Still, &wall, fade);
/// match pic { Some(p) => b.pict(p.pict(home, anchor)), None => glass_clock::lay(…) }
/// ```
#[derive(Debug, Default)]
pub struct Live {
    stem: String,
    seq: u64,
    /// (look key, path, offset of the picture from the glyphs, logical size).
    pic: Option<(String, std::path::PathBuf, f32, f32, f32, f32)>,
    /// The glyph size the picture was made for, and when.
    built: Option<(f32, f32, std::time::Instant)>,
}

/// A live clock's picture this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct LivePicture {
    pub path: std::path::PathBuf,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl LivePicture {
    pub fn pict(&self, home: Rect, anchor: usize) -> matrixkit_shell::Pict {
        matrixkit_shell::Pict { path: self.path.clone(), box_w: self.w as u32, box_h: self.h as u32, fit: matrixkit_image::Fit::Stretch, x: self.x, y: self.y, radius: 0.0, home, anchor, clip: None }
    }
}

impl Live {
    /// `stem`: the pictures' name stem (no ':' — stretched pictures share a
    /// texture per stem).
    pub fn new(stem: &str) -> Live {
        Live { stem: stem.replace(':', "-"), ..Live::default() }
    }

    /// The picture of `text` with its glyphs' top-left at (x, y). None: no
    /// map could be made (lay the drawn clock instead).
    #[allow(clippy::too_many_arguments)]
    pub fn picture(&mut self, atlas: &mut matrixkit_image::ImageAtlas, text: &str, look: &Look, x: f32, y: f32, scale: f32, pace: Pace, backdrop: &std::path::Path, fade: f32) -> Option<LivePicture> {
        let draft = pace == Pace::Draft;
        // The map depends on the shape only — never on where it lies.
        let key = format!("{text}|{}|{}|{}|{}|{draft}", look.digit_w, look.stretch, look.glass, look.weight);
        let ready = matches!(&self.pic, Some((_, path, ..)) if atlas.stretched_ready(path).is_some());
        let fresh = !(ready && matches!(&self.pic, Some((k, ..)) if *k == key));
        let (tw, th) = measure(text, look);
        let mut sx = 1.0;
        let mut sy = 1.0;
        let stretch_old = fresh && ready && pace == Pace::Stretch && self.built.is_some_and(|(_, _, at)| at.elapsed().as_secs_f32() < 0.3);
        if stretch_old {
            let (bw, bh, _) = self.built?;
            sx = tw / bw.max(1.0);
            sy = th / bh.max(1.0);
        } else if fresh {
            let (pw, ph, map, ox, oy, lw, lh, _band) = lens_map(text, look, x, y, if draft { 1.0 } else { scale })?;
            if let Some((_, old, ..)) = &self.pic {
                matrixkit_shell::lens::remove(old);
                atlas.unpin(old);
            }
            self.seq += 1;
            let path = std::path::PathBuf::from(format!("\u{0}{}-{}", self.stem, self.seq));
            atlas.put(&path, pw, ph, map);
            self.pic = Some((key, path, ox - x, oy - y, lw, lh));
            self.built = Some((tw, th, std::time::Instant::now()));
        }
        let (_, path, ix, iy, lw, lh) = self.pic.clone()?;
        atlas.pin(&path);
        let glass = lens::Glass::fitted(look.stroke() * 1.35 / 2.0);
        // Round 147: the body is a lens value only — no new map, every frame.
        // Its rim light as the cards' (0.06).
        matrixkit_shell::lens::set(&path, matrixkit_shell::lens::Lens { backdrop: backdrop.to_path_buf(), bend: glass.bend, scrim: glass.scrim, rim: lens::Glass::CARD_RIM, veil: 0.0, tint: look.tint(), opacity: fade });
        Some(LivePicture { path, x: x + ix * sx, y: y + iy * sy, w: (lw * sx).round(), h: (lh * sy).round() })
    }

    /// The last picture (for a frame that does not change the look).
    pub fn forget(&mut self, atlas: &mut matrixkit_image::ImageAtlas) {
        if let Some((_, old, ..)) = self.pic.take() {
            matrixkit_shell::lens::remove(&old);
            atlas.unpin(&old);
        }
        self.built = None;
    }
}

/// The clock as the desktop's glass (round 124): `text` with its glyphs'
/// top-left at (x, y) on the screen. Returns the picture and its logical
/// place: (pixels w, h, rgba, left, top, logical w, logical h).
#[allow(clippy::type_complexity)]
pub fn render(text: &str, look: &Look, x: f32, y: f32, scale: f32, backdrop: &dyn Fn(f32, f32) -> [f32; 3]) -> Option<(u32, u32, Vec<u8>, f32, f32, f32, f32)> {
    // Glass wants body: bolder strokes than the drawn clock.
    let sw = look.stroke() * 1.35;
    let pad = 2.0;
    let (tw, th) = measure(text, look);
    // Whole logical pixels: the house looks the picture up at box × scale.
    let (lw, lh) = ((tw + 2.0 * pad + sw * 0.4).ceil(), (th + 2.0 * pad + sw * 0.35).ceil());
    let mut shape = Shape::new(lw, lh);
    let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    let grow = sw * 0.175;
    let mut cx = pad + grow;
    for (k, c) in text.chars().enumerate() {
        if k > 0 {
            cx += look.gap();
        }
        let cw = if c == ':' { look.colon_w() } else { look.digit_w };
        if c == ':' {
            let r = sw * 0.5;
            shape = shape.op(vec![Cmd::Circle(cx + cw / 2.0, pad + grow + th * 0.34, r), Cmd::Circle(cx + cw / 2.0, pad + grow + th * 0.68, r)], Style::Fill(white));
        } else if let Some(d) = c.to_digit(10) {
            let base = look.stroke();
            for p in digit(d, cw - base, th - base) {
                shape = shape.op(shift(&p, cx + base / 2.0, pad + grow + base / 2.0), Style::Stroke { tint: white, width: sw });
            }
        }
        cx += cw;
    }
    let (ox, oy) = (x - pad - grow, y - pad - grow);
    let (pw, ph, rgba) = lens::picture(&shape, ox, oy, scale, lens::Glass::fitted(sw / 2.0), backdrop)?;
    Some((pw, ph, rgba, ox, oy, lw, lh))
}

/// A box blur of radius `r` (one pass, rows then columns).
fn box_blur(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut tmp = vec![0.0f32; w * h];
    let n = (2 * r + 1) as f32;
    for y in 0..h {
        let row = &src[y * w..(y + 1) * w];
        let mut sum: f32 = (0..=r).map(|x| row[x.min(w - 1)]).sum::<f32>() + row[0] * r as f32;
        for x in 0..w {
            tmp[y * w + x] = sum / n;
            sum += row[(x + r + 1).min(w - 1)] - row[x.saturating_sub(r)];
        }
    }
    let mut out = vec![0.0f32; w * h];
    for x in 0..w {
        let col = |y: usize| tmp[y.min(h - 1) * w + x];
        let mut sum: f32 = (0..=r).map(col).sum::<f32>() + col(0) * r as f32;
        for y in 0..h {
            out[y * w + x] = sum / n;
            sum += col(y + r + 1) - col(y.saturating_sub(r));
        }
    }
    out
}

/// One glyph as a glass shape in a box padded by `pad` on every side.
fn glyph_shape(c: char, cw: f32, look: &Look, pad: f32) -> Option<Shape> {
    let h = look.height();
    let sw = look.stroke();
    // The centre line sits half a stroke inside the glyph box.
    let (iw, ih) = (cw - sw, h - sw);
    let o = sw / 2.0 + pad;
    let g = look.glass.clamp(0.0, 1.0);
    let ink = look.ink;
    let body = Rgba { a: ink.a * (0.96 - 0.70 * g), ..ink };
    let rim = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.70 * g };
    let rim_low = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.22 * g };
    let shadow = Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.20 + 0.10 * (1.0 - g) };
    let mut shape = Shape::new(cw + 2.0 * pad, h + 2.0 * pad);
    if c == ':' {
        let r = sw * 0.62;
        let cx = cw / 2.0 + pad;
        let dots = |dy: f32| vec![Cmd::Circle(cx, h * 0.34 + pad + dy, r), Cmd::Circle(cx, h * 0.68 + pad + dy, r)];
        shape = shape.op(dots(sw * 0.12), Style::Fill(shadow));
        shape = shape.op(dots(0.0), Style::Fill(body));
        if g > 0.0 {
            shape = shape.op(dots(0.0), Style::Stroke { tint: rim, width: sw * 0.10 });
        }
        return Some(shape);
    }
    let d = c.to_digit(10)?;
    let paths: Vec<Vec<Cmd>> = digit(d, iw, ih).into_iter().map(|p| shift(&p, o, o)).collect();
    for p in &paths {
        shape = shape.op(shift(p, 0.0, sw * 0.14), Style::Stroke { tint: shadow, width: sw * 1.12 });
    }
    for p in &paths {
        shape = shape.op(p.clone(), Style::Stroke { tint: body, width: sw });
    }
    if g > 0.0 {
        // The light rides the upper-left edge, a faint echo the lower-right.
        let e = sw * 0.30;
        for p in &paths {
            shape = shape.op(shift(p, -e, -e), Style::Stroke { tint: rim, width: sw * 0.16 });
            shape = shape.op(shift(p, e, e), Style::Stroke { tint: rim_low, width: sw * 0.12 });
        }
    }
    Some(shape)
}

fn shift(path: &[Cmd], dx: f32, dy: f32) -> Vec<Cmd> {
    path.iter()
        .map(|c| match *c {
            Cmd::Move(x, y) => Cmd::Move(x + dx, y + dy),
            Cmd::Line(x, y) => Cmd::Line(x + dx, y + dy),
            Cmd::Quad(a, b, x, y) => Cmd::Quad(a + dx, b + dy, x + dx, y + dy),
            Cmd::Cubic(a, b, c2, d, x, y) => Cmd::Cubic(a + dx, b + dy, c2 + dx, d + dy, x + dx, y + dy),
            Cmd::Arc { cx, cy, r, start, sweep, connect } => Cmd::Arc { cx: cx + dx, cy: cy + dy, r, start, sweep, connect },
            Cmd::Circle(x, y, r) => Cmd::Circle(x + dx, y + dy, r),
            Cmd::Close => Cmd::Close,
        })
        .collect()
}

/// Turned half around in the w×h box (a 9 is a 6 upside down).
fn turn(path: &[Cmd], w: f32, h: f32) -> Vec<Cmd> {
    path.iter()
        .map(|c| match *c {
            Cmd::Move(x, y) => Cmd::Move(w - x, h - y),
            Cmd::Line(x, y) => Cmd::Line(w - x, h - y),
            Cmd::Quad(a, b, x, y) => Cmd::Quad(w - a, h - b, w - x, h - y),
            Cmd::Cubic(a, b, c2, d, x, y) => Cmd::Cubic(w - a, h - b, w - c2, h - d, w - x, h - y),
            Cmd::Arc { cx, cy, r, start, sweep, connect } => Cmd::Arc { cx: w - cx, cy: h - cy, r, start: start + PI, sweep, connect },
            Cmd::Circle(x, y, r) => Cmd::Circle(w - x, h - y, r),
            Cmd::Close => Cmd::Close,
        })
        .collect()
}

/// A rounded rectangle (x0, y0)–(x1, y1), corner radius `rr`, closed.
pub(crate) fn rounded(x0: f32, y0: f32, x1: f32, y1: f32, rr: f32) -> Vec<Cmd> {
    let rr = rr.min((x1 - x0) / 2.0).min((y1 - y0) / 2.0).max(0.0);
    vec![
        Cmd::Move(x0 + rr, y0),
        Cmd::Line(x1 - rr, y0),
        Cmd::Arc { cx: x1 - rr, cy: y0 + rr, r: rr, start: -PI / 2.0, sweep: PI / 2.0, connect: true },
        Cmd::Line(x1, y1 - rr),
        Cmd::Arc { cx: x1 - rr, cy: y1 - rr, r: rr, start: 0.0, sweep: PI / 2.0, connect: true },
        Cmd::Line(x0 + rr, y1),
        Cmd::Arc { cx: x0 + rr, cy: y1 - rr, r: rr, start: PI / 2.0, sweep: PI / 2.0, connect: true },
        Cmd::Line(x0, y0 + rr),
        Cmd::Arc { cx: x0 + rr, cy: y0 + rr, r: rr, start: PI, sweep: PI / 2.0, connect: true },
        Cmd::Close,
    ]
}

/// The centre lines of digit `d` in a w×h box (y grows downwards; a
/// positive sweep turns clockwise on screen).
fn digit(d: u32, w: f32, h: f32) -> Vec<Vec<Cmd>> {
    let r = w / 2.0;
    match d {
        0 => vec![rounded(0.0, 0.0, w, h, r)],
        1 => vec![vec![Cmd::Move(w * 0.12, h * 0.15), Cmd::Line(w * 0.62, 0.0), Cmd::Line(w * 0.62, h)]],
        2 => vec![vec![
            Cmd::Move(0.0, r),
            Cmd::Arc { cx: r, cy: r, r, start: PI, sweep: PI, connect: true },
            Cmd::Cubic(w, r + h * 0.22, w * 0.05, h * 0.62, 0.0, h),
            Cmd::Line(w, h),
        ]],
        3 => {
            let mid = h * 0.47;
            let upper_end = (mid - r * 0.55).max(r);
            let lower_start = (mid + r * 0.6).min(h - r);
            vec![
                vec![
                    Cmd::Move(0.0, r),
                    Cmd::Arc { cx: r, cy: r, r, start: PI, sweep: PI, connect: true },
                    Cmd::Line(w, upper_end),
                    Cmd::Cubic(w, mid, w * 0.6, mid, w * 0.32, mid),
                ],
                vec![
                    Cmd::Move(w * 0.32, mid),
                    Cmd::Cubic(w * 0.75, mid, w, mid, w, lower_start),
                    Cmd::Line(w, h - r),
                    Cmd::Arc { cx: r, cy: h - r, r, start: 0.0, sweep: PI, connect: true },
                ],
            ]
        }
        4 => vec![vec![Cmd::Move(w * 0.74, h), Cmd::Line(w * 0.74, 0.0), Cmd::Line(0.0, h * 0.66), Cmd::Line(w, h * 0.66)]],
        5 => {
            let shoulder = h * 0.42;
            let side = (shoulder + r * 0.85).min(h - r);
            vec![vec![
                Cmd::Move(w * 0.94, 0.0),
                Cmd::Line(w * 0.08, 0.0),
                Cmd::Line(w * 0.03, shoulder + r * 0.12),
                Cmd::Cubic(w * 0.3, shoulder - r * 0.12, w, shoulder - r * 0.12, w, side),
                Cmd::Line(w, h - r),
                Cmd::Arc { cx: r, cy: h - r, r, start: 0.0, sweep: PI, connect: true },
            ]]
        }
        6 | 9 => {
            let loop_top = h * 0.40;
            let six = vec![
                vec![
                    Cmd::Arc { cx: r, cy: r, r, start: -PI / 6.0, sweep: -5.0 * PI / 6.0, connect: false },
                    Cmd::Line(0.0, loop_top + r),
                ],
                rounded(0.0, loop_top, w, h, r),
            ];
            if d == 6 {
                six
            } else {
                six.iter().map(|p| turn(p, w, h)).collect()
            }
        }
        7 => vec![vec![Cmd::Move(0.0, 0.0), Cmd::Line(w, 0.0), Cmd::Line(w * 0.30, h)]],
        8 => {
            let mid = h * 0.46;
            vec![rounded(w * 0.06, 0.0, w * 0.94, mid, r * 0.88), rounded(0.0, mid, w, h, r)]
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_live_clock_is_made_once_per_look_and_drafts_cheaply() {
        let mut atlas = matrixkit_image::ImageAtlas::new();
        let mut live = Live::new("probe:uhr");
        let wall = std::path::Path::new("\u{0}probe-wand");
        let l = look();
        let a = live.picture(&mut atlas, "8:45", &l, 10.0, 20.0, 2.0, Pace::Still, wall, 1.0).expect("a picture");
        assert!(!a.path.to_string_lossy().contains(':'), "no ':' in a stretched name");
        let b = live.picture(&mut atlas, "8:45", &l, 30.0, 60.0, 2.0, Pace::Still, wall, 1.0).unwrap();
        assert_eq!(a.path, b.path, "moving it makes nothing new");
        assert_eq!((b.x - a.x, b.y - a.y), (20.0, 40.0));
        // A resize in the first 300 ms stretches the picture it has.
        let tall = Look { stretch: l.stretch * 1.5, ..l };
        let c = live.picture(&mut atlas, "8:45", &tall, 10.0, 20.0, 2.0, Pace::Stretch, wall, 1.0).unwrap();
        assert_eq!(c.path, a.path);
        assert!(c.h > a.h * 1.3, "{} → {}", a.h, c.h);
        // A draft is a new, smaller map; letting go makes it sharp.
        let bold = Look { weight: 0.9, ..l };
        let d = live.picture(&mut atlas, "8:45", &bold, 10.0, 20.0, 2.0, Pace::Draft, wall, 1.0).unwrap();
        assert_ne!(d.path, a.path);
        let e = live.picture(&mut atlas, "8:45", &bold, 10.0, 20.0, 2.0, Pace::Still, wall, 1.0).unwrap();
        assert_ne!(e.path, d.path, "sharp again after the draft");
        live.forget(&mut atlas);
    }

    #[test]
    fn the_default_body_is_as_legible_as_the_cards() {
        let l = look();
        assert!((l.tint() - lens::Glass::CARD_TINT).abs() < 0.001, "{}", l.tint());
        assert_eq!(Look { body: 0.0, ..l }.tint(), 0.0);
        assert_eq!(Look { body: 1.0, ..l }.tint(), 1.0);
    }

    #[test]
    fn the_weight_thickens_the_stroke() {
        let mut l = look();
        let mid = l.stroke();
        l.weight = 0.0;
        let thin = l.stroke();
        l.weight = 1.0;
        let bold = l.stroke();
        assert!(thin < mid && mid < bold);
        assert!((mid - 50.0 * (0.17 + 0.05 * 0.8)).abs() < 1e-3, "0.5 = the old clock");
        assert!(bold < l.digit_w * 0.4, "a bold digit keeps its hole");
    }

    fn look() -> Look {
        Look { digit_w: 50.0, stretch: 1.9, glass: 0.8, weight: Look::WEIGHT, body: Look::BODY, ink: Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } }
    }

    #[test]
    fn every_digit_has_strokes() {
        for d in 0..10 {
            assert!(!digit(d, 40.0, 76.0).is_empty(), "digit {d}");
        }
    }

    #[test]
    fn the_clock_grows_with_its_stretch() {
        let mut l = look();
        let (w1, h1) = measure("8:45", &l);
        l.stretch = 2.8;
        let (w2, h2) = measure("8:45", &l);
        assert_eq!(w1, w2);
        assert!(h2 > h1 * 1.4);
    }

    #[test]
    fn a_nine_is_a_turned_six() {
        let six = digit(6, 40.0, 80.0);
        let nine = digit(9, 40.0, 80.0);
        assert_eq!(six.len(), nine.len());
        match (&six[1][0], &nine[1][0]) {
            (Cmd::Move(x6, y6), Cmd::Move(x9, y9)) => {
                assert!((x6 + x9 - 40.0).abs() < 1e-4 && (y6 + y9 - 80.0).abs() < 1e-4);
            }
            _ => panic!("rounded starts with a move"),
        }
    }

    /// A look at the digits: `GLASS_CLOCK_PREVIEW=/path.ppm cargo test
    /// -p matrixkit-scene preview` paints 0–9 and a clock
    /// over a warm gradient (glass above, solid below).
    #[test]
    fn preview() {
        let Ok(out) = std::env::var("GLASS_CLOCK_PREVIEW") else { return };
        let (w, h) = (1100usize, 900usize);
        let mut img = vec![0u8; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let t = (x as f32 / w as f32 + y as f32 / h as f32) / 2.0;
                let v = [90.0 + 110.0 * (1.0 - t), 80.0 + 90.0 * (1.0 - t), 70.0 + 70.0 * (1.0 - t)];
                img[(y * w + x) * 3..][..3].copy_from_slice(&[v[0] as u8, v[1] as u8, v[2] as u8]);
            }
        }
        let mut blit = |shape: &Shape, x0: usize, y0: usize| {
            let (sw, sh, rgba) = matrixkit_shell::shapes::raster(shape, shape.w.ceil() as u32, shape.h.ceil() as u32).unwrap();
            for y in 0..sh as usize {
                for x in 0..sw as usize {
                    let (px, py) = (x0 + x, y0 + y);
                    if px >= w || py >= h {
                        continue;
                    }
                    let s = &rgba[(y * sw as usize + x) * 4..][..4];
                    let a = s[3] as f32 / 255.0;
                    for k in 0..3 {
                        let d = &mut img[(py * w + px) * 3 + k];
                        *d = (s[k] as f32 * a + *d as f32 * (1.0 - a)).min(255.0) as u8;
                    }
                }
            }
        };
        for (row, glass, stretch) in [(0usize, 0.85f32, 1.9f32)] {
            let l = Look { digit_w: 70.0, stretch, glass, weight: Look::WEIGHT, body: Look::BODY, ink: Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } };
            let pad = l.stroke() * 0.9;
            let mut x = 20.0;
            for c in "0123456789:".chars() {
                let cw = if c == ':' { l.colon_w() } else { l.digit_w };
                let shape = glyph_shape(c, cw, &l, pad).unwrap();
                blit(&shape, x as usize, 20 + row * 300);
                x += cw + l.gap() + 2.0 * pad * 0.3;
            }
        }
        // The lens over a striped backdrop (shows the bending).
        let back = |x: f32, y: f32| {
            let t = ((x * 0.05).sin() * 0.5 + 0.5) * 0.5 + (y / h as f32) * 0.3;
            [0.35 + 0.4 * t, 0.30 + 0.3 * t, 0.25 + 0.2 * t]
        };
        for y in 330..h {
            for x in 0..w {
                let c = back(x as f32, y as f32);
                img[(y * w + x) * 3..][..3].copy_from_slice(&[(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8]);
            }
        }
        let l = Look { digit_w: 80.0, stretch: 2.2, glass: 1.0, weight: Look::WEIGHT, body: Look::BODY, ink: Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } };
        let (pw, ph, rgba, ox, oy, _, _) = render("8:45", &l, 60.0, 380.0, 1.0, &back).unwrap();
        for yy in 0..ph as usize {
            for xx in 0..pw as usize {
                let (px, py) = (ox as usize + xx, oy as usize + yy);
                if px >= w || py >= h {
                    continue;
                }
                let s = &rgba[(yy * pw as usize + xx) * 4..][..4];
                let a = s[3] as f32 / 255.0;
                for k in 0..3 {
                    let d = &mut img[(py * w + px) * 3 + k];
                    *d = (s[k] as f32 * a + *d as f32 * (1.0 - a)).min(255.0) as u8;
                }
            }
        }
        // Two keypad circles over a smooth gradient (right half).
        let smooth = |x: f32, y: f32| [0.15 + 0.25 * (x / w as f32), 0.25 + 0.2 * (y / h as f32), 0.45];
        for y in 330..h {
            for x in 650..w {
                let c = smooth(x as f32, y as f32);
                img[(y * w + x) * 3..][..3].copy_from_slice(&[(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8]);
            }
        }
        for (cx, cy) in [(700.0f32, 400.0f32), (850.0, 400.0)] {
            let r = 60.0;
            let shape = Shape::new(2.0 * r, 2.0 * r).op(vec![Cmd::Circle(r, r, r - 0.5)], Style::Fill(Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }));
            let (pw, ph, rgba) = lens::picture(&shape, cx, cy, 1.0, if cx < 800.0 { lens::Glass::key(r) } else { lens::Glass::fitted(r) }, &smooth).unwrap();
            for yy in 0..ph as usize {
                for xx in 0..pw as usize {
                    let (px, py) = (cx as usize + xx, cy as usize + yy);
                    let s = &rgba[(yy * pw as usize + xx) * 4..][..4];
                    let a = s[3] as f32 / 255.0;
                    for k in 0..3 {
                        let d = &mut img[(py * w + px) * 3 + k];
                        *d = (s[k] as f32 * a + *d as f32 * (1.0 - a)).min(255.0) as u8;
                    }
                }
            }
        }
        let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
        f.extend_from_slice(&img);
        std::fs::write(out, f).unwrap();
    }

    #[test]
    fn glass_lays_a_rim_solid_does_not() {
        let mut l = look();
        let glassy = glyph_shape('8', 50.0, &l, 8.0).unwrap().ops.len();
        l.glass = 0.0;
        let solid = glyph_shape('8', 50.0, &l, 8.0).unwrap().ops.len();
        assert!(glassy > solid);
    }
}
