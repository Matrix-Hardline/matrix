//! The window house (iced replacement, camp 17 —
//! GRABUNG-ICED-ABLOESUNG.md): the iced-free RUNTIME as a
//! kit crate. Born in First Light (stage 2, then grown camp by
//! camp), extracted here so that EVERY app can move over: an
//! app is now a `Client` — ground, tree and answers to input;
//! everything else (Wayland registry, layer surface, buffer
//! dance with release tracking, diff and damage computation,
//! z-order compose, input decoding, fleet surfaces) lives
//! HERE.
//!
//! The contracts are those of the dig: view() is a pure
//! function of the model, the retained scene is the state tree,
//! only the diff gets painted, texts/pictures/curves are citizens
//! with a home rule, state lives exactly as long as its widget
//! (Keep), and rest costs nothing.

use std::os::fd::{AsFd, AsRawFd, FromRawFd};
use wayland_client::protocol::{
    wl_buffer, wl_callback, wl_compositor, wl_data_device, wl_data_device_manager, wl_data_offer,
    wl_data_source, wl_keyboard, wl_output, wl_pointer, wl_registry, wl_seat, wl_shm, wl_shm_pool,
    wl_region, wl_subcompositor, wl_subsurface, wl_surface, wl_touch,
};
use wayland_client::{
    delegate_noop, event_created_child, Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_protocols::wp::text_input::zv3::client::{
    zwp_text_input_manager_v3, zwp_text_input_v3,
};
use wayland_protocols::wp::viewporter::client::{wp_viewport, wp_viewporter};
use wayland_protocols::xdg::decoration::zv1::client::{
    zxdg_decoration_manager_v1, zxdg_toplevel_decoration_v1,
};
use wayland_protocols::ext::session_lock::v1::client::{
    ext_session_lock_manager_v1, ext_session_lock_surface_v1, ext_session_lock_v1,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use matrixkit_image::ImageAtlas;
use matrixkit_text::TextEngine;
use matrixkit_theme::scene::{coalesce, diff, hit, Keep, Placed, Rect};

pub mod fleet;
pub mod screencopy;

/// The CLIPBOARD (camp 88): copy and paste.
///
/// It is deliberately a house service and not a client contract: the
/// Wayland clipboard lives on the seat, not on the surface, and a
/// `&mut self` in the client has no handle on the house. So it sits
/// as a mailbox between the two — the app drops in, the house
/// delivers; the house fills, the app reads.
pub mod board {
    use std::sync::Mutex;

    static OUTGOING: Mutex<Option<String>> = Mutex::new(None);
    static HELD: Mutex<Option<String>> = Mutex::new(None);

    fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        // A poisoned lock must not cripple the clipboard:
        // it carries text, no state that could be half valid.
        m.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// COPY: the text wants onto the clipboard. The house picks it up
    /// on the next loop pass and offers it to the seat.
    pub fn write(text: impl Into<String>) {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        // Hold it ourselves right away too: whoever copies and pastes
        // right after should not wait for the detour via the compositor.
        *lock(&HELD) = Some(text.clone());
        *lock(&OUTGOING) = Some(text);
    }

    /// PASTE: whatever was last on the clipboard (filled by the house).
    pub fn read() -> Option<String> {
        lock(&HELD).clone()
    }

    pub(crate) fn take_outgoing() -> Option<String> {
        lock(&OUTGOING).take()
    }

    pub(crate) fn set_held(text: Option<String>) {
        *lock(&HELD) = text;
    }
}

/// A text of the scene: what, how, where — and its HOME, the
/// hull of the element it belongs to. If it changes, the whole
/// home becomes damage (glyphs paint freely, never clipped —
/// unless the view box says so).
#[derive(Clone, PartialEq)]
pub struct Text {
    pub s: String,
    pub px: f32,
    pub weight: matrixkit_theme::typo::Weight,
    pub family: &'static str,
    pub color: matrixkit_theme::Rgba,
    pub x: f32,
    pub y: f32,
    pub home: Rect,
    /// The place in the tree (Placed index): painting goes in Z ORDER.
    pub anchor: usize,
    /// The view box of the scroll: paint offset, clip here.
    pub clip: Option<Rect>,
}

/// LIVE GLASS (round 125, family `lens`): a picture registered here is a
/// LENS MAP (`matrixkit_scene::glass_clock::lens::map`) — the GPU draws it
/// as the desktop's glass over `backdrop`, a stretched picture of the same
/// scene, sampled every frame where the lens is NOW (moving glass bends
/// what lies behind it, like niri's glass). The CPU path skips it.
pub mod lens {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    #[derive(Clone, Debug, PartialEq)]
    pub struct Lens {
        /// The stretched picture behind (its path in the same scene).
        pub backdrop: PathBuf,
        /// Bend in the rim band (logical px), as glass.kdl `glass-bend`.
        pub bend: f32,
        /// Legibility scrim on bright ground (glass.kdl `glass-scrim`).
        pub scrim: f32,
        /// Extra rim light (0 = the desktop's).
        pub rim: f32,
        /// Darkening of the backdrop under the glass (a veil over it).
        pub veil: f32,
        /// A light body (0 = clear).
        pub tint: f32,
        /// The whole glass faded (1 = fully there).
        pub opacity: f32,
    }

    static LENSES: Mutex<Option<HashMap<PathBuf, Lens>>> = Mutex::new(None);

    /// Register (or update) `map` as live glass.
    pub fn set(map: &Path, lens: Lens) {
        if let Ok(mut g) = LENSES.lock() {
            g.get_or_insert_with(HashMap::new).insert(map.to_path_buf(), lens);
        }
    }

    pub fn get(map: &Path) -> Option<Lens> {
        LENSES.lock().ok()?.as_ref()?.get(map).cloned()
    }

    /// Forget the lens of `map` (its picture was replaced).
    pub fn remove(map: &Path) {
        if let Ok(mut g) = LENSES.lock() {
            if let Some(m) = g.as_mut() {
                m.remove(map);
            }
        }
    }

    /// Forget lenses whose map name starts with `prefix` (a new picture).
    pub fn forget(prefix: &str) {
        if let Ok(mut g) = LENSES.lock() {
            if let Some(m) = g.as_mut() {
                m.retain(|k, _| !k.to_string_lossy().starts_with(prefix));
            }
        }
    }
}

/// A picture of the scene: the atlas holds the pixels, the scene only
/// the box.
#[derive(Clone, PartialEq)]
pub struct Pict {
    pub path: std::path::PathBuf,
    pub box_w: u32,
    pub box_h: u32,
    pub fit: matrixkit_image::Fit,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub home: Rect,
    pub anchor: usize,
    /// A free CLIP — for pictures in a scroll area.
    /// Without it the house only clips at its own `home`, and an
    /// icon at the bottom edge of a scrolling list stuck out past
    /// it (finger test matrix-start, 17.08.2026).
    pub clip: Option<Rect>,
}

/// A vector curve of the scene: box and sample values, rasterised
/// at paint time by the vector machine.
#[derive(Clone, PartialEq)]
pub struct Curve {
    pub samples: Vec<f32>,
    /// The scale ceiling (probe: 10 % of the wall, monitor: 100 %).
    pub max: f32,
    pub tint: matrixkit_theme::Rgba,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub home: Rect,
    pub anchor: usize,
}

/// The tree of a frame: the four citizens plus the roles of the
/// hit test (Placed index → app meaning).
#[derive(Default)]
pub struct Scene {
    pub placed: Vec<Placed>,
    pub roles: Vec<Option<usize>>,
    pub texts: Vec<Text>,
    pub picts: Vec<Pict>,
    pub curves: Vec<Curve>,
}

/// SCENE BUILDER (R210 family): builds a `Scene` without index arithmetic.
/// Every citizen (text, picture) anchors on a Placed — the builder lays
/// that anchor and keeps the indices; roles ride along. Parts built once
/// can be cached as a `ScenePart` and appended later: their anchors are
/// rebased onto wherever they land (Matrix Tube's list during playback).
#[derive(Default)]
pub struct SceneBuilder {
    pub scene: Scene,
}

/// A finished scene part with anchors relative to its own start.
#[derive(Clone, Default)]
pub struct ScenePart {
    placed: Vec<matrixkit_theme::scene::Placed>,
    roles: Vec<Option<usize>>,
    texts: Vec<Text>,
    picts: Vec<Pict>,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// The next Placed index.
    pub fn len(&self) -> usize {
        self.scene.placed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scene.placed.is_empty()
    }

    /// A rectangle (`clip` = the scroll window it lives in). Its index.
    #[allow(clippy::too_many_arguments)]
    pub fn quad(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: matrixkit_theme::Rgba,
        radius: f32,
        role: Option<usize>,
        clip: Option<Rect>,
    ) -> usize {
        let i = self.len();
        self.scene.placed.push(matrixkit_theme::scene::Placed { x, y, w, h, color, radius, clip, shadow: None });
        self.scene.roles.push(role);
        i
    }

    /// A laid-out node: every Placed it makes gets `role` (and `clip`).
    /// Returns the first index and the node's size.
    pub fn node(
        &mut self,
        node: &matrixkit_theme::scene::Node,
        x: f32,
        y: f32,
        role: Option<usize>,
        clip: Option<Rect>,
    ) -> (usize, f32, f32) {
        let first = self.len();
        let mut placed = Vec::new();
        let (w, h) = matrixkit_theme::scene::lay_out(node, x, y, &mut placed);
        for mut q in placed {
            if clip.is_some() {
                q.clip = clip;
            }
            self.scene.placed.push(q);
            self.scene.roles.push(role);
        }
        (first, w, h)
    }

    /// A text at (x, y) on an anchor of its own measure. Its size.
    #[allow(clippy::too_many_arguments)]
    pub fn text(
        &mut self,
        engine: &mut TextEngine,
        s: impl Into<String>,
        px: f32,
        weight: matrixkit_theme::typo::Weight,
        family: &'static str,
        color: matrixkit_theme::Rgba,
        x: f32,
        y: f32,
        role: Option<usize>,
        clip: Option<Rect>,
    ) -> (f32, f32) {
        let s = s.into();
        let (w, h) = engine.measure_family(family, &s, px, weight);
        let clear = matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
        let anchor = self.quad(x, y, w, h, clear, 0.0, role, clip);
        let home = Rect::of(&self.scene.placed[anchor]);
        self.scene.texts.push(Text { s, px, weight, family, color, x, y, home, anchor, clip });
        (w, h)
    }

    /// A text in a style of the type scale.
    #[allow(clippy::too_many_arguments)]
    pub fn styled(
        &mut self,
        engine: &mut TextEngine,
        s: impl Into<String>,
        style: matrixkit_theme::typo::TextStyle,
        color: matrixkit_theme::Rgba,
        x: f32,
        y: f32,
        role: Option<usize>,
        clip: Option<Rect>,
    ) -> (f32, f32) {
        self.text(engine, s, style.px(), style.weight, matrixkit_text::FAMILY, color, x, y, role, clip)
    }

    /// A text anchored on an EXISTING Placed (a slot of a laid node).
    pub fn text_on(&mut self, anchor: usize, label: Text) {
        let mut t = label;
        t.anchor = anchor;
        self.scene.texts.push(t);
    }

    /// RISE (round 150): begin a new LAYER above everything laid so far, and
    /// return its anchor. The house paints a layer's quads first, then the
    /// pictures anchored in it, then its texts; a picture belongs to the
    /// layer of its ANCHOR. A slider (quads) laid after a glass card anchored
    /// on the ground lay UNDER the card — invisible (lock, 05.11.2026).
    /// Laid after `rise()`, with pictures anchored on what it returns, a
    /// sheet's quads, pictures and texts all lie above the layers before.
    ///
    /// ```ignore
    /// let top = b.rise();
    /// b.pict(Pict { anchor: top, .. });   // the glass card
    /// b.quad(..);                          // the slider: above the card? no —
    /// // a quad of the SAME layer still lies under its pictures: rise again
    /// // (or draw it as a picture) when it must lie on a picture.
    /// ```
    pub fn rise(&mut self) -> usize {
        let i = self.len();
        self.scene.placed.push(matrixkit_theme::scene::Placed {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
            color: matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
            radius: 0.0,
            clip: None,
            // An invisible shadow: what starts a layer (`starts_layer`).
            shadow: Some(matrixkit_theme::scene::Shadow { color: matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, dx: 0.0, dy: 0.0, blur: 1.0 }),
        });
        self.scene.roles.push(None);
        i
    }

    /// A picture anchored on an existing Placed.
    pub fn pict(&mut self, pict: Pict) {
        self.scene.picts.push(pict);
    }

    /// Everything built so far, as a part to cache (the builder empties).
    pub fn into_part(self) -> ScenePart {
        let Scene { placed, roles, texts, picts, .. } = self.scene;
        ScenePart { placed, roles, texts, picts }
    }

    /// Append a part laid at x = 0, moved `dx` to the right (round 172): a
    /// COLUMN — build it in its own builder as if it were the window, then
    /// set it beside another (Tube in landscape: video left, list right).
    /// Everything moves: boxes, their clips, texts, pictures and homes.
    pub fn append_shifted(&mut self, part: &ScenePart, dx: f32) {
        let shift = |r: Rect| Rect { x: r.x + dx, ..r };
        let base = self.len();
        self.scene.placed.extend(part.placed.iter().copied().map(|mut q| {
            q.x += dx;
            q.clip = q.clip.map(shift);
            q
        }));
        self.scene.roles.extend(part.roles.iter().copied());
        self.scene.texts.extend(part.texts.iter().cloned().map(|mut t| {
            t.anchor += base;
            t.x += dx;
            t.home = shift(t.home);
            t.clip = t.clip.map(shift);
            t
        }));
        self.scene.picts.extend(part.picts.iter().cloned().map(|mut q| {
            q.anchor += base;
            q.x += dx;
            q.home = shift(q.home);
            q.clip = q.clip.map(shift);
            q
        }));
    }

    /// Append a cached part: its anchors move to where it lands.
    pub fn append(&mut self, part: &ScenePart) {
        let base = self.len();
        self.scene.placed.extend(part.placed.iter().copied());
        self.scene.roles.extend(part.roles.iter().copied());
        self.scene.texts.extend(part.texts.iter().cloned().map(|mut t| {
            t.anchor += base;
            t
        }));
        self.scene.picts.extend(part.picts.iter().cloned().map(|mut q| {
            q.anchor += base;
            q
        }));
    }

    pub fn finish(self) -> Scene {
        self.scene
    }
}

/// A key, already DECODED (camp 19): the house holds the
/// xkb keymap the compositor sends and translates keycodes
/// into meaning — the app never sees raw codes. IME
/// (zwp_text_input_v3) is the documented keystone; until
/// then this is the direct key path.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Key {
    /// A character (xkb utf8, composed).
    Char(String),
    Up,
    Down,
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Enter,
    Escape,
    Tab,
    /// Page up/down, Insert — what a terminal and a reader
    /// need (camp 86).
    PageUp,
    PageDown,
    Insert,
    /// F1 … F12.
    Function(u8),
}

/// The MODIFIERS of a key (camp 86). The contract only knew
/// `shift`; a terminal needs Ctrl and Alt, otherwise there is no
/// Ctrl-C. Whoever does not need them stays with [`Client::key`] —
/// the house keeps passing only `shift` there.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

/// Camp 106: the SHAPES — the sixth citizen, without the scene
/// gaining a field. A shape is a stack of strokes (paths with
/// fill or stroke, also dashed) in a w×h box; it is
/// REGISTERED and gets a key `shape://<hash>`, which
/// the client puts into a `Pict` like a picture. The picture atlas
/// does not know the key — its resolver asks here and gets
/// the rows EXACTLY at the target size (screen scaling included, since
/// the request is in pixels). All three paint paths (SHM, scaled,
/// GPU) paint pictures — and so shapes — without a new line.
pub mod shapes {
    use matrixkit_theme::Rgba;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    #[derive(Clone, PartialEq, Debug)]
    pub enum Cmd {
        Move(f32, f32),
        Line(f32, f32),
        Quad(f32, f32, f32, f32),
        Cubic(f32, f32, f32, f32, f32, f32),
        /// Arc around (cx, cy) with radius r, start and sweep
        /// in radians; `connect` draws a line to it from the pen.
        Arc { cx: f32, cy: f32, r: f32, start: f32, sweep: f32, connect: bool },
        Circle(f32, f32, f32),
        Close,
    }

    #[derive(Clone, PartialEq, Debug)]
    pub enum Style {
        Fill(Rgba),
        Stroke { tint: Rgba, width: f32 },
        Dashed { tint: Rgba, width: f32, on: f32, off: f32, phase: f32 },
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct Op {
        pub path: Vec<Cmd>,
        pub style: Style,
    }

    /// A shape in the w×h box (logical pixels); the strokes lie in
    /// this box.
    #[derive(Clone, PartialEq, Debug, Default)]
    pub struct Shape {
        pub w: f32,
        pub h: f32,
        pub ops: Vec<Op>,
    }

    impl Shape {
        pub fn new(w: f32, h: f32) -> Self {
            Self { w, h, ops: Vec::new() }
        }

        pub fn op(mut self, path: Vec<Cmd>, style: Style) -> Self {
            self.ops.push(Op { path, style });
            self
        }

        /// The key of this shape: an FNV sum over size and strokes.
        pub fn key(&self) -> PathBuf {
            let mut f = Fnv(0xcbf2_9ce4_8422_2325);
            f.eat(self.w);
            f.eat(self.h);
            for op in &self.ops {
                for c in &op.path {
                    match c {
                        Cmd::Move(x, y) => f.eats(&[1.0, *x, *y]),
                        Cmd::Line(x, y) => f.eats(&[2.0, *x, *y]),
                        Cmd::Quad(a, b, x, y) => f.eats(&[3.0, *a, *b, *x, *y]),
                        Cmd::Cubic(a, b, c2, d, x, y) => f.eats(&[4.0, *a, *b, *c2, *d, *x, *y]),
                        Cmd::Arc { cx, cy, r, start, sweep, connect } => {
                            f.eats(&[5.0, *cx, *cy, *r, *start, *sweep, if *connect { 1.0 } else { 0.0 }])
                        }
                        Cmd::Circle(x, y, r) => f.eats(&[6.0, *x, *y, *r]),
                        Cmd::Close => f.eat(7.0),
                    }
                }
                match &op.style {
                    Style::Fill(t) => {
                        f.eat(10.0);
                        f.tint(t);
                    }
                    Style::Stroke { tint, width } => {
                        f.eat(11.0);
                        f.tint(tint);
                        f.eat(*width);
                    }
                    Style::Dashed { tint, width, on, off, phase } => {
                        f.eat(12.0);
                        f.tint(tint);
                        f.eats(&[*width, *on, *off, *phase]);
                    }
                }
            }
            PathBuf::from(format!("shape://{:016x}", f.0))
        }
    }

    /// FNV-1a over the bits of the numbers.
    struct Fnv(u64);

    impl Fnv {
        fn eat(&mut self, v: f32) {
            for b in v.to_bits().to_le_bytes() {
                self.0 ^= b as u64;
                self.0 = self.0.wrapping_mul(0x100_0000_01b3);
            }
        }

        fn eats(&mut self, vs: &[f32]) {
            for v in vs {
                self.eat(*v);
            }
        }

        fn tint(&mut self, t: &Rgba) {
            self.eats(&[t.r, t.g, t.b, t.a]);
        }
    }

    static REGISTRY: Mutex<Option<HashMap<PathBuf, Shape>>> = Mutex::new(None);
    const REGISTRY_CAP: usize = 1024;

    /// Register the shape; back comes its key for a `Pict`.
    /// Equal shapes share the key (the atlas holds ONE picture).
    pub fn register(shape: &Shape) -> PathBuf {
        let key = shape.key();
        if let Ok(mut g) = REGISTRY.lock() {
            let map = g.get_or_insert_with(HashMap::new);
            if map.len() >= REGISTRY_CAP {
                map.clear();
            }
            map.entry(key.clone()).or_insert_with(|| shape.clone());
        }
        key
    }

    pub fn is_shape_key(path: &Path) -> bool {
        path.to_str().is_some_and(|s| s.starts_with("shape://"))
    }

    /// The resolver for the atlas: rasterises the shape at the target size
    /// (pixels). None for foreign keys or unknown ones.
    pub fn resolve(path: &Path, bw: u32, bh: u32) -> Option<(u32, u32, Vec<u8>)> {
        if !is_shape_key(path) || bw == 0 || bh == 0 {
            return None;
        }
        let shape = REGISTRY.lock().ok()?.as_ref()?.get(path)?.clone();
        raster(&shape, bw, bh)
    }

    /// Rasterise the shape into bw×bh pixels (stretched from w×h).
    pub fn raster(shape: &Shape, bw: u32, bh: u32) -> Option<(u32, u32, Vec<u8>)> {
        let kx = if shape.w > 0.0 { bw as f32 / shape.w } else { 1.0 };
        let ky = if shape.h > 0.0 { bh as f32 / shape.h } else { 1.0 };
        let ks = (kx + ky) / 2.0;
        let mut cv = matrixkit_canvas::Canvas::new(bw, bh)?;
        for op in &shape.ops {
            let mut t = matrixkit_canvas::Trace::new();
            for c in &op.path {
                match *c {
                    Cmd::Move(x, y) => t.move_to(x * kx, y * ky),
                    Cmd::Line(x, y) => t.line_to(x * kx, y * ky),
                    Cmd::Quad(a, b, x, y) => t.quad_to(a * kx, b * ky, x * kx, y * ky),
                    Cmd::Cubic(a, b, c2, d, x, y) => t.cubic_to(a * kx, b * ky, c2 * kx, d * ky, x * kx, y * ky),
                    Cmd::Arc { cx, cy, r, start, sweep, connect } => t.arc(cx * kx, cy * ky, r * ks, start, sweep, connect),
                    Cmd::Circle(x, y, r) => t.circle(x * kx, y * ky, r * ks),
                    Cmd::Close => t.close(),
                }
            }
            match op.style {
                Style::Fill(tint) => cv.fill(t, tint),
                Style::Stroke { tint, width } => cv.stroke(t, tint, width * ks),
                Style::Dashed { tint, width, on, off, phase } => {
                    cv.stroke_dashed(t, tint, width * ks, &[on * ks, off * ks], phase * ks)
                }
            }
        }
        Some((bw, bh, cv.rgba()))
    }
}

/// Camp 108: the FILE DROP. When dragging in, the compositor makes
/// an offer (wl_data_offer); the house takes `text/uri-list`,
/// reads it on a thread on release and enters the paths here —
/// the loop picks them up and calls `Client::dropped`.
pub mod drops {
    use std::path::PathBuf;
    use std::sync::Mutex;

    static INCOMING: Mutex<Option<(Vec<PathBuf>, f64, f64)>> = Mutex::new(None);

    pub(crate) fn set(paths: Vec<PathBuf>, x: f64, y: f64) {
        if let Ok(mut g) = INCOMING.lock() {
            *g = Some((paths, x, y));
        }
    }

    pub(crate) fn take() -> Option<(Vec<PathBuf>, f64, f64)> {
        INCOMING.lock().ok().and_then(|mut g| g.take())
    }

    /// `text/uri-list` (RFC 2483) → local paths: one URI per line,
    /// `#` lines are comments, only `file://` counts, percent
    /// encoding is decoded.
    pub fn paths_from_uri_list(text: &str) -> Vec<PathBuf> {
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .filter_map(|l| l.strip_prefix("file://"))
            .map(|rest| {
                // file://host/path — the host (usually empty) is dropped.
                let path = match rest.find('/') {
                    Some(i) => &rest[i..],
                    None => rest,
                };
                PathBuf::from(percent_decode(path))
            })
            .collect()
    }

    fn percent_decode(s: &str) -> String {
        let b = s.as_bytes();
        let mut out = Vec::with_capacity(b.len());
        let mut i = 0;
        while i < b.len() {
            // Round 178: parse the two bytes, never slice `s` — `%ä` would cut a character (KDE writes unencoded
            // wallpaper paths) and panic.
            if b[i] == b'%' && i + 2 < b.len() {
                if let Some(v) = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    out.push(v);
                    i += 3;
                    continue;
                }
            }
            out.push(b[i]);
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn uri_lists_become_paths() {
            let p = paths_from_uri_list("# dropped\r\nfile:///home/user/Bild%201.png\r\nhttps://example.org/x\r\nfile://localhost/tmp/a.txt\r\n");
            assert_eq!(p, vec![PathBuf::from("/home/user/Bild 1.png"), PathBuf::from("/tmp/a.txt")]);
        }
    }
}

/// Where a press came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    Pointer,
    Touch,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::Pointer => "Zeiger",
            Source::Touch => "Berührung",
        }
    }
}

/// An app's answer to an event.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Refresh {
    /// Nothing to do — rest costs nothing.
    Calm,
    /// State changed: rebuild, paint the diff.
    Dirty,
    /// Ground changed (picture, size): everything anew.
    Full,
    /// The app wants to close (traffic light ×) — clean exit.
    Close,
    /// MAXIMISE and back (traffic light +, camp 92): the window
    /// only SWITCHES; HOW big a maximised window is,
    /// the compositor computes — from its work area, the
    /// exclusive zones of bar and dock and the gap of the canvas.
    /// That is exactly why not a single number stands here: the iced path
    /// asks the same place via winit, and only this way do the distance
    /// to the dock and the distance to the edge stay equal in both paths.
    /// On a layer surface the wish is meaningless and
    /// has no effect.
    Zoom,
    /// R207: the SHELF (traffic light −): on the canvas the
    /// window moves to niri's shelf surface, in a foreign host it is
    /// minimised (xdg_toplevel.set_minimized). On a layer surface
    /// no effect.
    Shelve,
    /// Repaint the FLEET (camp 35): the cards come
    /// fresh from fleet(), every surface is painted in full.
    Panes,
    /// SCROLL BLIT (camp 104): the content of a view box has
    /// moved by dy WHOLE logical pixels (dy > 0: downwards).
    /// The house shifts the already painted pixels and pays
    /// only for the uncovered strip plus everything that did not move
    /// along exactly. The hint is purely a SHORTCUT — if something
    /// does not fit (foreign citizens in the area, fractional pixels),
    /// the normal diff path computes and the picture stays correct.
    /// Two duties of the app: the ground under the area is
    /// VERTICALLY a single colour, and every scrolling citizen carries EXACTLY
    /// this area as its view box.
    Roll { x: i32, y: i32, w: i32, h: i32, dy: i32 },
    /// JUST SHOW AGAIN (camp 24): the same scene, freshly painted —
    /// for shader time (Matrix rain). On the GPU the house uses
    /// the cached conversion and does NOT call view(); on the CPU
    /// it acts like Dirty.
    Repaint,
}

/// The LOOM (camp 25, the threads camp): spin threads without
/// stopping the loop. A thread is coarse work (a
/// Mandelbrot field, a page fetch) — it runs on its own
/// std thread, and its result comes HOME as `Client::woven()`:
/// on the loop, never concurrent with the client. The client
/// stays a pure state machine; only the work moves out.
///
/// The waking: the thread writes a byte into the wake pipe, which
/// sits next to the Wayland fd in poll(2) — the loop keeps sleeping
/// its deadline sleep and still wakes at once when material
/// comes home.
pub struct Loom<W: Send + 'static> {
    tx: std::sync::mpsc::Sender<W>,
    wake: std::sync::Arc<std::os::fd::OwnedFd>,
}

impl<W: Send + 'static> Clone for Loom<W> {
    fn clone(&self) -> Self {
        Loom { tx: self.tx.clone(), wake: self.wake.clone() }
    }
}

impl<W: Send + 'static> Loom<W> {
    /// Spin a thread: the work runs alongside, the result
    /// comes back via woven(). Whoever spins several gets them
    /// in order of arrival — the app keeps the books on sequence
    /// numbers (the newest wins).
    pub fn spin<F>(&self, job: F)
    where
        F: FnOnce() -> W + Send + 'static,
    {
        let feeder = self.clone();
        std::thread::spawn(move || feeder.feed(job()));
    }

    /// Round 178 (stability audit): spin a PERIODIC job only if the last one with the same `key` has come home.
    /// A reading that hangs (a tool waiting on a dead daemon) no longer gains a new thread every pulse — the next
    /// pulse simply skips until it returns. Returns false when skipped. The key is per process.
    pub fn spin_once<F>(&self, key: &'static str, job: F) -> bool
    where
        F: FnOnce() -> W + Send + 'static,
    {
        use std::collections::HashSet;
        use std::sync::Mutex;
        static BUSY: Mutex<Option<HashSet<&'static str>>> = Mutex::new(None);
        {
            let mut busy = BUSY.lock().unwrap_or_else(|e| e.into_inner());
            if !busy.get_or_insert_with(HashSet::new).insert(key) {
                return false;
            }
        }
        // Frees the key even if the job panics.
        struct Free(&'static str);
        impl Drop for Free {
            fn drop(&mut self) {
                if let Some(set) = BUSY.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
                    set.remove(self.0);
                }
            }
        }
        let feeder = self.clone();
        std::thread::spawn(move || {
            let w = {
                let _free = Free(key);
                job()
            };
            feeder.feed(w);
        });
        true
    }

    /// Feed a finished piece from OUTSIDE (camp 26) — for
    /// event sources with their own thread (DBus services, sockets):
    /// no job, just send and wake. The material comes home as with
    /// spinning, via woven(), on the loop.
    pub fn feed(&self, w: W) {
        let _ = self.tx.send(w);
        use std::os::fd::AsRawFd;
        let byte = [1u8];
        unsafe { libc::write(self.wake.as_raw_fd(), byte.as_ptr() as *const libc::c_void, 1) };
    }
}

/// The keyboard wish of a layer surface (camp 23): palettes
/// want it EXCLUSIVELY, search fields on demand.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyboardWish {
    None,
    OnDemand,
    Exclusive,
}

/// Which layer the surface lives on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

/// Where a layer surface anchors (camp 26): the wall on all four
/// edges, panels at their corner.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Anchors {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl Anchors {
    /// All four edges — the full wall.
    pub fn all() -> Self {
        Anchors { top: true, right: true, bottom: true, left: true }
    }
}

/// How the main surface hangs on the compositor: as a layer (wall,
/// bars, overlays) or as a WINDOW (xdg-toplevel — camp 18:
/// the first app move). The window paints its head itself
/// (Frame family); niri takes care of space and focus.
pub enum Mount {
    Layer {
        /// Glass namespace of the surface.
        namespace: String,
        layer: Layer,
        /// The keyboard wish of the surface.
        keyboard: KeyboardWish,
        /// Where the surface anchors (camp 26).
        anchor: Anchors,
        /// Start size — 0 means: the axis fills up to its anchors.
        /// Panels grow afterwards via Client::extent().
        size: (u32, u32),
        /// Margin to the edge: top, right, bottom, left.
        margin: (i32, i32, i32, i32),
        /// The exclusive zone (camp 30): >0 reserves space at the
        /// anchor (the bar), 0 floats, −1 ignores foreign zones
        /// (the wall, overlays).
        exclusive: i32,
    },
    Window {
        app_id: String,
        title: String,
        /// Desired size, as long as the compositor names none.
        width: u32,
        height: u32,
    },
    /// Round 123: the SESSION LOCK (ext-session-lock-v1). The compositor
    /// shows nothing but lock surfaces until the client unlocks — and a
    /// crashed lock client leaves the session locked (a new lock client
    /// takes over). The client draws on `Client::output()`'s screen (or
    /// the first); every other screen gets a calm night surface. The
    /// session unlocks when the client closes (`Refresh::Close`) — only
    /// then; a lock the compositor refuses ends the process locked-free.
    Lock,
}

/// A ZOOM (round 61): where a pane's finished picture is shown right now,
/// in the pane's logical coordinates. While a client answers `pane_zoom`
/// with Some, the house draws the picture ONCE into a sub-surface and per
/// frame only tells the compositor this rectangle (wp_viewport destination
/// + sub-surface position) — no painting, no buffer, the compositor scales.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zoom {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// The sub-surface a zooming pane shows its picture in.
struct ZoomChild {
    surface: wl_surface::WlSurface,
    sub: wl_subsurface::WlSubsurface,
    viewport: wp_viewport::WpViewport,
    /// Round 104: the picture drawn by the fleet GPU at full scale (None =
    /// a CPU picture at scale 1 in an shm buffer).
    target: Option<matrixkit_gpu::Target>,
    /// Round 117: the content version the picture shows, and whether it
    /// stands as the pane's picture (the zoom ended, nothing repainted).
    version: Option<u64>,
    parked: bool,
}

impl ZoomChild {
    /// Tear down: the GPU target first (its swapchain lives on the surface).
    fn retire(mut self) {
        drop(self.target.take());
        self.viewport.destroy();
        self.sub.destroy();
        self.surface.destroy();
    }
}

/// WHERE ONE FLEET SURFACE HANGS when the client places it itself (R216,
/// the top bar's chips): its own anchors, margins and size, its own screen
/// and layer, keyboard and exclusive zone — instead of the fleet's flow.
/// A kept surface whose place changes is MOVED in place (margins, size,
/// anchors: a commit, no new surface); output and layer are fixed at
/// opening — a client that changes them gives the card a new signature.
#[derive(Clone, PartialEq, Debug)]
pub struct PanePlace {
    pub anchor: Anchors,
    /// top, right, bottom, left.
    pub margin: (i32, i32, i32, i32),
    /// 0 on an axis = fill between its anchors.
    pub size: (u32, u32),
    /// The screen by name (`wl_output` name, e.g. DSI-1); None = the
    /// compositor chooses.
    pub output: Option<String>,
    /// None = the main surface's layer (camp 36: islands inherit it).
    pub layer: Option<Layer>,
    pub keyboard: KeyboardWish,
    /// As `Mount::Layer::exclusive` (−1 = ignore other zones).
    pub exclusive: i32,
}

impl PanePlace {
    /// A VEIL (round 29): fills the whole output (dims, catches the tap
    /// beside a card). The keyboard wish is the decision that matters:
    /// `Exclusive` only for a modal question (a dialog that answers Enter/
    /// Esc) — over a card it swallowed every input, and the bar's name chip
    /// could not close its own card (02.11.2026). Default for cards: OnDemand.
    pub fn veil(output: Option<String>, layer: Option<Layer>, modal: bool) -> PanePlace {
        PanePlace {
            anchor: Anchors::all(),
            margin: (0, 0, 0, 0),
            size: (0, 0),
            output,
            layer,
            keyboard: if modal { KeyboardWish::Exclusive } else { KeyboardWish::OnDemand },
            exclusive: -1,
        }
    }

    /// A pane of fixed size at `anchor` with `margin` (top, right, bottom, left).
    pub fn at(anchor: Anchors, margin: (i32, i32, i32, i32), size: (u32, u32), output: Option<String>, layer: Option<Layer>) -> PanePlace {
        PanePlace { anchor, margin, size, output, layer, keyboard: KeyboardWish::None, exclusive: -1 }
    }
}

/// The few real differences between clients.
pub struct Options {
    pub mount: Mount,
    /// Camp 23: transparent main surface (Argb8888) — for
    /// palettes and veils. The ground starts FULLY TRANSPARENT;
    /// the painters carry the alpha channel along.
    pub argb: bool,
    /// Log prefix („licht" → `[licht] …`).
    pub tag: &'static str,
    /// Start offset and gap of the fleet.
    pub fleet_start: i32,
    pub fleet_gap: i32,
    /// The flow of the fleet (camp 38): downwards (control centre) or
    /// across (dock shelf).
    pub fleet_flow: fleet::Flow,
}

/// AN OVERLAY (R211 family, 02.11.2026): something the house lays over
/// any app's scene without the app knowing — Matrix Krabbler's creatures
/// walking over Tube's words, switched on from the top bar's app card. The
/// house asks it after every view, ticks it while it moves, lets it catch
/// a tap first, and hands it mailbox words the house does not know itself.
/// Implementations live where the drawing families are (`fam::Crawlers`);
/// apps opt in with `fam::run` instead of `shell::run`.
pub trait Overlay {
    /// A mailbox word (`crawl …`); true = it was ours (redraw).
    fn word(&mut self, word: &str) -> bool;
    /// Does it want frames now?
    fn moving(&self) -> bool;
    /// Lay itself over the app's scene (may also take texts out of it).
    fn lay(&mut self, engine: &mut TextEngine, scene: Scene, w: u32, h: u32) -> Scene;
    /// Time passed while moving; true = draw a new frame (an overlay may
    /// draw less often than it moves — the Krabbler draw 30 a second).
    fn glide(&mut self, dt: f32) -> bool;
    /// A press at (x, y); true = it was ours, the app does not get it.
    fn strike(&mut self, x: f32, y: f32) -> bool;
    /// The window closes — tidy what it published.
    fn closing(&mut self) {}
}

/// A STACK OF OVERLAYS (R211 family): several over one app, laid in order
/// (the first lowest). Words go to each until one takes it; a tap to the
/// topmost first.
#[derive(Default)]
pub struct Stack(pub Vec<Box<dyn Overlay>>);

impl Overlay for Stack {
    fn word(&mut self, word: &str) -> bool {
        self.0.iter_mut().any(|o| o.word(word))
    }
    fn moving(&self) -> bool {
        self.0.iter().any(|o| o.moving())
    }
    fn lay(&mut self, engine: &mut TextEngine, scene: Scene, w: u32, h: u32) -> Scene {
        self.0.iter_mut().fold(scene, |sc, o| o.lay(engine, sc, w, h))
    }
    fn glide(&mut self, dt: f32) -> bool {
        let mut draw = false;
        for o in self.0.iter_mut().filter(|o| o.moving()) {
            draw |= o.glide(dt);
        }
        draw
    }
    fn strike(&mut self, x: f32, y: f32) -> bool {
        self.0.iter_mut().rev().any(|o| o.strike(x, y))
    }
    fn closing(&mut self) {
        for o in self.0.iter_mut() {
            o.closing();
        }
    }
}

/// An app's contract with the window house.
/// The screen's scale (1, 2, 3 …) as the compositor reports it — for clients that
/// lay computed pictures in PHYSICAL pixels (put) and need to know the size for it: the painter
/// fetches a picture at (box × scale).
static SCREEN_SCALE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
pub fn screen_scale() -> u32 {
    SCREEN_SCALE.load(std::sync::atomic::Ordering::Relaxed).max(1)
}

/// FRAME METER (R210 family): how evenly an animating app really shows
/// its frames — taken, lost (produced but never shown), the largest gap,
/// gaps over 50 ms, the slowest view. One line every five seconds; this is
/// how the duo's 21.9-Hz frame clock was found (01.11.2026).
pub struct FrameMeter {
    tag: &'static str,
    since: std::time::Instant,
    last: Option<std::time::Instant>,
    last_nr: u64,
    taken: u32,
    lost: u64,
    max_gap_ms: f32,
    gaps_over_50: u32,
    max_view_ms: f32,
}

impl FrameMeter {
    pub fn new(tag: &'static str) -> Self {
        FrameMeter {
            tag,
            since: std::time::Instant::now(),
            last: None,
            last_nr: 0,
            taken: 0,
            lost: 0,
            max_gap_ms: 0.0,
            gaps_over_50: 0,
            max_view_ms: 0.0,
        }
    }

    /// A frame was taken for showing; `nr` counts up per produced frame
    /// (a jump = frames lost). A new run (`nr` smaller) starts afresh.
    pub fn frame(&mut self, nr: u64) {
        let now = std::time::Instant::now();
        if let Some(last) = self.last {
            let gap = now.duration_since(last).as_secs_f32() * 1000.0;
            self.max_gap_ms = self.max_gap_ms.max(gap);
            self.gaps_over_50 += (gap > 50.0) as u32;
        }
        if nr > self.last_nr && nr - self.last_nr < 10_000 {
            self.lost += nr - self.last_nr - 1;
        }
        self.last_nr = nr;
        self.last = Some(now);
        self.taken += 1;
    }

    /// The time one view took.
    pub fn view(&mut self, ms: f32) {
        self.max_view_ms = self.max_view_ms.max(ms);
    }

    /// Every five seconds the line (with `extra` appended), else None.
    pub fn report(&mut self, extra: &str) -> Option<String> {
        let secs = self.since.elapsed().as_secs_f32();
        if secs < 5.0 {
            return None;
        }
        let line = format!(
            "[{}] Takt 5 s: {} Bilder gezeigt ({:.1}/s), {} verpasst, grösste Lücke {:.0} ms, {} Lücken > 50 ms, langsamste Ansicht {:.1} ms{extra}",
            self.tag, self.taken, self.taken as f32 / secs, self.lost, self.max_gap_ms, self.gaps_over_50, self.max_view_ms
        );
        *self = FrameMeter { last: self.last, last_nr: self.last_nr, ..FrameMeter::new(self.tag) };
        Some(line)
    }
}

pub trait Client {
    /// The payload of a fleet surface.
    type Card: Clone;

    /// What a spun thread brings home (camp 25). Whoever spins no
    /// threads: `type Work = ();`.
    type Work: Send + 'static;

    /// Once per font machine: load fonts (symbols …).
    fn dress(&mut self, _engine: &mut TextEngine) {}

    /// Wind the warp (camp 25): once at start the house hands
    /// over the loom. Whoever has background work keeps a
    /// clone and spins with it.
    fn warp(&mut self, _loom: &Loom<Self::Work>) {}

    /// A spun thread came home — on the loop, never
    /// concurrent with the client. Order = arrival; the app keeps
    /// the books on sequence numbers.
    fn woven(&mut self, _work: Self::Work) -> Refresh {
        Refresh::Calm
    }

    /// The GROUND under the scene (the wall) — only called on a
    /// full repaint; the buffer arrives black.
    fn base(&mut self, _atlas: &mut ImageAtlas, _buf: &mut [u8], _w: u32, _h: u32) {}

    /// The tree from the model — a pure function. An empty scene
    /// is allowed (pure wall client). The ATLAS comes along
    /// (camp 22): whoever lays computed pictures (put) lays them here.
    fn view(
        &mut self,
        engine: &mut TextEngine,
        keep: &mut Keep<f32>,
        atlas: &mut ImageAtlas,
        w: u32,
        h: u32,
    ) -> Scene;

    /// A press on the scene: the house has already done the hit
    /// test — here comes the ROLE (or None for the field).
    fn strike(&mut self, role: Option<usize>, x: f64, y: f64, source: Source) -> Refresh;

    /// Pointer motion (the app's hover care). Rests while a
    /// press is held — then travel() speaks.
    fn motion(&mut self, _x: f64, _y: f64) -> Refresh {
        Refresh::Calm
    }

    /// The pointer LEAVES the surface (camp 91).
    ///
    /// Without this message every hover state gets stuck: the
    /// magnifier of the window bar stayed big and the traffic-light glyphs
    /// stayed visible once the pointer left the window —
    /// because `motion()` simply stopped coming. A hover without a
    /// counterpart is not a hover but a switch.
    fn depart(&mut self) -> Refresh {
        Refresh::Calm
    }

    /// The DRAG (camp 24): motion while the press is held —
    /// from strike() to release(). Dragging, moving,
    /// sliders live here.
    fn travel(&mut self, _x: f64, _y: f64, _source: Source) -> Refresh {
        Refresh::Calm
    }

    /// The RELEASE (camp 24): the end of a press. The role is
    /// the hit at the release point — None means: released next to
    /// everything (that is an answer too; a drag may end in
    /// empty space).
    fn release(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        Refresh::Calm
    }

    /// The wheel axis at point (x, y).
    /// Camp 106: the SECOND BUTTON (right click) — context menus, without
    /// `Source` having to carry a button. Default: rest.
    /// Round 161: by default a right click IS a long press (owner: «It
    /// should be a long tap») — every app with a hold menu answers the
    /// mouse too. An app with its own right click overrides this.
    fn secondary(&mut self, role: Option<usize>, x: f64, y: f64) -> Refresh {
        self.hold(role, x, y, Source::Pointer)
    }

    /// Camp 112: the PINCH — two fingers on the surface. (x, y) is
    /// the middle between them, `factor` the ratio of the distance
    /// to the previous event (1.0 at the start). The house ends the
    /// running drag of the first finger (first pinch, then release),
    /// the app zooms or leaves it. Default: rest.
    fn pinch(&mut self, _x: f64, _y: f64, _factor: f64) -> Refresh {
        Refresh::Calm
    }

    /// Camp 106: the LONG PRESS — pointer or finger rest for
    /// touch::LONG_PRESS_SECS without a drag (the house keeps the beat, the
    /// app gets the hit under the press). Default: rest.
    fn hold(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        Refresh::Calm
    }

    /// Camp 108: FILES drop onto the surface (dragged from files, stick,
    /// browser). The paths are local, the point is the release spot.
    /// Default: rest.
    fn dropped(&mut self, _paths: Vec<std::path::PathBuf>, _x: f64, _y: f64) -> Refresh {
        Refresh::Calm
    }

    fn axis(&mut self, _x: f64, _y: f64, _value: f64, _keep: &mut Keep<f32>) -> Refresh {
        Refresh::Calm
    }

    /// A decoded key (only with keyboard focus). `shift`
    /// is the EFFECTIVE shift state — for selection ranges.
    fn key(&mut self, _key: Key, _shift: bool) -> Refresh {
        Refresh::Calm
    }

    /// The same key WITH all modifiers (camp 86). Whoever
    /// overrides it gets Ctrl and Alt; whoever does not still sees
    /// only [`Client::key`] — the default passes through.
    fn key_with(&mut self, key: Key, mods: Mods) -> Refresh {
        self.key(key, mods.shift)
    }

    /// The 500 ms beat (cursor blinking, small clocks). Always
    /// runs; whoever blinks nothing answers Calm.
    fn beat(&mut self) -> Refresh {
        Refresh::Calm
    }

    /// R210 duo grammar: the top bar's app menu asks for the app's
    /// settings — its root pane (`fam::RootPane`). Nothing by default.
    fn root(&mut self) -> Refresh {
        Refresh::Calm
    }

    /// The window got (true) or lost (false) the keyboard focus — another
    /// window came to the front (03.11.2026: a video stage steps back).
    fn focus(&mut self, _on: bool) -> Refresh {
        Refresh::Calm
    }

    /// The window is about to close (×, the bar's menu or the compositor):
    /// last chance to tidy up — hand sound to the background, save.
    fn closing(&mut self) {}

    /// R209: this client TYPES (terminal, editors). On a finger device the
    /// house then holds the on-screen keyboard open while the window has
    /// the keyboard focus — the iced fields do it with their own beat.
    fn wants_keyboard(&self) -> bool {
        false
    }

    /// Is the app SPRINGING right now? (Camp 32: motion.) As long
    /// as yes, the house calls glide() at frame rate (~60 Hz); when
    /// everything rests, rest costs nothing again — the beat sleeps.
    fn animating(&self) -> bool {
        false
    }

    /// Round 137: the main surface is not seen right now (it lies on a
    /// panel switched off — the folded duo's back panel). The house then
    /// neither paints it nor waits for its frames: those never come, and
    /// the 100-ms safety line held every pane to ~9 frames a second.
    fn main_hidden(&self) -> bool {
        false
    }

    /// The MATRIX RAIN (GPU painting only): Some([r, g, b, strength])
    /// lets the digital rain fall behind the scene — above the
    /// ground, under all layers. None = dry.
    fn rain(&self) -> Option<[f32; 4]> {
        None
    }

    /// The frame beat (only while animating): dt in seconds since
    /// the last step — food for mk::motion::Spring::tick.
    fn glide(&mut self, _dt: f32) -> Refresh {
        Refresh::Calm
    }

    /// Does the app want TEXT right now? (A focused field.) The house
    /// reports it to the IME line (zwp_text_input_v3).
    fn wants_text(&self) -> bool {
        false
    }

    /// PASS-THROUGH (camp 47): a daemon surface without
    /// content must not eat input — if it says yes, the house
    /// sets an EMPTY input region, and clicks fall through
    /// to the session. The context service lives on this.
    fn passthrough(&self) -> bool {
        false
    }

    /// The window is MAXIMISED — or free again (camp 92).
    ///
    /// Reported is what the compositor says in the configure, not
    /// what the app wished: a wish can be refused,
    /// and a window can also be maximised from outside (key,
    /// tiling grip). Whoever carries the window key needs the
    /// message: rolling up and being maximised exclude each other
    /// — the desired size of a maximised window does not apply.
    fn zoomed(&mut self, _on: bool) {}

    /// The DESIRED SIZE of a layer surface (camp 26): panels
    /// grow with their content. None: as mounted (the wall
    /// never asks). The house checks after every paint; the new size
    /// comes back as a configure — an intermediate picture at the old size
    /// is allowed and brief.
    fn extent(&mut self) -> Option<(u32, u32)> {
        None
    }

    /// R207: the HEAD HEIGHT of a window (logical) — a press there
    /// that hits no button becomes a move by the
    /// compositor (xdg_toplevel.move). Foreign desktops know no
    /// Mod+drag; without this the window stays put. 0 = no head.
    fn headline(&self) -> f32 {
        0.0
    }

    /// R207: the CORNER RADIUS of the window (logical). The house cuts
    /// the four corners in the buffer (CPU) or with the corner mask (GPU)
    /// transparent — foreign compositors round nothing. 0 = square.
    fn corners(&self) -> f32 {
        0.0
    }

    /// R207: the SMALLEST window size (logical). In a foreign host the
    /// house reports it to the compositor (xdg_toplevel.set_min_size), so that
    /// nobody drags the window below a smartphone display — there
    /// every layout stops holding. On the canvas the compositor
    /// decides. (0, 0) = no lower bound.
    fn least(&self) -> (u32, u32) {
        matrixkit_theme::form::least()
    }

    /// WHERE a layer surface sits (camp 82): the distance to the
    /// anchored edges, in the same order as when mounting
    /// (top, right, bottom, left). None: as mounted.
    ///
    /// Together with [`Client::extent`] a client can put a
    /// surface at a FREE spot — instead of holding a
    /// screen-filling one and painting inside it. The
    /// context menu needs exactly this: the compositor lays its glass
    /// only under a MENU-SIZED surface, a screen-filling one
    /// would get it across the whole width (finger test 17.08.2026).
    fn offset(&mut self) -> Option<(i32, i32, i32, i32)> {
        None
    }

    /// The 2 s pulse: picture change, sun position, whatever the app wants.
    fn pulse(&mut self) -> Refresh {
        Refresh::Calm
    }

    /// After every paint: the fleet plan from the model.
    fn fleet(&mut self) -> Vec<fleet::Spec<Self::Card>> {
        Vec::new()
    }

    /// Paint a fleet surface (the buffer arrives black).
    fn pane(&mut self, _card: &Self::Card, _engine: &mut TextEngine, _canvas: &mut [u8], _w: u32, _h: u32) {}

    /// The FLEET LISTENS (camp 35): a press on a
    /// fleet surface — the card says which; the coordinates
    /// are surface-local. Whoever changes the state answers
    /// Refresh::Panes (and Dirty if the main surface
    /// has to come along too).
    /// Round 28: a right click on a fleet surface (a tray icon's menu, the
    /// media chip's «next»). Default: as before — a strike.
    fn pane_secondary(&mut self, card: &Self::Card, x: f64, y: f64) -> Refresh {
        self.pane_strike(card, x, y, Source::Pointer)
    }

    /// Round 163: the WHEEL over a fleet surface (a bar card, a popup) —
    /// `value` already scaled for a wheel (mk::input). Default: rest.
    fn pane_axis(&mut self, _card: &Self::Card, _x: f64, _y: f64, _value: f64) -> Refresh {
        Refresh::Calm
    }

    /// Round 28: a long press on a fleet surface (0.5 s, finger still).
    fn pane_hold(&mut self, _card: &Self::Card, _x: f64, _y: f64, _source: Source) -> Refresh {
        Refresh::Calm
    }

    fn pane_strike(&mut self, _card: &Self::Card, _x: f64, _y: f64, _source: Source) -> Refresh {
        Refresh::Calm
    }

    /// The drag on a fleet surface (between pane_strike and
    /// pane_release) — sliders on islands live here.
    fn pane_travel(&mut self, _card: &Self::Card, _x: f64, _y: f64, _source: Source) -> Refresh {
        Refresh::Calm
    }

    /// The release on a fleet surface.
    fn pane_release(&mut self, _card: &Self::Card, _x: f64, _y: f64, _source: Source) -> Refresh {
        Refresh::Calm
    }

    /// R216: the screen of the MAIN layer surface by name. Default: the
    /// process's `--output NAME` (`output_wish`). A shell part that knows
    /// its screen from the settings (the top bar: `bar-output`, DSI-2 on a
    /// duo) says it here — its watchdog starts it without arguments.
    fn output(&self) -> Option<String> {
        output_wish()
    }

    /// R216: this fleet surface PLACES ITSELF (`PanePlace`) — anchors,
    /// margins, size, screen, layer. None (default): the fleet's flow
    /// places it, as before. Asked when a surface opens and after every
    /// fleet sync (a changed place moves the surface in place).
    fn pane_place(&mut self, _card: &Self::Card) -> Option<PanePlace> {
        None
    }

    /// R216: this fleet surface is a SCENE — the house composes it at the
    /// surface's own buffer scale (crisp text on a 2× panel), like the main
    /// surface. None (default): `pane()` paints raw logical pixels. `w`/`h`
    /// are the configured logical size; hit-testing stays with the client
    /// (`pane_strike` gets surface-local logical coordinates). The atlas
    /// takes computed pictures (`put`: tray icons).
    fn pane_scene(&mut self, _card: &Self::Card, _engine: &mut TextEngine, _atlas: &mut ImageAtlas, _w: u32, _h: u32) -> Option<Scene> {
        None
    }

    /// R216: does this fleet surface need a new picture on `Refresh::Panes`?
    /// Default: always (every surface is repainted). A client with large
    /// surfaces (a card over a window) and small ones that tick (a clock
    /// chip) answers false for the ones that did not change. A surface is
    /// always painted when it is (re)configured.
    fn pane_stale(&mut self, _card: &Self::Card) -> bool {
        true
    }

    /// Round 60: this pane is in flight (a card growing out of its chip)
    /// — compose it at buffer scale 1, not the output's (2.75 on the Duo:
    /// 1902×1212 pixels cost up to 37 ms a frame on the CPU, ~7× fewer
    /// now). A moving flat body looks the same; the sharp picture comes
    /// with the first frame that answers false.
    fn pane_moving(&self, _card: &Self::Card) -> bool {
        false
    }

    /// Round 117: the content version of a pane (bumped by the client when
    /// what it SHOWS changes, not when it only moves). Some → a zoom's
    /// finished GPU picture stays as the pane's picture when the zoom ends
    /// (no full repaint: the end of a card's growth hitched 50–155 ms on
    /// the Duo) and serves the closing zoom; None (default) → always repaint.
    fn pane_version(&self, _card: &Self::Card) -> Option<u64> {
        None
    }

    /// Round 117: build the fleet's GPU device ahead, a few seconds after
    /// start while idle — its first use cost ~0.6 s on the Duo (the first
    /// card after a restart hung that long before it moved).
    fn warm_gpu(&self) -> bool {
        false
    }

    /// Round 107: paint this pane on the fleet GPU even when it is small
    /// (a pane that moves every frame: the bar's notification island).
    /// Large panes (cards) go there anyway.
    fn pane_gpu(&self, _card: &Self::Card) -> bool {
        false
    }

    /// Round 61: show this pane's finished picture zoomed into the given
    /// rectangle (a card growing out of its chip). `w`, `h`: the pane's
    /// logical size. Some → the house zooms instead of painting; the
    /// first frame that answers None paints the pane normally again.
    fn pane_zoom(&mut self, _card: &Self::Card, _w: u32, _h: u32) -> Option<Zoom> {
        None
    }

    /// Self-measurement: the price of the last paint in % of the
    /// wall (full repaint = 100).
    fn painted(&mut self, _percent: f32) {}
}

/// ONE reused SHM buffer; `stale` keeps the books on which
/// patches of the canvas changed since this buffer
/// was last attached.
struct Slot {
    buffer: wl_buffer::WlBuffer,
    map: memmap2::MmapMut,
    stale: Vec<Rect>,
    busy: bool,
}

/// A fleet surface: its own layer surface, fixed size, painted
/// in full once.
struct Pane<K> {
    id: u32,
    surface: wl_surface::WlSurface,
    /// The pane's role: a layer surface — or, under the session lock
    /// (round 124), a lock surface on its output (`lock`).
    layer: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    lock: Option<ext_session_lock_surface_v1::ExtSessionLockSurfaceV1>,
    shell: fleet::Shell,
    card: K,
    /// Only after the first configure may a buffer be attached
    /// (race find camp 36: a repaint before the configure
    /// is a protocol error — Broken pipe).
    ready: bool,
    /// R216: where it hangs when the client places it (`pane_place`).
    place: Option<PanePlace>,
    /// R216: the configured logical size (0 = not yet configured; the
    /// shell's size counts then).
    size: (u32, u32),
    /// R216: the surface's own preferred buffer scale (scene panes).
    scale: i32,
    /// Round 51: fingerprint of the picture last attached (0 = none). An
    /// unchanged picture is not sent again — `pane_stale` defaults to
    /// true, and the dock sent every pane anew on every change.
    shown: u64,
    /// Round 60 (pane beat probe): when the last picture went out.
    drawn_at: Option<std::time::Instant>,
    /// Round 61: the sub-surface while the pane zooms.
    zoom: Option<ZoomChild>,
    /// Round 86: the last scene with its buffer size and scale, and the
    /// picture it made — the next paint redraws only what changed (a
    /// switch in the control centre repainted 2.9 million pixels on the
    /// Duo, 107–248 ms: «the card and the control centre lag»).
    last: Option<(Scene, (u32, u32, u32))>,
    canvas: Vec<u8>,
    /// Round 91: the scroll's sub-pixel remainder (buffer pixels) — the
    /// blit moves whole pixels, the rest is carried so nothing drifts.
    roll_err: f32,
    /// Round 92: kept buffers (up to three, in turn) with what each still
    /// lacks — a picture copies only that, not the whole 9 MB pane.
    bufs: Vec<Slot>,
    buf_dims: (u32, u32),
    /// Round 94: the pane's own surface on the fleet GPU (large panes).
    target: Option<matrixkit_gpu::Target>,
}

/// THE BEAT PROBE (round 54, 03.11.2026 — Tube skipped a frame every few
/// seconds while the decoder was ahead: the window fetched no picture for
/// > 40 ms). Remembers when the last picture was drawn, how long drawing
/// took, and when the compositor released the frame before; a gap is
/// split into «drawing», «waiting for the compositor» and «the rest»
/// (the house's own loop). At most one line per second.
#[derive(Default)]
struct TickProbe {
    last_start: Option<std::time::Instant>,
    last_ms: f32,
    done_at: Option<std::time::Instant>,
    logged_at: Option<std::time::Instant>,
    /// The GPU painter's split of the last picture (acquire, upload,
    /// build+submit, present — ms), zero on the CPU path.
    phases: [f32; 4],
    prev_phases: [f32; 4],
    build: [f32; 4],
    prev_build: [f32; 4],
}

impl TickProbe {
    fn drawn(
        &mut self,
        moving: bool,
        start: std::time::Instant,
        asked_at: Option<std::time::Instant>,
    ) -> Option<String> {
        self.drawn_at(moving, start, std::time::Instant::now(), asked_at)
    }

    /// `drawn` with the end of drawing given (round 55: testable).
    fn drawn_at(
        &mut self,
        moving: bool,
        start: std::time::Instant,
        end: std::time::Instant,
        asked_at: Option<std::time::Instant>,
    ) -> Option<String> {
        let took = end.duration_since(start).as_secs_f32() * 1000.0;
        let pp = std::mem::replace(&mut self.prev_phases, self.phases);
        let pb = std::mem::replace(&mut self.prev_build, self.build);
        let before = self.last_start.replace(start);
        let prev_ms = std::mem::replace(&mut self.last_ms, took);
        if !moving {
            self.last_start = None;
            return None;
        }
        let gap = start.duration_since(before?).as_secs_f32() * 1000.0;
        if gap <= 45.0 || self.logged_at.is_some_and(|t| end.duration_since(t).as_secs_f32() < 1.0) {
            return None;
        }
        self.logged_at = Some(end);
        // The compositor's part: from the previous commit to its «done».
        let wait = match (asked_at, self.done_at) {
            (Some(a), Some(d)) if d >= a => d.duration_since(a).as_secs_f32() * 1000.0,
            _ => -1.0,
        };
        let rest = gap - prev_ms - wait.max(0.0);
        Some(format!(
            "Takt-Lücke {gap:.0} ms: Zeichnen davor {prev_ms:.0} ms, Warten auf den Compositor {}, Rest (Schleife/App) {rest:.0} ms, dieses Bild {took:.0} ms",
            if wait >= 0.0 { format!("{wait:.0} ms") } else { String::from("–") }
        ) + &format!(" | Zeichnen davor = Puffer holen {:.1} + Hochladen {:.1} + Aufbereiten {:.1} + Übergeben {:.1} ms; Aufbereiten = Vorlauf {:.1} + Schrift {:.1} + Malbefehle {:.1} + Abschicken {:.1} ms", pp[0], pp[1], pp[2], pp[3], pb[0], pb[1], pb[2], pb[3]))
    }
}

struct House<C: Client> {
    client: C,
    /// R211: laid over the client's scene (`Overlay`).
    overlay: Option<Box<dyn Overlay>>,
    tag: &'static str,
    argb: bool,
    compositor: Option<wl_compositor::WlCompositor>,
    /// Round 61: sub-surfaces for zooming panes (`Client::pane_zoom`).
    subcompositor: Option<wl_subcompositor::WlSubcompositor>,
    shm: Option<wl_shm::WlShm>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    surface: Option<wl_surface::WlSurface>,
    layer: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    /// Round 123: the session lock (`Mount::Lock`).
    lock_manager: Option<ext_session_lock_manager_v1::ExtSessionLockManagerV1>,
    session_lock: Option<ext_session_lock_v1::ExtSessionLockV1>,
    lock_surface: Option<ext_session_lock_surface_v1::ExtSessionLockSurfaceV1>,
    /// The other screens' lock surfaces (a night colour, nothing drawn).
    /// …with the buffer they show (replaced on every configure — a
    /// rotation while locked; the old one is destroyed, round 127).
    lock_others: Vec<(ext_session_lock_surface_v1::ExtSessionLockSurfaceV1, wl_surface::WlSurface, Option<wl_buffer::WlBuffer>)>,
    /// The compositor refused the lock (`finished`) — never unlock then.
    lock_refused: bool,
    /// Round 136: when the lock was asked for, and whether the compositor
    /// confirmed it (`locked`). Unconfirmed after a few seconds = give up:
    /// a half-made lock blocked every later one (niri waited for a panel
    /// that was switched off, 05.11.2026).
    lock_asked: Option<std::time::Instant>,
    lock_confirmed: bool,
    wm_base: Option<xdg_wm_base::XdgWmBase>,
    xdg: Option<xdg_surface::XdgSurface>,
    toplevel: Option<xdg_toplevel::XdgToplevel>,
    /// Desired size of the window as long as niri names none.
    want: (u32, u32),
    /// Is the window MAXIMISED right now? (Camp 92 — from the
    /// state field of the toplevel configure, not from the wish.)
    maximized: bool,
    /// A pending toggle wish of traffic light +, redeemed after
    /// painting — like the desired margin and the desired size.
    zoom_wish: Option<bool>,
    /// R207: the shelf is wished (traffic light −).
    shelve_wish: bool,
    size: (u32, u32),
    dirty: bool,
    full: bool,
    /// The accumulated scroll hint (area, sum dy) since the
    /// last paint (camp 104).
    roll_hint: Option<(Rect, i32)>,
    /// Two scrolls with DIFFERENT areas in one beat: no more
    /// shortcut until the next paint.
    roll_spoiled: bool,
    /// The GPU painting (MATRIX_GPU=1, stage 1: clear colour) — None
    /// while the switch is off or the seam failed.
    gpu: Option<matrixkit_gpu::Gpu>,
    /// Round 94: the fleet's GPU — ONE device for all large panes (cards),
    /// living on an unmapped helper surface that is never shown; each pane
    /// paints into a `Target` of its own. `pane_gpu_failed`: CPU for good.
    pane_gpu: Option<(matrixkit_gpu::Gpu, wl_surface::WlSurface)>,
    pane_gpu_failed: bool,
    /// Round 98: beats counted (the purge check runs every tenth).
    beats_seen: u32,
    gpu_tried: bool,
    /// The raw libwayland pointer of the connection — half the seam.
    display_ptr: *mut std::ffi::c_void,
    /// The window ground for the GPU (stage 5): client.base() as
    /// RGBA — rebuilt on full/resize, otherwise just shown.
    gpu_ground: Vec<u8>,
    gpu_ground_gen: u64,
    /// Size of the last BUILT ground + time — during a
    /// live resize it is repainted at most every 120 ms, in between
    /// the GPU shows the old ground stretched (camp 12).
    gpu_ground_size: (u32, u32),
    gpu_ground_at: Option<std::time::Instant>,
    /// The hour of birth — the clock of the rain.
    born: std::time::Instant,
    /// Show-again-only round (camp 24): scene unchanged.
    repaint_only: bool,
    /// The cached GPU conversion of the last scene — for
    /// repaint rounds without rebuilding view().
    gpu_cache: Option<GpuSceneCache>,
    /// The FRAME CALLBACK BEAT (camp 12): if a callback is pending,
    /// the animation loop waits for the compositor — otherwise
    /// the 16 ms poll drifts against vsync (micro-stutter).
    frame_pending: bool,
    frame_at: Option<std::time::Instant>,
    closed: bool,
    pointer_dev: Option<wl_pointer::WlPointer>,
    touch_dev: Option<wl_touch::WlTouch>,
    keyboard_dev: Option<wl_keyboard::WlKeyboard>,
    xkb: Option<xkbcommon::xkb::State>,
    seat: Option<wl_seat::WlSeat>,
    /// R207: the decoration manager — we paint the frame ourselves
    /// (head with traffic lights), foreign compositors should not put a second
    /// frame and a square shadow underneath.
    deco_manager: Option<zxdg_decoration_manager_v1::ZxdgDecorationManagerV1>,
    /// R207: the VIEWPORTER — in a foreign host the window tells the
    /// compositor its logical target size exactly (KWin, fractional
    /// scaling): without it fine stripes of the old spot remain
    /// when moving, because buffer and window size do not fall on
    /// the same pixel.
    viewporter: Option<wp_viewporter::WpViewporter>,
    /// The screens and their names (wl_output v4 `name`) — a layer surface
    /// started with `--output NAME` lands on exactly that one (R216: the
    /// duo's shell parts each belong on one panel).
    outputs: Vec<(wl_output::WlOutput, Option<String>)>,
    /// Round 76: the registry name of every bound output — so a screen niri
    /// withdraws (a suspend probe, a reconnect) leaves the list. Before, the
    /// dead output stayed FIRST and new panes landed on it: drawn, but deaf
    /// to touch (the dock and the context menu after the deep-sleep probes).
    output_globals: Vec<(u32, wl_output::WlOutput)>,
    viewport: Option<wp_viewport::WpViewport>,
    /// R207: the wallpaper of the foreign desktop (path, mtime) —
    /// if it changes, the palette grows anew from its seed.
    seed_watch: Option<(std::path::PathBuf, std::time::SystemTime)>,
    seed_ticks: u32,
    /// Round 52: the house watches itself for leaks (fds, RSS).
    leak_probe: matrixkit_theme::process::LeakProbe,
    leak_ticks: u32,
    /// Round 54 — the beat probe: while the client moves, a gap > 45 ms
    /// between two pictures is logged with what it was spent on.
    tick: TickProbe,
    /// R207: the corner radius of the window (logical, 0 = square) — from the
    /// client (`corners()`), cut into alpha by the house.
    corners: f32,
    /// The clipboard (camp 88): manager, device on the seat, and the
    /// source we are currently offering ourselves.
    data_manager: Option<wl_data_device_manager::WlDataDeviceManager>,
    data_device: Option<wl_data_device::WlDataDevice>,
    data_source: Option<wl_data_source::WlDataSource>,
    /// The OFFER of the other side under construction: first comes the object,
    /// then its formats, finally selection() says that it applies.
    offer_building: Option<(wl_data_offer::WlDataOffer, Vec<String>)>,
    /// The current offer — lives until the next one comes (the
    /// reader thread hangs on it).
    offer_current: Option<wl_data_offer::WlDataOffer>,
    /// Camp 108: the offer of a running drag over the surface
    /// (with the chosen format) and where the pointer last stood.
    drop_offer: Option<(wl_data_offer::WlDataOffer, String)>,
    drop_at: (f64, f64),
    /// The last input timestamp: `set_selection` needs it,
    /// otherwise the compositor refuses the clipboard.
    serial: u32,
    /// The IME line (zwp_text_input_v3), if the compositor
    /// offers it — and whether it is switched on right now.
    text_input: Option<zwp_text_input_v3::ZwpTextInputV3>,
    ime_manager: Option<zwp_text_input_manager_v3::ZwpTextInputManagerV3>,
    text_on: bool,
    /// Key repeat: rate/delay from the compositor, the
    /// held key with its next fire time.
    repeat_rate: i32,
    repeat_delay: i32,
    held: Option<(u32, Key, bool, std::time::Instant)>,
    /// R209: does this window hold the keyboard focus (wl_keyboard
    /// enter/leave)? The on-screen keyboard follows it.
    keyboard_focus: bool,
    pointer: (f64, f64),
    /// Is the pointer button held right now? (Camp 24: between
    /// strike and release, travel speaks instead of motion.)
    pointer_held: bool,
    /// Round 161: the scroll now comes from a wheel (wl_pointer axis_source).
    wheel: bool,
    /// Round 164: the serial of the press in progress (a window move needs it).
    press_serial: u32,
    /// Round 164: the WINDOW MOVE — the window wiggles since then, and the
    /// compositor drags it (xdg_toplevel.move) since `wiggle_move`.
    wiggle: Option<std::time::Instant>,
    wiggle_move: Option<std::time::Instant>,
    /// Round 171: a FINGER after the long press — a slow start is handed to
    /// the compositor's drag, a quick one is a flick to another screen.
    fling: Option<(std::time::Instant, matrixkit_theme::input::Fling)>,
    /// The last touch point — wl_touch::Up names none.
    touch_at: (f64, f64),
    /// Camp 112: all fingers down (id, x, y); two of them are
    /// a pinch. After a pinch the remaining finger is deaf until
    /// all are lifted — otherwise the drag jumps.
    touches: Vec<(i32, f64, f64)>,
    pinch_dist: Option<f64>,
    touch_numb: bool,
    /// R209 FLICK: a finger that came down on the window head (no
    /// button) — id and start point. Released far enough away, the
    /// window goes to the neighbouring panel (iced: `flick_area`).
    flick: Option<(i32, f64, f64)>,
    /// Camp 106: the held press for the long-press verdict.
    press: Option<Press>,
    /// The last desired layer size (camp 26: panels grow).
    extent_applied: (u32, u32),
    /// Pixels per logical pixel (1 on ordinary screens).
    scale: i32,
    offset_applied: Option<(i32, i32, i32, i32)>,
    /// Which fleet surface pointer/touch are over
    /// (camp 35) — None: the main surface.
    pointer_pane: Option<u32>,
    touch_pane: Option<u32>,
    /// The fleet wants a repaint (Refresh::Panes).
    panes_dirty: bool,
    /// Is the empty input region set right now (camp 47)?
    passthrough_applied: bool,
    /// Which layer the fleet surfaces live on (owner probe
    /// camp 36: they INHERIT the layer of the main surface — an
    /// overlay veil otherwise lay ABOVE its own islands and
    /// ate every click).
    pane_layer: zwlr_layer_shell_v1::Layer,
    scene: Scene,
    keep: Keep<f32>,
    base: Vec<u8>,
    canvas: Vec<u8>,
    engine: Option<TextEngine>,
    atlas: ImageAtlas,
    fleet: fleet::Fleet<C::Card, u32>,
    panes: Vec<Pane<C::Card>>,
    pane_seq: u32,
    slots: Vec<Slot>,
    gen: u32,
    buf_size: (u32, u32),
}

/// The left mouse button, as the kernel calls it (`linux/input.h`).
const BTN_LEFT: u32 = 0x110;
const BTN_RIGHT: u32 = 0x111;
/// How far a press may wander and still count as a long press.
const HOLD_SLOP: f64 = 6.0;

/// Camp 106: a held press — since when, where, from where, and whether
/// the long press was already reported.
struct Press {
    since: std::time::Instant,
    x: f64,
    y: f64,
    source: Source,
    fired: bool,
    /// Round 28: the fleet surface under the press (None = main surface).
    pane: Option<u32>,
}

fn press_begin<C: Client + 'static>(state: &mut House<C>, x: f64, y: f64, source: Source) {
    state.press = Some(Press { since: std::time::Instant::now(), x, y, source, fired: false, pane: None });
}

/// A press on a FLEET surface (round 28): its long press goes to
/// `pane_hold` (before, only the main surface knew long presses).
fn press_begin_pane<C: Client + 'static>(state: &mut House<C>, pid: u32, x: f64, y: f64, source: Source) {
    state.press = Some(Press { since: std::time::Instant::now(), x, y, source, fired: false, pane: Some(pid) });
}

fn press_moved<C: Client + 'static>(state: &mut House<C>, x: f64, y: f64) {
    if let Some(p) = &state.press {
        if (x - p.x).abs() > HOLD_SLOP || (y - p.y).abs() > HOLD_SLOP {
            state.press = None;
        }
    }
}

/// Due? Then `hold` to the client once — with the hit
/// under the press.
fn press_check<C: Client + 'static>(state: &mut House<C>, now: std::time::Instant) {
    let due = match &state.press {
        Some(p) if !p.fired => {
            now.duration_since(p.since).as_secs_f32() >= matrixkit_theme::touch::LONG_PRESS_SECS
        }
        _ => false,
    };
    if !due {
        return;
    }
    let (x, y, source, pane) = {
        let p = state.press.as_mut().expect("geprüft");
        p.fired = true;
        (p.x, p.y, p.source, p.pane)
    };
    if let Some(pid) = pane {
        if let Some(card) = pane_card(state, pid) {
            let r = state.client.pane_hold(&card, x, y, source);
            state.refresh(r);
        }
        return;
    }
    let role = hit(&state.scene.placed, x as f32, y as f32)
        .and_then(|i| state.scene.roles.get(i).copied().flatten());
    let r = state.client.hold(role, x, y, source);
    // Round 164 (owner: «Move windows by a long tap or with
    // the mouse — then the wiggle mode, then normal again»): a long press
    // on nothing the app wants starts the WINDOW MOVE.
    let means = matrixkit_theme::input::long_press(role.is_some(), !matches!(r, Refresh::Calm), state.toplevel.is_some(), matrixkit_theme::input::window_move());
    if means == matrixkit_theme::input::LongPress::MoveWindow {
        // Round 171: a finger flicks (or drags slowly), the mouse drags.
        if source == Source::Touch && state.toplevel.is_some() {
            fling_begin(state, x, y);
            return;
        }
        if window_move(state) {
            return;
        }
    }
    state.refresh(r);
}

/// Round 164: wiggle and hand the window to the compositor's drag (the same
/// finger or button keeps it). False: no window to move (layer, lock).
fn window_move<C: Client + 'static>(state: &mut House<C>) -> bool {
    let (Some(top), Some(seat)) = (&state.toplevel, &state.seat) else { return false };
    top._move(seat, state.press_serial);
    matrixkit_theme::feel::hold();
    let now = std::time::Instant::now();
    state.wiggle = Some(now);
    state.wiggle_move = Some(now);
    state.press = None;
    state.pointer_held = false;
    eprintln!("[{}] Fenster verschieben: Wackelmodus, der Compositor zieht", state.tag);
    state.refresh(Refresh::Dirty);
    true
}

/// Round 171: the finger's long press — the window wiggles, the finger is
/// read: a slow start hands over to the compositor's drag (fling_motion),
/// a quick one flings at the release (fling_release).
fn fling_begin<C: Client + 'static>(state: &mut House<C>, x: f64, y: f64) {
    matrixkit_theme::feel::hold();
    let now = std::time::Instant::now();
    state.wiggle = Some(now);
    state.wiggle_move = None;
    state.fling = Some((now, matrixkit_theme::input::Fling::new(x, y)));
    state.flick = None;
    state.press = None;
    eprintln!("[{}] Fenster verschieben: Wackelmodus, Finger — ziehen oder spicken", state.tag);
    state.refresh(Refresh::Dirty);
}

/// The finger moved while flinging: true = taken (the app sees nothing).
fn fling_motion<C: Client + 'static>(state: &mut House<C>, x: f64, y: f64) -> bool {
    let Some((t0, f)) = state.fling.as_mut() else { return false };
    let step = f.push(t0.elapsed().as_secs_f64(), x, y);
    state.touch_at = (x, y);
    if step == matrixkit_theme::input::FlingStep::HandOver {
        state.fling = None;
        if let (Some(top), Some(seat)) = (&state.toplevel, &state.seat) {
            top._move(seat, state.press_serial);
            state.wiggle_move = Some(std::time::Instant::now());
            eprintln!("[{}] Fenster verschieben: langsam — der Compositor zieht", state.tag);
        }
    }
    true
}

/// The finger left while flinging: true = taken.
fn fling_release<C: Client + 'static>(state: &mut House<C>) -> bool {
    let Some((_, f)) = state.fling.take() else { return false };
    state.press = None;
    state.touch_pane = None;
    match f.release() {
        Some(dir) => {
            eprintln!("[{}] Fenster gespickt: Richtung ({:.0}, {:.0}) px/s", state.tag, dir.0, dir.1);
            // The window lands elsewhere; its configure ends the wiggle.
            state.wiggle_move = Some(std::time::Instant::now());
            let tag = state.tag;
            std::thread::spawn(move || {
                if matrixkit_theme::canvas::window_fling(dir) {
                    matrixkit_theme::feel::tick();
                } else {
                    eprintln!("[{tag}] Spicken: in dieser Richtung liegt kein Bildschirm");
                }
            });
        }
        None => wiggle_end(state),
    }
    true
}

fn wiggle_end<C: Client + 'static>(state: &mut House<C>) {
    if state.wiggle.take().is_some() {
        state.wiggle_move = None;
        state.refresh(Refresh::Full);
    }
}

/// Input reaches us again after the drag (the compositor's grab ended).
fn wiggle_end_on_input<C: Client + 'static>(state: &mut House<C>) {
    if state.wiggle_move.is_some_and(|t| t.elapsed().as_secs_f32() > 0.3) {
        wiggle_end(state);
    }
}

/// The wiggle: the whole picture trembles — the kit's one wiggle language
/// (mk::edit::wiggle_window, round 165).
pub fn wiggle_offset(t: f32) -> (f32, f32) {
    matrixkit_theme::edit::wiggle_window(t)
}

fn jiggle<C: Client + 'static>(state: &House<C>, mut v: Scene) -> Scene {
    let Some(t0) = state.wiggle else { return v };
    let (dx, dy) = wiggle_offset(t0.elapsed().as_secs_f32());
    let mv = |r: &mut Rect| {
        r.x += dx;
        r.y += dy;
    };
    for p in &mut v.placed {
        p.x += dx;
        p.y += dy;
        if let Some(c) = &mut p.clip {
            mv(c);
        }
    }
    for t in &mut v.texts {
        t.x += dx;
        t.y += dy;
        mv(&mut t.home);
        if let Some(c) = &mut t.clip {
            mv(c);
        }
    }
    for b in &mut v.picts {
        b.x += dx;
        b.y += dy;
        mv(&mut b.home);
        if let Some(c) = &mut b.clip {
            mv(c);
        }
    }
    for c in &mut v.curves {
        c.x += dx;
        c.y += dy;
        mv(&mut c.home);
    }
    v
}

fn secondary<C: Client + 'static>(state: &mut House<C>, x: f64, y: f64) {
    let role = hit(&state.scene.placed, x as f32, y as f32)
        .and_then(|i| state.scene.roles.get(i).copied().flatten());
    let r = state.client.secondary(role, x, y);
    state.refresh(r);
}

/// How wide the grip at the window edge is, in logical pixels.
/// Narrow enough that it takes nothing away from the app; wide enough that
/// a finger hits it.
const GRIP: f64 = 8.0;

/// Which edge lies under (x, y)? Pure computation, so it can be
/// tested without a compositor.
fn grip_edge(x: f64, y: f64, w: f64, h: f64) -> Option<xdg_toplevel::ResizeEdge> {
    if w <= 2.0 * GRIP || h <= 2.0 * GRIP {
        return None;
    }
    let left = x < GRIP;
    let right = x >= w - GRIP;
    let top = y < GRIP;
    let bottom = y >= h - GRIP;
    use xdg_toplevel::ResizeEdge as E;
    Some(match (top, bottom, left, right) {
        (true, _, true, _) => E::TopLeft,
        (true, _, _, true) => E::TopRight,
        (_, true, true, _) => E::BottomLeft,
        (_, true, _, true) => E::BottomRight,
        (true, ..) => E::Top,
        (_, true, ..) => E::Bottom,
        (_, _, true, _) => E::Left,
        (_, _, _, true) => E::Right,
        _ => return None,
    })
}

impl<C: Client> House<C> {
    /// The grip at the own window edge — only for WINDOWS;
    /// layer surfaces have none and want none.
    fn grip_at(&self, x: f64, y: f64) -> Option<xdg_toplevel::ResizeEdge> {
        self.toplevel.as_ref()?;
        grip_edge(x, y, self.size.0 as f64, self.size.1 as f64)
    }

    fn refresh(&mut self, r: Refresh) {
        match r {
            Refresh::Calm => {}
            Refresh::Dirty => {
                self.dirty = true;
                self.repaint_only = false;
            }
            Refresh::Full => {
                self.full = true;
                self.dirty = true;
            }
            Refresh::Close => self.closed = true,
            Refresh::Zoom => {
                // The wish is the OPPOSITE of the current
                // state — and the current one comes from the compositor.
                self.zoom_wish = Some(!self.maximized);
                self.dirty = true;
            }
            Refresh::Shelve => {
                self.shelve_wish = true;
                self.dirty = true;
            }
            Refresh::Panes => {
                // Panes includes the main build: view() measures
                // fresh cards BEFORE fleet_sync opens them
                // (camp 38) — the diff keeps the wall still if
                // nothing happened there.
                self.panes_dirty = true;
                self.dirty = true;
            }
            Refresh::Repaint => {
                if !self.dirty {
                    self.repaint_only = true;
                }
                self.dirty = true;
            }
            Refresh::Roll { x, y, w, h, dy } => {
                self.dirty = true;
                let area = Rect { x: x as f32, y: y as f32, w: w as f32, h: h as f32 };
                if !self.roll_spoiled {
                    match &mut self.roll_hint {
                        Some((r, sum)) if *r == area => *sum += dy,
                        None => self.roll_hint = Some((area, dy)),
                        Some(_) => {
                            self.roll_hint = None;
                            self.roll_spoiled = true;
                        }
                    }
                }
            }
        }
    }

    fn engine(&mut self) -> TextEngine {
        self.engine.take().unwrap_or_else(|| {
            let mut e = TextEngine::new();
            self.client.dress(&mut e);
            e
        })
    }
}

impl<C: Client + 'static> Dispatch<wl_registry::WlRegistry, ()> for House<C> {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::GlobalRemove { name } = event {
            if let Some(i) = state.output_globals.iter().position(|(n, _)| *n == name) {
                let (_, gone) = state.output_globals.remove(i);
                state.outputs.retain(|(o, _)| *o != gone);
                eprintln!("[{}] Ausgang zurückgezogen (Registry {name})", state.tag);
                gone.release();
            }
            return;
        }
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, version.min(6), qh, ()),
                    );
                }
                "wl_shm" => {
                    state.shm = Some(registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ()));
                }
                "wl_seat" => {
                    state.seat =
                        Some(registry.bind::<wl_seat::WlSeat, _, _>(name, version.min(5), qh, ()));
                }
                "wl_data_device_manager" => {
                    state.data_manager = Some(
                        registry.bind::<wl_data_device_manager::WlDataDeviceManager, _, _>(
                            name,
                            version.min(3),
                            qh,
                            (),
                        ),
                    );
                }
                "zwp_text_input_manager_v3" => {
                    state.ime_manager = Some(
                        registry
                            .bind::<zwp_text_input_manager_v3::ZwpTextInputManagerV3, _, _>(name, 1, qh, ()),
                    );
                }
                "wl_output" if version >= 4 => {
                    let o = registry.bind::<wl_output::WlOutput, _, _>(name, 4, qh, ());
                    eprintln!("[{}] Ausgang angemeldet (Registry {name})", state.tag);
                    state.output_globals.push((name, o.clone()));
                    state.outputs.push((o, None));
                }
                "wl_subcompositor" => {
                    state.subcompositor = Some(registry.bind::<wl_subcompositor::WlSubcompositor, _, _>(name, 1, qh, ()));
                }
                "wp_viewporter" => {
                    state.viewporter = Some(registry.bind::<wp_viewporter::WpViewporter, _, _>(name, 1, qh, ()));
                }
                "zxdg_decoration_manager_v1" => {
                    state.deco_manager = Some(
                        registry.bind::<zxdg_decoration_manager_v1::ZxdgDecorationManagerV1, _, _>(name, 1, qh, ()),
                    );
                }
                "xdg_wm_base" => {
                    state.wm_base = Some(
                        registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, version.min(2), qh, ()),
                    );
                }
                "ext_session_lock_manager_v1" => {
                    state.lock_manager = Some(
                        registry.bind::<ext_session_lock_manager_v1::ExtSessionLockManagerV1, _, _>(name, 1, qh, ()),
                    );
                }
                "zwlr_layer_shell_v1" => {
                    state.layer_shell = Some(
                        registry
                            .bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(name, version.min(4), qh, ()),
                    );
                }
                _ => {}
            }
        }
    }
}

impl<C: Client + 'static> Dispatch<wl_seat::WlSeat, ()> for House<C> {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities { capabilities: WEnum::Value(caps) } = event {
            // Round 82: a device that LEAVES must be let go — the Duo's
            // touchscreen leaves and returns on every resume from deep sleep.
            // The old wl_touch stayed, is_none() was false on return, no new
            // one was taken: dock, context menu and every long-running app
            // went deaf to fingers (the mouse still worked), 04.11.2026.
            let v3 = seat.version() >= 3;
            if !caps.contains(wl_seat::Capability::Touch) {
                if let Some(t) = state.touch_dev.take() {
                    eprintln!("[{}] Sitz: Berührung weg", state.tag);
                    if v3 {
                        t.release();
                    }
                }
            }
            if !caps.contains(wl_seat::Capability::Pointer) {
                if let Some(t) = state.pointer_dev.take() {
                    if v3 {
                        t.release();
                    }
                }
            }
            if !caps.contains(wl_seat::Capability::Keyboard) {
                if let Some(t) = state.keyboard_dev.take() {
                    if v3 {
                        t.release();
                    }
                }
            }
            if caps.contains(wl_seat::Capability::Touch) && state.touch_dev.is_none() {
                eprintln!("[{}] Sitz: Berührung da", state.tag);
            }
            if caps.contains(wl_seat::Capability::Pointer) && state.pointer_dev.is_none() {
                state.pointer_dev = Some(seat.get_pointer(qh, ()));
            }
            if caps.contains(wl_seat::Capability::Touch) && state.touch_dev.is_none() {
                state.touch_dev = Some(seat.get_touch(qh, ()));
            }
            if caps.contains(wl_seat::Capability::Keyboard) && state.keyboard_dev.is_none() {
                state.keyboard_dev = Some(seat.get_keyboard(qh, ()));
            }
        }
    }
}

/// The clipboard DEVICE on the seat: here arrives what OTHERS have
/// copied. The round dance is always the same — first the offer object,
/// then its formats, finally `selection`.
impl<C: Client + 'static> Dispatch<wl_data_device::WlDataDevice, ()> for House<C> {
    fn event(
        state: &mut Self,
        _: &wl_data_device::WlDataDevice,
        event: wl_data_device::Event,
        _: &(),
        conn: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_device::Event::DataOffer { id } => {
                state.offer_building = Some((id, Vec::new()));
            }
            wl_data_device::Event::Enter { serial, x, y, id, .. } => {
                state.drop_at = (x, y);
                if let Some((old, _)) = state.drop_offer.take() {
                    old.destroy();
                }
                let Some(offer) = id else { return };
                let mimes = match state.offer_building.take() {
                    Some((built, mimes)) if built.id() == offer.id() => mimes,
                    other => {
                        if let Some((built, _)) = other {
                            built.destroy();
                        }
                        Vec::new()
                    }
                };
                if mimes.iter().any(|m| m == "text/uri-list") {
                    offer.accept(serial, Some(String::from("text/uri-list")));
                    if offer.version() >= 3 {
                        let copy = wl_data_device_manager::DndAction::Copy;
                        offer.set_actions(copy, copy);
                    }
                    state.drop_offer = Some((offer, String::from("text/uri-list")));
                } else {
                    // No file drag — accept nothing, hold nothing.
                    offer.accept(serial, None);
                    offer.destroy();
                }
            }
            wl_data_device::Event::Motion { x, y, .. } => {
                state.drop_at = (x, y);
            }
            wl_data_device::Event::Leave => {
                if let Some((offer, _)) = state.drop_offer.take() {
                    offer.destroy();
                }
            }
            wl_data_device::Event::Drop => {
                let Some((offer, mime)) = state.drop_offer.take() else { return };
                let (x, y) = state.drop_at;
                let mut fds = [0i32; 2];
                if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
                    offer.destroy();
                    return;
                }
                let (read_fd, write_fd) = (fds[0], fds[1]);
                offer.receive(mime, unsafe { std::os::fd::BorrowedFd::borrow_raw(write_fd) });
                let _ = conn.flush();
                unsafe { libc::close(write_fd) };
                let finish = offer.version() >= 3;
                std::thread::spawn(move || {
                    use std::io::Read;
                    let mut pipe = unsafe { std::fs::File::from_raw_fd(read_fd) };
                    let mut text = String::new();
                    if pipe.read_to_string(&mut text).is_ok() {
                        let paths = drops::paths_from_uri_list(&text);
                        if !paths.is_empty() {
                            drops::set(paths, x, y);
                        }
                    }
                });
                // The receive is triggered; the other side cleans up
                // once it has finished writing.
                if finish {
                    offer.finish();
                }
                offer.destroy();
            }
            wl_data_device::Event::Selection { id } => {
                // The previous offer goes — the reader thread is long
                // done, and a held offer is a leak.
                if let Some(old) = state.offer_current.take() {
                    old.destroy();
                }
                let Some(offer) = id else {
                    // Empty clipboard: hold nothing that no longer applies.
                    board::set_held(None);
                    state.offer_building = None;
                    return;
                };
                let mimes = match state.offer_building.take() {
                    Some((built, mimes)) if built.id() == offer.id() => mimes,
                    other => {
                        if let Some((built, _)) = other {
                            built.destroy();
                        }
                        Vec::new()
                    }
                };
                // The best text format, in this order.
                let want = ["text/plain;charset=utf-8", "UTF8_STRING", "text/plain"]
                    .into_iter()
                    .find(|m| mimes.iter().any(|x| x == m));
                let Some(want) = want else {
                    // No text included (a picture, a file): our
                    // own clipboard stays as it was.
                    offer.destroy();
                    return;
                };
                let mut fds = [0i32; 2];
                if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
                    offer.destroy();
                    return;
                }
                let (read_fd, write_fd) = (fds[0], fds[1]);
                offer.receive(String::from(want), unsafe {
                    std::os::fd::BorrowedFd::borrow_raw(write_fd)
                });
                // Without this flush the other side sees the request
                // only at the next frame — and the reader thread would hang.
                let _ = conn.flush();
                unsafe { libc::close(write_fd) };
                // READING HAPPENS ON A THREAD: the other side
                // writes at its own pace, and the house's
                // loop must not wait for it.
                std::thread::spawn(move || {
                    use std::io::Read;
                    let mut pipe = unsafe { std::fs::File::from_raw_fd(read_fd) };
                    let mut text = String::new();
                    if pipe.read_to_string(&mut text).is_ok() {
                        board::set_held(Some(text));
                    }
                });
                state.offer_current = Some(offer);
            }
            _ => {}
        }
    }

    event_created_child!(House<C>, wl_data_device::WlDataDevice, [
        wl_data_device::EVT_DATA_OFFER_OPCODE => (wl_data_offer::WlDataOffer, ()),
    ]);
}

/// An OFFER of the other side: only its formats interest us.
impl<C: Client + 'static> Dispatch<wl_data_offer::WlDataOffer, ()> for House<C> {
    fn event(
        state: &mut Self,
        offer: &wl_data_offer::WlDataOffer,
        event: wl_data_offer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_data_offer::Event::Offer { mime_type } = event {
            if let Some((built, mimes)) = state.offer_building.as_mut() {
                if built.id() == offer.id() {
                    mimes.push(mime_type);
                }
            }
        }
    }
}

/// Our own SOURCE: whoever pastes from us gets the text here.
impl<C: Client + 'static> Dispatch<wl_data_source::WlDataSource, String> for House<C> {
    fn event(
        state: &mut Self,
        source: &wl_data_source::WlDataSource,
        event: wl_data_source::Event,
        text: &String,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_source::Event::Send { fd, .. } => {
                let text = text.clone();
                // A thread here too: the other side reads at its own
                // pace, and a full pipe would otherwise block the house.
                std::thread::spawn(move || {
                    use std::io::Write;
                    let mut pipe = std::fs::File::from(fd);
                    let _ = pipe.write_all(text.as_bytes());
                });
            }
            wl_data_source::Event::Cancelled => {
                // Someone else copied — our source is dead.
                if state.data_source.as_ref().is_some_and(|s| s.id() == source.id()) {
                    state.data_source = None;
                }
                source.destroy();
            }
            _ => {}
        }
    }
}

impl<C: Client + 'static> Dispatch<wl_pointer::WlPointer, ()> for House<C> {
    fn event(
        state: &mut Self,
        _: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(event, wl_pointer::Event::Button { .. } | wl_pointer::Event::Axis { .. }) {
            big_cores::touched();
        }
        match event {
            wl_pointer::Event::Enter { surface, surface_x, surface_y, .. } => {
                // Camp 35: WHICH surface? The fleet listens itself.
                state.pointer_pane = state
                    .panes
                    .iter()
                    .find(|pn| pn.surface.id() == surface.id())
                    .map(|pn| pn.id);
                state.pointer = (surface_x, surface_y);
                if state.pointer_pane.is_none() {
                    let r = state.client.motion(surface_x, surface_y);
                    state.refresh(r);
                }
            }
            wl_pointer::Event::Motion { surface_x, surface_y, .. } => {
                wiggle_end_on_input(state);
                state.pointer = (surface_x, surface_y);
                press_moved(state, surface_x, surface_y);
                let r = match (state.pointer_pane, state.pointer_held) {
                    (Some(pid), true) => {
                        if let Some(card) = pane_card(state, pid) {
                            state.client.pane_travel(&card, surface_x, surface_y, Source::Pointer)
                        } else {
                            Refresh::Calm
                        }
                    }
                    (Some(_), false) => Refresh::Calm,
                    (None, true) => state.client.travel(surface_x, surface_y, Source::Pointer),
                    (None, false) => state.client.motion(surface_x, surface_y),
                };
                state.refresh(r);
            }
            wl_pointer::Event::Leave { .. } => {
                state.press = None;
                // The implicit grab ends with the button — if
                // the surface loses the pointer anyway, that counts
                // as a release (no drag stays open forever).
                if state.pointer_held {
                    state.pointer_held = false;
                    let (x, y) = state.pointer;
                    match state.pointer_pane {
                        Some(pid) => {
                            if let Some(card) = pane_card(state, pid) {
                                let r = state.client.pane_release(&card, x, y, Source::Pointer);
                                state.refresh(r);
                            }
                        }
                        None => release(state, x, y, Source::Pointer),
                    }
                }
                let war_haupt = state.pointer_pane.is_none();
                state.pointer_pane = None;
                if war_haupt {
                    // The pointer is GONE — the app must take back its
                    // hover state (camp 91).
                    let r = state.client.depart();
                    state.refresh(r);
                }
            }
            wl_pointer::Event::AxisSource { axis_source } => {
                state.wheel = matches!(axis_source, WEnum::Value(wl_pointer::AxisSource::Wheel) | WEnum::Value(wl_pointer::AxisSource::WheelTilt));
            }
            wl_pointer::Event::Axis {
                axis: WEnum::Value(wl_pointer::Axis::VerticalScroll),
                value,
                ..
            } => {
                let (x, y) = state.pointer;
                // Round 161: a wheel notch goes further (mk::input::wheel_scaled).
                let factor = matrixkit_theme::input::WHEEL_SPEEDS[matrixkit_theme::input::wheel_choice()].0;
                let value = matrixkit_theme::input::wheel_scaled(value, state.wheel, factor);
                // Round 163: over a fleet surface the wheel is that surface's
                // (the bar's cards did not scroll by mouse at all).
                if let Some(card) = state.pointer_pane.and_then(|pid| pane_card(state, pid)) {
                    let r = state.client.pane_axis(&card, x, y, value);
                    state.refresh(r);
                    return;
                }
                let mut keep = std::mem::take(&mut state.keep);
                let r = state.client.axis(x, y, value, &mut keep);
                state.keep = keep;
                state.refresh(r);
            }
            wl_pointer::Event::Button { state: btn, serial, button, .. } => {
                state.serial = serial;
                if btn == WEnum::Value(wl_pointer::ButtonState::Pressed) {
                    state.press_serial = serial;
                }
                wiggle_end_on_input(state);
                let (x, y) = state.pointer;
                // THE WINDOW EDGE (camp 89): a window without
                // a server frame does not get its grip for
                // free — it has to report the grab itself. Without
                // that, a client-painted window cannot be grabbed with the
                // mouse, and that is exactly what was missing.
                if button == BTN_LEFT
                    && btn == WEnum::Value(wl_pointer::ButtonState::Pressed)
                    && state.pointer_pane.is_none()
                {
                    if let Some(edge) = state.grip_at(x, y) {
                        if let (Some(top), Some(seat)) = (&state.toplevel, &state.seat) {
                            top.resize(seat, serial, edge);
                            // The compositor leads the drag from now on —
                            // the app must not see it as a click.
                            state.pointer_held = false;
                            return;
                        }
                    }
                }
                // R207: THE HEAD DRAGS — a press in the head strip that
                // hits no button hands the window to the
                // compositor for moving.
                if button == BTN_LEFT
                    && btn == WEnum::Value(wl_pointer::ButtonState::Pressed)
                    && state.pointer_pane.is_none()
                    && state.toplevel.is_some()
                {
                    let head = if matrixkit_theme::habitat::is_matrix() { 0.0 } else { state.client.headline() as f64 };
                    if head > 0.0 && y < head {
                        let role = hit(&state.scene.placed, x as f32, y as f32)
                            .and_then(|i| state.scene.roles.get(i).copied().flatten());
                        if role.is_none() {
                            if let (Some(top), Some(seat)) = (&state.toplevel, &state.seat) {
                                top._move(seat, serial);
                                state.pointer_held = false;
                                state.press = None;
                                return;
                            }
                        }
                    }
                }
                if button == BTN_RIGHT && state.pointer_pane.is_none() {
                    if btn == WEnum::Value(wl_pointer::ButtonState::Pressed) {
                        secondary(state, x, y);
                    }
                    return;
                }
                // Round 28: a right click on a fleet surface is its own deed.
                if button == BTN_RIGHT {
                    if btn == WEnum::Value(wl_pointer::ButtonState::Pressed) {
                        if let Some(card) = state.pointer_pane.and_then(|pid| pane_card(state, pid)) {
                            let r = state.client.pane_secondary(&card, x, y);
                            state.refresh(r);
                        }
                    }
                    return;
                }
                if btn == WEnum::Value(wl_pointer::ButtonState::Pressed) {
                    state.pointer_held = true;
                    match state.pointer_pane {
                        None => press_begin(state, x, y, Source::Pointer),
                        Some(pid) => press_begin_pane(state, pid, x, y, Source::Pointer),
                    }
                    match state.pointer_pane {
                        Some(pid) => {
                            if let Some(card) = pane_card(state, pid) {
                                let r = state.client.pane_strike(&card, x, y, Source::Pointer);
                                state.refresh(r);
                            }
                        }
                        None => strike(state, x, y, Source::Pointer),
                    }
                } else if state.pointer_held {
                    state.pointer_held = false;
                    state.press = None;
                    match state.pointer_pane {
                        Some(pid) => {
                            if let Some(card) = pane_card(state, pid) {
                                let r = state.client.pane_release(&card, x, y, Source::Pointer);
                                state.refresh(r);
                            }
                        }
                        None => release(state, x, y, Source::Pointer),
                    }
                }
            }
            _ => {}
        }
    }
}

/// Camp 19: the keyboard. The compositor sends the keymap as a
/// file (xkb v1); the house builds the xkb state from it and
/// translates every pressed key into meaning. The compositor keeps
/// the modifier state current — we only mirror it.
impl<C: Client + 'static> Dispatch<wl_keyboard::WlKeyboard, ()> for House<C> {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use xkbcommon::xkb;
        match event {
            wl_keyboard::Event::Keymap { format: WEnum::Value(wl_keyboard::KeymapFormat::XkbV1), fd, size } => {
                let ctx = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
                let keymap = unsafe {
                    xkb::Keymap::new_from_fd(
                        &ctx,
                        fd,
                        size as usize,
                        xkb::KEYMAP_FORMAT_TEXT_V1,
                        xkb::KEYMAP_COMPILE_NO_FLAGS,
                    )
                };
                if let Ok(Some(keymap)) = keymap {
                    state.xkb = Some(xkb::State::new(&keymap));
                }
            }
            wl_keyboard::Event::Modifiers { mods_depressed, mods_latched, mods_locked, group, .. } => {
                if let Some(x) = state.xkb.as_mut() {
                    x.update_mask(mods_depressed, mods_latched, mods_locked, 0, 0, group);
                }
            }
            wl_keyboard::Event::RepeatInfo { rate, delay } => {
                state.repeat_rate = rate;
                state.repeat_delay = delay;
            }
            wl_keyboard::Event::Enter { .. } => {
                state.keyboard_focus = true;
                let r = state.client.focus(true);
                state.refresh(r);
            }
            wl_keyboard::Event::Leave { .. } => {
                state.held = None;
                state.keyboard_focus = false;
                let r = state.client.focus(false);
                state.refresh(r);
            }
            wl_keyboard::Event::Key { key, state: kstate, serial, .. } => {
                // The timestamp of the input: `set_selection` only works
                // with one the compositor knows (camp 88).
                state.serial = serial;
                if kstate == WEnum::Value(wl_keyboard::KeyState::Released) {
                    if state.held.as_ref().is_some_and(|(raw, ..)| *raw == key) {
                        state.held = None;
                    }
                    return;
                }
                if kstate != WEnum::Value(wl_keyboard::KeyState::Pressed) {
                    return;
                }
                let Some(x) = state.xkb.as_ref() else { return };
                let code = xkb::Keycode::new(key + 8);
                let sym = x.key_get_one_sym(code);
                let shift =
                    x.mod_name_is_active(xkb::MOD_NAME_SHIFT, xkb::STATE_MODS_EFFECTIVE);
                let mods = Mods {
                    shift,
                    ctrl: x.mod_name_is_active(xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE),
                    alt: x.mod_name_is_active(xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE),
                    logo: x.mod_name_is_active(xkb::MOD_NAME_LOGO, xkb::STATE_MODS_EFFECTIVE),
                };
                let deed = match sym {
                    xkb::Keysym::BackSpace => Some(Key::Backspace),
                    xkb::Keysym::Delete => Some(Key::Delete),
                    xkb::Keysym::Up => Some(Key::Up),
                    xkb::Keysym::Down => Some(Key::Down),
                    xkb::Keysym::Left => Some(Key::Left),
                    xkb::Keysym::Right => Some(Key::Right),
                    xkb::Keysym::Home => Some(Key::Home),
                    xkb::Keysym::End => Some(Key::End),
                    xkb::Keysym::Return | xkb::Keysym::KP_Enter => Some(Key::Enter),
                    xkb::Keysym::Escape => Some(Key::Escape),
                    xkb::Keysym::Tab => Some(Key::Tab),
                    // With Shift, xkb calls Tab ISO_Left_Tab —
                    // the same key, the same contract (camp 31).
                    xkb::Keysym::ISO_Left_Tab => Some(Key::Tab),
                    xkb::Keysym::Page_Up => Some(Key::PageUp),
                    xkb::Keysym::Page_Down => Some(Key::PageDown),
                    xkb::Keysym::Insert => Some(Key::Insert),
                    f if (xkb::Keysym::F1.raw()..=xkb::Keysym::F12.raw()).contains(&f.raw()) => {
                        Some(Key::Function((f.raw() - xkb::Keysym::F1.raw() + 1) as u8))
                    }
                    _ => {
                        let utf8 = x.key_get_utf8(code);
                        if !utf8.is_empty() && !utf8.chars().any(|c| c.is_control()) {
                            Some(Key::Char(utf8))
                        } else {
                            // With Ctrl, xkb delivers the CONTROL CHARACTER
                            // (Ctrl-C = 0x03). The client wants the
                            // KEY, not the character — otherwise there is
                            // no Ctrl-C in the terminal (camp 86).
                            let base = xkb::keysym_to_utf8(sym);
                            let base = base.trim_end_matches('\0').to_string();
                            (!base.is_empty() && !base.chars().any(|c| c.is_control()))
                                .then_some(Key::Char(base))
                        }
                    }
                };
                if let Some(k) = deed {
                    // Repeat: the xkb keymap entry decides
                    // WHETHER the key repeats (Escape, for example, does not).
                    let repeats = x.get_keymap().key_repeats(code) && state.repeat_rate > 0;
                    if repeats {
                        state.held = Some((
                            key,
                            k.clone(),
                            shift,
                            std::time::Instant::now()
                                + std::time::Duration::from_millis(state.repeat_delay.max(0) as u64),
                        ));
                    }
                    let r = state.client.key_with(k, mods);
                    state.refresh(r);
                }
            }
            _ => {}
        }
    }
}

delegate_noop!(@<C: Client + 'static> House<C>: ignore zwp_text_input_manager_v3::ZwpTextInputManagerV3);

/// The IME line: the keystone. commit_string types like a
/// key, delete_surrounding deletes before it; the preedit
/// is today only NAMED — showing it in the field is the
/// last open item.
impl<C: Client + 'static> Dispatch<zwp_text_input_v3::ZwpTextInputV3, ()> for House<C> {
    fn event(
        state: &mut Self,
        _: &zwp_text_input_v3::ZwpTextInputV3,
        event: zwp_text_input_v3::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_text_input_v3::Event::PreeditString { text, .. } => {
                if let Some(t) = text {
                    eprintln!("[{}] IME-Vorschrift: „{}“", state.tag, t);
                }
            }
            zwp_text_input_v3::Event::CommitString { text } => {
                if let Some(t) = text {
                    if !t.is_empty() {
                        let r = state.client.key(Key::Char(t), false);
                        state.refresh(r);
                    }
                }
            }
            zwp_text_input_v3::Event::DeleteSurroundingText { before_length, .. } => {
                for _ in 0..before_length {
                    let r = state.client.key(Key::Backspace, false);
                    state.refresh(r);
                }
            }
            _ => {}
        }
    }
}

impl<C: Client + 'static> Dispatch<wl_touch::WlTouch, ()> for House<C> {
    fn event(
        state: &mut Self,
        _: &wl_touch::WlTouch,
        event: wl_touch::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        big_cores::touched();
        match event {
            wl_touch::Event::Down { serial, surface, id, x, y, .. } => {
                state.press_serial = serial;
                wiggle_end_on_input(state);
                state.touches.retain(|t| t.0 != id);
                state.touches.push((id, x, y));
                if state.touches.len() >= 2 {
                    // The pinch begins: the drag of the first finger
                    // ends (pinch first, so the app drops its drag,
                    // then the release into nothing).
                    if state.pinch_dist.is_none() {
                        let (cx, cy, d) = pinch_geometry(&state.touches);
                        state.pinch_dist = Some(d);
                        state.press = None;
                        let r = state.client.pinch(cx, cy, 1.0);
                        state.refresh(r);
                        if !state.touch_numb {
                            let (px, py) = state.touch_at;
                            match state.touch_pane.take() {
                                Some(pid) => {
                                    if let Some(card) = pane_card(state, pid) {
                                        let r = state.client.pane_release(&card, px, py, Source::Touch);
                                        state.refresh(r);
                                    }
                                }
                                None => {
                                    let r = state.client.release(None, px, py, Source::Touch);
                                    state.refresh(r);
                                }
                            }
                        }
                        state.touch_numb = true;
                    }
                    return;
                }
                if state.touch_numb {
                    return;
                }
                state.touch_pane = state
                    .panes
                    .iter()
                    .find(|pn| pn.surface.id() == surface.id())
                    .map(|pn| pn.id);
                state.touch_at = (x, y);
                // R209: on a device with its own panels the head is a flick
                // strip — remember a press there that hits no button.
                state.flick = None;
                if state.touch_pane.is_none()
                    && matrixkit_theme::device::panels().is_some()
                    && y < state.client.headline() as f64
                {
                    let role = hit(&state.scene.placed, x as f32, y as f32)
                        .and_then(|i| state.scene.roles.get(i).copied().flatten());
                    if role.is_none() {
                        state.flick = Some((id, x, y));
                    }
                }
                match state.touch_pane {
                    None => press_begin(state, x, y, Source::Touch),
                    Some(pid) => press_begin_pane(state, pid, x, y, Source::Touch),
                }
                match state.touch_pane {
                    Some(pid) => {
                        if let Some(card) = pane_card(state, pid) {
                            let r = state.client.pane_strike(&card, x, y, Source::Touch);
                            state.refresh(r);
                        }
                    }
                    None => strike(state, x, y, Source::Touch),
                }
            }
            wl_touch::Event::Motion { id, x, y, .. } => {
                if let Some(t) = state.touches.iter_mut().find(|t| t.0 == id) {
                    t.1 = x;
                    t.2 = y;
                }
                if let Some(last) = state.pinch_dist {
                    if state.touches.len() >= 2 {
                        let (cx, cy, d) = pinch_geometry(&state.touches);
                        if last > 1.0 && d > 1.0 {
                            let r = state.client.pinch(cx, cy, d / last);
                            state.refresh(r);
                        }
                        state.pinch_dist = Some(d.max(1.0));
                    }
                    return;
                }
                if state.touch_numb {
                    return;
                }
                if fling_motion(state, x, y) {
                    return;
                }
                state.touch_at = (x, y);
                press_moved(state, x, y);
                let r = match state.touch_pane {
                    Some(pid) => match pane_card(state, pid) {
                        Some(card) => state.client.pane_travel(&card, x, y, Source::Touch),
                        None => Refresh::Calm,
                    },
                    None => state.client.travel(x, y, Source::Touch),
                };
                state.refresh(r);
            }
            // Up names no point — the last touch point applies.
            // Cancel (the compositor takes the gesture for itself) ends
            // the drag just the same: no press stays open.
            wl_touch::Event::Up { id, .. } => {
                // Round 171: the flick (or a long press let go in place).
                if state.fling.is_some() {
                    state.touches.retain(|t| t.0 != id);
                    if state.touches.is_empty() {
                        state.touch_numb = false;
                    }
                    fling_release(state);
                    return;
                }
                // R209 FLICK: pulled far enough from the head and let go —
                // to the neighbouring panel (the same family as iced's
                // flick_area: mk::canvas::window_to_neighbour).
                if let Some((fid, x0, y0)) = state.flick.take() {
                    let (x1, y1) = state.touch_at;
                    let (dx, dy) = (x1 - x0, y1 - y0);
                    if fid == id && dx.hypot(dy) >= 60.0 {
                        let side = if dx.abs() >= dy.abs() {
                            if dx > 0.0 { "right" } else { "left" }
                        } else if dy > 0.0 {
                            "down"
                        } else {
                            "up"
                        };
                        std::thread::spawn(move || matrixkit_theme::canvas::window_to_neighbour(side));
                    }
                }
                state.touches.retain(|t| t.0 != id);
                if state.pinch_dist.is_some() {
                    if state.touches.len() < 2 {
                        state.pinch_dist = None;
                    }
                    if state.touches.is_empty() {
                        state.touch_numb = false;
                    }
                    return;
                }
                let numb = state.touch_numb;
                if state.touches.is_empty() {
                    state.touch_numb = false;
                }
                if numb {
                    return;
                }
                touch_release(state);
            }
            wl_touch::Event::Cancel => {
                if state.fling.take().is_some() {
                    wiggle_end(state);
                }
                state.touches.clear();
                state.pinch_dist = None;
                state.touch_numb = false;
                touch_release(state);
            }
            _ => {}
        }
    }
}

/// Middle and distance of the first two fingers (camp 112).
fn pinch_geometry(touches: &[(i32, f64, f64)]) -> (f64, f64, f64) {
    let (a, b) = (touches[0], touches[1]);
    ((a.1 + b.1) / 2.0, (a.2 + b.2) / 2.0, ((a.1 - b.1).powi(2) + (a.2 - b.2).powi(2)).sqrt())
}

/// The release of a finger at the last touch point.
fn touch_release<C: Client + 'static>(state: &mut House<C>) {
    state.press = None;
    let (x, y) = state.touch_at;
    match state.touch_pane.take() {
        Some(pid) => {
            if let Some(card) = pane_card(state, pid) {
                let r = state.client.pane_release(&card, x, y, Source::Touch);
                state.refresh(r);
            }
        }
        None => release(state, x, y, Source::Touch),
    }
}

/// The house's hit test: mk::scene::hit on the
/// retained scene, the role goes to the client.
fn strike<C: Client + 'static>(state: &mut House<C>, x: f64, y: f64, source: Source) {
    // The overlay catches its own taps first (a creature shooed away).
    if state.overlay.as_mut().is_some_and(|o| o.strike(x as f32, y as f32)) {
        state.refresh(Refresh::Dirty);
        return;
    }
    if state.scene.placed.is_empty() {
        return;
    }
    let Some(i) = hit(&state.scene.placed, x as f32, y as f32) else {
        return;
    };
    let role = state.scene.roles.get(i).copied().flatten();
    // R209 TAP ANSWER: a finger on a button gets the dock's short buzz
    // (`mk::device::haptic`, only on devices with a haptic helper).
    if role.is_some() && matches!(source, Source::Touch) {
        matrixkit_theme::feel::tap();
    }
    let r = state.client.strike(role, x, y, source);
    state.refresh(r);
    // A FRESH touch in a typing window (03.11.2026): the keyboard guard
    // re-arms even after ✕ — before, the window's steady «open!» beat kept
    // it locked until the field lost focus («keyboard won't start any more»).
    if matches!(source, Source::Touch) && state.client.wants_keyboard() && matrixkit_theme::device::finger_first() {
        matrixkit_theme::onscreen_keyboard::send("tap");
    }
}

/// The card of a fleet surface, freshly cloned (camp 35).
fn pane_card<C: Client + 'static>(state: &House<C>, pid: u32) -> Option<C::Card> {
    state.panes.iter().find(|pn| pn.id == pid).map(|pn| pn.card.clone())
}

/// The release ALWAYS goes to the client — even beside the scene:
/// a drag may end in empty space, and the app must be able to
/// close it. The role is the hit at the release point.
fn release<C: Client + 'static>(state: &mut House<C>, x: f64, y: f64, source: Source) {
    let role = hit(&state.scene.placed, x as f32, y as f32)
        .and_then(|i| state.scene.roles.get(i).copied().flatten());
    let r = state.client.release(role, x, y, source);
    state.refresh(r);
}

impl<C: Client + 'static> Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for House<C> {
    fn event(
        state: &mut Self,
        layer: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
                // Callback sovereignty: we answer the configure
                // ourselves — no watchdog, no dice roll.
                layer.ack_configure(serial);
                if let Some(idx) = state.panes.iter().position(|pn| pn.layer.as_ref().is_some_and(|l| l.id() == layer.id())) {
                    eprintln!(
                        "[{}] Flotte: Fläche {} konfiguriert {}×{}",
                        state.tag, idx, width, height
                    );
                    if width > 0 && height > 0 {
                        state.panes[idx].size = (width, height);
                    }
                    state.panes[idx].ready = true;
                    state.panes[idx].shown = 0; // (re)configured: always paint
                    draw_pane(state, idx, qh);
                    return;
                }
                if width > 0 && height > 0 {
                    if (width, height) != state.size {
                        state.full = true;
                    }
                    state.size = (width, height);
                }
                state.dirty = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                // Round 80: a FLEET pane closed by the compositor (its output
                // went away) — only that pane goes and comes back; before,
                // any pane's Closed ended the whole app.
                if let Some(idx) = state.panes.iter().position(|pn| pn.layer.as_ref().is_some_and(|l| l.id() == layer.id())) {
                    let wid = state.panes[idx].id;
                    eprintln!("[{}] Flotte: Fläche {} vom Compositor geschlossen — neu", state.tag, wid);
                    for gone in state.fleet.forget(wid) {
                        if let Some(pos) = state.panes.iter().position(|pn| pn.id == gone) {
                            let mut pane = state.panes.remove(pos);
                            if let Some(child) = pane.zoom.take() {
                                child.retire();
                            }
                            drop(pane.target.take());
                            for b in &pane.bufs {
                                b.buffer.destroy();
                            }
                            if let Some(l) = pane.layer.take() {
                                l.destroy();
                            }
                            pane.surface.destroy();
                        }
                    }
                    state.panes_dirty = true;
                    return;
                }
                state.closed = true;
            }
            _ => {}
        }
    }
}

delegate_noop!(@<C: Client + 'static> House<C>: ignore ext_session_lock_manager_v1::ExtSessionLockManagerV1);

/// Round 123: the lock itself — `locked` (all screens covered) or
/// `finished` (refused, or taken back by the compositor).
impl<C: Client + 'static> Dispatch<ext_session_lock_v1::ExtSessionLockV1, ()> for House<C> {
    fn event(
        state: &mut Self,
        _: &ext_session_lock_v1::ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_session_lock_v1::Event::Locked => {
                eprintln!("[{}] Sperre: gesperrt", state.tag);
                state.lock_confirmed = true;
            }
            ext_session_lock_v1::Event::Finished => {
                eprintln!("[{}] Sperre: vom Compositor abgelehnt/beendet", state.tag);
                state.lock_refused = true;
                state.closed = true;
            }
            _ => {}
        }
    }
}

/// Round 123: a lock surface is sized by the compositor (the whole
/// screen). The main one is the client's; the others get a night colour.
impl<C: Client + 'static> Dispatch<ext_session_lock_surface_v1::ExtSessionLockSurfaceV1, ()> for House<C> {
    fn event(
        state: &mut Self,
        ls: &ext_session_lock_surface_v1::ExtSessionLockSurfaceV1,
        event: ext_session_lock_surface_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let ext_session_lock_surface_v1::Event::Configure { serial, width, height } = event else { return };
        ls.ack_configure(serial);
        if let Some(idx) = state.panes.iter().position(|pn| pn.lock.as_ref().is_some_and(|l| l.id() == ls.id())) {
            eprintln!("[{}] Sperre: Flotten-Fläche {} {}×{}", state.tag, state.panes[idx].id, width, height);
            // Round 134: a night picture AT ONCE — until the pane's GPU
            // paints, niri showed its dark red «locked, nothing drawn».
            if !state.panes[idx].ready {
                night_fill(state, &state.panes[idx].surface.clone(), width, height, qh);
            }
            if width > 0 && height > 0 {
                state.panes[idx].size = (width, height);
            }
            state.panes[idx].ready = true;
            state.panes[idx].shown = 0;
            draw_pane(state, idx, qh);
            return;
        }
        if let Some(idx) = state.lock_others.iter().position(|(o, ..)| o.id() == ls.id()) {
            let side = state.lock_others[idx].1.clone();
            if let Some(buffer) = night_fill(state, &side, width, height, qh) {
                if let Some(old) = state.lock_others[idx].2.replace(buffer) {
                    old.destroy();
                }
            }
            return;
        }
        if width > 0 && height > 0 {
            if (width, height) != state.size {
                state.full = true;
            }
            state.size = (width, height);
        }
        state.dirty = true;
    }
}

/// Round 123/134: a lock surface's night picture (one deep colour, the
/// surface's whole size) — the buffer is returned for its owner to keep
/// and destroy.
fn night_fill<C: Client + 'static>(state: &mut House<C>, surface: &wl_surface::WlSurface, width: u32, height: u32, qh: &QueueHandle<House<C>>) -> Option<wl_buffer::WlBuffer> {
    let (w, h) = (width.max(1), height.max(1));
    let len = (w * h * 4) as usize;
    let (Some(shm), Some((fd, mut map))) = (state.shm.clone(), make_memfd(len)) else { return None };
    for px in map.chunks_exact_mut(4) {
        px.copy_from_slice(&[0x16, 0x10, 0x0c, 0xff]); // BGRA: a deep night
    }
    let pool = shm.create_pool(fd.as_fd(), len as i32, qh, ());
    let buffer = pool.create_buffer(0, w as i32, h as i32, (w * 4) as i32, wl_shm::Format::Argb8888, qh, (u32::MAX, usize::MAX));
    pool.destroy();
    surface.set_buffer_scale(1);
    surface.attach(Some(&buffer), 0, 0);
    surface.damage_buffer(0, 0, w as i32, h as i32);
    surface.commit();
    Some(buffer)
}

/// xdg_wm_base pings — whoever does not pong counts as dead.
impl<C: Client + 'static> Dispatch<xdg_wm_base::XdgWmBase, ()> for House<C> {
    fn event(
        _: &mut Self,
        wm: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm.pong(serial);
        }
    }
}

/// The window beat: ack, then paint — the same callback sovereignty
/// as for the layer surface.
impl<C: Client + 'static> Dispatch<xdg_surface::XdgSurface, ()> for House<C> {
    fn event(
        state: &mut Self,
        xdg: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            xdg.ack_configure(serial);
            if state.size == (0, 0) {
                state.size = state.want;
                state.full = true;
            }
            state.dirty = true;
        }
    }
}

impl<C: Client + 'static> Dispatch<wl_output::WlOutput, ()> for House<C> {
    fn event(state: &mut Self, output: &wl_output::WlOutput, event: wl_output::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let wl_output::Event::Name { name } = event {
            if let Some(slot) = state.outputs.iter_mut().find(|(o, _)| o == output) {
                slot.1 = Some(name);
            }
        }
    }
}

/// `--output NAME` of this process (the per-panel convention of the shell
/// parts, R209/R216) — None: the compositor chooses.
pub fn output_wish() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == "--output").and_then(|i| args.get(i + 1).cloned())
}

impl<C: Client + 'static> Dispatch<wp_viewporter::WpViewporter, ()> for House<C> {
    fn event(_: &mut Self, _: &wp_viewporter::WpViewporter, _: wp_viewporter::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl<C: Client + 'static> Dispatch<wp_viewport::WpViewport, ()> for House<C> {
    fn event(_: &mut Self, _: &wp_viewport::WpViewport, _: wp_viewport::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl<C: Client + 'static> Dispatch<zxdg_decoration_manager_v1::ZxdgDecorationManagerV1, ()> for House<C> {
    fn event(
        _: &mut Self,
        _: &zxdg_decoration_manager_v1::ZxdgDecorationManagerV1,
        _: zxdg_decoration_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl<C: Client + 'static> Dispatch<zxdg_toplevel_decoration_v1::ZxdgToplevelDecorationV1, ()> for House<C> {
    fn event(
        _: &mut Self,
        _: &zxdg_toplevel_decoration_v1::ZxdgToplevelDecorationV1,
        _: zxdg_toplevel_decoration_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // The compositor names its mode; we stay with our
        // frame — a second one would be a mistake, not decoration.
    }
}

impl<C: Client + 'static> Dispatch<xdg_toplevel::XdgToplevel, ()> for House<C> {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Configure { width, height, states } => {
                // Round 164: the compositor placed the moved window — calm again.
                if state.wiggle_move.is_some_and(|t| t.elapsed().as_secs_f32() > 0.3) {
                    wiggle_end(state);
                }
                // The STATE comes as an array of 32-bit values in
                // order of arrival (camp 92). Maximised is 1.
                let max_now = states
                    .chunks_exact(4)
                    .map(|b| u32::from_ne_bytes([b[0], b[1], b[2], b[3]]))
                    .filter_map(|v| xdg_toplevel::State::try_from(v).ok())
                    .any(|s| s == xdg_toplevel::State::Maximized);
                if max_now != state.maximized {
                    state.maximized = max_now;
                    state.client.zoomed(max_now);
                    eprintln!(
                        "[{}] Fenster {}",
                        state.tag,
                        if max_now { "MAXIMIERT" } else { "wieder frei" }
                    );
                }
                if width > 0 && height > 0 {
                    let neu = (width as u32, height as u32);
                    if neu != state.size {
                        state.full = true;
                    }
                    state.size = neu;
                }
                state.dirty = true;
            }
            xdg_toplevel::Event::Close => state.closed = true,
            _ => {}
        }
    }
}

delegate_noop!(@<C: Client + 'static> House<C>: ignore wl_compositor::WlCompositor);
delegate_noop!(@<C: Client + 'static> House<C>: ignore wl_subcompositor::WlSubcompositor);
delegate_noop!(@<C: Client + 'static> House<C>: ignore wl_subsurface::WlSubsurface);
impl<C: Client + 'static> Dispatch<wl_surface::WlSurface, ()> for House<C> {
    fn event(
        state: &mut Self,
        surface: &wl_surface::WlSurface,
        event: wl_surface::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        // R216: a fleet surface has its own scale (it may lie on another
        // screen) — it never touches the main surface's.
        if let wl_surface::Event::PreferredBufferScale { factor } = &event {
            if let Some(idx) = state.panes.iter().position(|pn| pn.surface.id() == surface.id()) {
                let neu = (*factor).max(1);
                if neu != state.panes[idx].scale {
                    state.panes[idx].scale = neu;
                    state.panes[idx].shown = 0;
                    if state.panes[idx].ready {
                        draw_pane(state, idx, qh);
                    }
                }
                return;
            }
        }
        // Round 123: the lock's night surfaces on other screens have no say.
        if state.lock_others.iter().any(|(_, side, _)| side.id() == surface.id()) {
            return;
        }
        // The SCALE of the screen (camp 87). Without it
        // the client paints in LOGICAL pixels, and the compositor stretches
        // the picture — on a 2× screen everything then looks
        // soft, text first. The client keeps computing
        // logically; the house rasterises more densely.
        if let wl_surface::Event::PreferredBufferScale { factor } = event {
            let neu = factor.max(1);
            if neu != state.scale {
                state.scale = neu;
                SCREEN_SCALE.store(neu as u32, std::sync::atomic::Ordering::Relaxed);
                state.slots.clear();
                state.full = true;
                state.dirty = true;
                eprintln!("[haus] Bildschirm-Skalierung: {neu}×");
            }
        }
    }
}
delegate_noop!(@<C: Client + 'static> House<C>: ignore wl_shm::WlShm);
delegate_noop!(@<C: Client + 'static> House<C>: ignore wl_shm_pool::WlShmPool);
delegate_noop!(@<C: Client + 'static> House<C>: ignore zwlr_layer_shell_v1::ZwlrLayerShellV1);
delegate_noop!(@<C: Client + 'static> House<C>: ignore wl_data_device_manager::WlDataDeviceManager);

/// Release tracking: the compositor returns a buffer, the
/// slot becomes free again. The generation protects against late
/// messages about long-buried buffers.
impl<C: Client + 'static> Dispatch<wl_callback::WlCallback, ()> for House<C> {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.frame_pending = false;
            state.tick.done_at = Some(std::time::Instant::now());
        }
    }
}

impl<C: Client + 'static> Dispatch<wl_buffer::WlBuffer, (u32, usize)> for House<C> {
    fn event(
        state: &mut Self,
        buffer: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        &(gen, idx): &(u32, usize),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = event {
            // Round 51 (03.11.2026): a pane buffer (`draw_pane`, marked
            // u32::MAX) is made fresh for every picture and used once —
            // released, it must be destroyed. It never was: the compositor
            // kept every one with its memfd (Duo: 944 «matrix-shell» buffers,
            // niri at 510 MB and 1066 fds after 10 minutes).
            if gen == u32::MAX {
                buffer.destroy();
                return;
            }
            // Round 92: a kept pane buffer (pane id << 8 | slot) is free again.
            if gen == PANE_BUF {
                let (id, si) = ((idx >> 8) as u32, idx & 0xff);
                if let Some(slot) = state.panes.iter_mut().find(|p| p.id == id).and_then(|p| p.bufs.get_mut(si)) {
                    slot.busy = false;
                }
                return;
            }
            if gen == state.gen {
                if let Some(slot) = state.slots.get_mut(idx) {
                    slot.busy = false;
                }
            }
        }
    }
}

impl<C: Client + 'static> Dispatch<wl_region::WlRegion, ()> for House<C> {
    fn event(
        _: &mut Self,
        _: &wl_region::WlRegion,
        _: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

/// The rasteriser: blend laid quads into the XRGB buffer —
/// full rounded-box distance function (negative inside), 1 px
/// smoothing, optionally clipped to the patch.
/// Camp 64: blend a rounded area into the buffer — with a
/// SOFT edge. `softness` 0 means: the rasteriser's 1 px smoothing
/// (a quad). Larger means: the distance to the edge
/// is faded out over this stretch, by smoothstep —
/// the S-curve that iced's almost linear falloff lacks and because of which
/// the iced version has to stack THREE layers.
#[allow(clippy::too_many_arguments)]
fn spread(
    buf: &mut [u8],
    bw: u32,
    bh: u32,
    bounds: (f32, f32, f32, f32),
    radius: f32,
    color: matrixkit_theme::Rgba,
    softness: f32,
    lo: (f32, f32),
    hi: (f32, f32),
) {
    let (bx, by, bwd, bht) = bounds;
    let (lo_x, lo_y) = lo;
    let (hi_x, hi_y) = hi;
    if lo_x >= hi_x || lo_y >= hi_y {
        return;
    }
    let (x0, y0) = (lo_x.max(0.0) as u32, lo_y.max(0.0) as u32);
    let (x1, y1) = (hi_x.ceil().min(bw as f32) as u32, hi_y.ceil().min(bh as f32) as u32);
    let r = radius.min(bwd / 2.0).min(bht / 2.0).max(0.0);
    let (cx, cy) = (bx + bwd / 2.0, by + bht / 2.0);
    let (hx, hy) = (bwd / 2.0 - r, bht / 2.0 - r);
    for py in y0..y1 {
        for px in x0..x1 {
            let qx = (px as f32 + 0.5 - cx).abs() - hx;
            let qy = (py as f32 + 0.5 - cy).abs() - hy;
            let dist =
                (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r;
            let cover = if softness <= 0.0 {
                (0.5 - dist).clamp(0.0, 1.0)
            } else {
                let t = (1.0 - dist / softness).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            };
            if cover <= 0.0 {
                continue;
            }
            let a = color.a * cover;
            let i = ((py * bw + px) * 4) as usize;
            if i + 3 >= buf.len() {
                continue;
            }
            buf[i] = (color.b * 255.0 * a + buf[i] as f32 * (1.0 - a)) as u8;
            buf[i + 1] = (color.g * 255.0 * a + buf[i + 1] as f32 * (1.0 - a)) as u8;
            buf[i + 2] = (color.r * 255.0 * a + buf[i + 2] as f32 * (1.0 - a)) as u8;
            buf[i + 3] = (255.0 * a + buf[i + 3] as f32 * (1.0 - a)) as u8;
        }
    }
}

pub fn paint(placed: &[Placed], buf: &mut [u8], bw: u32, bh: u32, clip: Option<&Rect>) {
    for p in placed {
        // The shadow first: it lies UNDER its quad, offset
        // and soft (camp 64). View boxes apply to it as to
        // everything else.
        if let Some(sh) = p.shadow {
            let (mut sx0, mut sy0) = (p.x + sh.dx - sh.blur, p.y + sh.dy - sh.blur);
            let (mut sx1, mut sy1) =
                (p.x + sh.dx + p.w + sh.blur, p.y + sh.dy + p.h + sh.blur);
            for c in [clip.copied(), p.clip].into_iter().flatten() {
                sx0 = sx0.max(c.x);
                sy0 = sy0.max(c.y);
                sx1 = sx1.min(c.x + c.w);
                sy1 = sy1.min(c.y + c.h);
            }
            spread(
                buf,
                bw,
                bh,
                (p.x + sh.dx, p.y + sh.dy, p.w, p.h),
                p.radius,
                sh.color,
                sh.blur,
                (sx0, sy0),
                (sx1, sy1),
            );
        }
        let (mut lo_x, mut lo_y) = (p.x.max(0.0), p.y.max(0.0));
        let (mut hi_x, mut hi_y) = ((p.x + p.w).min(bw as f32), (p.y + p.h).min(bh as f32));
        if let Some(c) = clip {
            lo_x = lo_x.max(c.x);
            lo_y = lo_y.max(c.y);
            hi_x = hi_x.min(c.x + c.w);
            hi_y = hi_y.min(c.y + c.h);
            if lo_x >= hi_x || lo_y >= hi_y {
                continue;
            }
        }
        // The quad's own view box (camp 29): scrolling
        // lists end hard at their edge.
        if let Some(c) = &p.clip {
            lo_x = lo_x.max(c.x);
            lo_y = lo_y.max(c.y);
            hi_x = hi_x.min(c.x + c.w);
            hi_y = hi_y.min(c.y + c.h);
            if lo_x >= hi_x || lo_y >= hi_y {
                continue;
            }
        }
        let (x0, y0) = (lo_x as u32, lo_y as u32);
        let (x1, y1) = (hi_x.ceil().min(bw as f32) as u32, hi_y.ceil().min(bh as f32) as u32);
        let r = p.radius.min(p.w / 2.0).min(p.h / 2.0);
        let (cx, cy) = (p.x + p.w / 2.0, p.y + p.h / 2.0);
        let (hx, hy) = (p.w / 2.0 - r, p.h / 2.0 - r);
        for py in y0..y1 {
            for px in x0..x1 {
                let qx = (px as f32 + 0.5 - cx).abs() - hx;
                let qy = (py as f32 + 0.5 - cy).abs() - hy;
                let dist =
                    (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r;
                let cover = (0.5 - dist).clamp(0.0, 1.0);
                if cover <= 0.0 {
                    continue;
                }
                let a = p.color.a * cover;
                let i = ((py * bw + px) * 4) as usize;
                buf[i] = (p.color.b * 255.0 * a + buf[i] as f32 * (1.0 - a)) as u8;
                buf[i + 1] = (p.color.g * 255.0 * a + buf[i + 1] as f32 * (1.0 - a)) as u8;
                buf[i + 2] = (p.color.r * 255.0 * a + buf[i + 2] as f32 * (1.0 - a)) as u8;
                // Carry alpha along (camp 23): XRGB ignores the
                // byte, ARGB needs it — premultiplied.
                buf[i + 3] = (255.0 * a + buf[i + 3] as f32 * (1.0 - a)) as u8;
            }
        }
    }
}

/// Painting in z order: quads, pictures, curves and texts in
/// ONE order. Without damage everything, with damage only the
/// patches (citizens whose home the damage touches).
#[allow(clippy::too_many_arguments)]
/// Stretch a scene into PIXELS (camp 87).
///
/// The client computes in logical pixels — that is its good
/// right and makes it independent of the screen. The buffer, however, carries
/// pixels, and on a 2× screen there are twice as many.
/// Instead of multiplying at twenty paint spots, the scene is
/// stretched ONCE; afterwards everything is in pixels.
fn stretch(scene: &Scene, k: f32) -> Scene {
    let rect = |r: &Rect| Rect { x: r.x * k, y: r.y * k, w: r.w * k, h: r.h * k };
    Scene {
        placed: scene
            .placed
            .iter()
            .map(|p| Placed {
                x: p.x * k,
                y: p.y * k,
                w: p.w * k,
                h: p.h * k,
                radius: p.radius * k,
                clip: p.clip.as_ref().map(rect),
                shadow: p.shadow.as_ref().map(|d| matrixkit_theme::scene::Shadow {
                    dx: d.dx * k,
                    dy: d.dy * k,
                    blur: d.blur * k,
                    ..*d
                }),
                ..*p
            })
            .collect(),
        roles: scene.roles.clone(),
        texts: scene
            .texts
            .iter()
            .map(|t| Text {
                px: t.px * k,
                x: t.x * k,
                y: t.y * k,
                home: rect(&t.home),
                clip: t.clip.as_ref().map(rect),
                s: t.s.clone(),
                ..*t
            })
            .collect(),
        picts: scene
            .picts
            .iter()
            .map(|b| Pict {
                box_w: ((b.box_w as f32) * k).round() as u32,
                box_h: ((b.box_h as f32) * k).round() as u32,
                x: b.x * k,
                y: b.y * k,
                radius: b.radius * k,
                home: rect(&b.home),
                clip: b.clip.as_ref().map(rect),
                path: b.path.clone(),
                ..*b
            })
            .collect(),
        curves: scene
            .curves
            .iter()
            .map(|c| Curve {
                x: c.x * k,
                y: c.y * k,
                w: c.w * k,
                h: c.h * k,
                home: rect(&c.home),
                samples: c.samples.clone(),
                ..*c
            })
            .collect(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn compose(
    engine: &mut TextEngine,
    atlas: &mut ImageAtlas,
    scene: &Scene,
    damage: Option<&[Rect]>,
    canvas: &mut [u8],
    w: u32,
    h: u32,
    scale: f32,
) {
    // Camp 103: the DIFF path now stretches only what is TOUCHED.
    //
    // Camp 87 stretched the whole scene „once instead of at twenty
    // paint spots" — cleanly thought, but the clone copies EVERY frame
    // all Placed AND all text strings. In the terminal (small scene)
    // unmeasurable; in the Werft (thousands of Markdown words) the
    // 2.8 % blink bubble cost 20 ms BASE LOAD — the owner: „the animations
    // all stutter quite a bit." Now: a full paint stretches as before
    // (rarely), the diff checks touching LOGICALLY (cheap, clone-free)
    // and stretches only the hits one by one.
    let k = scale;
    let scaled = (k - 1.0).abs() > 1e-6;
    match damage {
        None => {
            let stretched;
            let scene = if scaled {
                stretched = stretch(scene, k);
                &stretched
            } else {
                scene
            };
            compose_pass(engine, atlas, scene, None, canvas, w, h);
        }
        Some(logical) => {
            // Damage in pixels for painting; touching logically.
            let dmg_px: Vec<Rect> = if scaled {
                logical
                    .iter()
                    .map(|r| Rect { x: r.x * k, y: r.y * k, w: r.w * k, h: r.h * k })
                    .collect()
            } else {
                logical.to_vec()
            };
            let touched = |home: &Rect| logical.iter().any(|r| r.intersects(home));
            let n = scene.placed.len();
            let mut texts_at: Vec<Vec<&Text>> = vec![Vec::new(); n];
            for t in &scene.texts {
                if t.anchor < n && touched(&t.home) {
                    texts_at[t.anchor].push(t);
                }
            }
            let mut picts_at: Vec<Vec<&Pict>> = vec![Vec::new(); n];
            for b in &scene.picts {
                if b.anchor < n && touched(&b.home) {
                    picts_at[b.anchor].push(b);
                }
            }
            let mut curves_at: Vec<Vec<&Curve>> = vec![Vec::new(); n];
            for c in &scene.curves {
                if c.anchor < n && touched(&c.home) {
                    curves_at[c.anchor].push(c);
                }
            }
            let rect_k = |r: &Rect| Rect { x: r.x * k, y: r.y * k, w: r.w * k, h: r.h * k };
            for (i, p) in scene.placed.iter().enumerate() {
                let p_home = Rect { x: p.x - 1.0, y: p.y - 1.0, w: p.w + 2.0, h: p.h + 2.0 };
                let quad_hit = touched(&p_home)
                    || p.shadow.is_some() && touched(&Rect::of(p));
                if quad_hit {
                    let p_s = if scaled {
                        Placed {
                            x: p.x * k,
                            y: p.y * k,
                            w: p.w * k,
                            h: p.h * k,
                            radius: p.radius * k,
                            clip: p.clip.as_ref().map(rect_k),
                            shadow: p.shadow.as_ref().map(|d| matrixkit_theme::scene::Shadow {
                                dx: d.dx * k,
                                dy: d.dy * k,
                                blur: d.blur * k,
                                ..*d
                            }),
                            ..*p
                        }
                    } else {
                        *p
                    };
                    for r in &dmg_px {
                        paint(std::slice::from_ref(&p_s), canvas, w, h, Some(r));
                    }
                }
                for b in &picts_at[i] {
                    // Round 125: a lens map is GPU-only live glass.
                    if lens::get(&b.path).is_some() {
                        continue;
                    }
                    let (bw, bh) = if scaled {
                        (((b.box_w as f32) * k).round() as u32, ((b.box_h as f32) * k).round() as u32)
                    } else {
                        (b.box_w, b.box_h)
                    };
                    if let Some((iw, ih)) = atlas.fetch(&b.path, bw, bh, b.fit) {
                        let bx = b.x * k + (bw as f32 - iw as f32) / 2.0;
                        let by = b.y * k + (bh as f32 - ih as f32) / 2.0;
                        let home_s = rect_k(&b.home);
                        atlas.paint(&b.path, bw, bh, b.fit, bx, by, b.radius * k, b.clip.as_ref().map(rect_k).or(Some(home_s)), canvas, w, h);
                    }
                }
                for c in &curves_at[i] {
                    if c.samples.len() >= 2 {
                        let (cw, chh) = (c.w * k, c.h * k);
                        if let Some(mut cv) = matrixkit_canvas::Canvas::new(cw as u32, chh as u32) {
                            let kn = c.samples.len();
                            let step = cw / (kn as f32 - 1.0);
                            let ymap = |v: f32| chh - 1.0 - (v / c.max.max(0.001)).clamp(0.0, 1.0) * (chh - 2.0);
                            let mut ground = matrixkit_canvas::Trace::new();
                            ground.move_to(0.0, chh);
                            for (j, v) in c.samples.iter().enumerate() {
                                ground.line_to(j as f32 * step, ymap(*v));
                            }
                            ground.line_to(cw, chh);
                            ground.close();
                            cv.fill(ground, matrixkit_theme::Rgba { a: 0.20, ..c.tint });
                            let mut line = matrixkit_canvas::Trace::new();
                            line.move_to(0.0, ymap(c.samples[0]));
                            for (j, v) in c.samples.iter().enumerate().skip(1) {
                                line.line_to(j as f32 * step, ymap(*v));
                            }
                            cv.stroke(line, c.tint, 2.0);
                            cv.blit(c.x * k, c.y * k, Some(rect_k(&c.home)), canvas, w, h);
                        }
                    }
                }
                for t in &texts_at[i] {
                    engine.paint_clip(
                        t.family,
                        &t.s,
                        t.px * k,
                        t.weight,
                        t.color,
                        t.x * k,
                        t.y * k,
                        t.clip.as_ref().map(rect_k),
                        canvas,
                        w,
                        h,
                    );
                }
            }
        }
    }
}

/// The FULL-PAINT path: an (already stretched) scene without damage.
fn compose_pass(
    engine: &mut TextEngine,
    atlas: &mut ImageAtlas,
    scene: &Scene,
    damage: Option<&[Rect]>,
    canvas: &mut [u8],
    w: u32,
    h: u32,
) {
    let n = scene.placed.len();
    let mut texts_at: Vec<Vec<&Text>> = vec![Vec::new(); n];
    for t in &scene.texts {
        if t.anchor < n {
            texts_at[t.anchor].push(t);
        }
    }
    let mut picts_at: Vec<Vec<&Pict>> = vec![Vec::new(); n];
    for b in &scene.picts {
        if b.anchor < n {
            picts_at[b.anchor].push(b);
        }
    }
    let mut curves_at: Vec<Vec<&Curve>> = vec![Vec::new(); n];
    for c in &scene.curves {
        if c.anchor < n {
            curves_at[c.anchor].push(c);
        }
    }
    let touched = |home: &Rect| match damage {
        None => true,
        Some(rects) => rects.iter().any(|r| r.intersects(home)),
    };
    for (i, p) in scene.placed.iter().enumerate() {
        match damage {
            None => paint(std::slice::from_ref(p), canvas, w, h, None),
            Some(rects) => {
                for r in rects {
                    paint(std::slice::from_ref(p), canvas, w, h, Some(r));
                }
            }
        }
        for b in &picts_at[i] {
            if touched(&b.home) && lens::get(&b.path).is_none() {
                if let Some((iw, ih)) = atlas.fetch(&b.path, b.box_w, b.box_h, b.fit) {
                    let bx = b.x + (b.box_w as f32 - iw as f32) / 2.0;
                    let by = b.y + (b.box_h as f32 - ih as f32) / 2.0;
                    atlas.paint(&b.path, b.box_w, b.box_h, b.fit, bx, by, b.radius, b.clip.or(Some(b.home)), canvas, w, h);
                }
            }
        }
        for c in &curves_at[i] {
            if touched(&c.home) && c.samples.len() >= 2 {
                if let Some(mut cv) = matrixkit_canvas::Canvas::new(c.w as u32, c.h as u32) {
                    let k = c.samples.len();
                    let step = c.w / (k as f32 - 1.0);
                    let ymap = |v: f32| c.h - 1.0 - (v / c.max.max(0.001)).clamp(0.0, 1.0) * (c.h - 2.0);
                    let mut ground = matrixkit_canvas::Trace::new();
                    ground.move_to(0.0, c.h);
                    for (j, v) in c.samples.iter().enumerate() {
                        ground.line_to(j as f32 * step, ymap(*v));
                    }
                    ground.line_to(c.w, c.h);
                    ground.close();
                    cv.fill(ground, matrixkit_theme::Rgba { a: 0.20, ..c.tint });
                    let mut line = matrixkit_canvas::Trace::new();
                    line.move_to(0.0, ymap(c.samples[0]));
                    for (j, v) in c.samples.iter().enumerate().skip(1) {
                        line.line_to(j as f32 * step, ymap(*v));
                    }
                    cv.stroke(line, c.tint, 2.0);
                    cv.blit(c.x, c.y, Some(c.home), canvas, w, h);
                }
            }
        }
        for t in &texts_at[i] {
            if touched(&t.home) {
                engine.paint_clip(t.family, &t.s, t.px, t.weight, t.color, t.x, t.y, t.clip, canvas, w, h);
            }
        }
    }
}

/// Expand a patch to WHOLE logical pixels (camp 89).
///
/// The diff has two halves, and they round differently: the
/// reset to the ground rounds logically and multiplies
/// by the scale afterwards, the repaint multiplies first and
/// then truncates. On a 2× screen they thus gape apart by up
/// to one pixel — and what is reset but
/// not repainted stays BLACK (the ground is a
/// zero buffer). Whole logical pixels make both roundings the
/// same computation.
fn snap(r: &Rect) -> Rect {
    let (x0, y0) = (r.x.floor(), r.y.floor());
    let (x1, y1) = ((r.x + r.w).ceil(), (r.y + r.h).ceil());
    Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
}

/// Integer edges of a patch, clamped to the buffer size.
/// Round 87: only square-cornered, shadowless quads on whole logical
/// pixels — nothing a higher buffer scale would draw any sharper.
pub fn is_flat(scene: &Scene) -> bool {
    scene.texts.is_empty()
        && scene.picts.is_empty()
        && scene.curves.is_empty()
        && scene.placed.iter().all(|q| {
            q.radius <= 0.0 && q.shadow.is_none() && [q.x, q.y, q.w, q.h].iter().all(|v| (v - v.round()).abs() < 0.001)
        })
}

/// Round 98: is memory short right now (mk::memory, PSI or what is left)?
fn mk_memory_high() -> bool {
    matrixkit_theme::memory::pressure() == Some(matrixkit_theme::memory::Pressure::High)
}

/// Round 104: the fleet's GPU device (made on first use). False = no GPU
/// here (or MATRIX_CPU) — callers paint on the CPU.
fn fleet_gpu<C: Client + 'static>(state: &mut House<C>, qh: &QueueHandle<House<C>>) -> bool {
    if state.pane_gpu_failed || std::env::var_os("MATRIX_CPU").is_some() {
        return false;
    }
    if state.pane_gpu.is_none() {
        let Some(comp) = state.compositor.clone() else { return false };
        matrixkit_theme::gpu_path::prepare();
        let helper = comp.create_surface(qh, ());
        let sp = helper.id().as_ptr() as *mut std::ffi::c_void;
        match unsafe { matrixkit_gpu::Gpu::new(state.display_ptr, sp, 1, 1) } {
            Ok(g) => {
                eprintln!("[{}] Flotte malt auf der GPU: {}", state.tag, g.adapter_info);
                state.pane_gpu = Some((g, helper));
            }
            Err(e) => {
                eprintln!("[{}] Flotten-GPU fehlgeschlagen ({e}) — CPU bleibt", state.tag);
                helper.destroy();
                state.pane_gpu_failed = true;
                return false;
            }
        }
    }
    true
}

/// Round 104: draw `sc` with the fleet GPU into `surface` at buffer scale
/// `k` — the card's ZOOM picture, as sharp as the card itself (it was a
/// CPU picture at scale 1, stretched by the compositor: blurred while the
/// card grew, then a jump to sharp — «looks strange»).
#[allow(clippy::too_many_arguments)]
fn gpu_picture<C: Client + 'static>(state: &mut House<C>, qh: &QueueHandle<House<C>>, engine: &mut TextEngine, surface: &wl_surface::WlSurface, sc: &Scene, w: u32, h: u32, k: u32) -> Option<matrixkit_gpu::Target> {
    if !fleet_gpu(state, qh) {
        return None;
    }
    let (bw, bh) = (w * k, h * k);
    let sp = surface.id().as_ptr() as *mut std::ffi::c_void;
    let mut target = match state.pane_gpu.as_ref().map(|(g, _)| unsafe { g.add_target(state.display_ptr, sp, bw, bh) })? {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[{}] Zoom-Ziel fehlgeschlagen ({e}) — CPU-Bild", state.tag);
            return None;
        }
    };
    let mut atlas = std::mem::take(&mut state.atlas);
    let mut holder = Vec::new();
    let res = {
        let (quads, runs, texts, images, layers) = gpu_lists(sc, &mut atlas, &mut holder, w, h, k);
        let mut hands = Some(engine.gpu_handles());
        surface.set_buffer_scale(k as i32);
        let (gpu, _) = state.pane_gpu.as_mut()?;
        gpu.with_target(&mut target, |g| {
            g.resize(bw, bh);
            g.corner_radius = 0.0;
            g.render_frame(&mut hands, [0.0, 0.0, 0.0, 0.0], &quads, &runs, &images, None, None, &texts, &layers, k as f32)
        })
    };
    state.atlas = atlas;
    match res {
        Ok(_) => Some(target),
        Err(e) => {
            eprintln!("[{}] Zoom GPU: {e}", state.tag);
            None
        }
    }
}

/// Round 94: paint pane `idx` on the fleet GPU. False = not possible (the
/// caller paints on the CPU).
#[allow(clippy::too_many_arguments)]
fn draw_pane_gpu<C: Client + 'static>(state: &mut House<C>, idx: usize, qh: &QueueHandle<House<C>>, engine: &mut TextEngine, sc: &Scene, w: u32, h: u32, k: u32) -> bool {
    if !fleet_gpu(state, qh) {
        return false;
    }
    let (bw, bh) = (w * k, h * k);
    if state.panes[idx].target.is_none() {
        let sp = state.panes[idx].surface.id().as_ptr() as *mut std::ffi::c_void;
        let made = state.pane_gpu.as_ref().map(|(g, _)| unsafe { g.add_target(state.display_ptr, sp, bw, bh) });
        match made {
            Some(Ok(t)) => state.panes[idx].target = Some(t),
            Some(Err(e)) => {
                eprintln!("[{}] Flotten-Ziel fehlgeschlagen ({e}) — CPU bleibt", state.tag);
                state.pane_gpu_failed = true;
                return false;
            }
            None => return false,
        }
    }
    let mut atlas = std::mem::take(&mut state.atlas);
    let mut holder = Vec::new();
    let done = {
        let (quads, runs, texts, images, layers) = gpu_lists(sc, &mut atlas, &mut holder, w, h, k);
        let mut hands = Some(engine.gpu_handles());
        let mut target = state.panes[idx].target.take().expect("Ziel liegt");
        state.panes[idx].surface.set_buffer_scale(k as i32);
        let (gpu, _) = state.pane_gpu.as_mut().expect("GPU liegt");
        let res = gpu.with_target(&mut target, |g| {
            g.resize(bw, bh);
            g.corner_radius = 0.0;
            g.render_frame(&mut hands, [0.0, 0.0, 0.0, 0.0], &quads, &runs, &images, None, None, &texts, &layers, k as f32)
        });
        state.panes[idx].target = Some(target);
        match res {
            Ok(_) => true,
            Err(e) => {
                eprintln!("[{}] Flotte GPU: {e}", state.tag);
                false
            }
        }
    };
    state.atlas = atlas;
    done
}

/// Round 91: did the pane SCROLL? Most changed quads kept their clip, size
/// and colour and moved by the same dy inside that clip — then the clip is
/// the roll area. (Cards and the control centre repainted 2.9 million
/// pixels per scroll frame: «scrolling stutters», the owner 04.11.2026.)
pub fn pane_roll(old: &Scene, new: &Scene) -> Option<(Rect, f32)> {
    use std::collections::HashMap;
    let mut votes: HashMap<(u32, u32, u32, u32, i64), (usize, Rect, f32)> = HashMap::new();
    let mut changed = 0usize;
    for (a, b) in old.placed.iter().zip(&new.placed) {
        if a == b {
            continue;
        }
        changed += 1;
        let (Some(ca), Some(cb)) = (a.clip, b.clip) else { continue };
        if ca != cb || a.x != b.x || a.w != b.w || a.h != b.h || a.color != b.color || a.radius != b.radius {
            continue;
        }
        let dy = b.y - a.y;
        if dy.abs() < 0.01 {
            continue;
        }
        let key = (ca.x.to_bits(), ca.y.to_bits(), ca.w.to_bits(), ca.h.to_bits(), (dy * 64.0).round() as i64);
        votes.entry(key).or_insert((0, ca, dy)).0 += 1;
    }
    let (n, clip, dy) = votes.into_values().max_by_key(|v| v.0)?;
    (n >= 3 && n * 2 >= changed && dy.abs() < clip.h * 0.9).then_some((clip, dy))
}

/// Round 91: the old scene as it looks after scrolling `area` by `dy` —
/// every citizen clipped to `area` moves along; what then still differs
/// from the new scene is real change.
fn rolled_copy(old: &Scene, area: Rect, dy: f32) -> Scene {
    let moved = |c: &Option<Rect>| *c == Some(area);
    let shift = |r: Rect| Rect { y: r.y + dy, ..r };
    Scene {
        placed: old.placed.iter().map(|q| if moved(&q.clip) { Placed { y: q.y + dy, ..*q } } else { *q }).collect(),
        roles: Vec::new(),
        texts: old.texts.iter().map(|t| { let mut t = t.clone(); if moved(&t.clip) { t.y += dy; t.home = shift(t.home); } t }).collect(),
        picts: old.picts.iter().map(|b| { let mut b = b.clone(); if moved(&b.clip) { b.y += dy; b.home = shift(b.home); } b }).collect(),
        curves: old.curves.iter().map(|c| { let mut c = c.clone(); if c.home.intersects(&area) { c.y += dy; c.home = shift(c.home); } c }).collect(),
    }
}

#[cfg(test)]
mod roll_tests {
    use super::*;

    fn list(roll: f32) -> Scene {
        let mut b = SceneBuilder::new();
        let clip = Some(Rect { x: 0.0, y: 50.0, w: 300.0, h: 400.0 });
        b.quad(0.0, 0.0, 300.0, 40.0, matrixkit_theme::Rgba { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }, 0.0, None, None);
        for i in 0..20 {
            b.quad(10.0, 60.0 + i as f32 * 56.0 - roll, 280.0, 48.0, matrixkit_theme::Rgba { r: 0.2, g: 0.2, b: 0.2, a: 1.0 }, 12.0, Some(i), clip);
        }
        b.finish()
    }

    #[test]
    fn a_scroll_is_seen_and_leaves_no_damage_but_the_strip() {
        // a finger moves by fractions of a pixel
        let (frac_old, frac_new) = (list(0.0), list(7.4));
        let (fa, fdy) = pane_roll(&frac_old, &frac_new).expect("a fractional scroll");
        assert!(diff(&rolled_copy(&frac_old, fa, fdy).placed, &frac_new.placed).is_empty());
        let (old, new) = (list(0.0), list(30.0));
        let (area, dy) = pane_roll(&old, &new).expect("a scroll");
        assert_eq!((area.y, area.h, dy), (50.0, 400.0, -30.0));
        let moved = rolled_copy(&old, area, dy);
        assert!(diff(&moved.placed, &new.placed).is_empty());
        // a mere colour change is no scroll
        assert!(pane_roll(&old, &old).is_none());
    }
}

#[cfg(test)]
mod flat_tests {
    use super::*;

    #[test]
    fn a_veil_is_flat_a_card_is_not() {
        let mut veil = SceneBuilder::new();
        veil.quad(0.0, 0.0, 652.0, 489.0, matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.4 }, 0.0, Some(0), None);
        assert!(is_flat(&veil.finish()));
        let mut card = SceneBuilder::new();
        card.quad(8.0, 8.0, 300.0, 200.0, matrixkit_theme::Rgba { r: 0.2, g: 0.2, b: 0.2, a: 1.0 }, 22.0, None, None);
        assert!(!is_flat(&card.finish()));
        let mut half = SceneBuilder::new();
        half.quad(0.5, 0.0, 10.0, 10.0, matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, 0.0, None, None);
        assert!(!is_flat(&half.finish()));
    }
}

fn edges(r: &Rect, bw: u32, bh: u32) -> (u32, u32, u32, u32) {
    let x0 = (r.x.max(0.0).floor() as u32).min(bw);
    let y0 = (r.y.max(0.0).floor() as u32).min(bh);
    let x1 = ((r.x + r.w).ceil().max(0.0) as u32).clamp(x0, bw);
    let y1 = ((r.y + r.h).ceil().max(0.0) as u32).clamp(y0, bh);
    (x0, y0, x1, y1)
}

/// Home bookkeeping of a citizen pair: a change makes the old AND
/// new area damage.
fn burden<T: PartialEq>(old: &[T], new: &[T], home: impl Fn(&T) -> Rect, damage: &mut Vec<Rect>) {
    let pairs = old.len().max(new.len());
    for i in 0..pairs {
        match (old.get(i), new.get(i)) {
            (Some(a), Some(b)) if a == b => {}
            (a, b) => {
                if let Some(a) = a {
                    damage.push(home(a));
                }
                if let Some(b) = b {
                    damage.push(home(b));
                }
            }
        }
    }
    for b in new {
        if damage.iter().any(|r| r.intersects(&home(b))) {
            damage.push(home(b));
        }
    }
}

/// Like `burden`, but MULTISET-paired (camp 104): when scrolling
/// the citizens move within the vector (culling), index pairing would see everything
/// as damage. The runner first tries the spot after the
/// last hit — in the normal case linear instead of quadratic.
fn burden_pool<T>(
    old: &[T],
    new: &[T],
    eq: impl Fn(&T, &T) -> bool,
    home: impl Fn(&T) -> Rect,
    cascade: impl Fn(&T) -> bool,
    damage: &mut Vec<Rect>,
) {
    // Degenerate homes (cut to 0 by the view box —
    // citizens scrolled out) are no damage, just ballast.
    let push = |damage: &mut Vec<Rect>, r: Rect| {
        if r.w > 0.0 && r.h > 0.0 {
            damage.push(r);
        }
    };
    let mut taken = vec![false; new.len()];
    let mut cursor = 0usize;
    for a in old {
        let hit = if cursor < new.len() && !taken[cursor] && eq(&new[cursor], a) {
            Some(cursor)
        } else {
            (0..new.len()).find(|&j| !taken[j] && eq(&new[j], a))
        };
        match hit {
            Some(j) => {
                taken[j] = true;
                cursor = j + 1;
            }
            None => push(damage, home(a)),
        }
    }
    for (j, b) in new.iter().enumerate() {
        if !taken[j] {
            push(damage, home(b));
        }
    }
    // The CASCADE: whoever lies over damage and needs the fresh
    // ground beneath when repainting goes entirely into the damage.
    // ONLY those that really need it (translucent, rounded) —
    // the window-filling ground quad otherwise cut EVERY damage
    // and tore every scroll round up to 100 % (stage probe: quads+350,
    // diff 102.9 %).
    for b in new {
        if cascade(b) && damage.iter().any(|r| r.intersects(&home(b))) {
            push(damage, home(b));
        }
    }
}

/// Close enough for scroll pairing (camp 104c): f32 is not
/// associative — `(pane.y - off) + Σ` rounds differently from
/// `pane.y + Σ` minus dy —, and `anchor` is the Placed INDEX that
/// culling shifts for everyone. Exact equality therefore pairs
/// NOTHING when scrolling (live finding: dy=-170, 1018 patches,
/// 103 % damage). A hundredth of a pixel tolerance, anchor stays
/// out of it — painting comes from the NEW scene anyway.
fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

fn rect_near(a: &Rect, b: &Rect) -> bool {
    near(a.x, b.x) && near(a.y, b.y) && near(a.w, b.w) && near(a.h, b.h)
}

fn clip_near(a: &Option<Rect>, b: &Option<Rect>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => rect_near(a, b),
        _ => false,
    }
}

fn placed_near(a: &Placed, b: &Placed) -> bool {
    near(a.x, b.x)
        && near(a.y, b.y)
        && near(a.w, b.w)
        && near(a.h, b.h)
        && a.color == b.color
        && a.radius == b.radius
        && clip_near(&a.clip, &b.clip)
        && a.shadow == b.shadow
}

fn text_near(a: &Text, b: &Text) -> bool {
    a.s == b.s
        && a.px == b.px
        && a.weight == b.weight
        && a.family == b.family
        && a.color == b.color
        && near(a.x, b.x)
        && near(a.y, b.y)
        && rect_near(&a.home, &b.home)
        && clip_near(&a.clip, &b.clip)
}

fn pict_near(a: &Pict, b: &Pict) -> bool {
    a.path == b.path
        && a.box_w == b.box_w
        && a.box_h == b.box_h
        && a.fit == b.fit
        && a.radius == b.radius
        && near(a.x, b.x)
        && near(a.y, b.y)
        && rect_near(&a.home, &b.home)
        && clip_near(&a.clip, &b.clip)
}

/// The VISIBLE hull: the home, clipped at the view box.
fn visible(home: Rect, clip: Option<Rect>) -> Rect {
    match clip {
        None => home,
        Some(c) => {
            let x0 = home.x.max(c.x);
            let y0 = home.y.max(c.y);
            let x1 = (home.x + home.w).min(c.x + c.w);
            let y1 = (home.y + home.h).min(c.y + c.h);
            Rect { x: x0, y: y0, w: (x1 - x0).max(0.0), h: (y1 - y0).max(0.0) }
        }
    }
}

/// A flat GROUND does not blunt the blit (camp 104,
/// second attempt): a quad without rounding and shadow that
/// contains the area ENTIRELY is vertically a single colour — moving
/// changes nothing there. Without this exception the ground quad
/// of the Werft (place 0, window-filling, no view box) discarded EVERY
/// scroll hint — and SILENTLY: zero roll-blit lines in the log
/// were the only hint (the owner's „still stutters",
/// 15:12).
fn is_flat_ground(q: &Placed, area: &Rect) -> bool {
    let covers = |r: &Rect| {
        r.x <= area.x
            && r.y <= area.y
            && r.x + r.w >= area.x + area.w
            && r.y + r.h >= area.y + area.h
    };
    q.radius == 0.0
        && q.shadow.is_none()
        && covers(&Rect { x: q.x, y: q.y, w: q.w, h: q.h })
        && q.clip.as_ref().is_none_or(covers)
}

/// The cached GPU conversion (camp 24): everything render_frame
/// needs, in its OWN values — repaint rounds (rain beat) paint
/// from it, without view() and without reshaping.
struct GpuSceneCache {
    quads: Vec<matrixkit_gpu::GpuQuad>,
    runs: Vec<matrixkit_gpu::QuadRun>,
    texts: Vec<OwnedGpuText>,
    layers: Vec<matrixkit_gpu::GpuLayer>,
}

struct OwnedGpuText {
    s: String,
    px: f32,
    line: f32,
    weight: u16,
    family: String,
    color: [f32; 4],
    x: f32,
    y: f32,
    bounds: Option<(i32, i32, i32, i32)>,
}

/// A scroll area is blit-CAPABLE only if every citizen that
/// visibly reaches into it carries EXACTLY its view box or is a
/// flat ground under everything (camp 104) — a veil, a
/// drag ghost, a detail card blunts the shortcut, and
/// the normal path computes.
fn rollable(s: &Scene, area: &Rect) -> bool {
    s.placed.iter().all(|q| {
        q.clip == Some(*area)
            || is_flat_ground(q, area)
            || !visible(Rect { x: q.x, y: q.y, w: q.w, h: q.h }, q.clip).intersects(area)
    }) && s
        .texts
        .iter()
        .all(|t| t.clip == Some(*area) || !visible(t.home, t.clip).intersects(area))
        && s.picts
            .iter()
            .all(|c| c.clip == Some(*area) || !visible(c.home, c.clip).intersects(area))
        && s.curves.iter().all(|c| !c.home.intersects(area))
}

/// Shifts the pixels of an area by dy logical pixels —
/// the core of the scroll blit (camp 104). Row by row copy_within,
/// from below when moving down, so that nothing overwrites
/// itself.
fn blit_rows(canvas: &mut [u8], area: &Rect, dy: i32, k: u32, w: u32, h: u32) {
    let (pw, ph) = (w * k, h * k);
    let x0 = (((area.x.max(0.0)) as u32) * k).min(pw);
    let x1 = ((((area.x + area.w).max(0.0)) as u32) * k).min(pw);
    let y0 = (((area.y.max(0.0)) as u32) * k).min(ph);
    let y1 = ((((area.y + area.h).max(0.0)) as u32) * k).min(ph);
    let d = (dy.unsigned_abs() * k) as usize;
    if x1 <= x0 || y1 <= y0 || d == 0 || d >= (y1 - y0) as usize {
        return;
    }
    let stride = (pw * 4) as usize;
    let (a, b) = ((x0 * 4) as usize, (x1 * 4) as usize);
    if dy > 0 {
        for yy in (y0 as usize + d..y1 as usize).rev() {
            let src = (yy - d) * stride;
            let dst = yy * stride;
            canvas.copy_within(src + a..src + b, dst + a);
        }
    } else {
        for yy in y0 as usize..y1 as usize - d {
            let src = (yy + d) * stride;
            let dst = yy * stride;
            canvas.copy_within(src + a..src + b, dst + a);
        }
    }
}

/// Round 93: BIG CORES WHILE TOUCHED (Android puts the touched app on the
/// fast cluster). On the Duo the bar's thread sat on a 1.8 GHz little core:
/// scroll frames median 53 ms, on the big cores 35 ms. A touch or a click
/// moves the house's thread to the fastest cores; 1.5 s of rest lets it go
/// back to all of them. A device with equal cores does nothing.
pub mod big_cores {
    use std::cell::Cell;
    use std::time::{Duration, Instant};

    thread_local! {
        static LAST: Cell<Option<Instant>> = const { Cell::new(None) };
        static ON: Cell<bool> = const { Cell::new(false) };
    }

    /// The cores whose top clock is within 80 % of the fastest one — None
    /// when that is every core (nothing to prefer).
    pub fn fast() -> Option<Vec<usize>> {
        let maxes: Vec<(usize, u64)> = (0..64)
            .filter_map(|c| {
                let f = std::fs::read_to_string(format!("/sys/devices/system/cpu/cpu{c}/cpufreq/cpuinfo_max_freq")).ok()?;
                Some((c, f.trim().parse().ok()?))
            })
            .collect();
        let top = maxes.iter().map(|m| m.1).max()?;
        let fast: Vec<usize> = maxes.iter().filter(|m| m.1 * 10 >= top * 8).map(|m| m.0).collect();
        (fast.len() < maxes.len()).then_some(fast)
    }

    fn set(cores: Option<&[usize]>) {
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            match cores {
                Some(list) => list.iter().for_each(|c| libc::CPU_SET(*c, &mut set)),
                None => (0..libc::CPU_SETSIZE as usize).for_each(|c| libc::CPU_SET(c, &mut set)),
            }
            libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);
        }
    }

    fn cores() -> Option<&'static [usize]> {
        static FAST: std::sync::OnceLock<Option<Vec<usize>>> = std::sync::OnceLock::new();
        FAST.get_or_init(fast).as_deref()
    }

    /// An input event: onto the fast cores (once per touch, then cheap).
    pub fn touched() {
        LAST.with(|l| l.set(Some(Instant::now())));
        if !ON.with(Cell::get) {
            if let Some(c) = cores() {
                set(Some(c));
                ON.with(|o| o.set(true));
            }
        }
    }

    /// From the beat: after 1.5 s without input, every core again.
    pub fn rest() {
        if ON.with(Cell::get) && LAST.with(Cell::get).is_none_or(|t| t.elapsed() > Duration::from_millis(1500)) {
            set(None);
            ON.with(|o| o.set(false));
        }
    }
}

/// Round 92: the buffer mark of a kept pane buffer (see `draw_pane`).
const PANE_BUF: u32 = u32::MAX - 1;

fn make_memfd(len: usize) -> Option<(std::os::fd::OwnedFd, memmap2::MmapMut)> {
    let fd = unsafe { libc::memfd_create(c"matrix-shell".as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return None;
    }
    let owned = unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) };
    unsafe {
        if libc::ftruncate(owned.as_raw_fd(), len as i64) < 0 {
            return None;
        }
    }
    let map = unsafe { memmap2::MmapMut::map_mut(&owned).ok()? };
    Some((owned, map))
}

fn make_slot<C: Client + 'static>(state: &House<C>, qh: &QueueHandle<House<C>>, idx: usize) -> Option<Slot> {
    // The buffer carries PIXELS, not logical pixels (camp 87):
    // on a 2× screen it is twice as big, and
    // set_buffer_scale tells the compositor NOT to
    // stretch it.
    let s = state.scale.max(1) as u32;
    let (w, h) = (state.size.0 * s, state.size.1 * s);
    let len = (w * h * 4) as usize;
    let (owned, map) = make_memfd(len)?;
    let pool = state.shm.as_ref()?.create_pool(owned.as_fd(), len as i32, qh, ());
    // R207: round corners need an alpha channel — in the window too.
    let format = if state.argb || state.corners > 0.0 { wl_shm::Format::Argb8888 } else { wl_shm::Format::Xrgb8888 };
    let buffer = pool.create_buffer(
        0,
        w as i32,
        h as i32,
        (w * 4) as i32,
        format,
        qh,
        (state.gen, idx),
    );
    // Pool + fd may drop: the compositor holds the pages, we
    // the mmap — SHM contract.
    pool.destroy();
    Some(Slot {
        buffer,
        map,
        stale: vec![Rect { x: 0.0, y: 0.0, w: w as f32, h: h as f32 }],
        busy: false,
    })
}

/// R207: the OPAQUE region of a window in a foreign host — everything
/// except the four corner squares. A compositor that knows it need not
/// blend anything behind it and knows exactly which area the window
/// covers (KWin: suspected stripes when moving). On the canvas
/// nothing changes.
fn opaque_region<C: Client + 'static>(
    compositor: Option<&wl_compositor::WlCompositor>,
    is_window: bool,
    argb: bool,
    corners: f32,
    qh: &QueueHandle<House<C>>,
    surface: &wl_surface::WlSurface,
    w: u32,
    h: u32,
) {
    if !is_window || matrixkit_theme::habitat::is_matrix() || argb {
        return;
    }
    let Some(compositor) = compositor else { return };
    let region = compositor.create_region(qh, ());
    let r = corners.ceil() as i32;
    let (w, h) = (w as i32, h as i32);
    if r <= 0 || w <= 2 * r || h <= 2 * r {
        region.add(0, 0, w, h);
    } else {
        region.add(r, 0, w - 2 * r, h);
        region.add(0, r, w, h - 2 * r);
    }
    surface.set_opaque_region(Some(&region));
    region.destroy();
}

/// R207: recompute the SEED of the foreign-host palette — from the
/// wallpaper (if there is one), otherwise from the accent colour —
/// and write it into the seed file; the clients read it in the pulse.
fn reseed(tag: &'static str, wallpaper: Option<std::path::PathBuf>) {
    std::thread::spawn(move || {
        let seed = wallpaper
            .as_deref()
            .and_then(matrixkit_image::seed_of)
            .or_else(matrixkit_theme::seed::accent_color);
        match seed {
            Some(c) => {
                let hex = format!(
                    "#{:02x}{:02x}{:02x}",
                    (c.r * 255.0).round() as u8,
                    (c.g * 255.0).round() as u8,
                    (c.b * 255.0).round() as u8
                );
                let _ = std::fs::write(matrixkit_theme::seed::seed_path(), &hex);
                eprintln!(
                    "[{tag}] Palette im Gastland: Samen {hex} aus {}",
                    wallpaper.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| String::from("der Akzentfarbe"))
                );
            }
            None => eprintln!("[{tag}] Palette im Gastland: kein Samen gefunden — Standardpalette"),
        }
    });
}

/// R207: did the desktop change its wallpaper? Then seed anew.
fn watch_wallpaper<C: Client + 'static>(state: &mut House<C>) {
    let path = matrixkit_theme::seed::wallpaper_path();
    let stamp = path.as_ref().and_then(|p| std::fs::metadata(p).ok()).and_then(|m| m.modified().ok());
    let now = match (path, stamp) {
        (Some(p), Some(t)) => Some((p, t)),
        _ => None,
    };
    if now != state.seed_watch {
        if state.seed_watch.is_some() || now.is_some() {
            reseed(state.tag, now.as_ref().map(|(p, _)| p.clone()));
        }
        state.seed_watch = now;
    }
}

/// R207: round the four corners of an ARGB buffer (premultiplied)
/// — coverage from the distance to the corner centre, one pixel
/// smoothing. Only the four corner squares are touched.
fn round_corners(map: &mut [u8], pw: u32, ph: u32, r: f32) {
    let ri = r.ceil() as u32;
    if ri == 0 || pw < 2 * ri || ph < 2 * ri {
        return;
    }
    for cy in 0..ri {
        for cx in 0..ri {
            // Distance from the pixel centre to the circle centre (r, r).
            let dx = r - (cx as f32 + 0.5);
            let dy = r - (cy as f32 + 0.5);
            let d = (dx * dx + dy * dy).sqrt();
            let cover = (r + 0.5 - d).clamp(0.0, 1.0);
            if cover >= 1.0 {
                continue;
            }
            for (x, y) in [
                (cx, cy),
                (pw - 1 - cx, cy),
                (cx, ph - 1 - cy),
                (pw - 1 - cx, ph - 1 - cy),
            ] {
                let o = ((y * pw + x) * 4) as usize;
                for c in 0..4 {
                    map[o + c] = (map[o + c] as f32 * cover + 0.5) as u8;
                }
            }
        }
    }
}

/// Bring the master copy to the compositor: pick a free slot,
/// catch up stale patches, report ONLY these as damage.
fn present<C: Client + 'static>(state: &mut House<C>, qh: &QueueHandle<House<C>>) -> Option<()> {
    let (w, h) = state.size;
    if state.slots.iter().all(|s| s.stale.is_empty()) && !state.slots.is_empty() {
        return Some(());
    }
    let idx = match state.slots.iter().position(|s| !s.busy) {
        Some(i) => i,
        None if state.slots.len() < 2 => {
            let slot = make_slot(state, qh, state.slots.len())?;
            state.slots.push(slot);
            state.slots.len() - 1
        }
        None => {
            eprintln!("[{}] beide Puffer beim Compositor — warte auf Release", state.tag);
            return None;
        }
    };
    let corners = state.corners;
    let slot = &mut state.slots[idx];
    let stale = coalesce(std::mem::take(&mut slot.stale));
    // Patches are logical, buffer and canvas carry pixels.
    let k = state.scale.max(1) as u32;
    let pw = w * k;
    for r in &stale {
        let (x0, y0, x1, y1) = edges(r, w, h);
        for y in (y0 * k)..(y1 * k) {
            let a = ((y * pw + x0 * k) * 4) as usize;
            let b = ((y * pw + x1 * k) * 4) as usize;
            slot.map[a..b].copy_from_slice(&state.canvas[a..b]);
        }
    }
    // R207: round the corners IN THE BUFFER — the canvas stays square,
    // every patch that touches a corner is clipped anew when
    // copying. The compositor never sees a square corner.
    if corners > 0.0 {
        round_corners(&mut slot.map, pw, h * k, corners * k as f32);
    }
    slot.busy = true;
    let buffer = slot.buffer.clone();
    // The WINDOW GEOMETRY belongs with the buffer, not before it: whoever
    // reports it in advance and sends it empty does not grow again (the
    // unrolling of the window key got stuck that way — the compositor
    // held on to the small size, 16.08.2026).
    if let Some(xdg) = state.xdg.clone() {
        xdg.set_window_geometry(0, 0, w as i32, h as i32);
    }
    if let Some(vp) = &state.viewport {
        vp.set_destination(w as i32, h as i32);
    }
    let surface = state.surface.as_ref()?;
    opaque_region(state.compositor.as_ref(), state.toplevel.is_some(), state.argb, state.corners, qh, surface, w, h);
    surface.set_buffer_scale(state.scale.max(1));
    surface.attach(Some(&buffer), 0, 0);
    for r in &stale {
        let (x0, y0, x1, y1) = edges(r, w, h);
        surface.damage_buffer(
            (x0 * k) as i32,
            (y0 * k) as i32,
            ((x1 - x0) * k) as i32,
            ((y1 - y0) * k) as i32,
        );
    }
    surface.commit();
    Some(())
}

/// Does `q` begin a new GPU layer? (A citizen that may lie over earlier
/// text — a veil over 60 % of the window, a floating thing with a shadow,
/// an explicit `SceneBuilder::rise`.)
fn starts_layer(q: &Placed, window_area: f32) -> bool {
    q.shadow.is_some() || q.w * q.h >= 0.6 * window_area
}

#[cfg(test)]
mod wiggle_tests {
    #[test]
    fn the_wiggle_trembles_a_little() {
        for k in 0..100 {
            let (dx, dy) = super::wiggle_offset(k as f32 * 0.013);
            assert!(dx.abs() <= 2.3 && dy.abs() <= 1.7);
        }
        assert_ne!(super::wiggle_offset(0.02), super::wiggle_offset(0.05), "it moves");
    }
}

#[cfg(test)]
mod rise_tests {
    use super::*;

    #[test]
    fn rise_begins_invisible() {
        let mut b = SceneBuilder::new();
        let ground = b.quad(0.0, 0.0, 400.0, 800.0, matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, 0.0, None, None);
        let slider = b.quad(10.0, 600.0, 300.0, 6.0, matrixkit_theme::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, 3.0, None, None);
        let top = b.rise();
        let scene = b.finish();
        let area = 400.0 * 800.0;
        assert!(!starts_layer(&scene.placed[slider], area), "a plain quad stays in the ground's layer");
        assert!(starts_layer(&scene.placed[top], area), "rise() begins one");
        assert_eq!(scene.placed[top].color.a, 0.0, "and shows nothing");
        assert!(scene.placed[top].w * scene.placed[top].h == 0.0, "nor takes a touch");
        assert_eq!(ground, 0);
    }
}

/// Round 94: a scene as the GPU's lists (quads in runs, texts and pictures in
/// layers) — ONE conversion for the main surface and the fleet panes. Moved
/// verbatim out of the main surface's draw. `curve_holder` keeps the curves'
/// pixels alive as long as the lists borrow them.
#[allow(clippy::type_complexity)]
fn gpu_lists<'a>(
    v: &'a Scene,
    atlas: &'a mut ImageAtlas,
    curve_holder: &'a mut Vec<(String, Vec<u8>, u32, u32)>,
    w: u32,
    h: u32,
    k: u32,
) -> (Vec<matrixkit_gpu::GpuQuad>, Vec<matrixkit_gpu::QuadRun>, Vec<matrixkit_gpu::GpuText<'a>>, Vec<matrixkit_gpu::GpuImage<'a>>, Vec<matrixkit_gpu::GpuLayer>) {
    let kf = k as f32;
    curve_holder.clear();
    let mut quads = Vec::with_capacity(v.placed.len());
    let mut runs: Vec<matrixkit_gpu::QuadRun> = Vec::new();
    // LAYERS (stage 3): a citizen that may lie over earlier
    // text (veil from 60 % of the window area,
    // floating things with a shadow) begins a new layer —
    // otherwise the text beneath would rise through the ceiling.
    let mut layer_runs: Vec<u32> = vec![0];
    let mut layer_bounds: Vec<usize> = Vec::new();
    let window_area = (w * h) as f32;
    for (qi, q) in v.placed.iter().enumerate() {
        let clip = q.clip.map(|c| {
            let x = (c.x.max(0.0) * kf) as u32;
            let y = (c.y.max(0.0) * kf) as u32;
            (
                x,
                y,
                ((c.x + c.w).max(0.0) * kf).ceil() as u32 - x,
                ((c.y + c.h).max(0.0) * kf).ceil() as u32 - y,
            )
        });
        let idx = quads.len() as u32;
        if qi > 0 && starts_layer(q, window_area) {
            layer_bounds.push(qi);
            layer_runs.push(runs.len() as u32);
            // Force a run break: the layer starts fresh.
            runs.push(matrixkit_gpu::QuadRun { clip: None, range: idx..idx });
        }
        quads.push(matrixkit_gpu::GpuQuad {
            color: [q.color.r, q.color.g, q.color.b, q.color.a],
            pos: [q.x * kf, q.y * kf],
            size: [q.w * kf, q.h * kf],
            radius: q.radius * kf,
            shadow_blur: q.shadow.map_or(0.0, |sh| sh.blur * kf),
            shadow_offset: q
                .shadow
                .map_or([0.0, 0.0], |sh| [sh.dx * kf, sh.dy * kf]),
            shadow_color: q.shadow.map_or([0.0; 4], |sh| {
                [sh.color.r, sh.color.g, sh.color.b, sh.color.a]
            }),
        });
        match runs.last_mut() {
            Some(r) if r.clip == clip && !r.range.is_empty() => {
                r.range.end = idx + 1;
            }
            Some(r) if r.range.is_empty() => {
                r.clip = clip;
                r.range = idx..idx + 1;
            }
            _ => runs.push(matrixkit_gpu::QuadRun { clip, range: idx..idx + 1 }),
        }
    }
    // PICTURES (stage 4): first fetch (fetch, mutable),
    // then lay them immutably into the layers — and rasterise
    // the CURVES via tiny-skia as on the CPU, only
    // that the finished pixels travel on as a texture.
    for b in &v.picts {
        if b.fit == matrixkit_image::Fit::Stretch {
            // R210: the GPU stretches it — no CPU scaling here.
            atlas.touch_stretched(&b.path);
        } else {
            let _ = atlas.fetch(&b.path, b.box_w * k, b.box_h * k, b.fit);
        }
    }
    for c in &v.curves {
        if c.samples.len() < 2 {
            continue;
        }
        let (cw, chh) = (c.w * kf, c.h * kf);
        let Some(mut cv) = matrixkit_canvas::Canvas::new(cw as u32, chh as u32)
        else {
            continue;
        };
        let kn = c.samples.len();
        let step = cw / (kn as f32 - 1.0);
        let ymap = |vv: f32| {
            chh - 1.0 - (vv / c.max.max(0.001)).clamp(0.0, 1.0) * (chh - 2.0)
        };
        let mut ground = matrixkit_canvas::Trace::new();
        ground.move_to(0.0, chh);
        for (j, vv) in c.samples.iter().enumerate() {
            ground.line_to(j as f32 * step, ymap(*vv));
        }
        ground.line_to(cw, chh);
        ground.close();
        cv.fill(ground, matrixkit_theme::Rgba { a: 0.20, ..c.tint });
        let mut line = matrixkit_canvas::Trace::new();
        line.move_to(0.0, ymap(c.samples[0]));
        for (j, vv) in c.samples.iter().enumerate().skip(1) {
            line.line_to(j as f32 * step, ymap(*vv));
        }
        cv.stroke(line, c.tint, 2.0 * kf);
        use std::hash::{Hash, Hasher};
        let mut hh = std::collections::hash_map::DefaultHasher::new();
        for vv in &c.samples {
            vv.to_bits().hash(&mut hh);
        }
        c.max.to_bits().hash(&mut hh);
        c.tint.r.to_bits().hash(&mut hh);
        c.tint.g.to_bits().hash(&mut hh);
        c.tint.b.to_bits().hash(&mut hh);
        let key = format!("curve:{:x}", hh.finish());
        let pixels = cv.pixmap_mut().data_mut().to_vec();
        curve_holder.push((key, pixels, cw as u32, chh as u32));
    }
    let atlas: &'a ImageAtlas = atlas;
    let curve_holder: &'a Vec<(String, Vec<u8>, u32, u32)> = curve_holder;
    // Assign text and pictures to the layers
    // (anchor = Placed index).
    let n_layers = layer_bounds.len() + 1;
    let mut layer_texts: Vec<Vec<matrixkit_gpu::GpuText>> =
        (0..n_layers).map(|_| Vec::new()).collect();
    for t in &v.texts {
        let li = layer_bounds.partition_point(|b| *b <= t.anchor);
        layer_texts[li].push(matrixkit_gpu::GpuText {
            s: &t.s,
            px: t.px,
            line: t.px * matrixkit_text::LINE_HEIGHT,
            weight: match t.weight {
                matrixkit_theme::typo::Weight::Normal => 400,
                matrixkit_theme::typo::Weight::Medium => 500,
                matrixkit_theme::typo::Weight::Semibold => 600,
            },
            family: t.family,
            color: [t.color.r, t.color.g, t.color.b, t.color.a],
            x: t.x * kf,
            y: t.y * kf,
            bounds: t.clip.map(|c| {
                (
                    (c.x.max(0.0) * kf) as i32,
                    (c.y.max(0.0) * kf) as i32,
                    ((c.x + c.w) * kf).ceil() as i32,
                    ((c.y + c.h) * kf).ceil() as i32,
                )
            }),
        });
    }
    let px_clip = |c: &Rect| {
        let x = (c.x.max(0.0) * kf) as u32;
        let y = (c.y.max(0.0) * kf) as u32;
        (
            x,
            y,
            (((c.x + c.w).max(0.0) * kf).ceil() as u32).saturating_sub(x),
            (((c.y + c.h).max(0.0) * kf).ceil() as u32).saturating_sub(y),
        )
    };
    let mut layer_imgs: Vec<Vec<matrixkit_gpu::GpuImage>> =
        (0..n_layers).map(|_| Vec::new()).collect();
    // Round 126: a stream (one texture per name stem before ':') is ONE
    // picture per frame — video. Several stretched pictures of one stem in
    // the same frame are separate pictures: each keeps its own texture
    // (the lock's eleven glass keys showed only the last, 05.11.2026).
    let stem_of = |p: &std::path::Path| {
        let p = p.display().to_string();
        p.rsplit_once(':').map(|(s, _)| s.to_string()).unwrap_or(p)
    };
    let mut stems: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut seen: std::collections::HashSet<&std::path::Path> = std::collections::HashSet::new();
    for b in v.picts.iter().filter(|b| b.fit == matrixkit_image::Fit::Stretch) {
        // The same picture drawn twice (eleven keys, one map) is one.
        if seen.insert(b.path.as_path()) {
            *stems.entry(stem_of(&b.path)).or_insert(0) += 1;
        }
    }
    for b in &v.picts {
        let stretch = b.fit == matrixkit_image::Fit::Stretch;
        let found = if stretch {
            atlas.stretched_ready(&b.path)
        } else {
            atlas.peek_ready(&b.path, b.box_w * k, b.box_h * k, b.fit).map(|(w, h, d)| (w, h, d, false, false))
        };
        let Some((iw, ih, rgba, yuv, nv12)) = found else {
            continue;
        };
        // Stretched pictures fill their box (the texture keeps its
        // own size, the quad takes the box's).
        let (draw_w, draw_h) = if stretch {
            (b.box_w as f32 * kf, b.box_h as f32 * kf)
        } else {
            (iw as f32, ih as f32)
        };
        let li = layer_bounds.partition_point(|x| *x <= b.anchor);
        // Round 125: live glass — the backdrop is a stretched picture of
        // this scene; its texture key is made the same way as below.
        let glass = lens::get(&b.path).and_then(|l| {
            let bp = v.picts.iter().find(|p| p.path == l.backdrop && p.fit == matrixkit_image::Fit::Stretch)?;
            let (biw, bih, ..) = atlas.stretched_ready(&bp.path)?;
            Some(matrixkit_gpu::GpuLens {
                backdrop: (format!("{}|{}x{}|{}", bp.path.display(), bp.box_w * k, bp.box_h * k, false), biw, bih),
                back: [bp.x * kf, bp.y * kf, bp.box_w as f32 * kf, bp.box_h as f32 * kf],
                params: [l.bend * kf, l.scrim, l.rim, l.veil],
                more: [2.0, l.tint, l.opacity, 0.0],
            })
        });
        layer_imgs[li].push(matrixkit_gpu::GpuImage {
            key: (
                format!(
                    "{}|{}x{}|{}",
                    b.path.display(),
                    b.box_w * k,
                    b.box_h * k,
                    b.fit == matrixkit_image::Fit::Cover
                ),
                iw,
                ih,
            ),
            rgba,
            // Stretched pictures are computed frames (video: opaque,
            // so already premultiplied) — no per-frame CPU pass. A lens
            // map is data, never premultiplied.
            premultiplied: stretch || glass.is_some(),
            lens: glass,
            // ...and they share one texture per stream: the path up
            // to its running number (`\0tube:123` -> `\0tube`).
            yuv,
            nv12,
            stream: stretch.then(|| stem_of(&b.path)).filter(|stem| stems.get(stem).copied().unwrap_or(0) <= 1),
            x: b.x * kf,
            y: b.y * kf,
            w: draw_w,
            h: draw_h,
            radius: b.radius * kf,
            clip: Some(px_clip(
                &b.clip.unwrap_or(b.home),
            )),
        });
    }
    for (c, (key, pixels, cw, chh)) in v.curves.iter().zip(curve_holder.iter()) {
        let li = layer_bounds.partition_point(|x| *x <= c.anchor);
        layer_imgs[li].push(matrixkit_gpu::GpuImage {
            key: (key.clone(), *cw, *chh),
            rgba: pixels,
            premultiplied: true,
            x: c.x * kf,
            y: c.y * kf,
            w: *cw as f32,
            h: *chh as f32,
            radius: 0.0,
            clip: Some(px_clip(&c.home)),
            stream: None,
            lens: None,
            yuv: false,
            nv12: false,
        });
    }
    let mut texts: Vec<matrixkit_gpu::GpuText> = Vec::with_capacity(v.texts.len());
    let mut images: Vec<matrixkit_gpu::GpuImage> = Vec::new();
    let mut layers: Vec<matrixkit_gpu::GpuLayer> = Vec::with_capacity(n_layers);
    for (li, (bunch, ibunch)) in
        layer_texts.into_iter().zip(layer_imgs).enumerate()
    {
        let run_lo = layer_runs[li];
        let run_hi = layer_runs
            .get(li + 1)
            .copied()
            .unwrap_or(runs.len() as u32);
        let t_lo = texts.len() as u32;
        texts.extend(bunch);
        let i_lo = images.len() as u32;
        images.extend(ibunch);
        layers.push(matrixkit_gpu::GpuLayer {
            runs: run_lo..run_hi,
            images: i_lo..images.len() as u32,
            texts: t_lo..texts.len() as u32,
        });
    }
    // Keep the conversion for repaint rounds (only without
    // pictures — their pixels belong to the atlas).
    (quads, runs, texts, images, layers)
}

/// The drawing path: full (ground anew, complete scene) or diff (only
/// the difference) — the doctrine of all camps, now in the house.
fn draw<C: Client + 'static>(state: &mut House<C>, qh: &QueueHandle<House<C>>) -> Option<()> {
    let (w, h) = state.size;
    if w == 0 || h == 0 {
        return None;
    }
    // STAGE 1 of GPU PAINTING (MATRIX_GPU=1, GRABUNG-GPU-MALEREI.md):
    // prove the SEAM — wgpu device on the LIVING wl_surface,
    // clear colour instead of the CPU scene. From stage 2 on the
    // quad pipeline paints here; if the seam fails, the CPU path stays.
    if std::env::var_os("MATRIX_GPU").is_some() {
        // R209: the same graphics path as the iced apps (duo: Vulkan, Turnip).
        matrixkit_theme::gpu_path::prepare();
        let k = state.scale.max(1) as u32;
        if !state.gpu_tried {
            state.gpu_tried = true;
            let sp = state
                .surface
                .as_ref()
                .map(|s| s.id().as_ptr() as *mut std::ffi::c_void);
            if let Some(sp) = sp {
                match unsafe { matrixkit_gpu::Gpu::new(state.display_ptr, sp, w * k, h * k) }
                {
                    Ok(g) => {
                        eprintln!("[{}] GPU-Naht steht: {}", state.tag, g.adapter_info);
                        state.gpu = Some(g);
                    }
                    Err(e) => eprintln!(
                        "[{}] GPU-Naht fehlgeschlagen ({e}) — CPU-Pfad bleibt",
                        state.tag
                    ),
                }
            }
        }
        if state.gpu.is_some() {
            // REPAINT SHORTCUT (camp 24): only the shader time is
            // new (rain beat) — the cached conversion paints,
            // view() and shaping stand still. 17 % idle CPU
            // became ~2 % again this way.
            let repaint = state.repaint_only && state.gpu_cache.is_some();
            state.repaint_only = false;
            if repaint {
                let k = state.scale.max(1) as u32;
                let (pw2, ph2) = (w * k, h * k);
                let rain = state
                    .client
                    .rain()
                    .map(|r| [state.born.elapsed().as_secs_f32(), r[3], r[0], r[1], r[2]]);
                let mut engine = state.engine();
                let mut hands = Some(engine.gpu_handles());
                let ground_img = matrixkit_gpu::GpuImage {
                    key: (
                        format!("haus:grund:{}", state.gpu_ground_gen),
                        state.gpu_ground_size.0,
                        state.gpu_ground_size.1,
                    ),
                    rgba: &state.gpu_ground,
                    premultiplied: true,
                    x: 0.0,
                    y: 0.0,
                    w: pw2 as f32,
                    h: ph2 as f32,
                    radius: 0.0,
                    clip: None,
                    stream: None,
                    yuv: false,
                    nv12: false,
                    lens: None,
                };
                let cache = state.gpu_cache.as_ref().expect("cache liegt");
                let texts: Vec<matrixkit_gpu::GpuText> = cache
                    .texts
                    .iter()
                    .map(|t| matrixkit_gpu::GpuText {
                        s: &t.s,
                        px: t.px,
                        line: t.line,
                        weight: t.weight,
                        family: &t.family,
                        color: t.color,
                        x: t.x,
                        y: t.y,
                        bounds: t.bounds,
                    })
                    .collect();
                let corners_px = state.corners * k as f32;
                let gpu = state.gpu.as_mut().expect("gpu ist gesetzt");
                gpu.resize(pw2, ph2);
                gpu.corner_radius = corners_px;
                if let Some(sf) = &state.surface {
                    sf.frame(qh, ());
                }
                state.frame_pending = true;
                state.frame_at = Some(std::time::Instant::now());
                let _ = gpu.render_frame(
                    &mut hands,
                    // Round 90: a transparent surface clears to nothing.
                    if state.argb { [0.0, 0.0, 0.0, 0.0] } else { [0.0, 0.0, 0.0, 1.0] },
                    &cache.quads,
                    &cache.runs,
                    &[],
                    Some(&ground_img),
                    rain,
                    &texts,
                    &cache.layers,
                    k as f32,
                );
                drop(hands);
                state.engine = Some(engine);
                state.full = false;
                return Some(());
            }
            // STAGE 2: build the scene (hit()/scrolling need it too)
            // and paint its Placed citizens as instance runs.
            // Texts, pictures and curves follow in stages 3–4.
            let mut engine = state.engine();
            let mut keep = std::mem::take(&mut state.keep);
            let mut atlas = std::mem::take(&mut state.atlas);
            let view_clock = std::time::Instant::now();
            let v = state.client.view(&mut engine, &mut keep, &mut atlas, w, h);
            let v = match state.overlay.as_mut() {
                Some(o) => o.lay(&mut engine, v, w, h),
                None => v,
            };
            let v = jiggle(state, v);
            let view_us = view_clock.elapsed().as_micros();
            state.engine = Some(engine);
            keep.sweep();
            state.keep = keep;
            let kf = k as f32;
            // A scroll hint is meaningless on the GPU —
            // every frame is a full frame; just don't leave it lying.
            state.roll_hint = None;
            state.roll_spoiled = false;
            // The GROUND (stage 5): on full/resize fresh from
            // client.base(), BGRX → RGBA with full alpha.
            let (gpw, gph) = (w * k, h * k);
            if state.gpu_ground_size != (gpw, gph) || state.full {
                // During a LIVE resize do not paint every intermediate size:
                // fresh at most every 120 ms, in between the GPU stretches
                // the old ground (camp 12) — the last
                // state is caught up via dirty until it fits.
                let now = std::time::Instant::now();
                let due = state.gpu_ground.is_empty()
                    || state
                        .gpu_ground_at
                        .is_none_or(|t| now.duration_since(t).as_millis() >= 120);
                if due {
                    let mut ground = vec![0u8; (gpw * gph * 4) as usize];
                    state.client.base(&mut atlas, &mut ground, gpw, gph);
                    let argb = state.argb;
                    for px in ground.chunks_exact_mut(4) {
                        px.swap(0, 2);
                        // Round 90: a transparent surface keeps the ground's
                        // (premultiplied) alpha — the app menu's veil is 45 %
                        // black and stayed opaque on the GPU.
                        if !argb {
                            px[3] = 255;
                        }
                    }
                    state.gpu_ground = ground;
                    state.gpu_ground_gen += 1;
                    state.gpu_ground_size = (gpw, gph);
                    state.gpu_ground_at = Some(now);
                } else {
                    state.dirty = true;
                }
            }
            let (pw, ph) = (w * k, h * k);
            let mut curve_holder: Vec<(String, Vec<u8>, u32, u32)> = Vec::new();
            let (quads, runs, texts, images, layers) = gpu_lists(&v, &mut atlas, &mut curve_holder, w, h, k);
            state.gpu_cache = if images.is_empty() {
                Some(GpuSceneCache {
                    quads: quads.clone(),
                    runs: runs.clone(),
                    layers: layers.clone(),
                    texts: texts
                        .iter()
                        .map(|t| OwnedGpuText {
                            s: t.s.to_string(),
                            px: t.px,
                            line: t.line,
                            weight: t.weight,
                            family: t.family.to_string(),
                            color: t.color,
                            x: t.x,
                            y: t.y,
                            bounds: t.bounds,
                        })
                        .collect(),
                })
            } else {
                None
            };
            let n = (quads.len(), runs.len(), texts.len(), images.len());
            let mut engine = state.engine();
            let mut hands = Some(engine.gpu_handles());
            let gpu = state.gpu.as_mut().expect("gpu ist gesetzt");
            gpu.resize(pw, ph);
            // R207: the corner mask on the main path too (not only
            // on repaint) — otherwise the corners stay square on the
            // GPU (finger test 21:22).
            gpu.corner_radius = state.corners * kf;
            if let Some(s) = &state.surface {
                s.set_buffer_scale(state.scale.max(1));
            }
            if let Some(vp) = &state.viewport {
                vp.set_destination(state.size.0 as i32, state.size.1 as i32);
            }
            if let Some(xdg) = &state.xdg {
                xdg.set_window_geometry(0, 0, state.size.0 as i32, state.size.1 as i32);
            }
            if let Some(sf) = state.surface.clone() {
                opaque_region(state.compositor.as_ref(), state.toplevel.is_some(), state.argb, state.corners, qh, &sf, state.size.0, state.size.1);
            }
            let ground_img = matrixkit_gpu::GpuImage {
                key: (
                    format!("haus:grund:{}", state.gpu_ground_gen),
                    state.gpu_ground_size.0,
                    state.gpu_ground_size.1,
                ),
                rgba: &state.gpu_ground,
                premultiplied: true,
                x: 0.0,
                y: 0.0,
                w: gpw as f32,
                h: gph as f32,
                radius: 0.0,
                clip: None,
                stream: None,
                yuv: false,
                nv12: false,
                lens: None,
            };
            if let Some(sf) = &state.surface {
                // The callback hangs on the pending state and travels with
                // wgpu's present commit — the compositor reports
                // when a NEW picture makes sense.
                sf.frame(qh, ());
            }
            state.frame_pending = true;
            state.frame_at = Some(std::time::Instant::now());
            let rain = state.client.rain().map(|r| {
                [state.born.elapsed().as_secs_f32(), r[3], r[0], r[1], r[2]]
            });
            let painted = gpu.render_frame(
                &mut hands,
                // Round 90: a transparent surface (menus, veils) clears to
                // nothing — the ground carries its own alpha.
                if state.argb { [0.0, 0.0, 0.0, 0.0] } else { [0.0, 0.0, 0.0, 1.0] },
                &quads,
                &runs,
                &images,
                Some(&ground_img),
                rain,
                &texts,
                &layers,
                kf,
            );
            drop(hands);
            state.tick.phases = gpu.phases_ms;
            state.tick.build = gpu.build_ms;
            state.engine = Some(engine);
            state.atlas = atlas;
            // Sweeping on the GPU too (14.10.2026): without it every picture
            // ever laid stayed in the atlas — an app that lays a new full
            // picture per move (Tafel Werkstatt) grew by ~13 MB per
            // picture until the Surface froze. The software path
            // always swept (see draw()).
            state.atlas.sweep();
            match painted {
                Ok(us) => {
                    if std::env::var_os("MATRIX_PROBE").is_some() {
                        eprintln!(
                            "[{}] GPU-Bild: {} Quads/{} Läufe, {} Schriftzüge, {} Bilder, view {view_us} µs + mal {us} µs",
                            state.tag, n.0, n.1, n.2, n.3
                        );
                    }
                }
                Err(e) => eprintln!("[{}] GPU-Bild fehlgeschlagen: {e}", state.tag),
            }
            state.scene = v;
            state.client.painted(100.0);
            state.full = false;
            return Some(());
        }
    }
    let clock = std::time::Instant::now();
    let mut engine = state.engine();

    // The blit area of this round — for the buffer bookkeeping
    // below (show damage), not for the paint damage.
    let mut roll_area: Option<Rect> = None;
    let damage: Vec<Rect> = if state.full || state.canvas.is_empty() {
        if state.buf_size != (w, h) {
            for slot in state.slots.drain(..) {
                slot.buffer.destroy();
            }
            state.gen += 1;
            state.buf_size = (w, h);
        }
        state.atlas.sweep();
        state.roll_hint = None;
        state.roll_spoiled = false;
        // Ground and canvas carry PIXELS (camp 87) — the
        // client computes logically, the house rasterises more densely.
        let (pw, ph) = (w * state.scale.max(1) as u32, h * state.scale.max(1) as u32);
        state.base = vec![0u8; (pw * ph * 4) as usize];
        {
            let mut atlas = std::mem::take(&mut state.atlas);
            state.client.base(&mut atlas, &mut state.base, pw, ph);
            state.atlas = atlas;
        }
        state.canvas = state.base.clone();
        let mut keep = std::mem::take(&mut state.keep);
        let mut atlas = std::mem::take(&mut state.atlas);
        let v = state.client.view(&mut engine, &mut keep, &mut atlas, w, h);
        let v = match state.overlay.as_mut() {
            Some(o) => o.lay(&mut engine, v, w, h),
            None => v,
        };
        let v = jiggle(state, v);
        state.atlas = atlas;
        keep.sweep();
        state.keep = keep;
        compose(&mut engine, &mut state.atlas, &v, None, &mut state.canvas, pw, ph, state.scale.max(1) as f32);
        state.scene = v;
        state.client.painted(100.0);
        state.full = false;
        eprintln!(
            "[{}] voll gemalt: {:?} in {} ms ({} Atlas-Dekodierungen gesamt)",
            state.tag,
            state.size,
            clock.elapsed().as_millis(),
            state.atlas.decodes()
        );
        vec![Rect { x: 0.0, y: 0.0, w: w as f32, h: h as f32 }]
    } else {
        let mut keep = std::mem::take(&mut state.keep);
        let mut atlas = std::mem::take(&mut state.atlas);
        let v = state.client.view(&mut engine, &mut keep, &mut atlas, w, h);
        let v = match state.overlay.as_mut() {
            Some(o) => o.lay(&mut engine, v, w, h),
            None => v,
        };
        let v = jiggle(state, v);
        state.atlas = atlas;
        keep.sweep();
        state.keep = keep;
        // SCROLL BLIT (camp 104): if the app reported a scroll
        // and the area in the OLD as in the NEW scene consists only of
        // citizens with exactly this view box, the already
        // painted pixels move via copy_within — the diff pairs the
        // old tree SHIFTED by dy against the new one (multiset,
        // since culling shifts the indices) and pays only for
        // the uncovered strip plus the real changes.
        // Self-correcting: what does not fit exactly does not pair
        // and is repainted anyway.
        let hint = state.roll_hint.take();
        let roll = hint.filter(|(area, dy)| {
            *dy != 0
                && (dy.unsigned_abs() as f32) < area.h
                && rollable(&state.scene, area)
                && rollable(&v, area)
        });
        state.roll_spoiled = false;
        roll_area = roll.as_ref().map(|(a, _)| *a);
        // The rejection probe: a rejected scroll hint is
        // otherwise INVISIBLE (the normal path paints correctly after all) — and
        // that is exactly how the ground-quad case stayed silent.
        if std::env::var_os("MATRIX_PROBE").is_some() {
            if let (Some((area, dy)), true) = (hint, roll.is_none()) {
                let alien = state
                    .scene
                    .placed
                    .iter()
                    .chain(v.placed.iter())
                    .find(|q| {
                        q.clip != Some(area)
                            && !is_flat_ground(q, &area)
                            && visible(Rect { x: q.x, y: q.y, w: q.w, h: q.h }, q.clip)
                                .intersects(&area)
                    });
                eprintln!(
                    "[probe] roll-verworfen: dy={dy} fremder={:?}",
                    alien.map(|q| (q.x, q.y, q.w, q.h, q.radius, q.clip.is_some()))
                );
            }
        }
        let mut damage = if let Some((area, dy)) = roll {
            let dyf = dy as f32;
            let placed_rolled: Vec<Placed> = state
                .scene
                .placed
                .iter()
                .map(|q| {
                    let mut q = q.clone();
                    if q.clip == Some(area) {
                        q.y += dyf;
                    }
                    q
                })
                .collect();
            let texts_rolled: Vec<Text> = state
                .scene
                .texts
                .iter()
                .map(|t| {
                    let mut t = t.clone();
                    if t.clip == Some(area) {
                        t.y += dyf;
                        t.home.y += dyf;
                    }
                    t
                })
                .collect();
            let picts_rolled: Vec<Pict> = state
                .scene
                .picts
                .iter()
                .map(|c| {
                    let mut c = c.clone();
                    if c.clip == Some(area) {
                        c.y += dyf;
                        c.home.y += dyf;
                    }
                    c
                })
                .collect();
            // The canvas holds the last painted state — there
            // shifting IS painting.
            blit_rows(&mut state.canvas, &area, dy, state.scale.max(1) as u32, w, h);
            let strip = dy.unsigned_abs() as f32;
            let mut damage = vec![if dy > 0 {
                Rect { x: area.x, y: area.y, w: area.w, h: strip }
            } else {
                Rect { x: area.x, y: area.y + area.h - strip, w: area.w, h: strip }
            }];
            // The PAIR PROBE: at the first text with a box it names the
            // old and the new instance field by field — guessing
            // about the inequality cost two attempts.
            if std::env::var_os("MATRIX_PROBE").is_some() {
                if let Some(t) = texts_rolled.iter().find(|t| t.clip == Some(area)) {
                    match v.texts.iter().find(|n| n.s == t.s) {
                        Some(n) => eprintln!(
                            "[probe] paar alt(x{:.4} y{:.4} px{:.2} home {:.4},{:.4},{:.4}x{:.4}) neu(x{:.4} y{:.4} px{:.2} home {:.4},{:.4},{:.4}x{:.4}) clip_gleich={} s='{}'",
                            t.x, t.y, t.px, t.home.x, t.home.y, t.home.w, t.home.h,
                            n.x, n.y, n.px, n.home.x, n.home.y, n.home.w, n.home.h,
                            t.clip == n.clip,
                            t.s.chars().take(10).collect::<String>()
                        ),
                        None => eprintln!("[probe] paar: alter Text ohne neues Gegenstück"),
                    }
                }
                if let Some(q) = placed_rolled.iter().find(|q| q.clip == Some(area)) {
                    let n = v.placed.iter().find(|n| {
                        n.clip == Some(area) && near(n.w, q.w) && near(n.h, q.h)
                    });
                    if let Some(n) = n {
                        eprintln!(
                            "[probe] paar-quad alt({:.4},{:.4} {:.4}x{:.4} r{:.2} a{:.3}) neu({:.4},{:.4} {:.4}x{:.4} r{:.2} a{:.3}) farbe_gleich={} schatten_gleich={}",
                            q.x, q.y, q.w, q.h, q.radius, q.color.a,
                            n.x, n.y, n.w, n.h, n.radius, n.color.a,
                            q.color == n.color, q.shadow == n.shadow
                        );
                    }
                }
            }
            let d0 = damage.len();
            burden_pool(&placed_rolled, &v.placed, placed_near, |q| {
                visible(Rect { x: q.x, y: q.y, w: q.w, h: q.h }, q.clip)
            }, |q| q.color.a < 0.999 || q.radius > 0.0, &mut damage);
            let d1 = damage.len();
            burden_pool(&texts_rolled, &v.texts, text_near, |t| t.home, |_| true, &mut damage);
            let d2 = damage.len();
            burden_pool(&picts_rolled, &v.picts, pict_near, |c| c.home, |_| true, &mut damage);
            let d3 = damage.len();
            burden(&state.scene.curves, &v.curves, |c| c.home, &mut damage);
            let d4 = damage.len();
            if std::env::var_os("MATRIX_PROBE").is_some() {
                eprintln!(
                    "[probe] roll-stufen: quads+{} texte+{} bilder+{} kurven+{}",
                    d1 - d0, d2 - d1, d3 - d2, d4 - d3
                );
            }
            // Translucent quads over the strip need
            // the ground beneath fresh: full home into the damage.
            for q in &v.placed {
                if q.color.a >= 0.999 && q.radius <= 0.0 {
                    continue;
                }
                let hu = visible(Rect { x: q.x, y: q.y, w: q.w, h: q.h }, q.clip);
                if hu.w > 0.0
                    && hu.h > 0.0
                    && damage.iter().any(|r| r.intersects(&hu))
                {
                    damage.push(hu);
                }
            }
            if std::env::var_os("MATRIX_PROBE").is_some() {
                eprintln!("[probe] roll-blit: dy={dy} Flecken={}", damage.len());
                for (i, r) in damage.iter().take(8).enumerate() {
                    eprintln!("[probe]   fleck {i}: {:.1},{:.1} {:.1}x{:.1}", r.x, r.y, r.w, r.h);
                }
            }
            damage
        } else {
            diff(&state.scene.placed, &v.placed)
        };
        let rolled = roll.is_some();
        // Permanent probe (MATRIX_PROBE): names the first diverging
        // places in the diff — needed twice, stays in.
        if std::env::var_os("MATRIX_PROBE").is_some() && !damage.is_empty() {
            let mut shown = 0;
            for (i, (a, b)) in state.scene.placed.iter().zip(v.placed.iter()).enumerate() {
                if a != b && shown < 4 {
                    eprintln!(
                        "[probe] placed {i}: ({:.1},{:.1} {:.1}x{:.1}) -> ({:.1},{:.1} {:.1}x{:.1})",
                        a.x, a.y, a.w, a.h, b.x, b.y, b.w, b.h
                    );
                    shown += 1;
                }
            }
            if state.scene.placed.len() != v.placed.len() {
                eprintln!("[probe] placed-Zahl: {} -> {}", state.scene.placed.len(), v.placed.len());
            }
        }
        if !rolled {
            burden(&state.scene.picts, &v.picts, |b| b.home, &mut damage);
            burden(&state.scene.curves, &v.curves, |c| c.home, &mut damage);
            burden(&state.scene.texts, &v.texts, |t| t.home, &mut damage);
        }
        // Expand the patches to WHOLE logical pixels (camp 89).
        //
        // Otherwise the two halves of the diff compute differently: the
        // reset to the ground rounds LOGICALLY (`edges`, then
        // ×scale), the repaint rounds in PIXELS
        // (`r.y * k`, then truncated). For a patch that starts at
        // y = 506.6, pixel row 1012 is set black and
        // painted again only from 1013 — a hair-thin black line
        // that nobody ever touches again. That is exactly what the owner's
        // „fine black lines between the rows" and the
        // leftovers when scrolling were (17.08.2026).
        //
        // Whole logical pixels make both roundings the identity.
        let damage = coalesce(coalesce(damage).iter().map(snap).collect());
        // The patches are LOGICAL; the ground lies in pixels.
        let k = state.scale.max(1) as u32;
        let (pw, ph) = (w * k, h * k);
        for r in &damage {
            let (x0, y0, x1, y1) = edges(r, w, h);
            for y in (y0 * k)..(y1 * k) {
                let a = ((y * pw + x0 * k) * 4) as usize;
                let b = ((y * pw + x1 * k) * 4) as usize;
                state.canvas[a..b].copy_from_slice(&state.base[a..b]);
            }
        }
        compose(&mut engine, &mut state.atlas, &v, Some(&damage), &mut state.canvas, pw, ph, k as f32);
        if !damage.is_empty() {
            let px: f32 = damage.iter().map(|r| r.area()).sum();
            state.client.painted(100.0 * px / (w * h) as f32);
            eprintln!(
                "[{}] Diff: {} Flecken, {} px statt {} px ({:.1} %), gemalt in {} µs",
                state.tag,
                damage.len(),
                px as u64,
                w * h,
                100.0 * px / (w * h) as f32,
                clock.elapsed().as_micros()
            );
        }
        state.scene = v;
        damage
    };

    state.engine = Some(engine);
    fleet_sync(state, qh);
    for slot in &mut state.slots {
        slot.stale.extend(damage.iter().copied());
        // The BLIT shifts pixels in the master canvas
        // without them being paint damage — for the BUFFERS
        // the WHOLE area is stale nonetheless. Without this line
        // the slot got only strips and edges freshly copied,
        // the shifted middle stayed at the state of its
        // previous round: the owner's „the middle looks totally glitched"
        // (27.08.). Paint damage and show damage are, since the
        // blit, TWO different bookkeepings.
        if let Some(a) = roll_area {
            slot.stale.push(a);
        }
    }
    if state.slots.is_empty() && damage.is_empty() {
        return Some(());
    }
    present(state, qh)
}

/// The fleet sync: the plan comes from the client, the contract
/// (prefix keeping, stacking) from fleet, the surfaces from here —
/// edge-anchored, never computed from screen_size.
fn fleet_sync<C: Client + 'static>(state: &mut House<C>, qh: &QueueHandle<House<C>>) {
    let plan = state.client.fleet();
    let sync = state.fleet.sync(plan);
    for wid in sync.close {
        if let Some(pos) = state.panes.iter().position(|pn| pn.id == wid) {
            let mut pane = state.panes.remove(pos);
            if let Some(child) = pane.zoom.take() {
                child.retire();
            }
            drop(pane.target.take());
            for b in &pane.bufs {
                b.buffer.destroy();
            }
            if let Some(l) = pane.layer.take() {
                l.destroy();
            }
            if let Some(l) = pane.lock.take() {
                l.destroy();
            }
            pane.surface.destroy();
            eprintln!("[{}] Flotte: Fläche {} geschlossen", state.tag, wid);
        }
    }
    // R216: kept surfaces that place themselves follow their place —
    // margins, anchors and size change in place (one commit, no reopen).
    for idx in 0..state.panes.len() {
        let Some(old) = state.panes[idx].place.clone() else { continue };
        let card = state.panes[idx].card.clone();
        let Some(new) = state.client.pane_place(&card) else { continue };
        if new == old {
            continue;
        }
        let pane = &mut state.panes[idx];
        let Some(layer) = pane.layer.clone() else { continue };
        layer.set_anchor(layer_anchor(new.anchor));
        layer.set_margin(new.margin.0, new.margin.1, new.margin.2, new.margin.3);
        layer.set_exclusive_zone(new.exclusive);
        layer.set_keyboard_interactivity(layer_keyboard(new.keyboard));
        if new.size != old.size {
            layer.set_size(new.size.0, new.size.1);
            // The next picture waits for the configure of the new size.
            pane.ready = false;
        }
        pane.surface.commit();
        pane.place = Some(new);
    }
    for (shell, card) in sync.open {
        let (Some(compositor), Some(layer_shell)) =
            (state.compositor.clone(), state.layer_shell.clone())
        else {
            return;
        };
        let place = state.client.pane_place(&card);
        let surface = compositor.create_surface(qh, ());
        let output = place
            .as_ref()
            .and_then(|pl| pl.output.as_ref())
            .and_then(|want| state.outputs.iter().rev().find(|(_, n)| n.as_deref() == Some(want.as_str())).map(|(o, _)| o.clone()));
        // Round 124: under the session lock a pane is a lock surface on its
        // output (the compositor shows nothing else) — sized by it.
        if let Some(lock) = state.session_lock.clone() {
            let Some(output) = output else {
                eprintln!("[{}] Sperre: Flotten-Fläche ohne Ausgang — übersprungen", state.tag);
                surface.destroy();
                state.fleet.skipped();
                continue;
            };
            let ls = lock.get_lock_surface(&surface, &output, qh, ());
            state.pane_seq += 1;
            let id = state.pane_seq;
            eprintln!("[{}] Sperre: Flotten-Fläche {} als Sperrfläche", state.tag, id);
            state.fleet.adopt(id, shell, card.clone());
            let scale = state.scale.max(1);
            state.panes.push(Pane { id, surface, layer: None, lock: Some(ls), shell, card, ready: false, place, size: (0, 0), scale, shown: 0, drawn_at: None, zoom: None, last: None, canvas: Vec::new(), roll_err: 0.0, bufs: Vec::new(), buf_dims: (0, 0), target: None });
            continue;
        }
        let on_layer = place.as_ref().and_then(|pl| pl.layer).map(wl_layer).unwrap_or(state.pane_layer);
        let layer = layer_shell.get_layer_surface(
            &surface,
            output.as_ref(),
            on_layer,
            String::from(shell.namespace),
            qh,
            (),
        );
        match &place {
            // R216: the client's own place.
            Some(pl) => {
                layer.set_anchor(layer_anchor(pl.anchor));
                layer.set_margin(pl.margin.0, pl.margin.1, pl.margin.2, pl.margin.3);
                layer.set_size(pl.size.0, pl.size.1);
                layer.set_exclusive_zone(pl.exclusive);
                layer.set_keyboard_interactivity(layer_keyboard(pl.keyboard));
            }
            // The anchor follows the flow (camp 38): downwards the fleet
            // hangs top right, across bottom left (the dock shelf).
            None => {
                match state.fleet.flow() {
                    fleet::Flow::Down => {
                        layer.set_anchor(
                            zwlr_layer_surface_v1::Anchor::Top | zwlr_layer_surface_v1::Anchor::Right,
                        );
                        layer.set_margin(shell.main, shell.cross, 0, 0);
                    }
                    fleet::Flow::Across => {
                        layer.set_anchor(
                            zwlr_layer_surface_v1::Anchor::Bottom | zwlr_layer_surface_v1::Anchor::Left,
                        );
                        layer.set_margin(0, 0, shell.cross, shell.main);
                    }
                }
                layer.set_size(shell.width, shell.height);
                // The keyboard wish of the spec (addendum camp 36): a
                // clicked on-demand island TAKES the focus — the
                // keys keep arriving at the same client (ONE
                // wl_keyboard for the whole house), Escape stays audible.
                if shell.keyboard {
                    layer.set_keyboard_interactivity(
                        zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand,
                    );
                }
            }
        }
        surface.commit();
        state.pane_seq += 1;
        let id = state.pane_seq;
        eprintln!(
            "[{}] Flotte: Fläche {} öffnet bei main={} cross={}",
            state.tag, id, shell.main, shell.cross
        );
        state.fleet.adopt(id, shell, card.clone());
        let scale = state.scale.max(1);
        state.panes.push(Pane { id, surface, layer: Some(layer), lock: None, shell, card, ready: false, place, size: (0, 0), scale, shown: 0, drawn_at: None, zoom: None, last: None, canvas: Vec::new(), roll_err: 0.0, bufs: Vec::new(), buf_dims: (0, 0), target: None });
    }
}

/// The protocol's anchor bits for `Anchors`.
fn layer_anchor(a: Anchors) -> zwlr_layer_surface_v1::Anchor {
    let mut bits = zwlr_layer_surface_v1::Anchor::empty();
    if a.top {
        bits |= zwlr_layer_surface_v1::Anchor::Top;
    }
    if a.right {
        bits |= zwlr_layer_surface_v1::Anchor::Right;
    }
    if a.bottom {
        bits |= zwlr_layer_surface_v1::Anchor::Bottom;
    }
    if a.left {
        bits |= zwlr_layer_surface_v1::Anchor::Left;
    }
    bits
}

fn layer_keyboard(k: KeyboardWish) -> zwlr_layer_surface_v1::KeyboardInteractivity {
    match k {
        KeyboardWish::None => zwlr_layer_surface_v1::KeyboardInteractivity::None,
        KeyboardWish::OnDemand => zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand,
        KeyboardWish::Exclusive => zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
    }
}

fn wl_layer(l: Layer) -> zwlr_layer_shell_v1::Layer {
    match l {
        Layer::Background => zwlr_layer_shell_v1::Layer::Background,
        Layer::Bottom => zwlr_layer_shell_v1::Layer::Bottom,
        Layer::Top => zwlr_layer_shell_v1::Layer::Top,
        Layer::Overlay => zwlr_layer_shell_v1::Layer::Overlay,
    }
}

/// Paint a fleet surface: the client paints the card, the house
/// provides buffer and attaching.
/// Round 178 (memory audit): the atlas was swept only by full MAIN draws. Pane-only apps (the bar's chips, cards)
/// and CPU diff draws never swept, so every computed picture — e.g. one `shape://` ring per animation frame of a tap
/// glow — stayed until the output changed. Panes now sweep too, at most every 20 s: what any surface used within
/// that window survives, the rest goes (and is re-made on demand).
fn sweep_now_and_then(atlas: &mut matrixkit_image::ImageAtlas) {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    static LAST: Mutex<Option<Instant>> = Mutex::new(None);
    let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
    if last.is_none_or(|t| t.elapsed() >= Duration::from_secs(20)) {
        if last.is_some() {
            // Only what the atlas can make again (178b: sweep() here dropped the greeter's glass maps).
            atlas.sweep_remakeable();
        }
        *last = Some(Instant::now());
    }
}

fn draw_pane<C: Client + 'static>(state: &mut House<C>, idx: usize, qh: &QueueHandle<House<C>>) {
    sweep_now_and_then(&mut state.atlas);
    let shell = state.panes[idx].shell;
    let (w, h) = match state.panes[idx].size {
        (cw, ch) if cw > 0 && ch > 0 => (cw, ch),
        _ => (shell.width, shell.height),
    };
    if w == 0 || h == 0 {
        return;
    }
    let card = state.panes[idx].card.clone();
    // Round 61: a zooming pane only moves its picture.
    match state.client.pane_zoom(&card, w, h) {
        Some(z) => {
            if zoom_frame(state, idx, qh, z, w, h) {
                return;
            }
        }
        None => {
            // Round 117: a full-scale GPU picture of the same content stays
            // as the pane's picture — parked at full size, nothing repainted.
            let version = state.client.pane_version(&card);
            let keep = state.panes[idx].zoom.as_ref().is_some_and(|c| c.target.is_some() && c.version.is_some() && c.version == version);
            if keep {
                if let Some(child) = &mut state.panes[idx].zoom {
                    if !child.parked {
                        child.parked = true;
                        child.viewport.set_destination(w as i32, h as i32);
                        child.surface.commit();
                        child.sub.set_position(0, 0);
                        state.panes[idx].surface.commit();
                        matrixkit_theme::runtime::trail("matrix-takt-flaechen.log", &format!("[{}] Fläche {idx}: Zoom-Bild bleibt stehen (kein Neumalen)", state.tag), 400);
                    }
                }
                return;
            }
            if let Some(child) = state.panes[idx].zoom.take() {
                child.retire();
                state.panes[idx].shown = 0; // paint the pane itself again
            }
        }
    }
    let t_start = std::time::Instant::now();
    let mut engine = state.engine();
    // R216: a scene pane is composed at its own buffer scale.
    let scene = state.client.pane_scene(&card, &mut engine, &mut state.atlas, w, h);
    let t_scene = std::time::Instant::now();
    // Round 87: a FLAT pane (only square-cornered quads without shadow, no
    // text, pictures or curves — a veil, a backdrop) looks the same at
    // buffer scale 1: nine times fewer pixels on the Duo. The bar's card
    // veil (one quad over the panel) took 86–144 ms at scale 3.
    let flat = scene.as_ref().is_some_and(is_flat);
    // Round 104: a moving pane went to scale 1 to spare the CPU (blurred
    // while it slid). A card on the GPU keeps its full scale — the GPU
    // does not feel the 7× pixels.
    let wants_gpu = w * h >= 120_000 || state.client.pane_gpu(&card);
    let on_gpu = !flat && wants_gpu && !state.pane_gpu_failed && std::env::var_os("MATRIX_CPU").is_none();
    let k = if scene.is_some() && !flat && (on_gpu || !state.client.pane_moving(&card)) { state.panes[idx].scale.max(1) as u32 } else { 1 };
    let (bw, bh) = (w * k, h * k);
    let dims = (bw, bh, k);
    let len = (bw * bh * 4) as usize;
    // Round 94: a LARGE pane (a card) paints on the GPU — the CPU took
    // 20–250 ms a picture there (owner: «rather on the GPU»). Chips and
    // flat veils stay on the CPU, cheap as they are; any GPU failure falls
    // back to the CPU for good.
    if let Some(sc) = &scene {
        if !flat && wants_gpu && draw_pane_gpu(state, idx, qh, &mut engine, sc, w, h, k) {
            state.engine = Some(engine);
            let pane = &mut state.panes[idx];
            pane.last = None;
            pane.canvas = Vec::new();
            pane.shown = 0;
            for b in pane.bufs.drain(..) {
                b.buffer.destroy();
            }
            let t_end = std::time::Instant::now();
            let gap = pane.drawn_at.replace(t_end).map(|t| t_end.duration_since(t).as_secs_f32() * 1000.0);
            if state.client.animating() || state.panes[idx].zoom.is_none() {
                let ms = |a: std::time::Instant, b: std::time::Instant| b.duration_since(a).as_secs_f32() * 1000.0;
                let line = format!(
                    "[{}] Fläche {idx} {bw}×{bh} GPU: Abstand {} | Szene {:.1} + GPU {:.1} = {:.1} ms",
                    state.tag,
                    gap.map_or(String::from("–"), |g| format!("{g:.0} ms")),
                    ms(t_start, t_scene),
                    ms(t_scene, t_end),
                    ms(t_start, t_end)
                );
                matrixkit_theme::runtime::trail("matrix-takt-flaechen.log", &line, 400);
            }
            return;
        }
    }
    // Round 86: the DIFF path of the main surface, now for panes — only the
    // changed patches (whole logical pixels, camp 89) go back to clear and
    // are painted again; more than half the pane changed paints it whole.
    let mut partial: Option<Vec<Rect>> = None;
    // Round 91: a scroll moves the painted pixels instead of painting them.
    let mut blit: Option<(Rect, i32)> = None;
    if let (Some(sc), Some((old, odims))) = (&scene, &state.panes[idx].last) {
        if *odims == dims && state.panes[idx].canvas.len() == len {
            let damage_of = |old: &Scene| {
                let mut damage = diff(&old.placed, &sc.placed);
                burden(&old.picts, &sc.picts, |b| b.home, &mut damage);
                burden(&old.curves, &sc.curves, |c| c.home, &mut damage);
                burden(&old.texts, &sc.texts, |t| t.home, &mut damage);
                damage
            };
            let mut damage = damage_of(old);
            let mut area: f32 = damage.iter().map(|r| r.area()).sum();
            if area >= 0.5 * (w * h) as f32 {
                if let Some((roll, dy)) = pane_roll(old, sc) {
                    let want = dy * k as f32 + state.panes[idx].roll_err;
                    let px = want.round() as i32;
                    let moved_dy = px as f32 / k as f32;
                    // Compare at the EXACT scroll (a finger moves by fractions of a
                    // pixel); the blit's sub-pixel remainder is carried in roll_err.
                    let mut d = damage_of(&rolled_copy(old, roll, dy));
                    // the strip the scroll uncovers
                    let reach = dy.abs().max(moved_dy.abs()) + 1.0;
                    let strip = if dy < 0.0 {
                        Rect { x: roll.x, y: roll.y + roll.h - reach, w: roll.w, h: reach }
                    } else {
                        Rect { x: roll.x, y: roll.y, w: roll.w, h: reach }
                    };
                    d.push(strip);
                    // whatever lies over the roll area without being clipped to it
                    // (a header, a shadow) was moved by the blit too: paint it again
                    // (a header, the scroll bar): paint it again where it is now, where
                    // it was — and where the blit carried its old pixels (Round 92b:
                    // that copy was missed, the owner saw the scroll bar torn).
                    let grow = |r: Rect| Rect { x: r.x - 1.0, y: r.y - 1.0, w: r.w + 2.0, h: r.h + 2.0 };
                    for (q, was) in old.placed.iter().map(|q| (q, true)).chain(sc.placed.iter().map(|q| (q, false))) {
                        if q.clip != Some(roll) {
                            let v = visible(Rect { x: q.x, y: q.y, w: q.w, h: q.h }, q.clip);
                            if v.intersects(&roll) && v.area() < 0.25 * roll.area() {
                                d.push(grow(v));
                                if was {
                                    d.push(grow(Rect { y: v.y + moved_dy, ..v }));
                                }
                            }
                        }
                    }
                    let da: f32 = d.iter().map(|r| r.area()).sum();
                    if da < 0.5 * (w * h) as f32 {
                        state.panes[idx].roll_err = want - px as f32;
                        blit = Some((roll, px));
                        damage = d;
                        area = da;
                    }
                }
            }
            if area < 0.5 * (w * h) as f32 {
                partial = Some(coalesce(coalesce(damage).iter().map(snap).collect()));
            }
        }
    }
    let mut canvas = std::mem::take(&mut state.panes[idx].canvas);
    if let Some((roll, px)) = blit {
        // Move the painted rows of the roll area by `px` buffer rows.
        let (x0, y0, x1, y1) = edges(&roll, w, h);
        let (bx0, bx1) = ((x0 * k) as usize * 4, (x1 * k) as usize * 4);
        let (top, bottom) = ((y0 * k) as i32, (y1 * k) as i32);
        let row = |y: i32| y as usize * bw as usize * 4;
        if px < 0 {
            for y in top..(bottom + px) {
                let src = row(y - px);
                canvas.copy_within(src + bx0..src + bx1, row(y) + bx0);
            }
        } else if px > 0 {
            for y in ((top + px)..bottom).rev() {
                let src = row(y - px);
                canvas.copy_within(src + bx0..src + bx1, row(y) + bx0);
            }
        }
    }
    if partial.is_none() {
        state.panes[idx].roll_err = 0.0;
    }
    match (&scene, &partial) {
        (Some(_), Some(damage)) if damage.is_empty() => {
            // Nothing changed: no paint, no buffer.
            state.engine = Some(engine);
            state.panes[idx].canvas = canvas;
            state.panes[idx].last = scene.map(|sc| (sc, dims));
            return;
        }
        (Some(sc), Some(damage)) => {
            for r in damage {
                let (x0, y0, x1, y1) = edges(r, w, h);
                for y in (y0 * k)..(y1 * k) {
                    let a = ((y * bw + x0 * k) * 4) as usize;
                    let b = ((y * bw + x1 * k) * 4) as usize;
                    canvas[a..b].fill(0);
                }
            }
            compose(&mut engine, &mut state.atlas, sc, Some(damage), &mut canvas, bw, bh, k as f32);
        }
        (Some(sc), None) => {
            canvas = vec![0u8; len];
            compose(&mut engine, &mut state.atlas, sc, None, &mut canvas, bw, bh, k as f32);
        }
        (None, _) => {
            canvas = vec![0u8; len];
            state.client.pane(&card, &mut engine, &mut canvas, w, h);
        }
    }
    state.engine = Some(engine);
    let t_composed = std::time::Instant::now();

    // Round 51: the same picture again (same size, scale and pixels) is
    // not sent — no buffer, no compositor work. (The diff path knows it
    // changed.)
    if partial.is_none() {
        let print = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            (bw, bh, k).hash(&mut h);
            canvas.hash(&mut h);
            h.finish() | 1
        };
        if state.panes[idx].shown == print {
            state.panes[idx].canvas = canvas;
            state.panes[idx].last = scene.map(|sc| (sc, dims));
            return;
        }
        state.panes[idx].shown = print;
    } else {
        state.panes[idx].shown = 0;
    }
    let t_hashed = std::time::Instant::now();

    // Round 92: what changed this picture (logical), for the kept buffers.
    let changed: Vec<Rect> = match &partial {
        Some(damage) => {
            let mut v = damage.clone();
            if let Some((roll, _)) = blit {
                v.push(roll);
            }
            v
        }
        None => vec![Rect { x: 0.0, y: 0.0, w: w as f32, h: h as f32 }],
    };
    let Some(shm) = state.shm.clone() else { return };
    let pane_id = state.panes[idx].id;
    {
        let pane = &mut state.panes[idx];
        if pane.buf_dims != (bw, bh) {
            for b in pane.bufs.drain(..) {
                b.buffer.destroy();
            }
            pane.buf_dims = (bw, bh);
        }
        for b in pane.bufs.iter_mut() {
            b.stale.extend(changed.iter().copied());
        }
    }
    let free = state.panes[idx].bufs.iter().position(|b| !b.busy);
    let si = match free {
        Some(i) => Some(i),
        None if state.panes[idx].bufs.len() < 3 => make_memfd(len).map(|(owned, map)| {
            let si = state.panes[idx].bufs.len();
            let pool = shm.create_pool(owned.as_fd(), len as i32, qh, ());
            // the owner's find (7.8., „black corners"): fleet surfaces
            // are ARGB — zero is fully transparent; the painters have carried
            // alpha along premultiplied since camp 23.
            let buffer = pool.create_buffer(0, bw as i32, bh as i32, (bw * 4) as i32, wl_shm::Format::Argb8888, qh, (PANE_BUF, ((pane_id as usize) << 8) | si));
            pool.destroy();
            state.panes[idx].bufs.push(Slot { buffer, map, stale: vec![Rect { x: 0.0, y: 0.0, w: w as f32, h: h as f32 }], busy: false });
            si
        }),
        None => None,
    };
    let buffer = match si {
        Some(si) => {
            // Copy only what this buffer still lacks.
            let slot = &mut state.panes[idx].bufs[si];
            let stale = coalesce(std::mem::take(&mut slot.stale).iter().map(snap).collect());
            let row = (bw * 4) as usize;
            for r in &stale {
                let (x0, y0, x1, y1) = edges(r, w, h);
                let (a, b) = ((x0 * k) as usize * 4, (x1 * k) as usize * 4);
                for y in (y0 * k)..(y1 * k) {
                    let o = y as usize * row;
                    slot.map[o + a..o + b].copy_from_slice(&canvas[o + a..o + b]);
                }
            }
            slot.busy = true;
            slot.buffer.clone()
        }
        // All kept buffers are still with the compositor: one fresh buffer
        // as before (used once, destroyed on release) — no picture is lost.
        None => {
            let Some((owned, mut map)) = make_memfd(len) else { return };
            map[..len].copy_from_slice(&canvas[..len]);
            let pool = shm.create_pool(owned.as_fd(), len as i32, qh, ());
            let buffer = pool.create_buffer(0, bw as i32, bh as i32, (bw * 4) as i32, wl_shm::Format::Argb8888, qh, (u32::MAX, usize::MAX));
            pool.destroy();
            buffer
        }
    };
    // Keep the picture and its scene for the next diff.
    state.panes[idx].canvas = canvas;
    state.panes[idx].last = scene.map(|sc| (sc, dims));
    let pane = &state.panes[idx];
    pane.surface.set_buffer_scale(k as i32);
    pane.surface.attach(Some(&buffer), 0, 0);
    match &partial {
        Some(damage) => {
            if let Some((roll, _)) = blit {
                let (x0, y0, x1, y1) = edges(&roll, w, h);
                pane.surface.damage_buffer((x0 * k) as i32, (y0 * k) as i32, ((x1 - x0) * k) as i32, ((y1 - y0) * k) as i32);
            }
            for r in damage {
                let (x0, y0, x1, y1) = edges(r, w, h);
                pane.surface.damage_buffer((x0 * k) as i32, (y0 * k) as i32, ((x1 - x0) * k) as i32, ((y1 - y0) * k) as i32);
            }
        }
        None => pane.surface.damage_buffer(0, 0, bw as i32, bh as i32),
    }
    pane.surface.commit();
    // Round 60 — the pane beat probe: while the client moves, every
    // pane picture with its gap and where its time went (CPU path).
    let t_end = std::time::Instant::now();
    let gap = state.panes[idx].drawn_at.replace(t_end).map(|t| t_end.duration_since(t).as_secs_f32() * 1000.0);
    if state.client.animating() || blit.is_some() {
        let ms = |a: std::time::Instant, b: std::time::Instant| b.duration_since(a).as_secs_f32() * 1000.0;
        let line = format!(
            "[{}] Fläche {idx} {bw}×{bh}{}: Abstand {} | Szene {:.1} + Malen {:.1} + Prüfsumme {:.1} + Puffer/Übergabe {:.1} = {:.1} ms",
            state.tag,
            blit.map_or(String::new(), |(_, px)| format!(" Roll {px} px")),
            gap.map_or(String::from("–"), |g| format!("{g:.0} ms")),
            ms(t_start, t_scene),
            ms(t_scene, t_composed),
            ms(t_composed, t_hashed),
            ms(t_hashed, t_end),
            ms(t_start, t_end)
        );
        matrixkit_theme::runtime::trail("matrix-takt-flaechen.log", &line, 400);
    }
}

/// Round 61: one zoom frame of pane `idx`. The first frame paints the
/// finished picture (the client's `pane_scene`) into a fresh sub-surface
/// and gives the pane itself a transparent buffer; every frame then sets
/// only the sub-surface's position and its viewport's destination. False
/// when the compositor lacks sub-surfaces or viewports — the caller paints.
fn zoom_frame<C: Client + 'static>(state: &mut House<C>, idx: usize, qh: &QueueHandle<House<C>>, z: Zoom, w: u32, h: u32) -> bool {
    // Round 117: a kept picture serves only the content it shows — another
    // card on the same pane (switching cards) gets a picture of its own.
    let card_now = state.panes[idx].card.clone();
    let version = state.client.pane_version(&card_now);
    if version.is_some() && state.panes[idx].zoom.as_ref().is_some_and(|c| c.version != version) {
        if let Some(child) = state.panes[idx].zoom.take() {
            child.retire();
        }
    }
    if state.panes[idx].zoom.is_none() {
        let (Some(subc), Some(vpm), Some(comp), Some(shm)) =
            (state.subcompositor.clone(), state.viewporter.clone(), state.compositor.clone(), state.shm.clone())
        else {
            return false;
        };
        let card = state.panes[idx].card.clone();
        let t0 = std::time::Instant::now();
        let mut engine = state.engine();
        let scene = state.client.pane_scene(&card, &mut engine, &mut state.atlas, w, h);
        let t_scene = t0.elapsed().as_secs_f32() * 1000.0;
        let Some(sc) = scene else {
            state.engine = Some(engine);
            return false;
        };
        let parent = state.panes[idx].surface.clone();
        let surface = comp.create_surface(qh, ());
        let sub = subc.get_subsurface(&surface, &parent, qh, ());
        let viewport = vpm.get_viewport(&surface, qh, ());
        // Input falls through to the pane (its hit list is the picture's).
        let none = comp.create_region(qh, ());
        surface.set_input_region(Some(&none));
        none.destroy();
        // Round 104: a card's zoom picture on the fleet GPU at the pane's own
        // scale — sharp while it grows, the same picture the card ends on.
        let scale = state.panes[idx].scale.max(1) as u32;
        // Round 118: small panes the client wants on the GPU (menus,
        // dialogs) zoom crisp too — they were CPU pictures at scale 1.
        let target = if w * h >= 120_000 || state.client.pane_gpu(&card) { gpu_picture(state, qh, &mut engine, &surface, &sc, w, h, scale) } else { None };
        let k = if target.is_some() { scale } else { 1 };
        if target.is_none() {
            // The CPU picture at buffer scale 1: small and in motion — ~7×
            // fewer pixels on the Duo (2.75).
            let (bw, bh) = (w, h);
            let mut canvas = vec![0u8; (bw * bh * 4) as usize];
            compose(&mut engine, &mut state.atlas, &sc, None, &mut canvas, bw, bh, 1.0);
            let len = (bw * bh * 4) as usize;
            let Some((owned, mut map)) = make_memfd(len) else {
                state.engine = Some(engine);
                ZoomChild { surface, sub, viewport, target: None, version: None, parked: false }.retire();
                return false;
            };
            map[..len].copy_from_slice(&canvas[..len]);
            let pool = shm.create_pool(owned.as_fd(), len as i32, qh, ());
            let picture = pool.create_buffer(0, bw as i32, bh as i32, (bw * 4) as i32, wl_shm::Format::Argb8888, qh, (u32::MAX, usize::MAX));
            pool.destroy();
            surface.set_buffer_scale(1);
            surface.attach(Some(&picture), 0, 0);
            surface.damage_buffer(0, 0, bw as i32, bh as i32);
        }
        state.engine = Some(engine);
        // The pane itself goes fully transparent while its picture moves —
        // through its GPU target when it has one (a swapchain surface takes
        // no shm buffer), else a fresh (zeroed) memfd.
        let mut cleared = false;
        if let (Some(mut t), Some((gpu, _))) = (state.panes[idx].target.take(), state.pane_gpu.as_mut()) {
            let (pw, ph) = (w * scale, h * scale);
            cleared = gpu
                .with_target(&mut t, |g| {
                    g.resize(pw, ph);
                    g.render_frame(&mut None, [0.0, 0.0, 0.0, 0.0], &[], &[], &[], None, None, &[], &[], scale as f32)
                })
                .is_ok();
            state.panes[idx].target = Some(t);
        }
        if !cleared && state.panes[idx].target.is_none() {
            let len = (w * h * 4) as usize;
            let Some((clear_fd, _clear_map)) = make_memfd(len) else {
                ZoomChild { surface, sub, viewport, target, version: None, parked: false }.retire();
                return false;
            };
            let pool = shm.create_pool(clear_fd.as_fd(), len as i32, qh, ());
            let clear = pool.create_buffer(0, w as i32, h as i32, (w * 4) as i32, wl_shm::Format::Argb8888, qh, (u32::MAX, usize::MAX));
            pool.destroy();
            parent.set_buffer_scale(1);
            parent.attach(Some(&clear), 0, 0);
            parent.damage_buffer(0, 0, w as i32, h as i32);
        }
        let gpu = target.is_some();
        let version = state.client.pane_version(&card);
        state.panes[idx].zoom = Some(ZoomChild { surface, sub, viewport, target, version, parked: false });
        state.panes[idx].shown = 0;
        matrixkit_theme::runtime::trail(
            "matrix-takt-flaechen.log",
            &format!("[{}] Zoom-Bild {}×{} ({}): Szene {t_scene:.0} ms, gesamt {:.0} ms", state.tag, w * k, h * k, if gpu { "GPU" } else { "CPU" }, t0.elapsed().as_secs_f32() * 1000.0),
            400,
        );
    }
    let Some(child) = &mut state.panes[idx].zoom else { return false };
    // Round 117: a parked picture zooms again (the card closing) — no new picture.
    child.parked = false;
    child.viewport.set_destination((z.w.round() as i32).max(1), (z.h.round() as i32).max(1));
    child.surface.commit();
    child.sub.set_position(z.x.round() as i32, z.y.round() as i32);
    state.panes[idx].surface.commit();
    true
}

/// The window house runs: ONE poll(2), no executor, no
/// subscription tangle — the loop of First Light, now
/// for every client.
/// PAINT ON THE GPU (R211 family, 02.11.2026): call first thing in
/// `main`. Every house app that moves (video, animation) set MATRIX_GPU by
/// hand — Matrix Krabbler forgot it and drew a thousand dots a frame on
/// the CPU (93 %, stutter). `MATRIX_CPU=1` stays the emergency exit.
pub fn prefer_gpu() {
    if std::env::var_os("MATRIX_CPU").is_none() {
        std::env::set_var("MATRIX_GPU", "1");
    }
}

pub fn run<C: Client + 'static>(client: C, options: Options) -> Result<(), Box<dyn std::error::Error>> {
    run_with(client, options, None)
}

/// `run` with an overlay over the app (R211, `Overlay`).
pub fn run_with<C: Client + 'static>(client: C, options: Options, overlay: Option<Box<dyn Overlay>>) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[{}] Fensterhaus: wayland-client direkt, pid={}", options.tag, std::process::id());
    match matrixkit_theme::habitat::habitat() {
        matrixkit_theme::habitat::Habitat::Matrix => eprintln!("[{}] Lebensraum: Matrix (Leinwand)", options.tag),
        matrixkit_theme::habitat::Habitat::Foreign => {
            eprintln!(
                "[{}] Lebensraum: fremder Desktop ({}) — eigene Ecken, Kopf zieht, eigene Dekoration, Schriften aus dem Bündel",
                options.tag,
                matrixkit_theme::habitat::desktop_name()
            );
            // The SEED of the palette from the desktop's
            // wallpaper — computed once at start, written to the seed
            // file; Palette::load() reads it, the clients'
            // 2 s pulse pulls the colours along.
            reseed(options.tag, matrixkit_theme::seed::wallpaper_path());
        }
    }
    let conn = Connection::connect_to_env()?;
    let mut queue = conn.new_event_queue();
    let qh = queue.handle();
    let display = conn.display();
    display.get_registry(&qh, ());

    let mut state = House {
        client,
        overlay,
        tag: options.tag,
        argb: options.argb,
        deco_manager: None,
        viewporter: None,
        lock_manager: None,
        session_lock: None,
        lock_surface: None,
        lock_others: Vec::new(),
        lock_refused: false,
        lock_asked: None,
        lock_confirmed: false,
        outputs: Vec::new(),
        output_globals: Vec::new(),
        viewport: None,
        seed_watch: None,
        seed_ticks: 0,
        leak_probe: Default::default(),
        leak_ticks: 0,
        tick: TickProbe::default(),
        corners: 0.0,
        compositor: None,
        subcompositor: None,
        shm: None,
        layer_shell: None,
        surface: None,
        layer: None,
        wm_base: None,
        xdg: None,
        toplevel: None,
        want: (0, 0),
        maximized: false,
        zoom_wish: None,
        shelve_wish: false,
        size: (0, 0),
        dirty: false,
        full: true,
        closed: false,
        pointer_dev: None,
        touch_dev: None,
        keyboard_dev: None,
        xkb: None,
        seat: None,
        data_manager: None,
        data_device: None,
        drop_offer: None,
        drop_at: (0.0, 0.0),
        data_source: None,
        offer_building: None,
        offer_current: None,
        serial: 0,
        text_input: None,
        ime_manager: None,
        text_on: false,
        repeat_rate: 25,
        repeat_delay: 400,
        held: None,
        keyboard_focus: false,
        pointer: (0.0, 0.0),
        pointer_held: false,
        wheel: false,
        press_serial: 0,
        wiggle: None,
        wiggle_move: None,
        fling: None,
        touch_at: (0.0, 0.0),
        touches: Vec::new(),
        pinch_dist: None,
        touch_numb: false,
        flick: None,
        press: None,
        extent_applied: (0, 0),
        scale: 1,
        offset_applied: None,
        pointer_pane: None,
        touch_pane: None,
        // Round 167: the planned fleet opens at the start — it waited for a
        // Panes refresh by chance (the login keyboard never came, 05.11.2026).
        panes_dirty: true,
        roll_hint: None,
        roll_spoiled: false,
        gpu: None,
        pane_gpu: None,
        pane_gpu_failed: false,
        beats_seen: 0,
        gpu_tried: false,
        gpu_ground: Vec::new(),
        gpu_ground_gen: 0,
        gpu_ground_size: (0, 0),
        gpu_ground_at: None,
        born: std::time::Instant::now(),
        repaint_only: false,
        gpu_cache: None,
        frame_pending: false,
        frame_at: None,
        display_ptr: conn.backend().display_ptr() as *mut std::ffi::c_void,
        passthrough_applied: false,
        pane_layer: zwlr_layer_shell_v1::Layer::Top,
        scene: Scene::default(),
        keep: Keep::new(),
        base: Vec::new(),
        canvas: Vec::new(),
        engine: None,
        atlas: ImageAtlas::with_resolver(Box::new(shapes::resolve)),
        fleet: fleet::Fleet::flowing(options.fleet_start, options.fleet_gap, options.fleet_flow),
        panes: Vec::new(),
        pane_seq: 0,
        slots: Vec::new(),
        gen: 0,
        buf_size: (0, 0),
    };

    // The loom (camp 25): channel + wake pipe, then wind
    // the warp. The pipe is non-blocking — a lost
    // wake byte is harmless, the channel holds the material.
    let (work_tx, work_rx) = std::sync::mpsc::channel::<C::Work>();
    let mut pipe_fds = [0i32; 2];
    if unsafe { libc::pipe2(pipe_fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) } != 0 {
        return Err("Weck-Pfeife ließ sich nicht öffnen".into());
    }
    let wake_read = pipe_fds[0];
    let loom = Loom {
        tx: work_tx,
        wake: std::sync::Arc::new(unsafe {
            use std::os::fd::FromRawFd;
            std::os::fd::OwnedFd::from_raw_fd(pipe_fds[1])
        }),
    };
    state.client.warp(&loom);

    // The house's mailbox (R210, `mk::house`): words from the top bar for
    // a window — `root` (settings), `close`. Same wake pipe as the loom.
    let (house_tx, house_rx) = std::sync::mpsc::channel::<String>();
    let house_path = matches!(options.mount, Mount::Window { .. }).then(|| matrixkit_theme::house::socket(std::process::id()));
    if let Some(path) = &house_path {
        // What killed or crashed windows left behind goes first (their
        // mailboxes and Krabbler states never got a clean goodbye).
        let swept = matrixkit_theme::runtime::sweep_orphans("matrix-house-", ".sock") + matrixkit_theme::runtime::sweep_orphans("matrix-krabbler-", "");
        if swept > 0 {
            eprintln!("[{}] {swept} verwaiste Laufzeitdateien entfernt", options.tag);
        }
        let _ = std::fs::remove_file(path);
        if let Ok(sock) = std::os::unix::net::UnixDatagram::bind(path) {
            let wake = loom.wake.clone();
            std::thread::spawn(move || {
                let mut buf = [0u8; 256];
                while let Ok(n) = sock.recv(&mut buf) {
                    let _ = house_tx.send(String::from_utf8_lossy(&buf[..n]).trim().to_string());
                    use std::os::fd::AsRawFd;
                    let byte = [1u8];
                    unsafe { libc::write(wake.as_raw_fd(), byte.as_ptr() as *const libc::c_void, 1) };
                }
            });
        }
    }

    queue.roundtrip(&mut state)?;
    if let (Some(manager), Some(seat)) = (state.ime_manager.clone(), state.seat.clone()) {
        state.text_input = Some(manager.get_text_input(&seat, &qh, ()));
        eprintln!("[{}] IME-Leitung liegt (zwp_text_input_v3)", state.tag);
    }
    // The clipboard hangs on the SEAT, not on the surface (camp 88) —
    // hence here, right next to the IME line.
    if let (Some(manager), Some(seat)) = (state.data_manager.clone(), state.seat.clone()) {
        state.data_device = Some(manager.get_data_device(&seat, &qh, ()));
        eprintln!("[{}] Ablage liegt (wl_data_device)", state.tag);
    }
    let compositor = state.compositor.clone().ok_or("kein wl_compositor")?;
    let surface = compositor.create_surface(&qh, ());
    match options.mount {
        Mount::Layer { namespace, layer: on_layer, keyboard, anchor, size, margin, exclusive } => {
            let layer_shell = state
                .layer_shell
                .clone()
                .ok_or("kein zwlr_layer_shell_v1 — läuft hier ein Wayland-Compositor mit Layer-Shell?")?;
            let on_layer = match on_layer {
                Layer::Background => zwlr_layer_shell_v1::Layer::Background,
                Layer::Bottom => zwlr_layer_shell_v1::Layer::Bottom,
                Layer::Top => zwlr_layer_shell_v1::Layer::Top,
                Layer::Overlay => zwlr_layer_shell_v1::Layer::Overlay,
            };
            // R216: `--output NAME` puts the surface on that screen.
            let wanted = state.client.output();
            if wanted.is_some() && state.outputs.iter().any(|(_, n)| n.is_none()) {
                queue.roundtrip(&mut state)?;
            }
            let target = wanted.as_ref().and_then(|w| state.outputs.iter().rev().find(|(_, n)| n.as_deref() == Some(w.as_str())).map(|(o, _)| o.clone()));
            if let (Some(w), None) = (&wanted, &target) {
                eprintln!("[{}] Ausgang «{w}» nicht gefunden — der Compositor wählt", state.tag);
            }
            let layer = layer_shell.get_layer_surface(&surface, target.as_ref(), on_layer, namespace, &qh, ());
            let mut a = zwlr_layer_surface_v1::Anchor::empty();
            if anchor.top {
                a |= zwlr_layer_surface_v1::Anchor::Top;
            }
            if anchor.right {
                a |= zwlr_layer_surface_v1::Anchor::Right;
            }
            if anchor.bottom {
                a |= zwlr_layer_surface_v1::Anchor::Bottom;
            }
            if anchor.left {
                a |= zwlr_layer_surface_v1::Anchor::Left;
            }
            layer.set_anchor(a);
            layer.set_exclusive_zone(exclusive);
            state.pane_layer = on_layer;
            layer.set_size(size.0, size.1);
            layer.set_margin(margin.0, margin.1, margin.2, margin.3);
            state.extent_applied = size;
            match keyboard {
                KeyboardWish::None => {}
                KeyboardWish::OnDemand => layer.set_keyboard_interactivity(
                    zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand,
                ),
                KeyboardWish::Exclusive => layer.set_keyboard_interactivity(
                    zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
                ),
            }
            surface.commit();
            state.layer = Some(layer);
        }
        Mount::Window { app_id, title, width, height } => {
            // Camp 18: the window path — xdg-toplevel, the head
            // comes from the app's Frame family.
            let wm = state.wm_base.clone().ok_or("kein xdg_wm_base")?;
            state.want = (width, height);
            let xdg = wm.get_xdg_surface(&surface, &qh, ());
            let top = xdg.get_toplevel(&qh, ());
            top.set_app_id(app_id);
            top.set_title(title);
            // R207: decoration is the client's business — where the
            // compositor offers the frame (KDE), we decline.
            if let Some(m) = &state.deco_manager {
                let deco = m.get_toplevel_decoration(&top, &qh, ());
                deco.set_mode(zxdg_toplevel_decoration_v1::Mode::ClientSide);
            }
            // The habitat decides (R207): on the canvas the compositor
            // rounds and moves, in a foreign host the house does.
            state.corners = if matrixkit_theme::habitat::is_matrix() { 0.0 } else { state.client.corners() };
            if !matrixkit_theme::habitat::is_matrix() {
                if let Some(v) = &state.viewporter {
                    state.viewport = Some(v.get_viewport(&surface, &qh, ()));
                }
                // R207: never smaller than a phone — the form below that
                // does not exist.
                let (lw, lh) = state.client.least();
                if lw > 0 && lh > 0 {
                    top.set_min_size(lw as i32, lh as i32);
                    state.want = (width.max(lw), height.max(lh));
                }
            }
            surface.commit();
            state.xdg = Some(xdg);
            state.toplevel = Some(top);
        }
        Mount::Lock => {
            let manager = state
                .lock_manager
                .clone()
                .ok_or("kein ext_session_lock_manager_v1 — der Compositor kann nicht sperren")?;
            if state.outputs.iter().any(|(_, n)| n.is_none()) {
                queue.roundtrip(&mut state)?;
            }
            let wanted = state.client.output();
            let outputs = state.outputs.clone();
            let main = wanted
                .as_ref()
                .and_then(|w| outputs.iter().position(|(_, n)| n.as_deref() == Some(w.as_str())))
                .unwrap_or(0);
            let (main_output, main_name) = outputs.get(main).cloned().ok_or("kein Ausgang zum Sperren")?;
            let lock = manager.lock(&qh, ());
            state.lock_surface = Some(lock.get_lock_surface(&surface, &main_output, &qh, ()));
            // Round 124: screens the client's fleet covers get its panes.
            let plan = state.client.fleet();
            let taken: Vec<String> = plan.iter().filter_map(|spec| state.client.pane_place(&spec.pane).and_then(|pl| pl.output)).collect();
            for (k, (output, name)) in outputs.iter().enumerate() {
                if k == main || name.as_ref().is_some_and(|n| taken.contains(n)) {
                    continue;
                }
                let side = compositor.create_surface(&qh, ());
                let ls = lock.get_lock_surface(&side, output, &qh, ());
                eprintln!("[{}] Sperre: Nebenfläche auf {}", state.tag, name.as_deref().unwrap_or("?"));
                state.lock_others.push((ls, side, None));
            }
            eprintln!("[{}] Sperre: Hauptfläche auf {}", state.tag, main_name.as_deref().unwrap_or("?"));
            state.session_lock = Some(lock);
            state.lock_asked = Some(std::time::Instant::now());
        }
    }
    state.surface = Some(surface);

    let fd = conn.as_fd().as_raw_fd();
    let mut last_beat = std::time::Instant::now();
    let mut last_pulse = std::time::Instant::now();
    let mut last_glide = std::time::Instant::now();
    loop {
        // Round 136: a lock the compositor never confirmed is given up (the
        // lock object goes without an unlock — allowed before `locked`).
        if state.session_lock.is_some() && !state.lock_confirmed && state.lock_asked.is_some_and(|t| t.elapsed().as_secs() >= 6) {
            eprintln!("[{}] Sperre: nach 6 s nicht bestätigt — aufgegeben", state.tag);
            state.lock_refused = true;
            state.closed = true;
        }
        if state.closed {
            eprintln!("[{}] Fläche geschlossen — Ende.", state.tag);
            // Round 123: the lock opens only here — the client said Close.
            if let Some(lock) = state.session_lock.take() {
                // The lock surfaces go first in either case.
                for (ls, side, buffer) in state.lock_others.drain(..) {
                    ls.destroy();
                    side.destroy();
                    if let Some(b) = buffer {
                        b.destroy();
                    }
                }
                for pane in state.panes.iter_mut() {
                    if let Some(ls) = pane.lock.take() {
                        ls.destroy();
                    }
                }
                if let Some(ls) = state.lock_surface.take() {
                    ls.destroy();
                }
                if state.lock_refused {
                    lock.destroy();
                } else {
                    lock.unlock_and_destroy();
                    eprintln!("[{}] Sperre: entsperrt", state.tag);
                }
                let _ = queue.roundtrip(&mut state);
            }
            state.client.closing();
            if let Some(o) = state.overlay.as_mut() {
                o.closing();
            }
            if let Some(path) = &house_path {
                let _ = std::fs::remove_file(path);
            }
            return Ok(());
        }
        // Round 137: a hidden main surface is not painted (its panes are).
        if state.dirty && state.client.main_hidden() {
            state.dirty = false;
            state.frame_pending = false;
        }
        if state.dirty {
            state.dirty = false;
            // Set pass-through BEFORE painting (camp 47): the
            // new input region rides the buffer commit of
            // draw() — a merely state-carrying commit did not take
            // effect in the probe.
            let want_pass = state.client.passthrough();
            if want_pass != state.passthrough_applied {
                if let (Some(surface), Some(compositor)) = (&state.surface, &state.compositor) {
                    if want_pass {
                        let region = compositor.create_region(&qh, ());
                        surface.set_input_region(Some(&region));
                        region.destroy();
                    } else {
                        surface.set_input_region(None);
                    }
                    eprintln!(
                        "[{}] Durchlässigkeit: {}",
                        state.tag,
                        if want_pass { "AN (Eingabe fällt durch)" } else { "aus (Fläche hört)" }
                    );
                    state.passthrough_applied = want_pass;
                }
            }
            let moving = state.client.animating();
            let t0 = std::time::Instant::now();
            let asked_at = state.frame_at;
            if draw(&mut state, &qh).is_none() {
                state.dirty = true;
            }
            if let Some(line) = state.tick.drawn(moving, t0, asked_at) {
                eprintln!("[{}] {line}", state.tag);
                matrixkit_theme::runtime::trail("matrix-takt.log", &format!("[{}] {line}", state.tag), 300);
            }
            // Panels: check the desired size after painting
            // (camp 26) — the new size comes back as a configure.
            // Ask for the desired size ONCE. `extent()` may
            // consume state (the unroll size of the window key
            // is returned exactly once) — and an
            // `if let (Some(a), Some(b)) = (&x, f())` evaluates f()
            // EAGERLY, even if x is None. That is exactly where the
            // unrolling failed at first: the layer branch consumed
            // the value the window branch would have needed
            // (16.08.2026).
            // The DESIRED MARGIN (camp 82): like the desired size, asked once
            // per frame, so that a state-consuming
            // client is not asked twice.
            if let Some(layer) = state.layer.clone() {
                if let Some(m) = state.client.offset() {
                    if Some(m) != state.offset_applied {
                        state.offset_applied = Some(m);
                        layer.set_margin(m.0, m.1, m.2, m.3);
                        if let Some(s) = &state.surface {
                            s.commit();
                        }
                    }
                }
            }
            // The TRAFFIC LIGHT + (camp 92): the toggle wish goes to the
            // compositor after painting. The new size comes back as a
            // configure — the same path as any other
            // size change, no computation of its own.
            if let Some(on) = state.zoom_wish.take() {
                if let Some(top) = state.toplevel.clone() {
                    if on {
                        top.set_maximized();
                    } else {
                        top.unset_maximized();
                    }
                    if let Some(s) = &state.surface {
                        s.commit();
                    }
                }
            }
            // R207: the SHELF — on the canvas niri's shelf surface,
            // in a foreign host the compositor's minimise.
            if std::mem::take(&mut state.shelve_wish) {
                if let Some(top) = state.toplevel.clone() {
                    if matrixkit_theme::habitat::is_matrix() {
                        matrixkit_theme::canvas::window_to_shelf();
                    } else {
                        top.set_minimized();
                        if let Some(s) = &state.surface {
                            s.commit();
                        }
                    }
                }
            }
            let want = state.client.extent();
            // A MAXIMISED window has no desired size of its own:
            // the area belongs to the compositor, and a client that
            // writes against it fights it over the same number.
            let want = if state.maximized && state.layer.is_none() { None } else { want };
            if let Some(want) = want {
                if want != state.extent_applied {
                    let layer = state.layer.clone();
                    let xdg = state.xdg.clone();
                    let surface = state.surface.clone();
                    if let Some(layer) = layer {
                        state.extent_applied = want;
                        layer.set_size(want.0, want.1);
                        if let Some(s) = &surface {
                            s.commit();
                        }
                    } else if let Some(xdg) = xdg {
                        // WINDOWS may wish their size too
                        // (camp 77): the window key needs that,
                        // which rolls the window up onto the bar.
                        let _ = xdg;
                        state.extent_applied = want;
                        state.size = want;
                        // The next frame attaches the matching
                        // buffer AND reports the geometry with it.
                        state.full = true;
                        state.dirty = true;
                    }
                }
            }
        }
        // The fleet repaint (camp 35): fresh cards from
        // the plan, then every surface painted in full.
        if state.panes_dirty {
            state.panes_dirty = false;
            fleet_sync(&mut state, &qh);
            let fresh: Vec<(u32, C::Card)> =
                state.fleet.all().iter().map(|(w, p)| (*w, p.clone())).collect();
            for (wid, card) in fresh {
                if let Some(pn) = state.panes.iter_mut().find(|pn| pn.id == wid) {
                    pn.card = card;
                }
            }
            for idx in 0..state.panes.len() {
                let card = state.panes[idx].card.clone();
                if state.panes[idx].ready && state.client.pane_stale(&card) {
                    draw_pane(&mut state, idx, &qh);
                }
            }
        }
        conn.flush()?;
        // The sleep ends at the next deadline: beat (500 ms) or
        // key repeat — whichever comes first.
        let now = std::time::Instant::now();
        // Round 45 — the dark rhythm: panels asleep (power key, fold), so
        // nobody sees a frame. Beat every 3 s instead of 500 ms, pulse every
        // 12 s instead of 2 s, no spring frames. Measured on the Duo: the
        // panels off still drew ≈ 270 mA — the CPU never rested.
        let dark = matrixkit_theme::device::display_asleep();
        let beat_ms: u128 = if dark { 3000 } else { 500 };
        let beat_in = beat_ms.saturating_sub(now.duration_since(last_beat).as_millis());
        let mut wait = beat_in as i32;
        if let Some((_, _, _, next)) = &state.held {
            let rep_in = next.saturating_duration_since(now).as_millis() as i32;
            wait = wait.min(rep_in);
        }
        // Motion (camp 32): if the app springs, the house ticks
        // at frame rate — otherwise it sleeps its deadline sleep.
        let overlay_moving = state.overlay.as_ref().is_some_and(|o| o.moving());
        if (state.client.animating() || overlay_moving || state.wiggle.is_some()) && !dark {
            wait = wait.min(16);
        }
        if let Some(guard) = queue.prepare_read() {
            let mut fds = [
                libc::pollfd { fd, events: libc::POLLIN, revents: 0 },
                libc::pollfd { fd: wake_read, events: libc::POLLIN, revents: 0 },
            ];
            let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, wait.max(1)) };
            if n > 0 && fds[0].revents & libc::POLLIN != 0 {
                let _ = guard.read();
            } else {
                drop(guard);
            }
            if n > 0 && fds[1].revents & libc::POLLIN != 0 {
                // Drain the pipe — the material itself lies in the channel.
                let mut sink = [0u8; 64];
                while unsafe {
                    libc::read(wake_read, sink.as_mut_ptr() as *mut libc::c_void, sink.len())
                } > 0
                {}
            }
        }
        queue.dispatch_pending(&mut state)?;
        // Take home-coming threads (camp 25) — on the
        // loop, in order of arrival.
        while let Ok(w) = work_rx.try_recv() {
            let r = state.client.woven(w);
            state.refresh(r);
        }
        while let Ok(word) = house_rx.try_recv() {
            eprintln!("[{}] Leiste sagt «{word}»", state.tag);
            match word.as_str() {
                "root" => {
                    let r = state.client.root();
                    state.refresh(r);
                }
                "close" => state.closed = true,
                other => {
                    if state.overlay.as_mut().is_some_and(|o| o.word(other)) {
                        state.refresh(Refresh::Dirty);
                    }
                }
            }
        }
        // Fire a due repeat (rate from the compositor).
        let now = std::time::Instant::now();
        if let Some((raw, k, shift, next)) = state.held.clone() {
            if now >= next {
                let r = state.client.key(k.clone(), shift);
                state.refresh(r);
                let step = std::time::Duration::from_millis(
                    (1000 / state.repeat_rate.max(1)) as u64,
                );
                state.held = Some((raw, k, shift, next + step));
            }
        }
        // The frame beat (camp 32): only while the app springs;
        // dt capped, so that an overslept step does not make the spring
        // overshoot.
        let overlay_moving = state.overlay.as_ref().is_some_and(|o| o.moving());
        let dark = matrixkit_theme::device::display_asleep();
        // Round 164: the wiggle ends after 8 s at the latest (a touch drag
        // brings nothing back to us when it ends).
        if state.wiggle.is_some_and(|t| t.elapsed().as_secs_f32() > 8.0) {
            wiggle_end(&mut state);
        }
        if (state.client.animating() || overlay_moving || state.wiggle.is_some()) && !dark {
            // Camp 12: if a frame callback is pending, the
            // COMPOSITOR sets the beat — otherwise the poll drifts against vsync
            // and causes micro-stutter. The 100 ms ripcord keeps
            // covered windows alive.
            let held = state.frame_pending
                && !state.client.main_hidden()
                && state
                    .frame_at
                    .is_some_and(|t| now.duration_since(t).as_millis() < 100);
            let dt = now.duration_since(last_glide).as_secs_f32();
            // Round 107: at most one step per display frame (60 Hz). A
            // client whose main surface draws nothing (the bar's 1-px
            // spine) gets no frame callbacks — its panes were stepped every
            // 8 ms and the island painted 125 times a second.
            if !held && dt >= 0.0155 {
                let r = if state.client.animating() { state.client.glide(dt.min(0.05)) } else { Refresh::Calm };
                state.refresh(r);
                if overlay_moving && state.overlay.as_mut().is_some_and(|o| o.glide(dt.min(0.05))) {
                    state.refresh(Refresh::Dirty);
                }
                if state.wiggle.is_some() {
                    state.refresh(Refresh::Full);
                }
                last_glide = now;
            }
        } else {
            last_glide = now;
        }
        // Camp 106: has a press been held long enough without wandering?
        press_check(&mut state, now);
        // The 500 ms beat (blinking) and the app's 2 s pulse.
        if now.duration_since(last_beat).as_millis() >= if dark { 3000 } else { 500 } {
            last_beat = now;
            big_cores::rest();
            // Round 117: warm the fleet GPU once the house is idle a while.
            if state.beats_seen == 6 && state.pane_gpu.is_none() && !state.pane_gpu_failed && state.client.warm_gpu() && !state.client.animating() {
                let t = std::time::Instant::now();
                if fleet_gpu(&mut state, &qh) {
                    eprintln!("[{}] Flotten-GPU vorgewärmt in {} ms", state.tag, t.elapsed().as_millis());
                }
            }
            // Round 98: the purge contract (mk::memory) — under high memory
            // pressure the fleet's GPU device goes while no pane paints with
            // it; the next card makes it anew.
            state.beats_seen = state.beats_seen.wrapping_add(1);
            if state.beats_seen % 10 == 0
                && state.pane_gpu.is_some()
                && state.panes.iter().all(|p| p.target.is_none())
                && mk_memory_high()
            {
                if let Some((gpu, helper)) = state.pane_gpu.take() {
                    drop(gpu);
                    helper.destroy();
                    eprintln!("[{}] Speicher knapp: Flotten-GPU freigegeben", state.tag);
                }
            }
            let r = state.client.beat();
            state.refresh(r);
            // R209: a typing window in focus holds the on-screen keyboard
            // (finger devices; the guard closes it 4 s after the last word).
            if state.keyboard_focus
                && state.client.wants_keyboard()
                && matrixkit_theme::device::finger_first()
            {
                matrixkit_theme::onscreen_keyboard::send("open!");
            }
        }
        if now.duration_since(last_pulse).as_secs() >= if dark { 12 } else { 2 } {
            last_pulse = now;
            // Round 52: every 60th pulse (2 min awake) the leak probe.
            state.leak_ticks = state.leak_ticks.wrapping_add(1);
            if state.leak_ticks % 60 == 0 {
                if let Some(warning) = state.leak_probe.check_self() {
                    let line = format!("{} [{}] {warning}", matrixkit_theme::time::clock_hms(), state.tag);
                    eprintln!("[{}] {warning}", state.tag);
                    matrixkit_theme::runtime::trail("matrix-lecksonde.log", &line, 200);
                }
            }
            let r = state.client.pulse();
            state.refresh(r);
            // R207: in a foreign host the palette follows the wallpaper —
            // check path and mtime every 4 s, seed anew on change.
            if !matrixkit_theme::habitat::is_matrix() {
                state.seed_ticks += 1;
                if state.seed_ticks % 2 == 0 {
                    watch_wallpaper(&mut state);
                }
            }
        }
        // Delivering the clipboard (camp 88): what the app dropped in
        // is now offered to the seat. Only here, because
        // `set_selection` needs a timestamp — and there is
        // one only once the input has gone through.
        if let Some(text) = board::take_outgoing() {
            match (&state.data_device, &state.data_manager) {
                (Some(device), Some(manager)) => {
                    let source = manager.create_data_source(&qh, text);
                    source.offer(String::from("text/plain;charset=utf-8"));
                    source.offer(String::from("text/plain"));
                    source.offer(String::from("UTF8_STRING"));
                    device.set_selection(Some(&source), state.serial);
                    // The old source goes only NOW: the compositor
                    // sends it `cancelled` right away, and it cleans
                    // itself up.
                    state.data_source = Some(source);
                }
                _ => eprintln!("[{}] Kopieren ohne Ablage — der Compositor bietet keine an", state.tag),
            }
        }
        // Camp 108: dropped files to the app.
        if let Some((paths, x, y)) = drops::take() {
            let r = state.client.dropped(paths, x, y);
            state.refresh(r);
        }
        // The IME line follows the app's text wish.
        let want = state.client.wants_text();
        if want != state.text_on {
            state.text_on = want;
            if let Some(ti) = &state.text_input {
                if want {
                    ti.enable();
                } else {
                    ti.disable();
                }
                ti.commit();
            }
        }
    }
}

#[cfg(test)]
mod tick_tests {
    use super::TickProbe;
    use std::time::{Duration, Instant};

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn steady_frames_stay_quiet() {
        let t0 = Instant::now();
        let mut p = TickProbe::default();
        for i in 0..20 {
            let s = t0 + ms(i * 41);
            assert_eq!(p.drawn_at(true, s, s + ms(10), None), None);
        }
    }

    #[test]
    fn a_gap_is_split_and_reported() {
        let t0 = Instant::now();
        let mut p = TickProbe::default();
        p.drawn_at(true, t0, t0 + ms(14), None);
        p.done_at = Some(t0 + ms(44)); // released 30 ms after the commit
        let line = p.drawn_at(true, t0 + ms(52), t0 + ms(66), Some(t0 + ms(14))).unwrap();
        assert!(line.contains("Takt-Lücke 52 ms"), "{line}");
        assert!(line.contains("Zeichnen davor 14 ms"), "{line}");
        assert!(line.contains("Warten auf den Compositor 30 ms"), "{line}");
        assert!(line.contains("Rest (Schleife/App) 8 ms"), "{line}");
    }

    #[test]
    fn a_pause_is_no_gap_and_lines_are_rate_limited() {
        let t0 = Instant::now();
        let mut p = TickProbe::default();
        p.drawn_at(true, t0, t0 + ms(5), None);
        p.drawn_at(false, t0 + ms(500), t0 + ms(505), None); // stood still
        assert_eq!(p.drawn_at(true, t0 + ms(2000), t0 + ms(2005), None), None);
        assert!(p.drawn_at(true, t0 + ms(2060), t0 + ms(2065), None).is_some());
        assert_eq!(p.drawn_at(true, t0 + ms(2120), t0 + ms(2125), None), None); // < 1 s later
        assert!(p.drawn_at(true, t0 + ms(3200), t0 + ms(3205), None).is_some());
    }
}

#[cfg(test)]
mod guards {
    use super::{blit_rows, burden_pool, edges, rollable, snap, text_near, visible, Scene, Text};
    mod matrixkit_shell_edges {
        pub use wayland_protocols::xdg::shell::client::xdg_toplevel::ResizeEdge as E;
    }
    use matrixkit_theme::scene::Rect;

    /// The blit guard (camp 104): shifting in the canvas
    /// hits exactly the rows of the area, nothing beside them.
    #[test]
    fn blit_shifts_only_the_area() {
        // 4×6 logical pixels, scale 1; every row carries its
        // index as colour value.
        let (w, h) = (4u32, 6u32);
        let mut canvas = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                canvas[((y * w + x) * 4) as usize] = y as u8 + 1;
            }
        }
        let area = Rect { x: 1.0, y: 1.0, w: 2.0, h: 4.0 };
        // Content moves 2 DOWN (dy > 0).
        blit_rows(&mut canvas, &area, 2, 1, w, h);
        macro_rules! at {
            ($x:expr, $y:expr) => {
                canvas[(($y * w + $x) * 4) as usize]
            };
        }
        // Outside the area untouched.
        for y in 0..h {
            assert_eq!(at!(0, y), y as u8 + 1, "Spalte links");
            assert_eq!(at!(3, y), y as u8 + 1, "Spalte rechts");
        }
        assert_eq!(at!(1, 0), 1, "über der Fläche");
        assert_eq!(at!(1, 5), 6, "unter der Fläche");
        // Rows 3..5 now carry the state of 1..3.
        assert_eq!(at!(1, 3), 2);
        assert_eq!(at!(2, 4), 3);
        // And back UPWARDS: the strip at the top is the source again.
        blit_rows(&mut canvas, &area, -2, 1, w, h);
        assert_eq!(at!(1, 1), 2);
        assert_eq!(at!(2, 2), 3);
    }

    /// Multiset pairing (camp 104): scrolling citizens move within the
    /// vector — found again means no damage, no matter at which
    /// index; only what vanished and what is new pays.
    #[test]
    fn burden_pool_pairs_across_indices() {
        let mk_t = |y: f32, s: &str| Text {
            s: s.into(),
            px: 14.0,
            weight: matrixkit_theme::typo::Weight::Normal,
            family: "T",
            color: matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            x: 0.0,
            y,
            home: Rect { x: 0.0, y, w: 50.0, h: 10.0 },
            anchor: 0,
            clip: None,
        };
        // Old: A, B, C — new: B, C, D (A scrolled out at the top, D new).
        let old = vec![mk_t(0.0, "A"), mk_t(10.0, "B"), mk_t(20.0, "C")];
        let new = vec![mk_t(10.0, "B"), mk_t(20.0, "C"), mk_t(30.0, "D")];
        let mut damage = Vec::new();
        burden_pool(&old, &new, text_near, |t| t.home, |_| true, &mut damage);
        // Damage: A (gone) and D (new) — B and C pair with shifted indices.
        assert!(damage.iter().any(|r| r.y == 0.0), "A zahlt");
        assert!(damage.iter().any(|r| r.y == 30.0), "D zahlt");
        assert!(!damage.iter().any(|r| r.y == 10.0), "B zahlt nicht");
        assert!(!damage.iter().any(|r| r.y == 20.0), "C zahlt nicht");
    }

    /// Blit capability: a citizen WITHOUT the view box of the
    /// area (veil, ghost, detail card) blunts it.
    #[test]
    fn rollable_rejects_aliens() {
        let area = Rect { x: 10.0, y: 10.0, w: 100.0, h: 100.0 };
        let mut s = Scene::default();
        s.placed.push(matrixkit_theme::scene::Placed {
            x: 20.0,
            y: 20.0,
            w: 30.0,
            h: 30.0,
            color: matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            radius: 0.0,
            clip: Some(area),
            shadow: None,
        });
        assert!(rollable(&s, &area), "Bürger mit Kasten: tauglich");
        s.placed.push(matrixkit_theme::scene::Placed {
            x: 15.0,
            y: 15.0,
            w: 10.0,
            h: 10.0,
            color: matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.5 },
            radius: 0.0,
            clip: None,
            shadow: None,
        });
        assert!(!rollable(&s, &area), "Schleier ohne Kasten: stumpf");
        // Outside the area a foreign box does not disturb.
        s.placed[1].x = 200.0;
        s.placed[1].y = 200.0;
        assert!(rollable(&s, &area), "fern der Fläche: tauglich");
        // A flat GROUND under everything is allowed — exactly the
        // Werft case (window-filling quad, place 0, no box).
        s.placed.push(matrixkit_theme::scene::Placed {
            x: 0.0,
            y: 0.0,
            w: 500.0,
            h: 500.0,
            color: matrixkit_theme::Rgba { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
            radius: 0.0,
            clip: None,
            shadow: None,
        });
        assert!(rollable(&s, &area), "flacher Grund: tauglich");
        // With rounding it would NOT be vertically a single colour in the corners.
        s.placed[2].radius = 8.0;
        assert!(!rollable(&s, &area), "gerundeter Grund: stumpf");
        s.placed[2].radius = 0.0;
        // A ground that only SPLITS the area is no ground.
        s.placed[2].h = 60.0;
        assert!(!rollable(&s, &area), "halber Grund: stumpf");
        let _ = visible(Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }, None);
    }

    /// The patch guard (camp 89): reset and repaint
    /// must hit THE SAME pixels.
    ///
    /// Reset goes via `edges` (rounded logically, then
    /// ×scale), repaint via `r * k` with truncation. If the two
    /// gape apart, the difference stays black — exactly the
    /// hair-thin lines the owner saw in the terminal. After `snap`
    /// not a single pixel may lie between them.
    #[test]
    fn reset_and_repaint_cover_the_same_pixels() {
        let krumm = [
            Rect { x: 0.0, y: 506.6, w: 840.0, h: 18.4 },
            Rect { x: 13.2, y: 55.5, w: 120.9, h: 17.5 },
            Rect { x: 0.1, y: 0.9, w: 839.2, h: 559.4 },
            Rect { x: 7.75, y: 200.25, w: 3.5, h: 0.5 },
        ];
        for k in [1u32, 2, 3] {
            for r in &krumm {
                let s = snap(r);
                // Whole logical pixels — otherwise each half
                // rounds on its own again.
                assert_eq!(s.x, s.x.floor(), "x nicht ganz: {:?}", s.x);
                assert_eq!(s.y, s.y.floor(), "y nicht ganz: {:?}", s.y);
                assert_eq!((s.x + s.w), (s.x + s.w).floor(), "rechts nicht ganz");
                assert_eq!((s.y + s.h), (s.y + s.h).floor(), "unten nicht ganz");

                // Reset: as in draw()/present().
                let (x0, y0, x1, y1) = edges(&s, 840, 560);
                let (rx0, ry0, rx1, ry1) = (x0 * k, y0 * k, x1 * k, y1 * k);

                // Repaint: as in compose()/paint() — first stretch,
                // then truncate or round up.
                let kf = k as f32;
                let (px0, py0) = ((s.x * kf).max(0.0) as u32, (s.y * kf).max(0.0) as u32);
                let px1 = ((s.x + s.w) * kf).ceil().max(0.0) as u32;
                let py1 = ((s.y + s.h) * kf).ceil().max(0.0) as u32;

                assert_eq!(
                    (rx0, ry0, rx1, ry1),
                    (px0, py0, px1.min(840 * k), py1.min(560 * k)),
                    "Zurücksetzen und Neumalen klaffen bei {r:?}, Skalierung {k}"
                );
            }
        }
    }

    /// The edge guard (camp 89): every corner and every edge must
    /// report its grip, the middle NONE — otherwise the
    /// window frame eats the app's clicks.
    #[test]
    fn every_edge_offers_its_grip_and_the_middle_none() {
        use super::grip_edge;
        use matrixkit_shell_edges::*;
        let (w, h) = (840.0, 560.0);
        assert_eq!(grip_edge(2.0, 2.0, w, h), Some(E::TopLeft));
        assert_eq!(grip_edge(838.0, 2.0, w, h), Some(E::TopRight));
        assert_eq!(grip_edge(2.0, 558.0, w, h), Some(E::BottomLeft));
        assert_eq!(grip_edge(838.0, 558.0, w, h), Some(E::BottomRight));
        assert_eq!(grip_edge(400.0, 1.0, w, h), Some(E::Top));
        assert_eq!(grip_edge(400.0, 559.0, w, h), Some(E::Bottom));
        assert_eq!(grip_edge(1.0, 300.0, w, h), Some(E::Left));
        assert_eq!(grip_edge(839.0, 300.0, w, h), Some(E::Right));
        // The middle belongs to the app.
        assert_eq!(grip_edge(400.0, 300.0, w, h), None);
        assert_eq!(grip_edge(9.0, 9.0, w, h), None);
        // A tiny window has no edge to grab — otherwise
        // it would be ALL edge and no longer usable.
        assert_eq!(grip_edge(5.0, 5.0, 12.0, 12.0), None);
    }

    /// And the counter-test: WITHOUT snap it really gapes — otherwise
    /// the guard above guards a matter of course.
    #[test]
    fn without_snap_a_black_hairline_is_left_behind() {
        let r = Rect { x: 0.0, y: 506.6, w: 840.0, h: 18.4 };
        let k = 2u32;
        let (_, y0, _, _) = edges(&r, 840, 560);
        let zurueck = y0 * k; // 1012
        let neu = (r.y * k as f32) as u32; // 1013
        assert!(zurueck < neu, "die Lücke ist die Krankheit, nicht die Kur");
        assert_eq!(neu - zurueck, 1, "genau eine schwarze Bildzeile");
    }
}

#[cfg(test)]
mod shifted_tests {
    use super::*;

    /// Round 172: a column laid at x = 0 lands `dx` to the right — boxes,
    /// their clips and the anchors of what hangs on them.
    #[test]
    fn a_column_moves_with_everything() {
        let clip = Rect { x: 0.0, y: 0.0, w: 50.0, h: 50.0 };
        let mut col = SceneBuilder::new();
        col.quad(10.0, 20.0, 30.0, 40.0, matrixkit_theme::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, 0.0, Some(7), Some(clip));
        let part = col.into_part();
        let mut b = SceneBuilder::new();
        b.quad(0.0, 0.0, 5.0, 5.0, matrixkit_theme::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, 0.0, None, None);
        b.append_shifted(&part, 100.0);
        let q = b.scene.placed[1];
        assert_eq!((q.x, q.y), (110.0, 20.0));
        assert_eq!(q.clip.map(|c| c.x), Some(100.0));
        assert_eq!(b.scene.roles[1], Some(7));
    }
}
