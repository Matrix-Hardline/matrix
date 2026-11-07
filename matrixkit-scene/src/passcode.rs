//! PASSCODE FAMILY (round 126): the phone keypad of Matrix Sperre as a kit
//! family — twelve round cells (1–9 with their letters, 0 with «+», a
//! word cell bottom left and bottom right), the code dots that fill as
//! digits come and swing after a wrong code, and the keys as LIVE GLASS
//! (`matrixkit_shell::lens`) when a backdrop picture lies behind them.
//!
//! ```ignore
//! let keys = passcode::grid(area);
//! let map = passcode::glass_keys(atlas, keys[0].w, scale, &wall, veil, fade);
//! passcode::lay_dots(&mut b, cx, y, 4, typed, shake, fade);
//! passcode::lay_keys(&mut b, engine, &keys, &passcode::Look { map, held, left, right, fade, role_base });
//! match passcode::press(role, role_base) { Some(Press::Digit(c)) => … }
//! ```

use matrixkit_image::{Fit, ImageAtlas};
use matrixkit_shell::{Pict, SceneBuilder};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::Rect;
use std::path::{Path, PathBuf};

/// The letters under the digits (index = digit).
pub const LETTERS: [&str; 10] = ["+", "", "ABC", "DEF", "GHI", "JKL", "MNO", "PQRS", "TUV", "WXYZ"];
/// Cell 9: the word bottom left (e.g. «OK»).
pub const CELL_LEFT: usize = 9;
/// Cell 10: the 0.
pub const CELL_ZERO: usize = 10;
/// Cell 11: the word bottom right («Abbrechen» / «Löschen»).
pub const CELL_RIGHT: usize = 11;

const WHITE: mk::Rgba = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

/// What a press on a cell means.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Press {
    Digit(char),
    Left,
    Right,
}

/// The cell under `role` (roles run `role_base` … `role_base + 11`).
pub fn press(role: Option<usize>, role_base: usize) -> Option<Press> {
    match role?.checked_sub(role_base)? {
        k @ 0..=8 => Some(Press::Digit(char::from(b'1' + k as u8))),
        CELL_LEFT => Some(Press::Left),
        CELL_ZERO => Some(Press::Digit('0')),
        CELL_RIGHT => Some(Press::Right),
        _ => None,
    }
}

/// The twelve cells, as large as `area` allows (a finger's size at most
/// 84 px), centred: three columns, four rows.
pub fn grid(area: Rect) -> [Rect; 12] {
    let d = (area.h / (4.0 + 3.0 * 0.30)).min(area.w / (3.0 + 2.0 * 0.42)).min(84.0).floor();
    let (gx, gy) = (d * 0.42, d * 0.30);
    let left = area.x + (area.w - (3.0 * d + 2.0 * gx)) / 2.0;
    let top = area.y + (area.h - (4.0 * d + 3.0 * gy)) / 2.0;
    std::array::from_fn(|k| Rect { x: left + (k % 3) as f32 * (d + gx), y: top + (k / 3) as f32 * (d + gy), w: d, h: d })
}

/// The keys' live glass: ONE lens map for cells of `side` (made once),
/// registered over `backdrop` (a stretched picture of the same scene)
/// with this frame's veil and fade. Returns the map's path for `Look`.
pub fn glass_keys(atlas: &mut ImageAtlas, side: f32, scale: f32, backdrop: &Path, veil: f32, fade: f32) -> PathBuf {
    let side = side.floor();
    let path = PathBuf::from(format!("\u{0}passcode-glas-{}-{}", side as u32, scale as u32));
    let glass = crate::glass_clock::lens::Glass::key(side / 2.0);
    if atlas.stretched_ready(&path).is_none() {
        let r = side / 2.0;
        let shape = matrixkit_shell::shapes::Shape::new(side, side)
            .op(vec![matrixkit_shell::shapes::Cmd::Circle(r, r, r - 0.5)], matrixkit_shell::shapes::Style::Fill(WHITE));
        let t0 = std::time::Instant::now();
        if let Some((w, h, map)) = crate::glass_clock::lens::map(&shape, scale, glass.band) {
            atlas.put(&path, w, h, map);
        }
        eprintln!("[glas] Tastenkarte {side} gerechnet in {} ms", t0.elapsed().as_millis());
    }
    // Round 140: the keys outlive frames of the other panel.
    atlas.pin(&path);
    matrixkit_shell::lens::set(&path, matrixkit_shell::lens::Lens {
        backdrop: backdrop.to_path_buf(),
        bend: glass.bend,
        scrim: glass.scrim,
        rim: glass.rim,
        veil,
        tint: 0.0,
        opacity: fade,
    });
    path
}

/// The code dots centred on `cx`: `n` places, `typed` filled, swung
/// sideways by `shake` (a spring after a wrong code).
#[allow(clippy::too_many_arguments)]
pub fn lay_dots(b: &mut SceneBuilder, cx: f32, y: f32, n: usize, typed: usize, shake: f32, fade: f32) {
    let (dot, gap) = (13.0, 20.0);
    let total = n as f32 * dot + n.saturating_sub(1) as f32 * gap;
    let sx = cx - total / 2.0 + shake * 26.0;
    for k in 0..n {
        let x = sx + k as f32 * (dot + gap);
        if k < typed {
            b.quad(x, y, dot, dot, WHITE.with_alpha(0.95 * fade), dot / 2.0, None, None);
        } else {
            b.quad(x, y, dot, dot, WHITE.with_alpha(0.80 * fade), dot / 2.0, None, None);
            b.quad(x + 1.6, y + 1.6, dot - 3.2, dot - 3.2, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.45 * fade }, (dot - 3.2) / 2.0, None, None);
        }
    }
}

/// How the keys look this frame.
pub struct Look<'a> {
    /// The live-glass map (`glass_keys`); None = a light frosted disc.
    pub map: Option<&'a Path>,
    /// The cell under the finger (lit).
    pub held: Option<usize>,
    pub left: Option<&'a str>,
    pub right: Option<&'a str>,
    pub fade: f32,
    pub role_base: usize,
    /// Moved by (for sliding in).
    pub dy: f32,
    pub home: Rect,
}

/// The twelve cells.
pub fn lay_keys(b: &mut SceneBuilder, engine: &mut TextEngine, keys: &[Rect; 12], look: &Look) {
    let num_px = keys[0].w * 0.40;
    let fade = look.fade;
    for (k, cell) in keys.iter().enumerate() {
        let r = Rect { y: cell.y + look.dy, ..*cell };
        let role = Some(look.role_base + k);
        let digit = match k {
            0..=8 => Some(k + 1),
            CELL_ZERO => Some(0),
            _ => None,
        };
        if let Some(d) = digit {
            let anchor = b.quad(r.x, r.y, r.w, r.h, mk::Rgba { r: 0.5, g: 0.5, b: 0.5, a: 0.0 }, r.w / 2.0, role, None);
            match look.map {
                Some(map) => b.pict(Pict {
                    path: map.to_path_buf(),
                    box_w: r.w.floor() as u32,
                    box_h: r.w.floor() as u32,
                    // Stretch: found at any scale (a pane may draw at another).
                    fit: Fit::Stretch,
                    x: r.x,
                    y: r.y,
                    radius: r.w / 2.0,
                    home: look.home,
                    anchor,
                    clip: None,
                }),
                None => {
                    b.quad(r.x, r.y, r.w, r.h, WHITE.with_alpha(0.18 * fade), r.w / 2.0, role, None);
                }
            }
            if look.held == Some(k) {
                b.quad(r.x, r.y, r.w, r.h, WHITE.with_alpha(0.45), r.w / 2.0, role, None);
            }
            let num = d.to_string();
            let letters = if d == 1 { "" } else { LETTERS[d] };
            let (nw, nh) = engine.measure_family(matrixkit_text::FAMILY, &num, num_px, mk::typo::Weight::Normal);
            let ny = r.y + r.h * if letters.is_empty() { 0.5 } else { 0.44 } - nh / 2.0;
            b.text(engine, num, num_px, mk::typo::Weight::Normal, matrixkit_text::FAMILY, WHITE.with_alpha(fade), r.x + (r.w - nw) / 2.0, ny, role, None);
            if !letters.is_empty() {
                let lpx = num_px * 0.30;
                let (lw, _) = engine.measure_family(matrixkit_text::FAMILY, letters, lpx, mk::typo::Weight::Semibold);
                b.text(engine, letters, lpx, mk::typo::Weight::Semibold, matrixkit_text::FAMILY, WHITE.with_alpha(0.85 * fade), r.x + (r.w - lw) / 2.0, ny + nh * 0.98, role, None);
            }
            continue;
        }
        let word = match k {
            CELL_LEFT => look.left,
            _ => look.right,
        };
        let Some(word) = word else { continue };
        // A word cell: a full-size target, only the word shows.
        b.quad(r.x, r.y, r.w, r.h, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, r.w / 2.0, role, None);
        let px = mk::typo::BODY.px() * 1.15;
        let (ww, wh) = engine.measure_family(matrixkit_text::FAMILY, word, px, mk::typo::Weight::Medium);
        b.text(engine, word, px, mk::typo::Weight::Medium, matrixkit_text::FAMILY, WHITE.with_alpha(fade), r.x + (r.w - ww) / 2.0, r.y + (r.h - wh) / 2.0, role, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_fits_and_is_finger_sized() {
        for (w, h) in [(653.0, 489.0), (489.0, 653.0), (390.0, 520.0)] {
            let keys = grid(Rect { x: 0.0, y: 0.0, w, h });
            for k in &keys {
                assert!(k.x >= 0.0 && k.x + k.w <= w && k.y >= 0.0 && k.y + k.h <= h, "{w}×{h}: {k:?}");
                assert!(k.w >= 44.0, "{w}×{h}: {}", k.w);
                assert_eq!(k.w, k.w.floor(), "whole pixels (one glass map for all)");
            }
        }
    }

    #[test]
    fn presses_name_their_cells() {
        assert_eq!(press(Some(100), 100), Some(Press::Digit('1')));
        assert_eq!(press(Some(108), 100), Some(Press::Digit('9')));
        assert_eq!(press(Some(110), 100), Some(Press::Digit('0')));
        assert_eq!(press(Some(109), 100), Some(Press::Left));
        assert_eq!(press(Some(111), 100), Some(Press::Right));
        assert_eq!(press(Some(112), 100), None);
        assert_eq!(press(Some(5), 100), None);
    }
}
