//! Matrix Player, First Light version (iced replacement, camp 46 —
//! GRABUNG-ICED-ABLOESUNG.md), on the shared families since R210:
//!
//! * **`matrixkit-video`** runs picture AND sound (one ffmpeg, `-re`,
//!   picture as RGBA into the pipe, sound to PipeWire) — the reminder «TON»
//!   from camp 46 is closed with that. Pause = SIGSTOP, seek = restart.
//! * **`Fit::Stretch`**: the picture travels at most 1280 px wide and the GPU
//!   stretches it onto the stage (letterboxed on black velvet).
//! * **`fam::Chrome`** carries the window skin (traffic lights, title, roll-up,
//!   maximise), **`SceneBuilder`** builds the scene without index arithmetic.
//!
//! The house's FRAME CLOCK is the video clock: animating() while it
//! plays, glide() takes the newest frame, view() puts it into the atlas.
//! The TIMELINE: dragging seeks, release jumps. The wake assertion
//! (mk::assertion, Display) keeps the screen lit while it plays.

use crate::newest_capture;
use matrixkit_image::{Fit, ImageAtlas};
use matrixkit_scene::scene as fam;
use matrixkit_shell::{self as shell, Pict, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{Keep, Rect};
use matrixkit_video::{Spec, Video};
use std::path::PathBuf;

// Size like the reference (mkw::app::window in the iced gate).
const WIDTH: u32 = 960;
const HEIGHT: u32 = 680;
const CHROME_BASE: usize = 500;
/// Roles from here on belong to the root layer (camp 94).
const ROOT_BASE: usize = 700;
/// The roles of the playback bar (`fam::MediaBar`: button, timeline).
const BAR_BASE: usize = 0;
/// Widest picture in the pipe — the GPU stretches it onto the stage
/// (YUV, 1.5 bytes per pixel: up to 1080p without downscaling).
const PIPE_WIDTH: u32 = 1920;

struct Player {
    video: Option<Video>,
    /// Round 135: the file playing and its still (lock screen, bar).
    file: Option<std::path::PathBuf>,
    poster: Option<String>,
    poster_asked: bool,
    awake: Option<mk::assertion::Guard>,
    palette: mk::Palette,
    chrome: fam::Chrome,
    root: fam::RootPane,
    /// The newest frame due for the atlas, and the one shown.
    fresh: Option<(u64, u32, u32, Vec<u8>)>,
    shown: Option<u64>,
    /// ▶/⏸, timeline with buffer shadow, times (R210 family).
    bar: fam::MediaBar,
    meter: shell::FrameMeter,
    /// Subtitles next to the file (`film.srt`/`film.vtt`, R210 family).
    cues: Vec<matrixkit_video::subtitles::Cue>,
    /// Cinema mode (`mk::cinema::Announcer`): the backgrounds dim.
    kino: mk::cinema::Announcer,
    /// On the Duo the window takes the shape of the picture (+ bar).
    picture_fit: fam::PictureFit,
}

impl Player {
    /// Cinema mode follows the running picture.
    fn cinema(&mut self) {
        let playing = self.video.as_ref().is_some_and(|v| !v.paused() && !v.done());
        self.kino.follow(playing);
    }
}

impl Player {
    fn new() -> Self {
        let path = std::env::args()
            .filter(|a| a != "--erstlicht")
            .nth(1)
            .map(PathBuf::from)
            .or_else(newest_capture);
        let cues = path
            .as_deref()
            .and_then(matrixkit_video::subtitles::sidecar)
            .map(|(file, cues)| {
                eprintln!("[player] Untertitel {} ({} Zeilen)", file.display(), cues.len());
                cues
            })
            .unwrap_or_default();
        let video = path.as_deref().and_then(matrixkit_video::probe_file).map(|media| {
            Video::new(
                media,
                // R210: YUV frames, converted by the GPU (CPU house: by the atlas).
                Spec {
                    width: PIPE_WIDTH,
                    exact: None,
                    sound_name: String::from("Matrix Player"),
                    sound_only: false,
                    max_fps: Some(mk::canvas::refresh_hz().unwrap_or(60.0).min(60.0)),
                    yuv: std::env::var_os("MATRIX_CPU").is_none(),
                },
            )
        });
        Player {
            video,
            file: path.clone(),
            poster: None,
            poster_asked: false,
            awake: None,
            palette: mk::Palette::load().unwrap_or_default(),
            chrome: fam::Chrome::new("Matrix Player", CHROME_BASE),
            root: fam::RootPane::new(crate::APP_ID, &[]),
            fresh: None,
            shown: None,
            bar: fam::MediaBar::new(BAR_BASE),
            meter: shell::FrameMeter::new("player"),
            cues,
            kino: mk::cinema::Announcer::new(),
            picture_fit: fam::PictureFit::new("matrix-player"),
        }
    }

    fn play(&mut self, from_s: f64) {
        let Some(video) = &mut self.video else { return };
        video.play(from_s);
        self.awake = mk::assertion::hold(mk::assertion::Kind::Display, &video.media.title, 60);
        eprintln!("[player] dreht ab {from_s:.1} s (Bild und Ton)");
    }
}

impl shell::Client for Player {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
        // The film starts with the house.
        self.play(0.0);
    }

    fn base(&mut self, _atlas: &mut ImageAtlas, buf: &mut [u8], _w: u32, _h: u32) {
        let p = self.palette;
        let (b, g, r) = ((p.surface.b * 255.0) as u8, (p.surface.g * 255.0) as u8, (p.surface.r * 255.0) as u8);
        for px in buf.chunks_exact_mut(4) {
            px[0] = b;
            px[1] = g;
            px[2] = r;
            px[3] = 0xff;
        }
    }

    fn animating(&self) -> bool {
        self.root.animating() || self.chrome.animating() || self.video.as_ref().is_some_and(|v| v.running())
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        self.cinema();
        let mut dirty = self.chrome.glide(dt);
        dirty |= self.root.glide(dt);
        if let Some(video) = &self.video {
            if let Some(frame) = video.take_frame(self.fresh.as_ref().map(|f| f.0).or(self.shown)) {
                self.meter.frame(frame.0);
                self.fresh = Some(frame);
                dirty = true;
            }
            // The clock log only on request (MATRIX_TAKT=1).
            if std::env::var_os("MATRIX_TAKT").is_some() {
                if let Some(line) = self.meter.report(&format!("; Uhr {:.1} s", video.position_s())) {
                    eprintln!("{line}");
                }
            }
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

        // The stage: black velvet, the frame stretched on it (letterbox).
        let bar_h = fam::MediaBar::HEIGHT;
        let stage = Rect { x: 0.0, y: top, w: wf, h: hf - top - bar_h };
        // Duo: the window takes the picture's shape, the bar stays below.
        let picture = self.video.as_ref().map(|v| (v.media.width, v.media.height));
        self.picture_fit.follow(wf, hf, picture, top + bar_h, 0.0);
        let stage_first = b.quad(stage.x, stage.y, stage.w, stage.h, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, 0.0, Some(BAR_BASE), None);
        if let Some((nr, fw, fh, bytes)) = self.fresh.take() {
            let key = PathBuf::from(format!("\u{0}film:{nr}"));
            match self.video.as_ref() {
                Some(v) => v.lay_frame(atlas, &key, fw, fh, bytes),
                None => atlas.put_frame(&key, fw, fh, bytes, false),
            }
            self.shown = Some(nr);
        }
        if let (Some(nr), Some(video)) = (self.shown, &self.video) {
            let fit = fam::letterbox(stage, video.media.width, video.media.height);
            b.pict(Pict {
                path: PathBuf::from(format!("\u{0}film:{nr}")),
                box_w: fit.w as u32,
                box_h: fit.h as u32,
                fit: Fit::Stretch,
                x: fit.x,
                y: fit.y,
                radius: 0.0,
                home: stage,
                anchor: stage_first,
                clip: None,
            });
            if let Some(cue) = matrixkit_video::subtitles::cue_at(&self.cues, video.position_s()) {
                fam::lay_subtitle(&mut b, engine, fit, &cue.text, Some(BAR_BASE));
            }
        }

        // The bar: ▶/⏸, timeline with buffer shadow, times.
        if let Some(video) = &self.video {
            let view = fam::MediaView {
                prev: None,
                next: None,
                position: video.position_s() as f32,
                duration: video.media.duration_s as f32,
                playing: !video.paused() && !video.done(),
                ahead: video.stocked_s(),
                marks: &[],
            };
            self.bar.build(engine, &mut b, hf - bar_h, wf, &view, p);
        } else {
            let s = "Kein Video — matrix-player <Datei>";
            let sub = mk::typo::SUBTITLE;
            let (lw, _) = engine.measure_styled(s, sub.px(), sub.weight);
            b.styled(engine, s, sub, p.on_surface, (wf - lw) / 2.0, hf / 2.0, None, None);
        }

        // The ROOT LAYER lies ABOVE everything (camp 94).
        let about = fam::RootAbout {
            name: "Matrix Player",
            version: env!("CARGO_PKG_VERSION"),
            description: "Videos ansehen — Bildschirmaufnahmen zuerst. Der Decoder läuft außer Prozess, mit Ton.",
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

    fn strike(&mut self, role: Option<usize>, x: f64, _y: f64, source: Source) -> Refresh {
        let origin = source.label();
        // Open root layer: only its roles count (camp 94).
        if let Some(deed) = self.root.strike_at(role, ROOT_BASE) {
            return match deed {
                fam::RootDeed::Help => {
                    matrixkit_scene::help_open("Matrix Player");
                    Refresh::Calm
                }
                fam::RootDeed::Calm => Refresh::Calm,
                _ => Refresh::Dirty,
            };
        }
        match self.chrome.strike(role) {
            Some(fam::ChromeDeed::Close) => {
                eprintln!("[player] Treffer ({origin}): Ampel × — der Player schließt");
                if let Some(mut v) = self.video.take() {
                    v.stop();
                }
                return Refresh::Close;
            }
            Some(fam::ChromeDeed::Title) => {
                self.root.toggle();
                return Refresh::Dirty;
            }
            Some(fam::ChromeDeed::Answer(r)) => return r,
            None => {}
        }
        match self.bar.strike(role, x as f32) {
            Some(fam::MediaDeed::Scrub) => Refresh::Dirty,
            // The player shows no skip keys (MediaView prev/next None).
            Some(fam::MediaDeed::Prev | fam::MediaDeed::Next) => Refresh::Calm,
            Some(fam::MediaDeed::Toggle) => {
                let Some(video) = &mut self.video else { return Refresh::Calm };
                if video.done() {
                    eprintln!("[player] Treffer ({origin}): von vorn");
                    self.play(0.0);
                    return Refresh::Dirty;
                }
                let paused = !video.paused();
                video.set_paused(paused);
                if paused {
                    self.awake = None;
                } else {
                    self.awake = mk::assertion::hold(mk::assertion::Kind::Display, &video.media.title, 60);
                }
                eprintln!("[player] Treffer ({origin}): {}", if paused { "Pause" } else { "weiter" });
                Refresh::Dirty
            }
            None => Refresh::Calm,
        }
    }

    fn travel(&mut self, x: f64, _y: f64, _source: Source) -> Refresh {
        if self.bar.travel(x as f32) {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn release(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        let Some(target) = self.bar.release() else { return Refresh::Calm };
        eprintln!("[player] Sprung zu {target:.1} s (Loslass)");
        self.play(target as f64);
        Refresh::Dirty
    }

    /// The bar asks for the settings (Duo grammar R210).
    fn root(&mut self) -> Refresh {
        if self.root.open() {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    /// Another app in front on the stage's panel: the fitted stage steps
    /// back into the tiled layout; focus back, it takes its picture shape.
    fn focus(&mut self, on: bool) -> Refresh {
        if self.picture_fit.focus(on) {
            Refresh::Full
        } else {
            Refresh::Calm
        }
    }

    fn closing(&mut self) {
        self.kino.follow(false);
        mk::now_playing::clear();
    }


    fn depart(&mut self) -> Refresh {
        self.chrome.depart()
    }

    fn motion(&mut self, x: f64, y: f64) -> Refresh {
        self.chrome.motion(x, y)
    }

    fn beat(&mut self) -> Refresh {
        now_playing(self);
        self.cinema();
        let _ = self.root.beat();
        // Keep the time display current in the paused/ended state.
        Refresh::Dirty
    }

    fn key(&mut self, key: shell::Key, shift: bool) -> Refresh {
        // The open root layer captures the keyboard (camp 94).
        if self.root.key(&key, shift) {
            return Refresh::Dirty;
        }
        Refresh::Calm
    }

    fn pulse(&mut self) -> Refresh {
        // Round 178: collect decoders that ended by themselves (no zombies on the end screen).
        if let Some(v) = self.video.as_mut() {
            v.reap();
        }
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
            self.palette = fresh;
            return Refresh::Full;
        }
        Refresh::Calm
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[player] Matrix Player: Bewegtbild mit Ton auf den gemeinsamen Familien, pid={}", std::process::id());
    fam::run(
        Player::new(),
        shell::Options {
            mount: shell::Mount::Window {
                app_id: String::from("matrix-player"),
                title: String::from("Matrix Player"),
                width: WIDTH,
                height: HEIGHT,
            },
            argb: false,
            tag: "player",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}

/// Round 132: what plays goes to the kit board (the lock screen's
/// player, the bar) and the remotes' commands come back.
fn now_playing(player: &mut Player) {
    use mk::now_playing as np;
    // Round 135: a still of the file once (cut off the window's thread).
    if player.poster.is_none() {
        if let Some(file) = player.file.clone() {
            match matrixkit_video::poster::cached(&file) {
                Some(p) => player.poster = Some(p.to_string_lossy().into_owned()),
                None if !player.poster_asked => {
                    player.poster_asked = true;
                    let d = player.video.as_ref().map(|v| v.media.duration_s).unwrap_or(0.0);
                    std::thread::spawn(move || matrixkit_video::poster::make(&file, d));
                }
                None => {}
            }
        }
    }
    let Some(video) = player.video.as_mut() else {
        np::clear();
        return;
    };
    for cmd in np::take() {
        match cmd {
            np::Command::Toggle => {
                if video.done() {
                    video.play(0.0);
                } else {
                    let p = !video.paused();
                    video.set_paused(p);
                }
            }
            np::Command::Seek(t) => video.play(t),
            np::Command::Previous => video.play(0.0),
            np::Command::Next => {}
        }
    }
    np::publish(np::Playing {
        app: String::from("Matrix Player"),
        title: video.media.title.clone(),
        artist: String::new(),
        position_s: video.position_s(),
        duration_s: video.media.duration_s,
        playing: !video.paused() && !video.done(),
        art: player.poster.clone(),
        pid: 0,
        at_ms: 0,
        can_next: false,
        can_previous: true,
    });
}
