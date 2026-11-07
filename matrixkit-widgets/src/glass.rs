//! Liquid glass (R106) — the Matrix translation of the macOS 27 glass
//! machinery, measured on the living reference (see
//! LIQUID-GLASS-EXTRACT.md): shapes live as signed distance fields,
//! neighbors merge via smooth minimum (the container `spacing` is the
//! union distance), and the backdrop is refracted along the SDF
//! gradient with a chromatic fringe, variable blur and a rim light.
//!
//! Honest limit: a layer surface cannot capture the compositor
//! backdrop, so panels refract the KNOWN wallpaper — the caller passes
//! the wallpaper pixels plus its own rect on screen. True
//! behind-window glass needs compositor support (fork, later).

use iced::widget::shader::{self, Viewport};
use iced::Rectangle;
use matrixkit_theme as mk;

/// IS GLASS ON? (R209 family — bar, dock and Start asked it each): glass
/// is on unless switched off in the settings, and it never fights the
/// accessibility contract (reduced transparency = opaque surfaces).
pub fn wanted() -> bool {
    mk::glass_wanted()
}

/// One glass shape in surface-local logical pixels.
#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

pub const MAX_ELEMENTS: usize = 8;

/// The glass program for one surface. Elements merge liquidly when
/// closer than `spacing` (the GlassEffectContainer semantic).
#[derive(Debug, Clone)]
pub struct Liquid {
    pub elements: Vec<Element>,
    /// Union distance `k` — 0 keeps shapes separate.
    pub spacing: f32,
    /// Palette tone laid over the refracted backdrop.
    pub tint: mk::Rgba,
    /// Tint coverage (panel::coverage counterpart for glass).
    pub coverage: f32,
    /// Wallpaper pixels (RGBA8) and size.
    pub backdrop: std::sync::Arc<(Vec<u8>, u32, u32)>,
    /// This surface's rect on the screen, in the wallpaper's pixel
    /// space — maps surface UV to backdrop UV.
    pub screen_rect: Rectangle,
    /// Monotonic wallpaper stamp — bumps re-upload the texture.
    pub backdrop_stamp: u64,
}

impl<Message> shader::Program<Message> for Liquid {
    type State = ();
    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive(self.clone())
    }
}

#[derive(Debug)]
pub struct Primitive(Liquid);

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    // xy = min, zw = max, in surface UV of the shader quad
    elements: [[f32; 4]; MAX_ELEMENTS],
    radii: [[f32; 4]; 2], // 8 radii packed as two vec4
    tint: [f32; 4],
    // x = element count, y = spacing k, z = coverage, w = scale factor
    params: [f32; 4],
    // backdrop mapping: xy = offset in backdrop UV, zw = scale
    backdrop_map: [f32; 4],
    // xy = quad size in physical px, zw reserved
    size: [f32; 4],
}

pub struct GlassPipeline {
    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
    stamp: u64,
    tex_size: (u32, u32),
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl shader::Pipeline for GlassPipeline {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        // Starts with a 1×1 placeholder backdrop; the real wallpaper
        // arrives in the first prepare() (stamp 0 forces the upload).
        build_pipeline(device, queue, format, &[0, 0, 0, 255], (1, 1), 0)
    }
}

const SHADER: &str = r#"
struct Uniforms {
    elements: array<vec4<f32>, 8>,
    radii: array<vec4<f32>, 2>,
    tint: vec4<f32>,
    params: vec4<f32>,
    backdrop_map: vec4<f32>,
    size: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var backdrop_tex: texture_2d<f32>;
@group(0) @binding(2) var backdrop_smp: sampler;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOut {
    // Fullscreen-quad trick over the widget bounds.
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0), vec2<f32>(-1.0, 1.0), vec2<f32>(3.0, 1.0));
    var out: VertexOut;
    out.position = vec4<f32>(pos[i], 0.0, 1.0);
    out.uv = vec2<f32>(pos[i].x * 0.5 + 0.5, 0.5 - pos[i].y * 0.5);
    return out;
}

fn sd_rounded_rect(p: vec2<f32>, center: vec2<f32>, half: vec2<f32>, r: f32) -> f32 {
    let q = abs(p - center) - half + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// The measured recipe: shapes merge via smooth minimum — the
// GlassEffectContainer `spacing` is `k` (LIQUID-GLASS-EXTRACT.md).
fn smin(a: f32, b: f32, k: f32) -> f32 {
    if (k <= 0.0) { return min(a, b); }
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn scene_sdf(p: vec2<f32>) -> f32 {
    var d = 1e9;
    let n = i32(u.params.x);
    let k = u.params.y;
    for (var i = 0; i < 8; i = i + 1) {
        if (i >= n) { break; }
        let e = u.elements[i];
        let r = u.radii[i / 4][i % 4];
        let center = (e.xy + e.zw) * 0.5;
        let half = (e.zw - e.xy) * 0.5;
        d = smin(d, sd_rounded_rect(p, center, half, r), k);
    }
    return d;
}

fn backdrop(uv: vec2<f32>, lod: f32) -> vec3<f32> {
    let m = clamp(u.backdrop_map.xy + uv * u.backdrop_map.zw,
                  vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    return textureSampleLevel(backdrop_tex, backdrop_smp, m, lod).rgb;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let px = in.uv * u.size.xy;
    let d = scene_sdf(px);
    if (d > 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // SDF gradient = the glass normal in the plane (finite differences).
    let e = vec2<f32>(1.0, 0.0);
    let grad = normalize(vec2<f32>(
        scene_sdf(px + e.xy) - scene_sdf(px - e.xy),
        scene_sdf(px + e.yx) - scene_sdf(px - e.yx)) + vec2<f32>(1e-5, 0.0));

    // Refraction: strongest near the rim (measured look: the lens
    // bends the backdrop at the edges, the center stays calm).
    let depth = clamp(-d, 0.0, 24.0);
    let rim = 1.0 - smoothstep(0.0, 18.0, depth);
    let bend = grad * rim * 14.0;

    // Chromatic fringe: per-channel displacement scales.
    let base_uv = in.uv;
    let uv_r = base_uv + (bend * 1.15) / u.size.xy;
    let uv_g = base_uv + bend / u.size.xy;
    let uv_b = base_uv + (bend * 0.85) / u.size.xy;

    // Variable blur via mip bias: calmer (blurrier) in the middle.
    let lod = mix(1.2, 2.6, 1.0 - rim);
    var color = vec3<f32>(
        backdrop(uv_r, lod).r,
        backdrop(uv_g, lod).g,
        backdrop(uv_b, lod).b);

    // Tint / legibility layer (vibrantColorMatrix counterpart).
    color = mix(color, u.tint.rgb, u.params.z);

    // Rim light: gradient against the top-left light, edge band only.
    let light = normalize(vec2<f32>(-0.35, -1.0));
    let rim_band = smoothstep(2.5, 0.5, depth);
    let spec = max(dot(-grad, light), 0.0);
    color = color + vec3<f32>(1.0, 1.0, 1.0) * (rim_band * pow(spec, 3.0) * 0.35);
    // Soft inner shade at the bottom edge for depth.
    let shade = max(dot(grad, light), 0.0);
    color = color - vec3<f32>(1.0, 1.0, 1.0) * (rim_band * pow(shade, 3.0) * 0.18);

    // 1 px anti-aliased silhouette.
    let alpha = clamp(-d, 0.0, 1.0);
    return vec4<f32>(color * alpha, alpha);
}
"#;

impl shader::Primitive for Primitive {
    type Pipeline = GlassPipeline;

    fn prepare(
        &self,
        pipe: &mut GlassPipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let scale = viewport.scale_factor() as f32;
        let g = &self.0;
        let (ref pixels, tw, th) = *g.backdrop.as_ref();

        // Wallpaper changed (or first frame) -> (re)upload the texture.
        if pipe.stamp != g.backdrop_stamp || pipe.tex_size != (tw, th) {
            let (texture, bind_group) = build_texture(
                device, queue, &pipe.layout, &pipe.uniforms, &pipe.sampler, pixels, (tw, th),
            );
            pipe.texture = texture;
            pipe.bind_group = bind_group;
            pipe.stamp = g.backdrop_stamp;
            pipe.tex_size = (tw, th);
        }

        let mut elements = [[0.0f32; 4]; MAX_ELEMENTS];
        let mut radii = [[0.0f32; 4]; 2];
        for (i, e) in g.elements.iter().take(MAX_ELEMENTS).enumerate() {
            elements[i] = [
                e.x * scale,
                e.y * scale,
                (e.x + e.width) * scale,
                (e.y + e.height) * scale,
            ];
            radii[i / 4][i % 4] = e.radius * scale;
        }
        // Backdrop mapping: this surface's rect inside the wallpaper, UV.
        let map = [
            g.screen_rect.x / tw as f32,
            g.screen_rect.y / th as f32,
            g.screen_rect.width / tw as f32,
            g.screen_rect.height / th as f32,
        ];
        let uniforms = Uniforms {
            elements,
            radii,
            tint: [g.tint.r, g.tint.g, g.tint.b, 1.0],
            params: [
                g.elements.len().min(MAX_ELEMENTS) as f32,
                g.spacing * scale,
                g.coverage,
                scale,
            ],
            backdrop_map: map,
            size: [bounds.width * scale, bounds.height * scale, 0.0, 0.0],
        };
        queue.write_buffer(&pipe.uniforms, 0, bytemuck::bytes_of(&uniforms));
    }

    fn draw(
        &self,
        pipe: &GlassPipeline,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) -> bool {
        // Viewport and scissor are already set to our bounds by iced.
        render_pass.set_pipeline(&pipe.pipeline);
        render_pass.set_bind_group(0, &pipe.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        true
    }
}

fn build_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    uniforms: &wgpu::Buffer,
    sampler: &wgpu::Sampler,
    pixels: &[u8],
    (w, h): (u32, u32),
) -> (wgpu::Texture, wgpu::BindGroup) {
    // R106 fix: a texture's mip count must not exceed 1 + floor(log2(max
    // dim)) — a 1x1 placeholder allows exactly one level (the crash on
    // the live PC was mips=4 on 1x1). Cap at 4 for the frosted lods.
    let w = w.max(1);
    let h = h.max(1);
    let mips = (32 - w.max(h).leading_zeros()).clamp(1, 4);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("matrix-glass-backdrop"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * w),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    // Simple mip chain: downsample on the CPU once (box filter) — the
    // wallpaper changes rarely, quality is plenty for frosted lods.
    let mut prev: Vec<u8> = pixels.to_vec();
    let (mut pw, mut ph) = (w, h);
    for level in 1..mips {
        let nw = (pw / 2).max(1);
        let nh = (ph / 2).max(1);
        let mut next = vec![0u8; (nw * nh * 4) as usize];
        for y in 0..nh {
            for x in 0..nw {
                for c in 0..4usize {
                    let mut sum = 0u32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let sx = (x * 2 + dx).min(pw - 1);
                        let sy = (y * 2 + dy).min(ph - 1);
                        sum += prev[((sy * pw + sx) * 4) as usize + c] as u32;
                    }
                    next[((y * nw + x) * 4) as usize + c] = (sum / 4) as u8;
                }
            }
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &next,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * nw),
                rows_per_image: Some(nh),
            },
            wgpu::Extent3d { width: nw, height: nh, depth_or_array_layers: 1 },
        );
        prev = next;
        pw = nw;
        ph = nh;
    }
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("matrix-glass-bind"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: uniforms.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(sampler) },
        ],
    });
    (texture, bind_group)
}

fn build_pipeline(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    pixels: &[u8],
    size: (u32, u32),
    stamp: u64,
) -> GlassPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("matrix-glass-shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("matrix-glass-uniforms"),
        size: std::mem::size_of::<Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("matrix-glass-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        ..Default::default()
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("matrix-glass-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("matrix-glass-pipeline-layout"),
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("matrix-glass-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    let (texture, bind_group) =
        build_texture(device, queue, &layout, &uniforms, &sampler, pixels, size);
    GlassPipeline { pipeline, uniforms, bind_group, texture, stamp, tex_size: size, layout, sampler }
}

/// Load the current wallpaper as RGBA pixels for the backdrop texture.
/// Follows the light/dark pair like the wallpaper app; None if unset.
pub fn wallpaper_backdrop() -> Option<std::sync::Arc<(Vec<u8>, u32, u32)>> {
    let light = mk::Palette::load().map(|p| p.is_light).unwrap_or(false);
    let path = mk::wallpaper::read(light)?;
    let img = image::open(path.trim()).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    // Quarter resolution is plenty behind frosted glass and keeps the
    // upload cheap.
    let img = image::imageops::resize(
        &img,
        (w / 2).max(1),
        (h / 2).max(1),
        image::imageops::FilterType::Triangle,
    );
    let (w, h) = img.dimensions();
    Some(std::sync::Arc::new((img.into_raw(), w, h)))
}

/// Logical size of the first output (for backdrop mapping).
pub fn screen_size() -> (f32, f32) {
    let fallback = (1920.0, 1080.0);
    let Some(out) = mk::canvas::raw(&["msg", "-j", "outputs"]) else { return fallback };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) else { return fallback };
    let outputs = match &v {
        serde_json::Value::Array(a) => a.clone(),
        serde_json::Value::Object(m) => m.values().cloned().collect(),
        _ => return fallback,
    };
    for o in outputs {
        if let Some(mode) = o.get("logical") {
            let w = mode.get("width").and_then(|x| x.as_f64());
            let h = mode.get("height").and_then(|x| x.as_f64());
            if let (Some(w), Some(h)) = (w, h) {
                return (w as f32, h as f32);
            }
        }
    }
    fallback
}

/// R111 vibrant text (NSLumaReadingView extract): average wallpaper
/// luminance behind an on-screen region. `rect` is in logical pixels;
/// the wallpaper is assumed cover-filled like everywhere in the kit.
pub fn luma_of(
    backdrop: &(Vec<u8>, u32, u32),
    screen: (f32, f32),
    rect: Rectangle,
) -> f32 {
    let (pixels, w, h) = backdrop;
    let (sw, sh) = screen;
    // COVER-fill mapping (Surface finding 2026-07-23): the wallpaper
    // is cover-cropped and centered on screen, NOT stretched — with
    // independent x/y scaling a 16:9 image on the 3:2 Surface put the
    // measuring window into an invisible band and the ink went gray.
    let scale = (*w as f32 / sw.max(1.0)).max(*h as f32 / sh.max(1.0));
    let off_x = (*w as f32 - sw * scale) * 0.5;
    let off_y = (*h as f32 - sh * scale) * 0.5;
    let x0 = ((rect.x * scale + off_x).max(0.0) as u32).min(w.saturating_sub(1));
    let y0 = ((rect.y * scale + off_y).max(0.0) as u32).min(h.saturating_sub(1));
    let x1 = (((rect.x + rect.width) * scale + off_x) as u32).clamp(x0 + 1, *w);
    let y1 = (((rect.y + rect.height) * scale + off_y) as u32).clamp(y0 + 1, *h);
    // Stride sampling: a handful of rows is plenty for an average.
    let step_x = ((x1 - x0) / 32).max(1);
    let step_y = ((y1 - y0) / 8).max(1);
    let (mut sum, mut n) = (0.0f32, 0u32);
    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            let i = ((y * w + x) * 4) as usize;
            if i + 2 < pixels.len() {
                let (r, g, b) = (pixels[i], pixels[i + 1], pixels[i + 2]);
                sum += 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
                n += 1;
            }
            x += step_x;
        }
        y += step_y;
    }
    if n == 0 { 0.5 } else { sum / (n as f32 * 255.0) }
}

/// Convenience: measure a region against the CURRENT wallpaper
/// (decodes it — call on init/wallpaper change, not per frame).
pub fn region_luma(rect: Rectangle) -> Option<f32> {
    let backdrop = wallpaper_backdrop()?;
    Some(luma_of(&backdrop, screen_size(), rect))
}

/// Ink pair (primary, secondary) for text directly on glass: bright
/// backdrops get dark ink, dark ones light — blended through the mid
/// band so a wallpaper change never flips the text harshly.
pub fn vibrant_ink(luma: f32) -> (mk::Rgba, mk::Rgba) {
    let t = ((luma - 0.35) / 0.30).clamp(0.0, 1.0);
    let mix = |dark: f32, bright: f32| dark + (bright - dark) * t;
    let primary = mk::Rgba {
        r: mix(1.0, 0.11),
        g: mix(1.0, 0.11),
        b: mix(1.0, 0.13),
        a: 0.95,
    };
    let secondary = mk::Rgba { a: 0.68, ..primary };
    (primary, secondary)
}
