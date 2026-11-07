//! The lock's face and keypad in the house (round 123). See main.rs.

use matrixkit_image::{Fit, ImageAtlas};
use matrixkit_scene::{glass_clock, notices, passcode, clock_edit};
use matrixkit_scene::scene as fam;
use matrixkit_shell::{self as shell, Key, Loom, Pict, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::motion::Spring;
use matrixkit_theme::scene::{Keep, Rect};
use std::path::PathBuf;
use std::time::Instant;

/// Roles: the notification centre (round 129), then the player.
const NOTE_BASE: usize = 1_000_000;
const PLAYER_BASE: usize = 900;
/// The home zone at the bottom: a drag there always lifts.
const HOME_ZONE: f32 = 66.0;

/// Roles: the twelve keypad cells.
const KEY_BASE: usize = 100;
/// Round 141: the edit mode's «Abbr.» and «Fertig».
const EDIT_CANCEL: usize = 200;
const EDIT_DONE: usize = 201;
/// The probe opens by itself after this long.
const PROBE_S: f32 = 45.0;
/// The keypad goes back to the face after this long untouched.
const IDLE_S: f32 = 12.0;

const WHITE: mk::Rgba = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

pub enum Work {
    /// The wallpaper: path and picture (the live glass bends it itself).
    Wall(String, (u32, u32, Vec<u8>)),
    Answer(bool),
}

/// A touch on the face before it knows what it is (round 129).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Touch {
    /// In the list: a tap, a scroll or a push sideways — the first move says.
    List { x0: f32, y0: f32, role: Option<usize>, part: Part },
    /// Rolling the list.
    Scroll { y0: f32, roll0: f32 },
    /// Pushing a card sideways.
    Push { id: u32, x0: f32, dx0: f32 },
    /// A tap on the player (acted on at release).
    Player { role: Option<usize>, x: f32 },
}

/// Which part of the lock a surface shows (round 124: the duo's two
/// panels are one picture — the face above, the way in below).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Part {
    /// One screen: everything.
    Whole,
    /// The upper panel: date and clock; title and dots when lifted.
    Upper,
    /// The lower panel: the hint and home bar; the keypad when lifted.
    Lower,
}

/// Where the keypad's parts sit when it is fully up.
struct Pad {
    title: (f32, f32),
    dots: (f32, f32),
    keys: [Rect; 12],
}

struct Lock {
    loom: Option<Loom<Work>>,
    probe: bool,
    user: String,
    started: Instant,
    time: String,
    date: String,
    wall: Option<String>,
    /// Decoded, not yet in the atlas.
    fresh: Option<(u32, u32, Vec<u8>)>,
    serial: u64,
    shown: Option<(u32, u32)>,
    /// The glass clock's picture: what it was made for, and its place.
    /// Round 146: the clock as live glass (kit family glass_clock::Live).
    live: glass_clock::Live,
    /// 0 = the face, 1 = the keypad.
    lift: Spring,
    /// Round 140: the face hangs on the finger (kit family mk::touch::Follow).
    drag: Option<mk::touch::Follow>,
    /// The dots' side shake after a wrong code.
    shake: Spring,
    /// Success: everything leaves upwards, then the session opens.
    leave: Option<Spring>,
    entry: String,
    checking: bool,
    wrong_at: Option<Instant>,
    known_len: Option<usize>,
    held: Option<usize>,
    touched: Instant,
    stretch: f32,
    glass: f32,
    /// Round 141: the clock's stroke weight (0 … 1).
    weight: f32,
    /// Round 142: the clock's width against its usual width.
    width: f32,
    /// Round 147: the clock glass' body (0 clear … 1 opaque).
    body: f32,
    /// The code was accepted for the edit mode: the lock opens without it.
    trusted: bool,
    /// A long press on the clock asked for the code first.
    pending_edit: bool,
    /// Round 146: the edit mode's touch logic (kit family clock_edit::Session).
    edit: Option<clock_edit::Session>,
    /// Where the clock's glyphs lie (part, box, digit width).
    clock_at: Option<(Part, Rect, f32)>,
    sheet_at: Option<(Part, clock_edit::Sheet)>,
    size: (f32, f32),
    /// Round 124: the other panel's name when the duo is open (the lower
    /// part lives there as a lock pane).
    split: Option<String>,
    pane_size: (f32, f32),
    /// Round 136: the duo is folded to one panel now.
    folded: bool,
    /// Round 137: folded with the lower panel (DSI-2) in front.
    lower_front: bool,
    /// Round 139: the height of the surface the current swipe began on.
    drag_h: f32,
    pane_hits: (Vec<matrixkit_theme::scene::Placed>, Vec<Option<usize>>),
    /// Round 129: the notification centre and the player.
    groups: Vec<notices::Group>,
    notice: notices::State,
    inbox_stamp: Option<std::time::SystemTime>,
    /// Where the list lies on which part, its content height.
    list_area: Option<(Part, Rect)>,
    list_total: f32,
    playing: Option<mk::now_playing::Playing>,
    /// The player's card and time bar (part, card, bar).
    player_at: Option<(Part, Rect, Rect)>,
    touch: Option<Touch>,
    /// A notification tapped: opened once the code is right.
    pending_open: Option<u32>,
    icons: std::cell::RefCell<std::collections::HashMap<String, Option<PathBuf>>>,
    palette: mk::Palette,
}

fn time_and_date() -> (String, String) {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as libc::time_t).unwrap_or(0);
    // SAFETY: localtime_r writes only into the tm we own.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&now, &mut tm) }.is_null() {
        return (String::from("0:00"), String::new());
    }
    const DAYS: [&str; 7] = ["So.", "Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa."];
    const MONTHS: [&str; 12] = ["Jan.", "Feb.", "März", "Apr.", "Mai", "Juni", "Juli", "Aug.", "Sept.", "Okt.", "Nov.", "Dez."];
    (
        format!("{}:{:02}", tm.tm_hour, tm.tm_min),
        if mk::language::is_english() {
            // British (06.11.2026): «Thu 6 Nov».
            const DAYS_EN: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
            const MONTHS_EN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            format!("{} {} {}", DAYS_EN[tm.tm_wday.clamp(0, 6) as usize], tm.tm_mday, MONTHS_EN[tm.tm_mon.clamp(0, 11) as usize])
        } else {
            format!("{} {}. {}", DAYS[tm.tm_wday.clamp(0, 6) as usize], tm.tm_mday, MONTHS[tm.tm_mon.clamp(0, 11) as usize])
        },
    )
}

impl Lock {
    fn new(probe: bool) -> Lock {
        let (time, date) = time_and_date();
        let known_len = mk::settings::read("lock-code-length").and_then(|s| s.trim().parse().ok()).filter(|n: &usize| (1..=32).contains(n));
        // Round 148: the clock as the user made it (kit: clock_edit::Values).
        let clock_edit::Values { stretch, weight, width, body } = clock_edit::Values::load();
        let glass = mk::settings::read("lock-clock-glass").and_then(|s| s.trim().parse().ok()).unwrap_or(0.85f32).clamp(0.0, 1.0);
        let mut lock = Lock {
            loom: None,
            probe,
            user: crate::user(),
            started: Instant::now(),
            time,
            date,
            wall: None,
            fresh: None,
            serial: 0,
            shown: None,
            live: glass_clock::Live::new("sperre-uhr"),
            lift: Spring::snappy(0.0),
            drag: None,
            shake: Spring::with(0.0, 0.45, 0.6),
            leave: None,
            entry: String::new(),
            checking: false,
            wrong_at: None,
            known_len,
            held: None,
            touched: Instant::now(),
            stretch,
            glass,
            weight,
            width,
            body,
            trusted: false,
            pending_edit: false,
            edit: None,
            clock_at: None,
            sheet_at: None,
            size: (0.0, 0.0),
            // Round 136: on a duo BOTH panels always get a surface — what
            // each shows follows the fold every beat (`parts`).
            split: mk::device::panels().map(|_| String::from("DSI-2")),
            folded: matches!(mk::device::fold(), mk::device::Fold::Folded { .. }),
            drag_h: 0.0,
            lower_front: matches!(mk::device::fold(), mk::device::Fold::Folded { ref front } if front == "DSI-2"),
            pane_size: (0.0, 0.0),
            pane_hits: (Vec::new(), Vec::new()),
            groups: Vec::new(),
            notice: notices::State::default(),
            inbox_stamp: None,
            list_area: None,
            list_total: 0.0,
            playing: None,
            player_at: None,
            touch: None,
            pending_open: None,
            icons: std::cell::RefCell::new(std::collections::HashMap::new()),
            palette: mk::Palette::load().unwrap_or_default(),
        };
        lock.read_inbox();
        lock.playing = mk::now_playing::read();
        lock
    }

    /// The inbox again when it changed (round 129).
    fn read_inbox(&mut self) -> bool {
        let stamp = mk::inbox::stamp();
        if stamp == self.inbox_stamp && stamp.is_some() {
            return false;
        }
        self.inbox_stamp = stamp;
        self.groups = notices::group(&mk::inbox::read());
        if let Some(open) = &self.notice.open {
            if !self.groups.iter().any(|g| &g.app == open) {
                self.notice.open = None;
            }
        }
        true
    }

    /// An app's icon, looked up once.
    fn icon_of(&self, app: &str) -> Option<PathBuf> {
        self.icons.borrow_mut().entry(app.to_string()).or_insert_with(|| matrixkit_icons::for_app(app, &self.palette, 96)).clone()
    }

    /// Take notifications away (and tell the daemon).
    fn dismiss(&mut self, ids: &[u32]) {
        for id in ids {
            mk::inbox::send(mk::inbox::Command::Dismiss(*id));
        }
        for g in &mut self.groups {
            g.items.retain(|i| !ids.contains(&i.id));
        }
        self.groups.retain(|g| !g.items.is_empty());
        self.notice.swipe = None;
        if self.notice.open.as_ref().is_some_and(|o| !self.groups.iter().any(|g| &g.app == o)) {
            self.notice.open = None;
        }
        mk::feel::tap();
    }

    /// What the main surface (DSI-1) and the pane (DSI-2) show: opened one
    /// picture over both panels, folded the whole lock on each (the front
    /// one is seen, the back one is dark anyway).
    fn parts(&self) -> (Part, Part) {
        match (&self.split, self.folded) {
            (Some(_), false) => (Part::Upper, Part::Lower),
            (Some(_), true) => (Part::Whole, Part::Whole),
            (None, _) => (Part::Whole, Part::Whole),
        }
    }

    /// Dirty for one surface, Panes for both.
    fn both(&self, r: Refresh) -> Refresh {
        if self.split.is_some() && matches!(r, Refresh::Dirty) {
            Refresh::Panes
        } else {
            r
        }
    }

    /// The height the wallpaper spans (both panels when split) and how far
    /// down `part` begins in it.
    fn span(&self, part: Part, hf: f32) -> (f32, f32) {
        // Round 167: the kit family wall's span.
        match part {
            Part::Whole => matrixkit_scene::wall::span(matrixkit_scene::wall::Part::Whole, hf, 0.0),
            Part::Upper => matrixkit_scene::wall::span(matrixkit_scene::wall::Part::Upper, hf, self.pane_size.1),
            Part::Lower => matrixkit_scene::wall::span(matrixkit_scene::wall::Part::Lower, hf, self.size.1),
        }
    }

    /// Follow the canvas' picture; a new one is decoded off the surface.
    fn follow_wall(&mut self) {
        let now = mk::wallpaper::current();
        if now == self.wall {
            return;
        }
        self.wall = now.clone();
        let (Some(path), Some(loom)) = (now, &self.loom) else { return };
        loom.spin(move || {
            let p = std::path::Path::new(&path);
            let sharp = matrixkit_image::fitted_rgba(p, mk::device::image_long_side()).unwrap_or((0, 0, Vec::new()));
            Work::Wall(path, sharp)
        });
    }

    fn to_pad(&mut self) {
        self.lift.retarget(1.0);
        self.touched = Instant::now();
    }

    fn to_face(&mut self) {
        self.lift.retarget(0.0);
        self.entry.clear();
        self.pending_open = None;
        self.pending_edit = false;
    }

    /// Round 141: into the clock's edit mode.
    fn enter_edit(&mut self) {
        self.drag = None;
        self.touch = None;
        self.lift.retarget(0.0);
        self.edit = Some(clock_edit::Session::new(self.values()));
        eprintln!("[sperre] Uhr bearbeiten");
    }

    /// Leave upwards and open the session (the code was right).
    fn open_now(&mut self) {
        if let Some(id) = self.pending_open.take() {
            mk::inbox::send(mk::inbox::Command::Open(id));
        }
        let mut leave = Spring::snappy(0.0);
        leave.retarget(1.0);
        self.leave = Some(leave);
    }

    /// A long press: on the clock it opens the edit mode — after the code
    /// (nobody re-styles a locked phone).
    fn hold_at(&mut self, x: f32, y: f32, part: Part) -> Refresh {
        if self.edit.is_some() || self.leave.is_some() || self.lift.value > 0.2 {
            return Refresh::Calm;
        }
        let Some((cp, r, _)) = self.clock_at else { return Refresh::Calm };
        let near = Rect { x: r.x - 24.0, y: r.y - 24.0, w: r.w + 48.0, h: r.h + 48.0 };
        if cp != part || !clock_edit::inside(near, x, y) {
            return Refresh::Calm;
        }
        mk::feel::tap();
        self.drag = None;
        self.touch = None;
        if self.trusted {
            self.enter_edit();
        } else {
            self.pending_edit = true;
            self.to_pad();
        }
        Refresh::Panes
    }

    fn values(&self) -> clock_edit::Values {
        clock_edit::Values { stretch: self.stretch, weight: self.weight, width: self.width, body: self.body }
    }

    fn set_values(&mut self, v: clock_edit::Values) {
        self.stretch = v.stretch;
        self.weight = v.weight;
        self.width = v.width;
        self.body = v.body;
    }

    fn edit_strike(&mut self, role: Option<usize>, x: f32, y: f32, part: Part) -> Refresh {
        let targets = clock_edit::Targets {
            clock: self.clock_at.filter(|(p, ..)| *p == part).map(|(_, r, dw)| (r, dw)),
            sheet: self.sheet_at.filter(|(p, _)| *p == part).map(|(_, s)| s),
            cancel: EDIT_CANCEL,
            done: EDIT_DONE,
        };
        let mut v = self.values();
        let Some(edit) = self.edit.as_mut() else { return Refresh::Calm };
        let outcome = edit.strike(&mut v, role, x, y, &targets);
        self.set_values(v);
        match outcome {
            clock_edit::Outcome::Cancelled => {
                mk::feel::tap();
                self.edit = None;
                Refresh::Panes
            }
            clock_edit::Outcome::Done => {
                mk::feel::confirm();
                self.values().save();
                self.edit = None;
                Refresh::Panes
            }
            clock_edit::Outcome::Changed => Refresh::Panes,
            clock_edit::Outcome::Calm => Refresh::Calm,
        }
    }

    fn edit_travel(&mut self, x: f32, y: f32) -> Refresh {
        let sheet = self.sheet_at.map(|(_, s)| s);
        let mut v = self.values();
        let Some(edit) = self.edit.as_mut() else { return Refresh::Calm };
        let outcome = edit.travel(&mut v, x, y, sheet);
        self.set_values(v);
        if outcome == clock_edit::Outcome::Changed { Refresh::Panes } else { Refresh::Calm }
    }

    fn type_char(&mut self, c: char) -> Refresh {
        if self.checking || self.leave.is_some() {
            return Refresh::Calm;
        }
        self.touched = Instant::now();
        self.wrong_at = None;
        if self.entry.chars().count() < 64 {
            self.entry.push(c);
        }
        if self.lift.target < 1.0 {
            self.to_pad();
        }
        if self.known_len == Some(self.entry.chars().count()) {
            return self.submit();
        }
        Refresh::Dirty
    }

    fn submit(&mut self) -> Refresh {
        if self.checking || self.entry.is_empty() {
            return Refresh::Calm;
        }
        self.checking = true;
        let (user, code, probe) = (self.user.clone(), self.entry.clone(), self.probe);
        if let Some(loom) = &self.loom {
            loom.spin(move || Work::Answer((probe && code == "demo") || crate::pam::verify(&user, &code)));
        }
        Refresh::Dirty
    }

    fn press_cell(&mut self, cell: usize) -> Refresh {
        mk::feel::tap();
        self.held = Some(cell);
        match passcode::press(Some(cell), 0) {
            Some(passcode::Press::Digit(c)) => self.type_char(c),
            Some(passcode::Press::Left) if self.known_len.is_none() => self.submit(),
            Some(passcode::Press::Right) => {
                self.touched = Instant::now();
                if self.entry.is_empty() {
                    self.to_face();
                } else {
                    self.entry.pop();
                }
                Refresh::Dirty
            }
            _ => Refresh::Dirty,
        }
    }

    fn pad(&self, wf: f32, hf: f32, part: Part) -> Pad {
        let wide = wf > hf * 1.15;
        // The grid's column: the whole lower panel, the right part of a
        // wide screen, else the lower part.
        let (gx0, gw, gy0, gh) = if part == Part::Lower {
            (0.0, wf, hf * 0.04, hf * 0.92)
        } else if wide {
            (wf * 0.44, wf * 0.52, hf * 0.06, hf * 0.9)
        } else {
            (0.0, wf, hf * 0.30, hf * 0.62)
        };
        let keys = passcode::grid(Rect { x: gx0, y: gy0, w: gw, h: gh });
        let top = keys[0].y;
        let (title, dots) = if part == Part::Upper {
            ((wf / 2.0, hf * 0.40), (wf / 2.0, hf * 0.40 + 44.0))
        } else if wide {
            let cx = wf * 0.22;
            ((cx, hf * 0.40), (cx, hf * 0.40 + 40.0))
        } else {
            let cx = wf / 2.0;
            let t = (top - 92.0).max(hf * 0.08);
            ((cx, t), (cx, t + 44.0))
        };
        Pad { title, dots, keys }
    }

    fn lay_wall(&mut self, b: &mut SceneBuilder, atlas: &mut ImageAtlas, wf: f32, hf: f32, part: Part, ground: usize) {
        if let Some(sharp) = self.fresh.take() {
            self.serial += 1;
            atlas.put(&PathBuf::from(format!("\u{0}sperre-wand:{}", self.serial)), sharp.0, sharp.1, sharp.2);
            self.shown = Some((sharp.0, sharp.1));
        }
        let home = Rect { x: 0.0, y: 0.0, w: wf, h: hf };
        let Some((pw, ph)) = self.shown else { return };
        // One picture over both panels: cover the whole span, show our part.
        let (total, shift) = self.span(part, hf);
        // Round 167: the cover rule of the kit family wall.
        let (x, y, bw, bh) = matrixkit_scene::wall::cover(pw, ph, wf, total, shift);
        b.pict(Pict {
            path: PathBuf::from(format!("\u{0}sperre-wand:{}", self.serial)),
            box_w: bw as u32,
            box_h: bh as u32,
            fit: Fit::Stretch,
            x,
            y,
            radius: 0.0,
            home,
            anchor: ground,
            clip: Some(home),
        });
    }

    /// The clock as LIVE glass (round 125; round 146: kit family
    /// glass_clock::Live — sharp, stretched while the corner moves, a draft
    /// while the weight slider moves).
    fn clock_picture(&mut self, atlas: &mut ImageAtlas, look: &glass_clock::Look, x: f32, y: f32, fade: f32) -> Option<glass_clock::LivePicture> {
        if look.glass < 0.3 || self.shown.is_none() {
            return None;
        }
        let pace = self.edit.as_ref().map(|e| e.pace()).unwrap_or(glass_clock::Pace::Still);
        let wall = self.wall_path();
        self.live.picture(atlas, &self.time, look, x, y, shell::screen_scale() as f32, pace, &wall, fade)
    }

    /// The wallpaper picture's name (the live glass' backdrop).
    fn wall_path(&self) -> PathBuf {
        PathBuf::from(format!("\u{0}sperre-wand:{}", self.serial))
    }

    #[allow(clippy::too_many_arguments)]
    fn lay_face(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, wf: f32, hf: f32, dy: f32, fade: f32, ground: usize, part: Part) {
        let home = Rect { x: 0.0, y: 0.0, w: wf, h: hf };
        let wide = wf > hf * 1.15;
        if part == Part::Lower && self.edit.is_some() {
            // Round 141: the weight sheet lives on the lower panel.
            self.lay_sheet(b, engine, atlas, wf, hf, fade, ground, part);
            if !self.edit.as_ref().is_some_and(|e| e.sheet) {
                let hint = "Uhr antippen: Dicke, Deckkraft · Ecke ziehen: Grösse";
                let (hw, hh) = engine.measure_styled(hint, mk::typo::CALLOUT.px(), mk::typo::CALLOUT.weight);
                b.styled(engine, hint, mk::typo::CALLOUT, WHITE.with_alpha(0.75 * fade), (wf - hw) / 2.0, (hf - hh) / 2.0, None, None);
            }
            return;
        }
        if part == Part::Lower {
            // The lower panel: the centre above the way in.
            self.lay_centre(b, engine, atlas, Rect { x: 16.0, y: 14.0 + dy, w: wf - 32.0, h: hf - 14.0 - HOME_ZONE }, fade, part);
            self.lay_way_in(b, engine, wf, hf, dy, fade);
            return;
        }
        let ink = WHITE.with_alpha(0.92 * fade);
        let date_style = mk::typo::HEADLINE;
        let dpx = date_style.px() * 1.25;
        let (dw, dh) = engine.measure_family(matrixkit_text::FAMILY, &self.date, dpx, date_style.weight);
        let editing = self.edit.is_some();
        let mut y = (hf * if wide { 0.08 } else { 0.07 }) + dy;
        if editing {
            // Round 141: «Abbr.» and «Fertig» above the date. Round 143: the
            // date and the clock stay EXACTLY where the lock shows them —
            // the edit mode adds, it never moves (owner, 05.11.2026).
            let top = (y - 34.0 - 8.0).clamp(4.0, 10.0);
            clock_edit::lay_bar(b, engine, wf, top, fade, EDIT_CANCEL, EDIT_DONE, home, ground);
        }
        if self.probe {
            let note = "Probe — öffnet sich nach 45 s von selbst";
            let (nw, nh) = engine.measure_styled(note, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight);
            b.styled(engine, note, mk::typo::CAPTION, WHITE.with_alpha(0.7 * fade), (wf - nw) / 2.0, y - nh - 4.0, None, None);
        }
        b.text(engine, self.date.clone(), dpx, date_style.weight, matrixkit_text::FAMILY, ink, (wf - dw) / 2.0, y, None, None);
        y += dh + 6.0;
        // The clock: as large as the screen allows at its stretch.
        // Round 129: a player or notifications below → a smaller clock.
        // Round 143: the same in the edit mode — what is edited is the clock
        // as the lock shows it (player and notifications only hidden).
        let busy = self.playing.is_some() || (part == Part::Whole && !self.groups.is_empty());
        let room = hf * match (part, busy) {
            (Part::Upper, false) => 0.62,
            (Part::Upper, true) => 0.34,
            (_, false) if wide => 0.46,
            (_, true) if wide => 0.26,
            (_, false) => 0.30,
            (_, true) => 0.20,
        };
        // Round 141: the width comes from the room at the usual stretch — a
        // taller stretch makes the clock TALLER (the corner), not narrower.
        // Round 142: × the width factor (the corner sideways), never wider
        // than the surface.
        let mut digit_w = (room / 1.9).min(wf * if part == Part::Upper { 0.13 } else if wide { 0.10 } else { 0.17 }) * self.width;
        let probe = glass_clock::Look { digit_w, stretch: self.stretch, glass: self.glass, weight: self.weight, body: self.body, ink: WHITE };
        let (pw, _) = glass_clock::measure(&self.time, &probe);
        let room_w = wf - 2.0 * (clock_edit::PAD + 14.0);
        // The height is the user's too: a width cut at the surface's edge
        // keeps it (round 144).
        let tall = digit_w * self.stretch;
        if pw > room_w {
            digit_w *= room_w / pw;
        }
        let digit_w = digit_w.floor();
        let stretch = (tall / digit_w.max(1.0)).clamp(glass_clock::Look::STRETCH_MIN, glass_clock::Look::STRETCH_MAX);
        let look = glass_clock::Look { digit_w, stretch, glass: self.glass, weight: self.weight, body: self.body, ink: WHITE };
        let (cw, _) = glass_clock::measure(&self.time, &look);
        let (cx, cy) = ((wf - cw) / 2.0, y);
        // The desktop's glass when the picture is there; drawn glass before.
        match self.clock_picture(atlas, &look, cx, cy, fade) {
            Some(pic) => b.pict(pic.pict(home, ground)),
            None => glass_clock::lay(b, &self.time, cx, cy, &look, home, ground),
        }
        let (_, ch) = glass_clock::measure(&self.time, &look);
        self.clock_at = Some((part, Rect { x: cx, y: cy - dy, w: cw, h: ch }, digit_w));
        if editing {
            clock_edit::lay_frame(b, clock_edit::frame_of(Rect { x: cx, y: cy, w: cw, h: ch }), fade, home, ground);
            if part == Part::Whole {
                self.lay_sheet(b, engine, atlas, wf, hf, fade, ground, part);
            }
            return;
        }
        let mut y = cy + ch + 16.0;
        // The player (round 129).
        self.player_at = self.player_at.filter(|(p, ..)| *p != part);
        if let Some(playing) = self.playing.clone() {
            let w = (wf - 32.0).min(520.0).floor();
            let r = Rect { x: (wf - w) / 2.0, y, w, h: notices::PLAYER_H };
            let map = self.shown.is_some().then(|| glass_clock::lens::card(atlas, w, notices::PLAYER_H, notices::RADIUS + 4.0, shell::screen_scale() as f32, &self.wall_path(), 0.0, fade));
            let bar = notices::lay_player(b, engine, r, &playing, map.as_deref(), fade, PLAYER_BASE, home);
            self.player_at = Some((part, Rect { y: r.y - dy, ..r }, Rect { y: bar.y - dy, ..bar }));
            y += notices::PLAYER_H + 12.0;
        }
        if part == Part::Whole {
            self.lay_centre(b, engine, atlas, Rect { x: 16.0, y, w: wf - 32.0, h: (hf - HOME_ZONE - y).max(0.0) }, fade, part);
            self.lay_way_in(b, engine, wf, hf, dy, fade);
        }
    }

    /// Round 141: the weight sheet at the bottom of `part` (when open).
    #[allow(clippy::too_many_arguments)]
    fn lay_sheet(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, wf: f32, hf: f32, fade: f32, ground: usize, part: Part) {
        if !self.edit.as_ref().is_some_and(|e| e.sheet) {
            self.sheet_at = self.sheet_at.filter(|(p, _)| *p != part);
            return;
        }
        let home = Rect { x: 0.0, y: 0.0, w: wf, h: hf };
        let sh = clock_edit::sheet(wf, hf - 8.0);
        let map = self.shown.is_some().then(|| glass_clock::lens::card(atlas, sh.area.w, sh.area.h, 26.0, shell::screen_scale() as f32, &self.wall_path(), 0.0, fade));
        let pos = self.edit.as_ref().map(|e| e.stack.pos).unwrap_or(0.0);
        let v = self.values();
        clock_edit::lay_sheet(b, engine, &sh, &v, pos, map.as_deref(), fade, home, ground);
        self.sheet_at = Some((part, sh));
    }

    /// The notification centre in `area` (round 129): stacks per app, live
    /// glass cards; the area is remembered for the touches.
    fn lay_centre(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, area: Rect, fade: f32, part: Part) {
        if self.groups.is_empty() || area.h < notices::CARD_H {
            self.list_area = self.list_area.filter(|(p, _)| *p != part);
            return;
        }
        let w = area.w.min(560.0).floor();
        let area = Rect { x: area.x + (area.w - w) / 2.0, w, ..area };
        let map = self.shown.is_some().then(|| glass_clock::lens::card(atlas, w, notices::CARD_H, notices::RADIUS, shell::screen_scale() as f32, &self.wall_path(), 0.0, fade));
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let icon = |app: &str| self.icon_of(app);
        let look = notices::Look { glass: map.as_deref(), icon: &icon, now, fade, role_base: NOTE_BASE };
        let total = notices::lay(b, engine, area, &self.groups, &self.notice, &look);
        self.list_total = total;
        let max = (total - area.h).max(0.0);
        self.notice.roll = self.notice.roll.clamp(0.0, max);
        self.list_area = Some((part, area));
    }

    /// The way in: a word and the home bar at the bottom.
    fn lay_way_in(&self, b: &mut SceneBuilder, engine: &mut TextEngine, wf: f32, hf: f32, dy: f32, fade: f32) {
        let hint = "Nach oben wischen zum Entsperren";
        let (hw, hh) = engine.measure_styled(hint, mk::typo::CALLOUT.px(), mk::typo::CALLOUT.weight);
        let by = hf - 14.0 + dy;
        b.styled(engine, hint, mk::typo::CALLOUT, WHITE.with_alpha(0.78 * fade), (wf - hw) / 2.0, by - 16.0 - hh, None, None);
        let bar = (wf * 0.32).min(140.0);
        // Round 150: above the wallpaper picture (a quad of its layer lay
        // under it — the home bar was never seen).
        b.rise();
        b.quad((wf - bar) / 2.0, by, bar, 5.0, WHITE.with_alpha(0.85 * fade), 2.5, None, None);
    }

    #[allow(clippy::too_many_arguments)]
    fn lay_pad(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, wf: f32, hf: f32, dy: f32, fade: f32, ground: usize, part: Part) {
        let pad = self.pad(wf, hf, part);
        let home = Rect { x: 0.0, y: 0.0, w: wf, h: hf };
        // Live glass: one lens map for every key (the passcode family).
        let key_map = (part != Part::Upper && self.shown.is_some()).then(|| {
            passcode::glass_keys(atlas, pad.keys[0].w, shell::screen_scale() as f32, &self.wall_path(), 0.42 * self.lift.value.clamp(0.0, 1.0), fade)
        });
        if part != Part::Lower {
            self.lay_code_line(b, engine, &pad, dy, fade);
        }
        if part == Part::Upper {
            return;
        }
        let left = (self.known_len.is_none() && !self.entry.is_empty() && !self.checking).then_some("OK");
        let right = Some(if self.entry.is_empty() { "Abbrechen" } else { "Löschen" });
        let look = passcode::Look { map: key_map.as_deref(), held: self.held, left, right, fade, role_base: KEY_BASE, dy, home };
        passcode::lay_keys(b, engine, &pad.keys, &look);
        let _ = ground;
    }

    /// The title and the dots.
    fn lay_code_line(&self, b: &mut SceneBuilder, engine: &mut TextEngine, pad: &Pad, dy: f32, fade: f32) {
        let wrong = self.wrong_at.is_some_and(|t| t.elapsed().as_secs_f32() < 1.6);
        let title = if self.checking {
            "Wird geprüft …"
        } else if wrong {
            "Falscher Code"
        } else if self.pending_edit {
            "Code zum Bearbeiten"
        } else {
            "Code eingeben"
        };
        let tst = mk::typo::TITLE;
        let (tw, _) = engine.measure_styled(title, tst.px(), tst.weight);
        b.styled(engine, title, tst, WHITE.with_alpha(fade), pad.title.0 - tw / 2.0, pad.title.1 + dy, None, None);
        // The dots: the learnt length, or as many as typed (four at least).
        let typed = self.entry.chars().count();
        let n = self.known_len.unwrap_or(typed.max(4)).max(typed);
        passcode::lay_dots(b, pad.dots.0, pad.dots.1 + dy, n, typed, self.shake.value, fade);
    }

}

impl Lock {
    /// One surface's picture: `part` of the lock at w×h.
    fn compose(&mut self, engine: &mut TextEngine, atlas: &mut ImageAtlas, wf: f32, hf: f32, part: Part) -> Scene {
        let mut b = SceneBuilder::new();
        let ground = b.quad(0.0, 0.0, wf, hf, mk::Rgba { r: 0.05, g: 0.04, b: 0.04, a: 1.0 }, 0.0, None, None);
        self.lay_wall(&mut b, atlas, wf, hf, part, ground);
        let lift = self.lift.value.clamp(-0.15, 1.1);
        let gone = self.leave.as_ref().map(|s| s.value).unwrap_or(0.0);
        // The veil deepens as the keypad comes up.
        let veil = 0.42 * lift.clamp(0.0, 1.0) + 0.25 * gone;
        if veil > 0.005 {
            b.quad(0.0, 0.0, wf, hf, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: veil }, 0.0, None, None);
        }
        // One piece: the face rides up with the finger, the keypad follows from below.
        let face_dy = -lift * hf * 0.55;
        // Round 138: the glass stays in sight on its way up (it fades only
        // in the last part) — the swipe is seen, not just its end.
        let face_fade = (1.0 - (lift - 0.35).max(0.0) * 1.6).clamp(0.0, 1.0);
        if face_fade > 0.0 {
            self.lay_face(&mut b, engine, atlas, wf, hf, face_dy, face_fade, ground, part);
        }
        // Round 138: the keypad rides with the finger too (nearly 1:1).
        let pad_dy = (1.0 - lift) * hf * 0.6 - gone * hf;
        let pad_fade = (lift * 1.3 - 0.2).clamp(0.0, 1.0) * (1.0 - gone);
        if lift > 0.01 {
            self.lay_pad(&mut b, engine, atlas, wf, hf, pad_dy, pad_fade, ground, part);
        }
        b.finish()
    }

    fn strike_at(&mut self, role: Option<usize>, x: f64, y: f64, _source: Source, part: Part) -> Refresh {
        let r = self.strike_inner(role, x, y, _source, part);
        mk::touch::log("sperre", &format!("DRUCK {part:?} x={x:.0} y={y:.0} lift={:.2} touch={} drag={} size={:?} pane={:?}", self.lift.value, touch_name(&self.touch), self.drag.is_some(), self.size, self.pane_size));
        r
    }

    fn strike_inner(&mut self, role: Option<usize>, x: f64, y: f64, _source: Source, part: Part) -> Refresh {
        // Round 139: every touch knows its surface's height — the list hands
        // a swipe on to the lift without passing the drag start below.
        self.drag_h = self.touch_h(part);
        if self.edit.is_some() {
            return self.edit_strike(role, x as f32, y as f32, part);
        }
        if self.leave.is_some() {
            return Refresh::Calm;
        }
        self.touched = Instant::now();
        if let Some(cell) = role.and_then(|r| r.checked_sub(KEY_BASE)).filter(|c| *c < 12) {
            if self.lift.value > 0.8 {
                return self.press_cell(cell);
            }
        }
        let (x, y) = (x as f32, y as f32);
        // Round 129: the face's player and list take their own touches.
        if self.lift.value < 0.2 {
            let inside = |r: &Rect| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h;
            if let Some((pp, card, _)) = self.player_at {
                if pp == part && inside(&card) {
                    self.touch = Some(Touch::Player { role, x });
                    return Refresh::Calm;
                }
            }
            let bottom = self.touch_h(part);
            if let Some((lp, area)) = self.list_area {
                if lp == part && inside(&area) && y < bottom - HOME_ZONE {
                    self.touch = Some(Touch::List { x0: x, y0: y, role, part });
                    return Refresh::Calm;
                }
            }
        }
        self.drag_h = self.touch_h(part);
        self.drag = self.follow(y);
        Refresh::Calm
    }

    /// The face follows a finger landing at `y`: its way is the face's own
    /// travel (face_dy = −lift · 0.55 h) on the surface touched.
    fn follow(&self, y: f32) -> Option<mk::touch::Follow> {
        mk::touch::Follow::new(y, self.lift.value, self.drag_h * 0.55)
    }

    /// The height of the surface a touch of `part` lies on: the pane for
    /// the lower part, and whenever the main surface is hidden (never drawn,
    /// size 0).
    fn touch_h(&self, part: Part) -> f32 {
        if part == Part::Lower || self.size.1 < 1.0 || (self.split.is_some() && self.lower_front) {
            self.pane_size.1
        } else {
            self.size.1
        }
    }

    fn travel_to(&mut self, x: f64, y: f64, _source: Source) -> Refresh {
        let r = self.travel_inner(x, y, _source);
        mk::touch::log("sperre", &format!("ZUG y={y:.0} lift={:.2} touch={} drag={}", self.lift.value, touch_name(&self.touch), self.drag.is_some()));
        r
    }

    fn travel_inner(&mut self, x: f64, y: f64, _source: Source) -> Refresh {
        if self.edit.is_some() {
            return self.edit_travel(x as f32, y as f32);
        }
        let (xf, yf) = (x as f32, y as f32);
        match self.touch {
            Some(Touch::List { x0, y0, role, .. }) => {
                let (dx, dy) = (xf - x0, yf - y0);
                if dx.abs().max(dy.abs()) < 8.0 {
                    return Refresh::Calm;
                }
                if dx.abs() > dy.abs() {
                    match notices::card_at(role, NOTE_BASE, &self.groups, &self.notice) {
                        Some(id) => {
                            let dx0 = self.notice.swipe.filter(|(i, _)| *i == id).map(|(_, d)| d).unwrap_or(0.0);
                            self.notice.swipe = Some((id, dx0));
                            self.touch = Some(Touch::Push { id, x0, dx0 });
                        }
                        None => self.touch = None,
                    }
                } else {
                    self.notice.swipe = None;
                    // Round 138: a list that cannot roll that way hands the
                    // swipe to the lock — the glass follows the finger from
                    // anywhere (rolling stays where there is more to see).
                    let area_h = self.list_area.map(|(_, a)| a.h).unwrap_or(0.0);
                    let max = (self.list_total - area_h).max(0.0);
                    let can_roll = if dy < 0.0 { self.notice.roll < max - 0.5 } else { self.notice.roll > 0.5 };
                    if can_roll {
                        self.touch = Some(Touch::Scroll { y0, roll0: self.notice.roll });
                    } else {
                        self.touch = None;
                        self.drag = self.follow(y0);
                    }
                }
                return self.travel_inner(x, y, _source);
            }
            Some(Touch::Scroll { y0, roll0 }) => {
                let area_h = self.list_area.map(|(_, a)| a.h).unwrap_or(0.0);
                let max = (self.list_total - area_h).max(0.0);
                let want = roll0 - (yf - y0);
                // Rolled to the end and the finger goes on up: the rest of
                // the way lifts the lock (one gesture, no stop).
                if want > max + 12.0 {
                    self.notice.roll = max;
                    self.touch = None;
                    self.drag = self.follow(yf);
                    return Refresh::Dirty;
                }
                self.notice.roll = want.clamp(0.0, max);
                return Refresh::Dirty;
            }
            Some(Touch::Push { id, x0, dx0 }) => {
                self.notice.swipe = Some((id, (dx0 + xf - x0).clamp(-220.0, 0.0)));
                return Refresh::Dirty;
            }
            Some(Touch::Player { .. }) => return Refresh::Calm,
            None => {}
        }
        // Round 138: 1:1 — the face moves exactly as far as the finger
        // (face_dy = −lift · 0.55 h).
        // Round 139: the height of the surface the finger is ON — folded, the
        // main surface is never drawn (main_hidden) and its size stayed 0:
        // the way shrank to 1 px and the slightest move flung the glass up.
        // Round 140: the rules live in the kit (mk::touch::Follow).
        let Some(d) = &mut self.drag else { return Refresh::Calm };
        let v = d.travel(y as f32);
        self.lift.value = v;
        self.lift.target = v;
        self.lift.velocity = 0.0;
        Refresh::Dirty
    }

    fn release_at(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        let speed = self.drag.as_ref().map(|d| d.speed()).unwrap_or(0.0);
        let r = self.release_inner(_role, _x, _y, _source);
        mk::touch::log("sperre", &format!("LOS y={_y:.0} lift={:.2} ziel={:.2} tempo={speed:.2} touch={}", self.lift.value, self.lift.target, touch_name(&self.touch)));
        r
    }

    fn release_inner(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        if let Some(edit) = self.edit.as_mut() {
            // The grab ends: the clock's glass is made sharp at its new size.
            return if edit.release() == clock_edit::Outcome::Changed { Refresh::Panes } else { Refresh::Calm };
        }
        if self.held.take().is_some() && self.drag.is_none() {
            return Refresh::Dirty;
        }
        // Round 129: the list's and the player's touches end here.
        if let Some(t) = self.touch.take() {
            return self.touch_end(t);
        }
        let Some(d) = self.drag.take() else { return Refresh::Calm };
        // Only a real swipe opens (owner: «I don't even have to swipe
        // properly»); the rule is the kit's.
        match d.release() {
            mk::touch::Settle::Tap => {
                // A tap on the face: it hops a little, like a hint.
                if self.lift.target < 0.5 {
                    self.lift.velocity = 1.6;
                    self.lift.target = 0.0;
                }
            }
            mk::touch::Settle::Open { velocity } => {
                self.lift.velocity = velocity;
                if self.trusted {
                    // The code was given for the edit mode: no second time.
                    self.open_now();
                } else {
                    self.to_pad();
                }
            }
            mk::touch::Settle::Back { velocity } => {
                self.lift.velocity = velocity;
                self.to_face();
            }
        }
        Refresh::Dirty
    }

    /// A list or player touch let go (round 129).
    fn touch_end(&mut self, t: Touch) -> Refresh {
        use notices::Hit;
        match t {
            Touch::List { role, .. } => {
                let hit = notices::hit(role, NOTE_BASE, &self.groups, &self.notice);
                if !matches!(hit, Some(Hit::Remove(_))) && self.notice.swipe.take().is_some() {
                    return Refresh::Dirty;
                }
                match hit {
                    Some(Hit::Stack(app)) => {
                        mk::feel::tap();
                        self.notice.open = Some(app);
                    }
                    Some(Hit::Less) => {
                        mk::feel::tap();
                        self.notice.open = None;
                    }
                    Some(Hit::Clear(app)) => {
                        let ids: Vec<u32> = self.groups.iter().filter(|g| g.app == app).flat_map(|g| g.items.iter().map(|i| i.id)).collect();
                        self.dismiss(&ids);
                    }
                    Some(Hit::Remove(id)) => self.dismiss(&[id]),
                    Some(Hit::Note(id)) => {
                        // Opened once the code is right.
                        mk::feel::tap();
                        self.pending_open = Some(id);
                        self.to_pad();
                    }
                    None => {}
                }
                Refresh::Dirty
            }
            Touch::Push { id, .. } => {
                let dx = self.notice.swipe.map(|(_, d)| d).unwrap_or(0.0);
                self.notice.swipe = (dx < notices::OPEN_DX / 2.0).then_some((id, notices::OPEN_DX));
                Refresh::Dirty
            }
            Touch::Scroll { .. } => Refresh::Dirty,
            Touch::Player { role, x } => {
                use mk::now_playing::Command as C;
                let (Some((_, _, bar)), Some(p)) = (self.player_at, self.playing.as_ref()) else { return Refresh::Calm };
                let cmd = match notices::player_hit(role, PLAYER_BASE, x, bar) {
                    Some(notices::PlayerHit::Toggle) => Some(C::Toggle),
                    Some(notices::PlayerHit::Next) => Some(C::Next),
                    Some(notices::PlayerHit::Previous) => Some(C::Previous),
                    Some(notices::PlayerHit::Seek(f)) if p.duration_s > 0.0 => Some(C::Seek(f as f64 * p.duration_s)),
                    _ => None,
                };
                if let Some(cmd) = cmd {
                    mk::feel::tap();
                    mk::now_playing::send(cmd);
                    // Show it at once (the player confirms within a beat).
                    if let Some(p) = &mut self.playing {
                        match cmd {
                            C::Toggle => p.playing = !p.playing,
                            C::Seek(t) => {
                                p.position_s = t;
                                p.at_ms = mk::now_playing::now_ms();
                            }
                            _ => {}
                        }
                    }
                }
                Refresh::Dirty
            }
        }
    }

    fn key_in(&mut self, key: Key, _shift: bool) -> Refresh {
        match key {
            Key::Char(s) => {
                let mut r = Refresh::Calm;
                for c in s.chars().filter(|c| !c.is_control()) {
                    r = self.type_char(c);
                }
                r
            }
            Key::Backspace => {
                self.entry.pop();
                Refresh::Dirty
            }
            Key::Enter => {
                if self.lift.target < 1.0 {
                    self.to_pad();
                    return Refresh::Dirty;
                }
                self.submit()
            }
            Key::Escape => {
                self.to_face();
                Refresh::Dirty
            }
            _ => Refresh::Calm,
        }
    }

    fn beat_tick(&mut self) -> Refresh {
        let mut r = Refresh::Calm;
        // Round 136: folded / opened while locked — both surfaces anew.
        let fold = mk::device::fold();
        let folded = matches!(fold, mk::device::Fold::Folded { .. });
        self.lower_front = matches!(fold, mk::device::Fold::Folded { ref front } if front == "DSI-2");
        if folded != self.folded {
            eprintln!("[sperre] {}", if folded { "zugeklappt — ganze Sperre je Panel" } else { "aufgeklappt — oben und unten" });
            self.folded = folded;
            self.touch = None;
            self.drag = None;
            self.list_area = None;
            self.player_at = None;
            r = Refresh::Panes;
        }
        // Round 129: new notifications, what plays (its time moves).
        if self.read_inbox() {
            r = Refresh::Dirty;
        }
        let playing = mk::now_playing::read();
        if playing != self.playing {
            self.playing = playing;
            r = Refresh::Dirty;
        }
        if self.probe && self.started.elapsed().as_secs_f32() > PROBE_S && self.leave.is_none() {
            eprintln!("[sperre] Probe: öffnet sich von selbst");
            let mut leave = Spring::snappy(0.0);
            leave.retarget(1.0);
            self.leave = Some(leave);
            r = Refresh::Dirty;
        }
        let (time, date) = time_and_date();
        if time != self.time || date != self.date {
            self.time = time;
            self.date = date;
            r = Refresh::Dirty;
        }
        self.follow_wall();
        if self.lift.target >= 1.0 && !self.checking && self.drag.is_none() && self.touched.elapsed().as_secs_f32() > IDLE_S {
            self.to_face();
            r = Refresh::Dirty;
        }
        if self.wrong_at.is_some_and(|t| t.elapsed().as_secs_f32() > 1.6) {
            self.wrong_at = None;
            r = Refresh::Dirty;
        }
        r
    }

    fn glide_step(&mut self, dt: f32) -> Refresh {
        if self.drag.is_none() {
            self.lift.tick(dt);
        }
        // Round 147: the sheet's pages glide to the one chosen.
        if let Some(edit) = self.edit.as_mut() {
            edit.tick(dt);
        }
        self.shake.tick(dt);
        if let Some(leave) = &mut self.leave {
            leave.tick(dt);
            if leave.value > 0.96 {
                // Round 134: niri keeps this last picture and blends it into
                // the session — no hard cut from lock to desktop.
                mk::canvas::screen_transition(0);
                return Refresh::Close;
            }
        }
        Refresh::Dirty
    }

    fn woven_in(&mut self, work: Work) -> Refresh {
        match work {
            Work::Wall(path, sharp) => {
                if sharp.0 == 0 || self.wall.as_deref() != Some(path.as_str()) {
                    return Refresh::Calm;
                }
                self.fresh = Some(sharp);
                Refresh::Dirty
            }
            Work::Answer(true) => {
                eprintln!("[sperre] Code angenommen — öffne");
                let n = self.entry.chars().count();
                if self.known_len != Some(n) {
                    mk::settings::write("lock-code-length", &n.to_string());
                }
                self.entry.clear();
                mk::feel::confirm();
                // Round 141: the code was asked for the edit mode.
                if self.pending_edit {
                    self.pending_edit = false;
                    self.trusted = true;
                    self.enter_edit();
                    return Refresh::Panes;
                }
                // The notification tapped on the lock opens now.
                self.open_now();
                Refresh::Dirty
            }
            Work::Answer(false) => {
                eprintln!("[sperre] Code abgelehnt");
                self.checking = false;
                self.entry.clear();
                self.wrong_at = Some(Instant::now());
                // The impulse: the dots swing out and settle.
                self.shake.value = 0.0;
                self.shake.velocity = 9.0;
                mk::feel::warn();
                Refresh::Dirty
            }
        }
    }
}

impl shell::Client for Lock {
    type Card = ();
    type Work = Work;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &Loom<Work>) {
        self.loom = Some(loom.clone());
        self.follow_wall();
    }


    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let (wf, hf) = (w as f32, h as f32);
        self.size = (wf, hf);
        let part = self.parts().0;
        self.compose(engine, atlas, wf, hf, part)
    }

    fn strike(&mut self, role: Option<usize>, x: f64, y: f64, source: Source) -> Refresh {
        let part = self.parts().0;
        let r = self.strike_at(role, x, y, source, part);
        self.both(r)
    }

    fn hold(&mut self, _role: Option<usize>, x: f64, y: f64, _source: Source) -> Refresh {
        let part = self.parts().0;
        let r = self.hold_at(x as f32, y as f32, part);
        self.both(r)
    }

    fn travel(&mut self, x: f64, y: f64, source: Source) -> Refresh {
        let r = self.travel_to(x, y, source);
        self.both(r)
    }

    fn release(&mut self, role: Option<usize>, x: f64, y: f64, source: Source) -> Refresh {
        let r = self.release_at(role, x, y, source);
        self.both(r)
    }

    fn key(&mut self, key: Key, shift: bool) -> Refresh {
        let r = self.key_in(key, shift);
        self.both(r)
    }

    fn beat(&mut self) -> Refresh {
        let r = self.beat_tick();
        self.both(r)
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        let r = self.glide_step(dt);
        self.both(r)
    }

    fn woven(&mut self, work: Work) -> Refresh {
        let r = self.woven_in(work);
        self.both(r)
    }

    fn fleet(&mut self) -> Vec<shell::fleet::Spec<()>> {
        if self.split.is_none() {
            return Vec::new();
        }
        vec![shell::fleet::Spec { pane: (), width: 0, height: 0, cross: 0, namespace: "matrix-sperre-unten", keyboard: false, sig: (1, 1) }]
    }

    fn pane_place(&mut self, _card: &()) -> Option<shell::PanePlace> {
        Some(shell::PanePlace::veil(self.split.clone(), None, false))
    }

    fn pane_gpu(&self, _card: &()) -> bool {
        true
    }

    fn pane_scene(&mut self, _card: &(), engine: &mut TextEngine, atlas: &mut ImageAtlas, w: u32, h: u32) -> Option<Scene> {
        let (wf, hf) = (w as f32, h as f32);
        self.pane_size = (wf, hf);
        let part = self.parts().1;
        let scene = self.compose(engine, atlas, wf, hf, part);
        self.pane_hits = (scene.placed.clone(), scene.roles.clone());
        Some(scene)
    }

    fn pane_strike(&mut self, _card: &(), x: f64, y: f64, source: Source) -> Refresh {
        let role = matrixkit_theme::scene::hit(&self.pane_hits.0, x as f32, y as f32).and_then(|i| self.pane_hits.1.get(i).copied().flatten());
        let part = self.parts().1;
        let r = self.strike_at(role, x, y, source, part);
        self.both(r)
    }

    fn pane_hold(&mut self, _card: &(), x: f64, y: f64, _source: Source) -> Refresh {
        let part = self.parts().1;
        let r = self.hold_at(x as f32, y as f32, part);
        self.both(r)
    }

    fn pane_travel(&mut self, _card: &(), x: f64, y: f64, source: Source) -> Refresh {
        let r = self.travel_to(x, y, source);
        self.both(r)
    }

    fn pane_release(&mut self, _card: &(), x: f64, y: f64, source: Source) -> Refresh {
        let r = self.release_at(None, x, y, source);
        self.both(r)
    }






    fn animating(&self) -> bool {
        self.drag.is_none() && (!self.lift.is_settled() || !self.shake.is_settled())
            || self.leave.is_some()
            || self.edit.as_ref().is_some_and(|e| !e.stack.held() && (e.stack.pos - e.stack.page as f32).abs() > 0.002)
    }


    /// Round 137: folded with the lower panel in front, the main surface
    /// (upper panel) is dark — not painted, not waited for.
    fn main_hidden(&self) -> bool {
        self.split.is_some() && self.lower_front
    }

    fn output(&self) -> Option<String> {
        // Round 136: on a duo the main surface is always the upper panel,
        // the pane the lower one (see `parts`).
        mk::device::panels().map(|_| String::from("DSI-1"))
    }
}

pub fn run(probe: bool) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[sperre] Matrix Sperre im Haus (Runde 123), Probe: {probe}");
    // Round 130: apps behind the lock know it (they get no frames).
    mk::session_lock::set();
    // The face moves with the finger: the GPU paints (MATRIX_CPU=1 is the way out).
    shell::prefer_gpu();
    // Round 137: never wait for a panel's frame. Folded, the main surface
    // lies on the BACK panel, switched off — a FIFO present waited ~1 s per
    // picture for it and the whole lock (front panel too) ran at 1 frame/s.
    if std::env::var_os("MATRIX_PRESENT").is_none() {
        // SAFETY: at start, before any thread of ours exists.
        unsafe { std::env::set_var("MATRIX_PRESENT", "mailbox") };
    }
    let ran = shell::run(
        Lock::new(probe),
        shell::Options {
            mount: shell::Mount::Lock,
            argb: false,
            tag: "sperre",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    );
    mk::session_lock::clear();
    ran
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keypad_fits_both_duo_shapes() {
        let lock = Lock::new(true);
        for (w, h) in [(652.0, 489.0), (489.0, 652.0), (390.0, 844.0)] {
            let pad = lock.pad(w, h, Part::Whole);
            let lower = lock.pad(w, h, Part::Lower);
            for k in &lower.keys {
                assert!(k.x >= 0.0 && k.x + k.w <= w && k.y >= 0.0 && k.y + k.h <= h, "unten {w}×{h}: {k:?}");
            }
            for k in &pad.keys {
                assert!(k.x >= 0.0 && k.x + k.w <= w && k.y >= 0.0 && k.y + k.h <= h, "{w}×{h}: {k:?}");
                assert!(k.w >= 44.0, "{w}×{h}: a cell of {} is too small for a finger", k.w);
            }
            assert!(pad.dots.1 < pad.keys[0].y || w > h, "{w}×{h}: dots above the grid");
        }
    }

}


fn touch_name(t: &Option<Touch>) -> &'static str {
    match t {
        None => "-",
        Some(Touch::List { .. }) => "Liste",
        Some(Touch::Scroll { .. }) => "Rollen",
        Some(Touch::Push { .. }) => "Schieben",
        Some(Touch::Player { .. }) => "Player",
    }
}
