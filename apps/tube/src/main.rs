//! Matrix Tube — App (R210): YouTube in the Matrix house, without iced.
//!
//! The functions of Red (search, play without advertising, SponsorBlock)
//! in a Matrix app of our own: the light shell draws (GPU painter), the
//! scene families give the window skin (`fam::Chrome`) and the field,
//! `matrixkit-tube` searches and resolves, `matrixkit-video` plays picture
//! and sound.
//!
//! The window from top to bottom: head, stage (the video, letterboxed on
//! black), the control bar (play/pause, timeline, time), the search field,
//! the hits (thumbnail, title, channel · views · age). A finger scrolls the
//! list; a tap that did not move picks a hit. Everything slow (search,
//! thumbnails, resolving, SponsorBlock) runs on the loom and comes home
//! through `woven`.

mod library;
mod remote;

use matrixkit_image::{Fit, ImageAtlas};
use matrixkit_scene::scene as fam;
use matrixkit_scene::symbol;
use matrixkit_shell::{self as shell, Loom, Pict, Refresh, Scene, SceneBuilder, ScenePart, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{Keep, Rect};
use matrixkit_tube::{Cue, Hit, Skip, Watch};
use matrixkit_video::{Spec, Video};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

const APP_ID: &str = "matrix-tube";
/// The companion window on the other panel (`mk::companion`).
const STAGE_ROLE: &str = "buehne";
const WIDTH: u32 = 460;
const HEIGHT: u32 = 820;
/// Highest picture asked for (the duo decodes 1080p AV1 ~3x real time).
const MAX_HEIGHT: u32 = 1080;
/// Widest frame that travels through the pipe: the GPU stretches it onto
/// the stage (Fit::Stretch). Panel-sized RGBA was 146 MB/s of copying.
const PIPE_WIDTH: u32 = 1280;
/// YUV frames are 1.5 bytes a pixel and need no conversion in ffmpeg — up
/// to 1080p they travel at their own size (no scaling at all).
const PIPE_WIDTH_YUV: u32 = 1920;

// Roles.
const CHROME_BASE: usize = 500;
/// The root pane's roles (settings, about, help — `fam::RootPane`).
const ROOT_BASE: usize = 700;
/// The media bar's roles (`fam::MediaBar`: play key, timeline, previous,
/// next). Round 172: from 1 to 150 — its two new keys (base + 2, + 3)
/// would have been the search field (3) and «back» (4).
const BAR_BASE: usize = 150;
const FIELD: usize = 3;
const ROW_BASE: usize = 1000;

const THUMB_W: f32 = 128.0;
const THUMB_H: f32 = 72.0;
/// A finger that travelled less than this was a tap, not a scroll.
/// Quiet seconds before the list's panel switches its light off.
const PANEL_OFF_S: f32 = 120.0;

/// How this window is used (R210 duo form).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Everything in one window (laptop, tablet).
    Whole,
    /// Duo: search and list only — a pick goes to the stage.
    List,
    /// Duo: the video only, on the other panel (`--buehne`).
    Stage,
}

/// What comes home from the loom.
enum Work {
    /// A word of the list to the stage (`remote::obey`).
    Word(String),
    /// The stage's state for the list's player view.
    Status(String),
    /// Subtitle cues fetched for a language.
    Cues(String, Vec<Cue>),
    /// YouTube's «Als Nächstes» (up next) for a video.
    Related(String, Vec<Hit>),
    Hits(u64, Vec<Hit>),
    Thumb(String, Option<PathBuf>),
    Resolved(String, Result<Watch, String>),
    Skips(String, Vec<Skip>),
    /// A download's percent, and its end (`library`).
    DlProgress(f32),
    DlDone(Result<matrixkit_tube::library::Item, String>),
}

/// What the list shows — the cache key of its built scene part.
#[derive(Clone, PartialEq)]
struct ListKey {
    size: (u32, u32),
    top: u32,
    roll: u32,
    hits: u64,
    thumbs: usize,
    current: Option<String>,
    searching: bool,
    empty_search: bool,
    /// Recent searches shown and downloads known (the «✓» marks).
    history: usize,
    kept: usize,
    compact: bool,
}

struct Tube {
    mode: Mode,
    /// Frames per second asked of ffmpeg: 30, or the panel clock when that
    /// is slower (duo: 21.9 Hz) — even pacing instead of uneven drops.
    max_fps: f64,
    /// The stage has moved itself beside the list (once).
    placed: bool,
    palette: mk::Palette,
    chrome: fam::Chrome,
    /// Settings and about (title press, or the top bar's app menu on a duo).
    root: fam::RootPane,
    loom: Option<Loom<Work>>,
    /// A video id from the command line (a link) — played at start.
    direct: Option<String>,
    // The search field.
    search: String,
    cursor: usize,
    anchor: Option<usize>,
    blink: bool,
    field_focus: bool,
    searching: bool,
    search_seq: u64,
    hits: Vec<Hit>,
    hits_rev: u64,
    thumbs: HashMap<String, PathBuf>,
    thumbs_asked: HashSet<String>,
    // The list, its scroll and its cached scene part (during playback only
    // the stage changes — rebuilding the list each frame was wasted work).
    list: Rect,
    list_cache: Option<(ListKey, ScenePart, f32)>,
    roll: mk::motion::Spring,
    roll_max: f32,
    /// A finger on a list: tap (the row it means) or scroll (mk::touch).
    press: Option<mk::touch::Press<Option<usize>>>,
    // The video.
    current: Option<Hit>,
    status: String,
    watch: Option<Watch>,
    video: Option<Video>,
    /// Round 53: skipped frames already logged.
    skipped_seen: u64,
    skips: Vec<Skip>,
    /// The youngest frame waiting to be laid into the atlas.
    fresh: Option<(u64, u32, u32, Vec<u8>)>,
    /// The running number of the frame shown.
    shown: Option<u64>,
    /// The stage box (logical) the frame is stretched onto.
    stage_box: (u32, u32),
    /// Round 172: little room (a keyboard up, one panel lying wide) — the
    /// tabs move beside the field, the hint gives way to the history.
    compact: bool,
    /// Round 172: the list's column starts here (landscape: video left).
    col_dx: f32,
    /// Round 172: the stage may grow to the window's height (its column).
    stage_tall: bool,
    /// Play key, timeline with stock shadow and SponsorBlock marks.
    bar: fam::MediaBar,
    /// Highest picture asked for (the quality menu; 0 = Auto).
    quality: u32,
    /// Auto stepped down to 720p: the decoder fell behind the sound.
    auto_down: bool,
    /// Since when the picture lags the sound by more than 0.4 s.
    behind_since: Option<std::time::Instant>,
    /// Where a restarted video (new quality) begins.
    resume_at: f64,
    /// The subtitle language shown and its cues (stage).
    sub_lang: Option<String>,
    cues: Vec<Cue>,
    /// The stage's last report to the list.
    last_report: std::time::Instant,
    /// Duo list: the player view instead of the search (back = search).
    player_view: bool,
    /// Cinema mode announced (`mk::cinema`): the backgrounds dim while
    /// the video plays.
    kino: mk::cinema::Announcer,
    /// Duo stage: the window takes the picture's shape (`fam::PictureFit`).
    picture_fit: fam::PictureFit,
    /// Subtitle size step and the line last handed to the dock's strip.
    sub_size: usize,
    strip_text: Option<String>,
    /// Autoplay (stage): what comes next, whether it is on, when the
    /// countdown began, and whether it was called off for this video.
    up_next: Vec<Hit>,
    autoplay: bool,
    next_at: Option<std::time::Instant>,
    next_cancelled: bool,
    /// The player view's unfolded chip: 0 none, 1 quality, 2 subtitles,
    /// 3 sleep timer.
    unfolded: u8,
    /// The list steps back while nobody touches it (cinema mode).
    idle: fam::IdleDim,
    /// The list's panel light is off (2 min quiet while watching).
    panel_dark: bool,
    /// Sleep timer (stage): the choice, when it runs out, and since when
    /// «Bist du noch da?» (are you still there?) waits for an answer.
    sleep_choice: Option<u32>,
    sleep_at: Option<std::time::Instant>,
    asking_since: Option<std::time::Instant>,
    /// The list's player view scrolls (it outgrew the panel).
    player_roll: mk::motion::Spring,
    player_max: f32,
    /// What the stage reported last.
    remote: Option<matrixkit_video::remote::PlayerState>,
    /// Round 122: started as the duo's list (two windows) — and the duo is
    /// folded to one panel right now (then this window plays itself, the
    /// video above the list, like the laptop's one window).
    duo_list: bool,
    one_panel: bool,
    /// Round 134: the session was locked at the last beat (on unlocking,
    /// the dimmed list lights up again).
    was_locked: bool,
    /// Round 135: local files whose still was asked for (once each).
    poster_asked: std::collections::HashSet<String>,
    /// The stage, started again on unfolding, jumps here once it plays.
    pending_seek: Option<(String, f64)>,
    note: mk::Flash,
    /// Frame cadence measured over five seconds (stutter hunt, R210).
    meter: shell::FrameMeter,
    /// The library (`library`): downloads and playlists, the tab shown,
    /// the row whose «+» is open, and whether the field names a playlist.
    library: matrixkit_tube::library::Library,
    tab_library: bool,
    lib_open: Option<PathBuf>,
    naming: bool,
    /// A library key pressed — it acts on release unless the finger moved.
    tap: Option<usize>,
    /// The playlist whose «⋯» is open, the playlist the field renames, and
    /// whether the open thing's «Löschen» was tapped once (asks again).
    pl_open: Option<usize>,
    renaming: Option<usize>,
    confirm: bool,
    /// Recent searches (newest first), tappable when the field is empty.
    history: Vec<String>,
    /// Stage: the files still to play (a playlist), and a sound file's cover.
    queue: matrixkit_video::queue::Queue<PathBuf>,
    queue_name: Option<String>,
    /// Each entry as (path, title, channel, length) for the list.
    queue_info: Vec<(String, String, String, String)>,
    cover: Option<PathBuf>,
    /// A running download (kind, percent) and the note after it.
    /// Downloads: one runs, the others wait (matrixkit_tube::downloads).
    downloads: matrixkit_tube::downloads::Queue,
    dl_note: mk::Flash,
}

fn zero() -> Rect {
    Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }
}

impl Tube {
    fn new() -> Self {
        Tube {
            mode: Mode::Whole,
            max_fps: 30.0,
            placed: false,
            palette: mk::Palette::load().unwrap_or_default(),
            chrome: fam::Chrome::new("Matrix Tube", CHROME_BASE),
            root: fam::RootPane::new(APP_ID, &[]),
            loom: None,
            direct: None,
            search: String::new(),
            cursor: 0,
            anchor: None,
            blink: true,
            field_focus: true,
            searching: false,
            search_seq: 0,
            hits: Vec::new(),
            hits_rev: 0,
            thumbs: HashMap::new(),
            thumbs_asked: HashSet::new(),
            list: zero(),
            list_cache: None,
            roll: mk::motion::Spring::smooth(0.0),
            roll_max: 0.0,
            press: None,
            current: None,
            status: String::new(),
            watch: None,
            video: None,
            skipped_seen: 0,
            skips: Vec::new(),
            fresh: None,
            shown: None,
            stage_box: (0, 0),
            compact: false,
            col_dx: 0.0,
            stage_tall: false,
            bar: fam::MediaBar::new(BAR_BASE),
            quality: 0,
            auto_down: false,
            behind_since: None,
            resume_at: 0.0,
            sub_lang: None,
            cues: Vec::new(),
            last_report: std::time::Instant::now(),
            player_view: false,
            kino: mk::cinema::Announcer::new(),
            picture_fit: fam::PictureFit::new(APP_ID),
            sub_size: mk::cinema::subtitle::saved_size(),
            strip_text: None,
            up_next: Vec::new(),
            autoplay: mk::settings::read("tube-autoplay").as_deref() != Some("off"),
            next_at: None,
            next_cancelled: false,
            unfolded: 0,
            idle: fam::IdleDim::new(8.0),
            panel_dark: false,
            sleep_choice: None,
            sleep_at: None,
            asking_since: None,
            player_roll: mk::motion::Spring::smooth(0.0),
            player_max: 0.0,
            remote: None,
            duo_list: false,
            one_panel: false,
            was_locked: false,
            poster_asked: std::collections::HashSet::new(),
            pending_seek: None,
            note: mk::Flash::new(2.5),
            meter: shell::FrameMeter::new("tube"),
            library: matrixkit_tube::library::Library::load(),
            tab_library: false,
            lib_open: None,
            pl_open: None,
            tap: None,
            renaming: None,
            confirm: false,
            history: matrixkit_tube::library::history_load(),
            naming: false,
            queue: Default::default(),
            queue_name: None,
            queue_info: Vec::new(),
            cover: None,
            downloads: Default::default(),
            dl_note: mk::Flash::new(6.0),
        }
    }

    fn start_search(&mut self) {
        let query = self.search.trim().to_string();
        if query.is_empty() {
            return;
        }
        matrixkit_tube::library::push_recent(&mut self.history, &query, 8);
        matrixkit_tube::library::history_save(&self.history);
        self.search_seq += 1;
        self.searching = true;
        self.field_focus = false;
        let seq = self.search_seq;
        eprintln!("[tube] Suche «{query}»");
        if let Some(loom) = &self.loom {
            loom.spin(move || Work::Hits(seq, matrixkit_tube::search(&query)));
        }
    }

    fn pick(&mut self, i: usize) -> Refresh {
        let Some(hit) = self.hits.get(i).cloned() else { return Refresh::Calm };
        mk::feel::tap();
        if self.mode == Mode::List {
            stage_play(&hit);
            self.current = Some(hit);
            self.remote = None;
            self.player_view = true;
            return Refresh::Dirty;
        }
        self.play_hit(hit)
    }

    /// Round 122: the duo folded to one panel / opened again. Folded, the
    /// list takes the stage's video over at its place (the stage quits, the
    /// window becomes the one-window form: video above, list below);
    /// opened, the video goes back to the stage on the other panel.
    fn one_panel_switch(&mut self, on: bool) -> Refresh {
        use matrixkit_video::remote::Command;
        self.one_panel = on;
        // Round 135: the change of form as one soft blend.
        mk::canvas::screen_transition(0);
        if on {
            eprintln!("[tube] ein Panel — die Liste spielt selbst");
            let state = self.remote.take();
            remote::to_stage(Command::Quit);
            self.mode = Mode::Whole;
            self.player_view = false;
            if let (Some(hit), Some(st)) = (self.current.clone(), state) {
                if st.id == hit.id && st.duration > 0.0 {
                    let _ = self.play_hit(hit);
                    self.resume_at = st.position as f64;
                }
            }
        } else {
            eprintln!("[tube] zwei Panels — das Video geht auf die Bühne");
            let position = self.video.as_ref().map(|v| v.position_s());
            if let Some(mut v) = self.video.take() {
                v.stop();
            }
            self.watch = None;
            self.mode = Mode::List;
            if let (Some(hit), Some(pos)) = (self.current.clone(), position) {
                stage_play(&hit);
                self.pending_seek = Some((hit.id.clone(), pos));
                self.player_view = true;
            }
        }
        self.list_cache = None;
        Refresh::Full
    }

    fn play_hit(&mut self, hit: Hit) -> Refresh {
        eprintln!("[tube] Wahl {} «{}»", hit.id, hit.title);
        // A video from YouTube ends a running playlist.
        self.queue.clear();
        self.queue_info.clear();
        self.queue_name = None;
        if let Some(mut v) = self.video.take() {
            v.stop();
        }
        self.watch = None;
        self.skips.clear();
        self.shown = None;
        self.fresh = None;
        self.resume_at = 0.0;
        self.sub_lang = None;
        self.cues.clear();
        self.up_next.clear();
        self.next_at = None;
        self.next_cancelled = false;
        self.status = String::from("Lädt …");
        let id = hit.id.clone();
        self.current = Some(hit);
        let quality = self.effective_quality();
        if let Some(loom) = &self.loom {
            let a = id.clone();
            loom.spin(move || Work::Resolved(a.clone(), matrixkit_tube::resolve(&a, quality)));
            let b = id.clone();
            loom.spin(move || Work::Skips(b.clone(), matrixkit_tube::skips(&b)));
            let c = id;
            loom.spin(move || Work::Related(c.clone(), matrixkit_tube::related(&c)));
        }
        Refresh::Dirty
    }

    /// Round 129: publish what plays (now_playing family) and obey the
    /// remotes (the lock screen's player). Some = a command changed things.
    fn now_playing(&mut self) -> Option<Refresh> {
        use matrixkit_theme::now_playing as np;
        let mut changed = None;
        for cmd in np::take() {
            changed = Some(match cmd {
                np::Command::Toggle => self.toggle_pause(),
                np::Command::Seek(t) => {
                    if let Some(v) = &mut self.video {
                        v.play(t);
                    }
                    Refresh::Dirty
                }
                np::Command::Next => {
                    if !self.queue.is_empty() {
                        self.queue_jump(self.queue.at() + 1)
                    } else if let Some(next) = self.up_next.first().cloned() {
                        self.play_hit(next)
                    } else {
                        Refresh::Calm
                    }
                }
                np::Command::Previous => {
                    if let Some(v) = &mut self.video {
                        v.play(0.0);
                    }
                    Refresh::Dirty
                }
            });
        }
        match (&self.video, &self.current) {
            // Any playing medium — a web video or local files (a playlist
            // has no `watch`: it played unseen on the lock, 05.11.2026).
            (Some(v), Some(hit)) => {
                // A web video's thumbnail, a download's cover — or a still.
                let thumb = matrixkit_tube::cache_dir().join("thumbs").join(format!("{}.jpg", hit.id));
                let art = if thumb.exists() {
                    Some(thumb)
                } else if let Some(cover) = self.cover.clone().filter(|c| c.exists()) {
                    Some(cover)
                } else {
                    let file = std::path::PathBuf::from(&hit.id);
                    let still = matrixkit_video::poster::cached(&file);
                    if still.is_none() && file.exists() && self.poster_asked.insert(hit.id.clone()) {
                        let d = v.media.duration_s;
                        std::thread::spawn(move || matrixkit_video::poster::make(&file, d));
                    }
                    still
                };
                np::publish(np::Playing {
                    app: String::from("Matrix Tube"),
                    title: hit.title.clone(),
                    artist: if hit.channel.is_empty() { self.queue_name.clone().unwrap_or_default() } else { hit.channel.clone() },
                    position_s: v.position_s(),
                    duration_s: self.watch.as_ref().map(|w| w.media.duration_s).unwrap_or(v.media.duration_s),
                    playing: !v.paused() && !v.done(),
                    art: art.map(|a| a.to_string_lossy().into_owned()),
                    pid: 0,
                    at_ms: 0,
                    can_next: !self.queue.is_empty() || !self.up_next.is_empty(),
                    can_previous: true,
                });
            }
            _ => np::clear(),
        }
        changed
    }

    /// The height asked of the helper: Auto is 1080p, or 720p once the
    /// decoder could not keep up.
    fn effective_quality(&self) -> u32 {
        match self.quality {
            0 if self.auto_down => 720,
            0 => MAX_HEIGHT,
            h => h,
        }
    }

    /// Load the same video again where it stands (a new quality).
    fn reload_here(&mut self) {
        let Some(hit) = self.current.clone() else { return };
        // A local file has no other quality — never stop it for one.
        if self.watch.is_none() {
            return;
        }
        if let Some(mut v) = self.video.take() {
            self.resume_at = v.position_s();
            v.stop();
        }
        self.watch = None;
        self.shown = None;
        self.fresh = None;
        self.status = String::from("Lädt …");
        let quality = self.effective_quality();
        if let Some(loom) = &self.loom {
            let id = hit.id;
            loom.spin(move || Work::Resolved(id.clone(), matrixkit_tube::resolve(&id, quality)));
        }
    }

    /// Start the resolved video now (the box is the picture's own size
    /// until the stage has been laid — the GPU stretches it anyway).
    fn start_now(&mut self) {
        let Some(watch) = &self.watch else { return };
        if self.stage_box.0 < 2 || self.stage_box.1 < 2 {
            self.stage_box = (watch.media.width.max(2), watch.media.height.max(2));
        }
        if let Some(mut v) = self.video.take() {
            v.stop();
        }
        self.start_video();
    }

    fn start_video(&mut self) {
        let Some(watch) = &self.watch else { return };
        let (bw, bh) = self.stage_box;
        if bw < 2 || bh < 2 {
            return;
        }
        let k = shell::screen_scale();
        // R210: YUV unless the house paints on the CPU (it would convert).
        let yuv = std::env::var_os("MATRIX_CPU").is_none();
        let pipe = if yuv { PIPE_WIDTH_YUV } else { PIPE_WIDTH };
        // YUV: the picture travels at its own size (up to 1080p) — the GPU
        // stretches it. Scaling 1920 to the stage's 1896 cost a full swscale
        // pass per frame and the decoder fell behind the sound (01.11.2026).
        let width = if yuv { pipe } else { (bw * k).min(pipe) };
        let mut video = Video::new(
            watch.media.clone(),
            Spec {
                width,
                exact: None,
                sound_name: String::from("Matrix Tube"),
                sound_only: false,
                max_fps: Some(self.max_fps),
                yuv,
            },
        );
        video.play(self.resume_at);
        self.skipped_seen = 0;
        self.video = Some(video);
        self.status.clear();
    }

    fn toggle_pause(&mut self) -> Refresh {
        // Round 172: the duo's list steers the stage (space did nothing there).
        if self.mode == Mode::List {
            if let Some(state) = &mut self.remote {
                remote::to_stage(matrixkit_video::remote::Command::Toggle);
                state.playing = !state.playing;
                return Refresh::Dirty;
            }
            return Refresh::Calm;
        }
        let Some(video) = &mut self.video else { return Refresh::Calm };
        if video.done() {
            video.play(0.0);
        } else {
            let paused = !video.paused();
            video.set_paused(paused);
        }
        Refresh::Dirty
    }

    fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        if self.tab_library {
            return None;
        }
        let r = self.list;
        let x = x - self.col_dx;
        if x < r.x || x >= r.x + r.w || y < r.y || y >= r.y + r.h {
            return None;
        }
        let local = y - r.y + self.roll.value().max(0.0);
        let row = (local / (THUMB_H + mk::spacing::S)) as usize;
        (row < self.hits.len()).then_some(row)
    }

    /// The stage: black ground, the frame stretched onto its box, a status
    /// or skip note, then the control bar and the chosen video's titles.
    #[allow(clippy::too_many_arguments)]
    fn build_stage(&mut self, engine: &mut TextEngine, b: &mut SceneBuilder, atlas: &mut ImageAtlas, hit: &Hit, wf: f32, hf: f32, mut y: f32) -> f32 {
        let p = self.palette;
        let m = mk::spacing::M;
        let white = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.92 };
        // The picture gets most of the window: a duo panel lies wide
        // (652 × 489), and 45 % shrank a video to 320 × 180.
        // The stage window gives the picture all it has but bar and titles.
        // The duo stage is ONLY the video: the whole window (the owner
        // 01.11.2026 — bar, titles, quality and subtitles live in the list).
        let stage = if self.mode == Mode::Stage {
            Rect { x: 0.0, y, w: wf, h: (hf - y).max(40.0) }
        } else {
            // Round 172: in its own column the picture may take the height
            // (bar and titles below it, ~120 px).
            let cap = if self.stage_tall { hf - y - 120.0 } else { hf * 0.62 };
            Rect { x: 0.0, y, w: wf, h: (wf * 9.0 / 16.0).min(cap).max(40.0).floor() }
        };
        let stage_first = b.quad(stage.x, stage.y, stage.w, stage.h, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, 0.0, Some(BAR_BASE), None);
        // FIT THE STAGE (duo): the window takes the picture's shape.
        if self.mode == Mode::Stage {
            let picture = self.watch.as_ref().filter(|w| w.media.width > 0).map(|w| (w.media.width, w.media.height));
            // A subtitle strip taller than the dock takes room below.
            let below = if self.strip_on() { mk::cinema::subtitle::picture_shrink(self.sub_size) } else { 0.0 };
            self.picture_fit.follow(wf, hf - y, picture, 0.0, below);
        }
        let sound_only = self.watch.as_ref().is_some_and(|w| w.media.width == 0);
        if sound_only {
            self.build_cover(engine, b, stage);
        }
        if let Some(watch) = self.watch.as_ref().filter(|_| !sound_only) {
            let fitted = fam::letterbox(stage, watch.media.width, watch.media.height);
            let fit = (fitted.w as u32, fitted.h as u32);
            if fit != self.stage_box {
                // Stretched by the GPU — a new box needs no restart.
                self.stage_box = fit;
                if self.video.is_none() {
                    self.start_video();
                }
            }
        }
        if let Some((nr, fw, fh, bytes)) = self.fresh.take() {
            let key = PathBuf::from(format!("\u{0}tube:{nr}"));
            match self.video.as_ref() {
                Some(v) => v.lay_frame(atlas, &key, fw, fh, bytes),
                None => atlas.put_frame(&key, fw, fh, bytes, false),
            }
            self.shown = Some(nr);
        }
        if let (Some(nr), (bw, bh)) = (self.shown, self.stage_box) {
            b.pict(Pict {
                path: PathBuf::from(format!("\u{0}tube:{nr}")),
                box_w: bw,
                box_h: bh,
                fit: Fit::Stretch,
                x: stage.x + (stage.w - bw as f32) / 2.0,
                y: stage.y + (stage.h - bh as f32) / 2.0,
                radius: 0.0,
                home: stage,
                anchor: stage_first,
                clip: None,
            });
        }
        let note = self.note.text().map(String::from);
        if let Some(s) = (!self.status.is_empty()).then(|| self.status.clone()).or(note) {
            let s = mk::elide(&s, 60);
            let st = mk::typo::CALLOUT;
            let (tw, th) = engine.measure_styled(&s, st.px(), st.weight);
            b.styled(engine, s, st, white, (wf - tw) / 2.0, stage.y + stage.h - th - m, None, None);
        }
        self.build_subtitle(engine, b, stage);
        // «Bist du noch da?» over the paused picture (a tap says yes).
        if self.asking_since.is_some() {
            let t = mk::typo::TITLE;
            let line = "Bist du noch da?";
            let hint = "Antippen zum Weiterschauen";
            let (tw, th) = engine.measure_styled(line, t.px(), t.weight);
            let cap = mk::typo::CALLOUT;
            let (hw, hh) = engine.measure_styled(hint, cap.px(), cap.weight);
            let bw = tw.max(hw) + 2.0 * m;
            let bh = th + hh + 2.0 * m + mk::spacing::XS;
            let (bx, by) = (stage.x + (stage.w - bw) / 2.0, stage.y + (stage.h - bh) / 2.0);
            b.quad(bx, by, bw, bh, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.72 }, mk::radius::LARGE, Some(BAR_BASE), None);
            b.styled(engine, line, t, white, bx + (bw - tw) / 2.0, by + m, Some(BAR_BASE), None);
            b.styled(engine, hint, cap, mk::Rgba { a: 0.75, ..white }, bx + (bw - hw) / 2.0, by + m + th + mk::spacing::XS, Some(BAR_BASE), None);
        }
        y += stage.h;
        if self.mode == Mode::Stage {
            return y;
        }

        // The bar: play/pause, timeline with stock shadow and SponsorBlock
        // marks, time (`fam::MediaBar`).
        let marks: Vec<(f32, f32)> = self.skips.iter().map(|s| (s.from as f32, s.to as f32)).collect();
        let view = fam::MediaView {
            position: self.video.as_ref().map(|v| v.position_s() as f32).unwrap_or(0.0),
            duration: self.watch.as_ref().map(|w| w.media.duration_s as f32).unwrap_or(0.0),
            playing: self.video.as_ref().is_some_and(|v| !v.paused() && !v.done()),
            ahead: self.video.as_ref().and_then(|v| v.stocked_s()),
            marks: &marks,
            prev: None,
            next: None,
        };
        y += self.bar.build(engine, b, y, wf, &view, p) + mk::spacing::S;
        let cap = mk::typo::CAPTION;

        // Title and channel of the chosen video.
        let (_, th) = b.styled(engine, mk::elide(&hit.title, 46), mk::typo::SUBTITLE, p.on_surface, m, y, None, None);
        y += th + 2.0;
        let line = fam::dot_join(&[&hit.channel, &hit.views]);
        let (_, ch) = b.styled(engine, mk::elide(&line, 60), cap, p.on_surface_variant, m, y, None, None);
        y + ch + mk::spacing::S
    }

    /// Round 172: the search head — the field, and the tabs under it, or
    /// (compact) beside it as two round keys. Returns its height.
    fn build_head(&self, engine: &mut TextEngine, b: &mut SceneBuilder, wf: f32, y: f32) -> f32 {
        let m = mk::spacing::M;
        if !self.compact {
            let fh = self.build_field(engine, b, wf - 2.0 * m, y);
            let y2 = y + fh + mk::spacing::M;
            return fh + mk::spacing::M + if self.mode == Mode::List { 0.0 } else { self.build_tabs(engine, b, wf, y2) };
        }
        let keys = 2.0 * library::TAB_KEY + mk::spacing::S;
        let fh = self.build_field(engine, b, wf - 2.0 * m - keys - mk::spacing::S, y);
        self.build_tab_keys(engine, b, wf - m - keys, y + (fh - library::TAB_KEY) / 2.0);
        fh + mk::spacing::S
    }

    /// The search field (field family: rim, caret, leading magnifier),
    /// `fw` wide from the margin (round 172: room for the tab keys).
    fn build_field(&self, engine: &mut TextEngine, b: &mut SceneBuilder, fw: f32, y: f32) -> f32 {
        let p = self.palette;
        let m = mk::spacing::M;
        let spec = fam::FieldSpec {
            value: &self.search,
            cursor: self.cursor,
            anchor: self.anchor,
            focused: self.field_focus,
            blink: self.blink,
            placeholder: if self.renaming.is_some() {
                "Neuer Name der Playlist …"
            } else if self.naming {
                "Name der neuen Playlist …"
            } else {
                "Auf YouTube suchen …"
            },
            leading: Some(if self.renaming.is_some() { symbol::EDIT } else if self.naming { symbol::ADD } else { symbol::SEARCH }),
        };
        let field = fam::input_field(engine, &spec, fw, p);
        let (first, _, field_h) = b.node(&field.node, m, y, Some(FIELD), None);
        let ground = Rect::of(&b.scene.placed[first]);
        if let Some(glyph) = &field.leading {
            let gq = b.scene.placed[first + 2];
            b.text_on(
                first + 2,
                Text {
                    s: glyph.clone(),
                    px: mk::typo::symbol_px(mk::font_size::MEDIUM),
                    weight: mk::typo::Weight::Normal,
                    family: matrixkit_scene::symbol_family(),
                    color: p.on_surface_variant,
                    x: gq.x,
                    y: gq.y,
                    home: ground,
                    anchor: 0,
                    clip: None,
                },
            );
        }
        let slot = first + if field.leading.is_some() { 6 } else { 4 };
        let aq = b.scene.placed[slot];
        b.text_on(
            slot,
            Text {
                s: field.label.text.clone(),
                px: field.label.px,
                weight: field.label.weight,
                family: field.label.family,
                color: field.label.color,
                x: aq.x,
                y: aq.y,
                home: ground,
                anchor: 0,
                // Round 173: a long query stays inside the field (with the
                // tab keys beside it, it ran over them).
                clip: Some(ground),
            },
        );
        field_h
    }

    /// The hits as a scene part (anchors from 0) and how far it scrolls.
    fn build_list(&self, engine: &mut TextEngine, wf: f32, hf: f32, y: f32) -> (ScenePart, f32) {
        let p = self.palette;
        let m = mk::spacing::M;
        let mut b = SceneBuilder::new();
        let clip = Some(self.list);
        let off = self.roll.value().max(0.0);
        let row_h = THUMB_H + mk::spacing::S;
        let (body, cap) = (mk::typo::BODY, mk::typo::CAPTION);
        if self.hits.is_empty() {
            let s = if self.searching {
                "Sucht …"
            } else if self.search.is_empty() {
                "Suche oben eingeben — ohne Werbung, mit SponsorBlock."
            } else {
                "Keine Treffer — anders formulieren?"
            };
            // Round 172: little room — the history first, the hint gives way.
            let hint = !(self.compact && self.search.is_empty() && !self.searching && !self.history.is_empty());
            let sh = if hint { b.styled(engine, s, cap, p.on_surface_variant, m, y + m, None, clip).1 + mk::spacing::M } else { 0.0 };
            // Recent searches: one tap searches again (R210 round 19).
            if !self.searching && !self.history.is_empty() {
                let hy = y + m + sh;
                let (_, lh) = b.styled(engine, "Zuletzt gesucht", cap, p.on_surface_variant, m, hy, None, clip);
                fam::lay_choices(&mut b, engine, &self.history, None, m, hy + lh + mk::spacing::XS, wf - 2.0 * m, library::HISTORY_BASE, p);
            }
            return (b.into_part(), 0.0);
        }
        // How many, for what (the list's head).
        let head = format!("{} Treffer", self.hits.len());
        let (_, head_h) = b.styled(engine, head, cap, p.on_surface_variant, m, y - off, None, clip);
        let y = y + head_h + mk::spacing::XS;
        for (i, hit) in self.hits.iter().enumerate() {
            let ry = y + i as f32 * row_h - off;
            if ry + row_h < y || ry > hf {
                continue;
            }
            // Already downloaded: say so first («✓ MP3»).
            let kept: Vec<&str> = self.library.kept(&hit.id).iter().map(|k| if *k == matrixkit_tube::Keep::Mp3 { "✓ MP3" } else { "✓ MP4" }).collect();
            let mut parts: Vec<&str> = kept;
            parts.extend([hit.channel.as_str(), hit.views.as_str(), hit.age.as_str()]);
            let meta = fam::dot_join(&parts);
            let row = fam::MediaRow {
                thumb: self.thumbs.get(&hit.id).map(PathBuf::as_path),
                thumb_w: THUMB_W,
                thumb_h: THUMB_H,
                style: body,
                badge: Some(&hit.length),
                picked: self.current.as_ref().map(|c| &c.id) == Some(&hit.id),
                ..fam::MediaRow::new(&hit.title, &meta)
            };
            fam::lay_media_row(&mut b, engine, &row, m, ry, wf - 2.0 * m, Some(ROW_BASE + i), clip, p);
        }
        let total = head_h + mk::spacing::XS + self.hits.len() as f32 * row_h + m;
        (b.into_part(), (total - self.list.h).max(0.0))
    }

    fn view_inner(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let (wf, hf) = (w as f32, h as f32);
        let mut b = SceneBuilder::new();
        let mut y = self.chrome.build(engine, &mut b, w, h, self.palette);
        if self.chrome.rolled {
            return b.finish();
        }
        // Round 172 (owner: «optimise for all views»): a wide
        // window with a video (one panel lying wide, a TV) puts the video
        // LEFT and search and list RIGHT — under the video only a sliver of
        // list was left.
        let top = y;
        let side = self.mode == Mode::Whole && self.current.is_some() && wf >= 1.3 * hf && wf >= 640.0;
        let left_w = if side { (wf * 0.6).floor() } else { wf };
        self.stage_tall = side;
        self.col_dx = if side { left_w } else { 0.0 };
        if self.mode != Mode::List {
            if let Some(hit) = self.current.clone() {
                y = self.build_stage(engine, &mut b, atlas, &hit, left_w, hf, y);
            }
        }
        if side {
            let cw = wf - left_w;
            let mut col = SceneBuilder::new();
            self.build_column(engine, &mut col, cw, hf, top);
            b.append_shifted(&col.into_part(), left_w);
            return b.finish();
        }
        if self.mode == Mode::List && self.player_view && self.current.is_some() {
            self.build_player(engine, &mut b, wf, hf, y);
            return b.finish();
        }
        if self.mode == Mode::Stage {
            if self.current.is_none() {
                let p = self.palette;
                b.styled(engine, "Bühne — wähle ein Video in der Liste.", mk::typo::CALLOUT, p.on_surface_variant, mk::spacing::M, y + mk::spacing::M, None, None);
            }
            return b.finish();
        }
        self.build_column(engine, &mut b, wf, hf, y);
        b.finish()
    }

    /// The search column — head, the playing row (duo list), tabs, then the
    /// hits or the library — laid from x = 0 across `wf` (round 172: also
    /// as the right column beside the video).
    fn build_column(&mut self, engine: &mut TextEngine, b: &mut SceneBuilder, wf: f32, hf: f32, mut y: f32) {
        // Little room: a keyboard up or one panel lying wide.
        self.compact = hf - y < 430.0;
        y += mk::spacing::S;
        if self.mode == Mode::List && !self.compact {
            // The duo's list: the playing row sits between field and tabs.
            y += self.build_head(engine, b, wf, y);
            y += self.build_now_row(engine, b, wf, y);
            y += self.build_tabs(engine, b, wf, y);
        } else {
            y += self.build_head(engine, b, wf, y);
            if self.mode == Mode::List {
                y += self.build_now_row(engine, b, wf, y);
            }
        }

        self.list = Rect { x: 0.0, y, w: wf, h: (hf - y).max(0.0) };
        if self.tab_library {
            self.build_library(engine, b, wf, hf, y);
            return;
        }
        let key = ListKey {
            size: (wf as u32, hf as u32),
            top: y.to_bits(),
            roll: self.roll.value().to_bits(),
            hits: self.hits_rev,
            thumbs: self.thumbs.len(),
            current: self.current.as_ref().map(|c| c.id.clone()),
            searching: self.searching,
            empty_search: self.search.is_empty(),
            history: self.history.len(),
            kept: self.library.items.len(),
            compact: self.compact,
        };
        if self.list_cache.as_ref().map(|c| &c.0) != Some(&key) {
            let (part, roll_max) = self.build_list(engine, wf, hf, y);
            self.list_cache = Some((key, part, roll_max));
        }
        if let Some((_, part, roll_max)) = &self.list_cache {
            b.append(part);
            self.roll_max = *roll_max;
        }
    }

    /// Round 172: a keyboard's arrows seek ±10 s — the stage's (duo) or
    /// our own video.
    fn seek_by(&mut self, seconds: f64) -> Refresh {
        if self.mode == Mode::List {
            if let Some(state) = &mut self.remote {
                let to = (state.position as f64 + seconds).clamp(0.0, (state.duration as f64).max(0.0));
                remote::to_stage(matrixkit_video::remote::Command::Seek(to));
                state.position = to as f32;
                return Refresh::Dirty;
            }
            return Refresh::Calm;
        }
        if let Some(video) = &mut self.video {
            let to = (video.position_s() + seconds).max(0.0);
            video.play(to);
            return Refresh::Dirty;
        }
        Refresh::Calm
    }
}


impl shell::Client for Tube {
    type Card = ();
    type Work = Work;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &Loom<Work>) {
        self.loom = Some(loom.clone());
        if self.mode == Mode::Stage {
            let feeder = loom.clone();
            mk::companion::listen(APP_ID, STAGE_ROLE, move |word| feeder.feed(Work::Word(word)));
        }
        if self.mode == Mode::List {
            let feeder = loom.clone();
            mk::companion::listen(APP_ID, remote::LIST_ROLE, move |word| {
                if word.starts_with("state ") {
                    feeder.feed(Work::Status(word));
                }
            });
        }
        // `matrix-tube <link>` plays that video, `matrix-tube <words>`
        // starts with that search.
        if self.mode == Mode::Stage && !self.queue.is_empty() {
            let files = std::mem::take(&mut self.queue).into_items();
            let _ = self.play_files(files);
        } else if let Some(id) = self.direct.take() {
            let hit = Hit {
                id,
                title: String::from("YouTube-Link"),
                channel: String::new(),
                length: String::new(),
                views: String::new(),
                age: String::new(),
                snippet: String::new(),
            };
            let _ = self.play_hit(hit);
        } else if !self.search.is_empty() {
            self.start_search();
        }
    }

    fn woven(&mut self, work: Work) -> Refresh {
        match work {
            Work::Word(word) => self.obey(&word),
            Work::Status(state) => {
                let Some(state) = matrixkit_video::remote::PlayerState::parse(&state) else { return Refresh::Calm };
                // Round 122: the stage started again on unfolding — once it
                // plays, it jumps to where the one window was.
                if let Some((id, pos)) = &self.pending_seek {
                    if state.id == *id && state.duration > 0.0 {
                        remote::to_stage(matrixkit_video::remote::Command::Seek(*pos));
                        self.pending_seek = None;
                    }
                }
                // A stage that plays something else than the list thinks
                // (a link) still shows: the list follows it.
                if self.current.as_ref().map(|h| &h.id) != Some(&state.id) && !state.id.is_empty() {
                    self.current = Some(Hit {
                        id: state.id.clone(),
                        title: state.title.clone(),
                        channel: state.channel.clone(),
                        length: String::new(),
                        views: String::new(),
                        age: String::new(),
                        snippet: String::new(),
                    });
                }
                // The recommendations' thumbnails (the list draws them).
                for (nid, ..) in &state.up_next {
                    if self.thumbs_asked.insert(nid.clone()) {
                        let nid = nid.clone();
                        if let Some(loom) = &self.loom {
                            loom.spin(move || {
                                let path = matrixkit_tube::thumbnail(&nid);
                                Work::Thumb(nid, path)
                            });
                        }
                    }
                }
                self.remote = Some(state);
                Refresh::Dirty
            }
            Work::Related(id, hits) => {
                if self.current.as_ref().map(|h| &h.id) == Some(&id) {
                    eprintln!("[tube] Als Nächstes: {} Empfehlungen", hits.len());
                    self.up_next = hits.into_iter().filter(|h| h.id != id).collect();
                }
                Refresh::Calm
            }
            Work::Cues(lang, cues) => {
                if self.sub_lang.as_deref() == Some(lang.as_str()) {
                    eprintln!("[tube] Untertitel «{lang}»: {} Zeilen", cues.len());
                    self.cues = cues;
                }
                Refresh::Dirty
            }
            Work::Hits(seq, hits) => {
                if seq != self.search_seq {
                    return Refresh::Calm;
                }
                eprintln!("[tube] {} Treffer", hits.len());
                self.searching = false;
                self.hits = hits;
                self.hits_rev += 1;
                self.roll = mk::motion::Spring::smooth(0.0);
                for hit in &self.hits {
                    if self.thumbs_asked.insert(hit.id.clone()) {
                        let id = hit.id.clone();
                        if let Some(loom) = &self.loom {
                            loom.spin(move || {
                                let path = matrixkit_tube::thumbnail(&id);
                                Work::Thumb(id, path)
                            });
                        }
                    }
                }
                Refresh::Dirty
            }
            Work::Thumb(id, path) => match path {
                Some(p) => {
                    self.thumbs.insert(id, p);
                    Refresh::Dirty
                }
                None => Refresh::Calm,
            },
            Work::Resolved(id, result) => {
                if self.current.as_ref().map(|h| &h.id) != Some(&id) {
                    return Refresh::Calm;
                }
                match result {
                    Ok(watch) => {
                        eprintln!(
                            "[tube] aufgelöst: {}×{} {:.0} fps, {} Spur(en)",
                            watch.media.width,
                            watch.media.height,
                            watch.media.fps,
                            watch.media.inputs.len()
                        );
                        // A link has no title until now.
                        if let Some(cur) = &mut self.current {
                            if cur.channel.is_empty() {
                                cur.title = watch.media.title.clone();
                                cur.channel = watch.channel.clone();
                            }
                        }
                        self.watch = Some(watch);
                        // Start at once — not on the next view: folded shut
                        // nothing is drawn, and autoplay stalled (02.11.2026).
                        self.start_now();
                    }
                    Err(e) => {
                        eprintln!("[tube] Auflösung scheiterte: {e}");
                        self.status = format!("Nicht abspielbar — {e}");
                    }
                }
                Refresh::Dirty
            }
            Work::DlProgress(pct) => {
                self.downloads.progress(pct);
                Refresh::Dirty
            }
            Work::DlDone(result) => self.download_done(result),
            Work::Skips(id, skips) => {
                if self.current.as_ref().map(|h| &h.id) == Some(&id) {
                    eprintln!("[tube] SponsorBlock: {} Abschnitt(e)", skips.len());
                    self.skips = skips;
                }
                Refresh::Calm
            }
        }
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

    fn view(&mut self, engine: &mut TextEngine, keep: &mut Keep<f32>, atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let t0 = std::time::Instant::now();
        let mut scene = self.view_inner(engine, keep, atlas, w, h);
        // The root pane lies over everything (settings, about).
        if self.root.visible() {
            let mut b = SceneBuilder::new();
            b.scene = scene;
            let about = fam::RootAbout {
                name: "Matrix Tube",
                version: env!("CARGO_PKG_VERSION"),
                description: "YouTube ohne Werbung, mit SponsorBlock, Vorrat und Ton-Uhr — fürs Duo gebaut.",
            };
            self.root.lay(engine, &mut b, &about, w as f32, h as f32, self.palette, ROOT_BASE);
            scene = b.finish();
        }
        // The list steps back in cinema mode while nobody touches it.
        if self.mode == Mode::List && self.idle.level() > 0.0 {
            let mut b = SceneBuilder::new();
            b.scene = scene;
            self.idle.lay(&mut b, w as f32, h as f32);
            scene = b.finish();
        }
        let ms = t0.elapsed().as_secs_f32() * 1000.0;
        self.meter.view(ms);
        scene
    }
    fn animating(&self) -> bool {
        !self.roll.is_settled() || !self.player_roll.is_settled() || !self.idle.settled(self.idle_wanted()) || self.chrome.animating() || self.root.animating() || (self.video.as_ref().is_some_and(|v| v.running()) && self.watch.as_ref().is_some_and(|w| w.media.width > 0) && !(self.mode != Mode::List && mk::device::folded()))
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        let mut dirty = self.chrome.glide(dt);
        dirty |= self.root.glide(dt);
        dirty |= self.roll.tick(dt);
        let mut step_down = false;
        // Folded shut (duo): nobody sees a picture — the decoder waits on its
        // full queue (no work), the sound plays on.
        let folded = self.mode != Mode::List && mk::device::folded();
        if let Some(video) = self.video.as_mut().filter(|_| !folded) {
            if let Some(frame) = video.take_frame(self.fresh.as_ref().map(|f| f.0).or(self.shown)) {
                self.meter.frame(frame.0);
                self.fresh = Some(frame);
                dirty = true;
            }
            let stock = video.stocked_s().map(|s| if s.is_finite() { format!("{s:.0} s") } else { String::from("ganz") }).unwrap_or_else(|| String::from("—"));
            if let Some(line) = self.meter.report(&format!("; Ton-Uhr {:.1} s, Bild zurück {:.0} ms, Vorrat {stock}, Untertitel {}", video.position_s(), video.behind_s() * 1000.0, self.sub_lang.as_deref().unwrap_or("aus"))) {
                eprintln!("{line}");
            }
            // AUTO QUALITY: a picture more than 0.4 s behind the sound for 4 s
            // — the decoder cannot keep up; step down to 720p where it stands.
            // Not for local files (nothing to choose — the re-resolve failed
            // and stopped the music) and not while hidden: behind the lock
            // or with the panels asleep the window gets no frames (05.11.).
            let hidden = mk::session_lock::active() || mk::device::display_asleep();
            if self.quality == 0 && !self.auto_down && self.watch.is_some() && !hidden && !video.paused() && video.behind_s() > 0.4 {
                let since = *self.behind_since.get_or_insert_with(std::time::Instant::now);
                if since.elapsed().as_secs_f32() > 4.0 {
                    eprintln!("[tube] Auto: Bild {:.1} s hinter dem Ton — auf 720p", video.behind_s());
                    self.auto_down = true;
                    self.behind_since = None;
                    self.note.set("Auto: 720p (Gerät kam nicht nach)");
                    step_down = true;
                }
            } else {
                self.behind_since = None;
            }
            // SponsorBlock: inside a marked part, jump to its end.
            if !video.paused() && self.bar.dragging.is_none() {
                if let Some(to) = matrixkit_tube::skip_target(&self.skips, video.position_s()) {
                    let what = self.skips.iter().find(|s| (s.to - to).abs() < 0.01).map(|s| s.category.clone()).unwrap_or_default();
                    eprintln!("[tube] SponsorBlock: «{what}» übersprungen bis {to:.1} s");
                    video.play(to);
                    self.note.set(format!("Übersprungen: {}", category_name(&what)));
                    dirty = true;
                }
            }
        }
        if step_down {
            self.reload_here();
            dirty = true;
        }
        self.feed_strip();
        dirty |= self.sleep_tick();
        dirty |= self.queue_tick();
        dirty |= self.autoplay_tick();
        let wanted = self.idle_wanted();
        dirty |= self.idle.tick(dt, wanted);
        dirty |= self.player_roll.tick(dt);
        self.report();
        if dirty {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
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

    fn strike(&mut self, role: Option<usize>, x: f64, y: f64, source: Source) -> Refresh {
        let (xf, yf) = (x as f32, y as f32);
        // A touch wakes a dimmed list AND acts — the veil is see-through, the
        // list is visible (before, every first touch after 8 s of watching was
        // swallowed: «I can't operate the app», 02.11.2026). Only a
        // panel whose light was OFF swallows the touch: nobody saw what it hit.
        if self.mode == Mode::List {
            let was_dark = self.panel_light(false);
            self.idle.poke();
            if was_dark {
                return Refresh::Dirty;
            }
        }
        // «Bist du noch da?» on the stage: a tap says yes.
        if self.mode != Mode::List && self.asking_since.is_some() {
            return self.still_here();
        }
        if let Some(deed) = self.root.strike_at(role, ROOT_BASE) {
            return match deed {
                fam::RootDeed::Help => {
                    matrixkit_scene::help_open("Matrix Tube");
                    Refresh::Calm
                }
                fam::RootDeed::Calm => Refresh::Calm,
                _ => Refresh::Dirty,
            };
        }
        match self.chrome.strike(role) {
            Some(fam::ChromeDeed::Close) => {
                eprintln!("[tube] Ampel × ({})", source.label());
                return Refresh::Close;
            }
            Some(fam::ChromeDeed::Answer(r)) => return r,
            Some(fam::ChromeDeed::Title) => {
                self.root.toggle();
                return Refresh::Dirty;
            }
            None => {}
        }
        // Library keys act on release, so a moving finger scrolls.
        if let Some(r) = role.filter(|r| library::deferred(*r)) {
            self.press = Some(mk::touch::Press::new(xf, yf, self.roll.target, None));
            self.tap = Some(r);
            return Refresh::Calm;
        }
        if let Some(r) = self.library_strike(role, xf, yf) {
            return r;
        }
        if self.mode == Mode::List {
            if let Some(r) = self.player_strike(role, xf) {
                // A press on nothing — or on a row — may become a scroll.
                if self.player_view && matches!(r, Refresh::Calm) {
                    let row = self.press.take().and_then(|p| p.what);
                    self.press = Some(mk::touch::Press::new(xf, yf, self.player_roll.target, row));
                }
                return r;
            }
        }
        match self.bar.strike(role, xf) {
            Some(fam::MediaDeed::Toggle) => return self.toggle_pause(),
            Some(fam::MediaDeed::Scrub) => return Refresh::Dirty,
            // One window plays one video: no playlist steps here.
            Some(fam::MediaDeed::Prev | fam::MediaDeed::Next) | None => {}
        }
        match role {
            Some(FIELD) => {
                self.field_focus = true;
                self.cursor = self.search.chars().count();
                Refresh::Dirty
            }
            _ => {
                // The list: remember the press — a tap picks on release,
                // a moving finger scrolls.
                if self.list.h > 0.0 && yf >= self.list.y && xf >= self.col_dx {
                    self.press = Some(mk::touch::Press::new(xf, yf, self.roll.target, self.row_at(xf, yf)));
                    if self.field_focus && self.search.is_empty() {
                        self.field_focus = false;
                    }
                }
                Refresh::Calm
            }
        }
    }

    fn travel(&mut self, x: f64, y: f64, _source: Source) -> Refresh {
        if self.bar.travel(x as f32) {
            return Refresh::Dirty;
        }
        // The player view scrolls under a moving finger.
        if self.player_view && self.mode == Mode::List {
            if let Some(roll) = self.press.as_mut().and_then(|p| p.travel(x as f32, y as f32, self.player_max)) {
                self.player_roll = mk::motion::Spring::smooth(roll);
                return Refresh::Dirty;
            }
            return Refresh::Calm;
        }
        if let Some(roll) = self.press.as_mut().and_then(|p| p.travel(x as f32, y as f32, self.roll_max)) {
            self.roll = mk::motion::Spring::smooth(roll);
            return Refresh::Dirty;
        }
        Refresh::Calm
    }

    fn release(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        if let Some(target) = self.bar.release() {
            if self.mode == Mode::List {
                remote::to_stage(matrixkit_video::remote::Command::Seek(target as f64));
                if let Some(state) = &mut self.remote {
                    state.position = target;
                }
                return Refresh::Dirty;
            }
            if let Some(video) = &mut self.video {
                eprintln!("[tube] Sprung zu {target:.1} s");
                video.play(target as f64);
            }
            return Refresh::Dirty;
        }
        if let Some(r) = self.tap.take() {
            let still = self.press.take().map_or(true, |p| !p.scrolling());
            return if still { self.library_strike(Some(r), _x as f32, _y as f32).unwrap_or(Refresh::Calm) } else { Refresh::Calm };
        }
        if let Some(Some(i)) = self.press.take().and_then(mk::touch::Press::release) {
            {
                if self.player_view && self.mode == Mode::List {
                    return self.play_next(i);
                }
                if self.tab_library {
                    return self.play_library_item(i);
                }
                return self.pick(i);
            }
        }
        Refresh::Calm
    }

    fn axis(&mut self, x: f64, y: f64, value: f64, _keep: &mut Keep<f32>) -> Refresh {
        if self.mode == Mode::List {
            self.idle.poke();
            self.panel_light(false);
        }
        if self.player_view && self.mode == Mode::List {
            if self.player_max > 0.0 {
                self.player_roll.retarget((self.player_roll.target + value as f32 * 4.0).clamp(0.0, self.player_max));
                return Refresh::Dirty;
            }
            return Refresh::Calm;
        }
        let r = self.list;
        let lx = x as f32 - self.col_dx;
        let inside = lx >= r.x && lx < r.x + r.w && (y as f32) >= r.y && (y as f32) < r.y + r.h;
        if inside && self.roll_max > 0.0 {
            self.roll.retarget((self.roll.target + value as f32 * 4.0).clamp(0.0, self.roll_max));
            return Refresh::Dirty;
        }
        Refresh::Calm
    }

    fn motion(&mut self, x: f64, y: f64) -> Refresh {
        self.chrome.motion(x, y)
    }

    fn depart(&mut self) -> Refresh {
        self.chrome.depart()
    }

    fn key(&mut self, key: shell::Key, shift: bool) -> Refresh {
        if self.root.key(&key, shift) {
            return Refresh::Dirty;
        }
        if !self.field_focus {
            if key == shell::Key::Char(String::from(" ")) {
                return self.toggle_pause();
            }
            // Round 172: a keyboard steers — arrows seek, Escape goes back
            // from the player to the search.
            match key {
                shell::Key::Left => return self.seek_by(-10.0),
                shell::Key::Right => return self.seek_by(10.0),
                shell::Key::Escape if self.player_view => return self.player_strike(Some(remote::BACK), 0.0).unwrap_or(Refresh::Calm),
                _ => {}
            }
            if !matches!(key, shell::Key::Char(_)) {
                return Refresh::Calm;
            }
            self.field_focus = true;
        }
        match fam::field_edit(&mut self.search, &mut self.cursor, &mut self.anchor, &key, shift) {
            fam::FieldDeed::Changed => Refresh::Dirty,
            fam::FieldDeed::Submitted if self.renaming.is_some() => {
                let at = self.renaming.take().unwrap_or(0);
                if self.library.rename(at, &self.search) {
                    eprintln!("[tube] Playlist umbenannt: «{}»", self.search.trim());
                    self.library.save();
                } else {
                    self.note.set(String::from("Name leer oder schon vergeben"));
                }
                self.search.clear();
                self.cursor = 0;
                self.field_focus = false;
                Refresh::Dirty
            }
            fam::FieldDeed::Submitted if self.naming => {
                // The field named a new playlist.
                self.naming = false;
                let at = self.library.create(&self.search);
                eprintln!("[tube] Playlist «{}»", self.library.playlists[at].name);
                self.library.save();
                self.search.clear();
                self.cursor = 0;
                self.field_focus = false;
                Refresh::Dirty
            }
            fam::FieldDeed::Submitted => {
                self.start_search();
                Refresh::Dirty
            }
            fam::FieldDeed::Escaped => {
                self.field_focus = false;
                self.naming = false;
                self.renaming = None;
                Refresh::Dirty
            }
            fam::FieldDeed::Calm => Refresh::Calm,
        }
    }

    /// The top bar's app menu asks for the settings (duo grammar).
    fn root(&mut self) -> Refresh {
        if self.root.open() {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    /// Closing (×, the bar's menu, the compositor): playing, the sound goes
    /// on in the background (Red's background play).
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
        // Round 129: the board goes with the window (the background sound
        // publishes its own).
        matrixkit_theme::now_playing::clear();
        self.kino.follow(false);
        self.panel_light(false);
        if self.strip_text.take().is_some() {
            mk::cinema::subtitle::clear();
        }
        if let Some(mut v) = self.video.take() {
            if !v.paused() && !v.done() {
                background::hand_over(&v.media, v.position_s(), self.current.as_ref().map(|h| h.id.as_str()));
            }
            v.stop();
        }
    }

    fn beat(&mut self) -> Refresh {
        self.blink = !self.blink;
        // Round 129: what plays goes to the board (the lock screen's
        // player), and its commands come back.
        if self.mode != Mode::List {
            if let Some(r) = self.now_playing() {
                return r;
            }
        }
        // Round 122: folded to one panel / opened again (duo list only).
        if self.duo_list {
            let one = matches!(mk::device::fold(), mk::device::Fold::Folded { .. });
            if one != self.one_panel {
                return self.one_panel_switch(one);
            }
        }
        let _ = self.root.beat();
        // The list's panel: light off after 2 min of only watching, back as
        // soon as it is no longer just watching (pause, question, touch).
        if self.mode == Mode::List {
            // Round 134: back from the lock screen — the list wakes (it
            // looked broken, dark under a freshly opened session).
            let locked = mk::session_lock::active();
            if self.was_locked && !locked {
                self.idle.poke();
                self.panel_light(false);
            }
            self.was_locked = locked;
            let off = !locked && self.idle_wanted() && self.idle.quiet_s() >= PANEL_OFF_S;
            self.panel_light(off);
        }
        // Paused or ended, the stage still reports and counts down.
        let slept = self.sleep_tick();
        // Folded shut the window may not draw: the beat moves the playlist on.
        let moved_on = self.queue_tick();
        let counted = self.autoplay_tick() || slept || moved_on || !self.idle.settled(self.idle_wanted());
        self.report();
        if counted {
            return Refresh::Dirty;
        }
        // The stage window opens where the list is (niri puts new windows
        // on the focused panel) and moves itself to the other one, once.
        if self.mode == Mode::Stage && !self.placed {
            self.placed = true;
            std::thread::spawn(|| mk::canvas::window_to_neighbour("up"));
        }
        // Only the caret blinks — and the time line while paused.
        if self.field_focus || self.downloads.busy() || self.dl_note.alive() || self.video.as_ref().is_some_and(|v| v.paused()) {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    /// The search field types — on a duo the house holds the keyboard.
    fn wants_keyboard(&self) -> bool {
        // Round 134: watching (the list's player view) needs no keyboard —
        // a search field still holding the focus popped it up on unlocking.
        self.field_focus && !(self.mode == Mode::List && self.player_view)
    }

    fn wants_text(&self) -> bool {
        self.field_focus
    }

    fn pulse(&mut self) -> Refresh {
        // Round 178: collect decoders that ended by themselves (no zombies on the end screen).
        if let Some(v) = self.video.as_mut() {
            v.reap();
        }
        // Round 53: skipped frames (hitches) into the runtime log, so a
        // «Mikroruckler» has a number and a time.
        if let Some(v) = &self.video {
            let n = v.skipped();
            if n > self.skipped_seen {
                let line = format!("{} Bild(er) übersprungen, zuletzt {:.0} ms hinter dem Ton, Vorrat {:.0} s",
                    n - self.skipped_seen, v.behind_s() * 1000.0, v.stocked_s().unwrap_or(0.0));
                mk::runtime::trail("matrix-tube-takt.log", &line, 300);
            }
            self.skipped_seen = n;
        }
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
            self.palette = fresh;
            self.list_cache = None;
            return Refresh::Full;
        }
        Refresh::Calm
    }
}

/// German names of the SponsorBlock categories.
fn category_name(c: &str) -> &'static str {
    match c {
        "sponsor" => "Sponsor",
        "selfpromo" => "Eigenwerbung",
        "interaction" => "Abo-Aufruf",
        "intro" => "Intro",
        "outro" => "Abspann",
        _ => "Abschnitt",
    }
}

/// Duo: the stage on the other panel plays `hit` — started if needed.
fn stage_play(hit: &Hit) {
    eprintln!("[tube] an die Bühne: {} «{}»", hit.id, hit.title);
    if let Ok(me) = matrixkit_theme::process::own_exe_result() {
        let mut start = std::process::Command::new(me);
        start.arg("--buehne").arg(&hit.id);
        // The stage writes its log where it can be read later.
        let log = matrixkit_tube::cache_dir().join("buehne.log");
        if let Ok(file) = std::fs::File::create(&log) {
            start.stderr(file);
        }
        mk::companion::send_or_start(APP_ID, STAGE_ROLE, &format!("play {}", hit.id), &mut start);
    }
}

/// BACKGROUND SOUND: a video playing when the window closes goes on as
/// sound only, in a process of its own (`matrix-tube --ton`). The medium
/// and the position travel through a file in the runtime folder; opening
/// Matrix Tube again ends the background sound.
mod background {
    use std::path::PathBuf;

    fn runtime(name: &str) -> PathBuf {
        PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"))).join(name)
    }

    /// From the closing window: write the hand-over, start the player.
    pub fn hand_over(media: &matrixkit_video::Media, position_s: f64, id: Option<&str>) {
        let state = serde_json_lite(media, position_s, id);
        if std::fs::write(runtime("matrix-tube-ton.json"), state).is_err() {
            return;
        }
        let Ok(me) = matrixkit_theme::process::own_exe_result() else { return };
        let mut cmd = std::process::Command::new(me);
        cmd.arg("--ton").stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null());
        // Its own session: the sound outlives the window's process.
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        let _ = cmd.spawn();
        eprintln!("[tube] Ton läuft im Hintergrund weiter ab {position_s:.1} s");
    }

    fn serde_json_lite(media: &matrixkit_video::Media, position_s: f64, id: Option<&str>) -> String {
        let mut v = matrixkit_tube::media_json(media);
        v["position"] = position_s.into();
        // Round 129: the video's id — the background player's picture.
        if let Some(id) = id {
            v["id"] = id.into();
        }
        v.to_string()
    }

    /// At start of the window: end a running background sound.
    pub fn end() {
        let Ok(pid) = std::fs::read_to_string(runtime("matrix-tube-ton.pid")) else { return };
        let Ok(pid) = pid.trim().parse::<i32>() else { return };
        let cmdline = std::fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
        if String::from_utf8_lossy(&cmdline).contains("--ton") {
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
            eprintln!("[tube] Hintergrund-Ton beendet");
        }
        let _ = std::fs::remove_file(runtime("matrix-tube-ton.pid"));
    }

    /// The background player itself (`matrix-tube --ton`).
    pub fn run() {
        let Ok(text) = std::fs::read_to_string(runtime("matrix-tube-ton.json")) else { return };
        let _ = std::fs::remove_file(runtime("matrix-tube-ton.json"));
        let Ok(v) = text.parse::<serde_json::Value>() else { return };
        let Some(media) = matrixkit_tube::media_from_json(&v) else { return };
        let position = v.get("position").and_then(|p| p.as_f64()).unwrap_or(0.0);
        let _ = std::fs::write(runtime("matrix-tube-ton.pid"), std::process::id().to_string());
        let mut video = matrixkit_video::Video::new(
            media,
            matrixkit_video::Spec {
                width: 2,
                exact: None,
                sound_name: String::from("Matrix Tube"),
                sound_only: true,
                max_fps: None,
                yuv: false,
            },
        );
        video.play(position);
        // Round 129: the lock screen's player sees it and steers it.
        use matrixkit_theme::now_playing as np;
        let (title, id) = (video.media.title.clone(), v.get("id").and_then(|i| i.as_str()).unwrap_or_default().to_string());
        let art = matrixkit_tube::cache_dir().join("thumbs").join(format!("{id}.jpg"));
        while !video.done() {
            video.reap();
            for cmd in np::take() {
                match cmd {
                    np::Command::Toggle => {
                        let p = !video.paused();
                        video.set_paused(p);
                    }
                    np::Command::Seek(t) => video.play(t),
                    np::Command::Previous => video.play(0.0),
                    np::Command::Next => {}
                }
            }
            np::publish(np::Playing {
                app: String::from("Matrix Tube"),
                title: title.clone(),
                artist: String::from("im Hintergrund"),
                position_s: video.position_s(),
                duration_s: video.media.duration_s,
                playing: !video.paused(),
                art: art.exists().then(|| art.to_string_lossy().into_owned()),
                pid: 0,
                at_ms: 0,
                can_next: false,
                can_previous: true,
            });
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        np::clear();
        let _ = std::fs::remove_file(runtime("matrix-tube-ton.pid"));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--ton") {
        background::run();
        return Ok(());
    }
    background::end();
    // GPU painting like the terminal (video frames each a texture); the
    // graphics path (duo: Vulkan, Turnip) comes from mk::gpu_path inside
    // the house. MATRIX_CPU=1 is the emergency exit.
    shell::prefer_gpu();
    // 07.11.2026: Tube's own yt-dlp stays current (YouTube breaks old ones within weeks).
    matrixkit_tube::keep_helper_current();
    let mut tube = Tube::new();
    let mut words: Vec<String> = std::env::args().skip(1).collect();
    // Duo form: list here, stage on the other panel (`--buehne [id]`).
    if words.first().map(String::as_str) == Some("--buehne") {
        words.remove(0);
        tube.mode = Mode::Stage;
        tube.field_focus = false;
        tube.chrome.title = String::from("Matrix Tube");
        // `--dateien <a>\t<b>…`: local files (a playlist) instead of a video.
        if words.first().map(String::as_str) == Some("--dateien") && words.len() >= 2 {
            tube.queue = matrixkit_video::queue::Queue::new(words[1].split('\t').map(PathBuf::from).collect());
            words.clear();
        }
    } else if mk::device::panels().is_some() {
        tube.mode = Mode::List;
        tube.duo_list = true;
    }
    // `matrix-tube <link>` plays it; `matrix-tube <search words>` searches.
    if let [one] = words.as_slice() {
        tube.direct = matrixkit_tube::video_id(one);
    }
    if tube.direct.is_none() && !words.is_empty() {
        tube.search = words.join(" ");
        tube.cursor = tube.search.chars().count();
    }
    if let Some(hz) = mk::canvas::refresh_hz() {
        tube.max_fps = hz.min(30.0);
    }
    eprintln!("[tube] Matrix Tube, pid={}, Bildtakt {:.1}/s", std::process::id(), tube.max_fps);
    fam::run(
        tube,
        shell::Options {
            mount: shell::Mount::Window {
                app_id: String::from(APP_ID),
                title: String::from("Matrix Tube"),
                width: WIDTH,
                height: HEIGHT,
            },
            argb: false,
            tag: "tube",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}

#[cfg(test)]
mod role_tests {
    use super::*;

    /// Round 172: the media bar's four roles hit no other role of Tube.
    #[test]
    fn the_bar_shares_no_role() {
        let bar = BAR_BASE..BAR_BASE + 4;
        let single = [
            FIELD, remote::BACK, remote::NOW_ROW, remote::NEXT_NOW, remote::CANCEL, remote::CHIP_QUALITY, remote::CHIP_SUBS,
            remote::CHIP_AUTOPLAY, remote::CHIP_SLEEP, remote::STILL_HERE, library::TAB_SEARCH, library::TAB_LIBRARY,
        ];
        for r in single {
            assert!(!bar.contains(&r), "role {r} lies in the bar");
        }
        for (base, len) in [(remote::SUBS_BASE, 40), (remote::QUALITY_BASE, 10), (remote::REL_BASE, 40), (CHROME_BASE, 50), (ROOT_BASE, 50), (ROW_BASE, 100)] {
            assert!(bar.end <= base || base + len <= bar.start, "range {base}+{len} overlaps the bar");
        }
    }
}
