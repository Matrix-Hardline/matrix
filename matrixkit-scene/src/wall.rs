//! WALL FAMILY (round 167, 05.11.2026): a full-screen picture behind a
//! surface that is ONE picture over the duo's two panels — the lock screen
//! and the login screen both laid it by hand. The span is the height of
//! both panels; each surface shows its part (`shift` = how far down it
//! begins). The picture is the backdrop of every live glass on top (the
//! glass clock, glass keys, glass cards), so it carries a stable name.
//!
//! ```ignore
//! let mut wall = wall::Wall::new("anmeldung-wand");
//! wall.take(atlas, fresh);                       // a newly decoded picture
//! let (total, shift) = wall::span(part, hf, other_h);
//! wall.lay(&mut b, wf, hf, total, shift, ground, 0.22);
//! let top = b.rise();                            // everything after lies above
//! glass_key(&mut b, engine, atlas, &wall, …, top);
//! ```

use matrixkit_image::{Fit, ImageAtlas};
use matrixkit_shell::{Pict, SceneBuilder};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::Rect;
use std::path::PathBuf;

const WHITE: mk::Rgba = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

/// Which part of a two-panel picture a surface shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// One screen: the whole picture.
    Whole,
    /// The upper panel.
    Upper,
    /// The lower panel.
    Lower,
}

/// (span height, shift) for a surface `hf` high showing `part`, when the
/// other panel is `other_h` high (0 = not known yet: as high as this one).
pub fn span(part: Part, hf: f32, other_h: f32) -> (f32, f32) {
    let other = if other_h > 0.0 { other_h } else { hf };
    match part {
        Part::Whole => (hf, 0.0),
        Part::Upper => (hf + other, 0.0),
        Part::Lower => (other + hf, other),
    }
}

/// The picture and its name in the atlas.
#[derive(Debug)]
pub struct Wall {
    stem: String,
    serial: u64,
    shown: Option<(u32, u32)>,
}

impl Wall {
    /// `stem`: the picture's name stem (no ':' — the serial follows one).
    pub fn new(stem: &str) -> Wall {
        Wall { stem: stem.replace(':', "-"), serial: 0, shown: None }
    }

    /// A newly decoded picture (w, h, RGBA) goes into the atlas.
    pub fn take(&mut self, atlas: &mut ImageAtlas, fresh: Option<(u32, u32, Vec<u8>)>) {
        if let Some((w, h, rgba)) = fresh {
            if w == 0 || h == 0 {
                return;
            }
            self.serial += 1;
            atlas.put(&self.path(), w, h, rgba);
            self.shown = Some((w, h));
        }
    }

    /// The picture's name — the backdrop of live glass laid on top.
    pub fn path(&self) -> PathBuf {
        PathBuf::from(format!("\u{0}{}:{}", self.stem, self.serial))
    }

    pub fn shown(&self) -> Option<(u32, u32)> {
        self.shown
    }

    /// Cover the span, show this surface's part, a `veil` over it (0 = none).
    #[allow(clippy::too_many_arguments)]
    pub fn lay(&self, b: &mut SceneBuilder, wf: f32, hf: f32, total: f32, shift: f32, anchor: usize, veil: f32) {
        let Some((pw, ph)) = self.shown else { return };
        let home = Rect { x: 0.0, y: 0.0, w: wf, h: hf };
        let (x, y, bw, bh) = cover(pw, ph, wf, total, shift);
        b.pict(Pict { path: self.path(), box_w: bw as u32, box_h: bh as u32, fit: Fit::Stretch, x, y, radius: 0.0, home, anchor, clip: Some(home) });
        if veil > 0.0 {
            b.quad(0.0, 0.0, wf, hf, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: veil }, 0.0, None, None);
        }
    }
}

/// Where a picture `pw`×`ph` lies to cover `wf`×`total`, this surface
/// starting `shift` down: (x, y, w, h) (pure — tested).
pub fn cover(pw: u32, ph: u32, wf: f32, total: f32, shift: f32) -> (f32, f32, f32, f32) {
    let s = (wf / pw.max(1) as f32).max(total / ph.max(1) as f32);
    let (bw, bh) = (pw as f32 * s, ph as f32 * s);
    ((wf - bw) / 2.0, (total - bh) / 2.0 - shift, bw, bh)
}

/// GLASS KEY (round 167, from the login screen): a round key of live glass
/// over the wall with a symbol — the power keys. Without a wall, the plain
/// round icon. The role lies on an invisible quad (hits), lit when `lit`.
#[allow(clippy::too_many_arguments)]
pub fn glass_key(b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, wall: &Wall, glyph: char, x: f32, y: f32, d: f32, lit: bool, role: usize, home: Rect, anchor: usize, p: mk::Palette) {
    if wall.shown().is_none() {
        crate::scene::lay_round_icon(b, engine, glyph, x, y, d, lit, Some(role), None, p);
        return;
    }
    let map = crate::glass_clock::lens::card(atlas, d, d, d / 2.0, matrixkit_shell::screen_scale() as f32, &wall.path(), 0.0, 1.0);
    b.pict(Pict { path: map, box_w: d as u32, box_h: d as u32, fit: Fit::Stretch, x, y, radius: d / 2.0, home, anchor, clip: None });
    b.quad(x, y, d, d, WHITE.with_alpha(if lit { 0.30 } else { 0.0 }), d / 2.0, Some(role), None);
    let gpx = mk::typo::symbol_px(22.0);
    let family = crate::symbol_family();
    let (gw, gh) = engine.measure_family(family, &glyph.to_string(), gpx, mk::typo::Weight::Normal);
    b.text(engine, glyph.to_string(), gpx, mk::typo::Weight::Normal, family, WHITE, x + (d - gw) / 2.0, y + (d - gh) / 2.0, Some(role), None);
}

/// A glass card over the wall in `r` (the keyboard's ground); then a new
/// layer, so what follows lies on the glass. Returns the new anchor.
pub fn glass_ground(b: &mut SceneBuilder, atlas: &mut ImageAtlas, wall: &Wall, r: Rect, radius: f32, home: Rect, anchor: usize) -> usize {
    if wall.shown().is_some() {
        let map = crate::glass_clock::lens::card(atlas, r.w, r.h, radius, matrixkit_shell::screen_scale() as f32, &wall.path(), 0.0, 1.0);
        b.pict(Pict { path: map, box_w: r.w as u32, box_h: r.h as u32, fit: Fit::Stretch, x: r.x, y: r.y, radius, home, anchor, clip: None });
    }
    b.rise()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_panels_share_one_picture() {
        let (t_up, s_up) = span(Part::Upper, 491.0, 491.0);
        let (t_lo, s_lo) = span(Part::Lower, 491.0, 491.0);
        assert_eq!((t_up, s_up), (982.0, 0.0));
        assert_eq!((t_lo, s_lo), (982.0, 491.0));
        assert_eq!(span(Part::Upper, 491.0, 0.0), (982.0, 0.0), "the other panel not known yet");
        assert_eq!(span(Part::Whole, 700.0, 491.0), (700.0, 0.0));
        // The lower part continues where the upper one ends.
        let up = cover(1800, 1200, 653.0, 982.0, 0.0);
        let lo = cover(1800, 1200, 653.0, 982.0, 491.0);
        assert!((lo.1 - (up.1 - 491.0)).abs() < 1e-3);
        assert!(up.2 >= 653.0 && up.3 >= 982.0, "it covers");
    }
}
