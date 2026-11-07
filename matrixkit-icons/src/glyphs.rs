//! The glyphs — Icons 3.0 (2026-07-19): DESIGNED, not just generated.
//!
//! User doctrine of this round: "Apple designs the shapes, the system
//! generates the appearance — why not us?" Every glyph here is a
//! layered, storytelling object (papers INSIDE the folder, grooves with
//! a scale, a screen in a case) — the generator colors the layers from
//! the living palette and applies seams, shadows and gradients.
//! Sign-off of the shape language: lighthouse round 2026-07-19 ("now I am
//! thrilled"), basis: Files, Settings, Terminal.

use crate::{dark, light, with_alpha, round_rect, squircle, Glyph, Painter};
use tiny_skia::PathBuilder;

fn circle(cx: f32, cy: f32, r: f32) -> tiny_skia::Path {
    let mut pb = PathBuilder::new();
    pb.push_circle(cx, cy, r);
    pb.finish().expect("circle")
}

fn triangle(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> tiny_skia::Path {
    let mut pb = PathBuilder::new();
    pb.move_to(a.0, a.1);
    pb.line_to(b.0, b.1);
    pb.line_to(c.0, c.1);
    pb.close();
    pb.finish().expect("triangle")
}

/// matrix-sysmon: a measuring instrument — scale lines in the ground,
/// three levels of different height on a baseline, peak-value dot.
pub fn glyph_sysmon(g: &Glyph, z: &mut Painter) {
    for y in [76.0, 106.0, 136.0, 166.0] {
        z.fine(&round_rect(52.0, y, 204.0, y + 2.0, 1.0), with_alpha(g.b, 0.22));
    }
    z.fill(&round_rect(52.0, 186.0, 204.0, 194.0, 4.0), g.b);
    z.fill_history(&round_rect(70.0, 130.0, 98.0, 186.0, 6.0), light(g.b, 0.15), g.b);
    z.fill_history(&round_rect(114.0, 88.0, 142.0, 186.0, 6.0), light(g.a, 0.2), g.a);
    z.fill_history(&round_rect(158.0, 112.0, 186.0, 186.0, 6.0), light(g.c, 0.15), g.c);
    z.fine(&circle(128.0, 78.0, 5.0), light(g.a, 0.35));
}

/// matrix-colors: three paint plates on a palette — with a mixing core.
pub fn glyph_colors(g: &Glyph, z: &mut Painter) {
    z.fill_history(&squircle(128.0, 138.0, 86.0, 3.2), light(g.b, 0.08), dark(g.b, 0.14));
    z.fill(&circle(104.0, 116.0, 34.0), with_alpha(g.a, 0.88));
    z.fill(&circle(154.0, 122.0, 34.0), with_alpha(g.c, 0.88));
    z.fill(&circle(126.0, 160.0, 34.0), with_alpha(light(g.b, 0.3), 0.88));
    // Mixing core: where all three meet, it gets bright
    z.fine(&circle(128.0, 133.0, 12.0), with_alpha(light(g.a, 0.55), 0.85));
    z.ring(128.0, 133.0, 12.0, 2.0, with_alpha(dark(g.b, 0.3), 0.4));
}

/// matrix-help: an open book — fold, lines, reading ribbon.
pub fn glyph_help(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(52.0, 84.0, 204.0, 184.0, 14.0), light(g.b, 0.1), dark(g.b, 0.1));
    z.fill(&round_rect(62.0, 92.0, 124.0, 176.0, 8.0), light(g.c, 0.35));
    z.fill(&round_rect(132.0, 92.0, 194.0, 176.0, 8.0), light(g.c, 0.3));
    z.fine(&round_rect(126.0, 92.0, 130.0, 176.0, 2.0), with_alpha(dark(g.b, 0.4), 0.5));
    for (x1, x2) in [(72.0, 114.0), (142.0, 184.0)] {
        for i in 0..4 {
            let y = 106.0 + i as f32 * 16.0;
            let end = if i == 3 { x1 + (x2 - x1) * 0.6 } else { x2 };
            z.fine(&round_rect(x1, y, end, y + 4.0, 2.0), with_alpha(dark(g.b, 0.45), 0.55));
        }
    }
    // reading ribbon
    z.fill(&round_rect(160.0, 84.0, 172.0, 122.0, 3.0), g.a);
    z.fine(&triangle((160.0, 122.0), (172.0, 122.0), (166.0, 112.0)), dark(g.a, 0.25));
}

/// matrix-sounds: a speaker chassis with cone + sound arcs.
pub fn glyph_sounds(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(56.0, 104.0, 96.0, 152.0, 8.0), light(g.b, 0.12), g.b);
    z.fill(&triangle((96.0, 104.0), (96.0, 152.0), (140.0, 188.0)), g.b);
    z.fill(&triangle((96.0, 104.0), (140.0, 68.0), (140.0, 188.0)), light(g.b, 0.12));
    z.fine(&round_rect(64.0, 112.0, 88.0, 144.0, 5.0), with_alpha(dark(g.b, 0.3), 0.4));
    for (i, r) in [24.0_f32, 42.0, 60.0].iter().enumerate() {
        let a = 1.0 - i as f32 * 0.28;
        z.arc(148.0, 128.0, *r, -52.0, 52.0, 9.0, with_alpha(g.a, a));
    }
}

/// matrix-codes: the code card — digit field + running countdown ring.
pub fn glyph_codes(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(48.0, 84.0, 208.0, 172.0, 16.0), light(g.b, 0.14), g.b);
    z.fill(&round_rect(62.0, 100.0, 158.0, 132.0, 8.0), dark(g.b, 0.28));
    for i in 0..3 {
        z.fine(&circle(80.0 + i as f32 * 30.0, 116.0, 7.0), light(g.c, 0.25));
    }
    z.fine(&round_rect(62.0, 144.0, 130.0, 152.0, 4.0), with_alpha(light(g.b, 0.35), 0.7));
    // Countdown: ring + draining arc
    z.ring(182.0, 116.0, 17.0, 5.0, with_alpha(dark(g.b, 0.3), 0.6));
    z.arc(182.0, 116.0, 17.0, -90.0, 140.0, 5.0, g.a);
    z.fine(&circle(182.0, 116.0, 4.0), g.a);
}

/// matrix-keys-app: THE key — classic silhouette: bow with hole,
/// shaft, two bit teeth, light edge on the metal.
pub fn glyph_key_app(g: &Glyph, z: &mut Painter) {
    z.fill_history(&circle(84.0, 118.0, 34.0), light(g.a, 0.18), g.a);
    z.fine(&circle(84.0, 118.0, 15.0), dark(g.b, 0.35));
    z.fine(&circle(80.0, 114.0, 4.0), with_alpha(light(g.a, 0.5), 0.7));
    z.fill_history(&round_rect(112.0, 110.0, 204.0, 126.0, 7.0), light(g.a, 0.18), g.a);
    z.fill(&round_rect(158.0, 126.0, 172.0, 152.0, 4.0), g.a);
    z.fill(&round_rect(184.0, 126.0, 198.0, 162.0, 4.0), g.a);
    z.fine(&round_rect(116.0, 113.0, 200.0, 116.5, 1.7), with_alpha(light(g.a, 0.5), 0.6));
}

/// matrix-recovery: the guardian shield — two-layered, watchful eye.
pub fn glyph_guard(g: &Glyph, z: &mut Painter) {
    let shield = |s: f32| {
        let mut pb = PathBuilder::new();
        pb.move_to(128.0, 62.0 + s);
        pb.line_to(196.0 - s, 92.0 + s * 0.6);
        pb.line_to(188.0 - s, 158.0 - s * 0.4);
        pb.cubic_to(184.0 - s, 186.0 - s, 156.0 - s, 200.0 - s, 128.0, 208.0 - s);
        pb.cubic_to(100.0 + s, 200.0 - s, 72.0 + s, 186.0 - s, 68.0 + s, 158.0 - s * 0.4);
        pb.line_to(60.0 + s, 92.0 + s * 0.6);
        pb.close();
        pb.finish().expect("shield")
    };
    z.fill_history(&shield(0.0), light(g.b, 0.12), dark(g.b, 0.1));
    z.fill(&shield(12.0), dark(g.b, 0.22));
    z.fill(&circle(128.0, 136.0, 26.0), light(g.c, 0.25));
    z.fine(&circle(128.0, 136.0, 12.0), dark(g.b, 0.45));
    z.fine(&circle(133.0, 131.0, 4.0), light(g.c, 0.6));
    z.arc(128.0, 136.0, 26.0, 200.0, 340.0, 4.0, with_alpha(light(g.b, 0.4), 0.5));
}

/// matrix-settings: the precision instrument (lighthouse design) —
/// recessed grooves, scale, value fill, knurled knobs.
pub fn glyph_settings(g: &Glyph, z: &mut Painter) {
    for (y, kx) in [(86.0_f32, 162.0_f32), (128.0, 94.0), (170.0, 130.0)] {
        z.fill(&round_rect(58.0, y - 6.0, 198.0, y + 6.0, 6.0), dark(g.b, 0.4));
        z.fine(&round_rect(58.0, y + 4.0, 198.0, y + 7.0, 2.0), with_alpha(light(g.b, 0.4), 0.35));
        z.fine(&round_rect(61.0, y - 3.0, kx, y + 3.0, 3.0), g.b);
        let mut tx = 70.0;
        while tx < 192.0 {
            z.fine(&round_rect(tx, y - 14.0, tx + 2.0, y - 10.0, 1.0), with_alpha(light(g.b, 0.4), 0.4));
            tx += 12.0;
        }
    }
    for ((y, kx), c) in [(86.0_f32, 162.0_f32), (128.0, 94.0), (170.0, 130.0)]
        .iter()
        .zip([g.a, g.a, g.c])
    {
        let (y, kx) = (*y, *kx);
        z.fill_history(&circle(kx, y, 18.0), light(c, 0.25), c);
        for w in (0..360).step_by(45) {
            let wr = (w as f32).to_radians();
            z.fine(
                &circle(kx + 14.0 * wr.cos(), y + 14.0 * wr.sin(), 1.6),
                with_alpha(dark(c, 0.4), 0.7),
            );
        }
        z.fine(&circle(kx, y, 5.0), dark(c, 0.45));
    }
}

/// matrix-updater: the cycle — two arcs with arrowheads, in the
/// middle the fresh state as a dot (deliberately NO arrow-in-tray,
/// that belongs to the installer).
pub fn glyph_updater(g: &Glyph, z: &mut Painter) {
    z.fine(&circle(128.0, 128.0, 66.0), with_alpha(dark(g.b, 0.25), 0.5));
    z.arc(128.0, 128.0, 48.0, 210.0, 330.0, 13.0, g.a);
    z.arc(128.0, 128.0, 48.0, 30.0, 150.0, 13.0, g.a);
    // Arrowheads at the arc ends (at 330° top right, 150° bottom left)
    z.fill(&triangle((162.0, 86.0), (188.0, 100.0), (160.0, 118.0)), g.a);
    z.fill(&triangle((94.0, 170.0), (68.0, 156.0), (96.0, 138.0)), g.a);
    z.fill(&circle(128.0, 128.0, 15.0), light(g.c, 0.15));
    z.fine(&circle(123.0, 123.0, 5.0), with_alpha(light(g.c, 0.5), 0.8));
}

/// matrix-dev-access: brackets with depth + blinking cursor.
pub fn glyph_developer_access(g: &Glyph, z: &mut Painter) {
    let bracket = |x: f32, dir: f32| {
        let mut pb = PathBuilder::new();
        pb.move_to(x + 20.0 * dir, 88.0);
        pb.line_to(x, 88.0);
        pb.line_to(x, 168.0);
        pb.line_to(x + 20.0 * dir, 168.0);
        pb.line_to(x + 20.0 * dir, 154.0);
        pb.line_to(x + 14.0 * dir, 154.0);
        pb.line_to(x + 14.0 * dir, 102.0);
        pb.line_to(x + 20.0 * dir, 102.0);
        pb.close();
        pb.finish().expect("bracket")
    };
    z.fill(&bracket(70.0, 1.0), g.b);
    z.fill(&bracket(186.0, -1.0), g.b);
    z.fill(&triangle((142.0, 84.0), (122.0, 172.0), (108.0, 172.0)), g.a);
    z.fine(&triangle((142.0, 84.0), (128.0, 84.0), (114.0, 168.0)), with_alpha(light(g.a, 0.3), 0.5));
    z.fill(&round_rect(152.0, 148.0, 170.0, 172.0, 3.0), g.c);
}

/// matrix-canvas: the infinite surface — dot grid + two windows.
pub fn glyph_canvas(g: &Glyph, z: &mut Painter) {
    for gy in 0..4 {
        for gx in 0..5 {
            z.fine(
                &circle(64.0 + gx as f32 * 32.0, 76.0 + gy as f32 * 34.0, 2.4),
                with_alpha(g.b, 0.4),
            );
        }
    }
    z.fill_history(&round_rect(58.0, 92.0, 138.0, 158.0, 10.0), light(g.a, 0.15), g.a);
    z.fine(&round_rect(58.0, 92.0, 138.0, 104.0, 6.0), with_alpha(dark(g.a, 0.25), 0.7));
    z.fine(&circle(67.0, 98.0, 2.6), light(g.a, 0.5));
    z.fill_history(&round_rect(122.0, 128.0, 198.0, 186.0, 10.0), light(g.c, 0.15), g.c);
    z.fine(&round_rect(122.0, 128.0, 198.0, 140.0, 6.0), with_alpha(dark(g.c, 0.25), 0.7));
    z.fine(&circle(131.0, 134.0, 2.6), light(g.c, 0.5));
}

/// matrix-web: the globe — graticule of equator, latitudes and
/// central meridian, plus a glowing location with a locator ring.
pub fn glyph_web(g: &Glyph, z: &mut Painter) {
    z.fill_history(&circle(128.0, 128.0, 64.0), light(g.b, 0.22), dark(g.b, 0.08));
    z.ring(128.0, 128.0, 64.0, 4.5, dark(g.b, 0.28));
    // Graticule: latitude lines (shorter inside the sphere) + meridian
    for (y, half) in [(96.0_f32, 50.0_f32), (128.0, 62.0), (160.0, 50.0)] {
        z.fine(
            &round_rect(128.0 - half, y - 1.5, 128.0 + half, y + 1.5, 1.5),
            with_alpha(light(g.b, 0.45), 0.55),
        );
    }
    z.fine(&round_rect(126.0, 66.0, 130.0, 190.0, 2.0), with_alpha(light(g.b, 0.45), 0.55));
    z.arc(158.0, 128.0, 68.0, 115.0, 245.0, 3.0, with_alpha(light(g.b, 0.45), 0.5));
    z.arc(98.0, 128.0, 68.0, -65.0, 65.0, 3.0, with_alpha(light(g.b, 0.45), 0.5));
    // Location
    z.fine(&circle(152.0, 106.0, 7.5), light(g.a, 0.2));
    z.ring(152.0, 106.0, 13.0, 2.5, with_alpha(g.a, 0.6));
}

/// matrix-files: the folder WITH content (lighthouse design) — papers,
/// dog-ear, front with light edge, fold and seam.
pub fn glyph_files(g: &Glyph, z: &mut Painter) {
    z.fill(&round_rect(50.0, 66.0, 120.0, 112.0, 13.0), g.b);
    z.fill_history(&round_rect(48.0, 84.0, 208.0, 196.0, 17.0), light(g.b, 0.1), g.b);
    // Papers
    z.fill(&round_rect(66.0, 75.0, 194.0, 130.0, 5.0), with_alpha(light(g.c, 0.42), 0.9));
    z.fill(&round_rect(72.0, 69.0, 199.0, 125.0, 5.0), light(g.c, 0.52));
    z.fine(&triangle((199.0, 69.0), (199.0, 83.0), (186.0, 69.0)), light(g.c, 0.3));
    for (i, end) in [164.0_f32, 176.0, 150.0].iter().enumerate() {
        z.fine(
            &round_rect(84.0, 79.0 + i as f32 * 8.0, *end, 82.0 + i as f32 * 8.0, 1.5),
            with_alpha(dark(g.b, 0.4), 0.6),
        );
    }
    // Front
    z.fill_history(&round_rect(48.0, 106.0, 208.0, 196.0, 17.0), light(g.a, 0.16), g.a);
    z.fine(&round_rect(48.0, 106.0, 208.0, 118.0, 10.0), with_alpha(light(g.a, 0.4), 0.45));
    z.fine(&round_rect(60.0, 176.0, 196.0, 178.5, 1.2), with_alpha(dark(g.a, 0.4), 0.45));
    let mut x = 66.0;
    while x < 194.0 {
        z.fine(&circle(x, 184.0, 1.6), with_alpha(dark(g.a, 0.4), 0.45));
        x += 12.0;
    }
}

/// matrix-morpheus: the installation — an arrow lowers into the slot.
pub fn glyph_installer(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(120.0, 58.0, 136.0, 118.0, 8.0), light(g.a, 0.2), g.a);
    z.fill(&triangle((104.0, 112.0), (152.0, 112.0), (128.0, 144.0)), g.a);
    z.fill_history(&round_rect(56.0, 158.0, 200.0, 196.0, 14.0), light(g.b, 0.12), g.b);
    z.fill(&round_rect(104.0, 152.0, 152.0, 168.0, 8.0), dark(g.b, 0.35));
    z.fine(&round_rect(104.0, 164.0, 152.0, 168.0, 2.0), with_alpha(light(g.b, 0.35), 0.4));
    for i in 0..3 {
        z.fine(&circle(84.0 + i as f32 * 12.0, 182.0, 2.6), with_alpha(light(g.b, 0.4), 0.6));
    }
}

/// matrix-for-surface: the Secure-Boot stick maker — a USB stick with a
/// lock lowers into the slot (family kinship with the installer glyph).
pub fn glyph_surface_stick(g: &Glyph, z: &mut Painter) {
    // stick body with connector, pointing down into the slot
    z.fill_history(&round_rect(104.0, 56.0, 152.0, 130.0, 10.0), light(g.a, 0.18), g.a);
    z.fill(&round_rect(116.0, 126.0, 140.0, 150.0, 3.0), light(g.b, 0.4));
    z.fine(&round_rect(121.0, 132.0, 126.0, 144.0, 1.5), with_alpha(dark(g.b, 0.4), 0.5));
    z.fine(&round_rect(130.0, 132.0, 135.0, 144.0, 1.5), with_alpha(dark(g.b, 0.4), 0.5));
    // the lock: Secure Boot rides on the stick
    z.ring(128.0, 88.0, 9.0, 4.0, light(g.c, 0.25));
    z.fill(&round_rect(115.0, 90.0, 141.0, 114.0, 6.0), g.c);
    z.fine(&circle(128.0, 100.0, 2.8), with_alpha(dark(g.c, 0.45), 0.7));
    // the slot base (same grammar as the installer)
    z.fill_history(&round_rect(56.0, 158.0, 200.0, 196.0, 14.0), light(g.b, 0.12), g.b);
    z.fill(&round_rect(104.0, 152.0, 152.0, 168.0, 8.0), dark(g.b, 0.35));
    z.fine(&round_rect(104.0, 164.0, 152.0, 168.0, 2.0), with_alpha(light(g.b, 0.35), 0.4));
    for i in 0..3 {
        z.fine(&circle(84.0 + i as f32 * 12.0, 182.0, 2.6), with_alpha(light(g.b, 0.4), 0.6));
    }
}

/// matrix-construct: the development environment — a hammer over the
/// blueprint (the reference's tool motif), code chevrons in the ground.
pub fn glyph_construct(g: &Glyph, z: &mut Painter) {
    // blueprint with a fine grid
    z.fill_history(&round_rect(52.0, 72.0, 204.0, 196.0, 14.0), light(g.b, 0.14), g.b);
    for i in 0..4 {
        let y = 96.0 + i as f32 * 26.0;
        z.fine(&round_rect(62.0, y, 194.0, y + 1.6, 0.8), with_alpha(light(g.b, 0.4), 0.35));
    }
    // code chevrons live in the blueprint
    z.fine(&triangle((84.0, 118.0), (84.0, 146.0), (68.0, 132.0)), with_alpha(light(g.b, 0.5), 0.8));
    z.fine(&triangle((172.0, 118.0), (172.0, 146.0), (188.0, 132.0)), with_alpha(light(g.b, 0.5), 0.8));
    // the hammer: head on top, handle down into the work
    z.fill_history(&round_rect(94.0, 60.0, 162.0, 92.0, 10.0), light(g.a, 0.2), g.a);
    z.fine(&round_rect(94.0, 60.0, 162.0, 74.0, 8.0), with_alpha(light(g.a, 0.4), 0.45));
    z.fill(&round_rect(119.0, 90.0, 137.0, 172.0, 7.0), g.c);
    z.fine(&round_rect(119.0, 158.0, 137.0, 172.0, 6.0), with_alpha(dark(g.c, 0.3), 0.7));
}

/// matrix-icons: the composer — staggered layers with grip dots.
pub fn glyph_icons(g: &Glyph, z: &mut Painter) {
    z.fill(&squircle(104.0, 104.0, 50.0, 4.6), with_alpha(g.b, 0.85));
    z.fill_history(&round_rect(92.0, 92.0, 200.0, 172.0, 18.0), light(g.a, 0.14), g.a);
    z.fill(&squircle(168.0, 166.0, 32.0, 4.6), g.c);
    for (x, y) in [(92.0, 92.0), (200.0, 92.0), (92.0, 172.0), (200.0, 172.0)] {
        z.fine(&circle(x, y, 4.0), light(g.a, 0.5));
        z.ring(x, y, 4.0, 1.5, dark(g.a, 0.3));
    }
}

/// matrix-keyboard: the keyboard itself — key grid, one pressed.
pub fn glyph_keyboard(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(48.0, 88.0, 208.0, 176.0, 16.0), light(g.b, 0.12), g.b);
    let key = |x: f32, y: f32, w: f32| round_rect(x, y, x + w, y + 16.0, 4.0);
    for (row, n) in [(0, 7), (1, 7)] {
        for i in 0..n {
            let x = 60.0 + i as f32 * 20.0 + row as f32 * 6.0;
            let y = 100.0 + row as f32 * 22.0;
            if row == 1 && i == 3 {
                // the PRESSED key: deeper, darker
                z.fine(&key(x, y + 2.0, 16.0), dark(g.c, 0.3));
            } else {
                z.fill(&key(x, y, 16.0), light(g.c, 0.2));
            }
        }
    }
    z.fill(&round_rect(84.0, 146.0, 172.0, 162.0, 4.0), light(g.c, 0.2));
    z.fill(&round_rect(178.0, 146.0, 196.0, 162.0, 4.0), g.a);
}

/// matrix-terminal: the screen in the case (lighthouse design).
pub fn glyph_terminal(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(46.0, 62.0, 210.0, 194.0, 15.0), light(g.b, 0.14), g.b);
    z.fill(&round_rect(55.0, 71.0, 201.0, 185.0, 10.0), dark(g.b, 0.52));
    z.fine(&round_rect(55.0, 71.0, 201.0, 81.0, 8.0), with_alpha(dark(g.b, 0.7), 0.6));
    z.fill(&triangle((69.0, 98.0), (69.0, 122.0), (84.0, 110.0)), g.a);
    z.fill(&round_rect(92.0, 104.0, 134.0, 112.0, 3.0), light(g.c, 0.3));
    z.fill(&round_rect(141.0, 101.0, 153.0, 115.0, 2.0), g.a);
    z.fine(&round_rect(69.0, 131.0, 159.0, 136.0, 2.5), with_alpha(light(g.b, 0.25), 0.55));
    z.fine(&round_rect(69.0, 143.0, 129.0, 148.0, 2.5), with_alpha(light(g.b, 0.25), 0.4));
    z.fine(&triangle((55.0, 71.0), (125.0, 71.0), (75.0, 185.0)), with_alpha(light(g.b, 0.6), 0.06));
}

/// matrix-capture: the camera — case, lens rings, shutter.
pub fn glyph_capture(g: &Glyph, z: &mut Painter) {
    z.fill(&round_rect(96.0, 74.0, 160.0, 94.0, 8.0), g.b);
    z.fill_history(&round_rect(52.0, 88.0, 204.0, 182.0, 18.0), light(g.b, 0.14), g.b);
    z.fill(&round_rect(168.0, 76.0, 192.0, 88.0, 5.0), g.a);
    z.fill_history(&circle(128.0, 134.0, 34.0), light(g.a, 0.18), g.a);
    z.ring(128.0, 134.0, 34.0, 4.0, dark(g.a, 0.28));
    z.fill(&circle(128.0, 134.0, 20.0), dark(g.b, 0.5));
    z.fine(&circle(121.0, 127.0, 6.0), with_alpha(light(g.a, 0.6), 0.8));
    z.fine(&circle(184.0, 106.0, 5.0), light(g.c, 0.3));
}

/// matrix-player: the stage — play in a ring, progress with a grip.
pub fn glyph_player(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(48.0, 70.0, 208.0, 186.0, 16.0), light(g.b, 0.12), g.b);
    z.ring(128.0, 118.0, 32.0, 5.0, with_alpha(light(g.b, 0.35), 0.6));
    z.fill(&triangle((118.0, 100.0), (118.0, 136.0), (148.0, 118.0)), g.a);
    z.fill(&round_rect(66.0, 160.0, 190.0, 168.0, 4.0), dark(g.b, 0.35));
    z.fine(&round_rect(66.0, 160.0, 132.0, 168.0, 4.0), g.a);
    z.fill(&circle(132.0, 164.0, 8.0), light(g.a, 0.25));
}

/// matrix-disks: the disk — stacked platters (cylinder from the
/// side) with an activity dot and one dividing line (partition).
pub fn glyph_disks(g: &Glyph, z: &mut Painter) {
    // Three stacked ellipse platters as the disk stack.
    for (i, y) in [176.0_f32, 148.0, 120.0].iter().enumerate() {
        let c = if i == 0 { g.a } else { light(g.b, 0.06 + i as f32 * 0.06) };
        // Platter = flat rounded rectangle (approximated ellipse).
        z.fill_history(&round_rect(56.0, *y - 22.0, 200.0, *y + 22.0, 22.0), light(c, 0.14), c);
        z.fine(&round_rect(56.0, *y - 22.0, 200.0, *y - 8.0, 14.0), with_alpha(light(c, 0.4), 0.35));
    }
    // Dividing groove (partition) on the top platter.
    z.fine(&round_rect(150.0, 98.0, 154.0, 142.0, 2.0), with_alpha(dark(g.b, 0.4), 0.5));
    // Activity glow dot.
    z.fill(&circle(170.0, 176.0, 7.0), light(g.c, 0.3));
    z.fine(&circle(170.0, 176.0, 3.0), g.c);
}

/// matrix-mandelbrot: the Mandelbrot set itself — cardioid + buds as a
/// silhouette over a fine axis cross, locator ring like a zoom target.
/// The most famous set of mathematics is its own icon.
pub fn glyph_mandelbrot(g: &Glyph, z: &mut Painter) {
    // Axis cross of the complex plane, deliberately quiet.
    z.fine(&round_rect(48.0, 127.0, 208.0, 129.0, 1.0), with_alpha(g.b, 0.25));
    z.fine(&round_rect(155.0, 60.0, 157.0, 196.0, 1.0), with_alpha(g.b, 0.25));
    // Cardioid: z = e^it/2 - e^2it/4, scaled to tile size.
    let (cx, cy, k) = (156.0_f32, 128.0_f32, 88.0_f32);
    let cardioid = {
        let mut pb = PathBuilder::new();
        for i in 0..=64 {
            let t = i as f32 / 64.0 * std::f32::consts::TAU;
            let x = 0.5 * t.cos() - 0.25 * (2.0 * t).cos();
            let y = 0.5 * t.sin() - 0.25 * (2.0 * t).sin();
            let (px, py) = (cx + x * k, cy - y * k);
            if i == 0 { pb.move_to(px, py) } else { pb.line_to(px, py) }
        }
        pb.close();
        pb.finish().expect("cardioid")
    };
    z.fill_history(&cardioid, light(g.a, 0.2), g.a);
    // Period-2 bud and the small siblings on the axis.
    z.fill(&circle(cx - 1.0 * k, cy, 0.25 * k, ), g.a);
    z.fill(&circle(cx - 1.31 * k, cy, 0.06 * k), light(g.a, 0.15));
    z.fill(&circle(cx - 0.125 * k, cy - 0.744 * k, 0.094 * k), light(g.a, 0.1));
    z.fill(&circle(cx - 0.125 * k, cy + 0.744 * k, 0.094 * k), light(g.a, 0.1));
    // Zoom target at the seahorse valley.
    z.ring(cx - 0.75 * k, cy - 0.1 * k, 14.0, 2.5, with_alpha(g.c, 0.9));
    z.fine(&circle(cx - 0.75 * k, cy - 0.1 * k, 3.0), g.c);
}

/// matrix-markdown: the document — card, M sign, arrow, dog-ear.
pub fn glyph_markdown(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(56.0, 62.0, 200.0, 194.0, 14.0), light(g.b, 0.16), g.b);
    z.fine(&triangle((200.0, 62.0), (200.0, 92.0), (172.0, 62.0)), dark(g.b, 0.18));
    z.fill(&round_rect(74.0, 88.0, 84.0, 138.0, 4.0), g.a);
    z.fill(&triangle((74.0, 88.0), (99.0, 122.0), (84.0, 122.0)), g.a);
    z.fill(&triangle((124.0, 88.0), (99.0, 122.0), (114.0, 122.0)), g.a);
    z.fill(&round_rect(114.0, 88.0, 124.0, 138.0, 4.0), g.a);
    z.fill(&round_rect(152.0, 88.0, 168.0, 120.0, 5.0), g.c);
    z.fill(&triangle((140.0, 116.0), (180.0, 116.0), (160.0, 140.0)), g.c);
    for (i, end) in [176.0_f32, 158.0].iter().enumerate() {
        z.fine(
            &round_rect(74.0, 156.0 + i as f32 * 14.0, *end, 161.0 + i as f32 * 14.0, 2.5),
            with_alpha(dark(g.b, 0.4), 0.55),
        );
    }
}

/// matrix-rabbit: the usher for the Hardline net — head with two long
/// ears. Deliberately front-on and calm: a HOST that receives, not a
/// mascot that leaps.
pub fn glyph_rabbit(g: &Glyph, z: &mut Painter) {
    // Ears: two rounded uprights, slightly splayed. Inner ear in the
    // accent role — the one warm surface in the icon.
    z.fill_history(&round_rect(88.0, 44.0, 118.0, 148.0, 15.0), light(g.b, 0.24), g.b);
    z.fill_history(&round_rect(138.0, 44.0, 168.0, 148.0, 15.0), light(g.b, 0.24), g.b);
    z.fine(&round_rect(96.0, 58.0, 110.0, 136.0, 7.0), with_alpha(light(g.a, 0.15), 0.85));
    z.fine(&round_rect(146.0, 58.0, 160.0, 136.0, 7.0), with_alpha(light(g.a, 0.15), 0.85));
    // Head over the ear roots.
    z.fill_history(&circle(128.0, 164.0, 52.0), light(g.b, 0.28), dark(g.b, 0.04));
    // Eyes + nose: minimal marks, family line width.
    z.fine(&circle(108.0, 156.0, 6.5), dark(g.b, 0.55));
    z.fine(&circle(148.0, 156.0, 6.5), dark(g.b, 0.55));
    z.fine(&triangle((128.0, 176.0), (120.0, 168.0), (136.0, 168.0)), light(g.a, 0.1));
    // Whisker hint left/right of the nose.
    for (x0, x1) in [(84.0_f32, 106.0_f32), (150.0, 172.0)] {
        z.fine(
            &round_rect(x0, 178.0, x1, 181.0, 1.5),
            with_alpha(dark(g.b, 0.4), 0.5),
        );
    }
}

/// matrix-hardline: the node — a ring (the net), three peers on it,
/// the device's own key in the middle: a bright dot with a short shaft.
pub fn glyph_hardline(g: &Glyph, z: &mut Painter) {
    z.ring(128.0, 128.0, 58.0, 9.0, with_alpha(dark(g.b, 0.25), 0.7));
    for (cx, cy) in [(128.0, 70.0), (78.0, 158.0), (178.0, 158.0)] {
        z.fill_history(&circle(cx, cy, 13.0), light(g.b, 0.16), g.b);
    }
    z.fill_history(&circle(128.0, 128.0, 22.0), light(g.a, 0.18), g.a);
    z.fill(&round_rect(124.0, 128.0, 132.0, 168.0, 3.0), g.a);
    z.fill(&round_rect(132.0, 150.0, 142.0, 156.0, 2.0), g.a);
    z.fine(&circle(122.0, 121.0, 5.0), with_alpha(light(g.a, 0.5), 0.7));
}

/// matrix-arcade: the four stones of the Arcade (red, yellow, green, blue
/// in the web icon) and the small white one in the middle — here in the
/// palette's two accents; the painted bitmap is the app's real face.
pub fn glyph_arcade(g: &Glyph, z: &mut Painter) {
    for (k, (cx, cy)) in [(98.0, 100.0), (158.0, 100.0), (98.0, 160.0), (158.0, 160.0)].into_iter().enumerate() {
        let c = if k % 3 == 0 { g.a } else { g.b };
        z.fill_history(&circle(cx, cy, 24.0), light(c, 0.16), c);
    }
    z.fill(&circle(128.0, 130.0, 14.0), light(g.a, 0.6));
}

// --- Round 58 (03.11.2026): the Duo's apps without a face of their own.
// The dock's «icons» form showed them as letters on the bare tile.

/// matrix-tube: a video card stack — the front card plays (a round play
/// badge), two cards peek out behind it: a library of films.
pub fn glyph_tube(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(76.0, 62.0, 180.0, 74.0, 6.0), light(g.b, 0.30), light(g.b, 0.18));
    z.fill_history(&round_rect(62.0, 72.0, 194.0, 86.0, 8.0), light(g.b, 0.16), g.b);
    z.fill_history(&round_rect(46.0, 84.0, 210.0, 190.0, 22.0), light(g.a, 0.14), g.a);
    z.fill(&circle(128.0, 137.0, 30.0), with_alpha(light(g.a, 0.92), 0.95));
    z.fill(&triangle((119.0, 122.0), (119.0, 152.0), (145.0, 137.0)), dark(g.a, 0.15));
    z.fine(&round_rect(64.0, 174.0, 192.0, 179.0, 2.5), with_alpha(dark(g.a, 0.45), 0.6));
    z.fine(&round_rect(64.0, 174.0, 116.0, 179.0, 2.5), light(g.c, 0.35));
}

/// matrix-start: all apps — a three-by-three field of small tiles, the
/// middle one lit (where you are), the colours of the family around it.
pub fn glyph_start(g: &Glyph, z: &mut Painter) {
    for row in 0..3 {
        for col in 0..3 {
            let (x, y) = (62.0 + col as f32 * 48.0, 62.0 + row as f32 * 48.0);
            let c = match (row + col) % 3 {
                0 => g.a,
                1 => g.b,
                _ => g.c,
            };
            let c = if row == 1 && col == 1 { light(c, 0.25) } else { c };
            z.fill_history(&round_rect(x, y, x + 36.0, y + 36.0, 10.0), light(c, 0.16), c);
        }
    }
}

/// matrix-camera: the body with its lens — a ring around glass, the
/// shutter bump on top and a small light.
pub fn glyph_camera(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(96.0, 70.0, 150.0, 96.0, 8.0), light(g.b, 0.18), g.b);
    z.fill_history(&round_rect(48.0, 86.0, 208.0, 192.0, 24.0), light(g.b, 0.12), dark(g.b, 0.08));
    z.fill(&circle(128.0, 140.0, 40.0), dark(g.b, 0.35));
    z.fill_history(&circle(128.0, 140.0, 32.0), light(g.a, 0.18), g.a);
    z.fill(&circle(128.0, 140.0, 16.0), dark(g.a, 0.45));
    z.fill(&circle(119.0, 131.0, 6.0), with_alpha(light(g.a, 0.9), 0.85));
    z.fill(&circle(184.0, 106.0, 7.0), light(g.c, 0.3));
}

/// matrix-shopping: the shopping list — a board with a clip, three rows
/// with their circles, the first ticked and struck (it is in the basket).
pub fn glyph_shopping(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(64.0, 52.0, 192.0, 220.0, 20.0), light(g.b, 0.18), g.b);
    z.fill_history(&round_rect(100.0, 40.0, 156.0, 66.0, 9.0), light(g.c, 0.2), g.c);
    for k in 0..3 {
        let y = 104.0 + k as f32 * 38.0;
        if k == 0 {
            z.fill(&circle(94.0, y, 11.0), light(g.a, 0.15));
            z.fill(&round_rect(114.0, y - 3.0, 168.0, y + 3.0, 3.0), with_alpha(light(g.b, 0.6), 0.55));
        } else {
            z.ring(94.0, y, 9.0, 4.0, light(g.a, 0.35));
            z.fill(&round_rect(114.0, y - 6.0, 170.0 - k as f32 * 10.0, y + 6.0, 6.0), with_alpha(light(g.b, 0.75), 0.9));
        }
    }
}

/// matrix-krabbler: the crawler — a trail of dots that grows along a
/// wave, the newest dot bright at its head.
pub fn glyph_krabbler(g: &Glyph, z: &mut Painter) {
    let n = 11;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let x = 56.0 + t * 144.0;
        let y = 140.0 - (t * std::f32::consts::PI * 1.6).sin() * 40.0;
        let r = 6.0 + t * 12.0;
        let c = if i == n - 1 { light(g.c, 0.2) } else { with_alpha(if i % 2 == 0 { g.a } else { g.b }, 0.45 + t * 0.55) };
        z.fill(&circle(x, y, r), c);
    }
}

/// matrix-oracle: the seeing glass — a sphere on its stand with a
/// glint and a faint swirl of what is to come.
pub fn glyph_oracle(g: &Glyph, z: &mut Painter) {
    z.fill_history(&round_rect(80.0, 172.0, 176.0, 196.0, 10.0), light(g.b, 0.12), dark(g.b, 0.1));
    z.fill_history(&round_rect(96.0, 160.0, 160.0, 178.0, 8.0), light(g.b, 0.2), g.b);
    z.fill_history(&circle(128.0, 112.0, 58.0), light(g.a, 0.22), dark(g.a, 0.12));
    z.ring(128.0, 112.0, 30.0, 6.0, with_alpha(light(g.c, 0.3), 0.55));
    z.fill(&circle(128.0, 112.0, 10.0), light(g.c, 0.4));
    z.fill(&circle(104.0, 88.0, 12.0), with_alpha(light(g.a, 0.95), 0.7));
}
