//! The kit's text machine (iced replacement, stage 3 —
//! GRABUNG-ICED-ABLOESUNG.md).
//!
//! cosmic-text first-hand: the same foundation that also lies under
//! iced, only without a middleman. Born as stage 2 in the first
//! light (apps/wallpaper/src/licht.rs), moved here with stage 5
//! — the probe became a kit building block.
//!
//! The contract is deliberately split in two, so that mk::scene can stay pure
//! geometry:
//!
//! * `measure()` BEFORE layout — gives the layout machine the
//!   size of a label (it computes with rectangles, not with
//!   glyphs).
//! * `paint()` AFTER layout — blends the glyph coverage at the
//!   laid-out spot into an XRGB8888 buffer (little-endian, as
//!   wl_shm carries it).
//!
//! Measuring and painting do not know each other; whoever measures lays out, and whoever
//! has laid out paints. FontSystem and SwashCache are expensive — ONE
//! engine per client, not one per frame.

/// The surface's house font. Apps do not override it — whoever
/// needs mono (terminal, codes) later gets a contract of their own,
/// not a parameter to tweak.
pub const FAMILY: &str = "Inter Variable";

/// Line height as a factor of the font size — the same value with which
/// the first light already measured.
pub const LINE_HEIGHT: f32 = 1.3;

pub struct TextEngine {
    fonts: cosmic_text::FontSystem,
    swash: cosmic_text::SwashCache,
}

/// Round 178 (memory audit): cosmic-text's glyph image cache never evicts — one entry per glyph × size × subpixel
/// offset, for the whole life of a daemon (every notification ever shown, every frame of an animated header size).
/// Above this many glyphs it starts afresh; a few thousand cover any screen, refilling costs microseconds each.
const GLYPH_CACHE_MAX: usize = 6000;

impl Default for TextEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// mk::typo::Weight → cosmic-text. The kit's three weights are enough;
/// whoever needs more has not understood the typo ladder.
fn weight(w: matrixkit_theme::typo::Weight) -> cosmic_text::Weight {
    match w {
        matrixkit_theme::typo::Weight::Normal => cosmic_text::Weight::NORMAL,
        matrixkit_theme::typo::Weight::Medium => cosmic_text::Weight::MEDIUM,
        matrixkit_theme::typo::Weight::Semibold => cosmic_text::Weight::SEMIBOLD,
    }
}

/// Stage 61: a RUN — a piece of text with its own style. Several
/// runs in ONE paragraph are iced's `rich_text`: bold in the middle of a
/// sentence, a piece of code in mono, a coloured word. Wrapping
/// has to work across the run boundaries — that is why
/// this is a capability of the TEXT MACHINE, not a family: only whoever
/// shapes can break across boundaries.
#[derive(Clone, Copy)]
pub struct Run<'a> {
    pub text: &'a str,
    pub px: f32,
    pub weight: matrixkit_theme::typo::Weight,
    pub family: &'a str,
    pub color: matrixkit_theme::Rgba,
}


/// The font folder of an AppImage (`$APPDIR/usr/share/fonts`) or
/// next to the binary (`../share/fonts`) — if there is one.
pub fn bundled_fonts_dir() -> Option<std::path::PathBuf> {
    // On the canvas the fonts are in the image — the bundle
    // is for the host land (R207: the habitat decides).
    if matrixkit_theme::habitat::is_matrix() {
        return None;
    }
    if let Ok(appdir) = std::env::var("APPDIR") {
        let d = std::path::PathBuf::from(appdir).join("usr/share/fonts");
        if d.is_dir() {
            return Some(d);
        }
    }
    let exe = std::env::current_exe().ok()?;
    let d = exe.parent()?.parent()?.join("share/fonts");
    d.is_dir().then_some(d)
}

impl TextEngine {
    /// Keep the CPU glyph cache bounded (see GLYPH_CACHE_MAX).
    fn trim_glyphs(&mut self) {
        if self.swash.image_cache.len() > GLYPH_CACHE_MAX {
            self.swash.image_cache.clear();
            self.swash.outline_command_cache.clear();
        }
    }

    /// How many glyph images the CPU path holds (diagnostics, tests).
    pub fn glyph_cache_len(&self) -> usize {
        self.swash.image_cache.len()
    }

    pub fn new() -> Self {
        let mut fonts = cosmic_text::FontSystem::new();
        // R207: fonts an AppImage brings along — on a
        // foreign desktop Inter and the symbol font are missing otherwise.
        if let Some(dir) = bundled_fonts_dir() {
            fonts.db_mut().load_fonts_dir(&dir);
        }
        Self { fonts, swash: cosmic_text::SwashCache::new() }
    }

    /// Load a font from bytes (stage 9: the toolbars' symbol font)
    /// — afterwards it can be addressed by its
    /// family name (measure_family/paint_family).
    /// Empty bytes are silently discarded: no font is not an
    /// error, symbols then stay empty boxes.
    pub fn load_font(&mut self, data: Vec<u8>) {
        if !data.is_empty() {
            self.fonts.db_mut().load_font_data(data);
        }
    }

    fn rich(&mut self, runs: &[Run], wrap: Option<f32>) -> cosmic_text::Buffer {
        use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping};
        // The metrics follow the LARGEST run: a line
        // has ONE height, otherwise the baselines dance.
        let px = runs.iter().map(|r| r.px).fold(0.0f32, f32::max).max(1.0);
        let mut buf = Buffer::new(&mut self.fonts, Metrics::new(px, px * LINE_HEIGHT));
        buf.set_size(&mut self.fonts, wrap, None);
        let spans: Vec<(&str, Attrs)> = runs
            .iter()
            .map(|r| {
                (
                    r.text,
                    Attrs::new()
                        .family(Family::Name(r.family))
                        .weight(weight(r.weight))
                        .metrics(Metrics::new(r.px, px * LINE_HEIGHT))
                        .color(cosmic_text::Color::rgba(
                            (r.color.r * 255.0) as u8,
                            (r.color.g * 255.0) as u8,
                            (r.color.b * 255.0) as u8,
                            (r.color.a * 255.0) as u8,
                        )),
                )
            })
            .collect();
        buf.set_rich_text(
            &mut self.fonts,
            spans,
            &Attrs::new().family(Family::Name(FAMILY)),
            Shaping::Advanced,
            None,
        );
        buf.shape_until_scroll(&mut self.fonts, false);
        buf
    }

    /// The size of several runs in one paragraph — with an optional
    /// column width across which it wraps.
    pub fn measure_runs(&mut self, runs: &[Run], wrap: Option<f32>) -> (f32, f32) {
        let px = runs.iter().map(|r| r.px).fold(0.0f32, f32::max).max(1.0);
        let buf = self.rich(runs, wrap);
        let mut width = 0.0f32;
        let mut lines = 0;
        for run in buf.layout_runs() {
            width = width.max(run.line_w);
            lines += 1;
        }
        (width, lines as f32 * px * LINE_HEIGHT)
    }

    /// Paints the runs — EACH in ITS colour: the colour lives in the
    /// run, not in the call. That is the difference to paint_*,
    /// where one ink colours the whole text.
    #[allow(clippy::too_many_arguments)]
    pub fn paint_runs(
        &mut self,
        runs: &[Run],
        wrap: Option<f32>,
        x: f32,
        y: f32,
        clip: Option<matrixkit_theme::scene::Rect>,
        buf_out: &mut [u8],
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
        let fallback = cosmic_text::Color::rgba(255, 255, 255, 255);
        let tb = self.rich(runs, wrap);
        self.trim_glyphs();
        tb.draw(&mut self.fonts, &mut self.swash, fallback, |gx, gy, gw, gh, col| {
            for py in gy..gy + gh as i32 {
                for px_ in gx..gx + gw as i32 {
                    let sx = x as i32 + px_;
                    let sy = y as i32 + py;
                    if sx < cx0 || sy < cy0 || sx >= cx1 || sy >= cy1 {
                        continue;
                    }
                    let a = col.a() as f32 / 255.0;
                    if a <= 0.0 {
                        continue;
                    }
                    let idx = (sy as usize * bw as usize + sx as usize) * 4;
                    if idx + 3 >= buf_out.len() {
                        continue;
                    }
                    for (k, ch) in [col.b(), col.g(), col.r()].into_iter().enumerate() {
                        let dst = buf_out[idx + k] as f32;
                        buf_out[idx + k] = (ch as f32 * a + dst * (1.0 - a)) as u8;
                    }
                }
            }
        });
    }

    fn buffer(
        &mut self,
        family: &str,
        text: &str,
        px: f32,
        w: matrixkit_theme::typo::Weight,
    ) -> cosmic_text::Buffer {
        use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping};
        // The language (06.11.2026): measured, wrapped and painted alike.
        let text = matrixkit_theme::language::shown(text);
        // Round 178: a size of 0 (or NaN from a layout division) trips an assert inside cosmic-text.
        let px = if px.is_finite() && px >= 1.0 { px } else { 1.0 };
        let mut buf = Buffer::new(&mut self.fonts, Metrics::new(px, px * LINE_HEIGHT));
        buf.set_text(
            &mut self.fonts,
            &text,
            &Attrs::new().family(Family::Name(family)).weight(weight(w)),
            Shaping::Advanced,
            None,
        );
        buf.shape_until_scroll(&mut self.fonts, false);
        buf
    }

    /// Width/height of a label — food for the layout.
    /// Shorthand for normal weight; the kit's typo roles go
    /// through measure_styled (stage 7: the tiles use semibold).
    pub fn measure(&mut self, text: &str, px: f32) -> (f32, f32) {
        self.measure_styled(text, px, matrixkit_theme::typo::Weight::Normal)
    }

    /// Like measure, with font weight — the contract grows.
    pub fn measure_styled(
        &mut self,
        text: &str,
        px: f32,
        w: matrixkit_theme::typo::Weight,
    ) -> (f32, f32) {
        self.measure_family(FAMILY, text, px, w)
    }

    /// Like measure_styled, with font family — for everything that is not
    /// the house font (symbol font, later mono).
    pub fn measure_family(
        &mut self,
        family: &str,
        text: &str,
        px: f32,
        w: matrixkit_theme::typo::Weight,
    ) -> (f32, f32) {
        let buf = self.buffer(family, text, px, w);
        let mut width = 0.0f32;
        let mut lines = 0;
        for run in buf.layout_runs() {
            width = width.max(run.line_w);
            lines += 1;
        }
        (width, lines as f32 * px * LINE_HEIGHT)
    }

    /// The LINE WRAP (stage 27): wraps running text to
    /// a maximum width — word by word, by measuring (whoever measures
    /// wraps; the layout machine only sees the finished
    /// lines as '\n' text, which measure/paint have long handled
    /// as multi-line). A single word wider than the column
    /// is split hard — no overflow, ever.
    pub fn wrap_styled(
        &mut self,
        text: &str,
        px: f32,
        w: matrixkit_theme::typo::Weight,
        max_w: f32,
    ) -> String {
        self.wrap_family(FAMILY, text, px, w, max_w)
    }

    /// Like wrap_styled, with font family.
    pub fn wrap_family(
        &mut self,
        family: &str,
        text: &str,
        px: f32,
        w: matrixkit_theme::typo::Weight,
        max_w: f32,
    ) -> String {
        // Wrap what the screen shows (the language), not the German key.
        let text = matrixkit_theme::language::shown(text);
        let mut out = String::new();
        for (i, para) in text.split('\n').enumerate() {
            if i > 0 {
                out.push('\n');
            }
            let mut line = String::new();
            for word in para.split_whitespace() {
                // Split oversized single words first, then lay out.
                for piece in self.word_pieces(family, word, px, w, max_w) {
                    let probe = if line.is_empty() {
                        piece.clone()
                    } else {
                        format!("{line} {piece}")
                    };
                    let (pw, _) = self.measure_family(family, &probe, px, w);
                    if pw <= max_w || line.is_empty() {
                        line = probe;
                    } else {
                        out.push_str(&line);
                        out.push('\n');
                        line = piece;
                    }
                }
            }
            out.push_str(&line);
        }
        out
    }

    /// A word in pieces that each fit into the column —
    /// usually one (the word itself).
    fn word_pieces(
        &mut self,
        family: &str,
        word: &str,
        px: f32,
        w: matrixkit_theme::typo::Weight,
        max_w: f32,
    ) -> Vec<String> {
        let (ww, _) = self.measure_family(family, word, px, w);
        if ww <= max_w {
            return vec![word.to_string()];
        }
        let mut pieces = Vec::new();
        let mut piece = String::new();
        for c in word.chars() {
            let mut probe = piece.clone();
            probe.push(c);
            let (pw, _) = self.measure_family(family, &probe, px, w);
            if pw <= max_w || piece.is_empty() {
                piece = probe;
            } else {
                pieces.push(piece);
                piece = c.to_string();
            }
        }
        if !piece.is_empty() {
            pieces.push(piece);
        }
        pieces
    }

    /// Blend the label at (x, y) into the XRGB buffer. The
    /// buffer is `bw × bh` pixels, 4 bytes per pixel (B G R X);
    /// blended over the background, never replaced. Shorthand for
    /// normal weight — typo roles go through paint_styled.
    #[allow(clippy::too_many_arguments)]
    /// The GPU seam (stage 3, GRABUNG-GPU-MALEREI.md): cryoglyph
    /// needs FontSystem and SwashCache mutable AT THE SAME TIME —
    /// one grip, both hands. Only the GPU painting reaches in here.
    pub fn gpu_handles(
        &mut self,
    ) -> (&mut cosmic_text::FontSystem, &mut cosmic_text::SwashCache) {
        (&mut self.fonts, &mut self.swash)
    }

    pub fn paint(
        &mut self,
        text: &str,
        px: f32,
        color: matrixkit_theme::Rgba,
        x: f32,
        y: f32,
        buf: &mut [u8],
        bw: u32,
        bh: u32,
    ) {
        self.paint_styled(text, px, matrixkit_theme::typo::Weight::Normal, color, x, y, buf, bw, bh)
    }

    /// Like paint, with font weight — the contract grows.
    #[allow(clippy::too_many_arguments)]
    pub fn paint_styled(
        &mut self,
        text: &str,
        px: f32,
        w: matrixkit_theme::typo::Weight,
        color: matrixkit_theme::Rgba,
        x: f32,
        y: f32,
        buf: &mut [u8],
        bw: u32,
        bh: u32,
    ) {
        self.paint_family(FAMILY, text, px, w, color, x, y, buf, bw, bh)
    }

    /// Like paint_styled, with font family.
    #[allow(clippy::too_many_arguments)]
    pub fn paint_family(
        &mut self,
        family: &str,
        text: &str,
        px: f32,
        w: matrixkit_theme::typo::Weight,
        color: matrixkit_theme::Rgba,
        x: f32,
        y: f32,
        buf: &mut [u8],
        bw: u32,
        bh: u32,
    ) {
        self.paint_clip(family, text, px, w, color, x, y, None, buf, bw, bh)
    }

    /// Like paint_family, with a VIEW BOX (stage 14: the scroll): no
    /// glyph pixel leaves the box — whoever scrolls paints the
    /// content offset and clips here, instead of shaping anew.
    #[allow(clippy::too_many_arguments)]
    pub fn paint_clip(
        &mut self,
        family: &str,
        text: &str,
        px: f32,
        w: matrixkit_theme::typo::Weight,
        color: matrixkit_theme::Rgba,
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
        let tb = self.buffer(family, text, px, w);
        let base = cosmic_text::Color::rgba(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        );
        self.trim_glyphs();
        tb.draw(&mut self.fonts, &mut self.swash, base, |gx, gy, gw, gh, c| {
            let a = c.a() as f32 / 255.0;
            if a <= 0.0 {
                return;
            }
            for dy in 0..gh {
                for dx in 0..gw {
                    let px_ = x as i32 + gx + dx as i32;
                    let py_ = y as i32 + gy + dy as i32;
                    if px_ < cx0 || py_ < cy0 || px_ >= cx1 || py_ >= cy1 {
                        continue;
                    }
                    let i = ((py_ as u32 * bw + px_ as u32) * 4) as usize;
                    buf[i] = (c.b() as f32 * a + buf[i] as f32 * (1.0 - a)) as u8;
                    buf[i + 1] = (c.g() as f32 * a + buf[i + 1] as f32 * (1.0 - a)) as u8;
                    buf[i + 2] = (c.r() as f32 * a + buf[i + 2] as f32 * (1.0 - a)) as u8;
                    buf[i + 3] = (255.0 * a + buf[i + 3] as f32 * (1.0 - a)) as u8;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_grows_with_content_and_size() {
        let mut e = TextEngine::new();
        let (w1, h1) = e.measure("Matrix", 20.0);
        assert!(w1 > 0.0 && h1 > 0.0, "ein Wort hat Masse");
        let (w2, _) = e.measure("Matrix Matrix Matrix", 20.0);
        assert!(w2 > w1, "mehr Text ist breiter");
        let (w3, h3) = e.measure("Matrix", 40.0);
        assert!(w3 > w1 && h3 > h1, "größere Schrift ist größer");
    }

    #[test]
    fn weight_is_part_of_the_measure() {
        // Stage 7: the tiles use semibold (SUBTITLE). A
        // weight that the measurement does not reach would give layouts
        // that do not fit what is painted.
        let mut e = TextEngine::new();
        let (wn, hn) = e.measure_styled("Einstellungen", 16.0, matrixkit_theme::typo::Weight::Normal);
        let (ws, hs) = e.measure_styled("Einstellungen", 16.0, matrixkit_theme::typo::Weight::Semibold);
        assert!(wn > 0.0 && ws > 0.0);
        assert!(ws >= wn, "Semibold ist nie schmaler als Normal");
        assert_eq!(hn, hs, "das Gewicht ändert die Zeilenhöhe nicht");
    }

    #[test]
    fn the_family_parameter_is_the_same_road() {
        // The shorthands are only the house-font abbreviation — no
        // second measuring path that could drift.
        let mut e = TextEngine::new();
        assert_eq!(
            e.measure_styled("Matrix", 20.0, matrixkit_theme::typo::Weight::Normal),
            e.measure_family(FAMILY, "Matrix", 20.0, matrixkit_theme::typo::Weight::Normal)
        );
    }

    #[test]
    fn the_clip_keeps_every_glyph_inside() {
        // Stage 14: the view box. Below the edge every
        // byte stays quiet — scrolling means offsetting AND clipping.
        let mut e = TextEngine::new();
        let (bw, bh) = (200u32, 60u32);
        let mut buf = vec![0u8; (bw * bh * 4) as usize];
        let white = matrixkit_theme::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let clip = matrixkit_theme::scene::Rect { x: 0.0, y: 0.0, w: 200.0, h: 20.0 };
        e.paint_clip(FAMILY, "Matrix", 30.0, matrixkit_theme::typo::Weight::Normal,
            white, 8.0, 2.0, Some(clip), &mut buf, bw, bh);
        let row = |y: u32| &buf[(y * bw * 4) as usize..((y + 1) * bw * 4) as usize];
        assert!((0..20).any(|y| row(y).iter().any(|&b| b != 0)), "im Kasten wird gemalt");
        assert!((20..bh).all(|y| row(y).iter().all(|&b| b == 0)), "unter der Kante Stille");
    }

    #[test]
    fn measure_is_stable() {
        // The layout relies on it: measure twice, one size.
        let mut e = TextEngine::new();
        assert_eq!(e.measure("MatrixKit malt selbst", 30.0),
                   e.measure("MatrixKit malt selbst", 30.0));
    }

    #[test]
    fn wrap_breaks_at_words_and_respects_the_column() {
        // Stage 27: the wrap. Every line fits into the column,
        // no word is torn without need.
        let mut e = TextEngine::new();
        let w = matrixkit_theme::typo::Weight::Normal;
        let text = "Der Schlüssel liegt bereit und der Weg durch die Matrix ist frei";
        let wrapped = e.wrap_styled(text, 15.0, w, 160.0);
        assert!(wrapped.contains('\n'), "langer Text bricht");
        for line in wrapped.split('\n') {
            let (lw, _) = e.measure_styled(line, 15.0, w);
            assert!(lw <= 160.0, "jede Zeile bleibt in der Spalte: {line} ({lw})");
        }
        assert_eq!(wrapped.replace('\n', " "), text, "kein Zeichen geht verloren");
        // And the height grows with the lines — measure knows '\n'.
        let (_, h1) = e.measure_styled("eine Zeile", 15.0, w);
        let (_, h2) = e.measure_styled(&wrapped, 15.0, w);
        assert!(h2 > h1 * 1.5, "mehrzeilig ist höher");
    }

    #[test]
    fn an_oversized_word_is_cut_not_overflowed() {
        let mut e = TextEngine::new();
        let w = matrixkit_theme::typo::Weight::Normal;
        let wrapped = e.wrap_styled("Donaudampfschifffahrtsgesellschaftskapitän", 16.0, w, 90.0);
        assert!(wrapped.contains('\n'), "das Übermaß-Wort wird geteilt");
        for line in wrapped.split('\n') {
            let (lw, _) = e.measure_styled(line, 16.0, w);
            assert!(lw <= 90.0, "auch Stücke bleiben in der Spalte");
        }
    }

    #[test]
    fn paint_mixes_coverage_only_inside_the_buffer() {
        let mut e = TextEngine::new();
        let (bw, bh) = (200u32, 60u32);
        let mut buf = vec![0u8; (bw * bh * 4) as usize];
        let white = matrixkit_theme::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        e.paint("Matrix", 30.0, white, 8.0, 8.0, &mut buf, bw, bh);
        assert!(buf.iter().any(|&b| b != 0), "Glyphen hinterlassen Deckung");
        // And painted far outside: not a byte moves, no panic.
        let before = buf.clone();
        e.paint("Matrix", 30.0, white, 10_000.0, 10_000.0, &mut buf, bw, bh);
        assert_eq!(before, buf);
    }
}

#[cfg(test)]
mod run_tests {
    use super::*;
    use matrixkit_theme::typo::Weight;

    fn ink() -> matrixkit_theme::Rgba {
        matrixkit_theme::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }

    #[test]
    fn runs_share_one_line_and_break_across_their_borders() {
        let mut e = TextEngine::new();
        let plain = [Run { text: "Hallo Welt", px: 16.0, weight: Weight::Normal, family: FAMILY, color: ink() }];
        let split = [
            Run { text: "Hallo ", px: 16.0, weight: Weight::Normal, family: FAMILY, color: ink() },
            Run { text: "Welt", px: 16.0, weight: Weight::Semibold, family: FAMILY, color: ink() },
        ];
        let (w1, h1) = e.measure_runs(&plain, None);
        let (w2, h2) = e.measure_runs(&split, None);
        // One line stays one line — even with two styles.
        assert!((h1 - h2).abs() < 0.01, "eine Zeile hat EINE Hoehe");
        assert!(w2 >= w1, "halbfett ist nie schmaler");
        // The wrap applies ACROSS the run boundary: the same
        // sentence in a narrow column gets taller than in a wide one.
        let (_, wide) = e.measure_runs(&split, Some(1000.0));
        let (narrow_w, narrow_h) = e.measure_runs(&split, Some(40.0));
        assert!(narrow_h > wide, "die enge Spalte bricht");
        assert!(narrow_w <= 40.5, "und bleibt in der Spalte");
    }

    #[test]
    fn every_run_keeps_its_own_ink() {
        // Two runs, two colours: painting must not lay ONE ink
        // over everything (the difference to paint_styled).
        let mut e = TextEngine::new();
        let red = matrixkit_theme::Rgba { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        let blue = matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
        let runs = [
            Run { text: "AAAA", px: 24.0, weight: Weight::Normal, family: FAMILY, color: red },
            Run { text: "BBBB", px: 24.0, weight: Weight::Normal, family: FAMILY, color: blue },
        ];
        let (w, h) = e.measure_runs(&runs, None);
        let (bw, bh) = (w.ceil() as u32 + 8, h.ceil() as u32 + 8);
        let mut buf = vec![0u8; (bw * bh * 4) as usize];
        e.paint_runs(&runs, None, 2.0, 2.0, None, &mut buf, bw, bh);
        let (mut reds, mut blues) = (0, 0);
        for px in buf.chunks_exact(4) {
            if px[2] > 100 && px[0] < 60 { reds += 1 }
            if px[0] > 100 && px[2] < 60 { blues += 1 }
        }
        assert!(reds > 0, "der erste Lauf ist rot");
        assert!(blues > 0, "der zweite Lauf ist blau");
    }
}
