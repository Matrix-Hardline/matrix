//! Matrix GPU painting (stages 1–2, see GRABUNG-GPU-MALEREI.md):
//! wgpu DIRECTLY on the window house's wl_surface — without winit,
//! without iced. Stage 1 is the SEAM (device on the surface,
//! clear colour); stage 2 the QUAD PIPELINE: the scene's `Placed`
//! citizens as ONE instanced draw — rounding, anti-aliasing and
//! shadow are computed by the fragment from a signed distance function
//! (the core finding of the dig, camp 2).
//!
//! The beat stays with the house: painting happens when the house paints —
//! no render loop of its own runs here.

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};

/// A tiny block_on for the two wgpu futures during setup —
/// saves the pollster dependency. The waker parks the thread and
/// wakes it again; for setup futures (one-off, short) that is enough.
pub mod nv12;

fn block_on<F: std::future::Future>(mut fut: F) -> F::Output {
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    struct Alarm(std::thread::Thread);
    impl Wake for Alarm {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    // Safety: `fut` lives on this stack and is never moved
    // while the pin lives.
    let mut fut = unsafe { std::pin::Pin::new_unchecked(&mut fut) };
    let waker = Waker::from(Arc::new(Alarm(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(out) => return out,
            Poll::Pending => std::thread::park(),
        }
    }
}

/// The quad shader — after the dig's template (iced_wgpu
/// solid.wgsl), slimmed down to our citizens: ONE radius instead of
/// four, no border, the shadow stays. Coordinates are PIXELS;
/// the ortho matrix in the globals brings them to NDC.
const QUAD_SHADER: &str = r#"
struct Globals {
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;

// The unit quad from the vertex index (dig, vertex.wgsl):
// #: 0 1 2 3 4 5 → x: 1 1 0 0 0 1, y: 1 0 0 0 1 1
fn vertex_position(vertex_index: u32) -> vec2<f32> {
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u));
}

fn rounded_box_sdf(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - half_size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - radius;
}

struct VsIn {
    @builtin(vertex_index) index: u32,
    @location(0) color: vec4<f32>,
    @location(1) pos: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) shadow_blur: f32,
    @location(5) shadow_offset: vec2<f32>,
    @location(6) shadow_color: vec4<f32>,
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) pos: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) shadow_blur: f32,
    @location(5) shadow_offset: vec2<f32>,
    @location(6) shadow_color: vec4<f32>,
}

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    // The instance grows by the shadow offset and softness so the
    // shadow has room inside the triangle pair (dig, Lager 2).
    let grow = vec2<f32>(in.shadow_blur, in.shadow_blur);
    let origin = in.pos + min(in.shadow_offset, vec2<f32>(0.0)) - grow;
    let span = in.size + abs(in.shadow_offset) + 2.0 * grow;
    let p = origin + span * vertex_position(in.index);
    out.position = globals.transform * vec4<f32>(p, 0.0, 1.0);
    // Premultiplied, throughout.
    out.color = vec4<f32>(in.color.rgb * in.color.a, in.color.a);
    out.shadow_color = vec4<f32>(in.shadow_color.rgb * in.shadow_color.a, in.shadow_color.a);
    out.pos = in.pos;
    out.size = in.size;
    out.radius = min(in.radius, min(in.size.x, in.size.y) / 2.0);
    out.shadow_blur = in.shadow_blur;
    out.shadow_offset = in.shadow_offset;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let center = in.pos + in.size * 0.5;
    let dist = rounded_box_sdf(in.position.xy - center, in.size * 0.5, in.radius);
    // Anti-aliasing falls out of the distance for free.
    let quad_alpha = clamp(0.5 - dist, 0.0, 1.0);
    let quad_color = in.color * quad_alpha;
    if in.shadow_color.a > 0.0 {
        let sdist = rounded_box_sdf(
            in.position.xy - center - in.shadow_offset,
            in.size * 0.5,
            in.radius,
        );
        let shadow_alpha =
            1.0 - smoothstep(-in.shadow_blur, in.shadow_blur, max(sdist, 0.0));
        return mix(quad_color, in.shadow_color, (1.0 - quad_alpha) * shadow_alpha);
    }
    return quad_color;
}
"#;

/// The image shader (stage 4): a textured unit quad, rounded
/// corners via the same SDF as for the quads. All textures lie
/// PREMULTIPLIED in memory — the shader never multiplies.
const IMAGE_SHADER: &str = r#"
struct Globals {
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

fn vertex_position(vertex_index: u32) -> vec2<f32> {
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u));
}

fn rounded_box_sdf(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - half_size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - radius;
}

struct VsIn {
    @builtin(vertex_index) index: u32,
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) radius: f32,
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) pos: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) radius: f32,
}

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let unit = vertex_position(in.index);
    out.position = globals.transform * vec4<f32>(in.pos + in.size * unit, 0.0, 1.0);
    out.uv = unit;
    out.pos = in.pos;
    out.size = in.size;
    out.radius = min(in.radius, min(in.size.x, in.size.y) / 2.0);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let center = in.pos + in.size * 0.5;
    let dist = rounded_box_sdf(in.position.xy - center, in.size * 0.5, in.radius);
    let mask = clamp(0.5 - dist, 0.0, 1.0);
    return textureSample(tex, samp, in.uv) * mask;
}

// R210: a video frame as planar YUV 4:2:0 in ONE R8 texture of
// w x (h*3/2): the Y plane, then U and V (w/2 x h/2 each) packed row by
// row. Luma filters bilinearly inside its plane, chroma is fetched; BT.709
// limited range — what ffmpeg's yuv420p of a web video carries.
@fragment
fn fs_yuv(in: VsOut) -> @location(0) vec4<f32> {
    let center = in.pos + in.size * 0.5;
    let dist = rounded_box_sdf(in.position.xy - center, in.size * 0.5, in.radius);
    let mask = clamp(0.5 - dist, 0.0, 1.0);
    let dims = vec2<i32>(textureDimensions(tex));
    let w = dims.x;
    let h = dims.y * 2 / 3;
    let ly = clamp(in.uv.y * f32(h), 0.5, f32(h) - 0.5) / f32(dims.y);
    let luma = textureSample(tex, samp, vec2<f32>(in.uv.x, ly)).r;
    let px = clamp(vec2<i32>(in.uv * vec2<f32>(f32(w), f32(h))), vec2<i32>(0), vec2<i32>(w - 1, h - 1));
    let c = (px.y / 2) * (w / 2) + px.x / 2;
    let uo = w * h + c;
    let vo = w * h + (w / 2) * (h / 2) + c;
    let u = textureLoad(tex, vec2<i32>(uo % w, uo / w), 0).r - 0.5019608;
    let v = textureLoad(tex, vec2<i32>(vo % w, vo / w), 0).r - 0.5019608;
    let l = (luma - 0.0627451) * 1.164;
    let rgb = clamp(vec3<f32>(l + 1.793 * v, l - 0.213 * u - 0.533 * v, l + 2.112 * u), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb, 1.0) * mask;
}

// Round 67: NV12 in the same R8 stack — the Y plane, then ONE plane of
// interleaved U/V (w bytes per row, h/2 rows): the Venus hardware
// decoder's own layout, so its frames reach the GPU with row copies only.
@fragment
fn fs_nv12(in: VsOut) -> @location(0) vec4<f32> {
    let center = in.pos + in.size * 0.5;
    let dist = rounded_box_sdf(in.position.xy - center, in.size * 0.5, in.radius);
    let mask = clamp(0.5 - dist, 0.0, 1.0);
    let dims = vec2<i32>(textureDimensions(tex));
    let w = dims.x;
    let h = dims.y * 2 / 3;
    let ly = clamp(in.uv.y * f32(h), 0.5, f32(h) - 0.5) / f32(dims.y);
    let luma = textureSample(tex, samp, vec2<f32>(in.uv.x, ly)).r;
    let px = clamp(vec2<i32>(in.uv * vec2<f32>(f32(w), f32(h))), vec2<i32>(0), vec2<i32>(w - 1, h - 1));
    let row = h + px.y / 2;
    let col = (px.x / 2) * 2;
    let u = textureLoad(tex, vec2<i32>(col, row), 0).r - 0.5019608;
    let v = textureLoad(tex, vec2<i32>(min(col + 1, w - 1), row), 0).r - 0.5019608;
    let l = (luma - 0.0627451) * 1.164;
    let rgb = clamp(vec3<f32>(l + 1.793 * v, l - 0.213 * u - 0.533 * v, l + 2.112 * u), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(rgb, 1.0) * mask;
}
"#;

/// LIVE GLASS (round 125): the desktop's liquid glass (niri glass.frag,
/// the card rule) drawn by the app itself — every frame anew from what
/// lies behind NOW. The picture is a LENS MAP of its shape (made once on
/// the CPU, `matrixkit_scene::glass_clock::lens::map`): rg = the outward
/// normal (0.5 + 0.5 n), b = the rim band (0 deep inside … 1 at the edge,
/// squared as in niri), a = coverage. The backdrop texture (the wallpaper
/// picture of the same scene) is sampled at the CURRENT screen position,
/// bent along the normal in the rim band with the chromatic fringe
/// (1.15 / 1.0 / 0.85), frosted like one Kawase pass, dimmed by the
/// legibility scrim, lit along the vertical axis with shaded flanks.
const LENS_SHADER: &str = r#"
struct Globals {
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var map_tex: texture_2d<f32>;
@group(1) @binding(1) var map_samp: sampler;
@group(2) @binding(0) var back_tex: texture_2d<f32>;
@group(2) @binding(1) var back_samp: sampler;

fn vertex_position(vertex_index: u32) -> vec2<f32> {
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u));
}

struct VsIn {
    @builtin(vertex_index) index: u32,
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    // backdrop placement in pixels: x, y, w, h
    @location(2) back: vec4<f32>,
    // bend (px), scrim, extra rim light, veil (darkening of the backdrop)
    @location(3) params: vec4<f32>,
    // frost offset (px), body tint, opacity, -
    @location(4) more: vec4<f32>,
}

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) back: vec4<f32>,
    @location(2) params: vec4<f32>,
    @location(3) more: vec4<f32>,
}

@vertex
fn vs_lens(in: VsIn) -> VsOut {
    var out: VsOut;
    let unit = vertex_position(in.index);
    out.position = globals.transform * vec4<f32>(in.pos + in.size * unit, 0.0, 1.0);
    out.uv = unit;
    out.back = in.back;
    out.params = in.params;
    out.more = in.more;
    return out;
}

fn back_at(p: vec2<f32>, b: vec4<f32>) -> vec3<f32> {
    let uv = clamp((p - b.xy) / b.zw, vec2<f32>(0.0), vec2<f32>(1.0));
    return textureSampleLevel(back_tex, back_samp, uv, 0.0).rgb;
}

// One Kawase pass: four diagonal taps.
fn frost(p: vec2<f32>, o: f32, b: vec4<f32>) -> vec3<f32> {
    return 0.25 * (back_at(p + vec2<f32>(-o, -o), b) + back_at(p + vec2<f32>(o, -o), b)
        + back_at(p + vec2<f32>(-o, o), b) + back_at(p + vec2<f32>(o, o), b));
}

@fragment
fn fs_lens(in: VsOut) -> @location(0) vec4<f32> {
    let m = textureSampleLevel(map_tex, map_samp, in.uv, 0.0);
    let cov = m.a;
    if (cov <= 0.002) {
        return vec4<f32>(0.0);
    }
    let n = normalize(m.rg * 2.0 - vec2<f32>(1.0) + vec2<f32>(1e-5));
    let rim = m.b;
    let p = in.position.xy;
    let bend = in.params.x * rim;
    let o = in.more.x;
    var c = vec3<f32>(
        frost(p - n * bend * 1.15, o, in.back).r,
        frost(p - n * bend, o, in.back).g,
        frost(p - n * bend * 0.85, o, in.back).b);
    c = c * (1.0 - in.params.w);
    if (in.params.y > 0.0) {
        let lum = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
        c = c * (1.0 - smoothstep(0.55, 0.95, lum) * in.params.y);
    }
    let up = -n.y;
    let rim_edge = rim * rim * rim;
    var lit = abs(up) * 0.72;
    if (up >= 0.0) {
        lit = abs(up);
    }
    c = c + (vec3<f32>(1.0) - c) * in.more.y;
    c = clamp(c + vec3<f32>(rim_edge * lit * 0.22 + rim_edge * in.params.z - rim_edge * abs(n.x) * 0.08), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(c * cov, cov) * clamp(in.more.z, 0.0, 1.0);
}
"#;

/// The MATRIX RAIN (overkill 2): procedural digital rain as a
/// fragment shader — columns of falling pseudo-glyphs (3×5 bit patterns
/// from hashes, no font needed), head cell bright, the tail fades
/// exponentially. Meant to be subtle: strength ~0.3, behind the
/// content, above the ground.
const RAIN_SHADER: &str = r#"
struct RainGlobals {
    // x: time s, y: strength, z/w: area in px
    a: vec4<f32>,
    // xyz: ink, w: cell width px
    b: vec4<f32>,
}

@group(0) @binding(0) var<uniform> g: RainGlobals;

fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    // One triangle over the whole screen.
    let x = f32(i32(i) / 2) * 4.0 - 1.0;
    let y = f32(i32(i) % 2) * 4.0 - 1.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let time = g.a.x;
    let strength = g.a.y;
    let cw = g.b.w;
    let ch = cw * 1.35;
    let col = floor(pos.x / cw);
    let row = floor(pos.y / ch);
    let rows = g.a.w / ch;
    // Each column falls with its own speed and offset.
    let seed = hash(vec2<f32>(col, 7.0));
    let speed = 4.0 + seed * 9.0;
    let head = fract(seed * 13.7 + time * speed / rows) * (rows + 24.0) - 12.0;
    let dist = head - row;
    if dist < 0.0 {
        return vec4<f32>(0.0);
    }
    var b = exp(-dist * 0.16);
    // Glyph flicker: cells change their pattern in rhythm.
    let tick = floor(time * (2.0 + seed * 4.0));
    let alive = step(0.22, hash(vec2<f32>(col * 31.0 + row, tick)));
    b = b * alive;
    // The 3×5 bit pattern of the pseudo glyph.
    let local = vec2<f32>(
        floor((pos.x - col * cw) / (cw / 3.0)),
        floor((pos.y - row * ch) / (ch / 5.0)),
    );
    let bit = step(0.45, hash(local + vec2<f32>(col * 17.0, row * 5.0 + tick)));
    b = b * bit;
    // The head cell glows brighter and whiter.
    let is_head = step(dist, 1.0);
    let tint = mix(g.b.xyz, vec3<f32>(0.85, 1.0, 0.9), is_head * 0.7);
    let a = clamp(b * strength * (1.0 + is_head * 1.2), 0.0, 1.0);
    return vec4<f32>(tint * a, a);
}
"#;

/// ONE quad instance — 64 bytes, congruent with the
/// shader's vertex attributes. Coordinates in PIXELS.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GpuQuad {
    pub color: [f32; 4],
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub radius: f32,
    pub shadow_blur: f32,
    pub shadow_offset: [f32; 2],
    pub shadow_color: [f32; 4],
}

/// A run of equally clipped quads: clip box (pixels,
/// None = whole surface) and the instance range. The scene delivers
/// its citizens in Z order — consecutive ones with the same
/// box become ONE draw (dig, camp 3: scissor groups).
#[derive(Clone, Debug, PartialEq)]
pub struct QuadRun {
    pub clip: Option<(u32, u32, u32, u32)>,
    pub range: std::ops::Range<u32>,
}

/// ONE text run for the GPU (stage 3): shaped LOGICALLY
/// (cosmic-text), rasterised with the scale — position and bounds
/// in PIXELS, as with the quads.
#[derive(Clone, Debug)]
pub struct GpuText<'a> {
    pub s: &'a str,
    /// Font size LOGICAL (the scale rasterises more densely).
    pub px: f32,
    /// Line height LOGICAL.
    pub line: f32,
    /// cosmic weight (400/500/600 …).
    pub weight: u16,
    pub family: &'a str,
    pub color: [f32; 4],
    pub x: f32,
    pub y: f32,
    /// Clip bounds in pixels (x0, y0, x1, y1); None = unbounded.
    pub bounds: Option<(i32, i32, i32, i32)>,
}

/// ONE image for the GPU (stage 4): the key addresses the
/// texture cache; the pixels only count the FIRST time (upload).
/// Curves come ready PREMULTIPLIED from tiny-skia, decoded images
/// come straight and are multiplied in during upload.
#[derive(Clone, Debug)]
pub struct GpuImage<'a> {
    pub key: (String, u32, u32),
    pub rgba: &'a [u8],
    pub premultiplied: bool,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub clip: Option<(u32, u32, u32, u32)>,
    /// R210 stream slot: pictures that replace each other (video frames)
    /// share ONE texture of this slot — a new key of the same size is
    /// written into it instead of allocating a texture and bind group
    /// per frame (that, plus the stale frames waiting for the sweep,
    /// made video stutter on the duo).
    pub stream: Option<String>,
    /// R210: `rgba` holds planar YUV 4:2:0 (w*h*3/2 bytes) — uploaded as
    /// one R8 texture and converted in the shader (`fs_yuv`).
    pub yuv: bool,
    /// Round 67: with `yuv`, the chroma is NV12 (interleaved UV after the
    /// Y plane — the Venus hardware decoder's layout); shader `fs_nv12`.
    pub nv12: bool,
    /// Round 125: this picture is a LENS MAP drawn as live glass over the
    /// backdrop texture (`LENS_SHADER`). Upload it with `premultiplied`
    /// true — the map's channels are data, not colour.
    pub lens: Option<GpuLens>,
}

/// Round 125: live glass over a backdrop of the same frame.
#[derive(Clone, Debug)]
pub struct GpuLens {
    /// The backdrop picture's texture key (it is drawn in this frame too).
    pub backdrop: (String, u32, u32),
    /// Where the backdrop lies, in pixels: x, y, w, h.
    pub back: [f32; 4],
    /// Bend (px), scrim, extra rim light, veil.
    pub params: [f32; 4],
    /// Frost offset (px), body tint, opacity, -.
    pub more: [f32; 4],
}

/// Round 125: a live glass instance.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct LensInst {
    pos: [f32; 2],
    size: [f32; 2],
    back: [f32; 4],
    params: [f32; 4],
    more: [f32; 4],
}

/// The image instance in the vertex buffer (20 bytes).
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct ImgInst {
    pos: [f32; 2],
    size: [f32; 2],
    radius: f32,
}

/// A LAYER in Z order: first its quad runs, then its
/// images, then its text runs. The house draws layer boundaries
/// where a citizen lies OVER earlier text (veil,
/// floating cards) — otherwise text would rise through the ceiling
/// (dig, camp 3).
#[derive(Clone, Debug)]
pub struct GpuLayer {
    /// Range in `runs`.
    pub runs: std::ops::Range<u32>,
    /// Range in `images`.
    pub images: std::ops::Range<u32>,
    /// Range in `texts`.
    pub texts: std::ops::Range<u32>,
}

/// The key of the shape cache: text, size, weight, family.
type TextKey = (String, u32, u16, String);

/// The GPU device on a live Wayland surface.
/// At most this many bytes of image textures stay around (what this frame shows always stays).
const TEXTURE_BUDGET: u64 = 160 * 1024 * 1024;

/// Round 94: a surface of its own on a shared `Gpu` (see `Gpu::add_target`).
pub struct Target {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

pub struct Gpu {
    surface: wgpu::Surface<'static>,
    /// Round 94: kept to make further surfaces (`Target`) on this device.
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    quad_pipeline: wgpu::RenderPipeline,
    /// R207: the CORNER MASK — the same quad SDF, but as a
    /// multiplier on the finished frame (destination × source alpha): what
    /// lies outside the rounded rectangle becomes transparent. Foreign
    /// compositors round nothing on their own.
    mask_pipeline: wgpu::RenderPipeline,
    mask_buf: wgpu::Buffer,
    /// Corner radius in PIXELS (0 = no mask).
    pub corner_radius: f32,
    globals_buf: wgpu::Buffer,
    globals_bind: wgpu::BindGroup,
    /// The instance buffer and its capacity (in instances).
    quad_buf: wgpu::Buffer,
    quad_cap: usize,
    /// Stage 3: glyph atlas and viewport (cryoglyph).
    text_atlas: cryoglyph::TextAtlas,
    text_viewport: cryoglyph::Viewport,
    /// One renderer per LAYER — each holds its prepared
    /// glyphs; grows with the deepest scene.
    text_renderers: Vec<cryoglyph::TextRenderer>,
    /// Shaped cosmic buffers per (text, size, weight, family) —
    /// with a frame stamp for the sweep.
    text_bufs: std::collections::HashMap<TextKey, (cosmic_text::Buffer, u64)>,
    frame_n: u64,
    /// Round 54 (beat probe): the last `render_frame` split in ms —
    /// [acquire the surface texture, upload images, build + submit, present].
    pub phases_ms: [f32; 4],
    /// …and «build» split: [quads/images bookkeeping, text, pass, submit].
    pub build_ms: [f32; 4],
    upload_us: u128,
    /// Stage 4: image pipeline, sampler, texture cache, instances.
    image_pipeline: wgpu::RenderPipeline,
    /// The same quad with the YUV fragment shader (R210 video frames).
    yuv_pipeline: wgpu::RenderPipeline,
    nv12_pipeline: wgpu::RenderPipeline,
    image_layout: wgpu::BindGroupLayout,
    image_sampler: wgpu::Sampler,
    textures: std::collections::HashMap<(String, u32, u32), (wgpu::Texture, wgpu::BindGroup, u64)>,
    /// Stream slot -> the key its texture holds now (R210).
    streams: std::collections::HashMap<String, (String, u32, u32)>,
    img_buf: wgpu::Buffer,
    img_cap: usize,
    /// Round 125: live glass (`LENS_SHADER`).
    lens_pipeline: wgpu::RenderPipeline,
    lens_buf: wgpu::Buffer,
    lens_cap: usize,
    /// Der Matrix-Regen (Overkill 2): Pipeline + Uniform.
    rain_pipeline: wgpu::RenderPipeline,
    rain_buf: wgpu::Buffer,
    rain_bind: wgpu::BindGroup,
    /// Name of the adapter + backend — for the log and the dig.
    pub adapter_info: String,
}

/// Initial size of the instance buffer — the dig's number.
const INITIAL_QUADS: usize = 2_000;

/// PREMULTIPLY straight RGBA rows — all textures
/// of the GPU painting lie premultiplied, the shader never
/// multiplies (stage 4). Decoded images come straight, curves from
/// tiny-skia already multiplied in.
/// Upload RGBA rows into an existing texture (premultiplied on the way
/// when they are not yet).
fn write_rgba(queue: &wgpu::Queue, texture: &wgpu::Texture, rgba: &[u8], premultiplied: bool, tw: u32, th: u32, bpp: u32) {
    let data: std::borrow::Cow<[u8]> = if premultiplied {
        std::borrow::Cow::Borrowed(rgba)
    } else {
        let mut v = rgba.to_vec();
        premultiply(&mut v);
        std::borrow::Cow::Owned(v)
    };
    queue.write_texture(
        wgpu::TexelCopyTextureInfo { texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        &data[..(tw * th * bpp) as usize],
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(tw * bpp), rows_per_image: Some(th) },
        wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
    );
}

fn premultiply(v: &mut [u8]) {
    for px in v.chunks_exact_mut(4) {
        let a = px[3] as u32;
        px[0] = (px[0] as u32 * a / 255) as u8;
        px[1] = (px[1] as u32 * a / 255) as u8;
        px[2] = (px[2] as u32 * a / 255) as u8;
    }
}

#[cfg(test)]
mod guards {
    use super::premultiply;

    /// The premultiply guard (stage 4): fully opaque stays
    /// untouched, fully transparent becomes black, half computes
    /// half — and alpha ALWAYS stays.
    #[test]
    fn premultiply_keeps_alpha_and_scales_color() {
        let mut v = vec![
            200, 100, 50, 255, // opaque: unchanged
            200, 100, 50, 0, // transparent: colour drops to 0
            200, 100, 50, 128, // half: colour halved (rounded)
        ];
        premultiply(&mut v);
        assert_eq!(&v[0..4], &[200, 100, 50, 255]);
        assert_eq!(&v[4..8], &[0, 0, 0, 0]);
        assert_eq!(v[7], 0, "Alpha bleibt");
        assert_eq!(&v[8..12], &[100, 50, 25, 128]);
    }
}

fn quad_buffer(device: &wgpu::Device, cap: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("matrixkit-gpu quads"),
        size: (cap * std::mem::size_of::<GpuQuad>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

impl Gpu {
    /// Builds instance → surface → adapter → device → quad pipeline on
    /// the house's RAW Wayland pointers.
    ///
    /// # Safety
    ///
    /// `display` and `surface` must be valid, live pointers to
    /// `wl_display` and `wl_surface` and outlive the `Gpu`.
    pub unsafe fn new(
        display: *mut std::ffi::c_void,
        surface: *mut std::ffi::c_void,
        w: u32,
        h: u32,
    ) -> Result<Gpu, String> {
        let display = std::ptr::NonNull::new(display).ok_or("wl_display ist null")?;
        let surface = std::ptr::NonNull::new(surface).ok_or("wl_surface ist null")?;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
                    display,
                )),
                raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(
                    surface,
                )),
            })
        }
        .map_err(|e| format!("Fläche: {e}"))?;

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .map_err(|e| format!("Adapter: {e}"))?;

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("matrixkit-gpu"),
            // Round 97: SMALL memory blocks. wgpu's default («Performance»)
            // starts at 128 MB and grows to 512 MB per block: every Matrix app
            // held 380–450 MB of GPU memory (= RAM on the Duo); six of them plus
            // a rotation (all swapchains anew) filled the 5.5 GB and froze the
            // Duo (04.11.2026). MemoryUsage: 8 MB, growing to 64 MB.
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }))
        .map_err(|e| format!("Gerät: {e}"))?;

        let mut config = surface
            .get_default_config(&adapter, w.max(1), h.max(1))
            .ok_or("Fläche und Adapter passen nicht zusammen")?;
        // As in the reference dig: prefer a non-sRGB format (the
        // kit's colours are already in display space) and alpha
        // premultiplied, if the surface supports it.
        let caps = surface.get_capabilities(&adapter);
        if let Some(f) = caps.formats.iter().find(|f| !f.is_srgb()) {
            config.format = *f;
        }
        if caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            config.alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
        }
        // R210: MATRIX_PRESENT=fifo|mailbox|immediate picks the present
        // mode (default FIFO). Measured on the duo 01.11.2026: Mailbox
        // showed the same 18 frames/s as FIFO — the limit is the
        // compositor's frame callbacks, not the swapchain.
        let wanted = match std::env::var("MATRIX_PRESENT").as_deref() {
            Ok("mailbox") => wgpu::PresentMode::Mailbox,
            Ok("immediate") => wgpu::PresentMode::Immediate,
            _ => wgpu::PresentMode::Fifo,
        };
        if caps.present_modes.contains(&wanted) {
            config.present_mode = wanted;
        }
        // Round 54: MATRIX_FRAME_LATENCY=N — frames in flight (swapchain
        // depth, default 2). Probe for Tube: queue.submit blocked 10–14 ms
        // per video frame on the duo (Turnip/KGSL) — a third image may
        // already be free when the next frame is submitted.
        if let Some(n) = std::env::var("MATRIX_FRAME_LATENCY").ok().and_then(|v| v.parse::<u32>().ok()) {
            config.desired_maximum_frame_latency = n.clamp(1, 4);
        }
        eprintln!("[gpu] Bilder in der Schwebe: {}", config.desired_maximum_frame_latency);
        eprintln!("[gpu] Present-Modus {:?} (möglich: {:?})", config.present_mode, caps.present_modes);
        surface.configure(&device, &config);

        // ---- The quad pipeline (stage 2) --------------------------
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("matrixkit-gpu quad"),
            source: wgpu::ShaderSource::Wgsl(QUAD_SHADER.into()),
        });
        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("matrixkit-gpu globals"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("matrixkit-gpu globals"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let globals_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("matrixkit-gpu globals"),
            layout: &globals_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buf.as_entire_binding(),
            }],
        });
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("matrixkit-gpu quad"),
                bind_group_layouts: &[&globals_layout],
                push_constant_ranges: &[],
            });
        let quad_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("matrixkit-gpu quad"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<GpuQuad>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x4, // color
                            1 => Float32x2, // pos
                            2 => Float32x2, // size
                            3 => Float32,   // radius
                            4 => Float32,   // shadow_blur
                            5 => Float32x2, // shadow_offset
                            6 => Float32x4, // shadow_color
                        ],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        // Premultiplied over: source × 1, destination × (1−a).
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        // R207: the corner mask — destination × source alpha.
        let mask_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("matrixkit-gpu mask"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<GpuQuad>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x4, // color
                            1 => Float32x2, // pos
                            2 => Float32x2, // size
                            3 => Float32,   // radius
                            4 => Float32,   // shadow_blur
                            5 => Float32x2, // shadow_offset
                            6 => Float32x4, // shadow_color
                        ],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        // Premultiplied over: source × 1, destination × (1−a).
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::Zero,
                                dst_factor: wgpu::BlendFactor::SrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::Zero,
                                dst_factor: wgpu::BlendFactor::SrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        let mask_buf = quad_buffer(&device, 1);
        let quad_buf = quad_buffer(&device, INITIAL_QUADS);

        // ---- The glyph atlas (stage 3) --------------------------
        let text_cache = cryoglyph::Cache::new(&device);
        let text_atlas =
            cryoglyph::TextAtlas::new(&device, &queue, &text_cache, config.format);
        let text_viewport = cryoglyph::Viewport::new(&device, &text_cache);

        // ---- The image pipeline (stage 4) --------------------------
        let img_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("matrixkit-gpu image"),
            source: wgpu::ShaderSource::Wgsl(IMAGE_SHADER.into()),
        });
        let image_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("matrixkit-gpu image"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let image_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("matrixkit-gpu image"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let img_pipe_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("matrixkit-gpu image"),
                bind_group_layouts: &[&globals_layout, &image_layout],
                push_constant_ranges: &[],
            });
        let image_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("matrixkit-gpu image"),
                layout: Some(&img_pipe_layout),
                vertex: wgpu::VertexState {
                    module: &img_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<ImgInst>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x2, // pos
                            1 => Float32x2, // size
                            2 => Float32,   // radius
                        ],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &img_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        let yuv_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("matrixkit-gpu image"),
                layout: Some(&img_pipe_layout),
                vertex: wgpu::VertexState {
                    module: &img_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<ImgInst>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x2, // pos
                            1 => Float32x2, // size
                            2 => Float32,   // radius
                        ],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &img_shader,
                    entry_point: Some("fs_yuv"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        let nv12_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("matrixkit-gpu image"),
                layout: Some(&img_pipe_layout),
                vertex: wgpu::VertexState {
                    module: &img_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<ImgInst>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float32x2, // pos
                            1 => Float32x2, // size
                            2 => Float32,   // radius
                        ],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &img_shader,
                    entry_point: Some("fs_nv12"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        let img_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("matrixkit-gpu images"),
            size: (64 * std::mem::size_of::<ImgInst>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Round 125: live glass — the lens map and the backdrop, two textures.
        let lens_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("matrixkit-gpu lens"),
            source: wgpu::ShaderSource::Wgsl(LENS_SHADER.into()),
        });
        let lens_pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("matrixkit-gpu lens"),
            bind_group_layouts: &[&globals_layout, &image_layout, &image_layout],
            push_constant_ranges: &[],
        });
        let lens_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("matrixkit-gpu lens"),
            layout: Some(&lens_pipe_layout),
            vertex: wgpu::VertexState {
                module: &lens_shader,
                entry_point: Some("vs_lens"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<LensInst>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2, // pos
                        1 => Float32x2, // size
                        2 => Float32x4, // back
                        3 => Float32x4, // params
                        4 => Float32x4, // more
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &lens_shader,
                entry_point: Some("fs_lens"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let lens_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("matrixkit-gpu lenses"),
            size: (16 * std::mem::size_of::<LensInst>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ---- The Matrix rain (overkill 2) ------------------------
        let rain_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("matrixkit-gpu rain"),
            source: wgpu::ShaderSource::Wgsl(RAIN_SHADER.into()),
        });
        let rain_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("matrixkit-gpu rain"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let rain_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("matrixkit-gpu rain"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let rain_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("matrixkit-gpu rain"),
            layout: &rain_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: rain_buf.as_entire_binding(),
            }],
        });
        let rain_pipe_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("matrixkit-gpu rain"),
                bind_group_layouts: &[&rain_layout],
                push_constant_ranges: &[],
            });
        let rain_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("matrixkit-gpu rain"),
                layout: Some(&rain_pipe_layout),
                vertex: wgpu::VertexState {
                    module: &rain_shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &rain_shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let info = adapter.get_info();
        let adapter_info = format!("{} ({:?})", info.name, info.backend);

        Ok(Gpu {
            surface,
            instance,
            adapter,
            device,
            queue,
            config,
            quad_pipeline,
            mask_pipeline,
            mask_buf,
            corner_radius: 0.0,
            globals_buf,
            globals_bind,
            quad_buf,
            quad_cap: INITIAL_QUADS,
            text_atlas,
            text_viewport,
            text_renderers: Vec::new(),
            text_bufs: std::collections::HashMap::new(),
            frame_n: 0,
            phases_ms: [0.0; 4],
            build_ms: [0.0; 4],
            upload_us: 0,
            image_pipeline,
            yuv_pipeline,
            nv12_pipeline,
            image_layout,
            image_sampler,
            textures: std::collections::HashMap::new(),
            streams: std::collections::HashMap::new(),
            img_buf,
            img_cap: 64,
            lens_pipeline,
            lens_buf,
            lens_cap: 16,
            rain_pipeline,
            rain_buf,
            rain_bind,
            adapter_info,
        })
    }

    /// New surface size in PIXELS.
    /// Round 94: a further surface painted by THIS device (pipelines, glyph
    /// atlas and picture cache shared) — the window house's fleet panes
    /// (cards) each get one instead of a device of their own. Mailbox when
    /// offered: a hidden pane must never block the app in a FIFO wait.
    ///
    /// # Safety
    /// `display` and `surface` must be live Wayland pointers for as long as
    /// the target lives; drop the target BEFORE destroying the wl_surface.
    pub unsafe fn add_target(&self, display: *mut std::ffi::c_void, surface: *mut std::ffi::c_void, w: u32, h: u32) -> Result<Target, String> {
        let display = std::ptr::NonNull::new(display).ok_or("wl_display ist null")?;
        let surface = std::ptr::NonNull::new(surface).ok_or("wl_surface ist null")?;
        let surface = unsafe {
            self.instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: RawDisplayHandle::Wayland(WaylandDisplayHandle::new(display)),
                raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(surface)),
            })
        }
        .map_err(|e| format!("Fläche: {e}"))?;
        let caps = surface.get_capabilities(&self.adapter);
        if !caps.formats.contains(&self.config.format) {
            return Err(String::from("das Bildformat passt nicht zur Fläche"));
        }
        let mut config = self.config.clone();
        config.width = w.max(1);
        config.height = h.max(1);
        if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            config.present_mode = wgpu::PresentMode::Mailbox;
        }
        if !caps.alpha_modes.contains(&config.alpha_mode) {
            config.alpha_mode = caps.alpha_modes[0];
        }
        surface.configure(&self.device, &config);
        Ok(Target { surface, config })
    }

    /// Round 94: paint into `target` — the device switches to it for the
    /// call (`f` sees an ordinary Gpu: resize, render_frame …).
    pub fn with_target<R>(&mut self, target: &mut Target, f: impl FnOnce(&mut Gpu) -> R) -> R {
        std::mem::swap(&mut self.surface, &mut target.surface);
        std::mem::swap(&mut self.config, &mut target.config);
        let out = f(self);
        std::mem::swap(&mut self.surface, &mut target.surface);
        std::mem::swap(&mut self.config, &mut target.config);
        out
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 || (self.config.width, self.config.height) == (w, h) {
            return;
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
    }

    fn frame(&mut self) -> Result<wgpu::SurfaceTexture, String> {
        match self.surface.get_current_texture() {
            Ok(f) => Ok(f),
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                // Rebind the surface and ask once more.
                self.surface.configure(&self.device, &self.config);
                self.surface
                    .get_current_texture()
                    .map_err(|e| format!("Bild: {e}"))
            }
            Err(e) => Err(format!("Bild: {e}")),
        }
    }

    /// Stage 1: paint a frame in a clear colour and present it.
    /// Returns: the paint time in microseconds.
    pub fn clear(&mut self, r: f64, g: f64, b: f64, a: f64) -> Result<u128, String> {
        self.render_quads([r, g, b, a], &[], &[])
    }

    /// Stage 2 (without text): everything one layer.
    pub fn render_quads(
        &mut self,
        clear: [f64; 4],
        quads: &[GpuQuad],
        runs: &[QuadRun],
    ) -> Result<u128, String> {
        let layer =
            GpuLayer { runs: 0..runs.len() as u32, images: 0..0, texts: 0..0 };
        let mut dummy_fonts = None;
        self.render_frame(
            &mut dummy_fonts,
            clear,
            quads,
            runs,
            &[],
            None,
            None,
            &[],
            std::slice::from_ref(&layer),
            1.0,
        )
    }

    /// Stages 2+3: paint the scene in LAYERS — per layer first
    /// the quad runs (scissor + instanced draw), then its
    /// text runs from the glyph atlas. `fonts` carries the text machine's
    /// two hands; None means: no text in the frame.
    /// Returns: the paint time in microseconds (up to presenting).
    pub fn render_frame(
        &mut self,
        fonts: &mut Option<(&mut cosmic_text::FontSystem, &mut cosmic_text::SwashCache)>,
        clear: [f64; 4],
        quads: &[GpuQuad],
        runs: &[QuadRun],
        images: &[GpuImage<'_>],
        ground: Option<&GpuImage<'_>>,
        rain: Option<[f32; 5]>,
        texts: &[GpuText<'_>],
        layers: &[GpuLayer],
        scale: f32,
    ) -> Result<u128, String> {
        let clear = wgpu::Color { r: clear[0], g: clear[1], b: clear[2], a: clear[3] };
        let clock = std::time::Instant::now();
        if quads.len() > self.quad_cap {
            self.quad_cap = quads.len().next_power_of_two();
            self.quad_buf = quad_buffer(&self.device, self.quad_cap);
        }
        if !quads.is_empty() {
            // Safety: GpuQuad is repr(C) and purely f32 —
            // the byte view is lossless and without padding.
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    quads.as_ptr() as *const u8,
                    std::mem::size_of_val(quads),
                )
            };
            self.queue.write_buffer(&self.quad_buf, 0, bytes);
        }
        // Ortho: pixels → NDC (y grows downwards).
        let (w, h) = (self.config.width as f32, self.config.height as f32);
        let transform: [f32; 16] = [
            2.0 / w, 0.0, 0.0, 0.0, //
            0.0, -2.0 / h, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            -1.0, 1.0, 0.0, 1.0,
        ];
        let tbytes = unsafe {
            std::slice::from_raw_parts(transform.as_ptr() as *const u8, 64)
        };
        self.queue.write_buffer(&self.globals_buf, 0, tbytes);

        let acquire_at = std::time::Instant::now();
        let frame = self.frame()?;
        let acquired_at = std::time::Instant::now();
        self.upload_us = 0;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("matrixkit-gpu frame"),
            });

        self.frame_n += 1;
        // ---- PREPARE images (stage 4): upload missing
        // textures (straight pixels are multiplied in along the way —
        // ALL textures lie premultiplied), refresh stamps,
        // write instances. ----
        if !images.is_empty() || ground.is_some() {
            for im in images.iter().chain(ground) {
                let stamp = self.frame_n;
                if let Some((_, _, st)) = self.textures.get_mut(&im.key) {
                    *st = stamp;
                    continue;
                }
                let (tw, th) = (im.key.1.max(1), im.key.2.max(1));
                // A YUV frame (R210): one R8 texture, the planes stacked.
                let (tw, th, bpp) = if im.yuv { (tw, th * 3 / 2, 1) } else { (tw, th, 4) };
                if im.rgba.len() < (tw * th * bpp) as usize {
                    continue;
                }
                // A stream slot of the same size and kind: write in place —
                // no new texture, no new bind group.
                if let Some(slot) = &im.stream {
                    let old = self.streams.get(slot).cloned();
                    if let Some(old) = old.filter(|o| (o.1, o.2) == (im.key.1, im.key.2)) {
                        if let Some((texture, bind, _)) = self.textures.remove(&old) {
                            let t = std::time::Instant::now();
                            write_rgba(&self.queue, &texture, im.rgba, im.premultiplied || im.yuv, tw, th, bpp);
                            self.upload_us += t.elapsed().as_micros();
                            self.textures.insert(im.key.clone(), (texture, bind, stamp));
                            self.streams.insert(slot.clone(), im.key.clone());
                            continue;
                        }
                    }
                    self.streams.insert(slot.clone(), im.key.clone());
                }
                let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("matrixkit-gpu image"),
                    size: wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: if im.yuv { wgpu::TextureFormat::R8Unorm } else { wgpu::TextureFormat::Rgba8Unorm },
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let t = std::time::Instant::now();
                write_rgba(&self.queue, &texture, im.rgba, im.premultiplied || im.yuv, tw, th, bpp);
                self.upload_us += t.elapsed().as_micros();
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("matrixkit-gpu image"),
                    layout: &self.image_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.image_sampler),
                        },
                    ],
                });
                self.textures.insert(im.key.clone(), (texture, bind, stamp));
            }
            // Sweep: textures nobody has shown for a long time drop — and
            // the store stays under a BYTE budget. Without the
            // budget, an image per frame (computed view,
            // 20 MB at full screen 2×) collected 256 textures = 5 GB; on an
            // Intel chip that is main memory — the Surface froze
            // five times (14.10.2026). Keep the youngest first.
            let n = self.frame_n;
            self.textures.retain(|_, (_, _, st)| n - *st < 256);
            let bytes = |k: &(String, u32, u32)| (k.1 as u64) * (k.2 as u64) * 4;
            let mut total: u64 = self.textures.keys().map(bytes).sum();
            if total > TEXTURE_BUDGET {
                let mut stale: Vec<((String, u32, u32), u64)> = self.textures.iter().filter(|(_, (_, _, st))| *st != n).map(|(k, (_, _, st))| (k.clone(), *st)).collect();
                stale.sort_by_key(|(_, st)| *st);
                for (k, _) in stale {
                    if total <= TEXTURE_BUDGET {
                        break;
                    }
                    total -= bytes(&k);
                    self.textures.remove(&k);
                }
            }
            // Write instances.
            if images.len() + 1 > self.img_cap {
                self.img_cap = (images.len() + 1).next_power_of_two();
                self.img_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("matrixkit-gpu images"),
                    size: (self.img_cap * std::mem::size_of::<ImgInst>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }
            let insts: Vec<ImgInst> = images
                .iter()
                .chain(ground)
                .map(|im| ImgInst {
                    pos: [im.x, im.y],
                    size: [im.w, im.h],
                    radius: im.radius,
                })
                .collect();
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    insts.as_ptr() as *const u8,
                    std::mem::size_of_val(insts.as_slice()),
                )
            };
            self.queue.write_buffer(&self.img_buf, 0, bytes);
        }
        // Round 125: the live glass instances (image index -> lens slot).
        let lens_slots: Vec<Option<u32>> = {
            let mut slots = Vec::with_capacity(images.len());
            let mut insts: Vec<LensInst> = Vec::new();
            for im in images.iter() {
                match &im.lens {
                    Some(l) => {
                        slots.push(Some(insts.len() as u32));
                        insts.push(LensInst { pos: [im.x, im.y], size: [im.w, im.h], back: l.back, params: l.params, more: l.more });
                    }
                    None => slots.push(None),
                }
            }
            if !insts.is_empty() {
                if insts.len() > self.lens_cap {
                    self.lens_cap = insts.len().next_power_of_two();
                    self.lens_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("matrixkit-gpu lenses"),
                        size: (self.lens_cap * std::mem::size_of::<LensInst>()) as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                }
                let bytes = unsafe { std::slice::from_raw_parts(insts.as_ptr() as *const u8, std::mem::size_of_val(insts.as_slice())) };
                self.queue.write_buffer(&self.lens_buf, 0, bytes);
            }
            slots
        };

        // ---- PREPARE text (before the pass) -------------------
        let text_at = std::time::Instant::now();
        if let Some((fs, swash)) = fonts.as_mut() {
            self.text_viewport.update(
                &self.queue,
                cryoglyph::Resolution {
                    width: self.config.width,
                    height: self.config.height,
                },
            );
            // Shape what is missing; refresh stamps.
            for t in texts {
                let key: TextKey =
                    (t.s.to_string(), t.px.to_bits(), t.weight, t.family.to_string());
                let stamp = self.frame_n;
                self.text_bufs
                    .entry(key)
                    .and_modify(|(_, st)| *st = stamp)
                    .or_insert_with(|| {
                        let mut buf = cosmic_text::Buffer::new(
                            fs,
                            cosmic_text::Metrics::new(t.px, t.line),
                        );
                        buf.set_size(fs, None, None);
                        buf.set_text(
                            fs,
                            // The language (06.11.2026), as the engine measured it.
                            &matrixkit_theme::language::shown(t.s),
                            &cosmic_text::Attrs::new()
                                .family(cosmic_text::Family::Name(t.family))
                                .weight(cosmic_text::Weight(t.weight)),
                            cosmic_text::Shaping::Advanced,
                            None,
                        );
                        buf.shape_until_scroll(fs, false);
                        (buf, stamp)
                    });
            }
            // Sweep before the cache proliferates.
            if self.text_bufs.len() > 4096 {
                let n = self.frame_n;
                self.text_bufs.retain(|_, (_, st)| n - *st < 64);
            }
            while self.text_renderers.len() < layers.len() {
                self.text_renderers.push(cryoglyph::TextRenderer::new(
                    &mut self.text_atlas,
                    &self.device,
                    wgpu::MultisampleState::default(),
                    None,
                ));
            }
            for (li, layer) in layers.iter().enumerate() {
                let slice = &texts[layer.texts.start as usize..layer.texts.end as usize];
                let areas = slice.iter().map(|t| {
                    let key: TextKey =
                        (t.s.to_string(), t.px.to_bits(), t.weight, t.family.to_string());
                    let (buf, _) = &self.text_bufs[&key];
                    cryoglyph::TextArea {
                        buffer: buf,
                        left: t.x,
                        top: t.y,
                        scale,
                        bounds: match t.bounds {
                            Some((x0, y0, x1, y1)) => cryoglyph::TextBounds {
                                left: x0,
                                top: y0,
                                right: x1,
                                bottom: y1,
                            },
                            None => cryoglyph::TextBounds::default(),
                        },
                        default_color: cosmic_text::Color::rgba(
                            (t.color[0] * 255.0) as u8,
                            (t.color[1] * 255.0) as u8,
                            (t.color[2] * 255.0) as u8,
                            (t.color[3] * 255.0) as u8,
                        ),
                    }
                });
                self.text_renderers[li]
                    .prepare(
                        &self.device,
                        &self.queue,
                        &mut encoder,
                        fs,
                        &mut self.text_atlas,
                        &self.text_viewport,
                        areas,
                        swash,
                    )
                    .map_err(|e| format!("Schrift: {e:?}"))?;
            }
        }
        let pass_at = std::time::Instant::now();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let (pw, ph) = (self.config.width, self.config.height);
            // The MATRIX RAIN (overkill 2) falls OVER the ground,
            // under all layers.
            if let Some(r) = rain {
                let u: [f32; 8] = [
                    r[0],
                    r[1],
                    pw as f32,
                    ph as f32,
                    r[2],
                    r[3],
                    r[4],
                    14.0 * scale,
                ];
                let ub = unsafe {
                    std::slice::from_raw_parts(u.as_ptr() as *const u8, 32)
                };
                self.queue.write_buffer(&self.rain_buf, 0, ub);
            }
            // The GROUND first (stage 5): client.base() as a texture,
            // under all layers — exact parity with the CPU path,
            // without a new client contract.
            if let Some(g) = ground {
                if let Some((_, bind, _)) = self.textures.get(&g.key) {
                    pass.set_pipeline(&self.image_pipeline);
                    pass.set_bind_group(0, &self.globals_bind, &[]);
                    pass.set_vertex_buffer(0, self.img_buf.slice(..));
                    pass.set_scissor_rect(0, 0, pw, ph);
                    pass.set_bind_group(1, bind, &[]);
                    let gi = images.len() as u32;
                    pass.draw(0..6, gi..gi + 1);
                }
            }
            if rain.is_some() {
                pass.set_pipeline(&self.rain_pipeline);
                pass.set_bind_group(0, &self.rain_bind, &[]);
                pass.set_scissor_rect(0, 0, pw, ph);
                pass.draw(0..3, 0..1);
            }
            for (li, layer) in layers.iter().enumerate() {
                if !quads.is_empty() && !layer.runs.is_empty() {
                    pass.set_pipeline(&self.quad_pipeline);
                    pass.set_bind_group(0, &self.globals_bind, &[]);
                    pass.set_vertex_buffer(0, self.quad_buf.slice(..));
                    for run in &runs[layer.runs.start as usize..layer.runs.end as usize] {
                        // Scissor clamped to the surface — wgpu
                        // punishes overhang with a panic abort.
                        let (x, y, cw, ch) = match run.clip {
                            Some((x, y, cw, ch)) => {
                                let x = x.min(pw);
                                let y = y.min(ph);
                                (x, y, cw.min(pw - x), ch.min(ph - y))
                            }
                            None => (0, 0, pw, ph),
                        };
                        if cw == 0 || ch == 0 || run.range.is_empty() {
                            continue;
                        }
                        pass.set_scissor_rect(x, y, cw, ch);
                        pass.draw(0..6, run.range.clone());
                    }
                }
                if !layer.images.is_empty() {
                    pass.set_pipeline(&self.image_pipeline);
                    pass.set_bind_group(0, &self.globals_bind, &[]);
                    pass.set_vertex_buffer(0, self.img_buf.slice(..));
                    for ii in layer.images.clone() {
                        let im = &images[ii as usize];
                        let Some((_, bind, _)) = self.textures.get(&im.key) else {
                            continue;
                        };
                        let (x, y, cw, ch) = match im.clip {
                            Some((x, y, cw, ch)) => {
                                let x = x.min(pw);
                                let y = y.min(ph);
                                (x, y, cw.min(pw - x), ch.min(ph - y))
                            }
                            None => (0, 0, pw, ph),
                        };
                        if cw == 0 || ch == 0 {
                            continue;
                        }
                        pass.set_scissor_rect(x, y, cw, ch);
                        // Round 125: live glass — the map and the backdrop.
                        if let (Some(l), Some(Some(slot))) = (&im.lens, lens_slots.get(ii as usize)) {
                            if let Some((_, back_bind, _)) = self.textures.get(&l.backdrop) {
                                pass.set_pipeline(&self.lens_pipeline);
                                pass.set_bind_group(0, &self.globals_bind, &[]);
                                pass.set_bind_group(1, bind, &[]);
                                pass.set_bind_group(2, back_bind, &[]);
                                pass.set_vertex_buffer(0, self.lens_buf.slice(..));
                                pass.draw(0..6, *slot..*slot + 1);
                                pass.set_pipeline(&self.image_pipeline);
                                pass.set_vertex_buffer(0, self.img_buf.slice(..));
                            }
                            continue;
                        }
                        if im.yuv {
                            pass.set_pipeline(if im.nv12 { &self.nv12_pipeline } else { &self.yuv_pipeline });
                        }
                        pass.set_bind_group(1, bind, &[]);
                        pass.draw(0..6, ii..ii + 1);
                        if im.yuv {
                            pass.set_pipeline(&self.image_pipeline);
                        }
                    }
                }
                if fonts.is_some() && !layer.texts.is_empty() {
                    // The text clips via TextBounds — the
                    // quad scissor must not choke it.
                    pass.set_scissor_rect(0, 0, pw, ph);
                    self.text_renderers[li]
                        .render(&self.text_atlas, &self.text_viewport, &mut pass)
                        .map_err(|e| format!("Schrift-Zug: {e:?}"))?;
                }
            }
            // R207: last, the CORNER MASK over the whole frame — what
            // lies outside the rounded rectangle becomes transparent
            // (blend destination × source alpha). Foreign compositors do not round
            // windows; niri rounds anyway, doing it twice does no harm.
            if self.corner_radius > 0.5 {
                let mask = GpuQuad {
                    color: [1.0, 1.0, 1.0, 1.0],
                    pos: [0.0, 0.0],
                    size: [pw as f32, ph as f32],
                    radius: self.corner_radius,
                    shadow_blur: 0.0,
                    shadow_offset: [0.0, 0.0],
                    shadow_color: [0.0, 0.0, 0.0, 0.0],
                };
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        (&mask as *const GpuQuad) as *const u8,
                        std::mem::size_of::<GpuQuad>(),
                    )
                };
                self.queue.write_buffer(&self.mask_buf, 0, bytes);
                pass.set_pipeline(&self.mask_pipeline);
                pass.set_bind_group(0, &self.globals_bind, &[]);
                pass.set_vertex_buffer(0, self.mask_buf.slice(..));
                pass.set_scissor_rect(0, 0, pw, ph);
                pass.draw(0..6, 0..1);
            }
        }
        let submit_at = std::time::Instant::now();
        self.queue.submit(Some(encoder.finish()));
        let submitted_at = std::time::Instant::now();
        frame.present();
        let up = self.upload_us as f32 / 1000.0;
        let ms = |a: std::time::Instant, b: std::time::Instant| b.duration_since(a).as_secs_f32() * 1000.0;
        self.phases_ms = [ms(acquire_at, acquired_at), up, (ms(acquired_at, submitted_at) - up).max(0.0), submitted_at.elapsed().as_secs_f32() * 1000.0];
        // The «build» part split further (round 54): before the text,
        // text shaping + prepare, the pass, the submit.
        self.build_ms = [(ms(acquired_at, text_at) - up).max(0.0), ms(text_at, pass_at), ms(pass_at, submit_at), ms(submit_at, submitted_at)];
        // The sweep after every frame — the reference dig does it
        // the same way (iced lib.rs: atlas.trim() at the end of the frame);
        // without it the glyph atlas proliferates with every text run
        // ever seen (camp 13).
        self.text_atlas.trim();
        Ok(clock.elapsed().as_micros())
    }
}
