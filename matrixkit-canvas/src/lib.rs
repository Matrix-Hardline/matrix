//! The vector machine (iced replacement, stage 4, second piece —
//! GRABUNG-ICED-ABLOESUNG.md, camp 16).
//!
//! iced's canvas widget rasterises its paths in the software renderer with
//! tiny-skia — but tiny-skia is NOT iced: a standalone
//! crate, long since in the vendor tree. So we use it directly, like
//! cosmic-text for the font: the intermediary layer falls, the
//! foundation stays deliberately chosen.
//!
//! The contract in kit language:
//! * `Trace` draws the path (move/line/quad/cubic/close) — pure
//!   geometry, like a scene node before layout.
//! * `Canvas::fill()`/`stroke()` rasterise into their own sheet,
//!   transparent on top (premultiplied, anti-aliased).
//! * `blit()` blends the sheet into the XRGB buffer — with a
//!   clip box, like the text machine and the image atlas.
//!
//! Measuring, laying out, painting stay separate: whoever draws does not know the
//! buffer; whoever blends does not know the paths.

use matrixkit_theme::Rgba;

/// A path in the making — the thin wrapper over tiny-skia's builder.
#[derive(Default)]
pub struct Trace {
    b: tiny_skia::PathBuilder,
}

impl Trace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.b.move_to(x, y);
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.b.line_to(x, y);
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.b.quad_to(cx, cy, x, y);
    }

    pub fn cubic_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.b.cubic_to(c1x, c1y, c2x, c2y, x, y);
    }

    pub fn close(&mut self) {
        self.b.close();
    }

    /// A circle as a closed subpath.
    pub fn circle(&mut self, cx: f32, cy: f32, r: f32) {
        self.b.push_circle(cx, cy, r);
    }

    /// A circular arc around (cx, cy): angles in radians, clockwise
    /// for positive `sweep` (screen y points down). With
    /// `connect` a line is drawn from the pen, otherwise it starts anew.
    /// Cubic pieces of at most 90° — the classic approximation.
    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, start: f32, sweep: f32, connect: bool) {
        let (sx, sy) = (cx + r * start.cos(), cy + r * start.sin());
        if connect {
            self.b.line_to(sx, sy);
        } else {
            self.b.move_to(sx, sy);
        }
        let n = ((sweep.abs() / std::f32::consts::FRAC_PI_2).ceil() as usize).max(1);
        let step = sweep / n as f32;
        let k = 4.0 / 3.0 * (step / 4.0).tan();
        let mut a0 = start;
        for _ in 0..n {
            let a1 = a0 + step;
            let (c0, s0) = (a0.cos(), a0.sin());
            let (c1, s1) = (a1.cos(), a1.sin());
            self.b.cubic_to(
                cx + r * (c0 - k * s0),
                cy + r * (s0 + k * c0),
                cx + r * (c1 + k * s1),
                cy + r * (s1 - k * c1),
                cx + r * c1,
                cy + r * s1,
            );
            a0 = a1;
        }
    }

    fn finish(self) -> Option<tiny_skia::Path> {
        self.b.finish()
    }
}

fn color(c: Rgba) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(
        c.r.clamp(0.0, 1.0),
        c.g.clamp(0.0, 1.0),
        c.b.clamp(0.0, 1.0),
        c.a.clamp(0.0, 1.0),
    )
    .unwrap_or(tiny_skia::Color::TRANSPARENT)
}

/// The drawing sheet: its own pixels, transparent on top — only blit()
/// brings it over the ground.
pub struct Canvas {
    pixmap: tiny_skia::Pixmap,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Option<Self> {
        Some(Self { pixmap: tiny_skia::Pixmap::new(w, h)? })
    }

    pub fn clear(&mut self) {
        self.pixmap.fill(tiny_skia::Color::TRANSPARENT);
    }

    /// The sheet itself — for trades that rasterise directly into it
    /// (camp 62: resvg paints into a pixmap, not via Trace).
    pub fn pixmap_mut(&mut self) -> tiny_skia::PixmapMut<'_> {
        self.pixmap.as_mut()
    }

    /// The sheet as PNG bytes — the way into the image atlas.
    ///
    /// Via PNG instead of raw bytes, because tiny-skia stores PREMULTIPLIED
    /// and `ImageAtlas::unpack` expects straight channels;
    /// the PNG detour does the conversion instead of guessing it
    /// (needed for the SVG icons of foreign apps, 16.08.2026).
    pub fn png(&self) -> Option<Vec<u8>> {
        self.pixmap.encode_png().ok()
    }

    /// Fill the path (anti-aliased, winding rule).
    pub fn fill(&mut self, trace: Trace, tint: Rgba) {
        let Some(path) = trace.finish() else { return };
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(color(tint));
        paint.anti_alias = true;
        self.pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            None,
        );
    }

    /// Stroke the path (round caps and joins — the kit look).
    pub fn stroke(&mut self, trace: Trace, tint: Rgba, width: f32) {
        let Some(path) = trace.finish() else { return };
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(color(tint));
        paint.anti_alias = true;
        let stroke = tiny_skia::Stroke {
            width,
            line_cap: tiny_skia::LineCap::Round,
            line_join: tiny_skia::LineJoin::Round,
            ..Default::default()
        };
        self.pixmap.stroke_path(
            &path,
            &paint,
            &stroke,
            tiny_skia::Transform::identity(),
            None,
        );
    }

    /// Stroke the path DASHED: `dash` = alternating on/off lengths,
    /// `phase` shifts the pattern (running dots along an edge).
    pub fn stroke_dashed(&mut self, trace: Trace, tint: Rgba, width: f32, dash: &[f32], phase: f32) {
        let Some(path) = trace.finish() else { return };
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(color(tint));
        paint.anti_alias = true;
        let mut pattern: Vec<f32> = dash.iter().map(|d| d.max(0.01)).collect();
        if pattern.len() % 2 == 1 {
            pattern.extend_from_slice(&dash.iter().map(|d| d.max(0.01)).collect::<Vec<_>>());
        }
        let stroke = tiny_skia::Stroke {
            width,
            line_cap: tiny_skia::LineCap::Round,
            line_join: tiny_skia::LineJoin::Round,
            dash: tiny_skia::StrokeDash::new(pattern, phase),
            ..Default::default()
        };
        self.pixmap.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
    }

    /// The sheet as STRAIGHT RGBA rows (not premultiplied) — the way
    /// the image atlas expects placed images (camp 106).
    pub fn rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pixmap.data().len());
        for px in self.pixmap.pixels() {
            let c = px.demultiply();
            out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
        }
        out
    }

    pub fn size(&self) -> (u32, u32) {
        (self.pixmap.width(), self.pixmap.height())
    }

    /// Blend the sheet at (x, y) into an XRGB8888 buffer (B G R X)
    /// — the source is premultiplied, the clip box
    /// cuts as everywhere in the kit.
    #[allow(clippy::too_many_arguments)]
    pub fn blit(
        &self,
        x: f32,
        y: f32,
        clip: Option<matrixkit_theme::scene::Rect>,
        buf: &mut [u8],
        bw: u32,
        bh: u32,
    ) {
        let (cx0, cy0, cx1, cy1) = match clip {
            Some(c) => (
                c.x.max(0.0) as i32,
                c.y.max(0.0) as i32,
                ((c.x + c.w).min(bw as f32)) as i32,
                ((c.y + c.h).min(bh as f32)) as i32,
            ),
            None => (0, 0, bw as i32, bh as i32),
        };
        let data = self.pixmap.data();
        let (w, h) = (self.pixmap.width(), self.pixmap.height());
        for sy in 0..h {
            let py = y as i32 + sy as i32;
            if py < cy0 || py >= cy1 {
                continue;
            }
            for sx in 0..w {
                let px = x as i32 + sx as i32;
                if px < cx0 || px >= cx1 {
                    continue;
                }
                let s = ((sy * w + sx) * 4) as usize;
                let a = data[s + 3] as f32 / 255.0;
                if a <= 0.0 {
                    continue;
                }
                // Source premultiplied: only attenuate the ground,
                // add the colour.
                let d = ((py as u32 * bw + px as u32) * 4) as usize;
                buf[d] = (data[s + 2] as f32 + buf[d] as f32 * (1.0 - a)) as u8;
                buf[d + 1] = (data[s + 1] as f32 + buf[d + 1] as f32 * (1.0 - a)) as u8;
                buf[d + 2] = (data[s] as f32 + buf[d + 2] as f32 * (1.0 - a)) as u8;
                buf[d + 3] = (data[s + 3] as f32 + buf[d + 3] as f32 * (1.0 - a)) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgba = Rgba { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };

    fn px(buf: &[u8], bw: u32, x: u32, y: u32) -> &[u8] {
        &buf[((y * bw + x) * 4) as usize..((y * bw + x) * 4 + 4) as usize]
    }

    #[test]
    fn a_filled_path_lands_inside_and_nowhere_else() {
        let mut cv = Canvas::new(20, 20).unwrap();
        let mut t = Trace::new();
        t.move_to(4.0, 4.0);
        t.line_to(16.0, 4.0);
        t.line_to(16.0, 16.0);
        t.line_to(4.0, 16.0);
        t.close();
        cv.fill(t, RED);
        let mut buf = vec![0u8; 20 * 20 * 4];
        cv.blit(0.0, 0.0, None, &mut buf, 20, 20);
        assert!(px(&buf, 20, 10, 10)[2] > 200, "innen liegt das Rot");
        assert!(px(&buf, 20, 1, 1).iter().all(|&b| b == 0), "außen bleibt der Grund");
    }

    #[test]
    fn a_stroke_follows_the_line() {
        let mut cv = Canvas::new(20, 20).unwrap();
        let mut t = Trace::new();
        t.move_to(2.0, 10.0);
        t.line_to(18.0, 10.0);
        cv.stroke(t, RED, 2.0);
        let mut buf = vec![0u8; 20 * 20 * 4];
        cv.blit(0.0, 0.0, None, &mut buf, 20, 20);
        assert!(px(&buf, 20, 10, 10)[2] > 100, "auf der Linie liegt Farbe");
        assert!(px(&buf, 20, 10, 3).iter().all(|&b| b == 0), "abseits bleibt Stille");
    }

    #[test]
    fn the_blit_respects_the_clip_and_the_ground() {
        let mut cv = Canvas::new(8, 8).unwrap();
        let mut t = Trace::new();
        t.move_to(0.0, 0.0);
        t.line_to(8.0, 0.0);
        t.line_to(8.0, 8.0);
        t.line_to(0.0, 8.0);
        t.close();
        cv.fill(t, Rgba { a: 0.5, ..RED });
        // Ground: blue (B at +0), clip: only the upper half.
        let (bw, bh) = (8u32, 8u32);
        let mut buf = Vec::new();
        for _ in 0..(bw * bh) {
            buf.extend_from_slice(&[255, 0, 0, 0xff]);
        }
        let clip = matrixkit_theme::scene::Rect { x: 0.0, y: 0.0, w: 8.0, h: 4.0 };
        cv.blit(0.0, 0.0, Some(clip), &mut buf, bw, bh);
        // Top: half red over blue (both visible) …
        let top = px(&buf, bw, 4, 2);
        assert!(top[2] > 80 && top[0] > 80, "halbes Rot MISCHT sich über den Grund");
        // … below the edge: pure blue.
        let low = px(&buf, bw, 4, 6);
        assert_eq!((low[0], low[2]), (255, 0), "unter der Klip-Kante bleibt der Grund");
    }
}

/// Camp 62: SVG. iced's `svg` widget rasterises with resvg — and resvg
/// is NOT iced but a standalone crate, long since in the
/// vendor tree. So used directly, like cosmic-text for the font
/// and tiny-skia for the path. With this the vector machine gets its
/// last missing input: files instead of program paths.
pub struct Svg {
    tree: usvg::Tree,
}

impl Svg {
    /// Reads an SVG source. Broken SVG gives None — the caller
    /// decides whether to show a placeholder (empty state).
    pub fn parse(data: &[u8]) -> Option<Self> {
        let options = usvg::Options::default();
        usvg::Tree::from_data(data, &options).ok().map(|tree| Svg { tree })
    }

    /// The drawing's intrinsic size — what the layout needs to know.
    pub fn size(&self) -> (f32, f32) {
        let s = self.tree.size();
        (s.width(), s.height())
    }

    /// Rasterises into its own sheet, scaled to the TARGET SIZE
    /// (contain: the drawing keeps its aspect ratio and
    /// sits centred). `blit` blends the result like any other
    /// curve — drawing and blending still do not know each other.
    pub fn raster(&self, w: u32, h: u32) -> Option<Canvas> {
        if w == 0 || h == 0 {
            return None;
        }
        let mut canvas = Canvas::new(w, h)?;
        canvas.clear();
        let (sw, sh) = self.size();
        if sw <= 0.0 || sh <= 0.0 {
            return Some(canvas);
        }
        let scale = (w as f32 / sw).min(h as f32 / sh);
        let dx = (w as f32 - sw * scale) / 2.0;
        let dy = (h as f32 - sh * scale) / 2.0;
        let transform = tiny_skia::Transform::from_translate(dx, dy).pre_scale(scale, scale);
        resvg::render(&self.tree, transform, &mut canvas.pixmap_mut());
        Some(canvas)
    }
}

#[cfg(test)]
mod svg_tests {
    use super::*;

    // Caution: inside a br#"…"# the sequence "# ends the string —
    // hence a colour name instead of a hex value.
    const CIRCLE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10">
        <rect x="0" y="0" width="20" height="10" fill="red"/></svg>"#;

    #[test]
    fn a_drawing_knows_its_own_measure_and_keeps_its_ratio() {
        let svg = Svg::parse(CIRCLE).expect("gueltiges SVG");
        let (w, h) = svg.size();
        assert!((w - 20.0).abs() < 0.01 && (h - 10.0).abs() < 0.01);
        // Contain in ein QUADRAT: die Zeichnung fuellt die Breite,
        // sitzt senkrecht mittig — oben und unten bleibt Luft.
        let canvas = svg.raster(40, 40).expect("Blatt");
        let mut buf = vec![0u8; 40 * 40 * 4];
        canvas.blit(0.0, 0.0, None, &mut buf, 40, 40);
        let opaque_row = |y: usize| (0..40).filter(|x| buf[(y * 40 + x) * 4 + 3] > 0).count();
        assert!(opaque_row(20) > 30, "in der Mitte malt sie");
        assert_eq!(opaque_row(1), 0, "oben bleibt sie leer");
        assert_eq!(opaque_row(38), 0, "unten auch");
    }

    #[test]
    fn broken_svg_says_no_instead_of_panicking() {
        assert!(Svg::parse(b"das ist kein svg").is_none());
        let svg = Svg::parse(CIRCLE).unwrap();
        assert!(svg.raster(0, 10).is_none(), "ein Nullmass ist kein Blatt");
    }
}

#[cfg(test)]
mod shape_tests {
    use super::*;

    #[test]
    fn a_dashed_line_has_gaps_and_a_circle_a_solid_heart() {
        let mut cv = Canvas::new(60, 20).unwrap();
        let mut t = Trace::new();
        t.move_to(0.0, 10.0);
        t.line_to(60.0, 10.0);
        let ink = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        cv.stroke_dashed(t, ink, 2.0, &[6.0, 6.0], 0.0);
        let rgba = cv.rgba();
        let alpha = |x: usize| rgba[(10 * 60 + x) * 4 + 3];
        assert!(alpha(2) > 200, "die erste Strecke ist Tinte");
        assert!(alpha(9) < 40, "danach eine Lücke: {}", alpha(9));
        assert!(alpha(15) > 200, "dann wieder Tinte");

        let mut cv = Canvas::new(40, 40).unwrap();
        let mut t = Trace::new();
        t.circle(20.0, 20.0, 15.0);
        cv.fill(t, ink);
        let rgba = cv.rgba();
        assert_eq!(rgba[(20 * 40 + 20) * 4 + 3], 255, "die Mitte ist voll");
        assert_eq!(rgba[3], 0, "die Ecke bleibt frei");

        let mut cv = Canvas::new(40, 40).unwrap();
        let mut t = Trace::new();
        t.arc(20.0, 20.0, 15.0, 0.0, std::f32::consts::PI, false);
        cv.stroke(t, ink, 3.0);
        let rgba = cv.rgba();
        assert!(rgba[(35 * 40 + 20) * 4 + 3] > 200, "der Halbbogen läuft unten durch");
        assert!(rgba[(5 * 40 + 20) * 4 + 3] < 40, "oben ist kein Bogen");
    }
}
