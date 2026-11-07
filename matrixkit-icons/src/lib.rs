//! Drawing core of the living icons — as a library so apps can render
//! their own icon live (e.g. in the About panel of the root layer).

use matrixkit_theme as mk;
use tiny_skia::{
    BlendMode,
    Color, FillRule, FilterQuality, GradientStop, LinearGradient, Mask, Paint, Path, PathBuilder, Pixmap, PixmapPaint, Point,
    Shader, SpreadMode, Stroke, Transform,
};

/// Master canvas of the app icons.
pub const SIZE: u32 = 256;

/// The motif colors of a glyph — primary/secondary/tertiary in the
/// standard style, shades of the text color in the tinted style.
pub struct Glyph {
    pub a: mk::Rgba,
    pub b: mk::Rgba,
    pub c: mk::Rgba,
}

/// Drawing surface for glyphs: wraps pixmap + offset so the same glyph
/// code first paints the shadow (offset) and then the motif.
pub struct Painter<'a> {
    pub pixmap: &'a mut Pixmap,
    pub dy: f32,
    /// Layer depth (user, 2026-07-08): every shape casts a soft shadow
    /// onto what lies BELOW it — not just the glyph as a whole onto
    /// the tile. Off for raw drawings (Plymouth).
    pub layers_shadow: bool,
    /// Material standard (2026-07-08, "make the existing icons better"):
    /// fill() refines automatically — subtle translucency (0.92) and a
    /// gloss edge on EVERY shape. The built-in glyphs inherit the
    /// composer look without a single one being rewritten.
    pub material_standard: bool,
}

impl<'a> Painter<'a> {
    /// The default for app icons: depth + material on.
    pub fn new(pixmap: &'a mut Pixmap) -> Self {
        Painter { pixmap, dy: 0.0, layers_shadow: true, material_standard: true }
    }
    /// Raw (Plymouth/special cases): flat, pure.
    pub fn raw(pixmap: &'a mut Pixmap) -> Self {
        Painter { pixmap, dy: 0.0, layers_shadow: false, material_standard: false }
    }
}

impl Painter<'_> {
    fn raw_fill(&mut self, path: &Path, c: mk::Rgba, dy: f32) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba(c.r, c.g, c.b, c.a).unwrap_or(Color::BLACK));
        paint.anti_alias = true;
        self.pixmap.fill_path(
            path,
            &paint,
            FillRule::Winding,
            Transform::from_translate(0.0, self.dy + dy),
            None,
        );
    }

    pub fn fill(&mut self, path: &Path, c: mk::Rgba) {
        if self.material_standard {
            let refined = mk::Rgba { a: c.a * 0.92, ..c };
            self.fill_ex(path, refined, BlendMode::SourceOver, true);
        } else {
            self.fill_ex(path, c, BlendMode::SourceOver, false);
        }
    }

    /// Icons 3.0: gradient fill — every shape carries light from above.
    /// Seams and layer shadows like fill(), but with its own gradient.
    pub fn fill_history(&mut self, path: &Path, top: mk::Rgba, bottom: mk::Rgba) {
        if self.layers_shadow {
            for (d, a) in [(2.0, 0.10), (4.0, 0.08), (6.0, 0.05)] {
                self.raw_fill(path, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a }, d);
            }
        }
        if self.material_standard {
            self.raw_fill(path, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.5 }, -2.0);
        }
        let b = path.bounds();
        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.shader = LinearGradient::new(
            Point::from_xy(b.x(), b.top()),
            Point::from_xy(b.x(), b.bottom()),
            vec![GradientStop::new(0.0, ts(top)), GradientStop::new(1.0, ts(bottom))],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .unwrap_or(Shader::SolidColor(ts(top)));
        self.pixmap.fill_path(path, &paint, FillRule::Winding, Transform::from_translate(0.0, self.dy), None);
    }

    /// Icons 3.0 inner drawing: details WITHOUT seams and shadows.
    pub fn fine(&mut self, path: &Path, c: mk::Rgba) {
        self.raw_fill(path, c, 0.0);
    }

    /// A ring (circle outline) — grooves, lenses, countdown rings.
    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, width: f32, c: mk::Rgba) {
        let mut pb = PathBuilder::new();
        pb.push_circle(cx, cy, r);
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(ts(c));
            paint.anti_alias = true;
            let stroke = Stroke { width: width, ..Default::default() };
            self.pixmap.stroke_path(&path, &paint, &stroke, Transform::from_translate(0.0, self.dy), None);
        }
    }

    /// A circular arc (degrees, 0 = right, clockwise) — sound,
    /// countdown, globe latitudes.
    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, from: f32, until: f32, width: f32, c: mk::Rgba) {
        let mut pb = PathBuilder::new();
        let steps = 32;
        for i in 0..=steps {
            let w = (from + (until - from) * i as f32 / steps as f32).to_radians();
            let (x, y) = (cx + r * w.cos(), cy + r * w.sin());
            if i == 0 { pb.move_to(x, y) } else { pb.line_to(x, y) }
        }
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(ts(c));
            paint.anti_alias = true;
            let stroke = Stroke { width: width, ..Default::default() };
            self.pixmap.stroke_path(&path, &paint, &stroke, Transform::from_translate(0.0, self.dy), None);
        }
    }

    /// Material fill (Icon Composer): blend mode + gloss light.
    pub fn fill_ex(&mut self, path: &Path, c: mk::Rgba, blend: BlendMode, gloss: bool) {
        if self.layers_shadow {
            // Fake blur: three offset alpha layers = a soft cast shadow.
            for (d, a) in [(2.0, 0.10), (4.0, 0.08), (6.0, 0.05)] {
                self.raw_fill(path, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a }, d);
            }
        }
        if gloss {
            // Specular fake: bright layer minimally up — the seam
            // protruding at the top stays visible as a gloss edge.
            self.raw_fill(path, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.55 }, -2.5);
        }
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba(c.r, c.g, c.b, c.a).unwrap_or(Color::BLACK));
        paint.anti_alias = true;
        paint.blend_mode = blend;
        self.pixmap.fill_path(
            path,
            &paint,
            FillRule::Winding,
            Transform::from_translate(0.0, self.dy),
            None,
        );
    }
}

/// A MatrixKit app icon: name = .desktop/icon name + glyph recipe.
/// The tile comes from the same recipe for all apps (family membership).
pub struct IconSpec {
    pub name: &'static str,
    pub glyph_fn: fn(&Glyph, &mut Painter),
    /// R207: a PAINTED icon instead of tile + glyph — embedded
    /// PNG bytes (SIZE×SIZE, own tile, real transparency). The
    /// palette does not recolour it; it is the app's face, just as
    /// the operator drew it (Rabbit, 21.09.2026).
    pub bitmap: Option<&'static [u8]>,
}

/// All MatrixKit apps. New apps add exactly ONE line + drawing fn here.
pub const ICONS: &[IconSpec] = &[
    IconSpec { name: "matrix-sysmon", glyph_fn: glyph_sysmon, bitmap: None },
    IconSpec { name: "matrix-colors", glyph_fn: glyph_colors, bitmap: None },
    IconSpec { name: "matrix-help", glyph_fn: glyph_help, bitmap: None },
    IconSpec { name: "matrix-sounds", glyph_fn: glyph_sounds, bitmap: None },
    IconSpec { name: "matrix-codes", glyph_fn: glyph_codes, bitmap: None },
    IconSpec { name: "matrix-keys-app", glyph_fn: glyph_key_app, bitmap: None },
    IconSpec { name: "matrix-recovery", glyph_fn: glyph_guard, bitmap: None },
    IconSpec { name: "matrix-settings", glyph_fn: glyph_settings, bitmap: None },
    IconSpec { name: "matrix-updater", glyph_fn: glyph_updater, bitmap: None },
    IconSpec { name: "matrix-dev-access", glyph_fn: glyph_developer_access, bitmap: None },
    IconSpec { name: "matrix-canvas", glyph_fn: glyph_canvas, bitmap: None },
    IconSpec { name: "matrix-web", glyph_fn: glyph_web, bitmap: None },
    IconSpec { name: "matrix-files", glyph_fn: glyph_files, bitmap: None },
    IconSpec { name: "matrix-morpheus", glyph_fn: glyph_installer, bitmap: None },
    IconSpec { name: "matrix-icons", glyph_fn: glyph_icons, bitmap: None },
    IconSpec { name: "matrix-keyboard", glyph_fn: glyph_keyboard, bitmap: None },
    IconSpec { name: "matrix-terminal", glyph_fn: glyph_terminal, bitmap: None },
    IconSpec { name: "matrix-capture", glyph_fn: glyph_capture, bitmap: None },
    IconSpec { name: "matrix-player", glyph_fn: glyph_player, bitmap: None },
    IconSpec { name: "matrix-markdown", glyph_fn: glyph_markdown, bitmap: None },
    IconSpec { name: "matrix-mandelbrot", glyph_fn: glyph_mandelbrot, bitmap: None },
    IconSpec { name: "matrix-disks", glyph_fn: glyph_disks, bitmap: None },
    IconSpec { name: "matrix-for-surface", glyph_fn: glyph_surface_stick, bitmap: None },
    IconSpec { name: "matrix-construct", glyph_fn: glyph_construct, bitmap: None },
    IconSpec { name: "matrix-rabbit", glyph_fn: glyph_rabbit, bitmap: Some(include_bytes!("../assets/matrix-rabbit.png")) },
    IconSpec { name: "matrix-hardline", glyph_fn: glyph_hardline, bitmap: None },
    IconSpec { name: "matrix-arcade", glyph_fn: glyph_arcade, bitmap: Some(include_bytes!("../assets/matrix-arcade.png")) },
    // Round 58: the Duo's apps that showed a letter in the dock.
    IconSpec { name: "matrix-tube", glyph_fn: glyph_tube, bitmap: None },
    IconSpec { name: "matrix-start", glyph_fn: glyph_start, bitmap: None },
    IconSpec { name: "matrix-camera", glyph_fn: glyph_camera, bitmap: None },
    IconSpec { name: "matrix-krabbler", glyph_fn: glyph_krabbler, bitmap: None },
    IconSpec { name: "matrix-oracle", glyph_fn: glyph_oracle, bitmap: None },
    // Round 81: Matrix Einkauf.
    IconSpec { name: "matrix-shopping", glyph_fn: glyph_shopping, bitmap: None },
];

#[derive(PartialEq)]
pub enum Style {
    Standard,
    Tinted,
}
pub fn style() -> Style {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    // R107: icon-style is the contract; icon-stil read as legacy, value
    // "tinted" (legacy: "getoent").
    let raw = std::fs::read_to_string(format!("{home}/.config/matrix/icon-style"))
        .or_else(|_| std::fs::read_to_string(format!("{home}/.config/matrix/icon-stil")))
        .unwrap_or_default();
    match raw.trim() {
        "tinted" | "getoent" => Style::Tinted,
        _ => Style::Standard,
    }
}

// --- The tile: squircle with depth ------------------------------------------

/// Superellipse (|x/r|^n + |y/r|^n = 1) — the reference's continuous curvature.
pub fn squircle(cx: f32, cy: f32, r: f32, n: f32) -> Path {
    let mut pb = PathBuilder::new();
    let steps = 256;
    let exp = 2.0 / n;
    for i in 0..steps {
        let t = (i as f32 / steps as f32) * std::f32::consts::TAU;
        let (s, c) = t.sin_cos();
        let x = cx + r * c.abs().powf(exp) * c.signum();
        let y = cy + r * s.abs().powf(exp) * s.signum();
        if i == 0 {
            pb.move_to(x, y);
        } else {
            pb.line_to(x, y);
        }
    }
    pb.close();
    pb.finish().expect("squircle")
}

pub(crate) fn light(c: mk::Rgba, f: f32) -> mk::Rgba {
    mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }.over(c, f)
}

pub(crate) fn dark(c: mk::Rgba, f: f32) -> mk::Rgba {
    mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }.over(c, f)
}

pub(crate) fn ts(c: mk::Rgba) -> Color {
    Color::from_rgba(c.r, c.g, c.b, c.a).unwrap_or(Color::BLACK)
}

/// The shared tile of all MatrixKit icons: squircle, gentle vertical
/// gradient (brighter on top), light edge at the top.
pub fn tile(p: &mk::Palette, pixmap: &mut Pixmap) {
    let form = squircle(128.0, 128.0, 120.0, 4.6);

    // Vertical gradient: light falls from above
    let top = light(p.surface_container, 0.10);
    let bottom = dark(p.surface_container, 0.08);
    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.shader = LinearGradient::new(
        Point::from_xy(128.0, 8.0),
        Point::from_xy(128.0, 248.0),
        vec![
            GradientStop::new(0.0, ts(top)),
            GradientStop::new(1.0, ts(bottom)),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap_or(Shader::SolidColor(ts(p.surface_container)));
    pixmap.fill_path(&form, &paint, FillRule::Winding, Transform::identity(), None);
    light_edge(pixmap);
}

/// The tile's light edge: fine border gradient, strongest at the top, gone
/// below (round 96: also laid over a theme's icon in the Matrix form).
fn light_edge(pixmap: &mut Pixmap) {
    let edge = squircle(128.0, 128.0, 118.5, 4.6);
    let mut light = Paint::default();
    light.anti_alias = true;
    light.shader = LinearGradient::new(
        Point::from_xy(128.0, 8.0),
        Point::from_xy(128.0, 160.0),
        vec![
            GradientStop::new(0.0, Color::from_rgba(1.0, 1.0, 1.0, 0.22).unwrap()),
            GradientStop::new(1.0, Color::from_rgba(1.0, 1.0, 1.0, 0.0).unwrap()),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap_or(Shader::SolidColor(Color::TRANSPARENT));
    let stroke = Stroke { width: 3.0, ..Default::default() };
    pixmap.stroke_path(&edge, &light, &stroke, Transform::identity(), None);
}

// --- The glyphs (Icons 3.0: DESIGNED, see glyphs.rs) -------------------
mod glyphs;
pub use glyphs::*;

pub fn round_rect(x1: f32, y1: f32, x2: f32, y2: f32, r: f32) -> Path {
    let r = r.min((x2 - x1) / 2.0).min((y2 - y1) / 2.0);
    let k = 0.552_285 * r;
    let mut pb = PathBuilder::new();
    pb.move_to(x1 + r, y1);
    pb.line_to(x2 - r, y1);
    pb.cubic_to(x2 - r + k, y1, x2, y1 + r - k, x2, y1 + r);
    pb.line_to(x2, y2 - r);
    pb.cubic_to(x2, y2 - r + k, x2 - r + k, y2, x2 - r, y2);
    pb.line_to(x1 + r, y2);
    pb.cubic_to(x1 + r - k, y2, x1, y2 - r + k, x1, y2 - r);
    pb.line_to(x1, y1 + r);
    pb.cubic_to(x1, y1 + r - k, x1 + r - k, y1, x1 + r, y1);
    pb.close();
    pb.finish().expect("round_rect")
}


/// The bare tile as PNG (round 57): the same squircle, gradient and light
/// edge every app icon stands on — the dock lays a stand-in's mark on it,
/// so apps without a glyph keep the exact shape of the others.
pub fn tile_png(palette: &mk::Palette) -> Option<Vec<u8>> {
    let mut pixmap = Pixmap::new(SIZE, SIZE)?;
    tile(palette, &mut pixmap);
    pixmap.encode_png().ok()
}

/// Render a complete app icon as PNG — the same pipeline as the
/// generator (tile + shadow + glyph in the current style).
pub fn render_png(name: &str, palette: &mk::Palette) -> Option<Vec<u8>> {
    // Composer recipes of the user override the built-in glyphs.
    if let Some(recipe) = recipe_load(name) {
        return recipe_png(&recipe, palette);
    }
    let spec = ICONS.iter().find(|s| s.name == name)?;
    icon_pixmap(palette, spec)?.encode_png().ok()
}

/// An app's finished icon as a pixmap (SIZE×SIZE): a painted
/// bitmap if the app brings one — otherwise tile + glyph in the
/// current style. ONE path for runtime (render_png), generator and
/// image build.
pub fn icon_pixmap(palette: &mk::Palette, spec: &IconSpec) -> Option<Pixmap> {
    if let Some(bytes) = spec.bitmap {
        let src = Pixmap::decode_png(bytes).ok()?;
        if src.width() == SIZE && src.height() == SIZE {
            return Some(src);
        }
        let mut out = Pixmap::new(SIZE, SIZE)?;
        let s = SIZE as f32 / src.width().max(1) as f32;
        out.draw_pixmap(
            0,
            0,
            src.as_ref(),
            &tiny_skia::PixmapPaint { quality: tiny_skia::FilterQuality::Bicubic, ..Default::default() },
            Transform::from_scale(s, s),
            None,
        );
        return Some(out);
    }
    let glyph = match style() {
        Style::Standard => Glyph {
            a: palette.primary,
            b: palette.secondary,
            c: palette.tertiary,
        },
        Style::Tinted => Glyph {
            a: with_alpha(palette.on_surface, 1.0),
            b: with_alpha(palette.on_surface, 0.72),
            c: with_alpha(palette.on_surface, 0.5),
        },
    };
    let mut pixmap = Pixmap::new(SIZE, SIZE)?;
    tile(palette, &mut pixmap);
    // Layer shadows are handled by the painter shape by shape.
    (spec.glyph_fn)(&glyph, &mut Painter::new(&mut pixmap));
    Some(pixmap)
}

pub fn with_alpha(c: mk::Rgba, a: f32) -> mk::Rgba {
    c.with_alpha(a)
}

/// The living avatar: circle surface in primary_container with a 3×3
/// dot grid (the Matrix sign) — no photo, no letter, but fresh from
/// the palette on every wallpaper change.
/// Boot sign for Plymouth: the Matrix dot grid on transparent ground
/// in calm warm white — deliberately palette-free, because at power-on
/// there is no user and no wallpaper yet.
pub fn plymouth_logo_png() -> Option<Vec<u8>> {
    let mut pixmap = Pixmap::new(SIZE, SIZE)?;
    let warm_white = mk::Rgba { r: 0.902, g: 0.890, b: 0.851, a: 1.0 };
    let half = with_alpha(warm_white, 0.45);
    let mut z = Painter::raw(&mut pixmap);
    let d = 30.0;
    for row in 0..3 {
        for column in 0..3 {
            let x = 128.0 - d / 2.0 + (column as f32 - 1.0) * 52.0;
            let y = 128.0 - d / 2.0 + (row as f32 - 1.0) * 52.0;
            let color = if (row + column) % 2 == 0 { warm_white } else { half };
            z.fill(&round_rect(x, y, x + d, y + d, d / 2.0), color);
        }
    }
    pixmap.encode_png().ok()
}

/// A single frame of the Plymouth loading animation (two-step plays the
/// frames in a run_loop): three dots whose opacity wanders sinusoidally —
/// the boot rendition of the calm Matrix pulse.
pub fn plymouth_animation_png(frame: u32, frames: u32) -> Option<Vec<u8>> {
    let mut pixmap = Pixmap::new(96, 20)?;
    let warm_white = mk::Rgba { r: 0.902, g: 0.890, b: 0.851, a: 1.0 };
    let mut z = Painter::new(&mut pixmap);
    let d = 12.0;
    for i in 0..3 {
        let phase = (frame as f32 / frames as f32) * std::f32::consts::TAU - i as f32 * 0.9;
        let opacity = 0.30 + 0.70 * (0.5 + 0.5 * phase.sin());
        let x = 12.0 + i as f32 * 30.0;
        z.fill(&round_rect(x, 4.0, x + d, 4.0 + d, d / 2.0), with_alpha(warm_white, opacity));
    }
    pixmap.encode_png().ok()
}

pub fn avatar_png(p: &mk::Palette) -> Option<Vec<u8>> {
    let mut pixmap = Pixmap::new(SIZE, SIZE)?;
    // circle = superellipse with exponent 2
    let circle = squircle(128.0, 128.0, 120.0, 2.0);
    let top = light(p.primary_container, 0.10);
    let bottom = dark(p.primary_container, 0.08);
    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.shader = LinearGradient::new(
        Point::from_xy(128.0, 8.0),
        Point::from_xy(128.0, 248.0),
        vec![
            GradientStop::new(0.0, ts(top)),
            GradientStop::new(1.0, ts(bottom)),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    )
    .unwrap_or(Shader::SolidColor(ts(p.primary_container)));
    pixmap.fill_path(&circle, &paint, FillRule::Winding, Transform::identity(), None);

    let full = p.on_primary_container;
    let half = with_alpha(p.on_primary_container, 0.45);
    let mut z = Painter::raw(&mut pixmap);
    let d = 30.0;
    for row in 0..3 {
        for column in 0..3 {
            let x = 128.0 - d / 2.0 + (column as f32 - 1.0) * 52.0;
            let y = 128.0 - d / 2.0 + (row as f32 - 1.0) * 52.0;
            let color = if (row + column) % 2 == 0 { full } else { half };
            z.fill(&round_rect(x, y, x + d, y + d, d / 2.0), color);
        }
    }
    pixmap.encode_png().ok()
}

// ===========================================================================
// The recipe system (Matrix Icon Composer, 2026-07-08): icons as DATA.
// The composer (app #22) stores compositions as JSON recipes at
// ~/.config/matrix/icons/<name>.json; render_png loads user recipes
// BEFORE the built-in glyphs — composed icons take effect system-wide
// immediately (dock, launcher, notifications) and live with the
// palette because they build on the glyph tones a/b/c (standard/tinted
// palette fades come for free).
// ===========================================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "form", rename_all = "kebab-case")]
pub enum RecipeForm {
    /// Superellipse around (cx, cy): r = half edge length, n = squareness
    /// (2 = circle, 5 = squircle tile).
    Squircle { cx: f32, cy: f32, r: f32, n: f32 },
    /// Rectangle x1,y1 → x2,y2 with corner radius.
    RoundRect { x1: f32, y1: f32, x2: f32, y2: f32, r: f32 },
}

fn one() -> f32 { 1.0 }
fn normal() -> String { String::from("normal") }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RecipeLayer {
    #[serde(flatten)]
    pub form: RecipeForm,
    /// Palette slot: "a" (primary), "b" (secondary), "c" (tertiary).
    pub slot: String,
    /// Opacity 0.15–1.0 — the reference Photos look lives on translucency.
    #[serde(default = "one")]
    pub opacity: f32,
    /// "normal" | "multiplizieren" — overlaps get richer.
    #[serde(default = "normal")]
    pub blend: String,
    /// Gloss light at the top edge (specular fake: bright layer dy −2).
    #[serde(default)]
    pub gloss: bool,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Recipe {
    pub layers: Vec<RecipeLayer>,
}

pub fn recipe_path(name: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::PathBuf::from(format!("{home}/.config/matrix/icons/{name}.json"))
}

pub fn recipe_load(name: &str) -> Option<Recipe> {
    let raw = std::fs::read_to_string(recipe_path(name)).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_recipe(name: &str, recipe: &Recipe) -> std::io::Result<()> {
    let path = recipe_path(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(recipe).unwrap_or_default())
}

fn draw_recipe(recipe: &Recipe, g: &Glyph, z: &mut Painter) {
    for layer in &recipe.layers {
        let base = match layer.slot.as_str() {
            "b" => g.b,
            "c" => g.c,
            _ => g.a,
        };
        let color = mk::Rgba {
            a: base.a * layer.opacity.clamp(0.15, 1.0),
            ..base
        };
        let blend = if layer.blend == "multiplizieren" {
            BlendMode::Multiply
        } else {
            BlendMode::SourceOver
        };
        let path = match &layer.form {
            RecipeForm::Squircle { cx, cy, r, n } => squircle(*cx, *cy, *r, *n),
            RecipeForm::RoundRect { x1, y1, x2, y2, r } => round_rect(*x1, *y1, *x2, *y2, *r),
        };
        z.fill_ex(&path, color, blend, layer.gloss);
    }
}

/// Render a recipe fully (tile + shadow layer + glyph) — exactly the
/// path of the built-in icons; also the composer preview.
pub fn recipe_png(recipe: &Recipe, palette: &mk::Palette) -> Option<Vec<u8>> {
    let glyph = match style() {
        Style::Standard => Glyph {
            a: palette.primary,
            b: palette.secondary,
            c: palette.tertiary,
        },
        Style::Tinted => Glyph {
            a: with_alpha(palette.on_surface, 1.0),
            b: with_alpha(palette.on_surface, 0.72),
            c: with_alpha(palette.on_surface, 0.5),
        },
    };
    let mut pixmap = Pixmap::new(SIZE, SIZE)?;
    tile(palette, &mut pixmap);
    draw_recipe(recipe, &glyph, &mut Painter::new(&mut pixmap));
    pixmap.encode_png().ok()
}

/// The path to a FOREIGN app icon (`Icon=` from the
/// desktop file), resolved via the icon theme: hicolor,
/// Flatpak exports, pixmaps — largest raster first, scalable
/// SVG as the fallback.
///
/// Until the finger probe on 16.08.2026 this search lived in the iced gate
/// of matrix-start and delivered iced handles. The first-light path
/// did not know it and showed only the initial for EVERY foreign app
/// — yet the search is pure path work. It belongs here,
/// so that both paths find the same file.
pub fn foreign_icon_path(icon: &str) -> Option<std::path::PathBuf> {
    let exists = |p: String| -> Option<std::path::PathBuf> {
        let path = std::path::PathBuf::from(&p);
        path.exists().then_some(path)
    };
    if icon.starts_with('/') {
        return exists(icon.to_string());
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let roots = [
        String::from("/usr/share/icons"),
        String::from("/var/lib/flatpak/exports/share/icons"),
        format!("{home}/.local/share/flatpak/exports/share/icons"),
        format!("{home}/.local/share/icons"),
    ];
    for root in &roots {
        for size in ["128x128", "256x256", "96x96", "64x64", "48x48"] {
            if let Some(p) = exists(format!("{root}/hicolor/{size}/apps/{icon}.png")) {
                return Some(p);
            }
        }
        if let Some(p) = exists(format!("{root}/hicolor/scalable/apps/{icon}.svg")) {
            return Some(p);
        }
    }
    for p in [
        format!("/usr/share/pixmaps/{icon}.png"),
        format!("/usr/share/pixmaps/{icon}.svg"),
    ] {
        if let Some(p) = exists(p) {
            return Some(p);
        }
    }
    None
}

// ------------------------------------------------------------------
// Round 88: ICON THEMES (owner: «be able to choose the WhiteSur icons
// instead of the Matrix icons»). The setting `icon-theme` names an installed
// freedesktop icon theme; «Matrix» (or nothing) keeps the family above.
// An app the theme does not know keeps its Matrix icon.

/// The name of the kit's own icons in the theme list.
pub const MATRIX_THEME: &str = "Matrix";

fn icon_roots() -> Vec<std::path::PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    [format!("{home}/.local/share/icons"), format!("{home}/.icons"), String::from("/usr/local/share/icons"), String::from("/usr/share/icons")]
        .into_iter()
        .map(std::path::PathBuf::from)
        .collect()
}

fn theme_dir(theme: &str) -> Option<std::path::PathBuf> {
    icon_roots().into_iter().map(|r| r.join(theme)).find(|d| d.join("index.theme").exists())
}

/// The installed themes that carry application icons («Matrix» first;
/// cursor-only themes and the bare fallbacks are left out).
pub fn themes() -> Vec<String> {
    let mut out = vec![String::from(MATRIX_THEME)];
    for root in icon_roots() {
        let Ok(entries) = std::fs::read_dir(&root) else { continue };
        let mut found: Vec<String> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if matches!(name.as_str(), "hicolor" | "default" | "locolor") || out.contains(&name) {
                    return None;
                }
                let index = std::fs::read_to_string(e.path().join("index.theme")).ok()?;
                index.contains("Context=Applications").then_some(name)
            })
            .collect();
        found.sort();
        out.extend(found);
    }
    out
}

/// The chosen theme; None = the Matrix icons.
pub fn current_theme() -> Option<String> {
    matrixkit_theme::settings::read("icon-theme").filter(|t| !t.is_empty() && t != MATRIX_THEME)
}

/// Round 96: everything that changes how app icons look (theme + form) —
/// what a cache must be dropped for.
pub fn look() -> Option<String> {
    current_theme().map(|t| format!("{t}|{}", if matrix_form() { "matrix" } else { "original" }))
}

pub fn set_theme(name: &str) {
    matrixkit_theme::settings::write("icon-theme", name);
}

/// The application folders of a theme, best first (scalable, then large).
fn app_dirs(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let Ok(index) = std::fs::read_to_string(dir.join("index.theme")) else { return Vec::new() };
    let mut found: Vec<(u32, String)> = Vec::new();
    let mut section: Option<String> = None;
    let (mut size, mut apps, mut scalable) = (0u32, false, false);
    let flush = |section: &Option<String>, size: u32, apps: bool, scalable: bool, found: &mut Vec<(u32, String)>| {
        // The one-colour «symbolic» folders are not app icons for a dock.
        if let (Some(s), true) = (section, apps && !section.as_deref().is_some_and(|s| s.contains("symbolic"))) {
            found.push((if scalable { u32::MAX } else { size }, s.clone()));
        }
    };
    for line in index.lines().map(str::trim) {
        if line.starts_with('[') && line.ends_with(']') {
            flush(&section, size, apps, scalable, &mut found);
            let name = &line[1..line.len() - 1];
            section = (name != "Icon Theme").then(|| name.to_string());
            (size, apps, scalable) = (0, false, false);
        } else if let Some(v) = line.strip_prefix("Size=") {
            size = v.parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("Context=") {
            apps = v == "Applications";
        } else if let Some(v) = line.strip_prefix("Type=") {
            scalable = v == "Scalable";
        }
    }
    flush(&section, size, apps, scalable, &mut found);
    found.sort_by(|a, b| b.0.cmp(&a.0));
    found.into_iter().map(|(_, s)| dir.join(s)).collect()
}

fn inherits(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(dir.join("index.theme"))
        .ok()
        .and_then(|t| t.lines().find_map(|l| l.trim().strip_prefix("Inherits=").map(|v| v.split(',').map(|s| s.trim().to_string()).collect())))
        .unwrap_or_default()
}

/// Where `icon` lives in `theme` (or a theme it inherits, hicolor aside —
/// the foreign lookup knows that one).
pub fn themed_icon_path(theme: &str, icon: &str) -> Option<std::path::PathBuf> {
    let mut queue = vec![theme.to_string()];
    let mut seen = Vec::new();
    while let Some(t) = queue.pop() {
        if seen.contains(&t) || t == "hicolor" || seen.len() > 4 {
            continue;
        }
        seen.push(t.clone());
        let Some(dir) = theme_dir(&t) else { continue };
        for d in app_dirs(&dir) {
            for ext in ["svg", "png"] {
                let p = d.join(format!("{icon}.{ext}"));
                if p.exists() {
                    return Some(p);
                }
            }
        }
        queue.extend(inherits(&dir));
    }
    // GNOME's way (06.11.2026, the owner: «GNOME's icons should work
    // too»): Adwaita brings no app icons of its own — every app
    // ships its colour icon in hicolor. A family without app icons shows
    // those; a family with them (WhiteSur) keeps the Matrix icon for the
    // apps it does not know.
    if !brings_app_icons(&family_of(theme)) {
        return foreign_icon_path(icon);
    }
    None
}

/// Whether a theme carries app icons of its own (any file in a
/// non-symbolic Applications folder). Kept per theme: asked per icon.
pub fn brings_app_icons(theme: &str) -> bool {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static KNOWN: Mutex<Option<HashMap<String, bool>>> = Mutex::new(None);
    let mut guard = KNOWN.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    if let Some(v) = map.get(theme) {
        return *v;
    }
    let v = theme_dir(theme).is_some_and(|dir| {
        app_dirs(&dir).iter().any(|d| std::fs::read_dir(d).is_ok_and(|mut e| e.next().is_some()))
    });
    map.insert(theme.to_string(), v);
    v
}

/// A theme with its variants (WhiteSur: WhiteSur-dark, WhiteSur-light;
/// Adwaita: AdwaitaLegacy, HighContrast). `variants[0]` is the theme itself.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeFamily {
    pub name: String,
    pub variants: Vec<String>,
}

/// The family a theme belongs to: a theme whose name begins with another's
/// («WhiteSur-dark», «AdwaitaLegacy») or that inherits another directly
/// («HighContrast» → Adwaita) is that one's variant.
fn root_in(theme: &str, all: &[String], inherited: &dyn Fn(&str) -> Vec<String>) -> String {
    let mut t = theme.to_string();
    for _ in 0..4 {
        let by_name = all.iter().filter(|o| *o != &t && t.starts_with(o.as_str())).max_by_key(|o| o.len()).cloned();
        let by_inherit = || inherited(&t).into_iter().find(|i| all.contains(i) && !i.starts_with(t.as_str()));
        match by_name.or_else(by_inherit) {
            Some(up) => t = up,
            None => break,
        }
    }
    t
}

pub fn family_of(theme: &str) -> String {
    let all = themes();
    root_in(theme, &all, &|t| theme_dir(t).map(|d| inherits(&d)).unwrap_or_default())
}

fn families_in(all: &[String], inherited: &dyn Fn(&str) -> Vec<String>) -> Vec<ThemeFamily> {
    let mut out: Vec<ThemeFamily> = Vec::new();
    for t in all {
        let root = root_in(t, all, inherited);
        match out.iter_mut().find(|f| f.name == root) {
            Some(f) => {
                if *t != root {
                    f.variants.push(t.clone());
                }
            }
            None => {
                let mut variants = vec![root.clone()];
                if *t != root {
                    variants.push(t.clone());
                }
                out.push(ThemeFamily { name: root, variants });
            }
        }
    }
    out
}

/// The installed themes grouped into families («Matrix» first, alone).
pub fn theme_families() -> Vec<ThemeFamily> {
    families_in(&themes(), &|t| theme_dir(t).map(|d| inherits(&d)).unwrap_or_default())
}

/// A variant's name in its family: «Standard», «Dunkel», «Hell», «Legacy»,
/// «Hoher Kontrast» — otherwise the part after the family's name.
pub fn variant_label(family: &str, variant: &str) -> String {
    if family == variant {
        return String::from("Standard");
    }
    let rest = variant.strip_prefix(family).unwrap_or(variant).trim_start_matches(['-', '_', ' ']);
    match rest.to_ascii_lowercase().as_str() {
        "dark" => String::from("Dunkel"),
        "light" => String::from("Hell"),
        "highcontrast" => String::from("Hoher Kontrast"),
        "legacy" => String::from("Legacy"),
        _ => rest.to_string(),
    }
}

/// The icon name a desktop file gives an app (`Icon=`), if any.
pub fn desktop_icon(app_id: &str) -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    for dir in [format!("{home}/.local/share/applications"), String::from("/usr/local/share/applications"), String::from("/usr/share/applications"), String::from("/var/lib/flatpak/exports/share/applications")] {
        if let Ok(text) = std::fs::read_to_string(format!("{dir}/{app_id}.desktop")) {
            return text.lines().find_map(|l| l.strip_prefix("Icon=").map(|v| v.trim().to_string()));
        }
    }
    None
}

/// An icon of the chosen theme as PNG bytes (`size` × `size`), SVGs drawn
/// by the kit's rasterizer. None without a chosen theme or a match. In the
/// Matrix form (the default, `matrix_form`) it comes out as a 256 px tile.
pub fn theme_png(icon: &str, size: u32) -> Option<Vec<u8>> {
    let theme = current_theme()?;
    let path = themed_icon_path(&theme, icon)?;
    let bytes = std::fs::read(&path).ok()?;
    let form = matrix_form();
    let raster = if form { SIZE.max(size) } else { size };
    let png = if path.extension().is_some_and(|e| e == "svg") {
        matrixkit_canvas::Svg::parse(&bytes)?.raster(raster, raster)?.png()?
    } else {
        bytes
    };
    if form {
        let palette = mk::Palette::load().unwrap_or_default();
        return in_matrix_form(&png, &palette).or(Some(png));
    }
    Some(png)
}

/// Round 96 (owner: «the WhiteSur icons should have the same shape as the
/// Matrix icons»): theme icons in the Matrix form — the default; the setting
/// `icon-theme-form` = `original` keeps the theme's own outline.
pub fn matrix_form() -> bool {
    matrixkit_theme::settings::read("icon-theme-form").as_deref() != Some("original")
}

pub fn set_matrix_form(on: bool) {
    matrixkit_theme::settings::write("icon-theme-form", if on { "matrix" } else { "original" });
}

/// A theme icon in the MATRIX FORM: its own margin and shadow cut away (the
/// visible motif's bounds), scaled to fill the tile, masked by the Matrix
/// squircle over the Matrix tile (round or odd outlines get the same
/// silhouette), finished with the tile's light edge. A 256 px PNG.
pub fn in_matrix_form(png: &[u8], palette: &mk::Palette) -> Option<Vec<u8>> {
    let src = Pixmap::decode_png(png).ok()?;
    let (w, h) = (src.width(), src.height());
    let data = src.data();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0u32, 0u32);
    for y in 0..h {
        for x in 0..w {
            if data[((y * w + x) * 4 + 3) as usize] > 96 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let (bw, bh) = ((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32);
    let scale = 240.0 / bw.max(bh);
    let tx = 128.0 - (x0 as f32 + bw / 2.0) * scale;
    let ty = 128.0 - (y0 as f32 + bh / 2.0) * scale;
    let mut out = Pixmap::new(SIZE, SIZE)?;
    tile(palette, &mut out);
    let mut mask = Mask::new(SIZE, SIZE)?;
    mask.fill_path(&squircle(128.0, 128.0, 120.0, 4.6), FillRule::Winding, true, Transform::identity());
    let paint = PixmapPaint { quality: FilterQuality::Bicubic, ..Default::default() };
    out.draw_pixmap(0, 0, src.as_ref(), &paint, Transform::from_row(scale, 0.0, 0.0, scale, tx, ty), Some(&mask));
    light_edge(&mut out);
    out.encode_png().ok()
}

/// Round 88b (owner: «the icons in the dock still look the same» — the
/// dock held only Matrix apps, which no theme knows): a Matrix app wears the
/// theme's icon for its ROLE, in order of preference (freedesktop names).
pub const STAND_INS: &[(&str, &[&str])] = &[
    ("matrix-files", &["org.gnome.Nautilus", "system-file-manager"]),
    ("matrix-settings", &["org.gnome.Settings", "preferences-system"]),
    ("matrix-tube", &["youtube", "multimedia-video-player"]),
    ("matrix-player", &["org.gnome.Totem", "multimedia-video-player"]),
    ("matrix-terminal", &["org.gnome.Console", "utilities-terminal"]),
    ("matrix-camera", &["org.gnome.Snapshot", "camera-app"]),
    ("matrix-help", &["org.gnome.Yelp", "help-browser"]),
    ("matrix-capture", &["org.gnome.Screenshot", "gnome-screenshot"]),
    ("matrix-markdown", &["org.gnome.TextEditor", "accessories-text-editor"]),
    ("matrix-shopping", &["basket", "gnome-todo"]),
    ("matrix-sysmon", &["org.gnome.SystemMonitor", "system-monitor"]),
    ("matrix-start", &["view-app-grid"]),
    ("matrix-krabbler", &["applications-games"]),
    ("matrix-web", &["firefox", "web-browser"]),
];

/// THE app icon (one door for dock, start and the rest): the chosen theme's
/// icon for the app (its desktop `Icon=`, else its id), else the Matrix
/// family's.
pub fn app_png(app_id: &str, desktop: Option<&str>, palette: &mk::Palette, size: u32) -> Option<Vec<u8>> {
    if current_theme().is_some() {
        let name = desktop.map(str::to_string).or_else(|| desktop_icon(app_id));
        if let Some(png) = name.as_deref().and_then(|n| theme_png(n, size)).or_else(|| theme_png(app_id, size)) {
            return Some(png);
        }
        let roles = STAND_INS.iter().find(|(id, _)| *id == app_id).map(|(_, r)| *r).unwrap_or(&[]);
        if let Some(png) = roles.iter().find_map(|n| theme_png(n, size)) {
            return Some(png);
        }
    }
    render_png(app_id, palette)
}

/// Round 95: THE app icon as a FILE (scenes and the dock draw pictures from
/// paths): app_png, written once per app, size, theme and palette into the
/// runtime directory, then found again. One cache for every app — the dock
/// carried its own until now.
/// The icon of an app known by its NAME for people or its app id (a
/// notification's sender, «Matrix Player», «firefox»): the app index
/// first, house apps without a desktop entry by their name (round 129 —
/// was the bar's own lookup, the lock screen needs it too).
pub fn for_app(app: &str, palette: &mk::Palette, size: u32) -> Option<std::path::PathBuf> {
    let low = app.to_lowercase();
    let apps = mk::apps::index();
    let entry = apps.iter().find(|e| e.name.to_lowercase() == low || e.app_id.to_lowercase() == low || e.app_id.to_lowercase().ends_with(&format!(".{low}")));
    match entry {
        Some(e) => icon_file(&e.app_id, e.icon.as_deref(), palette, size),
        None => icon_file(&low, None, palette, size).or_else(|| icon_file(&low.replace(' ', "-"), None, palette, size)),
    }
}

pub fn icon_file(app_id: &str, desktop: Option<&str>, palette: &mk::Palette, size: u32) -> Option<std::path::PathBuf> {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    format!("{palette:?}").hash(&mut h);
    look().hash(&mut h);
    desktop.hash(&mut h);
    let dir = matrixkit_theme::runtime::path("matrix-app-icons");
    let file = dir.join(format!("{}-{size}-{:x}.png", app_id.replace('/', "_"), h.finish()));
    if file.exists() {
        freshen(&file);
        return Some(file);
    }
    let png = app_png(app_id, desktop, palette, size)?;
    std::fs::create_dir_all(&dir).ok()?;
    let tmp = file.with_extension("new");
    std::fs::write(&tmp, png).ok()?;
    std::fs::rename(&tmp, &file).ok()?;
    prune_icon_files(&dir);
    Some(file)
}

/// Round 178 (memory audit): the runtime directory is RAM, and every palette (each new wallpaper) left a full set
/// of icon files behind for good. A file still asked for gets a fresh stamp (at most hourly, a cheap metadata look);
/// beyond ICON_FILES_MAX the files nobody asked for in a day go.
const ICON_FILES_MAX: usize = 300;
const DAY: std::time::Duration = std::time::Duration::from_secs(24 * 3600);

fn freshen(file: &std::path::Path) {
    let old = std::fs::metadata(file)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age > std::time::Duration::from_secs(3600));
    if old {
        if let Ok(f) = std::fs::File::options().append(true).open(file) {
            let _ = f.set_modified(std::time::SystemTime::now());
        }
    }
}

fn prune_icon_files(dir: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let files: Vec<(std::path::PathBuf, std::time::SystemTime)> = entries
        .flatten()
        .filter_map(|e| Some((e.path(), e.metadata().ok()?.modified().ok()?)))
        .collect();
    if files.len() <= ICON_FILES_MAX {
        return;
    }
    for (path, stamp) in files {
        if stamp.elapsed().is_ok_and(|age| age > DAY) {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod theme_tests {
    #[test]
    fn a_theme_icon_takes_the_matrix_form() {
        // a small round icon with a wide transparent margin
        let mut src = tiny_skia::Pixmap::new(128, 128).unwrap();
        let mut paint = tiny_skia::Paint::default();
        paint.set_color(tiny_skia::Color::from_rgba8(220, 40, 40, 255));
        let circle = tiny_skia::PathBuilder::from_circle(64.0, 64.0, 30.0).unwrap();
        src.fill_path(&circle, &paint, tiny_skia::FillRule::Winding, tiny_skia::Transform::identity(), None);
        let png = super::in_matrix_form(&src.encode_png().unwrap(), &matrixkit_theme::Palette::default()).unwrap();
        let out = tiny_skia::Pixmap::decode_png(&png).unwrap();
        assert_eq!((out.width(), out.height()), (256, 256));
        let at = |x: u32, y: u32| out.pixel(x, y).unwrap();
        // the motif fills the tile: red in the middle and near the edge of the circle
        assert!(at(128, 128).red() > 150 && at(128, 128).green() < 120);
        assert!(at(128, 14).alpha() == 255, "the tile reaches the squircle's top");
        // the corner of the canvas stays transparent (squircle outline)
        assert_eq!(at(1, 1).alpha(), 0);
    }

    #[test]
    fn a_theme_index_names_its_app_folders_best_first() {
        let dir = std::env::temp_dir().join(format!("mk-theme-{}", std::process::id()));
        let t = dir.join("Probe");
        std::fs::create_dir_all(t.join("apps/scalable")).unwrap();
        std::fs::create_dir_all(t.join("apps/48")).unwrap();
        std::fs::create_dir_all(t.join("apps/symbolic")).unwrap();
        std::fs::write(
            t.join("index.theme"),
            "[Icon Theme]\nName=Probe\nInherits=hicolor\nDirectories=apps/48,apps/scalable,apps/symbolic,places/48\n\n[apps/48]\nSize=48\nContext=Applications\n\n[apps/scalable]\nSize=48\nContext=Applications\nType=Scalable\n\n[apps/symbolic]\nSize=16\nContext=Applications\nType=Scalable\n\n[places/48]\nSize=48\nContext=Places\n",
        )
        .unwrap();
        let dirs = super::app_dirs(&t);
        assert_eq!(dirs, vec![t.join("apps/scalable"), t.join("apps/48")]);
        std::fs::write(t.join("apps/48/firefox.png"), b"x").unwrap();
        // found through the index, not through a guess
        let home = std::env::var_os("HOME");
        std::env::set_var("HOME", &dir);
        std::fs::create_dir_all(dir.join(".icons")).unwrap();
        let _ = std::fs::remove_dir_all(dir.join(".icons/Probe"));
        std::os::unix::fs::symlink(&t, dir.join(".icons/Probe")).unwrap();
        assert_eq!(super::themed_icon_path("Probe", "firefox"), Some(dir.join(".icons/Probe/apps/48/firefox.png")));
        assert!(super::themes().iter().any(|n| n == "Probe"));
        assert_eq!(super::themes()[0], "Matrix");
        if let Some(h) = home {
            std::env::set_var("HOME", h);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod family_tests {
    /// Round 58: every app of the family draws (no panic, a picture).
    #[test]
    fn every_icon_renders() {
        let p = matrixkit_theme::Palette::default();
        for spec in super::ICONS {
            assert!(super::render_png(spec.name, &p).is_some_and(|png| png.len() > 1000), "{} zeichnet nicht", spec.name);
        }
        assert!(super::tile_png(&p).is_some());
    }
}

#[cfg(test)]
mod theme_family_tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn variants_join_their_family() {
        let all = names(&["Matrix", "Adwaita", "AdwaitaLegacy", "HighContrast", "WhiteSur", "WhiteSur-dark", "WhiteSur-light", "oxygen"]);
        let inherited = |t: &str| match t {
            "Adwaita" => names(&["AdwaitaLegacy", "hicolor"]),
            "HighContrast" => names(&["Adwaita"]),
            "AdwaitaLegacy" | "WhiteSur" | "WhiteSur-dark" | "WhiteSur-light" => names(&["hicolor"]),
            _ => Vec::new(),
        };
        let f = families_in(&all, &inherited);
        let got: Vec<(String, Vec<String>)> = f.into_iter().map(|f| (f.name, f.variants)).collect();
        assert_eq!(got, vec![
            (String::from("Matrix"), names(&["Matrix"])),
            (String::from("Adwaita"), names(&["Adwaita", "AdwaitaLegacy", "HighContrast"])),
            (String::from("WhiteSur"), names(&["WhiteSur", "WhiteSur-dark", "WhiteSur-light"])),
            (String::from("oxygen"), names(&["oxygen"])),
        ]);
    }

    #[test]
    fn variant_names_read_well() {
        assert_eq!(variant_label("WhiteSur", "WhiteSur"), "Standard");
        assert_eq!(variant_label("WhiteSur", "WhiteSur-dark"), "Dunkel");
        assert_eq!(variant_label("Adwaita", "AdwaitaLegacy"), "Legacy");
        assert_eq!(variant_label("Adwaita", "HighContrast"), "Hoher Kontrast");
    }
}
