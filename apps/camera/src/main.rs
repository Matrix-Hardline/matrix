//! Matrix Kamera (R215, 02.11.2026): the duo's IMX351 without Android and
//! without CamX. `duo-camgrab` streams raw frames at 30 fps with its own
//! auto exposure (mk::camera::Stream); a worker thread fetches the newest
//! frame from its socket and develops it at half resolution
//! (mk::camera::develop_half); the house shows it letterboxed. The shutter
//! stops the stream, takes an 11 MP still with the stream's exposure,
//! develops it at full resolution, saves it to ~/Bilder/Kamera and starts the stream again.

use matrixkit_image::{Fit, ImageAtlas};
use matrixkit_scene::scene as fam;
use matrixkit_shell::{self as shell, Pict, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::camera::{self as cam, Develop, Exposure, Stream};
use matrixkit_theme::scene::{Keep, Rect};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

const APP_ID: &str = "matrix-camera";
const WIDTH: u32 = 960;
const HEIGHT: u32 = 680;
const CHROME_BASE: usize = 500;
const ROOT_BASE: usize = 700;
const SHUTTER: usize = 10;
const THUMB: usize = 11;
/// Height of the strip below the picture (shutter, last photo, status).
const STRIP: f32 = 112.0;
/// One line of the hd stream at 30 fps (frame of 8368 lines).
const HD_LINE_S: f64 = 1.0 / (30.0 * 8368.0);

/// What the worker hands to the house.
#[derive(Default)]
struct Live {
    frame: Option<(u64, u32, u32, Vec<u8>)>,
    fps: f32,
    exposure: Exposure,
    note: Option<String>,
}

enum Done {
    Saved(PathBuf),
    Failed(String),
}

struct Camera {
    palette: mk::Palette,
    chrome: fam::Chrome,
    root: fam::RootPane,
    live: Arc<Mutex<Live>>,
    stream: Arc<Mutex<Option<Stream>>>,
    /// The worker pauses fetching while a still is taken.
    paused: Arc<AtomicBool>,
    fresh: Option<(u64, u32, u32, Vec<u8>)>,
    shown: Option<(u64, u32, u32)>,
    busy: bool,
    flash: f32,
    last_photo: Option<PathBuf>,
    toast: Option<(String, Instant)>,
    done_rx: mpsc::Receiver<Done>,
    done_tx: mpsc::Sender<Done>,
    awake: Option<mk::assertion::Guard>,
}

impl Camera {
    fn new() -> Camera {
        let (done_tx, done_rx) = mpsc::channel();
        let cam = Camera {
            palette: mk::Palette::load().unwrap_or_default(),
            chrome: fam::Chrome::new("Matrix Kamera", CHROME_BASE),
            root: fam::RootPane::new(APP_ID, &[]),
            live: Arc::new(Mutex::new(Live::default())),
            stream: Arc::new(Mutex::new(None)),
            paused: Arc::new(AtomicBool::new(false)),
            fresh: None,
            shown: None,
            busy: false,
            flash: 0.0,
            last_photo: newest_photo().inspect(|p| {
                // An older photo: make its thumbnail in the background once.
                let p = p.clone();
                std::thread::spawn(move || matrixkit_image::thumbs::ensure(&p, 192, 192));
            }),
            toast: None,
            done_rx,
            done_tx,
            awake: mk::assertion::hold(mk::assertion::Kind::Display, "Kamera", 60),
        };
        start_stream(&cam.stream, &cam.live);
        spawn_worker(Arc::clone(&cam.live), Arc::clone(&cam.stream), Arc::clone(&cam.paused));
        cam
    }

    fn shoot(&mut self) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.flash = 1.0;
        let exposure = self.live.lock().map(|l| l.exposure).unwrap_or_default();
        let (stream, live, paused, tx) = (Arc::clone(&self.stream), Arc::clone(&self.live), Arc::clone(&self.paused), self.done_tx.clone());
        std::thread::spawn(move || {
            paused.store(true, Ordering::SeqCst);
            if let Ok(mut s) = stream.lock() {
                if let Some(mut s) = s.take() {
                    s.stop();
                }
            }
            let result = still(exposure);
            start_stream(&stream, &live);
            paused.store(false, Ordering::SeqCst);
            let _ = tx.send(match result {
                Ok(path) => Done::Saved(path),
                Err(e) => Done::Failed(e),
            });
        });
    }
}

/// The 11 MP still: raw → developed at full resolution (4032x2784) → PNG.
fn still(exposure: Exposure) -> Result<PathBuf, String> {
    let exposure = if exposure.lines == 0 { Exposure { lines: 4184, gain_code: 512 } } else { exposure };
    let raw = mk::runtime::path("matrix-camera-still.raw");
    let frame = cam::take_still(&raw, exposure).map_err(|e| format!("Aufnahme: {e}"))?;
    let _ = std::fs::remove_file(&raw);
    if !frame.view().sound() {
        return Err(String::from("Aufnahme unvollständig"));
    }
    // Full 11 MP, bilinear demosaic (R215 round 17).
    let (w, h, rgba) = cam::develop_full_view(frame.view(), &Develop::auto(&frame));
    let dir = photo_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Ordner: {e}"))?;
    let stamp = std::process::Command::new("date").arg("+%Y%m%d-%H%M%S").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let path = dir.join(format!("Matrix-{stamp}.png"));
    image::save_buffer(&path, &rgba, w, h, image::ExtendedColorType::Rgba8).map_err(|e| format!("Speichern: {e}"))?;
    let _ = matrixkit_image::thumbs::ensure(&path, 192, 192);
    eprintln!("[kamera] Foto {} ({w}x{h}, e {} g {})", path.display(), exposure.lines, exposure.gain_code);
    Ok(path)
}

fn photo_dir() -> PathBuf {
    let home = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp")));
    home.join("Bilder").join("Kamera")
}

fn newest_photo() -> Option<PathBuf> {
    let mut all: Vec<PathBuf> = std::fs::read_dir(photo_dir()).ok()?.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "png")).collect();
    all.sort();
    all.pop()
}

fn start_stream(stream: &Arc<Mutex<Option<Stream>>>, live: &Arc<Mutex<Live>>) {
    let started = Stream::start();
    if let Ok(mut l) = live.lock() {
        l.note = started.as_ref().err().map(|e| format!("Kamera nicht startbar ({}: {e})", cam::TOOL));
    }
    if let Ok(mut s) = stream.lock() {
        *s = started.ok();
    }
}

/// Read the newest frame where it lies (the stream's mapped ring — no
/// copy), develop it, hand it over.
fn spawn_worker(live: Arc<Mutex<Live>>, stream: Arc<Mutex<Option<Stream>>>, paused: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let mut ring: Option<cam::Shared> = None;
        let (mut last, mut dev, mut count) = (u64::MAX, None::<Develop>, 0u32);
        let mut window = (Instant::now(), 0u32);
        let (mut dev_ms, mut raced) = (0.0f32, 0u32);
        let mut copy: Vec<u8> = Vec::new();
        loop {
            if paused.load(Ordering::SeqCst) {
                ring = None;
                last = u64::MAX;
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            if ring.is_none() {
                ring = cam::Shared::connect(&cam::socket_path()).ok();
                if ring.is_none() {
                    std::thread::sleep(Duration::from_millis(150));
                    continue;
                }
            }
            let shared = ring.as_ref().unwrap();
            let Some(view) = shared.latest() else {
                std::thread::sleep(Duration::from_millis(3));
                continue;
            };
            if view.header.frame == last {
                std::thread::sleep(Duration::from_millis(3));
                continue;
            }
            let nr = view.header.frame;
            last = nr;
            // The ring is uncached memory: one sequential copy into cached
            // memory first (scattered reads cost ~450 ms a frame, measured).
            let t0 = Instant::now();
            copy.clear();
            copy.extend_from_slice(view.data);
            if shared.raced(nr) {
                raced += 1;
                continue;
            }
            let view = cam::View { header: view.header, data: &copy };
            // Round 178: a short or inconsistent frame is skipped (it panicked the develop thread).
            if !view.sound() {
                continue;
            }
            if dev.is_none() || count % 15 == 0 {
                dev = Some(Develop::auto_view(view));
            }
            count += 1;
            let (w, h, rgba) = cam::develop_half_view(view, dev.as_ref().unwrap());
            dev_ms += t0.elapsed().as_secs_f32() * 1000.0;
            window.1 += 1;
            let elapsed = window.0.elapsed().as_secs_f32();
            let exposure = stream.lock().ok().and_then(|s| s.as_ref().map(|s| s.exposure())).unwrap_or_default();
            if let Ok(mut l) = live.lock() {
                l.frame = Some((nr, w, h, rgba));
                l.exposure = exposure;
                if elapsed >= 1.0 {
                    l.fps = window.1 as f32 / elapsed;
                }
            }
            if elapsed >= 1.0 {
                let n = window.1.max(1) as f32;
                if std::env::var_os("MATRIX_TAKT").is_some() {
                    eprintln!("[kamera] {:.1} fps, entwickeln {:.1} ms, überholt {raced}", n / elapsed, dev_ms / n);
                }
                window = (Instant::now(), 0);
                (dev_ms, raced) = (0.0, 0);
            }
        }
    });
}

fn status(fps: f32, e: Exposure) -> String {
    if e.lines == 0 {
        return String::from("Kamera startet …");
    }
    let ms = e.lines as f64 * HD_LINE_S * 1000.0;
    let gain = 1024.0 / (1024.0 - e.gain_code.min(960) as f64);
    format!("Live · {fps:.0} fps · {ms:.0} ms · {gain:.1}×").replace('.', ",")
}

impl shell::Client for Camera {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn base(&mut self, _atlas: &mut ImageAtlas, buf: &mut [u8], _w: u32, _h: u32) {
        for px in buf.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
    }

    fn animating(&self) -> bool {
        true
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        let mut dirty = self.chrome.glide(dt) | self.root.glide(dt);
        if let Ok(mut l) = self.live.lock() {
            if let Some(f) = l.frame.take() {
                self.fresh = Some(f);
                dirty = true;
            }
        }
        if self.flash > 0.0 {
            self.flash = (self.flash - dt * 3.0).max(0.0);
            dirty = true;
        }
        if let Ok(done) = self.done_rx.try_recv() {
            self.busy = false;
            match done {
                Done::Saved(p) => {
                    let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    self.toast = Some((format!("Gespeichert: Bilder/Kamera/{name}"), Instant::now()));
                    self.last_photo = Some(p);
                }
                Done::Failed(e) => self.toast = Some((format!("Foto misslungen — {e}"), Instant::now())),
            }
            dirty = true;
        }
        if self.toast.as_ref().is_some_and(|t| t.1.elapsed() > Duration::from_secs(4)) {
            self.toast = None;
            dirty = true;
        }
        if dirty {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let p = self.palette;
        let (wf, hf) = (w as f32, h as f32);
        let mut b = SceneBuilder::new();
        let top = self.chrome.build(engine, &mut b, w, h, p);
        if self.chrome.rolled {
            return b.finish();
        }
        let stage = Rect { x: 0.0, y: top, w: wf, h: (hf - top - STRIP).max(1.0) };
        let first = b.quad(stage.x, stage.y, stage.w, stage.h, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, 0.0, None, None);
        if let Some((nr, fw, fh, bytes)) = self.fresh.take() {
            atlas.put_frame(&PathBuf::from(format!("\u{0}kamera:{nr}")), fw, fh, bytes, false);
            self.shown = Some((nr, fw, fh));
        }
        let (fps, exposure, note) = self.live.lock().map(|l| (l.fps, l.exposure, l.note.clone())).unwrap_or_default();
        if let Some((nr, fw, fh)) = self.shown {
            let fit = fam::letterbox(stage, fw, fh);
            b.pict(Pict {
                path: PathBuf::from(format!("\u{0}kamera:{nr}")),
                box_w: fit.w as u32,
                box_h: fit.h as u32,
                fit: Fit::Stretch,
                x: fit.x,
                y: fit.y,
                radius: 0.0,
                home: stage,
                anchor: first,
                clip: None,
            });
        } else {
            let s = note.as_deref().unwrap_or("Kamera startet …");
            let sub = mk::typo::SUBTITLE;
            let (lw, _) = engine.measure_styled(s, sub.px(), sub.weight);
            b.styled(engine, s, sub, mk::Rgba { r: 0.85, g: 0.85, b: 0.85, a: 1.0 }, (wf - lw) / 2.0, stage.y + stage.h / 2.0, None, None);
        }
        // The flash of the release lies over the picture for a moment.
        if self.flash > 0.0 {
            b.quad(stage.x, stage.y, stage.w, stage.h, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: self.flash * 0.6 }, 0.0, None, None);
        }

        // The strip: last photo left, shutter centre, status right.
        let sy = hf - STRIP;
        b.quad(0.0, sy, wf, STRIP, p.surface, 0.0, None, None);
        let shutter = 72.0;
        fam::lay_shutter(&mut b, wf / 2.0, sy + STRIP / 2.0, shutter, self.busy, self.flash, Some(SHUTTER), None, p);
        let thumb = 64.0;
        if let Some(photo) = &self.last_photo {
            let home = Rect { x: 24.0, y: sy + (STRIP - thumb) / 2.0, w: thumb, h: thumb };
            let anchor = b.quad(home.x, home.y, home.w, home.h, p.surface_container_high, 12.0, Some(THUMB), None);
            // Only the small thumbnail (matrixkit_image::thumbs) — never the
            // 11-MP original in the frame (round 26; same lesson as the
            // wallpaper gallery). Made right after saving, off the surface.
            let small = matrixkit_image::thumbs::path_for(photo, 192, 192);
            if small.is_file() {
                b.pict(Pict { path: small, box_w: thumb as u32, box_h: thumb as u32, fit: Fit::Cover, x: home.x, y: home.y, radius: 12.0, home, anchor, clip: None });
            }
        }
        let line = self.toast.as_ref().map(|t| t.0.clone()).unwrap_or_else(|| if self.busy { String::from("Foto wird aufgenommen …") } else { status(fps, exposure) });
        let cap = mk::typo::BODY;
        let (lw, lh) = engine.measure_styled(&line, cap.px(), cap.weight);
        let right = wf / 2.0 + shutter / 2.0 + 24.0;
        let x = (wf - 24.0 - lw).max(right);
        b.styled(engine, &line, cap, p.on_surface_variant, x, sy + (STRIP - lh) / 2.0, None, None);

        let about = fam::RootAbout {
            name: "Matrix Kamera",
            version: env!("CARGO_PKG_VERSION"),
            description: "Die Duo-Kamera ohne Android: Rohbild aus dem Kernel, eigene Belichtung, eigene Entwicklung. Fotos in Bilder/Kamera.",
        };
        self.root.lay(engine, &mut b, &about, wf, hf, p, ROOT_BASE);
        b.finish()
    }

    fn zoomed(&mut self, on: bool) {
        self.chrome.set_zoomed(on);
    }

    fn extent(&mut self) -> Option<(u32, u32)> {
        self.chrome.extent()
    }

    fn headline(&self) -> f32 {
        self.chrome.headline()
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, source: Source) -> Refresh {
        if let Some(deed) = self.root.strike_at(role, ROOT_BASE) {
            return match deed {
                fam::RootDeed::Help => {
                    matrixkit_scene::help_open("Matrix Kamera");
                    Refresh::Calm
                }
                fam::RootDeed::Calm => Refresh::Calm,
                _ => Refresh::Dirty,
            };
        }
        match self.chrome.strike(role) {
            Some(fam::ChromeDeed::Close) => return Refresh::Close,
            Some(fam::ChromeDeed::Title) => {
                self.root.toggle();
                return Refresh::Dirty;
            }
            Some(fam::ChromeDeed::Answer(r)) => return r,
            None => {}
        }
        match role {
            Some(SHUTTER) => {
                eprintln!("[kamera] Auslöser ({})", source.label());
                self.shoot();
                Refresh::Dirty
            }
            Some(THUMB) => {
                if let Some(p) = &self.last_photo {
                    // Round 178: the kit's opener (xdg-open is missing on the Duo; the child was never reaped).
                    mk::open::open(&p.to_string_lossy());
                }
                Refresh::Calm
            }
            _ => Refresh::Calm,
        }
    }

    fn root(&mut self) -> Refresh {
        if self.root.open() {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn closing(&mut self) {
        self.paused.store(true, Ordering::SeqCst);
        if let Ok(mut s) = self.stream.lock() {
            if let Some(mut s) = s.take() {
                s.stop();
            }
        }
        self.awake = None;
    }

    fn depart(&mut self) -> Refresh {
        self.chrome.depart()
    }

    fn motion(&mut self, x: f64, y: f64) -> Refresh {
        self.chrome.motion(x, y)
    }

    fn key(&mut self, key: shell::Key, shift: bool) -> Refresh {
        if self.root.key(&key, shift) {
            return Refresh::Dirty;
        }
        if matches!(&key, shell::Key::Char(c) if c == " ") {
            self.shoot();
            return Refresh::Dirty;
        }
        Refresh::Calm
    }

    fn pulse(&mut self) -> Refresh {
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
            self.palette = fresh;
            return Refresh::Full;
        }
        Refresh::Calm
    }
}

fn main() {
    shell::prefer_gpu();
    eprintln!("[kamera] Matrix Kamera, pid={}", std::process::id());
    let result = fam::run(
        Camera::new(),
        shell::Options {
            mount: shell::Mount::Window { app_id: String::from(APP_ID), title: String::from("Matrix Kamera"), width: WIDTH, height: HEIGHT },
            argb: false,
            tag: "camera",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    );
    if let Err(e) = result {
        eprintln!("[kamera] Fensterhaus-Fehler: {e}");
        std::process::exit(1);
    }
}
