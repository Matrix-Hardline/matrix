//! Matrix Player — App #30 (R71): Videos ansehen, Bildschirmaufnahmen zuerst.
//!
//! The reference blueprint: the media reference layer does not decode
//! in the app but out of process (a media service process) and hands
//! finished frames over. Here ffmpeg takes that role — as a child
//! process pushing raw RGBA frames through a pipe. The window itself
//! Chrome, Palette, Familien, Player-Grammatik (Play/Pause, Zeitleiste,
//! mm:ss-Anzeigen).
//!
//! Without an argument the player opens the newest screen recording —
//! the shortest path to "did my recording work?".

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, container, mouse_area, row, Space};
use matrixkit_widgets::iced::{Element, Length, Subscription, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use mkw::color;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

mod licht;

const APP_ID: &str = "matrix-player";
/// Decode width: ffmpeg scales down, the pipe stays slim.
/// (4K recordings would be ~1 GB/s raw — this way ~70 MB/s.)
const DECODE_WIDTH: u32 = 1024;

fn main() -> iced::Result {
    // R210: the STANDARD CHANGE (like the terminal, Lager 95) — the light
    // house is the normal path since it plays sound (matrixkit-video) and
    // stretches on the GPU. The iced path stays as fallback (`--iced`), and
    // as safety net: if the light cannot START, iced takes over.
    // `--erstlicht` stays accepted (scripts and habits).
    if !std::env::args().any(|a| a == "--iced") {
        matrixkit_shell::prefer_gpu();
        match licht::run() {
            Ok(()) => return Ok(()),
            Err(e) => eprintln!("[player] Fensterhaus-Fehler ({e}) — Rückfall auf iced"),
        }
    }
    mkw::app::window::<App>(APP_ID, 960.0, 680.0)
}

impl mkw::app::Program for App {
    type Msg = Msg;
    fn new() -> (Self, Task<Msg>) {
        App::new()
    }
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        App::update(self, msg)
    }
    fn view(&self) -> Element<'_, Msg> {
        App::view(self)
    }
    fn title(&self) -> String {
        match &self.film {
            Some(f) => format!("Matrix Player — {}", f.name()),
            None => String::from("Matrix Player"),
        }
    }
    fn subscription(&self) -> iced::Subscription<Msg> {
        App::subscription(self)
    }
}

// ------------------------------------------------------------- the movie

/// Profile from ffprobe: what lies there, how long, how fast.
#[derive(Debug, Clone)]
pub(crate) struct Movie {
    pub(crate) path: PathBuf,
    pub(crate) duration_s: f64,
    pub(crate) fps: f64,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Movie {
    pub(crate) fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Decode measures: width fixed, height from the aspect ratio, rounded
    /// even — MUST match the ffmpeg scaling exactly (frame size!).
    pub(crate) fn decode_masse(&self) -> (u32, u32) {
        let b = DECODE_WIDTH.min(self.width).max(2) & !1;
        let h = ((self.height as f64 * b as f64 / self.width as f64).round() as u32).max(2) & !1;
        (b, h)
    }
}

/// Query ffprobe — duration, measures, frame rate.
pub(crate) fn probe(path: &Path) -> Option<Movie> {
    let out = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,avg_frame_rate:format=duration",
            "-of", "default=nw=1",
        ])
        .arg(path)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut width = 0u32;
    let mut height = 0u32;
    let mut fps = 0f64;
    let mut duration = 0f64;
    for line in text.lines() {
        if let Some(w) = line.strip_prefix("width=") {
            width = w.trim().parse().unwrap_or(0);
        } else if let Some(w) = line.strip_prefix("height=") {
            height = w.trim().parse().unwrap_or(0);
        } else if let Some(w) = line.strip_prefix("duration=") {
            duration = w.trim().parse().unwrap_or(0.0);
        } else if let Some(w) = line.strip_prefix("avg_frame_rate=") {
            let mut parts = w.trim().split('/');
            let z: f64 = parts.next().and_then(|t| t.parse().ok()).unwrap_or(0.0);
            let n: f64 = parts.next().and_then(|t| t.parse().ok()).unwrap_or(1.0);
            if n > 0.0 && z > 0.0 {
                fps = z / n;
            }
        }
    }
    if width == 0 || height == 0 || duration <= 0.0 {
        return None;
    }
    Some(Movie {
        path: path.to_path_buf(),
        duration_s: duration,
        fps: if fps > 0.0 { fps } else { 30.0 },
        width,
        height,
    })
}

/// Without an argument: the newest screen recording.
pub(crate) fn newest_capture() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let folder = PathBuf::from(home).join("Videos/Bildschirmaufnahmen");
    let mut videos: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(&folder)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            let mp4 = p
                .extension()
                .map(|x| matches!(x.to_str(), Some("mp4" | "webm" | "mkv" | "mov")))
                .unwrap_or(false);
            if !mp4 {
                return None;
            }
            Some((e.metadata().ok()?.modified().ok()?, p))
        })
        .collect();
    videos.sort();
    videos.pop().map(|(_, p)| p)
}

// --------------------------------------------------------- the deck

/// Shared state between decoder thread and window.
pub(crate) struct Deck {
    /// Youngest frame: (running number, width, height, RGBA).
    pub(crate) image: Mutex<Option<(u64, u32, u32, Vec<u8>)>>,
    /// Abspielposition in Millisekunden.
    pub(crate) pos_ms: AtomicU64,
    /// Movie finished (the thread read EOF).
    pub(crate) done: AtomicBool,
    /// Pause = the thread does not read; the full pipe stalls ffmpeg.
    pub(crate) paused: AtomicBool,
    /// Every restart (seek) counts up; old threads recognize themselves by it.
    pub(crate) generation: AtomicU64,
}

pub(crate) fn decode(
    run: Arc<Deck>,
    mut pipe: std::process::ChildStdout,
    gen: u64,
    from_s: f64,
    fps: f64,
    b: u32,
    h: u32,
) {
    let frame_len = (b * h * 4) as usize;
    let mut puffer = vec![0u8; frame_len];
    let frame_duration = Duration::from_secs_f64(1.0 / fps);
    let mut next = Instant::now();
    let mut nr: u64 = 0;
    loop {
        if run.generation.load(Ordering::SeqCst) != gen {
            return; // a seek has taken over
        }
        if run.paused.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(40));
            next = Instant::now();
            continue;
        }
        if pipe.read_exact(&mut puffer).is_err() {
            // EOF: movie finished — unless we were superseded.
            if run.generation.load(Ordering::SeqCst) == gen {
                run.done.store(true, Ordering::SeqCst);
                run.paused.store(true, Ordering::SeqCst);
            }
            return;
        }
        nr += 1;
        let pos = (from_s * 1000.0) as u64 + (nr as f64 * 1000.0 / fps) as u64;
        run.pos_ms.store(pos, Ordering::SeqCst);
        *run.image.lock().unwrap_or_else(|e| e.into_inner()) = Some((gen * 1_000_000 + nr, b, h, puffer.clone()));
        // Pace: throttle to frame rate; if far behind, catch up.
        next += frame_duration;
        let now = Instant::now();
        if next > now {
            std::thread::sleep(next - now);
        } else if now - next > Duration::from_millis(300) {
            next = now;
        }
    }
}

// ------------------------------------------------------------- the app

struct App {
    frame: mkw::Frame,
    /// R158: held WHILE playing — the screen must not sleep during a
    /// film, and the governor must not nap us. Dropping it releases,
    /// so a pause or a crash frees the machine by itself.
    awake: Option<mk::assertion::Guard>,
    film: Option<Movie>,
    run: Arc<Deck>,
    child: Arc<Mutex<Option<Child>>>,
    /// Most recently uploaded frame (frame number + GPU handle).
    handle: Option<iced::widget::image::Handle>,
    /// The previous handle lives one tick longer: its death would
    /// otherwise free the texture while it is still on screen.
    handle_old: Option<iced::widget::image::Handle>,
    last_nr: u64,
    /// While dragging the timeline: target position instead of live position.
    dragging: Option<f32>,
}

#[derive(Debug, Clone)]
enum Msg {
    Frame(mkw::FrameMsg),
    Key(mkw::Key),
    Tick,
    PlayPause,
    Drag(f32),
    Jump,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let path = std::env::args()
            .nth(1)
            .map(PathBuf::from)
            .or_else(newest_capture);
        let film = path.as_deref().and_then(probe);
        let mut app = Self {
            frame: mkw::Frame::new(APP_ID, &[]),
            awake: None,
            film,
            run: Arc::new(Deck {
                image: Mutex::new(None),
                pos_ms: AtomicU64::new(0),
                done: AtomicBool::new(false),
                paused: AtomicBool::new(false),
                generation: AtomicU64::new(0),
            }),
            child: Arc::new(Mutex::new(None)),
            handle: None,
            handle_old: None,
            last_nr: 0,
            dragging: None,
        };
        if app.film.is_some() {
            app.play(0.0);
            app.assert_while_playing();
        }
        (app, Task::none())
    }

    /// Start the decoder (anew) — from second `from_s`. The old child is
    /// ended; its thread recognizes the new generation and leaves.
    /// R158 (the assertion dig): declare busy while a film runs, and
    /// let go the moment it pauses or ends. The deadline is Apple's
    /// safety net — a crashed player frees the screen within a minute
    /// even though nobody released anything.
    fn assert_while_playing(&mut self) {
        let playing = self.film.is_some()
            && !self.run.paused.load(Ordering::SeqCst)
            && !self.run.done.load(Ordering::SeqCst);
        if playing {
            if self.awake.is_none() {
                let what = self
                    .film
                    .as_ref()
                    .map(|f| format!("Video läuft: {}", f.name()))
                    .unwrap_or_else(|| String::from("Video läuft"));
                self.awake = mk::assertion::hold(mk::assertion::Kind::Display, &what, 60);
            }
        } else {
            self.awake = None;
        }
    }

    fn play(&self, from_s: f64) {
        let Some(film) = &self.film else { return };
        let gen = self.run.generation.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(mut old) = self.child.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = old.kill();
            let _ = old.wait();
        }
        self.run.done.store(false, Ordering::SeqCst);
        self.run.paused.store(false, Ordering::SeqCst);
        self.run
            .pos_ms
            .store((from_s * 1000.0) as u64, Ordering::SeqCst);
        let (b, h) = film.decode_masse();
        let Ok(mut child) = Command::new("ffmpeg")
            .args(["-v", "error", "-ss"])
            .arg(format!("{from_s:.3}"))
            .arg("-i")
            .arg(&film.path)
            .args(["-vf", &format!("scale={b}:{h}"), "-f", "rawvideo", "-pix_fmt", "rgba", "-"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        else {
            return;
        };
        let pipe = child.stdout.take();
        *self.child.lock().unwrap_or_else(|e| e.into_inner()) = Some(child);
        if let Some(pipe) = pipe {
            let run = Arc::clone(&self.run);
            let fps = film.fps;
            std::thread::spawn(move || decode(run, pipe, gen, from_s, fps, b, h));
        }
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Frame(m) => self.frame.update(m).map(Msg::Frame),
            Msg::Key(t) => {
                if self.frame.handle_key(t) {
                    return Task::none();
                }
                if matches!(t, mkw::Key::Activate) {
                    return self.update(Msg::PlayPause);
                }
                Task::none()
            }
            Msg::Tick => {
                // Lift the youngest frame onto the GPU — only if it is new.
                if let Some((nr, b, h, rgba)) = self.run.image.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                    if *nr != self.last_nr {
                        self.last_nr = *nr;
                        self.handle_old = self.handle.take();
                        self.handle = Some(iced::widget::image::Handle::from_rgba(
                            *b,
                            *h,
                            rgba.clone(),
                        ));
                    }
                }
                // R158: renew the assertion well before its 60 s
                // deadline, and drop it the moment the film ends.
                self.assert_while_playing();
                Task::none()
            }
            Msg::PlayPause => {
                if self.run.done.load(Ordering::SeqCst) {
                    // Finished: from the start (replay).
                    self.play(0.0);
                } else {
                    let new = !self.run.paused.load(Ordering::SeqCst);
                    self.run.paused.store(new, Ordering::SeqCst);
                }
                self.assert_while_playing();
                Task::none()
            }
            Msg::Drag(v) => {
                self.dragging = Some(v);
                Task::none()
            }
            Msg::Jump => {
                if let Some(target) = self.dragging.take() {
                    self.play(target as f64);
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Msg> {
        let mut subs = vec![
            self.frame.subscription().map(Msg::Frame),
            mkw::key_subscription(Msg::Key),
        ];
        // Frame pulse only while playing — pause costs nothing.
        if self.film.is_some() && !self.run.paused.load(Ordering::SeqCst) {
            subs.push(mkw::tick("player", Duration::from_millis(33)).map(|_| Msg::Tick));
        }
        Subscription::batch(subs)
    }

    fn view(&self) -> Element<'_, Msg> {
        let p = self.frame.palette;

        let content: Element<'_, Msg> = match &self.film {
            None => mkw::empty_state(
                mkw::symbol::PLAY_ARROW,
                "Kein Video",
                "matrix-player <Datei> — oder erst eine Bildschirmaufnahme machen (Mod+Shift+5).",
                p,
            ),
            Some(film) => {
                let duration = film.duration_s as f32;
                let done = self.run.done.load(Ordering::SeqCst);
                let paused = self.run.paused.load(Ordering::SeqCst);
                let pos = self
                    .dragging
                    .unwrap_or(self.run.pos_ms.load(Ordering::SeqCst) as f32 / 1000.0)
                    .min(duration);

                // The stage: the frame, centered, on black velvet. The previous
                // frame lies underneath in the stack: if the renderer loses the
                // new image for one tick (NVIDIA texture swap), the previous
                // one shows instead of black.
                let image = |h: &iced::widget::image::Handle| {
                    iced::widget::image(h.clone())
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .content_fit(iced::ContentFit::Contain)
                };
                let stage: Element<'_, Msg> = match (&self.handle, &self.handle_old) {
                    (Some(new), Some(old)) => iced::widget::stack![image(old), image(new)]
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into(),
                    (Some(new), None) => image(new).into(),
                    _ => Space::new().width(Length::Fill).height(Length::Fill).into(),
                };
                let stage = container(stage)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(1.0)
                    .style(|_| container::Style {
                        background: Some(iced::Color::BLACK.into()),
                        border: iced::Border {
                            radius: mk::radius::NORMAL.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    });

                // Player reference-design row: play/pause, time, bar, remaining time.
                let glyph = if done {
                    mkw::symbol::RESTART
                } else if paused {
                    mkw::symbol::PLAY_ARROW
                } else {
                    mkw::symbol::PAUSE
                };
                let button_sym: Element<'_, Msg> =
                    container(mkw::symbol::<Msg>(glyph, mk::icon_size::NORMAL, p.on_surface))
                        .padding(4.0)
                        .into();
                let button = mouse_area(mkw::magnifier(button_sym))
                    .on_press(Msg::PlayPause)
                    .interaction(iced::mouse::Interaction::Pointer);

                let panel = mkw::bar_slider(0.0..=duration, pos, 0.05, p, Msg::Drag)
                    .on_release(Msg::Jump)
                    .width(Length::Fill);

                let control = row![
                    button,
                    Space::new().width(mk::spacing::M),
                    mkw::txt(
                        mk::format::duration_mmss(pos as u64),
                        mk::typo::CALLOUT,
                        p.on_surface_variant
                    ),
                    Space::new().width(mk::spacing::M),
                    panel,
                    Space::new().width(mk::spacing::M),
                    mkw::txt(
                        format!("\u{2212}{}", mk::format::duration_mmss((duration - pos).max(0.0) as u64)),
                        mk::typo::CALLOUT,
                        p.on_surface_variant
                    ),
                ]
                .align_y(iced::Alignment::Center);

                column![
                    stage,
                    Space::new().height(mk::spacing::M),
                    control,
                    Space::new().height(mk::spacing::XS),
                    mkw::footer(
                        format!(
                            "{} · {}×{} · {:.0} fps",
                            film.name(),
                            film.width,
                            film.height,
                            film.fps
                        ),
                        p
                    ),
                ]
                .into()
            }
        };

        let content = container(content)
            .padding(mk::spacing::L)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(color(p.surface_container).into()),
                border: iced::Border {
                    radius: mk::CORNER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        self.frame.window(
            "Matrix Player",
            env!("CARGO_PKG_VERSION"),
            "Videos ansehen — Bildschirmaufnahmen zuerst. Der Decoder läuft außer Prozess, wie im Leitbild.",
            content.into(),
            Msg::Frame,
        )
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
