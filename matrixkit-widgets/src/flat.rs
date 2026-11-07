//! flat — the kit's guaranteed 2D painting layer for canvases.
//!
//! Born from the reference app dig (R-reference app, 23.08.2026): on the Surface's
//! Intel iGPU the iced canvas never showed VECTOR meshes (fill/stroke
//! invisible, even in separate frames) and silently dropped the
//! rotation of drawn images — while plain images and `fill_text`
//! rendered reliably and in call order on every machine we have.
//!
//! DOCTRINE: a MatrixKit canvas that must work everywhere paints with
//! IMAGES only. This module is that brush box:
//!
//! * [`Paint`] — colored 1×1 quads and soft-edged disc sprites, baked
//!   once per color and drawn at any size and opacity.
//! * [`q`] / [`dot`] / [`dotted`] — rectangles, circles, and lines as
//!   disc chains (rotation-free; broad chains read as brush strokes).
//! * [`Sheet`] — sprite-sheet loading: iced images know no source
//!   rectangle, so a sheet is sliced ONCE into per-cell handles;
//!   layered sheets (body + clothes) composite via [`Sheet::dress`].
//!
//! Everything here is renderer-agnostic data (image handles) — if the
//! mesh path ever heals, callers lose nothing by having used this.

use iced::widget::canvas;
use iced::widget::image::Handle;
use iced::{Point, Rectangle, Size};
use std::collections::HashMap;
use std::path::Path;

// ------------------------------------------------------------ paint

fn key(c: (u8, u8, u8)) -> u32 {
    ((c.0 as u32) << 16) | ((c.1 as u32) << 8) | c.2 as u32
}

/// Pre-baked color primitives: one opaque 1×1 quad and one 33×33 disc
/// per registered color. Register every color up front — lookups never
/// allocate, and an unknown color falls back to the first registered.
pub struct Paint {
    quads: HashMap<u32, Handle>,
    discs: HashMap<u32, Handle>,
}

impl Paint {
    pub fn new(colors: &[(u8, u8, u8)]) -> Self {
        let mut paint = Self { quads: HashMap::new(), discs: HashMap::new() };
        for c in colors {
            paint.add(*c);
        }
        paint
    }

    pub fn add(&mut self, c: (u8, u8, u8)) {
        let k = key(c);
        self.quads
            .entry(k)
            .or_insert_with(|| Handle::from_rgba(1, 1, vec![c.0, c.1, c.2, 255]));
        self.discs.entry(k).or_insert_with(|| {
            let n = 33u32;
            let r = 16.0f32;
            let mut buf = Vec::with_capacity((n * n * 4) as usize);
            for y in 0..n {
                for x in 0..n {
                    let d = (((x as f32 - 16.0).powi(2) + (y as f32 - 16.0).powi(2)).sqrt()
                        - r)
                        .max(0.0);
                    let a = ((1.0 - d).clamp(0.0, 1.0) * 255.0) as u8;
                    buf.extend_from_slice(&[c.0, c.1, c.2, a]);
                }
            }
            Handle::from_rgba(n, n, buf)
        });
    }

    pub fn quad(&self, c: (u8, u8, u8)) -> &Handle {
        self.quads
            .get(&key(c))
            .unwrap_or_else(|| self.quads.values().next().expect("Paint without colors"))
    }

    pub fn disc(&self, c: (u8, u8, u8)) -> &Handle {
        self.discs
            .get(&key(c))
            .unwrap_or_else(|| self.discs.values().next().expect("Paint without colors"))
    }
}

// ------------------------------------------------------------ brushes

/// A colored rectangle, painted as an image quad.
pub fn q(frame: &mut canvas::Frame, h: &Handle, r: Rectangle, opacity: f32) {
    frame.draw_image(r, canvas::Image::new(h.clone()).opacity(opacity));
}

/// A filled circle from a disc sprite.
pub fn dot(frame: &mut canvas::Frame, h: &Handle, c: Point, r: f32, opacity: f32) {
    frame.draw_image(
        Rectangle::new(Point::new(c.x - r, c.y - r), Size::new(2.0 * r, 2.0 * r)),
        canvas::Image::new(h.clone()).opacity(opacity),
    );
}

/// A line as a chain of discs — rotation-free; wide chains with small
/// spacing read as solid brush strokes, sparse ones as dotted routes.
pub fn dotted(
    frame: &mut canvas::Frame,
    h: &Handle,
    a: Point,
    b: Point,
    r: f32,
    space: f32,
    opacity: f32,
) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len = (dx * dx + dy * dy).sqrt();
    let steps = (len / space).ceil().max(1.0) as usize;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        dot(frame, h, Point::new(a.x + dx * t, a.y + dy * t), r, opacity);
    }
}

// ------------------------------------------------------------ sheets

/// A loaded sprite sheet. iced images know no source rectangle, so
/// cells are cut ONCE into their own handles and drawn like any image.
pub struct Sheet(image::RgbaImage);

impl Sheet {
    pub fn open(path: &Path) -> Option<Self> {
        Some(Self(image::open(path).ok()?.to_rgba8()))
    }

    /// Alpha-composite an equally laid-out layer on top (LPC-style
    /// layered sheets: body + clothes share one frame grid).
    pub fn dress(mut self, layer: &Sheet) -> Self {
        let (w, h) = (
            self.0.width().min(layer.0.width()),
            self.0.height().min(layer.0.height()),
        );
        for y in 0..h {
            for x in 0..w {
                let top = layer.0.get_pixel(x, y).0;
                let a = top[3] as u32;
                if a == 0 {
                    continue;
                }
                let base = &mut self.0.get_pixel_mut(x, y).0;
                for i in 0..3 {
                    base[i] = ((top[i] as u32 * a + base[i] as u32 * (255 - a)) / 255) as u8;
                }
                base[3] = base[3].max(top[3]);
            }
        }
        self
    }

    /// Cut one rectangle into its own handle.
    pub fn cut(&self, x: u32, y: u32, w: u32, h: u32) -> Handle {
        let mut out = Vec::with_capacity((w * h * 4) as usize);
        for row in y..(y + h).min(self.0.height()) {
            for col in x..(x + w).min(self.0.width()) {
                out.extend_from_slice(&self.0.get_pixel(col, row).0);
            }
        }
        Handle::from_rgba(w.min(self.0.width() - x), h.min(self.0.height() - y), out)
    }

    /// Cut a full cell grid, row-major: handles[row * columns + column].
    pub fn grid(&self, columns: u32, rows: u32, cell_w: u32, cell_h: u32) -> Vec<Handle> {
        let mut out = Vec::with_capacity((columns * rows) as usize);
        for row in 0..rows {
            for column in 0..columns {
                out.push(self.cut(column * cell_w, row * cell_h, cell_w, cell_h));
            }
        }
        out
    }
}
