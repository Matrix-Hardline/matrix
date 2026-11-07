//! Matrix Video (R210) — moving pictures with sound, iced-free.
//!
//! Grown out of Matrix Player's light path (Lager 46): ffmpeg decodes out
//! of process and hands finished frames through a pipe; the window takes
//! them from a DECK. Since the second round (01.11.2026) it plays like the
//! big players do:
//!
//! * **a stock** — a web input is fetched by curl into memory as fast as
//!   the line allows, up to two minutes ahead (`STOCK_S`, at most
//!   `STOCK_MAX` bytes); playing starts once three seconds lie ready. A
//!   stalling line eats the stock instead of the picture.
//! * **the sound is the clock** — a sound ffmpeg decodes to raw samples,
//!   we write them to the sound server ourselves (`pacat`) and count them:
//!   the play position is what has been heard. If the sound stalls, the
//!   clock stands; nothing can run away from it.
//! * **the picture follows the clock** — a picture ffmpeg decodes as fast
//!   as it is asked (no `-re`), every frame carries its time (constant
//!   rate via `fps=`), a small queue holds the next few. `take_frame` hands
//!   out the youngest frame that is due and drops late ones.
//! * **pause = SIGSTOP** of both decoders (the clock stands with the
//!   sound), a seek is a restart at the new time with a fresh generation.
//!
//! A medium without sound runs on the wall clock from its first frame.

pub mod venus;
pub mod film;
pub mod dash;

/// POSTER (round 135): a still of a video file for places that show what
/// plays (the lock screen's player, the bar) when nothing else gives a
/// picture — cut once by ffmpeg at a tenth of the length, kept in the
/// cache under a name from the file's path, size and time.
pub mod poster {
    use std::path::{Path, PathBuf};

    fn cache_path(file: &Path) -> Option<PathBuf> {
        use std::hash::{Hash, Hasher};
        let meta = std::fs::metadata(file).ok()?;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        file.hash(&mut h);
        meta.len().hash(&mut h);
        meta.modified().ok().hash(&mut h);
        let base = std::env::var("XDG_CACHE_HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache"));
        Some(base.join("matrix-poster").join(format!("{:016x}.jpg", h.finish())))
    }

    /// The still if it was cut already (cheap — for every beat).
    pub fn cached(file: &Path) -> Option<PathBuf> {
        cache_path(file).filter(|p| p.exists())
    }

    /// Cut the still (blocking: ffmpeg, about a second — call it off the
    /// window's thread). `duration_s` picks the moment (a tenth in).
    pub fn make(file: &Path, duration_s: f64) -> Option<PathBuf> {
        let out = cache_path(file)?;
        if out.exists() {
            return Some(out);
        }
        std::fs::create_dir_all(out.parent()?).ok()?;
        let at = if duration_s > 0.0 { (duration_s * 0.1).min(30.0) } else { 3.0 };
        let ok = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-nostdin", "-y", "-ss"])
            .arg(format!("{at:.2}"))
            .arg("-i")
            .arg(file)
            .args(["-frames:v", "1", "-vf", "scale=256:256:force_original_aspect_ratio=increase,crop=256:256"])
            .arg(&out)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        (ok && out.exists()).then_some(out)
    }
}

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// One input for ffmpeg: a path or URL, with the HTTP headers it needs
/// (a stream URL often only answers to the client that resolved it).
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    pub url: String,
    pub headers: Vec<(String, String)>,
    /// Bytes per second of the stream (0 = unknown) — sizes the stock.
    pub byterate: f64,
}

impl Input {
    pub fn file(path: &Path) -> Self {
        Input { url: path.to_string_lossy().into_owned(), headers: Vec::new(), byterate: 0.0 }
    }

    fn is_web(&self) -> bool {
        self.url.starts_with("http://") || self.url.starts_with("https://")
    }
}

/// What will be played: the inputs (one with picture and sound, or a
/// picture input followed by a sound input) and their measures.
#[derive(Debug, Clone, PartialEq)]
pub struct Media {
    pub inputs: Vec<Input>,
    pub duration_s: f64,
    /// Frames per second of the picture.
    pub fps: f64,
    /// The picture's codec as the source names it ("avc1.64001f", "h264",
    /// "vp09…"; empty = unknown) — round 66: H.264 may go to the Venus
    /// hardware decoder.
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub title: String,
}

/// Measure a local file with ffprobe — duration and picture size.
pub fn probe_file(path: &Path) -> Option<Media> {
    let out = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=codec_name,width,height,avg_frame_rate:format=duration",
            "-of", "default=nw=1",
        ])
        .arg(path)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let field = |key: &str| {
        text.lines().find_map(|l| l.strip_prefix(key)).map(|v| v.trim().to_string())
    };
    let width: u32 = field("width=")?.parse().ok()?;
    let height: u32 = field("height=")?.parse().ok()?;
    let duration_s: f64 = field("duration=").and_then(|d| d.parse().ok()).unwrap_or(0.0);
    let fps = field("avg_frame_rate=").map(|r| parse_rate(&r)).unwrap_or(0.0);
    if width == 0 || height == 0 {
        return None;
    }
    Some(Media {
        inputs: vec![Input::file(path)],
        duration_s,
        fps: if fps > 0.0 { fps } else { 30.0 },
        codec: field("codec_name=").unwrap_or_default(),
        width,
        height,
        title: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
    })
}

/// Measure any local medium — with a picture like `probe_file`, or sound
/// only (MP3: width and height 0, the player plays it `sound_only`).
pub fn probe_any(path: &Path) -> Option<Media> {
    if let Some(m) = probe_file(path) {
        return Some(m);
    }
    let out = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=nw=1:nk=1"])
        .arg(path)
        .output()
        .ok()?;
    let duration_s: f64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
    Some(Media {
        inputs: vec![Input::file(path)],
        duration_s,
        fps: 0.0,
        codec: String::new(),
        width: 0,
        height: 0,
        title: path.file_stem().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
    })
}

/// "30000/1001" or "25" -> frames per second (0 if unreadable).
pub fn parse_rate(rate: &str) -> f64 {
    let mut parts = rate.trim().split('/');
    let z: f64 = parts.next().and_then(|t| t.parse().ok()).unwrap_or(0.0);
    let n: f64 = parts.next().and_then(|t| t.parse().ok()).unwrap_or(1.0);
    if n > 0.0 { z / n } else { 0.0 }
}

/// The decode size for a picture of `width` × `height` drawn at most
/// `max_width` wide: even numbers (the codecs want them), aspect kept.
pub fn decode_size(width: u32, height: u32, max_width: u32) -> (u32, u32) {
    let w = max_width.min(width).max(2) & !1;
    let h = ((height as f64 * w as f64 / width.max(1) as f64).round() as u32).max(2) & !1;
    (w, h)
}

/// How far a web input is fetched ahead (seconds of the stream).
pub const STOCK_S: f64 = 120.0;
/// ...but never more than this many bytes per input.
pub const STOCK_MAX: usize = 48 << 20;
/// Playing starts once this much lies ready (or the stream is complete).
pub const STOCK_START_S: f64 = 3.0;
/// One range request of the stock.
const PIECE: usize = 4 << 20;
/// Frames decoded ahead of the clock. Round 53 (03.11.2026, the owner: «now
/// and then micro-stutters»): 4 frames were 0.17 s at 24/s — one heavy frame of
/// a software AV1 decode (≈ 2 cores on the Duo) next to other load emptied
/// the queue. 12 are half a second (≈ 37 MB at 1080p YUV).
const FRAME_QUEUE: usize = 12;
/// The sound format we write to the server.
const RATE: u64 = 48_000;
const BYTES_PER_SAMPLE: u64 = 4; // s16le, two channels
/// The sound server's buffer (pacat --latency-msec): what is written is
/// heard this much later. Round 53: 80 → 200 ms — a busy Duo (decoder,
/// CPU deep sleep since 03.11.) needs room; the clock subtracts the
/// measured latency, so picture and sound stay together.
const SOUND_LATENCY_MS: u64 = 200;

/// What the reader threads and the window share.
pub struct Deck {
    /// Decoded frames waiting for their time: (time s, running number, bytes).
    frames: Mutex<VecDeque<(f64, u64, Vec<u8>)>>,
    /// Round 66: the frame size when the decoder chose it (Venus hands
    /// the source size; the GPU scales) — else the Video's size.
    frame_size: Mutex<Option<(u32, u32)>>,
    /// Round 67: the frames are NV12 (Venus) — `Video::nv12`.
    nv12: AtomicBool,
    room: Condvar,
    /// Sound bytes handed to the server in this generation.
    sound_bytes: AtomicU64,
    /// The sound decoder delivered (a medium with sound) / gave up.
    sound_on: AtomicBool,
    sound_gone: AtomicBool,
    /// When the sound ended — the clock runs on by the wall from there, so
    /// the last frames still come due.
    sound_end: Mutex<Option<Instant>>,
    /// What the server says has been HEARD (µs of this playback) and when
    /// it said so — pa_simple_get_latency after every write.
    heard: Mutex<Option<(u64, Instant)>>,
    /// How far the newest shown frame lagged the clock (ms) — a decoder
    /// that cannot keep up shows here first.
    behind_ms: AtomicU64,
    /// Round 53: frames passed over because a younger one was already due
    /// (what the eye sees as a hitch).
    skipped: AtomicU64,
    /// The picture decoder reached its end.
    picture_end: AtomicBool,
    /// Wall-clock fallback (no sound): when the first frame came, and the
    /// paused time to subtract (ms).
    first_frame: Mutex<Option<Instant>>,
    paused_ms: AtomicU64,
    /// Every (re)start counts up; older threads recognise themselves.
    pub generation: AtomicU64,
}

impl Deck {
    fn new() -> Self {
        Deck {
            frames: Mutex::new(VecDeque::new()),
            frame_size: Mutex::new(None),
            nv12: AtomicBool::new(false),
            room: Condvar::new(),
            sound_bytes: AtomicU64::new(0),
            sound_on: AtomicBool::new(false),
            sound_gone: AtomicBool::new(false),
            sound_end: Mutex::new(None),
            heard: Mutex::new(None),
            behind_ms: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
            picture_end: AtomicBool::new(false),
            first_frame: Mutex::new(None),
            paused_ms: AtomicU64::new(0),
            generation: AtomicU64::new(0),
        }
    }

    fn current(&self, gen: u64) -> bool {
        self.generation.load(Ordering::SeqCst) == gen
    }
}

/// A fetched-ahead web input: curl fills it, a writer drains it into the
/// decoder. Shared between those two threads and the window (`stocked_s`).
struct Stock {
    chunks: Mutex<(VecDeque<Vec<u8>>, usize, bool)>, // chunks, bytes, complete
    changed: Condvar,
    cap: usize,
    byterate: f64,
}

/// A playing medium: decoders, fetchers, the sound writer, one deck.
pub struct Video {
    pub media: Media,
    pub deck: Arc<Deck>,
    /// Every child of this playback (curls, decoders, sound writer).
    children: Vec<Child>,
    /// The decoders among them (SIGSTOP on pause).
    decoders: Vec<u32>,
    stocks: Vec<Arc<Stock>>,
    /// The curl of each stock's current piece (0 = none) — killed on stop.
    fetchers: Vec<Arc<std::sync::atomic::AtomicU32>>,
    /// Round 128: the DASH index of each web input (None = it has none) —
    /// a seek fetches from the fragment that holds the time.
    dash: std::collections::HashMap<String, Option<Arc<dash::Index>>>,
    paused: bool,
    paused_at: Option<Instant>,
    from_s: f64,
    size: (u32, u32),
    /// Name of the sound stream in the mixer (the app's name).
    sound_name: String,
    /// Sound only (background audio: no picture decoded at all).
    sound_only: bool,
    /// The picture rate that reaches the window (capped).
    out_fps: f64,
    max_fps: Option<f64>,
    yuv: bool,
}

/// How a video plays.
#[derive(Debug, Clone)]
pub struct Spec {
    /// Widest picture the window will draw (decode is scaled down to it).
    pub width: u32,
    /// Or the EXACT picture size (physical px) — for windows that lay the
    /// frame into a box of their own (Matrix shell: box × scale).
    pub exact: Option<(u32, u32)>,
    /// The stream's name in the sound mixer.
    pub sound_name: String,
    /// Only the sound (background play) — no picture is decoded.
    pub sound_only: bool,
    /// At most this many pictures per second reach the window (a 60 fps
    /// stream on a small panel: half the scaling, pipe and upload work).
    pub max_fps: Option<f64>,
    /// Frames as planar YUV 4:2:0 instead of RGBA (R210): ffmpeg skips
    /// the colour conversion (measured on the duo: half its CPU time),
    /// the GPU converts in its shader. Lay them with `ImageAtlas::put_yuv`.
    pub yuv: bool,
}

/// Every child dies with its player — a killed or crashed player must not
/// leave sound or downloads running on their own.
fn die_with_parent(cmd: &mut Command) {
    unsafe {
        use std::os::unix::process::CommandExt;
        cmd.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
            Ok(())
        });
    }
}

impl Video {
    pub fn new(media: Media, spec: Spec) -> Self {
        let size = spec.exact.unwrap_or_else(|| decode_size(media.width, media.height, spec.width));
        Video {
            media,
            deck: Arc::new(Deck::new()),
            children: Vec::new(),
            decoders: Vec::new(),
            stocks: Vec::new(),
            fetchers: Vec::new(),
            dash: std::collections::HashMap::new(),
            paused: false,
            paused_at: None,
            from_s: 0.0,
            size,
            sound_name: spec.sound_name,
            sound_only: spec.sound_only,
            out_fps: 0.0,
            max_fps: spec.max_fps,
            yuv: spec.yuv,
        }
    }

    /// Are the frames YUV 4:2:0 (else RGBA)?
    pub fn yuv(&self) -> bool {
        self.yuv
    }

    /// Round 67: are the YUV frames NV12 (the hardware decoder's layout —
    /// lay them with `ImageAtlas::put_nv12`), else I420 (`put_yuv`)?
    pub fn nv12(&self) -> bool {
        self.deck.nv12.load(Ordering::SeqCst)
    }

    /// Lay a frame from `take_frame` into the atlas in its own layout —
    /// the one call a player needs (RGBA, I420 or NV12).
    pub fn lay_frame(&self, atlas: &mut matrixkit_image::ImageAtlas, key: &Path, w: u32, h: u32, bytes: Vec<u8>) {
        if self.yuv && self.nv12() {
            atlas.put_nv12(key, w, h, bytes);
        } else {
            atlas.put_frame(key, w, h, bytes, self.yuv);
        }
    }

    /// The decoded picture size.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// A new picture size (the window changed): restart where it stands.
    pub fn resize(&mut self, size: (u32, u32)) {
        if size == self.size || size.0 < 2 || size.1 < 2 {
            return;
        }
        self.size = size;
        if !self.children.is_empty() {
            let at = self.position_s();
            let paused = self.paused;
            self.play(at);
            self.set_paused(paused);
        }
    }

    /// Start (or restart) at `from_s` seconds.
    pub fn play(&mut self, from_s: f64) {
        self.stop_children();
        let from_s = from_s.max(0.0);
        let gen = self.deck.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let deck = &self.deck;
        deck.frames.lock().unwrap_or_else(|e| e.into_inner()).clear();
        *deck.frame_size.lock().unwrap_or_else(|e| e.into_inner()) = None;
        deck.nv12.store(false, Ordering::SeqCst);
        deck.sound_bytes.store(0, Ordering::SeqCst);
        deck.sound_on.store(false, Ordering::SeqCst);
        deck.sound_gone.store(false, Ordering::SeqCst);
        *deck.sound_end.lock().unwrap_or_else(|e| e.into_inner()) = None;
        *deck.heard.lock().unwrap_or_else(|e| e.into_inner()) = None;
        deck.picture_end.store(self.sound_only, Ordering::SeqCst);
        *deck.first_frame.lock().unwrap_or_else(|e| e.into_inner()) = None;
        deck.paused_ms.store(0, Ordering::SeqCst);
        self.paused = false;
        self.paused_at = None;
        self.from_s = from_s;
        let fps = if self.media.fps > 0.0 { self.media.fps } else { 30.0 };
        self.out_fps = self.max_fps.filter(|m| *m > 0.0 && fps > *m + 0.5).unwrap_or(fps);

        // The sound: the last input (one combined input, or the audio
        // input after the video input).
        if let Some(input) = self.media.inputs.last().cloned() {
            self.start_sound(&input, from_s, gen);
        }
        if !self.sound_only {
            if let Some(input) = self.media.inputs.first().cloned() {
                self.start_picture(&input, from_s, gen);
            }
        }
    }

    /// An ffmpeg for `input`: from the stock (a web input from the start)
    /// or reading the input itself (files; seeks — a time's byte offset is
    /// unknown, ffmpeg finds it with range requests).
    ///
    /// Round 128: a web SEEK goes through the stock as well when the input
    /// has a DASH index — the head, then fragments from the one holding
    /// `from_s`, in fast range pieces (ffmpeg's own seek is one throttled
    /// request: 1.8× real time on the Duo, the sound tore at every dip).
    /// The third value is that fragment's start time; `trim` asks ffmpeg to
    /// drop what lies before `from_s` (the Venus path drops it itself).
    fn decoder(&mut self, input: &Input, from_s: f64, gen: u64, trim: bool) -> (Command, Option<Arc<Stock>>, Option<f64>) {
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-v", "error"]);
        if input.is_web() && from_s < 0.5 {
            if let Some(stock) = self.fetch(input, gen, Vec::new(), 0) {
                cmd.args(["-i", "pipe:0"]).stdin(Stdio::piped());
                return (cmd, Some(stock), Some(0.0));
            }
        }
        if input.is_web() {
            let index = self.dash.entry(input.url.clone()).or_insert_with(|| dash::fetch(input).map(Arc::new)).clone();
            if let Some((offset, start)) = index.as_ref().and_then(|i| i.locate(from_s)) {
                let head = index.as_ref().map(|i| i.head.clone()).unwrap_or_default();
                if let Some(stock) = self.fetch(input, gen, head, offset) {
                    eprintln!("[video] Sprung über den Vorrat: Abschnitt ab {start:.2} s (Byte {offset})");
                    cmd.args(["-i", "pipe:0"]).stdin(Stdio::piped());
                    if trim && from_s - start > 0.01 {
                        cmd.arg("-ss").arg(format!("{:.3}", from_s - start));
                    }
                    return (cmd, Some(stock), Some(start));
                }
            }
        }
        cmd.arg("-nostdin").stdin(Stdio::null());
        if !input.headers.is_empty() {
            let lines: String = input.headers.iter().map(|(k, v)| format!("{k}: {v}\r\n")).collect();
            cmd.arg("-headers").arg(lines);
        }
        cmd.arg("-ss").arg(format!("{from_s:.3}")).arg("-i").arg(&input.url);
        (cmd, None, None)
    }

    fn start_sound(&mut self, input: &Input, from_s: f64, gen: u64) {
        let (mut cmd, stock, _) = self.decoder(input, from_s, gen, true);
        cmd.args(["-map", "0:a:0", "-vn", "-f", "s16le", "-ac", "2", "-ar"])
            .arg(RATE.to_string())
            .arg("pipe:1")
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        die_with_parent(&mut cmd);
        let Ok(mut decoder) = cmd.spawn() else {
            self.deck.sound_gone.store(true, Ordering::SeqCst);
            return;
        };
        if let (Some(stock), Some(sink)) = (stock, decoder.stdin.take()) {
            drain(stock, sink, Arc::clone(&self.deck), gen);
        }
        let Some(samples) = decoder.stdout.take() else {
            self.deck.sound_gone.store(true, Ordering::SeqCst);
            return;
        };
        self.decoders.push(decoder.id());
        self.children.push(decoder);
        // The sound server straight through libpulse-simple: it tells how
        // much still waits in its buffer — the clock. pacat (a fixed
        // guess of its buffer) is the fallback without the library.
        if let Some(pulse) = pulse::Simple::open(&self.sound_name, RATE as u32, 2, SOUND_LATENCY_MS as u32) {
            let deck = Arc::clone(&self.deck);
            named("mv-ton", move || sound_loop(deck, samples, Sink::Pulse(pulse), gen));
            return;
        }
        let mut server = Command::new("pacat");
        server
            .args(["--raw", "--format=s16le", "--channels=2"])
            .arg(format!("--rate={RATE}"))
            .arg(format!("--latency-msec={SOUND_LATENCY_MS}"))
            .arg(format!("--client-name={}", self.sound_name))
            .arg(format!("--stream-name={}", self.sound_name))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        die_with_parent(&mut server);
        let Ok(mut writer) = server.spawn() else {
            self.deck.sound_gone.store(true, Ordering::SeqCst);
            return;
        };
        let Some(out) = writer.stdin.take() else {
            self.deck.sound_gone.store(true, Ordering::SeqCst);
            return;
        };
        let deck = Arc::clone(&self.deck);
        named("mv-ton", move || sound_loop(deck, samples, Sink::Cat(out), gen));
        self.children.push(writer);
    }

    fn start_picture(&mut self, input: &Input, from_s: f64, gen: u64) {
        if self.start_venus(input, from_s, gen) {
            return;
        }
        let (mut cmd, stock, _) = self.decoder(input, from_s, gen, true);
        let (w, h) = self.size;
        let pix = if self.yuv { "yuv420p" } else { "rgba" };
        // A constant rate: frame n is due at from_s + n / out_fps.
        cmd.args(["-map", "0:v:0", "-an", "-vf"])
            .arg(format!("fps={},scale={w}:{h}:flags=fast_bilinear", self.out_fps))
            .args(["-f", "rawvideo", "-pix_fmt", pix, "pipe:1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        die_with_parent(&mut cmd);
        let Ok(mut decoder) = cmd.spawn() else {
            self.deck.picture_end.store(true, Ordering::SeqCst);
            return;
        };
        if let (Some(stock), Some(sink)) = (stock, decoder.stdin.take()) {
            drain(stock, sink, Arc::clone(&self.deck), gen);
        }
        if let Some(pipe) = decoder.stdout.take() {
            let deck = Arc::clone(&self.deck);
            let fps = self.out_fps;
            let len = if self.yuv { (w * h * 3 / 2) as usize } else { (w * h * 4) as usize };
            named("mv-bild", move || read_frames(deck, pipe, gen, from_s, fps, len));
        }
        self.decoders.push(decoder.id());
        self.children.push(decoder);
    }

    /// Round 66: H.264 through the Snapdragon's own decoder (Surface Duo,
    /// `venus`): ffmpeg only unpacks the stream (`-c copy`, access-unit
    /// delimiters), Venus decodes at ~200 fps for 1080p, a thread turns its
    /// NV12 into the I420 the shader takes. False = not here or not this
    /// medium; the software path runs instead. `MATRIX_VENUS=0` switches
    /// it off.
    fn start_venus(&mut self, input: &Input, from_s: f64, gen: u64) -> bool {
        let codec = self.media.codec.to_ascii_lowercase();
        if !self.yuv || !(codec.starts_with("avc") || codec == "h264") || std::env::var("MATRIX_VENUS").is_ok_and(|v| v == "0") {
            return false;
        }
        let Some(device) = venus::decoder_device() else { return false };
        let (sw, sh) = (self.media.width.max(16), self.media.height.max(16));
        let decoder = match venus::Decoder::open(&device, venus::Codec::H264, sw, sh) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[video] Venus-Decoder: {e} — Software-Weg");
                return false;
            }
        };
        // Where does the copied stream begin? A seek lands on the key
        // picture before `from_s`; its time comes from ffprobe and the
        // pictures before `from_s` are dropped (sound and picture agree).
        let (mut cmd, stock, fragment) = self.decoder(input, from_s, gen, false);
        // A fragment begins with a key picture: its start is the stream's.
        let start_s = if from_s < 0.5 { 0.0 } else { fragment.or_else(|| key_time(input, from_s)).unwrap_or(from_s) };
        cmd.args(["-map", "0:v:0", "-an", "-c:v", "copy", "-bsf:v", "h264_mp4toannexb,h264_metadata=aud=insert", "-f", "h264", "pipe:1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        die_with_parent(&mut cmd);
        let Ok(mut unpacker) = cmd.spawn() else { return false };
        if let (Some(stock), Some(sink)) = (stock, unpacker.stdin.take()) {
            drain(stock, sink, Arc::clone(&self.deck), gen);
        }
        let Some(pipe) = unpacker.stdout.take() else { return false };
        let deck = Arc::clone(&self.deck);
        let (src_fps, out_fps) = (if self.media.fps > 0.0 { self.media.fps } else { 30.0 }, self.out_fps);
        named("mv-venus", move || venus_frames(deck, pipe, decoder, gen, start_s, from_s, src_fps, out_fps));
        self.decoders.push(unpacker.id());
        self.children.push(unpacker);
        eprintln!("[video] Venus-Decoder: H.264 {sw}x{sh} in Hardware (ab {start_s:.2} s)");
        true
    }

    /// curl fetches `input` into a new stock (own thread), in PIECES of
    /// `PIECE` bytes, each its own range request: YouTube throttles one
    /// long request to about playing speed (measured 01.11.2026: 9 s of
    /// stock after 35 s) and serves short ranges at line speed — its own
    /// player and yt-dlp fetch in pieces too. A server that ignores the
    /// range sends everything in the first answer; that is fine as well.
    ///
    /// Round 128: `head` goes first (a seek's DASH head), the pieces begin
    /// at byte `start`. A piece that comes SHORT before the known end is
    /// fetched again from where it broke — before, any torn answer counted
    /// as the end of the file: the sound stopped early and autoplay moved
    /// on (owner: «the sound cuts out shortly before the end», 05.11.2026).
    fn fetch(&mut self, input: &Input, gen: u64, head: Vec<u8>, start: u64) -> Option<Arc<Stock>> {
        let byterate = if input.byterate > 0.0 { input.byterate } else { 250_000.0 };
        let stock = Arc::new(Stock {
            chunks: Mutex::new((VecDeque::new(), 0, false)),
            changed: Condvar::new(),
            // At least room for three pieces — a sound stream's two minutes
            // (~2 MB) were smaller than one piece and the fetcher waited forever.
            cap: ((byterate * STOCK_S) as usize).clamp(3 * PIECE, STOCK_MAX),
            byterate,
        });
        if !head.is_empty() {
            let mut g = stock.chunks.lock().unwrap_or_else(|e| e.into_inner());
            g.1 += head.len();
            g.0.push_back(head);
        }
        let fill = Arc::clone(&stock);
        let deck = Arc::clone(&self.deck);
        let input = input.clone();
        let pid = Arc::new(std::sync::atomic::AtomicU32::new(0));
        self.fetchers.push(Arc::clone(&pid));
        named("mv-vorrat", move || {
            let mut offset: usize = start as usize;
            let total = dash::total_size(&input).map(|t| t as usize);
            let mut retries = 0u32;
            let mut buf = vec![0u8; 64 << 10];
            'pieces: loop {
                // Room for a whole piece first (or a new playback).
                {
                    let mut g = fill.chunks.lock().unwrap_or_else(|e| e.into_inner());
                    while g.1 > 0 && g.1 + PIECE > fill.cap && deck.current(gen) {
                        g = fill.changed.wait_timeout(g, Duration::from_millis(200)).map(|r| r.0).unwrap_or_else(|e| e.into_inner().0);
                    }
                }
                if !deck.current(gen) {
                    return;
                }
                let mut curl = Command::new("curl");
                curl.args(["-s", "-L", "--fail", "--retry", "3", "--retry-all-errors", "-r"])
                    .arg(format!("{}-{}", offset, offset + PIECE - 1));
                for (k, v) in &input.headers {
                    curl.arg("-H").arg(format!("{k}: {v}"));
                }
                curl.arg(&input.url).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
                die_with_parent(&mut curl);
                let Ok(mut child) = curl.spawn() else { break };
                pid.store(child.id(), Ordering::SeqCst);
                let Some(mut source) = child.stdout.take() else { break };
                let mut got = 0usize;
                loop {
                    let n = match source.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => n,
                    };
                    if !deck.current(gen) {
                        let _ = child.kill();
                        let _ = child.wait();
                        return;
                    }
                    got += n;
                    let mut g = fill.chunks.lock().unwrap_or_else(|e| e.into_inner());
                    g.0.push_back(buf[..n].to_vec());
                    g.1 += n;
                    fill.changed.notify_all();
                }
                let _ = child.wait();
                pid.store(0, Ordering::SeqCst);
                offset += got;
                if got < PIECE {
                    // Short before the known end: torn — again from here.
                    if total.is_some_and(|t| offset < t) && retries < 8 && deck.current(gen) {
                        retries += 1;
                        eprintln!("[video] Vorrat: Stück abgerissen bei {offset} von {} Byte — neu ({retries})", total.unwrap_or(0));
                        std::thread::sleep(Duration::from_millis(250 * retries as u64));
                        continue 'pieces;
                    }
                    // The end (or a server without ranges that sent it
                    // all at once, or an answer that failed for good).
                    break 'pieces;
                }
                retries = 0;
            }
            let mut g = fill.chunks.lock().unwrap_or_else(|e| e.into_inner());
            g.2 = true;
            fill.changed.notify_all();
        });
        self.stocks.push(Arc::clone(&stock));
        Some(stock)
    }

    /// Frames skipped so far because the next was already due (round 53).
    pub fn skipped(&self) -> u64 {
        self.deck.skipped.load(Ordering::Relaxed)
    }

    /// How far the last shown frame was behind the clock (seconds): near
    /// 0 is in step; growing = the decoder cannot keep up.
    pub fn behind_s(&self) -> f64 {
        self.deck.behind_ms.load(Ordering::Relaxed) as f64 / 1000.0
    }

    /// Seconds fetched ahead (the smallest stock), None without stocks.
    pub fn stocked_s(&self) -> Option<f64> {
        self.stocks
            .iter()
            .map(|s| {
                let g = s.chunks.lock().unwrap_or_else(|e| e.into_inner());
                if g.2 { f64::INFINITY } else { g.1 as f64 / s.byterate }
            })
            .reduce(f64::min)
    }

    /// Pause or resume — the decoders are stopped and continued; the clock
    /// stands with the sound.
    pub fn set_paused(&mut self, paused: bool) {
        if paused == self.paused {
            return;
        }
        for pid in &self.decoders {
            let signal = if paused { libc::SIGSTOP } else { libc::SIGCONT };
            // SAFETY: plain kill(2) on our own child's pid.
            unsafe {
                libc::kill(*pid as i32, signal);
            }
        }
        if paused {
            self.paused_at = Some(Instant::now());
        } else if let Some(at) = self.paused_at.take() {
            self.deck.paused_ms.fetch_add(at.elapsed().as_millis() as u64, Ordering::SeqCst);
        }
        self.paused = paused;
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn done(&self) -> bool {
        let d = &self.deck;
        let picture_done = d.picture_end.load(Ordering::SeqCst) && d.frames.lock().map(|f| f.is_empty()).unwrap_or(true);
        let sound_done = d.sound_gone.load(Ordering::SeqCst);
        !self.children.is_empty() && picture_done && (sound_done || !d.sound_on.load(Ordering::SeqCst))
    }

    /// Play position in seconds: what has been HEARD (sound written minus
    /// the server's buffer), or the wall clock since the first frame.
    pub fn position_s(&self) -> f64 {
        let d = &self.deck;
        if d.sound_on.load(Ordering::SeqCst) {
            let ended = *d.sound_end.lock().unwrap_or_else(|e| e.into_inner());
            if let Some((heard_us, at)) = *d.heard.lock().unwrap_or_else(|e| e.into_inner()) {
                // Between two writes (20 ms) the sound plays on — move the
                // clock with it, but never further than a write's worth;
                // after the end the wall carries the last frames.
                let on = match ended {
                    Some(end) => end.duration_since(at).as_micros() as u64 + end.elapsed().as_micros() as u64,
                    None if self.paused => 0,
                    None => (at.elapsed().as_micros() as u64).min(40_000),
                };
                return self.from_s + (heard_us + on) as f64 / 1_000_000.0;
            }
            let written_ms = d.sound_bytes.load(Ordering::SeqCst) * 1000 / (RATE * BYTES_PER_SAMPLE);
            let after_end = d.sound_end.lock().unwrap_or_else(|e| e.into_inner()).map(|t| t.elapsed().as_millis() as u64).unwrap_or(0);
            return self.from_s + (written_ms + after_end).saturating_sub(SOUND_LATENCY_MS) as f64 / 1000.0;
        }
        if d.sound_gone.load(Ordering::SeqCst) {
            if let Some(first) = *d.first_frame.lock().unwrap_or_else(|e| e.into_inner()) {
                let now = self.paused_at.unwrap_or_else(Instant::now);
                let ms = now.duration_since(first).as_millis() as u64;
                return self.from_s + ms.saturating_sub(d.paused_ms.load(Ordering::SeqCst)) as f64 / 1000.0;
            }
        }
        self.from_s
    }

    /// The youngest frame that is due by the clock, if it is not `last`
    /// (running number). Frames that came too late are dropped; frames
    /// still early stay queued.
    pub fn take_frame(&self, last: Option<u64>) -> Option<(u64, u32, u32, Vec<u8>)> {
        let clock = self.position_s();
        let (w, h) = self.deck.frame_size.lock().unwrap_or_else(|e| e.into_inner()).unwrap_or(self.size);
        let mut q = self.deck.frames.lock().unwrap_or_else(|e| e.into_inner());
        let mut due = None;
        while q.front().is_some_and(|f| f.0 <= clock + 0.005) {
            if due.is_some() {
                self.deck.skipped.fetch_add(1, Ordering::Relaxed);
            }
            due = q.pop_front();
        }
        // Before the sound starts: show the first frame as a still.
        if due.is_none() && last.is_none() && !self.deck.sound_on.load(Ordering::SeqCst) {
            due = q.pop_front();
        }
        self.deck.room.notify_all();
        let (at, nr, bytes) = due?;
        self.deck.behind_ms.store(((clock - at).max(0.0) * 1000.0) as u64, Ordering::Relaxed);
        (Some(nr) != last).then_some((nr, w, h, bytes))
    }

    /// Is a picture running that wants frames (the window's clock)?
    pub fn running(&self) -> bool {
        !self.children.is_empty() && !self.paused && !self.done()
    }

    pub fn stop(&mut self) {
        self.deck.generation.fetch_add(1, Ordering::SeqCst);
        self.stop_children();
    }

    /// Round 178 (stability audit; the Duo showed `[ffmpeg] <defunct>`): decoders and the sound writer that ended by
    /// themselves stay zombies until the next play/stop. `try_wait` collects their exit (the kernel entry goes away)
    /// and keeps the `Child` in the list, so `done()`/`running()` read exactly as before. Call it on the app's pulse.
    pub fn reap(&mut self) {
        for child in &mut self.children {
            let _ = child.try_wait();
        }
    }

    fn stop_children(&mut self) {
        self.deck.room.notify_all();
        for stock in self.stocks.drain(..) {
            stock.changed.notify_all();
        }
        for pid in self.fetchers.drain(..) {
            let p = pid.swap(0, Ordering::SeqCst);
            if p > 0 {
                // SAFETY: plain kill(2) on our own curl child.
                unsafe {
                    libc::kill(p as i32, libc::SIGKILL);
                }
            }
        }
        for mut child in self.children.drain(..) {
            // A stopped child must be continued to die cleanly.
            unsafe {
                libc::kill(child.id() as i32, libc::SIGCONT);
            }
            let _ = child.kill();
            let _ = child.wait();
        }
        self.decoders.clear();
    }
}

impl Drop for Video {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The writer of a stock: waits for the start reserve, then hands the
/// chunks to the decoder as it reads them (own thread).
fn drain(stock: Arc<Stock>, mut sink: ChildStdin, deck: Arc<Deck>, gen: u64) {
    named("mv-speise", move || {
        let start = (stock.byterate * STOCK_START_S) as usize;
        let began = Instant::now();
        {
            let mut g = stock.chunks.lock().unwrap_or_else(|e| e.into_inner());
            while g.1 < start && !g.2 && deck.current(gen) && began.elapsed() < Duration::from_secs(10) {
                g = stock.changed.wait_timeout(g, Duration::from_millis(100)).map(|r| r.0).unwrap_or_else(|e| e.into_inner().0);
            }
        }
        loop {
            let chunk = {
                let mut g = stock.chunks.lock().unwrap_or_else(|e| e.into_inner());
                loop {
                    if !deck.current(gen) {
                        return;
                    }
                    if let Some(c) = g.0.pop_front() {
                        g.1 -= c.len();
                        stock.changed.notify_all();
                        break c;
                    }
                    if g.2 {
                        return; // complete — dropping the sink ends the input
                    }
                    g = stock.changed.wait_timeout(g, Duration::from_millis(200)).map(|r| r.0).unwrap_or_else(|e| e.into_inner().0);
                }
            };
            if sink.write_all(&chunk).is_err() {
                return;
            }
        }
    });
}

/// The sound: raw samples from the decoder to the server, counted — the
/// count IS the clock.
/// Where the samples go.
enum Sink {
    Pulse(pulse::Simple),
    Cat(ChildStdin),
}

fn sound_loop(deck: Arc<Deck>, mut samples: std::process::ChildStdout, mut out: Sink, gen: u64) {
    let mut buf = vec![0u8; (RATE * BYTES_PER_SAMPLE / 50) as usize]; // 20 ms
    loop {
        let n = match samples.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if !deck.current(gen) {
            return;
        }
        let written = match &mut out {
            Sink::Pulse(p) => p.write(&buf[..n]),
            Sink::Cat(c) => c.write_all(&buf[..n]).is_ok(),
        };
        if !written {
            break;
        }
        let total = deck.sound_bytes.fetch_add(n as u64, Ordering::SeqCst) + n as u64;
        if let Sink::Pulse(p) = &out {
            let written_us = total * 1_000_000 / (RATE * BYTES_PER_SAMPLE);
            let heard = written_us.saturating_sub(p.latency_us());
            *deck.heard.lock().unwrap_or_else(|e| e.into_inner()) = Some((heard, Instant::now()));
        }
        deck.sound_on.store(true, Ordering::SeqCst);
    }
    if deck.current(gen) {
        *deck.sound_end.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
        deck.sound_gone.store(true, Ordering::SeqCst);
        // Without any sound the picture keeps the wall clock; with sound
        // the clock stood still at its end — the picture's end follows.
        if deck.sound_bytes.load(Ordering::SeqCst) == 0 {
            deck.sound_on.store(false, Ordering::SeqCst);
        }
    }
}

/// The reader: whole frames from the pipe into the queue, each with its
/// time; waits while the queue is full (the decoder waits with it).
/// A named thread (round 67): `mv-ton`, `mv-bild`, `mv-venus`, `mv-vorrat`,
/// `mv-speise` — per-thread CPU (/proc/PID/task/*/comm) says who works.
fn named<F: FnOnce() + Send + 'static>(name: &str, f: F) {
    if std::thread::Builder::new().name(name.to_string()).spawn(f).is_err() {
        eprintln!("[video] Faden {name} startet nicht");
    }
}

/// The time of the key picture a seek to `at_s` lands on (ffprobe reads
/// the first packet after the same seek ffmpeg does).
fn key_time(input: &Input, at_s: f64) -> Option<f64> {
    let mut probe = Command::new("ffprobe");
    probe.args(["-v", "error", "-select_streams", "v:0", "-read_intervals"]).arg(format!("{at_s:.3}%+#1"));
    if !input.headers.is_empty() {
        let lines: String = input.headers.iter().map(|(k, v)| format!("{k}: {v}\r\n")).collect();
        probe.arg("-headers").arg(lines);
    }
    probe.args(["-show_entries", "packet=pts_time", "-of", "csv=p=0"]).arg(&input.url).stdin(Stdio::null()).stderr(Stdio::null());
    let out = probe.output().ok()?;
    String::from_utf8_lossy(&out.stdout).lines().find_map(|l| l.trim().trim_end_matches(',').parse::<f64>().ok())
}

/// The Venus thread: cut the unpacked stream into access units, decode,
/// thin to the window's rate, turn NV12 (Venus layout) into tight I420.
#[allow(clippy::too_many_arguments)]
fn venus_frames(deck: Arc<Deck>, mut pipe: std::process::ChildStdout, mut decoder: venus::Decoder, gen: u64, start_s: f64, from_s: f64, src_fps: f64, out_fps: f64) {
    let mut pending: Vec<u8> = Vec::with_capacity(1 << 20);
    let mut chunk = vec![0u8; 256 << 10];
    let mut source_nr: u64 = 0;
    let mut kept: u64 = 0;
    let step = (out_fps / src_fps).min(1.0);
    let mut deliver = |f: venus::Frame<'_>| {
        let at = start_s + source_nr as f64 / src_fps;
        let take = ((source_nr + 1) as f64 * step).floor() > (source_nr as f64 * step).floor();
        source_nr += 1;
        if !take || at + 0.5 / src_fps < from_s || !deck.current(gen) {
            return;
        }
        // Round 67: NV12 stays NV12 — row copies only (the shader
        // `fs_nv12` converts); the I420 shuffle cost the player ~30 % CPU.
        let (w, h) = (f.width as usize & !1, f.height as usize & !1);
        let mut out = vec![0u8; w * h * 3 / 2];
        let (y_plane, uv_plane) = out.split_at_mut(w * h);
        let luma = f.luma();
        for row in 0..h {
            y_plane[row * w..row * w + w].copy_from_slice(&luma[row * f.stride..row * f.stride + w]);
        }
        let chroma = f.chroma();
        for row in 0..h / 2 {
            uv_plane[row * w..row * w + w].copy_from_slice(&chroma[row * f.stride..row * f.stride + w]);
        }
        *deck.frame_size.lock().unwrap_or_else(|e| e.into_inner()) = Some((w as u32, h as u32));
        deck.nv12.store(true, Ordering::SeqCst);
        {
            let mut first = deck.first_frame.lock().unwrap_or_else(|e| e.into_inner());
            if first.is_none() {
                *first = Some(Instant::now());
            }
        }
        kept += 1;
        let mut q = deck.frames.lock().unwrap_or_else(|e| e.into_inner());
        // Round 67: decode in BURSTS — a full queue waits until it is down
        // to a third. Frame by frame, every queue woke the video block
        // from its power collapse (QBUF 6 ms, 34 % kernel time measured);
        // in bursts it sleeps between them like it does on Android.
        if q.len() >= FRAME_QUEUE {
            while q.len() > FRAME_QUEUE / 3 && deck.current(gen) {
                q = deck.room.wait_timeout(q, Duration::from_millis(100)).map(|r| r.0).unwrap_or_else(|e| e.into_inner().0);
            }
        }
        if deck.current(gen) {
            q.push_back((at, gen * 1_000_000 + kept, out));
        }
    };
    let mut ts: i64 = 0;
    loop {
        let n = match pipe.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if !deck.current(gen) {
            return;
        }
        pending.extend_from_slice(&chunk[..n]);
        let (units, consumed) = venus::access_units(&pending);
        for u in units {
            ts += 1;
            if let Err(e) = decoder.decode(&pending[u], ts, &mut deliver) {
                eprintln!("[video] Venus: {e}");
                deck.picture_end.store(true, Ordering::SeqCst);
                return;
            }
        }
        pending.drain(..consumed);
    }
    if deck.current(gen) && !pending.is_empty() {
        let _ = decoder.decode(&pending, ts + 1, &mut deliver);
    }
    let _ = decoder.finish(&mut deliver);
    if deck.current(gen) {
        deck.picture_end.store(true, Ordering::SeqCst);
    }
}

fn read_frames(deck: Arc<Deck>, mut pipe: std::process::ChildStdout, gen: u64, from_s: f64, fps: f64, len: usize) {
    let mut nr: u64 = 0;
    loop {
        let mut buffer = vec![0u8; len];
        if pipe.read_exact(&mut buffer).is_err() {
            if deck.current(gen) {
                deck.picture_end.store(true, Ordering::SeqCst);
            }
            return;
        }
        if !deck.current(gen) {
            return;
        }
        let at = from_s + nr as f64 / fps;
        nr += 1;
        {
            let mut first = deck.first_frame.lock().unwrap_or_else(|e| e.into_inner());
            if first.is_none() {
                *first = Some(Instant::now());
            }
        }
        let mut q = deck.frames.lock().unwrap_or_else(|e| e.into_inner());
        while q.len() >= FRAME_QUEUE && deck.current(gen) {
            q = deck.room.wait_timeout(q, Duration::from_millis(100)).map(|r| r.0).unwrap_or_else(|e| e.into_inner().0);
        }
        if !deck.current(gen) {
            return;
        }
        q.push_back((at, gen * 1_000_000 + nr, buffer));
    }
}

/// libpulse-simple, loaded at run time (no build dependency): a playback
/// stream we write samples to and ask for its latency.
mod pulse {
    use std::ffi::{c_char, c_int, c_void, CString};

    #[repr(C)]
    struct SampleSpec {
        format: c_int,
        rate: u32,
        channels: u8,
    }

    #[repr(C)]
    struct BufferAttr {
        maxlength: u32,
        tlength: u32,
        prebuf: u32,
        minreq: u32,
        fragsize: u32,
    }

    type NewFn = unsafe extern "C" fn(
        *const c_char,
        *const c_char,
        c_int,
        *const c_char,
        *const c_char,
        *const SampleSpec,
        *const c_void,
        *const BufferAttr,
        *mut c_int,
    ) -> *mut c_void;
    type WriteFn = unsafe extern "C" fn(*mut c_void, *const c_void, usize, *mut c_int) -> c_int;
    type LatencyFn = unsafe extern "C" fn(*mut c_void, *mut c_int) -> u64;
    type FreeFn = unsafe extern "C" fn(*mut c_void);

    const PA_SAMPLE_S16LE: c_int = 3;
    const PA_STREAM_PLAYBACK: c_int = 1;

    pub struct Simple {
        handle: *mut c_void,
        write: WriteFn,
        latency: LatencyFn,
        free: FreeFn,
    }

    // The stream lives on the one sound thread that owns it.
    unsafe impl Send for Simple {}

    impl Simple {
        pub fn open(name: &str, rate: u32, channels: u8, target_ms: u32) -> Option<Simple> {
            // SAFETY: dlopen/dlsym with valid C strings; the symbols have the
            // documented libpulse-simple signatures.
            unsafe {
                let lib = libc::dlopen(c"libpulse-simple.so.0".as_ptr(), libc::RTLD_NOW);
                if lib.is_null() {
                    return None;
                }
                let sym = |n: &std::ffi::CStr| {
                    let p = libc::dlsym(lib, n.as_ptr());
                    (!p.is_null()).then_some(p)
                };
                let new: NewFn = std::mem::transmute(sym(c"pa_simple_new")?);
                let write: WriteFn = std::mem::transmute(sym(c"pa_simple_write")?);
                let latency: LatencyFn = std::mem::transmute(sym(c"pa_simple_get_latency")?);
                let free: FreeFn = std::mem::transmute(sym(c"pa_simple_free")?);
                let spec = SampleSpec { format: PA_SAMPLE_S16LE, rate, channels };
                let bytes_per_ms = rate * channels as u32 * 2 / 1000;
                let attr = BufferAttr {
                    maxlength: u32::MAX,
                    tlength: target_ms * bytes_per_ms,
                    prebuf: u32::MAX,
                    minreq: u32::MAX,
                    fragsize: u32::MAX,
                };
                let app = CString::new(name).ok()?;
                let mut err: c_int = 0;
                let handle = new(
                    std::ptr::null(),
                    app.as_ptr(),
                    PA_STREAM_PLAYBACK,
                    std::ptr::null(),
                    app.as_ptr(),
                    &spec,
                    std::ptr::null(),
                    &attr,
                    &mut err,
                );
                (!handle.is_null()).then_some(Simple { handle, write, latency, free })
            }
        }

        /// Blocks while the server's buffer is full — that paces the sound.
        pub fn write(&mut self, data: &[u8]) -> bool {
            let mut err: c_int = 0;
            // SAFETY: handle is a live pa_simple; data is a valid slice.
            unsafe { (self.write)(self.handle, data.as_ptr() as *const c_void, data.len(), &mut err) == 0 }
        }

        /// How long the written samples still wait before they are heard.
        pub fn latency_us(&self) -> u64 {
            let mut err: c_int = 0;
            // SAFETY: handle is a live pa_simple.
            let l = unsafe { (self.latency)(self.handle, &mut err) };
            if l == u64::MAX { 0 } else { l }
        }
    }

    impl Drop for Simple {
        fn drop(&mut self) {
            // SAFETY: handle came from pa_simple_new and is freed once.
            unsafe { (self.free)(self.handle) }
        }
    }
}

/// SUBTITLES (R210 family): WebVTT and SRT to cues, the cue at a time,
/// fetching a track, finding one beside a video file (`film.srt`).
pub mod subtitles {
    use std::path::{Path, PathBuf};

    /// One subtitle line on screen.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Cue {
        pub from: f64,
        pub to: f64,
        pub text: String,
    }

    fn time(t: &str) -> Option<f64> {
        let t = t.trim().split_whitespace().next()?.replace(',', ".");
        let parts: Vec<&str> = t.split(':').collect();
        let (h, m, s) = match parts.as_slice() {
            [h, m, s] => (h.parse::<f64>().ok()?, m.parse::<f64>().ok()?, s.parse::<f64>().ok()?),
            [m, s] => (0.0, m.parse::<f64>().ok()?, s.parse::<f64>().ok()?),
            _ => return None,
        };
        Some(h * 3600.0 + m * 60.0 + s)
    }

    fn untag(line: &str) -> String {
        let mut out = String::new();
        let mut inside = false;
        for c in line.chars() {
            match c {
                '<' | '{' => inside = true,
                '>' | '}' => inside = false,
                _ if !inside => out.push(c),
                _ => {}
            }
        }
        out.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&nbsp;", " ").trim().to_string()
    }

    /// WebVTT or SRT (both: a timing line `a --> b`, text lines below, a
    /// blank line between cues; SRT numbers its cues and writes `00:00:01,000`).
    /// Tags (<i>, <c>, {\an8}) are stripped; the rolling repeats of
    /// automatic captions (each cue repeating the line before) are folded.
    pub fn parse(text: &str) -> Vec<Cue> {
        let mut cues: Vec<Cue> = Vec::new();
        let mut lines = text.lines().peekable();
        while let Some(line) = lines.next() {
            let Some((a, b)) = line.split_once("-->") else { continue };
            let (Some(from), Some(to)) = (time(a), time(b)) else { continue };
            let mut body: Vec<String> = Vec::new();
            while let Some(next) = lines.peek() {
                if next.trim().is_empty() {
                    break;
                }
                let t = untag(next);
                if !t.is_empty() {
                    body.push(t);
                }
                lines.next();
            }
            if let (Some(prev), Some(first)) = (cues.last(), body.first()) {
                if prev.text.ends_with(first.as_str()) {
                    body.remove(0);
                }
            }
            let text = body.join("\n");
            if !text.is_empty() && to > from + 0.02 {
                cues.push(Cue { from, to, text });
            }
        }
        cues
    }

    /// The cue to show at `t` seconds.
    pub fn cue_at(cues: &[Cue], t: f64) -> Option<&Cue> {
        cues.iter().rev().find(|c| c.from <= t && t < c.to)
    }

    /// Fetch a track from the web (empty on failure).
    pub fn fetch(url: &str) -> Vec<Cue> {
        std::process::Command::new("curl")
            .args(["-s", "-L", "--max-time", "15"])
            .arg(url)
            .output()
            .ok()
            .map(|o| parse(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or_default()
    }

    /// A subtitle file beside a video (`film.mkv` → `film.srt`, `film.vtt`,
    /// `film.de.srt` …), the first that parses.
    pub fn sidecar(video: &Path) -> Option<(PathBuf, Vec<Cue>)> {
        let dir = video.parent()?;
        let stem = video.file_stem()?.to_string_lossy().to_string();
        let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                name.starts_with(&stem) && (name.ends_with(".srt") || name.ends_with(".vtt"))
            })
            .collect();
        found.sort();
        found.into_iter().find_map(|p| {
            let cues = parse(&std::fs::read_to_string(&p).ok()?);
            (!cues.is_empty()).then_some((p, cues))
        })
    }
}

/// THE REMOTE (R210 family): one window plays, another steers it — on a
/// duo the stage and the list (Matrix Tube). Words travel through
/// `mk::companion`; this is their grammar, typed.
/// THE PLAY QUEUE (R210 family, 02.11.2026): a list played in order — a
/// playlist of files, an album. The whole list stays (a player shows it and
/// steps back); `at` is the one playing. the owner: «I can't see what's
/// coming next» / «can't go back».
pub mod queue {
    /// What «back» means at this moment.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Back {
        /// Into the entry: back to its start.
        Restart,
        /// At its start: the entry before.
        To(usize),
    }

    /// Seconds into an entry after which «back» restarts it (as the big
    /// players do).
    pub const RESTART_AFTER_S: f32 = 3.0;

    pub fn back_step(position_s: f32, at: usize) -> Back {
        if position_s > RESTART_AFTER_S || at == 0 {
            Back::Restart
        } else {
            Back::To(at - 1)
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Queue<T> {
        items: Vec<T>,
        at: usize,
    }

    impl<T> Default for Queue<T> {
        fn default() -> Self {
            Queue { items: Vec::new(), at: 0 }
        }
    }

    impl<T> Queue<T> {
        pub fn new(items: Vec<T>) -> Queue<T> {
            Queue { items, at: 0 }
        }

        pub fn items(&self) -> &[T] {
            &self.items
        }

        pub fn into_items(self) -> Vec<T> {
            self.items
        }

        pub fn at(&self) -> usize {
            self.at
        }

        pub fn len(&self) -> usize {
            self.items.len()
        }

        pub fn is_empty(&self) -> bool {
            self.items.is_empty()
        }

        pub fn current(&self) -> Option<&T> {
            self.items.get(self.at)
        }

        pub fn has_next(&self) -> bool {
            self.at + 1 < self.items.len()
        }

        /// Go to entry `k` (None when there is none).
        pub fn jump(&mut self, k: usize) -> Option<&T> {
            if k < self.items.len() {
                self.at = k;
            }
            self.items.get(k)
        }

        /// The next entry, moved to.
        pub fn advance(&mut self) -> Option<&T> {
            if !self.has_next() {
                return None;
            }
            self.at += 1;
            self.items.get(self.at)
        }

        pub fn clear(&mut self) {
            self.items.clear();
            self.at = 0;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn queue_steps_and_back() {
            let mut q = Queue::new(vec!["a", "b", "c"]);
            assert_eq!(q.current(), Some(&"a"));
            assert_eq!(q.advance(), Some(&"b"));
            assert_eq!(q.jump(2), Some(&"c"));
            assert!(!q.has_next());
            assert_eq!(q.advance(), None);
            assert_eq!(q.jump(7), None);
            assert_eq!(q.at(), 2);
            assert_eq!(back_step(10.0, 2), Back::Restart);
            assert_eq!(back_step(1.0, 2), Back::To(1));
            assert_eq!(back_step(1.0, 0), Back::Restart);
        }
    }
}

pub mod remote {
    use serde_json::{json, Value};

    /// What the steering window asks.
    #[derive(Debug, Clone, PartialEq)]
    pub enum Command {
        Play(String),
        Toggle,
        Seek(f64),
        /// Highest picture height (the quality menu).
        Quality(u32),
        /// A subtitle language, or None = off.
        Subs(Option<String>),
        /// Subtitle size step (`mk::cinema::subtitle::SIZES`).
        SubSize(usize),
        /// Autoplay on or off (the next video starts at the end).
        Autoplay(bool),
        /// Not the next one this time (the countdown stops).
        Cancel,
        /// Sleep timer: minutes, 0 = at the end of this video, None = off.
        Sleep(Option<u32>),
        /// «Are you still there?» — yes (play on).
        StillHere,
        /// Play local files in this order (a playlist, or one download).
        PlayFiles(Vec<String>),
        /// Jump to this entry of the running playlist.
        QueueAt(usize),
        /// Round 122: end the stage quietly (no background sound) — the list
        /// takes the video over (the duo folded to one panel).
        Quit,
    }

    impl Command {
        pub fn word(&self) -> String {
            match self {
                Command::Play(id) => format!("play {id}"),
                Command::Toggle => String::from("toggle"),
                Command::Seek(t) => format!("seek {t:.2}"),
                Command::Quality(h) => format!("quality {h}"),
                Command::Subs(lang) => format!("subs {}", lang.as_deref().unwrap_or("off")),
                Command::SubSize(s) => format!("subsize {s}"),
                Command::Autoplay(on) => format!("autoplay {}", if *on { "on" } else { "off" }),
                Command::Cancel => String::from("cancel"),
                Command::Sleep(m) => format!("sleep {}", m.map(|m| m.to_string()).unwrap_or_else(|| String::from("off"))),
                Command::StillHere => String::from("here"),
                Command::PlayFiles(files) => format!("files {}", files.join("\t")),
                Command::QueueAt(k) => format!("queueat {k}"),
                Command::Quit => String::from("quit"),
            }
        }

        pub fn parse(word: &str) -> Option<Command> {
            let (verb, rest) = word.trim().split_once(' ').unwrap_or((word.trim(), ""));
            Some(match verb {
                "play" if !rest.is_empty() => Command::Play(rest.to_string()),
                "toggle" => Command::Toggle,
                "seek" => Command::Seek(rest.parse().ok()?),
                "quality" => Command::Quality(rest.parse().ok()?),
                "subs" => Command::Subs((rest != "off" && !rest.is_empty()).then(|| rest.to_string())),
                "subsize" => Command::SubSize(rest.parse().ok()?),
                "autoplay" => Command::Autoplay(rest == "on"),
                "cancel" => Command::Cancel,
                "sleep" => Command::Sleep(if rest == "off" { None } else { Some(rest.parse().ok()?) }),
                "here" => Command::StillHere,
                "queueat" => Command::QueueAt(rest.parse().ok()?),
                "quit" => Command::Quit,
                "files" if !rest.is_empty() => Command::PlayFiles(rest.split('\t').map(String::from).collect()),
                _ => return None,
            })
        }
    }

    /// What the playing window reports.
    #[derive(Debug, Clone, Default, PartialEq)]
    pub struct PlayerState {
        pub id: String,
        pub title: String,
        pub channel: String,
        pub position: f32,
        pub duration: f32,
        pub playing: bool,
        /// Seconds fetched ahead (infinite = all).
        pub ahead: Option<f64>,
        pub marks: Vec<(f32, f32)>,
        pub quality: u32,
        /// Offered subtitle tracks: (language, name).
        pub subs: Vec<(String, String)>,
        pub sub: Option<String>,
        /// Subtitle size step.
        pub sub_size: usize,
        pub status: String,
        /// Autoplay on, and the seconds left before the next one starts.
        pub autoplay: bool,
        pub countdown: Option<f32>,
        /// What comes next: (id, title, channel, length).
        pub up_next: Vec<(String, String, String, String)>,
        /// Sleep timer: the choice (minutes, 0 = video end) and seconds left.
        pub sleep: Option<u32>,
        pub sleep_left: Option<f32>,
        /// «Are you still there?» asked: seconds until the screens go off.
        pub asking: Option<f32>,
        /// A playlist of local files: its name, every entry (path, title,
        /// channel, length) and which one plays.
        pub playlist: Option<String>,
        pub queue: Vec<(String, String, String, String)>,
        pub queue_at: usize,
    }

    impl PlayerState {
        /// The status word: `state <json>`.
        pub fn word(&self) -> String {
            let v = json!({
                "id": self.id, "title": self.title, "channel": self.channel,
                "pos": self.position, "dur": self.duration, "playing": self.playing,
                "ahead": self.ahead.map(|a| if a.is_finite() { a } else { -1.0 }),
                "marks": self.marks.iter().map(|(a, b)| json!([a, b])).collect::<Vec<_>>(),
                "quality": self.quality,
                "subs": self.subs.iter().map(|(l, n)| json!([l, n])).collect::<Vec<_>>(),
                "sub": self.sub, "subsize": self.sub_size, "status": self.status,
                "autoplay": self.autoplay, "countdown": self.countdown,
                "next": self.up_next.iter().map(|(i, t, c, l)| json!([i, t, c, l])).collect::<Vec<_>>(),
                "sleep": self.sleep, "sleepleft": self.sleep_left, "asking": self.asking,
                "playlist": self.playlist, "queueat": self.queue_at,
                "queue": self.queue.iter().map(|(i, t, c, l)| json!([i, t, c, l])).collect::<Vec<_>>(),
            });
            format!("state {v}")
        }

        /// Rows of four strings (id, title, channel, length).
        fn quads(v: Option<&Value>) -> Vec<(String, String, String, String)> {
            v.and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|t| {
                            let f = |k: usize| t.get(k).and_then(Value::as_str).map(String::from);
                            Some((f(0)?, f(1)?, f(2).unwrap_or_default(), f(3).unwrap_or_default()))
                        })
                        .collect()
                })
                .unwrap_or_default()
        }

        pub fn parse(word: &str) -> Option<PlayerState> {
            let v: Value = serde_json::from_str(word.strip_prefix("state ")?).ok()?;
            let f = |k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(0.0);
            let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
            Some(PlayerState {
                id: s("id"),
                title: s("title"),
                channel: s("channel"),
                position: f("pos") as f32,
                duration: f("dur") as f32,
                playing: v.get("playing").and_then(Value::as_bool).unwrap_or(false),
                ahead: v.get("ahead").and_then(Value::as_f64).map(|a| if a < 0.0 { f64::INFINITY } else { a }),
                marks: v
                    .get("marks")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(|m| Some((m.get(0)?.as_f64()? as f32, m.get(1)?.as_f64()? as f32))).collect())
                    .unwrap_or_default(),
                quality: v.get("quality").and_then(Value::as_u64).unwrap_or(0) as u32,
                subs: v
                    .get("subs")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(|t| Some((t.get(0)?.as_str()?.to_string(), t.get(1)?.as_str()?.to_string()))).collect())
                    .unwrap_or_default(),
                sub: v.get("sub").and_then(Value::as_str).map(String::from),
                sub_size: v.get("subsize").and_then(Value::as_u64).unwrap_or(1) as usize,
                status: s("status"),
                autoplay: v.get("autoplay").and_then(Value::as_bool).unwrap_or(true),
                countdown: v.get("countdown").and_then(Value::as_f64).map(|c| c as f32),
                up_next: Self::quads(v.get("next")),
                sleep: v.get("sleep").and_then(Value::as_u64).map(|m| m as u32),
                sleep_left: v.get("sleepleft").and_then(Value::as_f64).map(|s| s as f32),
                asking: v.get("asking").and_then(Value::as_f64).map(|s| s as f32),
                playlist: v.get("playlist").and_then(Value::as_str).map(String::from),
                queue: Self::quads(v.get("queue")),
                queue_at: v.get("queueat").and_then(Value::as_u64).unwrap_or(0) as usize,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rates_parse() {
        assert!((parse_rate("30000/1001") - 29.97).abs() < 0.01);
        assert_eq!(parse_rate("25"), 25.0);
        assert_eq!(parse_rate("0/0"), 0.0);
    }

    #[test]
    fn decode_size_is_even_and_keeps_the_aspect() {
        assert_eq!(decode_size(1920, 1080, 1350), (1350, 758));
        assert_eq!(decode_size(640, 360, 1350), (640, 360));
        let (w, h) = decode_size(1081, 607, 4000);
        assert_eq!((w % 2, h % 2), (0, 0));
    }

    #[test]
    fn web_inputs_are_recognised() {
        assert!(Input { url: "https://x".into(), headers: vec![], byterate: 0.0 }.is_web());
        assert!(!Input::file(Path::new("/tmp/a.webm")).is_web());
    }

    #[test]
    fn vtt_and_srt_become_cues() {
        let vtt = "WEBVTT\n\n00:00:01.000 --> 00:00:02.500 align:start\nHallo <c>Welt</c>\n\n00:00:02.500 --> 00:00:04.000\nHallo Welt\nwie geht's\n\n01:02.000 --> 01:03.000\nA &amp; B\n";
        let c = subtitles::parse(vtt);
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].text, "Hallo Welt");
        assert_eq!(c[1].text, "wie geht's");
        assert_eq!(c[2].from, 62.0);
        assert_eq!(subtitles::cue_at(&c, 3.0).map(|c| c.text.as_str()), Some("wie geht's"));
        let srt = "1\n00:00:01,000 --> 00:00:02,000\n<i>Erste</i> Zeile\n\n2\n00:00:03,500 --> 00:00:05,000\n{\\an8}Zweite\n";
        let s = subtitles::parse(srt);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0], subtitles::Cue { from: 1.0, to: 2.0, text: "Erste Zeile".into() });
        assert_eq!(s[1].text, "Zweite");
    }

    #[test]
    fn remote_words_round_trip() {
        use remote::{Command, PlayerState};
        for c in [Command::Play("abc".into()), Command::Toggle, Command::Seek(12.5), Command::Quality(480), Command::Subs(None), Command::Subs(Some("de".into())), Command::SubSize(3), Command::Autoplay(false), Command::Autoplay(true), Command::Cancel, Command::Sleep(None), Command::Sleep(Some(0)), Command::Sleep(Some(90)), Command::StillHere, Command::PlayFiles(vec!["/a b.mp3".into(), "/c.mp4".into()]), Command::QueueAt(2)] {
            assert_eq!(Command::parse(&c.word()), Some(c));
        }
        let st = PlayerState { id: "x".into(), position: 3.0, duration: 9.0, playing: true, ahead: Some(f64::INFINITY), marks: vec![(1.0, 2.0)], quality: 720, subs: vec![("de".into(), "Deutsch".into())], sub: Some("de".into()), sub_size: 2, autoplay: true, countdown: Some(3.5), up_next: vec![("id".into(), "T".into(), "K".into(), "1:00".into())], sleep: Some(90), sleep_left: Some(12.5), asking: None, ..Default::default() };
        assert_eq!(PlayerState::parse(&st.word()), Some(st));
    }
}
