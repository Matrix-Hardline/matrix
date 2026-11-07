//! The image atlas (iced replacement, stage 4, first piece —
//! GRABUNG-ICED-ABLOESUNG.md, camp 15).
//!
//! iced keeps images in a GPU atlas; our scene floor paints on
//! the CPU — so the atlas here is a DECODE MEMORY:
//! every image is opened ONCE and scaled ONCE to its target size
//! (the 780 ms lesson of first light: decoding is
//! the most expensive step of the whole paint path), after that a
//! frame only costs the blit. The contract follows the kit language:
//!
//! * `fetch()` fetches (and decodes at most the first time) —
//!   like measure() before layout it gives the layout the sizes.
//! * `paint()` blits after layout: rounded corners via the same
//!   distance function as the rasteriser, clip box like the
//!   text machine, blended over the ground.
//! * `sweep()` sweeps what nobody has fetched since the last sweep
//!   — the same life rule as mk::scene::Keep.
//!
//! Failed decodes are remembered AS A FAILURE:
//! a broken image costs one attempt, not one per frame.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// How the image fills its target size (iced ContentFit, the two
/// modes the kit really uses).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fit {
    /// Fill and crop centred (wall, tile thumbnails).
    Cover,
    /// Show whole, smaller if need be (the view).
    Contain,
    /// R210: a PUT picture of any size, stretched onto the box when it
    /// is drawn — the GPU scales it for free (video frames: a 1280-wide
    /// frame on a 1470-wide stage instead of 146 MB/s of panel-sized
    /// RGBA through pipe and upload). On the CPU path the atlas scales a
    /// copy once per frame.
    Stretch,
}

struct Entry {
    w: u32,
    h: u32,
    /// RGBA rows at target size — source alpha is preserved.
    data: Vec<u8>,
    used: bool,
    /// Placed (put) instead of decoded — already lies exactly at the target size.
    put: bool,
    /// R210: planar YUV 4:2:0 instead of RGBA (video frames for the GPU,
    /// which converts in its shader) — `data` is w*h*3/2 bytes.
    yuv: bool,
    /// Round 67: with `yuv`, the chroma is NV12 (one interleaved UV plane,
    /// the Venus hardware decoder's own layout) instead of I420.
    nv12: bool,
}

/// Camp 106: the RESOLVER — whoever knows a key that no
/// filesystem has (shape://…) computes the rows at the target size.
/// It is asked only when the map misses, before the decoder.
pub type Resolver = Box<dyn Fn(&Path, u32, u32) -> Option<(u32, u32, Vec<u8>)> + Send + Sync>;

#[derive(Default)]
pub struct ImageAtlas {
    map: HashMap<(PathBuf, u32, u32, bool), Option<Entry>>,
    decodes: usize,
    resolver: Option<Resolver>,
    /// Round 140: computed pictures that outlive frames that do not draw
    /// them (newest last, at most `PINS`).
    pins: Vec<PathBuf>,
}

impl ImageAtlas {
    pub fn new() -> Self {
        Self::default()
    }

    /// With a resolver for synthetic keys (camp 106).
    pub fn with_resolver(resolver: Resolver) -> Self {
        Self { resolver: Some(resolver), ..Self::default() }
    }

    pub fn set_resolver(&mut self, resolver: Resolver) {
        self.resolver = Some(resolver);
    }

    /// How often decoding really happened — the atlas's yardstick.
    pub fn decodes(&self) -> usize {
        self.decodes
    }

    /// Sweep: what nobody has fetched since the last sweep()
    /// drops; the marks are cleared for the next round.
    /// How many pictures may be pinned at once (the oldest pin goes).
    pub const PINS: usize = 16;

    /// PIN (round 140): keep a COMPUTED picture (`put`) across frames that
    /// do not draw it. sweep() drops whatever a frame did not draw — and a
    /// house with several surfaces sweeps after EACH of them: the lock's
    /// glass keys (lower panel) went after every frame of the upper one and
    /// were rebuilt (50–200 ms) all the time, the clock's glass went while
    /// the keypad was up. Glass families pin their maps themselves.
    pub fn pin(&mut self, path: &Path) {
        if let Some(i) = self.pins.iter().position(|p| p == path) {
            let p = self.pins.remove(i);
            self.pins.push(p);
            return;
        }
        self.pins.push(path.to_path_buf());
        if self.pins.len() > Self::PINS {
            self.pins.remove(0);
        }
    }

    /// Let a pinned picture go with the next sweep that does not draw it.
    pub fn unpin(&mut self, path: &Path) {
        self.pins.retain(|p| p != path);
    }

    pub fn sweep(&mut self) {
        // A file that could not be read stays remembered (no decode per
        // frame); a COMPUTED name ('\0…') that was missing never comes from
        // disk — kept, every new running number left a leaf behind
        // (round 127, the lock's minute clock).
        let pins = &self.pins;
        self.map.retain(|(p, ..), e| {
            e.as_ref().is_some_and(|e| e.used || (e.put && pins.iter().any(|q| q == p)))
                // Round 178: a failed SHAPE resolve (the registry was cleared mid-view) is forgotten like a computed
                // one, so the next frame tries again — kept, the shape stayed invisible at that size for good.
                || (e.is_none() && {
                    let name = p.as_os_str().to_string_lossy();
                    !name.starts_with('\0') && !name.starts_with("shape://")
                })
        });
        for e in self.map.values_mut().flatten() {
            e.used = false;
        }
    }

    /// Round 178b: the gentle sweep for a house's PANES — drops only what the atlas can make again by itself (files,
    /// `shape://` resolves) and leaves every `put` picture alone. The first version swept panes with sweep(): the
    /// greeter's glass maps (computed, put once, drawn by a pane that does not re-put them) went after 20 s and the
    /// glass showed only its colour fringe. Computed pictures belong to their owner and the main sweep.
    pub fn sweep_remakeable(&mut self) {
        self.map.retain(|_, e| match e {
            Some(e) => e.used || e.put,
            None => true,
        });
        for e in self.map.values_mut().flatten() {
            if !e.put {
                e.used = false;
            }
        }
    }

    /// Camp 22: the atlas also accepts COMPUTED images —
    /// put() stores finished RGBA rows under a synthetic
    /// key (no filesystem needed); fetch()/paint()
    /// treat them like any decoded image. Whoever recomputes
    /// chooses a NEW key (sequence number): the diff sees
    /// the change, sweep() sweeps away the old state.
    pub fn put(&mut self, key: &Path, w: u32, h: u32, rgba: Vec<u8>) {
        debug_assert_eq!(rgba.len(), (w * h * 4) as usize);
        self.map.insert(
            (key.to_path_buf(), w, h, true),
            Some(Entry { w, h, data: rgba, used: true, put: true, yuv: false, nv12: false }),
        );
    }

    /// R210: a video frame as planar YUV 4:2:0 (BT.709, limited range):
    /// `w` and `h` even, `yuv` = w*h Y bytes, then U and V at half size.
    /// Meant for Fit::Stretch — the GPU converts it in its shader, the CPU
    /// path converts it once when it scales the copy.
    pub fn put_yuv(&mut self, key: &Path, w: u32, h: u32, yuv: Vec<u8>) {
        debug_assert_eq!(yuv.len(), (w * h * 3 / 2) as usize);
        self.map.insert(
            (key.to_path_buf(), w, h, true),
            Some(Entry { w, h, data: yuv, used: true, put: true, yuv: true, nv12: false }),
        );
    }

    /// A decoded video frame, whatever its kind (R210 family): YUV 4:2:0
    /// or RGBA — the one call players need (`matrixkit_video::Video::yuv`).
    /// Round 67: a video frame as NV12 (Y plane, then interleaved UV —
    /// w*h*3/2 bytes): the hardware decoder's layout, so its frames reach
    /// the GPU with plain row copies (the shader `fs_nv12` converts).
    pub fn put_nv12(&mut self, key: &Path, w: u32, h: u32, nv12: Vec<u8>) {
        debug_assert_eq!(nv12.len(), (w * h * 3 / 2) as usize);
        self.map.insert(
            (key.to_path_buf(), w, h, true),
            Some(Entry { w, h, data: nv12, used: true, put: true, yuv: true, nv12: true }),
        );
    }

    pub fn put_frame(&mut self, key: &Path, w: u32, h: u32, bytes: Vec<u8>, yuv: bool) {
        if yuv {
            self.put_yuv(key, w, h, bytes);
        } else {
            self.put(key, w, h, bytes);
        }
    }

    /// Camp 26: DELIVERED images (PNG bytes from an icon painter,
    /// from a network fetch) to RGBA rows EXACTLY at the target size — for
    /// put(), without a detour via the filesystem. (put keys carry
    /// the size; whoever places must match it.)
    pub fn unpack(bytes: &[u8], bw: u32, bh: u32) -> Option<(u32, u32, Vec<u8>)> {
        let img = image::load_from_memory(bytes).ok()?;
        let img = img.resize_exact(bw, bh, image::imageops::FilterType::Triangle);
        let rgba = img.to_rgba8();
        Some((bw, bh, rgba.into_raw()))
    }

    /// The key under which this image REALLY lies.
    ///
    /// The fit switch belongs to the DECODE PATH, not to the
    /// result: a placed image already lies exactly at the
    /// target size, where contain and cover coincide. Without
    /// this handle a contain image misses everything that put()
    /// stored under cover — the atlas then reaches for the
    /// filesystem, which never succeeds for a synthetic key,
    /// and silently remembers the failure. (Matrix
    /// Home painted empty tiles, finger probe 16.08.2026.)
    fn slot(&self, path: &Path, bw: u32, bh: u32, fit: Fit) -> (PathBuf, u32, u32, bool) {
        let exact = (path.to_path_buf(), bw, bh, fit == Fit::Cover);
        if self.map.contains_key(&exact) {
            return exact;
        }
        let twin = (path.to_path_buf(), bw, bh, fit != Fit::Cover);
        if self.map.get(&twin).is_some_and(|e| e.as_ref().is_some_and(|e| e.put)) {
            return twin;
        }
        exact
    }

    /// Fetches the SCALED size of an image in the box (bw × bh, depending
    /// on fit) — decodes at most the first time. None means:
    /// this image is not readable (and stays remembered
    /// as a failure).
    pub fn fetch(&mut self, path: &Path, bw: u32, bh: u32, fit: Fit) -> Option<(u32, u32)> {
        if fit == Fit::Stretch {
            return self.fetch_stretched(path, bw, bh);
        }
        let key = self.slot(path, bw, bh, fit);
        // Round 126: a COMPUTED picture asked at a box a rounding off its
        // own size (logical × scale) was simply not found — it vanished
        // without a word (the lock's glass clock, 05.11.2026). Same shape
        // → scale the put picture to the box, as Stretch does.
        if fit == Fit::Contain && !self.map.contains_key(&key) {
            if let Some(src) = self.put_key(path) {
                let (sa, ba) = (src.1 as f32 / src.2.max(1) as f32, bw as f32 / bh.max(1) as f32);
                if (sa - ba).abs() <= 0.03 * ba {
                    return self.fetch_stretched(path, bw, bh);
                }
            }
        }
        if !self.map.contains_key(&key) {
            let resolved = self
                .resolver
                .as_ref()
                .and_then(|r| r(path, bw, bh))
                .map(|(w, h, data)| Entry { w, h, data, used: true, put: true, yuv: false, nv12: false });
            let entry = resolved.or_else(|| Self::decode(path, bw, bh, fit));
            if entry.is_some() {
                self.decodes += 1;
            }
            self.map.insert(key.clone(), entry);
        }
        let entry = self.map.get_mut(&key)?.as_mut()?;
        entry.used = true;
        Some((entry.w, entry.h))
    }

    /// The put picture under `path`, whatever its size (Fit::Stretch).
    fn put_key(&self, path: &Path) -> Option<(PathBuf, u32, u32, bool)> {
        self.map
            .iter()
            .filter(|((p, ..), e)| p == path && e.as_ref().is_some_and(|e| e.put))
            .map(|(k, _)| k.clone())
            .max_by_key(|k| k.1 * k.2)
    }

    /// CPU path of Fit::Stretch: a copy scaled to the box, kept as a put
    /// entry of the box size (swept with the frame like the source).
    fn fetch_stretched(&mut self, path: &Path, bw: u32, bh: u32) -> Option<(u32, u32)> {
        let exact = (path.to_path_buf(), bw, bh, true);
        if let Some(Some(e)) = self.map.get_mut(&exact) {
            e.used = true;
            return Some((e.w, e.h));
        }
        let source = self.put_key(path)?;
        let entry = self.map.get_mut(&source)?.as_mut()?;
        entry.used = true;
        let rgba = match (entry.yuv, entry.nv12) {
            (true, true) => nv12_to_rgba(&entry.data, entry.w, entry.h),
            (true, false) => yuv420_to_rgba(&entry.data, entry.w, entry.h),
            _ => entry.data.clone(),
        };
        let img = image::RgbaImage::from_raw(entry.w, entry.h, rgba)?;
        let scaled = image::imageops::resize(&img, bw.max(1), bh.max(1), image::imageops::FilterType::Triangle);
        self.map.insert(exact, Some(Entry { w: bw.max(1), h: bh.max(1), data: scaled.into_raw(), used: true, put: true, yuv: false, nv12: false }));
        Some((bw.max(1), bh.max(1)))
    }

    /// GPU path of Fit::Stretch: mark the put picture as used (call before
    /// the immutable `stretched_ready`).
    pub fn touch_stretched(&mut self, path: &Path) {
        if let Some(key) = self.put_key(path) {
            if let Some(Some(e)) = self.map.get_mut(&key) {
                e.used = true;
            }
        }
    }

    /// GPU path of Fit::Stretch: the put picture in its own size — the
    /// painter stretches it onto the box.
    /// The last values say whether the bytes are YUV 4:2:0 (`put_yuv`)
    /// and, if so, whether its chroma is NV12 (`put_nv12`).
    pub fn stretched_ready(&self, path: &Path) -> Option<(u32, u32, &[u8], bool, bool)> {
        let key = self.put_key(path)?;
        let e = self.map.get(&key)?.as_ref()?;
        Some((e.w, e.h, &e.data, e.yuv, e.nv12))
    }

    /// The GPU seam (stage 4, GRABUNG-GPU-MALEREI.md): the fully
    /// scaled RGBA image plus its size, for the texture upload —
    /// decoded at most the first time, like fetch().
    pub fn peek(&mut self, path: &Path, bw: u32, bh: u32, fit: Fit) -> Option<(u32, u32, &[u8])> {
        self.fetch(path, bw, bh, fit)?;
        let key = self.slot(path, bw, bh, fit);
        let entry = self.map.get(&key)?.as_ref()?;
        Some((entry.w, entry.h, &entry.data))
    }

    /// Like peek(), but IMMUTABLE — for the second grab,
    /// when fetch() already has everything in place: many images per frame
    /// need many simultaneous borrows.
    pub fn peek_ready(&self, path: &Path, bw: u32, bh: u32, fit: Fit) -> Option<(u32, u32, &[u8])> {
        let key = self.slot(path, bw, bh, fit);
        let entry = self.map.get(&key)?.as_ref()?;
        Some((entry.w, entry.h, &entry.data))
    }

    fn decode(path: &Path, bw: u32, bh: u32, fit: Fit) -> Option<Entry> {
        let img = image::open(path).ok()?;
        let img = match fit {
            Fit::Cover => img.resize_to_fill(bw, bh, image::imageops::FilterType::Triangle),
            Fit::Contain => img.resize(bw, bh, image::imageops::FilterType::Triangle),
            Fit::Stretch => img.resize_exact(bw, bh, image::imageops::FilterType::Triangle),
        };
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        Some(Entry { w, h, data: rgba.into_raw(), used: false, put: false, yuv: false, nv12: false })
    }

    /// Blits the (fetched) entry at (x, y) into an
    /// XRGB8888 buffer (B G R X, little-endian): rounded corners via
    /// the rasteriser's distance function, optional clip box,
    /// source alpha blended over the ground.
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        path: &Path,
        bw_box: u32,
        bh_box: u32,
        fit: Fit,
        x: f32,
        y: f32,
        radius: f32,
        clip: Option<matrixkit_theme::scene::Rect>,
        buf: &mut [u8],
        cw: u32,
        ch: u32,
    ) {
        let Some((iw, ih)) = self.fetch(path, bw_box, bh_box, fit) else {
            return;
        };
        let key = self.slot(path, bw_box, bh_box, fit);
        let Some(Some(entry)) = self.map.get(&key) else {
            return;
        };
        let (cx0, cy0, cx1, cy1) = match clip {
            Some(c) => (
                c.x.max(0.0) as i32,
                c.y.max(0.0) as i32,
                ((c.x + c.w).min(cw as f32)) as i32,
                ((c.y + c.h).min(ch as f32)) as i32,
            ),
            None => (0, 0, cw as i32, ch as i32),
        };
        let r = radius.min(iw as f32 / 2.0).min(ih as f32 / 2.0);
        let (mx, my) = (x + iw as f32 / 2.0, y + ih as f32 / 2.0);
        let (hx, hy) = (iw as f32 / 2.0 - r, ih as f32 / 2.0 - r);
        for sy in 0..ih {
            let py = y as i32 + sy as i32;
            if py < cy0 || py >= cy1 {
                continue;
            }
            for sx in 0..iw {
                let px = x as i32 + sx as i32;
                if px < cx0 || px >= cx1 {
                    continue;
                }
                // Corner coverage: the FULL rounded-box distance function
                // (negative inside — otherwise a radius-0 box covers only
                // half; the finding of this test).
                let qx = (px as f32 + 0.5 - mx).abs() - hx;
                let qy = (py as f32 + 0.5 - my).abs() - hy;
                let dist = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt()
                    + qx.max(qy).min(0.0)
                    - r;
                let cover = (0.5 - dist).clamp(0.0, 1.0);
                if cover <= 0.0 {
                    continue;
                }
                let s = ((sy * iw + sx) * 4) as usize;
                let a = entry.data[s + 3] as f32 / 255.0 * cover;
                if a <= 0.0 {
                    continue;
                }
                let d = ((py as u32 * cw + px as u32) * 4) as usize;
                buf[d] = (entry.data[s + 2] as f32 * a + buf[d] as f32 * (1.0 - a)) as u8;
                buf[d + 1] = (entry.data[s + 1] as f32 * a + buf[d + 1] as f32 * (1.0 - a)) as u8;
                buf[d + 2] = (entry.data[s] as f32 * a + buf[d + 2] as f32 * (1.0 - a)) as u8;
                buf[d + 3] = (255.0 * a + buf[d + 3] as f32 * (1.0 - a)) as u8;
            }
        }
    }
}


/// Planar YUV 4:2:0 (BT.709, limited range) to RGBA — the CPU fallback of
/// `put_yuv`; the GPU does the same in its shader.
pub fn yuv420_to_rgba(yuv: &[u8], w: u32, h: u32) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut out = vec![255u8; w * h * 4];
    if yuv.len() < w * h * 3 / 2 {
        return out;
    }
    let (ys, rest) = yuv.split_at(w * h);
    let (us, vs) = rest.split_at(w * h / 4);
    for y in 0..h {
        for x in 0..w {
            let c = (y / 2) * (w / 2) + x / 2;
            let l = (ys[y * w + x] as f32 - 16.0) * 1.164;
            let u = us[c] as f32 - 128.0;
            let v = vs[c] as f32 - 128.0;
            let o = (y * w + x) * 4;
            out[o] = (l + 1.793 * v).clamp(0.0, 255.0) as u8;
            out[o + 1] = (l - 0.213 * u - 0.533 * v).clamp(0.0, 255.0) as u8;
            out[o + 2] = (l + 2.112 * u).clamp(0.0, 255.0) as u8;
        }
    }
    out
}

/// NV12 (Y plane, interleaved UV) to RGBA — the CPU fallback of
/// `put_nv12`, same colours as `yuv420_to_rgba`.
pub fn nv12_to_rgba(nv12: &[u8], w: u32, h: u32) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut i420 = vec![0u8; w * h * 3 / 2];
    if nv12.len() < w * h * 3 / 2 {
        return vec![255u8; w * h * 4];
    }
    i420[..w * h].copy_from_slice(&nv12[..w * h]);
    let (us, vs) = i420[w * h..].split_at_mut(w * h / 4);
    for (i, pair) in nv12[w * h..w * h * 3 / 2].chunks_exact(2).enumerate().take(w * h / 4) {
        us[i] = pair[0];
        vs[i] = pair[1];
    }
    yuv420_to_rgba(&i420, w as u32, h as u32)
}

#[cfg(test)]
mod pin_tests {
    use super::*;

    #[test]
    fn a_pinned_picture_outlives_frames_that_do_not_draw_it() {
        let mut a = ImageAtlas::new();
        let (pinned, loose) = (Path::new("\0glas-a"), Path::new("\0glas-b"));
        a.put(pinned, 2, 2, vec![255; 16]);
        a.put(loose, 2, 2, vec![255; 16]);
        a.pin(pinned);
        a.sweep(); // the frame that made them
        a.sweep(); // a frame of another surface
        assert!(a.stretched_ready(pinned).is_some());
        assert!(a.stretched_ready(loose).is_none());
        a.unpin(pinned);
        a.sweep();
        a.sweep();
        assert!(a.stretched_ready(pinned).is_none());
    }

    #[test]
    fn pins_are_bounded() {
        let mut a = ImageAtlas::new();
        for k in 0..ImageAtlas::PINS + 4 {
            a.pin(Path::new(&format!("\0p{k}")));
        }
        assert_eq!(a.pins.len(), ImageAtlas::PINS);
        assert_eq!(a.pins[0], PathBuf::from("\0p4"));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_put_picture_is_found_a_rounding_off_its_size() {
        let mut atlas = super::ImageAtlas::new();
        let key = std::path::Path::new("\u{0}probe-glas");
        atlas.put(key, 300, 150, vec![255u8; 300 * 150 * 4]);
        // asked at 301×150 (a box of 100.33 logical px at 3×, rounded)
        assert_eq!(atlas.fetch(key, 301, 150, super::Fit::Contain), Some((301, 150)));
        assert!(atlas.peek_ready(key, 301, 150, super::Fit::Contain).is_some());
        // a different shape is not bent into place
        assert_eq!(atlas.fetch(key, 150, 150, super::Fit::Contain), None);
    }

    use super::*;

    #[test]
    fn nv12_and_i420_give_the_same_colours() {
        let (w, h) = (4usize, 2usize);
        let mut i420 = vec![100u8; w * h];
        i420.extend_from_slice(&[60, 200]); // U
        i420.extend_from_slice(&[180, 90]); // V
        let mut nv12 = vec![100u8; w * h];
        nv12.extend_from_slice(&[60, 180, 200, 90]);
        assert_eq!(nv12_to_rgba(&nv12, w as u32, h as u32), yuv420_to_rgba(&i420, w as u32, h as u32));
    }

    /// A small real PNG as a sample (red, 8×4).
    fn probe_png(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        let img = image::RgbaImage::from_pixel(8, 4, image::Rgba([255, 0, 0, 255]));
        img.save(&path).expect("Probestück schreiben");
        path
    }

    #[test]
    fn the_atlas_decodes_once_and_remembers_failures() {
        let path = probe_png("mk-atlas-einmal.png");
        let mut atlas = ImageAtlas::new();
        let a = atlas.fetch(&path, 4, 4, Fit::Cover);
        let b = atlas.fetch(&path, 4, 4, Fit::Cover);
        assert_eq!(a, b);
        assert_eq!(atlas.decodes(), 1, "zweimal holen, EINMAL dekodieren");
        // A different target size is a different entry …
        atlas.fetch(&path, 2, 2, Fit::Cover);
        assert_eq!(atlas.decodes(), 2);
        // … and a broken image costs exactly one attempt.
        let broken = std::env::temp_dir().join("mk-atlas-kaputt.png");
        std::fs::write(&broken, b"kein Bild").unwrap();
        assert!(atlas.fetch(&broken, 4, 4, Fit::Cover).is_none());
        assert!(atlas.fetch(&broken, 4, 4, Fit::Cover).is_none());
        assert_eq!(atlas.decodes(), 2, "Fehlschläge werden gemerkt, nie wiederholt");
    }

    #[test]
    fn cover_fills_and_contain_shows_all() {
        let path = probe_png("mk-atlas-masse.png"); // 8×4, landscape
        let mut atlas = ImageAtlas::new();
        // Cover into a square: fills BOTH sides.
        assert_eq!(atlas.fetch(&path, 4, 4, Fit::Cover), Some((4, 4)));
        // Contain: fully visible, aspect ratio holds.
        assert_eq!(atlas.fetch(&path, 4, 4, Fit::Contain), Some((4, 2)));
    }

    #[test]
    fn computed_images_live_like_decoded_ones() {
        // Camp 22: put() delivers, fetch() finds, sweep() sweeps.
        let mut atlas = ImageAtlas::new();
        let key = PathBuf::from("\u{0}mandel:1");
        atlas.put(&key, 2, 2, vec![255; 16]);
        assert_eq!(atlas.fetch(&key, 2, 2, Fit::Cover), Some((2, 2)));
        assert_eq!(atlas.decodes(), 0, "gerechnet ist nicht dekodiert");
        atlas.sweep();
        assert_eq!(atlas.fetch(&key, 2, 2, Fit::Cover), Some((2, 2)), "geholt überlebt die Kehrung");
        atlas.sweep();
        atlas.sweep();
        assert!(atlas.fetch(&key, 2, 2, Fit::Cover).is_none() || atlas.decodes() > 0,
            "wen niemand holt, den kehrt sweep — der Schlüssel ist kein Dateipfad");
    }

    #[test]
    fn sweep_keeps_only_the_fetched() {
        let path = probe_png("mk-atlas-kehrung.png");
        let mut atlas = ImageAtlas::new();
        atlas.fetch(&path, 4, 4, Fit::Cover);
        atlas.fetch(&path, 2, 2, Fit::Cover);
        atlas.sweep(); // both were fetched → both stay
        atlas.fetch(&path, 4, 4, Fit::Cover);
        atlas.sweep(); // only 4×4 has been fetched since
        assert_eq!(atlas.decodes(), 2);
        atlas.fetch(&path, 2, 2, Fit::Cover);
        assert_eq!(atlas.decodes(), 3, "der gekehrte Eintrag muss neu dekodieren");
    }

    #[test]
    fn paint_blits_inside_bounds_and_clip() {
        let path = probe_png("mk-atlas-blit.png");
        let mut atlas = ImageAtlas::new();
        let (cw, chh) = (16u32, 8u32);
        let mut buf = vec![0u8; (cw * chh * 4) as usize];
        let clip = matrixkit_theme::scene::Rect { x: 0.0, y: 0.0, w: 16.0, h: 2.0 };
        atlas.paint(&path, 8, 4, Fit::Cover, 2.0, 0.0, 0.0, Some(clip), &mut buf, cw, chh);
        // In the box: red blended (B G R X → R at +2).
        let px = |x: u32, y: u32| &buf[((y * cw + x) * 4) as usize..((y * cw + x) * 4 + 4) as usize];
        assert!(px(4, 1)[2] > 200, "im Kasten liegt das Rot");
        // Below the clip edge: silence.
        assert!((2..chh).all(|y| (0..cw).all(|x| px(x, y).iter().all(|&b| b == 0))));
    }

    /// A placed image must be found under BOTH fit paths:
    /// it already lies at the target size, and its key is
    /// synthetic — if the atlas reaches for the filesystem, the
    /// image is lost forever (finger probe Matrix Home, 16.08.).
    #[test]
    fn a_laid_image_is_found_under_either_fit() {
        let key = std::path::PathBuf::from("\u{0}probe:tile:1");
        let mut atlas = ImageAtlas::new();
        atlas.put(&key, 2, 2, vec![255u8; 2 * 2 * 4]);
        assert_eq!(atlas.fetch(&key, 2, 2, Fit::Cover), Some((2, 2)));
        assert_eq!(
            atlas.fetch(&key, 2, 2, Fit::Contain),
            Some((2, 2)),
            "Contain muss finden, was put() unter Cover ablegte"
        );
        assert_eq!(atlas.decodes(), 0, "gelegt ist nie dekodiert");
        let mut buf = vec![0u8; 4 * 4 * 4];
        atlas.paint(&key, 2, 2, Fit::Contain, 1.0, 1.0, 0.0, None, &mut buf, 4, 4);
        assert!(buf.iter().any(|&b| b != 0), "Contain-paint muss malen");
    }
}

/// R207: the SEED from the foreign desktop's wallpaper — the
/// colour with the most saturation that really occurs in the image
/// (matugen likewise takes the strongest hue, not the mean).
/// Scaled down (64×64), hue into 24 bins, weight = saturation;
/// the fullest bin gives the seed as a weighted mean.
/// IMAGE FITTED TO THE SCREEN (R209 family, iced-free): decode `path` and
/// shrink it so its longest side is at most `long` px (never enlarged) —
/// RGBA8 with its size. A wallpaper of 8400 × 5600 decoded whole is 188 MB
/// per surface; on a duo panel (`mk::device::image_long_side()`, 1800) the
/// fitted copy is ~10 MB. None if the file does not decode.
pub fn fitted_rgba(path: &Path, long: u32) -> Option<(u32, u32, Vec<u8>)> {
    // Round 166: the format from the CONTENT, not the name — the login
    // share's copy («mobile.wand») has no extension and failed in 0 ms.
    let picture = image::ImageReader::open(path).ok()?.with_guessed_format().ok()?.decode().ok()?;
    // R216: a huge photo first shrinks with the fast box filter to twice
    // the goal, then the smooth filter does the last step (a 20-MP
    // wallpaper took seconds through Triangle alone on the duo).
    let picture = if picture.width().max(picture.height()) > long * 2 {
        picture.thumbnail(long * 2, long * 2)
    } else {
        picture
    };
    let picture = if picture.width().max(picture.height()) > long {
        picture.resize(long, long, image::imageops::FilterType::Triangle)
    } else {
        picture
    };
    let rgba = picture.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some((w, h, rgba.into_raw()))
}

pub fn wallpaper_seed() -> Option<matrixkit_theme::Rgba> {
    let path = matrixkit_theme::seed::wallpaper_path()?;
    seed_of(&path)
}

pub fn seed_of(path: &Path) -> Option<matrixkit_theme::Rgba> {
    let img = image::open(path).ok()?;
    let small = img.thumbnail(64, 64).to_rgb8();
    let mut bins: Vec<(f32, [f32; 3])> = vec![(0.0, [0.0; 3]); 24];
    for px in small.pixels() {
        let (r, g, b) = (px[0] as f32 / 255.0, px[1] as f32 / 255.0, px[2] as f32 / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let chroma = max - min;
        if chroma < 0.08 || max < 0.12 || min > 0.95 {
            continue;
        }
        let h = if max == r {
            ((g - b) / chroma).rem_euclid(6.0)
        } else if max == g {
            (b - r) / chroma + 2.0
        } else {
            (r - g) / chroma + 4.0
        };
        let bin = ((h / 6.0 * 24.0) as usize).min(23);
        let w = chroma * chroma;
        bins[bin].0 += w;
        bins[bin].1[0] += r * w;
        bins[bin].1[1] += g * w;
        bins[bin].1[2] += b * w;
    }
    let (w, sum) = bins.into_iter().max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))?;
    if w <= 0.0 {
        return None;
    }
    Some(matrixkit_theme::Rgba { r: sum[0] / w, g: sum[1] / w, b: sum[2] / w, a: 1.0 })
}

/// THUMBNAILS ON DISK (round 24, 02.11.2026): a picture shown small (a
/// gallery tile) is decoded ONCE from its large original — off the
/// drawing thread — and kept as a small PNG under ~/.cache/matrix/thumbs.
/// Before, the wallpaper gallery decoded 20-megapixel JPEGs inside the
/// frame, and tiles that scrolled out were swept and decoded again on the
/// way back: «it scrolls very badly».
pub mod thumbs {
    use std::hash::{Hash, Hasher};
    use std::path::{Path, PathBuf};

    fn dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
        PathBuf::from(home).join(".cache/matrix/thumbs")
    }

    /// Where the thumbnail of `src` at w×h lives (the original's size and
    /// change time are in the key: a changed picture gets a new thumb).
    pub fn path_for(src: &Path, w: u32, h: u32) -> PathBuf {
        let meta = std::fs::metadata(src).ok();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        src.hash(&mut hasher);
        meta.as_ref().map(|m| m.len()).hash(&mut hasher);
        meta.and_then(|m| m.modified().ok()).hash(&mut hasher);
        (w, h).hash(&mut hasher);
        dir().join(format!("{:016x}.png", hasher.finish()))
    }

    /// The thumbnail, made if missing (call this OFF the drawing thread):
    /// cropped to fill w×h (like Fit::Cover). None if the original is
    /// unreadable.
    pub fn ensure(src: &Path, w: u32, h: u32) -> Option<PathBuf> {
        let target = path_for(src, w, h);
        if target.is_file() {
            return Some(target);
        }
        let img = image::open(src).ok()?;
        // A cheap first shrink keeps the filter's work small on huge photos.
        let img = if img.width() > w * 4 && img.height() > h * 4 { img.thumbnail(w * 4, h * 4) } else { img };
        let small = img.resize_to_fill(w.max(1), h.max(1), image::imageops::FilterType::Triangle);
        std::fs::create_dir_all(dir()).ok()?;
        let tmp = target.with_extension("tmp.png");
        small.to_rgba8().save(&tmp).ok()?;
        std::fs::rename(&tmp, &target).ok()?;
        Some(target)
    }

    /// Can `ensure_video` make a thumbnail of this file (by extension)?
    pub fn is_video(src: &Path) -> bool {
        let ext = src.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        matches!(ext.as_str(), "mp4" | "mkv" | "webm" | "mov" | "m4v" | "avi")
    }

    /// A VIDEO'S THUMBNAIL (round 30): one frame a few seconds in, cropped
    /// to fill w×h, through ffmpeg (seek before the input: fast even on a
    /// 3 GB film), at low priority. Call OFF the drawing thread. None when
    /// ffmpeg is missing or the file has no picture (yet).
    pub fn ensure_video(src: &Path, w: u32, h: u32) -> Option<PathBuf> {
        let target = path_for(src, w, h);
        if target.is_file() {
            return Some(target);
        }
        // At most two ffmpeg at once (a folder of films asks for all of
        // them in one frame; the loom spins a thread each).
        static GATE: std::sync::Mutex<u32> = std::sync::Mutex::new(0);
        static FREE: std::sync::Condvar = std::sync::Condvar::new();
        {
            let mut busy = GATE.lock().ok()?;
            while *busy >= 2 {
                busy = FREE.wait(busy).ok()?;
            }
            *busy += 1;
        }
        struct Leave;
        impl Drop for Leave {
            fn drop(&mut self) {
                if let Ok(mut busy) = GATE.lock() {
                    *busy = busy.saturating_sub(1);
                }
                FREE.notify_one();
            }
        }
        let _leave = Leave;
        if target.is_file() {
            return Some(target);
        }
        std::fs::create_dir_all(dir()).ok()?;
        let tmp = target.with_extension(format!("{}.tmp.png", std::process::id()));
        let filter = format!("scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h}");
        for at in ["8", "1", "0"] {
            let ok = std::process::Command::new("nice")
                .args(["-n", "15", "ffmpeg", "-nostdin", "-loglevel", "error", "-y", "-ss", at, "-i"])
                .arg(src)
                .args(["-frames:v", "1", "-vf", &filter])
                .arg(&tmp)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok_and(|s| s.success());
            // A clip shorter than the seek writes nothing: try earlier.
            if ok && std::fs::metadata(&tmp).is_ok_and(|m| m.len() > 0) {
                std::fs::rename(&tmp, &target).ok()?;
                return Some(target);
            }
        }
        let _ = std::fs::remove_file(&tmp);
        None
    }

    /// Picture or video: the thumbnail by kind.
    pub fn ensure_any(src: &Path, w: u32, h: u32) -> Option<PathBuf> {
        if is_video(src) {
            ensure_video(src, w, h)
        } else {
            ensure(src, w, h)
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn makes_once_and_reuses() {
            let base = std::env::temp_dir().join(format!("mk-thumb-{}", std::process::id()));
            std::fs::create_dir_all(&base).unwrap();
            std::env::set_var("HOME", &base);
            let src = base.join("big.png");
            image::RgbaImage::from_pixel(800, 600, image::Rgba([10, 200, 30, 255])).save(&src).unwrap();
            let t = super::ensure(&src, 160, 90).unwrap();
            let img = image::open(&t).unwrap();
            assert_eq!((img.width(), img.height()), (160, 90));
            let again = super::ensure(&src, 160, 90).unwrap();
            assert_eq!(t, again);
            assert_ne!(super::path_for(&src, 160, 90), super::path_for(&src, 320, 180));
            assert!(super::ensure(&base.join("missing.jpg"), 10, 10).is_none());
            let _ = std::fs::remove_dir_all(&base);
        }
    }
}

/// THE BACKDROP (round 27): how bright the wallpaper is under a strip of
/// the screen — for ink on glass (mk::glass::ink). The wallpaper is shown
/// Cover on its screen, so a thumbnail cropped to the SCREEN's aspect maps
/// the screen linearly (cached through `thumbs`).
pub mod backdrop {
    use std::path::Path;

    /// Mean luminance (0..1) of `rect` (x, y, w, h in screen pixels) on a
    /// screen of `screen` size showing `wallpaper` Cover. Call off the
    /// drawing thread the first time (it may decode the original once).
    pub fn luma(wallpaper: &Path, screen: (f32, f32), rect: (f32, f32, f32, f32)) -> Option<f32> {
        let tw = 192u32;
        let th = ((tw as f32 * screen.1 / screen.0.max(1.0)).round() as u32).max(1);
        let thumb = super::thumbs::ensure(wallpaper, tw, th)?;
        let img = image::open(thumb).ok()?.to_rgba8();
        let (w, h) = img.dimensions();
        let sx = w as f32 / screen.0.max(1.0);
        let sy = h as f32 / screen.1.max(1.0);
        Some(luma_of(img.as_raw(), w, h, (rect.0 * sx, rect.1 * sy, rect.2 * sx, rect.3 * sy)))
    }

    /// The backdrop of a strip on a named screen showing the CURRENT
    /// wallpaper: (path, luma). The path tells the caller when to ask again.
    pub fn strip(output: &str, rect_of: impl Fn(f32, f32) -> (f32, f32, f32, f32)) -> Option<(String, f32)> {
        let path = matrixkit_theme::wallpaper::current()?;
        let (sw, sh) = matrixkit_theme::canvas::output_size(output)?;
        let l = luma(Path::new(&path), (sw, sh), rect_of(sw, sh))?;
        Some((path, l))
    }

    /// Pure part (tested): mean Rec.709 luminance of a region of RGBA.
    pub fn luma_of(rgba: &[u8], w: u32, h: u32, rect: (f32, f32, f32, f32)) -> f32 {
        let x0 = (rect.0.max(0.0) as u32).min(w.saturating_sub(1));
        let y0 = (rect.1.max(0.0) as u32).min(h.saturating_sub(1));
        let x1 = ((rect.0 + rect.2).ceil() as u32).clamp(x0 + 1, w);
        let y1 = ((rect.1 + rect.3).ceil() as u32).clamp(y0 + 1, h);
        let (mut sum, mut n) = (0.0f64, 0u64);
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y * w + x) * 4) as usize;
                let (r, g, b) = (rgba[i] as f64, rgba[i + 1] as f64, rgba[i + 2] as f64);
                sum += (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0;
                n += 1;
            }
        }
        if n == 0 { 0.0 } else { (sum / n as f64) as f32 }
    }

    #[cfg(test)]
    mod tests {
    #[test]
    fn the_pane_sweep_keeps_computed_pictures() {
        let mut a = crate::ImageAtlas::new();
        let key = std::path::Path::new("\0glass-map");
        a.put(key, 1, 1, vec![0, 0, 0, 255]);
        a.sweep(); // a main sweep clears the mark
        a.sweep_remakeable(); // the pane sweep must not drop it
        assert!(a.map.keys().any(|(p, ..)| p == key));
    }

        #[test]
        fn region_mean() {
            // 4x2: left half white, right half black.
            let mut px = Vec::new();
            for _ in 0..2 {
                for x in 0..4 {
                    let v = if x < 2 { 255 } else { 0 };
                    px.extend_from_slice(&[v, v, v, 255]);
                }
            }
            assert!((super::luma_of(&px, 4, 2, (0.0, 0.0, 2.0, 2.0)) - 1.0).abs() < 1e-3);
            assert!(super::luma_of(&px, 4, 2, (2.0, 0.0, 2.0, 2.0)) < 1e-3);
            assert!((super::luma_of(&px, 4, 2, (0.0, 0.0, 4.0, 2.0)) - 0.5).abs() < 1e-3);
        }
    }
}
