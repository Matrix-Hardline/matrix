//! R216 — the top bar in the HOUSE (matrixkit-shell, no iced).
//!
//! One process, several surfaces, as on iced (R209):
//! - the SPINE is the main layer surface. On a duo (`mk::device::bar_chips`)
//!   it is an invisible 1 px line (namespace `matrix-bar-spine`) that only
//!   reserves the strip (`--reservieren`: `reserved_zone()`, 10 px margin);
//!   elsewhere it is the whole bar (namespace `matrix-bar`), its widgets
//!   laid in one strip.
//! - every widget is a glass CHIP of its own (namespace `matrix-chip`), a
//!   fleet surface that places itself (`Client::pane_place`): the left zone
//!   from the left edge, the right zone from the right edge, the centre
//!   zone as one chip the compositor centres. A chip that grows or moves is
//!   moved in place — no surface reopens when the clock ticks.
//! - the APP CARD (the name chip on a duo): veil + glass card laid exactly
//!   over the focused window (`mk::canvas::window_frame`), two columns when
//!   wide, scrolling by drag; settings, «Panels tauschen», Krabbler, close.
//! - the SESSION and MATRIX menus as glass cards under their chip, a clear
//!   backdrop that closes them; final actions ask on a glass dialog.
//!
//! Everything a chip shows is measured with the real text engine (the
//! iced bar estimated widths); the readers (`focus_read`, `battery_read`,
//! …) are the iced path's own. Bell and control centre talk to their
//! daemons exactly as before.

use crate::{
    battery_read, focus_read, media, net_read, reserved_zone, tray, user_read, volume_read, widgets_read, Action, Widget,
    BAR_PANEL_DUO, HEIGHT, INSET,
};
use matrixkit_image::{Fit, ImageAtlas};
use matrixkit_scene::scene as fam;
use matrixkit_scene::symbol;
use matrixkit_shell::{self as shell, Anchors, Key, KeyboardWish, Layer, Loom, PanePlace, Pict, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{hit, Keep, Placed, Rect};
use matrixkit_widgets as mkw;

/// Air between two chips and at the panel sides — the dock's measures.
const CHIP_GAP: i32 = mk::spacing::XS as i32;
const CHIP_SIDE: i32 = 10;
/// The bar floats 10 px below the top edge (R50).
const TOP: i32 = 10;
/// Inside the whole bar (no chips): air at the ends and between widgets.
const STRIP_SIDE: i32 = INSET as i32 + 4;
const STRIP_GAP: i32 = mk::spacing::XS as i32;
/// Menu card measures (a finger's row).
const MENU_W: u32 = 260;
const MENU_ROW: f32 = 44.0;
const MENU_SEP: f32 = 9.0;
const MENU_HEAD: f32 = 52.0;
/// The glass dialog's card (as `mkw::glass_dialog`).
const DIALOG_W: u32 = 340;
const DIALOG_H: u32 = 200;
/// Fallback card width when the window's frame is unknown.
const CARD_W: u32 = 360;
/// Round 113: the wiggle mode's row under the bar.
const EDIT_BAR_W: u32 = 260;
/// A drag longer than this is a scroll, not a tap.
const SLOP: f64 = 8.0;

fn veil_tint() -> mk::Rgba {
    mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.45 }
}

fn clear() -> mk::Rgba {
    mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Side {
    Left,
    Right,
    Center,
}

/// What a chip surface shows.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Chip {
    Widget(Widget),
    /// The whole centre zone in one chip.
    Center,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MenuKind {
    Session,
    Matrix,
    /// Round 113: the widget library of the wiggle mode («+ Widget»).
    Widgets,
}

/// One fleet surface of the bar.
#[derive(Clone, PartialEq, Debug)]
pub enum Card {
    Chip(Chip),
    /// The clear surface under an open menu: a tap closes it.
    Backdrop,
    Menu(MenuKind),
    CardVeil,
    AppCard,
    DialogVeil,
    Dialog,
    /// Round 105: the notification island (the chips flowed together).
    Island,
    /// Round 113: the wiggle mode's row under the bar («+ Widget», «Fertig»).
    EditBar,
}

/// Round 105: the island's timing.
const MORPH_SECONDS: f32 = 0.42;
const ISLAND_HOLD: f32 = 1.4;
const ISLAND_SPEED: f32 = 42.0;
const ISLAND_CHARS: usize = 140;
/// Round 106: the island stands this long before the chips beneath it
/// shrink away (it must be painted first), and the chips stand this long
/// at full band again before the island goes (they must be painted first).
const ISLAND_SETTLE: f32 = 0.15;

/// Round 105: a notification shown in the bar.
struct Island {
    item: mk::inbox::Item,
    line: String,
    icon: Option<std::path::PathBuf>,
    age: f32,
    /// Marquee: 0 hold at the start, 1 scrolling, 2 hold at the end.
    phase: u8,
    phase_t: f32,
    offset: f32,
    cycles: u32,
    /// Measured when laid: the line's width and the room it has.
    text_w: f32,
    room: f32,
    /// Round 107: the offset last painted (repaint only when it moved).
    painted: f32,
}

impl Island {
    fn overflow(&self) -> f32 {
        (self.text_w - self.room).max(0.0)
    }

    /// Advance the marquee; true when the picture moved.
    fn step(&mut self, dt: f32) -> bool {
        self.age += dt;
        if self.overflow() <= 0.0 || self.room <= 0.0 {
            return self.age < 0.25;
        }
        self.phase_t += dt;
        match self.phase {
            0 if self.phase_t >= ISLAND_HOLD => {
                self.phase = 1;
                self.phase_t = 0.0;
            }
            1 => {
                self.offset = (self.offset + ISLAND_SPEED * dt).min(self.overflow());
                if self.offset >= self.overflow() {
                    self.phase = 2;
                    self.phase_t = 0.0;
                }
                return true;
            }
            2 if self.phase_t >= ISLAND_HOLD => {
                self.cycles += 1;
                // Round 111: the last round stays at its end and leaves from
                // there (it jumped back to the start two frames before going).
                if self.cycles >= 2 {
                    self.phase = 3;
                    return false;
                }
                self.phase = 0;
                self.phase_t = 0.0;
                self.offset = 0.0;
                return true;
            }
            _ => {}
        }
        self.age < 0.25
    }

    /// Time to go: a short line after 5 s, a long one after two rounds (20 s at most).
    fn done(&self) -> bool {
        (self.age >= 5.0 && (self.overflow() <= 0.0 || self.cycles >= 2)) || self.age >= 20.0
    }
}

/// What a press means.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Deed {
    Chip(Widget),
    Tray(usize),
    Close,
    Act(Action),
    App(&'static str),
    Answer(bool),
    Settings,
    /// Round 105: a tap on the island.
    IslandOpen,
    /// Round 113: the wiggle mode — enter, «+ Widget», add one, «Fertig».
    EditEnter,
    EditAdd,
    AddWidget(Widget),
    EditDone,
    Swap,
    CloseApp,
    CrawlSwitch,
    Species(usize),
    CardClose,
    /// Round 31, the user card: a notification (open / dismiss), all away,
    /// do not disturb.
    InboxOpen(u32),
    InboxClear,
    Dnd,
    /// Round 34, the control card: a toggle or an action of the control
    /// centre, and a slider (volume, brightness) by its key.
    Control(&'static str),
    Slide(&'static str),
    /// Round 35, the app card: start the app at this index of `apps`.
    Launch(usize),
}

/// Which card the bar shows (round 31/34): the focused app's, the
/// user's (session + notifications) or the control centre's.
#[derive(Clone, Copy, PartialEq, Debug)]
enum CardKind {
    App,
    User,
    Central,
    /// Round 35: every installed app (the dock's «Apps» tile asks for it).
    Apps,
}

/// Where the app card lies.
#[derive(Clone, PartialEq, Debug)]
enum CardPlace {
    /// Exactly over the window (R214): output, x, y, w, h.
    At(String, i32, i32, u32, u32),
    /// Unknown geometry: centred on the window's output, this tall.
    Centered(String, u32),
}

/// The top bar's app card (R210/R214): which app, which window, what it offers.
#[derive(Clone)]
struct AppCard {
    window: mk::canvas::AppWindow,
    app_id: String,
    title: String,
    /// «Matrix-App · oberes Panel».
    about: String,
    /// R211: what crawls in it (None: not a house app — no Krabbler).
    crawl: Option<Vec<String>>,
    /// Wide window: two columns (app | Krabbler).
    wide: bool,
    /// A house window (mailbox): it has settings (root pane).
    light: bool,
    /// The app's windows lie on both panels.
    pair: bool,
    /// The device has panels (duo section shown).
    duo: bool,
    place: CardPlace,
    kind: CardKind,
    /// Scroll offset and what it scrolls over (set when drawn).
    roll: f32,
    content_h: f32,
    view_h: f32,
}

/// A drawn surface, kept for hit-testing (the house leaves it to us).
struct Laid {
    card: Card,
    placed: Vec<Placed>,
    roles: Vec<Option<usize>>,
    deeds: Vec<Deed>,
}

/// A press in progress: on which surface (None = the spine), on what.
struct Press {
    card: Option<Card>,
    deed: Option<Deed>,
    x: f64,
    y: f64,
    moved: bool,
    roll0: f32,
    /// Round 113: when it began, and whether the second stage answered.
    since: std::time::Instant,
    edited: bool,
}

pub enum Work {
    /// The focused window changed (event stream).
    Focus(Option<String>),
    /// Round 27: how bright the wallpaper is behind the chips.
    Backdrop(Option<(String, f32)>),
    /// Round 34: the control centre's state, read off the frame.
    Controls(mk::quick::State),
    /// Round 35: a word in the bar's mailbox (`mk::bar::ask`).
    Ask(String),
    /// Round 117: the outputs' sizes, measured off the loop.
    Sizes(Vec<(String, (f32, f32))>),
}

struct Bar {
    /// Round 27: (wallpaper, luma) behind the chip row — glass ink follows it.
    backdrop: Option<(String, f32)>,
    loom: Option<Loom<Work>>,
    palette: mk::Palette,
    watcher: mk::PaletteWatcher,
    /// Compositor glass under chips and cards: they draw no body.
    glass: bool,
    zones: (Vec<Widget>, Vec<Widget>, Vec<Widget>),
    chips: bool,
    output: Option<String>,
    user: String,
    /// Round 31: the notification history (mk::inbox), and its stamp.
    inbox: Vec<mk::inbox::Item>,
    inbox_stamp: Option<std::time::SystemTime>,
    /// Round 34: the control card's state (None until read), its slider
    /// tracks (card-local, by key) and the slider under the finger.
    controls: Option<mk::quick::State>,
    tracks: std::cell::RefCell<Vec<(&'static str, Rect)>>,
    drag: Option<&'static str>,
    /// Round 35: the app card's list (read when it opens).
    apps: Vec<mk::apps::Entry>,
    /// Round 95: the app list's icon files, looked up once — the list is laid
    /// on every scroll frame (twice). Cleared when palette or theme change.
    app_icons: std::cell::RefCell<std::collections::HashMap<String, Option<std::path::PathBuf>>>,
    icon_theme: Option<String>,
    clock: String,
    clock_format: &'static str,
    minute: u64,
    focus: Option<String>,
    /// The focus title as shown (elided to its room).
    focus_shown: String,
    battery: Option<(u8, bool)>,
    now_playing: Option<media::NowPlaying>,
    now_label: String,
    sys: sysinfo::System,
    cpu: f32,
    ram: String,
    volume: Option<String>,
    /// Round 46: the sound generation the volume was read at.
    volume_gen: u64,
    net: Option<String>,
    pulse_counter: u8,
    tray_items: std::sync::Arc<std::sync::Mutex<Vec<tray::TrayItem>>>,
    tray_command: std::sync::mpsc::Sender<tray::TrayCommand>,
    tray_sig: Vec<String>,
    /// Measured in view(): the chips in a row (chip, side, offset, width).
    row: Vec<(Chip, Side, i32, u32)>,
    panel_w: f32,
    menu: Option<MenuKind>,
    dialog: Option<Action>,
    card: Option<AppCard>,
    flash: mk::motion::TapFlash<Widget>,
    flashing: Vec<Widget>,
    flash_acc: f32,
    press: Option<Press>,
    laid: Vec<Laid>,
    main_deeds: Vec<Deed>,
    /// Surfaces that need a new picture on the next `Refresh::Panes`.
    stale: Vec<Card>,
    /// Round 59 (owner: «the card as if out of the button»): the card body
    /// grows out of the chip that opened it — 0 = the chip's size at its
    /// place, 1 = the whole card. `grown_for` is the card it grew for.
    grow: mk::motion::Spring,
    grown_for: Option<CardKind>,
    /// Round 60: the chip's place, measured once per opening (it asked
    /// niri for the output size on every frame — 50 ms each).
    grow_from: Rect,
    /// Round 61: where the dock's «Apps» tile sits (screen x, width).
    apps_from: Option<(f32, f32)>,
    /// Round 62: menus and the dialog pop out of their chip / the middle
    /// the same way (only without glass — niri draws the glass at the
    /// surface's full size, a zoom on it would look torn).
    pop: mk::motion::Spring,
    popped_for: Option<Card>,
    /// Round 63: the card is shrinking back into its chip; it goes when
    /// the spring arrives at 0.
    closing: bool,
    /// Round 117: the app card's content version (mk shell `pane_version`).
    card_rev: u64,
    /// Round 117: output sizes, measured in the background (opening a card
    /// asked niri synchronously — part of 166 ms before the user card moved).
    out_sizes: std::collections::HashMap<String, (f32, f32)>,
    sizes_tick: u32,
    /// Round 115: a screen film runs (the capture chip glows red).
    recording: bool,
    /// Round 113: the app card picks an app for the dock (its «+» in the
    /// wiggle mode) instead of starting it.
    picking: bool,
    /// Round 113: the wiggle mode (mk::edit), its clock, the chip dragged
    /// (widget, horizontal offset in points).
    editing: bool,
    edit_t: f32,
    edit_drag: Option<(Widget, f64)>,
    edit_acc: f32,
    /// Round 105: the notification island, its queue, the newest id seen,
    /// and the chips' flow: 0 = in their places, 1 = merged into the
    /// island's band (`morph_dir` +1 merging, −1 splitting, 0 at rest);
    /// `morph_rects` = each chip's (from, to) for this flow.
    island: Option<Island>,
    island_queue: std::collections::VecDeque<mk::inbox::Item>,
    /// Round 108: every notification seen, as (id, arrival) — the daemon
    /// counts from 1 again after a restart (a new session), while the
    /// inbox board can outlive it: «highest id seen» hid every new one.
    island_seen: std::collections::HashSet<(u32, u64)>,
    morph: f32,
    morph_dir: i8,
    morph_rects: Vec<(Chip, Rect, Rect)>,
    /// Round 106: seconds the island still stands while the chips come back
    /// beneath it (before they flow apart).
    split_wait: f32,
}

impl Bar {
    /// Measure the wallpaper behind the chip row off the surface (round 27).
    fn backdrop_ask(&self) {
        let (Some(loom), Some(out)) = (&self.loom, self.output.clone()) else { return };
        let band = (TOP + HEIGHT as i32) as f32;
        loom.spin(move || Work::Backdrop(matrixkit_image::backdrop::strip(&out, |w, _| (0.0, 0.0, w, band))));
    }

    fn new() -> Bar {
        let am_pm = mk::command::first_line("date", &["+%p"]).map(|p| !p.trim().is_empty()).unwrap_or(false);
        let (items, command) = tray::start();
        let chips = mk::device::bar_chips();
        // R209: ONE bar, on the lower panel of a duo (`bar-output` names
        // another); elsewhere `--output` or the compositor's choice.
        let output = if mk::device::is_duo() {
            Some(mk::device::duo_panel("bar-output", BAR_PANEL_DUO))
        } else {
            shell::output_wish()
        };
        let mut bar = Bar {
            palette: mk::Palette::load().unwrap_or_default(),
            watcher: mk::PaletteWatcher::new(),
            glass: mk::glass_wanted(),
            zones: widgets_read(),
            chips,
            output,
            user: user_read(),
            inbox: mk::inbox::read(),
            inbox_stamp: mk::inbox::stamp(),
            recording: mk::quick::recording(),
            card_rev: 0,
            out_sizes: std::collections::HashMap::new(),
            sizes_tick: 0,
            picking: false,
            editing: mk::edit::active(),
            edit_t: 0.0,
            edit_drag: None,
            edit_acc: 0.0,
            island: None,
            island_queue: std::collections::VecDeque::new(),
            island_seen: mk::inbox::read().iter().map(|i| (i.id, i.at)).collect(),
            morph: 0.0,
            morph_dir: 0,
            morph_rects: Vec::new(),
            split_wait: 0.0,
            controls: None,
            tracks: std::cell::RefCell::new(Vec::new()),
            drag: None,
            apps: Vec::new(),
            app_icons: Default::default(),
            icon_theme: matrixkit_icons::look(),
            clock: String::new(),
            clock_format: if am_pm { "+%a. %-I:%M %p" } else { "+%a. %H:%M" },
            minute: 0,
            focus: focus_read(),
            focus_shown: String::new(),
            battery: battery_read(),
            now_playing: None,
            now_label: String::new(),
            sys: sysinfo::System::new(),
            cpu: 0.0,
            ram: String::new(),
            volume: None,
            volume_gen: 0,
            net: None,
            pulse_counter: 0,
            tray_items: items,
            tray_command: command,
            tray_sig: Vec::new(),
            row: Vec::new(),
            panel_w: 0.0,
            menu: None,
            dialog: None,
            card: None,
            flash: mk::motion::TapFlash::default(),
            flashing: Vec::new(),
            flash_acc: 0.0,
            press: None,
            laid: Vec::new(),
            main_deeds: Vec::new(),
            stale: Vec::new(),
            grow: mk::motion::Spring::with(1.0, 0.34, 0.06),
            grown_for: None,
            grow_from: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            apps_from: None,
            pop: mk::motion::Spring::with(1.0, GROW_SECONDS, 0.22),
            popped_for: None,
            closing: false,
            backdrop: None,
            loom: None,
        };
        bar.clock_tick();
        bar.measure();
        // The claim stands whenever the bar has chips; the switch is read
        // live on both sides (mk::island::active).
        if bar.chips {
            mk::island::claim();
        }
        bar
    }

    fn glide_rest(&mut self, dt: f32) -> Refresh {
        // Round 105: the island (and the chips flowing) first.
        if let Some(r) = self.island_glide(dt) {
            if !matches!(r, Refresh::Calm) {
                return r;
            }
        }
        // Round 62: a menu / the dialog popping.
        if let Some(c) = self.popped_for.clone() {
            if !self.pop.is_settled() {
                self.pop.tick(dt);
                if (self.pop.value - 1.0).abs() < 0.01 && self.pop.velocity.abs() < 0.3 {
                    self.pop.value = 1.0;
                    self.pop.velocity = 0.0;
                }
                if !self.stale.contains(&c) {
                    self.stale.push(c);
                }
                return Refresh::Panes;
            }
        }
        // Round 59: the card grows out of its chip — the pane at frame rate.
        if self.card.is_some() && !self.grow.is_settled() {
            self.grow.tick(dt);
            // Arrived for the eye: stop here, so the full card (content
            // and all) is drawn once, not through the spring's long tail.
            let goal = self.grow.target;
            if (self.grow.value - goal).abs() < 0.01 && self.grow.velocity.abs() < 0.3 {
                self.grow.value = goal;
                self.grow.velocity = 0.0;
            }
            // Round 63: shrunk back into the chip — now it really goes.
            if self.closing && self.grow.is_settled() {
                return self.close_card();
            }
            if !self.stale.contains(&Card::AppCard) {
                self.stale.push(Card::AppCard);
            }
            return Refresh::Panes;
        }
        self.flash_acc += dt;
        if self.flash_acc < 0.05 {
            return Refresh::Calm;
        }
        self.flash_acc = 0.0;
        self.flash.tick();
        for w in self.flashing.clone() {
            self.touch_chip_of(w);
        }
        let flash = &self.flash;
        self.flashing.retain(|w| flash.contains(w));
        if self.chips {
            Refresh::Panes
        } else {
            Refresh::Dirty
        }
    }

    /// Round 117: an output's size — remembered (refreshed in the
    /// background every few seconds), asked only the first time.
    fn out_size(&mut self, out: &str) -> Option<(f32, f32)> {
        if let Some(s) = self.out_sizes.get(out) {
            return Some(*s);
        }
        let s = mk::canvas::output_size(out)?;
        self.out_sizes.insert(out.to_string(), s);
        Some(s)
    }

    // ------------------------------------------------------------ editing

    /// Round 113: into the wiggle mode (the long press's second stage, the
    /// Matrix menu, or the dock asked).
    fn edit_enter(&mut self, by_press: bool) -> Refresh {
        if by_press {
            mk::feel::stage();
            mk::edit::enter();
        }
        eprintln!("[bar] Bearbeiten: an");
        self.editing = true;
        self.edit_t = 0.0;
        self.menu = None;
        self.stale = self.plan();
        Refresh::Panes
    }

    fn edit_leave(&mut self, tell: bool) -> Refresh {
        if tell {
            mk::edit::leave();
        }
        eprintln!("[bar] Bearbeiten: aus");
        self.editing = false;
        self.edit_drag = None;
        if self.menu == Some(MenuKind::Widgets) {
            self.menu = None;
        }
        self.stale = self.plan();
        Refresh::Panes
    }

    /// The zones changed in the wiggle mode: store them and lay the row anew.
    fn zones_store(&mut self, zones: (Vec<Widget>, Vec<Widget>, Vec<Widget>)) -> Refresh {
        crate::widgets_write(&zones);
        self.zones = zones;
        mk::feel::select();
        // view() lays the row and syncs the fleet; the glide repaints
        self.stale = self.plan();
        Refresh::Dirty
    }

    fn widget_remove(&mut self, w: Widget) -> Refresh {
        let mut z = self.zones.clone();
        for zone in [&mut z.0, &mut z.1, &mut z.2] {
            zone.retain(|x| *x != w);
        }
        eprintln!("[bar] Bearbeiten: «{}» entfernt", w.word());
        self.zones_store(z)
    }

    fn widget_add(&mut self, w: Widget) -> Refresh {
        let mut z = self.zones.clone();
        // new widgets join the right group at its left end
        z.2.insert(0, w);
        eprintln!("[bar] Bearbeiten: «{}» hinzugefügt", w.word());
        self.menu = None;
        self.zones_store(z)
    }

    /// A dragged chip dropped: its new place inside its own zone.
    fn edit_drop(&mut self) -> Refresh {
        let Some((w, dx)) = self.edit_drag.take() else { return Refresh::Calm };
        let mut z = self.zones.clone();
        let zone = if z.0.contains(&w) { &mut z.0 } else if z.1.contains(&w) { &mut z.1 } else { &mut z.2 };
        let centres: Vec<f32> = zone
            .iter()
            .filter_map(|x| self.row.iter().find(|(c, ..)| *c == Chip::Widget(*x)).map(|(_, side, off, cw)| self.x_of(*side, *off, *cw) + *cw as f32 / 2.0))
            .collect();
        let Some(from) = zone.iter().position(|x| *x == w) else { return Refresh::Panes };
        if centres.len() != zone.len() {
            return Refresh::Panes;
        }
        // the right zone is laid from the right edge inward, but `zones`
        // keeps reading order — the centres say where things are on screen
        let mut order: Vec<(f32, Widget)> = centres.iter().copied().zip(zone.iter().copied()).collect();
        order.sort_by(|a, b| a.0.total_cmp(&b.0));
        let screen: Vec<Widget> = order.iter().map(|(_, x)| *x).collect();
        let sorted_centres: Vec<f32> = order.iter().map(|(c, _)| *c).collect();
        let at = screen.iter().position(|x| *x == w).unwrap_or(from);
        let to = mk::edit::slot_at(&sorted_centres, sorted_centres[at] + dx as f32);
        let mut moved = screen.clone();
        mk::edit::move_item(&mut moved, at, to);
        if moved == screen {
            self.stale = self.plan();
            return Refresh::Panes;
        }
        *zone = moved;
        eprintln!("[bar] Bearbeiten: «{}» verschoben", w.word());
        self.zones_store(z)
    }

    // ------------------------------------------------------------ island

    /// Round 31: the inbox changed (a new notification, one handled)?
    /// True when every chip must be laid anew (the unread dot came or went).
    fn inbox_check(&mut self) -> bool {
        let stamp = mk::inbox::stamp();
        if stamp == self.inbox_stamp {
            return false;
        }
        self.inbox_stamp = stamp;
        let fresh = mk::inbox::read();
        if fresh == self.inbox {
            return false;
        }
        let had = self.inbox.is_empty();
        self.inbox = fresh;
        self.touch_chip_of(Widget::User);
        if self.card.as_ref().is_some_and(|c| c.kind == CardKind::User) {
            self.touch(Card::AppCard);
        }
        self.island_note();
        had != self.inbox.is_empty()
    }

    /// Round 105: new notifications go into the island's queue.
    fn island_note(&mut self) {
        if !self.chips || !mk::island::enabled() {
            return;
        }
        let mut fresh: Vec<mk::inbox::Item> = self.inbox.iter().filter(|i| !self.island_seen.contains(&(i.id, i.at))).cloned().collect();
        fresh.sort_by_key(|i| (i.at, i.id));
        for i in &fresh {
            self.island_seen.insert((i.id, i.at));
        }
        // forget what left the board (it stays small)
        let now_ids: std::collections::HashSet<(u32, u64)> = self.inbox.iter().map(|i| (i.id, i.at)).collect();
        self.island_seen.retain(|k| now_ids.contains(k));
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        for item in fresh {
            // a minute late is history, not news
            if now.saturating_sub(item.at) <= 60 && !Self::dnd() {
                self.island_queue.push_back(item);
            }
        }
        while self.island_queue.len() > 5 {
            self.island_queue.pop_front();
        }
    }

    /// Start the next island when the bar is free (no card, menu, dialog).
    fn island_try(&mut self) -> bool {
        if self.editing || self.island.is_some() || self.morph_dir != 0 || self.card.is_some() || self.menu.is_some() || self.dialog.is_some() || self.panel_w <= 0.0 {
            return false;
        }
        let Some(item) = self.island_queue.pop_front() else { return false };
        let line = mk::island::line(&item.title, &item.text, ISLAND_CHARS);
        let icon = island_icon(&item.app, &self.palette);
        // Each chip from its place to its share of the band (touching, in
        // row order) — they flow together into the island's glass.
        let mut now: Vec<(Chip, Rect)> = self.row.iter().map(|(c, side, off, w)| (*c, Rect { x: self.x_of(*side, *off, *w), y: 0.0, w: *w as f32, h: HEIGHT as f32 })).collect();
        now.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
        let band = self.island_band();
        let total: f32 = now.iter().map(|(_, r)| r.w).sum::<f32>().max(1.0);
        let mut x = band.x;
        let n = now.len();
        self.morph_rects = now
            .into_iter()
            .enumerate()
            .map(|(i, (c, from))| {
                let w = if i + 1 == n { band.x + band.w - x } else { (from.w * band.w / total).round() };
                let to = Rect { x, y: 0.0, w: w.max(1.0), h: from.h };
                x += w;
                (c, from, to)
            })
            .collect();
        self.island = Some(Island { item, line, icon, age: 0.0, phase: 0, phase_t: 0.0, offset: 0.0, cycles: 0, text_w: 0.0, room: 0.0, painted: 0.0 });
        self.morph = 0.0;
        self.morph_dir = if mk::motion_reduced() { 0 } else { 1 };
        if self.morph_dir == 0 {
            self.morph = 1.0;
        }
        self.stale = self.plan();
        true
    }

    /// The island's band in the bar (logical, the chips' row).
    fn island_band(&self) -> Rect {
        Rect { x: CHIP_SIDE as f32, y: 0.0, w: (self.panel_w - 2.0 * CHIP_SIDE as f32).max(1.0), h: HEIGHT as f32 }
    }

    /// Let the island go: the chips flow apart again.
    fn island_end(&mut self) -> Refresh {
        if self.island.is_none() || self.morph_dir < 0 {
            return Refresh::Calm;
        }
        self.morph_dir = -1;
        self.split_wait = ISLAND_SETTLE;
        if mk::motion_reduced() {
            self.morph = 0.0;
            self.split_wait = 0.0;
        }
        self.stale = self.plan();
        Refresh::Panes
    }

    /// Round 106: the chips are hidden beneath a painted island (no seams
    /// of their glass shine through it).
    fn chips_hidden(&self) -> bool {
        self.morph_dir == 0 && self.morph >= 1.0 && self.island.as_ref().is_some_and(|i| i.age >= ISLAND_SETTLE)
    }

    /// Round 106: the island card is in the plan.
    fn island_up(&self) -> bool {
        self.island.is_some() && self.morph >= 1.0 && (self.morph_dir == 0 || (self.morph_dir < 0 && self.split_wait > 0.0))
    }

    /// A chip's rectangle while the chips flow (None at rest).
    fn morph_rect(&self, chip: &Chip) -> Option<Rect> {
        if self.morph <= 0.0 && self.morph_dir == 0 {
            return None;
        }
        let (_, from, to) = self.morph_rects.iter().find(|(c, ..)| c == chip)?;
        if self.chips_hidden() {
            let band = self.island_band();
            return Some(Rect { x: band.x + band.w / 2.0, y: HEIGHT as f32 / 2.0, w: 1.0, h: 1.0 });
        }
        // Round 108: ONE step — merging the chips stand on their share of
        // the band at once, splitting back on their places; the compositor
        // glides the glass between (niri 0019 «glass-glide»). Stepping every
        // frame waited for a configure per step: 13–39 ms, jerky.
        if self.morph_dir < 0 && self.split_wait <= 0.0 {
            return Some(*from);
        }
        // First two frames: empty at the old place (the faces leave), then
        // the jump — else the old faces rode on the gliding glass.
        if self.morph_dir > 0 && self.morph < 0.08 {
            return Some(*from);
        }
        Some(*to)
    }

    /// The island's frame step (round 105); None = nothing of the island moves.
    fn island_glide(&mut self, dt: f32) -> Option<Refresh> {
        if self.morph_dir < 0 && self.split_wait > 0.0 {
            // the chips come back beneath the island first
            self.split_wait = (self.split_wait - dt).max(0.0);
            self.stale = self.plan();
            return Some(Refresh::Panes);
        }
        if self.morph_dir != 0 {
            self.morph = (self.morph + dt / MORPH_SECONDS * self.morph_dir as f32).clamp(0.0, 1.0);
            if self.morph >= 1.0 && self.morph_dir > 0 {
                self.morph_dir = 0;
                self.stale = self.plan();
            } else if self.morph <= 0.0 && self.morph_dir < 0 {
                self.morph_dir = 0;
                self.island = None;
                self.morph_rects.clear();
                if !self.island_try() {
                    self.stale = self.plan();
                }
            } else {
                self.stale = self.plan();
            }
            return Some(Refresh::Panes);
        }
        let island = self.island.as_mut()?;
        let before = island.age;
        let moved = island.step(dt);
        if island.done() {
            return Some(self.island_end());
        }
        if before < ISLAND_SETTLE && island.age >= ISLAND_SETTLE {
            // the island is painted: the chips beneath it shrink away
            self.stale = self.plan();
            return Some(Refresh::Panes);
        }
        // Round 107: a new picture only when the text moved a visible step
        // (half a logical point) — or while it fades in.
        let step = (island.offset - island.painted).abs() >= 0.5 || island.age < 0.3;
        if moved && step {
            island.painted = island.offset;
            self.touch(Card::Island);
            return Some(Refresh::Panes);
        }
        Some(Refresh::Calm)
    }

    /// The island's picture.
    fn lay_island(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, deeds: &mut Vec<Deed>, w: f32, h: f32) {
        let p = self.palette;
        let (ink, _) = mk::glass::ink(&p, self.backdrop.as_ref().map(|b| b.1));
        let glass = self.glass;
        let Some(island) = self.island.as_mut() else { return };
        if !glass {
            b.quad(0.0, 0.0, w, h, p.surface_container_high, mk::radius::LARGE, None, None);
        }
        deeds.push(Deed::IslandOpen);
        let whole = b.quad(0.0, 0.0, w, h, clear(), mk::radius::LARGE, Some(deeds.len() - 1), None);
        let fade = (island.age / 0.25).clamp(0.0, 1.0);
        let ink = mk::Rgba { a: ink.a * fade, ..ink };
        let cy = h / 2.0;
        let side = 26.0;
        let ix = INSET + 2.0;
        match &island.icon {
            Some(path) => {
                let home = Rect { x: ix, y: cy - side / 2.0, w: side, h: side };
                b.pict(Pict { path: path.clone(), box_w: side as u32, box_h: side as u32, fit: Fit::Contain, x: ix, y: cy - side / 2.0, radius: 6.0, home, anchor: whole, clip: None });
            }
            None => {
                let symfam = matrixkit_scene::symbol_family();
                let px = mk::typo::symbol_px(mk::font_size::LARGE);
                let g = symbol::NOTIFICATIONS.to_string();
                let (gw, gh) = engine.measure_family(symfam, &g, px, mk::typo::Weight::Normal);
                b.text(engine, g, px, mk::typo::Weight::Normal, symfam, ink, ix + (side - gw) / 2.0, cy - gh / 2.0, Some(deeds.len() - 1), None);
            }
        }
        let _ = atlas;
        let tx = ix + side + mk::spacing::S;
        let room = (w - tx - INSET - 4.0).max(0.0);
        let st = mk::typo::CALLOUT;
        let (tw, th) = engine.measure_styled(&island.line, st.px(), st.weight);
        island.text_w = tw;
        island.room = room;
        // The text slides left and vanishes just before the icon.
        let clip = Rect { x: tx, y: 0.0, w: room, h };
        b.styled(engine, island.line.clone(), st, ink, tx - island.offset, cy - th / 2.0, Some(deeds.len() - 1), Some(clip));
    }

    // ------------------------------------------------------------ readers

    /// The clock string — read when the minute turns (cheap check).
    fn clock_tick(&mut self) -> bool {
        let minute = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() / 60).unwrap_or(0);
        if minute == self.minute && !self.clock.is_empty() {
            return false;
        }
        self.minute = minute;
        // Round 56: the kit's clock instead of starting `date` each minute.
        match Some(mk::time::clock_label(self.clock_format.contains("%p"))).filter(|z| !z.is_empty()) {
            Some(z) if z != self.clock => {
                self.clock = z;
                true
            }
            _ => false,
        }
    }

    /// The 2-s readings; returns the chips whose face changed.
    fn measure(&mut self) -> Vec<Widget> {
        let mut changed = Vec::new();
        let np = media::read();
        let label = np.as_ref().map_or_else(String::new, |np| {
            let base = if np.title.is_empty() { np.artist.clone() } else { np.title.clone() };
            mk::elide(&base, 25)
        });
        if label != self.now_label || np.as_ref().map(|n| n.playing) != self.now_playing.as_ref().map(|n| n.playing) || np.is_some() != self.now_playing.is_some() {
            changed.push(Widget::Media);
        }
        self.now_playing = np;
        self.now_label = label;
        let wants_pulse = [&self.zones.0, &self.zones.1, &self.zones.2].iter().any(|z| z.contains(&Widget::Pulse));
        if wants_pulse {
            self.sys.refresh_cpu_usage();
            self.sys.refresh_memory();
            self.cpu = self.sys.global_cpu_usage();
            self.ram = mk::format::bytes_decimal(self.sys.used_memory());
            // Round 46: the volume only after a sound change.
            let gen = mk::audio::generation();
            if gen != self.volume_gen {
                self.volume_gen = gen;
                self.volume = volume_read();
            }
            changed.push(Widget::Pulse);
        }
        self.pulse_counter = self.pulse_counter.wrapping_add(1);
        if self.pulse_counter % 5 == 0 {
            if wants_pulse {
                self.net = net_read();
            }
            let new = battery_read();
            // Round 75: keep the shared smoothed draw alive (the estimate in
            // the user card and the settings reads it).
            let _ = mk::power::smoothed_draw();
            // Charge chime (R33): only on plugging in.
            if let (Some((_, was)), Some((_, now))) = (self.battery, new) {
                if !was && now {
                    std::thread::spawn(|| mk::feedback::now("geraete", "06-geraet-verbunden.wav"));
                }
            }
            if new != self.battery {
                changed.push(Widget::Battery);
            }
            self.battery = new;
            // A fallback for the event stream: the focus every 10 s.
            let focus = focus_read();
            if focus != self.focus {
                self.focus = focus;
                changed.push(Widget::Focus);
            }
        }
        let sig: Vec<String> = self
            .tray_items
            .lock()
            .map(|t| t.iter().map(|i| format!("{}{}{}", i.service, i.path, i.icon.as_ref().map_or(0, |ic| ic.2.len()))).collect())
            .unwrap_or_default();
        if sig != self.tray_sig {
            self.tray_sig = sig;
            changed.push(Widget::Tray);
        }
        changed
    }

    fn tray(&self) -> Vec<tray::TrayItem> {
        self.tray_items.lock().map(|t| t.clone()).unwrap_or_default()
    }

    fn dnd() -> bool {
        matches!(mk::settings::read("do-not-disturb").as_deref(), Some("on") | Some("an"))
    }

    /// Has this widget something to show on this device right now?
    fn live(&self, w: Widget) -> bool {
        match w {
            Widget::Battery => self.battery.is_some(),
            Widget::Media => self.now_playing.is_some(),
            Widget::Tray => !self.tray_sig.is_empty(),
            // Round 31: the notifications live in the user card now.
            Widget::Bell => false,
            _ => true,
        }
    }

    /// What tapping this widget does (None = nothing to press).
    fn pressable(&self, w: Widget) -> bool {
        match w {
            Widget::Matrix | Widget::User | Widget::Bell | Widget::Central | Widget::Capture | Widget::Media => true,
            // R210 duo grammar: the name chip holds the window's menu.
            // An open card keeps the chip pressable: the second tap closes it
            // (the veil takes the focus away, 02.11.2026).
            Widget::Focus => self.chips && (self.focus.is_some() || self.card.as_ref().is_some_and(|c| c.kind == CardKind::App)),
            Widget::Clock | Widget::Pulse | Widget::Battery | Widget::Tray => false,
        }
    }

    fn pulse_text(&self) -> String {
        let mut parts: Vec<String> = vec![format!("CPU {:.0} %", self.cpu), self.ram.clone()];
        if let Some(v) = &self.volume {
            parts.push(format!("Ton {v}"));
        }
        if let Some(n) = &self.net {
            parts.push(n.clone());
        }
        parts.join("  ·  ")
    }

    fn glyph_of(&self, w: Widget) -> Option<char> {
        Some(match w {
            Widget::Matrix => symbol::APPS,
            Widget::Bell => {
                if Self::dnd() {
                    symbol::VOLUME_OFF
                } else {
                    symbol::NOTIFICATIONS
                }
            }
            Widget::Central => symbol::TUNE,
            Widget::Capture => symbol::IMAGE,
            Widget::Battery => {
                if self.battery.is_some_and(|b| b.1) {
                    symbol::BATTERY_CHARGING
                } else {
                    symbol::BATTERY_FULL
                }
            }
            Widget::Media => {
                if self.now_playing.as_ref().is_some_and(|n| n.playing) {
                    symbol::PAUSE
                } else {
                    symbol::PLAY_ARROW
                }
            }
            _ => return None,
        })
    }

    /// The text of a widget's face.
    fn text_of(&self, w: Widget) -> String {
        match w {
            Widget::User => self.user.clone(),
            Widget::Focus => self.focus_shown.clone(),
            Widget::Clock => self.clock.clone(),
            Widget::Pulse => self.pulse_text(),
            Widget::Battery => self.battery.map(|(p, _)| format!("{p} %")).unwrap_or_default(),
            Widget::Media => self.now_label.clone(),
            _ => String::new(),
        }
    }

    // ------------------------------------------------------------ layout

    /// A face's natural width (content + its air), measured.
    fn face_width(&self, engine: &mut TextEngine, w: Widget) -> f32 {
        let m = mk::spacing::M;
        let st = mk::typo::CALLOUT;
        let text = |engine: &mut TextEngine, s: &str| if s.is_empty() { 0.0 } else { engine.measure_styled(s, st.px(), st.weight).0 };
        let glyph = |engine: &mut TextEngine, g: char, px: f32| engine.measure_family(matrixkit_scene::symbol_family(), &g.to_string(), px, mk::typo::Weight::Normal).0;
        let large = mk::typo::symbol_px(mk::font_size::LARGE);
        let small = mk::typo::symbol_px(mk::typo::CALLOUT.size + 3.0);
        // A pressable chip keeps the family button's inset (iced R50c).
        let button = |content: f32| content + 2.0 * m + 2.0 * INSET;
        let plain = |content: f32| content + 2.0 * m;
        match w {
            Widget::Matrix | Widget::Bell | Widget::Central | Widget::Capture => button(glyph(engine, self.glyph_of(w).unwrap_or(' '), large)),
            Widget::User => button(text(engine, &self.user)),
            Widget::Media => button(glyph(engine, self.glyph_of(w).unwrap_or(' '), large) + mk::spacing::S + text(engine, &self.now_label)),
            Widget::Focus => plain(text(engine, &self.focus_shown)),
            Widget::Clock => plain(text(engine, &self.clock)),
            Widget::Pulse => plain(text(engine, &self.pulse_text())),
            Widget::Battery => plain(glyph(engine, self.glyph_of(w).unwrap_or(' '), small) + mk::spacing::XS + text(engine, &self.text_of(w))),
            Widget::Tray => {
                let n = self.tray_sig.len() as f32;
                n * 28.0 + (n - 1.0).max(0.0) * mk::spacing::XXS + 2.0 * INSET
            }
        }
    }

    /// Lay the row: every live widget's chip width and its place along
    /// the panel. The focus title yields to everything else (R209: never
    /// into the clock).
    fn lay_row(&mut self, engine: &mut TextEngine, panel_w: f32) {
        self.panel_w = panel_w;
        let (side, gap) = if self.chips { (CHIP_SIDE, CHIP_GAP) } else { (STRIP_SIDE, STRIP_GAP) };
        let live = |list: &[Widget], bar: &Bar| list.iter().copied().filter(|w| bar.live(*w)).collect::<Vec<_>>();
        let (left, center, right) = (live(&self.zones.0, self), live(&self.zones.1, self), live(&self.zones.2, self));
        let snap = |w: f32| fam::snap_up(w, 8);
        let width = |bar: &Bar, engine: &mut TextEngine, w: Widget| snap(bar.face_width(engine, w));
        let center_w = if center.is_empty() {
            0
        } else {
            let sum: f32 = center.iter().map(|w| self.face_width(engine, *w)).sum::<f32>() + (center.len() as f32 - 1.0) * mk::spacing::M;
            snap(sum)
        };
        let right_w: Vec<u32> = right.iter().rev().map(|w| width(self, engine, *w)).collect();
        // The focus title's room: what is left of the left half (or before
        // the right group when nothing is centred).
        // While the card is open the chip keeps the card's app name.
        let mut title = self.card.as_ref().filter(|c| c.kind == CardKind::App).map(|c| c.title.clone()).or_else(|| self.focus.clone()).unwrap_or_else(|| String::from("Matrix"));
        title = mk::elide(&title, mk::device::bar_title_chars());
        let others: f32 = left.iter().filter(|w| **w != Widget::Focus).map(|w| width(self, engine, *w) as f32 + gap as f32).sum();
        let right_sum: f32 = right_w.iter().map(|w| *w as f32 + gap as f32).sum();
        let limit = if center_w > 0 { panel_w / 2.0 - center_w as f32 / 2.0 } else { panel_w - side as f32 - right_sum };
        let room = (limit - side as f32 - others - 2.0 * gap as f32 - 2.0 * mk::spacing::M - 8.0).max(24.0);
        let st = mk::typo::CALLOUT;
        self.focus_shown = fam::ellipsize(engine, &title, st.px(), st.weight, room);
        let left_w: Vec<u32> = left.iter().map(|w| width(self, engine, *w)).collect();
        let mut row = Vec::new();
        for ((w, off), wd) in left.iter().zip(fam::chip_offsets(&left_w, side, gap)).zip(&left_w) {
            row.push((Chip::Widget(*w), Side::Left, off, *wd));
        }
        for ((w, off), wd) in right.iter().rev().zip(fam::chip_offsets(&right_w, side, gap)).zip(&right_w) {
            row.push((Chip::Widget(*w), Side::Right, off, *wd));
        }
        if center_w > 0 {
            row.push((Chip::Center, Side::Center, 0, center_w));
        }
        self.row = row;
    }

    /// The x of a row entry inside a surface `panel_w` wide.
    fn x_of(&self, side: Side, off: i32, w: u32) -> f32 {
        match side {
            Side::Left => off as f32,
            Side::Right => self.panel_w - off as f32 - w as f32,
            Side::Center => (self.panel_w - w as f32) / 2.0,
        }
    }

    // ------------------------------------------------------------ faces

    /// Lay one widget's face centred in its box (x, y, bw, bh). A pressable
    /// widget gets a whole-box target first.
    #[allow(clippy::too_many_arguments)]
    fn lay_face(&self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, deeds: &mut Vec<Deed>, w: Widget, x: f32, y: f32, bw: f32, bh: f32) {
        let p = self.palette;
        // Chips are glass over the wallpaper: their ink follows the backdrop.
        let (ink, ink2) = mk::glass::ink(&p, self.backdrop.as_ref().map(|b| b.1));
        let role = self.pressable(w).then(|| {
            deeds.push(Deed::Chip(w));
            deeds.len() - 1
        });
        if let Some(r) = role {
            b.quad(x, y, bw, bh, clear(), mk::radius::LARGE, Some(r), None);
        }
        let symfam = matrixkit_scene::symbol_family();
        let st = mk::typo::CALLOUT;
        let (cx, cy) = (x + bw / 2.0, y + bh / 2.0);
        let large = mk::typo::symbol_px(mk::font_size::LARGE);
        match w {
            Widget::Tray => {
                let items = self.tray();
                let mut ix = x + INSET;
                for (i, it) in items.iter().enumerate() {
                    deeds.push(Deed::Tray(i));
                    let r = deeds.len() - 1;
                    let slot = b.quad(ix, cy - 14.0, 28.0, 28.0, clear(), mk::radius::SMALL, Some(r), None);
                    match &it.icon {
                        Some((iw, ih, rgba)) => {
                            let key = std::path::PathBuf::from(format!("/matrix-bar/tray/{}{}-{}", it.service, it.path, rgba.len()));
                            atlas.put(&key, *iw, *ih, rgba.clone());
                            let home = Rect { x: ix, y: cy - 14.0, w: 28.0, h: 28.0 };
                            b.pict(Pict { path: key, box_w: 20, box_h: 20, fit: Fit::Contain, x: ix + 4.0, y: cy - 10.0, radius: 0.0, home, anchor: slot, clip: None });
                        }
                        None => {
                            let s = it.title.chars().next().unwrap_or('•').to_string();
                            let (tw, th) = engine.measure_styled(&s, st.px(), st.weight);
                            b.styled(engine, s, st, ink, ix + (28.0 - tw) / 2.0, cy - th / 2.0, Some(r), None);
                        }
                    }
                    ix += 28.0 + mk::spacing::XXS;
                }
            }
            _ => {
                let glyph = self.glyph_of(w);
                let text = self.text_of(w);
                let gpx = if w == Widget::Battery { mk::typo::symbol_px(st.size + 3.0) } else { large };
                let (gw, gh) = glyph.map(|g| engine.measure_family(symfam, &g.to_string(), gpx, mk::typo::Weight::Normal)).unwrap_or((0.0, 0.0));
                let (tw, th) = if text.is_empty() { (0.0, 0.0) } else { engine.measure_styled(&text, st.px(), st.weight) };
                let between = if glyph.is_some() && !text.is_empty() {
                    if w == Widget::Battery {
                        mk::spacing::XS
                    } else {
                        mk::spacing::S
                    }
                } else {
                    0.0
                };
                let tint = if matches!(w, Widget::Battery | Widget::Pulse) { ink2 } else { ink };
                let mut gx = cx - (gw + between + tw) / 2.0;
                if let Some(g) = glyph {
                    b.text(engine, g.to_string(), gpx, mk::typo::Weight::Normal, symfam, tint, gx, cy - gh / 2.0, role, None);
                    gx += gw + between;
                }
                if !text.is_empty() {
                    b.styled(engine, text, st, tint, gx, cy - th / 2.0, role, None);
                }
                // Round 31: unread notifications — a dot at the user's name.
                if w == Widget::User && !self.inbox.is_empty() {
                    b.quad(gx + tw + 3.0, cy - th / 2.0 - 1.0, 8.0, 8.0, p.primary, 4.0, role, None);
                }
            }
        }
    }

    /// The centre zone's widgets side by side, centred in (x, y, bw, bh).
    #[allow(clippy::too_many_arguments)]
    fn lay_center(&self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, deeds: &mut Vec<Deed>, x: f32, y: f32, bw: f32, bh: f32) {
        let center: Vec<Widget> = self.zones.1.iter().copied().filter(|w| self.live(*w)).collect();
        let widths: Vec<f32> = center.iter().map(|w| self.face_width(engine, *w)).collect();
        let total = widths.iter().sum::<f32>() + (center.len() as f32 - 1.0).max(0.0) * mk::spacing::M;
        let mut cx = x + (bw - total) / 2.0;
        for (w, wd) in center.iter().zip(widths) {
            self.lay_face(b, engine, atlas, deeds, *w, cx, y, wd, bh);
            cx += wd + mk::spacing::M;
        }
    }

    /// Is this widget's menu or card open (its chip stays lit)?
    fn held_open(&self, w: Widget) -> bool {
        (w == Widget::User && self.menu == Some(MenuKind::Session))
            || (w == Widget::Matrix && self.menu == Some(MenuKind::Matrix))
            || (w == Widget::Focus && self.card.as_ref().is_some_and(|c| c.kind == CardKind::App))
            || (w == Widget::User && self.card.as_ref().is_some_and(|c| c.kind == CardKind::User))
            || (w == Widget::Central && self.card.as_ref().is_some_and(|c| c.kind == CardKind::Central))
    }

    fn pressed_on(&self, card: &Card) -> bool {
        self.press.as_ref().is_some_and(|pr| pr.card.as_ref() == Some(card) && pr.deed.is_some() && !pr.moved)
    }

    // ------------------------------------------------------------ menus

    /// The menu's rows: None = separator.
    fn menu_rows(&self, kind: MenuKind) -> Vec<Option<(char, String, Deed, Option<String>)>> {
        match kind {
            MenuKind::Session => {
                let mut rows = Vec::new();
                for a in Action::ALLE {
                    if matches!(a, Action::Standby) {
                        rows.push(None);
                    }
                    rows.push(Some((a.glyph(), a.title().to_string(), Deed::Act(a), None)));
                }
                rows
            }
            MenuKind::Matrix => {
                let badge = mk::badge::read("matrix-updater").map(|n| if n == "1" { String::from("1 Update") } else { format!("{n} Updates") });
                vec![
                    Some((symbol::INFO, String::from("Über diese Matrix"), Deed::App("matrix-help"), None)),
                    None,
                    Some((symbol::TUNE, String::from("Systemeinstellungen …"), Deed::App("matrix-settings"), None)),
                    Some((symbol::RESTART, String::from("Softwareupdate …"), Deed::App("matrix-updater"), badge)),
                    None,
                    Some((symbol::WIDGETS, String::from("Leiste & Dock anpassen"), Deed::EditEnter, None)),
                    None,
                    Some((symbol::LOCK, String::from("Bildschirm sperren"), Deed::Act(Action::Lock), None)),
                    Some((symbol::LOGOUT, format!("{} abmelden …", self.user), Deed::Act(Action::Logout), None)),
                ]
            }
            MenuKind::Widgets => {
                let (l, c, r) = &self.zones;
                let missing: Vec<Widget> = Widget::ALL.into_iter().filter(|w| !l.contains(w) && !c.contains(w) && !r.contains(w)).collect();
                if missing.is_empty() {
                    return vec![Some((symbol::CHECK, String::from("Alle Widgets sind in der Leiste"), Deed::EditDone, None))];
                }
                missing.into_iter().map(|w| Some((self.glyph_of(w).unwrap_or(symbol::WIDGETS), w.label().to_string(), Deed::AddWidget(w), None))).collect()
            }
        }
    }

    fn menu_height(&self, kind: MenuKind) -> u32 {
        let rows = self.menu_rows(kind);
        let n = rows.iter().filter(|r| r.is_some()).count() as f32;
        let seps = rows.len() as f32 - n;
        let head = if kind == MenuKind::Session { MENU_HEAD } else { 0.0 };
        (2.0 * mk::spacing::S + head + n * MENU_ROW + seps * MENU_SEP).ceil() as u32
    }

    fn lay_menu(&self, b: &mut SceneBuilder, engine: &mut TextEngine, deeds: &mut Vec<Deed>, kind: MenuKind, w: f32, h: f32) {
        let p = self.palette;
        if !self.glass {
            b.quad(0.0, 0.0, w, h, p.surface_container_high, mk::CORNER_RADIUS + 6.0, None, None);
        }
        let s = mk::spacing::S;
        let mut y = s;
        if kind == MenuKind::Session {
            let head = mk::typo::HEADLINE;
            let name = fam::ellipsize(engine, &self.user, head.px(), head.weight, w - 4.0 * s);
            let (_, hh) = engine.measure_styled(&name, head.px(), head.weight);
            b.styled(engine, name, head, p.on_surface, 2.0 * s, y + (MENU_HEAD - hh) / 2.0, None, None);
            y += MENU_HEAD;
        }
        let symfam = matrixkit_scene::symbol_family();
        let gpx = mk::typo::symbol_px(mk::font_size::MEDIUM);
        let body = mk::typo::BODY;
        let pressed = self.press.as_ref().filter(|pr| pr.card == Some(Card::Menu(kind))).and_then(|pr| pr.deed);
        for row in self.menu_rows(kind) {
            let Some((g, title, deed, badge)) = row else {
                b.quad(2.0 * s, y + MENU_SEP / 2.0 - 0.5, w - 4.0 * s, 1.0, p.outline.with_alpha(0.25), 0.0, None, None);
                y += MENU_SEP;
                continue;
            };
            deeds.push(deed);
            let r = Some(deeds.len() - 1);
            let ground = if pressed == Some(deed) { p.on_surface.over(p.surface_container_high, mk::state_layer::PRESSED).with_alpha(0.6) } else { clear() };
            b.quad(s, y, w - 2.0 * s, MENU_ROW, ground, mk::CORNER_RADIUS, r, None);
            let (gw, gh) = engine.measure_family(symfam, &g.to_string(), gpx, mk::typo::Weight::Normal);
            b.text(engine, g.to_string(), gpx, mk::typo::Weight::Normal, symfam, p.on_surface, 2.0 * s, y + (MENU_ROW - gh) / 2.0, r, None);
            let tx = 2.0 * s + gw.max(20.0) + s;
            let badge_w = badge.as_ref().map(|t| engine.measure_styled(t, body.px(), body.weight).0 + s).unwrap_or(0.0);
            let shown = fam::ellipsize(engine, &title, body.px(), body.weight, w - tx - 2.0 * s - badge_w);
            let (_, th) = engine.measure_styled(&shown, body.px(), body.weight);
            b.styled(engine, shown, body, p.on_surface, tx, y + (MENU_ROW - th) / 2.0, r, None);
            if let Some(t) = badge {
                let (bw, bh) = engine.measure_styled(&t, body.px(), body.weight);
                b.styled(engine, t, body, p.on_surface_variant, w - 2.0 * s - bw, y + (MENU_ROW - bh) / 2.0, r, None);
            }
            y += MENU_ROW;
        }
    }

    // ------------------------------------------------------------ app card

    /// The card's rows at (x, y), `w` wide; returns the bottom y.
    #[allow(clippy::too_many_arguments)]
    fn lay_card_content(&self, card: &AppCard, b: &mut SceneBuilder, engine: &mut TextEngine, deeds: &mut Vec<Deed>, x: f32, y: f32, w: f32, clip: Option<Rect>) -> f32 {
        match card.kind {
            CardKind::User => return self.lay_user_content(card, b, engine, deeds, x, y, w, clip),
            CardKind::Central => return self.lay_central_content(card, b, engine, deeds, x, y, w, clip),
            CardKind::Apps => return self.lay_apps_content(card, b, engine, deeds, x, y, w, clip),
            CardKind::App => {}
        }
        let p = self.palette;
        let mut deed = |d: Deed| {
            deeds.push(d);
            deeds.len() - 1
        };
        let wide = card.wide && card.crawl.is_some();
        let col = if wide { (w - mk::spacing::L) / 2.0 } else { w };
        // Head: the app's name and a line about it.
        // A clear head across the whole card: name, a line about it, and a
        // round ✕ that closes the CARD (not the app) — 02.11.2026: «the card
        // could look better».
        let shut = deed(Deed::CardClose);
        let hy = fam::lay_card_head(b, engine, &card.title, &card.about, shut, x, y, w, clip, p);
        // The app's side.
        let (settings, swap, close) = (deed(Deed::Settings), deed(Deed::Swap), deed(Deed::CloseApp));
        let (switch, species): (usize, Vec<usize>) = (deed(Deed::CrawlSwitch), (0..mk::crawlers::SPECIES.len()).map(|i| deed(Deed::Species(i))).collect());
        let mut page = fam::FormPage::new(b, x, hy, col, clip, p);
        page.section(engine, "");
        if card.light {
            page.nav(engine, settings, Some(symbol::SETTINGS), "Einstellungen", None, None);
        }
        // Round 85: folded back, the other panel is dark (or off) — a window
        // sent there would be out of reach.
        if card.duo && mk::device::duo_front_panel().is_none() {
            let (label, hint) = if card.pair {
                ("Panels tauschen", "Duo — die Fenster liegen auf beiden Panels")
            } else {
                ("Aufs andere Panel", "Duo — das Fenster wechselt das Panel")
            };
            page.nav(engine, swap, Some(symbol::SWAP_VERT), label, Some(hint), None);
        }
        page.action(engine, close, Some(symbol::LOGOUT), "App beenden", Some("Schliesst alle Fenster dieser App"), p.error);
        // Wide: the Krabbler get a column of their own beside it.
        let (mut page, left_end) = if wide {
            let (end, _) = page.finish();
            (fam::FormPage::new(b, x + col + mk::spacing::L, hy, col, clip, p), end)
        } else {
            (page, 0.0)
        };
        if let Some(keys) = &card.crawl {
            // The Krabbler side (R211): the switch and the species.
            page.section(engine, "");
            page.toggle(engine, switch, "Krabbler", Some("Wortwesen laufen über diese App"), !keys.is_empty(), false);
            for (i, (key, name, diet)) in mk::crawlers::SPECIES.iter().enumerate() {
                let tint = mk::night_glass::NEON[i % mk::night_glass::NEON.len()];
                page.mark(engine, species[i], tint, name, Some(diet), keys.iter().any(|k| k == key));
            }
        }
        let (end, _) = page.finish();
        end.max(left_end)
    }

    /// THE USER CARD (round 31): who is signed in, the notifications
    /// (open by a tap, all away, do not disturb) and the session's deeds.
    /// Wide: notifications | session side by side.
    #[allow(clippy::too_many_arguments)]
    fn lay_user_content(&self, card: &AppCard, b: &mut SceneBuilder, engine: &mut TextEngine, deeds: &mut Vec<Deed>, x: f32, y: f32, w: f32, clip: Option<Rect>) -> f32 {
        let p = self.palette;
        let mut deed = |d: Deed| {
            deeds.push(d);
            deeds.len() - 1
        };
        let col = if card.wide { (w - mk::spacing::L) / 2.0 } else { w };
        let shut = deed(Deed::CardClose);
        let hy = fam::lay_card_head(b, engine, &self.user, &card.about, shut, x, y, w, clip, p);
        // Notifications.
        let dnd = Self::dnd();
        let (dnd_role, clear_role) = (deed(Deed::Dnd), deed(Deed::InboxClear));
        let items: Vec<(usize, &mk::inbox::Item)> = self.inbox.iter().take(30).map(|it| (deed(Deed::InboxOpen(it.id)), it)).collect();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let mut page = fam::FormPage::new(b, x, hy, col, clip, p);
        page.section(engine, "Mitteilungen");
        page.toggle(engine, dnd_role, "Nicht stören", Some("Mitteilungen kommen still in diese Liste"), dnd, false);
        if items.is_empty() {
            page.nav(engine, clear_role, Some(symbol::NOTIFICATIONS), "Keine Mitteilungen", Some("Alles gelesen"), None);
        } else {
            for (role, it) in &items {
                let age = if it.at > 0 { mk::time::relative(it.at as i64, now as i64) } else { String::new() };
                let text = mk::elide(&it.text.replace('\n', " "), 80);
                let desc = fam::dot_join(&[&it.app, &age, &text]);
                let title = if it.title.is_empty() { it.app.clone() } else { mk::elide(&it.title, 60) };
                page.nav(engine, *role, Some(symbol::NOTIFICATIONS), &title, Some(&desc), None);
            }
            page.action(engine, clear_role, Some(symbol::DELETE), "Alle löschen", None, p.error);
        }
        // The session.
        let (mut page, left_end) = if card.wide {
            let (end, _) = page.finish();
            (fam::FormPage::new(b, x + col + mk::spacing::L, hy, col, clip, p), end)
        } else {
            (page, 0.0)
        };
        page.section(engine, "Sitzung");
        for (g, title, d, badge) in self.menu_rows(MenuKind::Session).into_iter().flatten() {
            let role = deed(d);
            let ink = if matches!(d, Deed::Act(Action::Logout)) { p.error } else { p.on_surface };
            page.action(engine, role, Some(g), &title, badge.as_deref(), ink);
        }
        let (end, _) = page.finish();
        end.max(left_end)
    }

    /// Open (or close) the user card (round 31: the user chip; the bell
    /// is gone, its history lives here).
    /// Round 119: the open card wears glass — first the user card, then
    /// (owner: «now the rest of the cards too») every card, with glass on.
    fn card_glass(&self) -> bool {
        self.glass && self.card.is_some()
    }

    fn open_user_card(&mut self) -> Refresh {
        if self.card.as_ref().is_some_and(|c| c.kind == CardKind::User) {
            return self.close_card_soft();
        }
        let t0 = std::time::Instant::now();
        let r = self.open_user_card_now();
        mk::runtime::trail("matrix-takt-flaechen.log", &format!("[bar] Benutzerkarte vorbereitet in {:.0} ms", t0.elapsed().as_secs_f32() * 1000.0), 400);
        r
    }

    fn open_user_card_now(&mut self) -> Refresh {
        mkw::panel::close_other_popups("");
        self.menu = None;
        self.card = None;
        self.inbox = mk::inbox::read();
        let out = self.output.clone().unwrap_or_else(|| String::from("DSI-2"));
        let (ow, oh) = self.out_size(&out).unwrap_or((652.0, 489.0));
        let (x, y, w, h) = mk::device::card_frame((ow, oh), TOP + reserved_zone(), 0);
        let host = std::fs::read_to_string("/etc/hostname").map(|h| h.trim().to_string()).unwrap_or_default();
        self.card = Some(AppCard {
            window: mk::canvas::AppWindow { id: 0, pid: 0, output: out.clone() },
            app_id: String::new(),
            title: self.user.clone(),
            about: {
                // Round 75: the battery's time left beside who is signed in.
                let mut parts = vec![String::from("Angemeldet")];
                if !host.is_empty() {
                    parts.push(host.clone());
                }
                if let Some(b) = mk::power::battery() {
                    parts.push(format!("{} %", b.percent));
                    if let Some((h, charging)) = mk::power::hours_left() {
                        parts.push(mk::power::label(h, charging));
                    }
                }
                parts.join(" · ")
            },
            crawl: None,
            wide: w >= 560,
            light: false,
            pair: false,
            duo: false,
            place: CardPlace::At(out, x, y, w, h),
            kind: CardKind::User,
            roll: 0.0,
            content_h: 0.0,
            view_h: 0.0,
        });
        self.touch_chip_of(Widget::User);
        Refresh::Panes
    }

    /// Read the control state off the frame (tools: wpctl, nmcli …).
    fn controls_ask(&self) {
        if let Some(loom) = &self.loom {
            // Round 178: one reading at a time (a hung wpctl/nmcli no longer gains a thread every pulse).
            loom.spin_once("bar-controls", || Work::Controls(mk::quick::read()));
        }
    }

    /// THE CONTROL CARD (round 34, «without iced»): the control centre's
    /// controls in the person's order, a section per category, sliders
    /// for volume and brightness. Wide: two columns.
    #[allow(clippy::too_many_arguments)]
    fn lay_central_content(&self, card: &AppCard, b: &mut SceneBuilder, engine: &mut TextEngine, deeds: &mut Vec<Deed>, x: f32, y: f32, w: f32, clip: Option<Rect>) -> f32 {
        let p = self.palette;
        self.tracks.borrow_mut().clear();
        let col = if card.wide { (w - mk::spacing::L) / 2.0 } else { w };
        let hx = x + mk::spacing::M;
        deeds.push(Deed::CardClose);
        let hy = fam::lay_card_head(b, engine, "Kontrollzentrum", &card.about, deeds.len() - 1, x, y, w, clip, p);
        let Some(state) = &self.controls else {
            b.styled(engine, "Wird gelesen …", mk::typo::BODY, p.on_surface_variant, hx, hy, None, clip);
            return hy + 40.0;
        };
        let title_of = |c: matrixkit_scene::central::Category| match c {
            matrixkit_scene::central::Category::Suggest => "Vorschläge",
            matrixkit_scene::central::Category::Sound => "Klang",
            matrixkit_scene::central::Category::Radio => "Funk",
            matrixkit_scene::central::Category::Display => "Bildschirm",
            matrixkit_scene::central::Category::Access => "Bedienungshilfen",
            matrixkit_scene::central::Category::System => "System",
        };
        let controls: Vec<&'static matrixkit_scene::central::Control> = matrixkit_scene::central::load().iter().filter_map(|k| matrixkit_scene::central::find(k)).collect();
        // Consecutive controls of one category form a section.
        let mut groups: Vec<Vec<&'static matrixkit_scene::central::Control>> = Vec::new();
        for c in controls {
            match groups.last_mut() {
                Some(g) if g[0].category == c.category => g.push(c),
                _ => groups.push(vec![c]),
            }
        }
        // Wide: the first half of the sections left, the rest right.
        let split = if card.wide { groups.len().div_ceil(2) } else { groups.len() };
        let mut end = 0.0_f32;
        let mut page = fam::FormPage::new(b, x, hy, col, clip, p);
        for (gi, g) in groups.iter().enumerate() {
            if gi == split {
                let (e, sl) = page.finish();
                end = end.max(e);
                self.note_tracks(deeds, sl);
                page = fam::FormPage::new(b, x + col + mk::spacing::L, hy, col, clip, p);
            }
            page.section(engine, title_of(g[0].category));
            for c in g {
                match state.kind(c.key) {
                    mk::quick::Kind::Toggle(on) => {
                        deeds.push(Deed::Control(c.key));
                        page.toggle(engine, deeds.len() - 1, c.title, Some(c.subtitle), on, false);
                    }
                    mk::quick::Kind::Slider(v) => {
                        deeds.push(Deed::Slide(c.key));
                        page.slider(engine, deeds.len() - 1, c.title, None, v);
                    }
                    mk::quick::Kind::Action => {
                        deeds.push(Deed::Control(c.key));
                        page.action(engine, deeds.len() - 1, Some(c.glyph), c.title, Some(c.subtitle), p.on_surface);
                    }
                }
            }
        }
        let (e, sl) = page.finish();
        self.note_tracks(deeds, sl);
        end.max(e)
    }

    /// Remember the slider tracks of a laid page by their control key.
    fn note_tracks(&self, deeds: &[Deed], sliders: Vec<(usize, Rect)>) {
        let mut t = self.tracks.borrow_mut();
        for (role, rect) in sliders {
            if let Some(Deed::Slide(key)) = deeds.get(role) {
                t.push((key, rect));
            }
        }
    }

    /// THE APP CARD (round 35, «without iced»): every installed app in
    /// alphabetical sections; a tap starts it. Wide: two columns.
    #[allow(clippy::too_many_arguments)]
    fn lay_apps_content(&self, card: &AppCard, b: &mut SceneBuilder, engine: &mut TextEngine, deeds: &mut Vec<Deed>, x: f32, y: f32, w: f32, clip: Option<Rect>) -> f32 {
        let p = self.palette;
        let col = if card.wide { (w - mk::spacing::L) / 2.0 } else { w };
        deeds.push(Deed::CardClose);
        let hy = fam::lay_card_head(b, engine, &card.title, &card.about, deeds.len() - 1, x, y, w, clip, p);
        // Sections by first letter; wide: the second half of the list
        // starts the right column at a letter boundary.
        let letter = |e: &mk::apps::Entry| e.name.chars().next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default();
        let half = if card.wide { self.apps.len().div_ceil(2) } else { self.apps.len() };
        let mut end = 0.0_f32;
        let mut page = fam::FormPage::new(b, x, hy, col, clip, p);
        let mut current = String::new();
        let mut switched = !card.wide;
        for (i, e) in self.apps.iter().enumerate() {
            let l = letter(e);
            if l != current {
                if !switched && i >= half {
                    let (e2, _) = page.finish();
                    end = end.max(e2);
                    page = fam::FormPage::new(b, x + col + mk::spacing::L, hy, col, clip, p);
                    switched = true;
                }
                page.section(engine, &l);
                current = l;
            }
            deeds.push(Deed::Launch(i));
            let about = mk::apps::origin(&e.app_id);
            // Round 95: each app with its icon (the chosen theme, else Matrix).
            let icon = self
                .app_icons
                .borrow_mut()
                .entry(e.app_id.clone())
                .or_insert_with(|| matrixkit_icons::icon_file(&e.app_id, e.icon.as_deref(), &p, 96))
                .clone();
            page.nav_pict(engine, deeds.len() - 1, icon.as_deref(), &e.name, about);
        }
        let (e2, _) = page.finish();
        end.max(e2)
    }

    /// Open (or close) the app card on `out` (the asking dock's panel).
    fn open_apps_card(&mut self, out: Option<String>) -> Refresh {
        if self.card.as_ref().is_some_and(|c| c.kind == CardKind::Apps) {
            return self.close_card_soft();
        }
        mkw::panel::close_other_popups("");
        self.menu = None;
        if self.card.is_some() {
            self.close_card();
        }
        self.apps = mk::apps::index();
        let out = out.or_else(|| self.output.clone()).unwrap_or_else(|| String::from("DSI-1"));
        let (ow, oh) = self.out_size(&out).unwrap_or((652.0, 489.0));
        // Under the bar on its own panel; above the dock (the asker) on its.
        let top = if Some(&out) == self.output.as_ref() { TOP + reserved_zone() } else { 0 };
        let (x, y, w, h) = mk::device::card_frame((ow, oh), top, mk::device::dock_clearance() as i32 - mk::device::WINDOW_GAP as i32);
        let n = self.apps.len();
        self.card = Some(AppCard {
            window: mk::canvas::AppWindow { id: 0, pid: 0, output: out.clone() },
            app_id: String::new(),
            title: String::from(if self.picking { "App hinzufügen" } else { "Apps" }),
            about: if self.picking { format!("{n} Programme · antippen zum Anheften ans Dock") } else { format!("{n} Programme · antippen zum Starten") },
            crawl: None,
            wide: w >= 560,
            light: false,
            pair: false,
            duo: false,
            place: CardPlace::At(out, x, y, w, h),
            kind: CardKind::Apps,
            roll: 0.0,
            content_h: 0.0,
            view_h: 0.0,
        });
        Refresh::Panes
    }

    /// Open (or close) the control card (round 34: the control chip).
    fn open_central_card(&mut self) -> Refresh {
        if self.card.as_ref().is_some_and(|c| c.kind == CardKind::Central) {
            return self.close_card_soft();
        }
        mkw::panel::close_other_popups("");
        self.menu = None;
        if self.card.is_some() {
            self.close_card();
        }
        self.controls_ask();
        let out = self.output.clone().unwrap_or_else(|| String::from("DSI-2"));
        let (ow, oh) = self.out_size(&out).unwrap_or((652.0, 489.0));
        let (x, y, w, h) = mk::device::card_frame((ow, oh), TOP + reserved_zone(), 0);
        self.card = Some(AppCard {
            window: mk::canvas::AppWindow { id: 0, pid: 0, output: out.clone() },
            app_id: String::new(),
            title: String::from("Kontrollzentrum"),
            about: String::from("Ton, Funk, Bildschirm und Bedienungshilfen"),
            crawl: None,
            wide: w >= 560,
            light: false,
            pair: false,
            duo: false,
            place: CardPlace::At(out, x, y, w, h),
            kind: CardKind::Central,
            roll: 0.0,
            content_h: 0.0,
            view_h: 0.0,
        });
        self.touch_chip_of(Widget::Central);
        Refresh::Panes
    }

    /// A slider under the finger at card-local `x`: show the value at once,
    /// write it on release (`slide_commit`).
    fn slide_to(&mut self, key: &'static str, x: f64) -> Refresh {
        let track = self.tracks.borrow().iter().find(|(k, _)| *k == key).map(|(_, r)| *r);
        let Some(track) = track else { return Refresh::Calm };
        let t = fam::slider_value(track, x as f32);
        if let Some(st) = &mut self.controls {
            match key {
                "volume" => st.volume = st.volume.map(|(_, m)| (t, m)),
                "brightness" => st.brightness = st.brightness.map(|(_, src)| (t * 100.0, src)),
                _ => {}
            }
        }
        self.touch(Card::AppCard);
        Refresh::Panes
    }

    fn slide_commit(&mut self, key: &'static str) {
        let Some(st) = self.controls.clone() else { return };
        let Some(loom) = &self.loom else { return };
        loom.spin(move || {
            let mut st = st;
            match key {
                "volume" => {
                    let t = st.volume.map(|(v, _)| v).unwrap_or(0.0);
                    st.slide("volume", t);
                }
                "brightness" => {
                    if let Some((v, src)) = st.brightness {
                        mk::quick::brightness_commit(v, src);
                    }
                }
                _ => {}
            }
            Work::Controls(mk::quick::read())
        });
    }

    fn lay_card(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, deeds: &mut Vec<Deed>, w: f32, h: f32) {
        let Some(mut card) = self.card.clone() else { return };
        let p = self.palette;
        // Round 61: the finished card at rest — the growth is the house's
        // zoom of this one picture (`pane_zoom`), not a repaint per frame.
        let body = Rect { x: GROW_INSET, y: GROW_INSET, w: w - 2.0 * GROW_INSET, h: h - 2.0 * GROW_INSET };
        // Round 119: a glass card has no body of its own — niri's glass is it.
        if !self.card_glass() {
            b.quad(body.x, body.y, body.w, body.h, p.surface_container, 22.0, None, None);
        }
        let pad = mk::spacing::XL;
        let inner = (w - 2.0 * pad).max(120.0);
        // Measure once at the origin, then lay for real: centred when it
        // fits, scrolled inside the padding when it does not.
        let mut scratch = SceneBuilder::new();
        let mut sd = Vec::new();
        let content_h = self.lay_card_content(&card, &mut scratch, engine, &mut sd, pad, 0.0, inner, None);
        let avail = (h - 2.0 * pad).max(1.0);
        card.content_h = content_h;
        card.view_h = avail;
        card.roll = card.roll.clamp(0.0, (content_h - avail).max(0.0));
        let (top, clip) = if content_h <= avail {
            (pad + (avail - content_h) / 2.0, None)
        } else {
            (pad - card.roll, Some(Rect { x: 0.0, y: pad, w, h: avail }))
        };
        self.lay_card_content(&card, b, engine, deeds, pad, top, inner, clip);
        if clip.is_some() {
            // A thin scroll hint at the right edge.
            let portion = avail / content_h;
            let th = (avail * portion).max(24.0);
            let ty = pad + (avail - th) * (card.roll / (content_h - avail));
            b.quad(w - 6.0, ty, 3.0, th, p.on_surface_variant.with_alpha(0.4), 1.5, None, None);
        }
        if let Some(c) = &mut self.card {
            c.content_h = card.content_h;
            c.view_h = card.view_h;
            c.roll = card.roll;
        }
    }

    /// Round 62: a menu or the dialog pops out of where it comes from —
    /// the session menu out of the user chip (top right), the Matrix menu
    /// out of the Matrix chip (top left), the dialog out of the middle.
    fn pop_zoom(&mut self, card: &Card, w: f32, h: f32) -> Option<shell::Zoom> {
        if self.glass || mk::motion_reduced() {
            return None;
        }
        if self.popped_for.as_ref() != Some(card) {
            self.popped_for = Some(card.clone());
            self.pop = mk::motion::Spring::with(0.0, GROW_SECONDS * 0.8, 0.22);
            self.pop.retarget(1.0);
        }
        if self.pop.is_settled() {
            return None;
        }
        let t = self.pop.value().max(0.0);
        let s = 0.15 + 0.85 * t;
        let (ax, ay) = match card {
            Card::Menu(MenuKind::Session) => (1.0, 0.0),
            Card::Menu(MenuKind::Matrix) => (0.0, 0.0),
            _ => (0.5, 0.5),
        };
        Some(shell::Zoom { x: (w - w * s) * ax, y: (h - h * s) * ay, w: w * s, h: h * s })
    }

    /// Round 59b: where the card's chip sits, in the card pane's
    /// coordinates — the card grows out of it. The user and control chips
    /// are the last on the right of the bar, the app chip the first on the
    /// left; the dock's «Apps» tile is below the card.
    fn grow_origin(&self, card: &AppCard, w: f32, h: f32) -> Rect {
        let (cw, ch) = (w.min(120.0), h.min(40.0));
        let pane_x = match &card.place {
            CardPlace::At(_, x, _, _, _) => *x as f32,
            CardPlace::Centered(..) => 0.0,
        };
        let out_w = self.out_sizes.get(&card.window.output).copied().or_else(|| mk::canvas::output_size(&card.window.output)).map(|s| s.0).unwrap_or(w + pane_x);
        match card.kind {
            CardKind::User => Rect { x: (out_w - mk::spacing::S - cw - pane_x).clamp(0.0, w - cw), y: 0.0, w: cw, h: ch },
            CardKind::Central => Rect { x: (out_w - mk::spacing::S - 2.0 * cw - pane_x).clamp(0.0, w - cw), y: 0.0, w: cw, h: ch },
            CardKind::App => Rect { x: (mk::spacing::S - pane_x).clamp(0.0, w - cw), y: 0.0, w: cw, h: ch },
            // The dock's tile when it said where (else the middle below).
            CardKind::Apps => match self.apps_from {
                Some((tx, tw)) => Rect { x: (tx - pane_x).clamp(0.0, (w - tw.min(w)).max(0.0)), y: h - ch, w: tw.min(w), h: ch },
                None => Rect { x: (w - cw) / 2.0, y: h - ch, w: cw, h: ch },
            },
        }
    }

    /// Open (or close) the focused app's card (R210/R214).
    fn open_card(&mut self) -> Refresh {
        if self.card.as_ref().is_some_and(|c| c.kind == CardKind::App) {
            return self.close_card_soft();
        }
        if self.card.is_some() {
            self.close_card();
        }
        mkw::panel::close_other_popups("");
        self.menu = None;
        let Some((window, app_id)) = mk::canvas::focused_app_window() else { return Refresh::Calm };
        let windows = mk::canvas::app_windows(&app_id);
        let light = window.pid > 0 && mk::house::listens(window.pid);
        let duo = mk::device::panels().is_some();
        let place_words = match (duo, window.output.as_str()) {
            (true, "DSI-1") => String::from("oberes Panel"),
            (true, "DSI-2") => String::from("unteres Panel"),
            (_, o) => o.to_string(),
        };
        // The card has no text field: the on-screen keyboard goes first,
        // and the window grows back before it is measured (R214).
        if mk::runtime::path("matrix-keyboard-offen").exists() {
            mk::onscreen_keyboard::send("close");
            std::thread::sleep(std::time::Duration::from_millis(350));
        }
        let frame = mk::canvas::window_frame(window.id, Some(TOP + reserved_zone()));
        mk::runtime::write("matrix-card-frame", &format!("window {} on {} → {:?} (haus)", window.id, window.output, frame));
        let crawl = light.then(|| mk::crawlers::state(window.pid));
        let wide = frame.as_ref().is_some_and(|f| f.3 >= 560);
        let place = match frame {
            Some((out, x, y, w, h)) if w >= 200 && h >= 160 => CardPlace::At(out, x, y, w, h),
            _ => {
                // An estimate of the rows (the card scrolls when short).
                let mut h = 2.0 * mk::spacing::XL + 60.0 + 2.0 * 52.0 + if duo { 60.0 } else { 0.0 } + mk::spacing::L;
                if crawl.is_some() {
                    h += mk::spacing::L + 60.0 + 4.0 * 52.0;
                }
                CardPlace::Centered(window.output.clone(), (h as u32).min(640))
            }
        };
        self.card = Some(AppCard {
            about: format!("{} · {}", if light { "Matrix-App" } else { "App" }, place_words),
            title: self.focus.clone().unwrap_or_else(|| app_id.clone()),
            pair: mk::canvas::spans_panels(&windows),
            crawl,
            wide,
            light,
            duo,
            window,
            app_id,
            place,
            kind: CardKind::App,
            roll: 0.0,
            content_h: 0.0,
            view_h: 0.0,
        });
        self.touch_chip_of(Widget::Focus);
        Refresh::Panes
    }

    /// Round 63: close as the card came — it shrinks back into its chip
    /// (the toggle chip, ✕, the veil, Escape). Switching to another card
    /// closes at once (`close_card`).
    fn close_card_soft(&mut self) -> Refresh {
        if self.closing {
            return Refresh::Calm;
        }
        if self.card.is_none() || mk::motion_reduced() || self.grown_for.is_none() || self.card_glass() {
            return self.close_card();
        }
        self.closing = true;
        self.grow = mk::motion::Spring::with(self.grow.value().clamp(0.0, 1.0), GROW_SECONDS * 0.6, 0.0);
        self.grow.retarget(0.0);
        if !self.stale.contains(&Card::AppCard) {
            self.stale.push(Card::AppCard);
        }
        Refresh::Panes
    }

    fn close_card(&mut self) -> Refresh {
        self.closing = false;
        self.picking = false;
        self.grown_for = None;
        if let Some(c) = self.card.take() {
            self.touch_chip_of(match c.kind {
                CardKind::User => Widget::User,
                CardKind::Central => Widget::Central,
                CardKind::App | CardKind::Apps => Widget::Focus,
            });
            self.drag = None;
        }
        Refresh::Panes
    }

    // ------------------------------------------------------------ deeds

    fn touch(&mut self, card: Card) {
        // Round 117: what the card SHOWS changed (the glide's frames push
        // the card directly and leave this alone).
        if matches!(card, Card::AppCard | Card::Menu(_) | Card::Dialog) {
            self.card_rev = self.card_rev.wrapping_add(1);
        }
        if !self.stale.contains(&card) {
            self.stale.push(card);
        }
    }

    fn touch_chip_of(&mut self, w: Widget) {
        if self.chips {
            let in_center = self.zones.1.contains(&w);
            self.touch(Card::Chip(if in_center { Chip::Center } else { Chip::Widget(w) }));
        }
    }

    fn set_menu(&mut self, menu: Option<MenuKind>) -> Refresh {
        if menu.is_some() {
            mkw::panel::close_other_popups("");
        }
        self.menu = menu;
        self.touch_chip_of(Widget::User);
        self.touch_chip_of(Widget::Matrix);
        Refresh::Panes
    }

    fn act(&mut self, deed: Deed) -> Refresh {
        match deed {
            Deed::EditEnter => {
                self.menu = None;
                self.edit_enter(true)
            }
            Deed::EditAdd => self.set_menu(Some(MenuKind::Widgets)),
            Deed::AddWidget(w) => self.widget_add(w),
            Deed::EditDone => self.edit_leave(true),
            // Round 105: a tap on the island — the notification's own action
            // where it has one, else the list of notifications (user card).
            Deed::IslandOpen => {
                let Some(item) = self.island.as_ref().map(|i| i.item.clone()) else { return Refresh::Calm };
                let _ = self.island_end();
                if item.opens {
                    mk::inbox::send(mk::inbox::Command::Open(item.id));
                    return Refresh::Panes;
                }
                let _ = self.open_user_card();
                self.stale = self.plan();
                return Refresh::Panes;
            }
            Deed::Chip(w) => {
                // The dock tile's answer (R209): a short buzz and a lit edge.
                self.flash.tap(w);
                if !self.flashing.contains(&w) {
                    self.flashing.push(w);
                }
                self.touch_chip_of(w);
                match w {
                    Widget::Matrix => {
                        let open = self.menu == Some(MenuKind::Matrix);
                        return self.set_menu((!open).then_some(MenuKind::Matrix));
                    }
                    Widget::User => return self.open_user_card(),
                    Widget::Bell => {
                        mkw::panel::close_other_popups("matrix-notifications");
                        // Toggle signal to the notification daemon (history).
                        let _ = std::fs::write(mk::runtime::path("matrix-notifications-show"), "1");
                        self.menu = None;
                    }
                    // Round 34: the control centre is a card of the bar now.
                    Widget::Central => return self.open_central_card(),
                    Widget::Capture => {
                        mkw::panel::close_other_popups("matrix-capture");
                        mkw::panel::app_start_with("matrix-capture", &["panel"]);
                        self.menu = None;
                    }
                    Widget::Media => {
                        if let Some(np) = &self.now_playing {
                            media::play_pause(np.player.clone());
                        }
                    }
                    Widget::Focus => return self.open_card(),
                    _ => {}
                }
                Refresh::Panes
            }
            Deed::Tray(i) => {
                if let Some(it) = self.tray().get(i) {
                    let _ = self.tray_command.send(tray::TrayCommand::Activate(it.service.clone(), it.path.clone()));
                }
                Refresh::Calm
            }
            Deed::Close => self.set_menu(None),
            Deed::Act(a) if a.permanent() => {
                // R209: final things ask on glass.
                self.menu = None;
                // From the user card: the card gives way to the question.
                if self.card.as_ref().is_some_and(|c| c.kind != CardKind::App) {
                    self.close_card();
                }
                self.dialog = Some(a);
                self.touch_chip_of(Widget::User);
                self.touch_chip_of(Widget::Matrix);
                Refresh::Panes
            }
            Deed::Act(a) => {
                if self.card.as_ref().is_some_and(|c| c.kind != CardKind::App) {
                    self.close_card();
                }
                a.execute();
                self.set_menu(None)
            }
            Deed::App(id) => {
                // "name area" → write the wish file, then start (R41).
                let mut parts = id.splitn(2, ' ');
                let name = parts.next().unwrap_or(id);
                if let Some(area) = parts.next() {
                    let _ = std::fs::write(mk::runtime::path(&format!("{name}-area")), area);
                }
                mkw::panel::app_start(name);
                if self.card.as_ref().is_some_and(|c| c.kind != CardKind::App) {
                    self.close_card();
                }
                self.set_menu(None)
            }
            Deed::Answer(yes) => {
                if let Some(a) = self.dialog.take() {
                    if yes {
                        mk::feel::tap();
                        // The surfaces go first; the action may end the session.
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(120));
                            a.execute();
                        });
                    }
                }
                Refresh::Panes
            }
            Deed::Settings => {
                if let Some(card) = &self.card {
                    if card.light {
                        mk::house::send(card.window.pid, "root");
                    }
                }
                self.close_card()
            }
            Deed::Swap => {
                if let Some(card) = &self.card {
                    let (app_id, window, pair) = (card.app_id.clone(), card.window.clone(), card.pair);
                    std::thread::spawn(move || {
                        if pair {
                            mk::canvas::swap_panels(&app_id);
                        } else {
                            mk::canvas::window_to_other_panel(&window);
                        }
                    });
                }
                mk::feel::tap();
                self.close_card()
            }
            Deed::CloseApp => {
                if let Some(card) = &self.card {
                    mk::canvas::close_app(&card.app_id, 0);
                }
                mk::feel::tap();
                self.close_card()
            }
            Deed::CrawlSwitch | Deed::Species(_) => {
                // The card stays open: several species can be chosen.
                let Some(card) = &mut self.card else { return Refresh::Calm };
                let Some(keys) = &mut card.crawl else { return Refresh::Calm };
                match deed {
                    Deed::CrawlSwitch if !keys.is_empty() => keys.clear(),
                    Deed::CrawlSwitch => *keys = mk::crawlers::SPECIES.iter().map(|(k, _, _)| k.to_string()).collect(),
                    Deed::Species(i) => {
                        let key = mk::crawlers::SPECIES[i.min(mk::crawlers::SPECIES.len() - 1)].0.to_string();
                        if let Some(at) = keys.iter().position(|k| *k == key) {
                            keys.remove(at);
                        } else {
                            keys.push(key);
                        }
                    }
                    _ => {}
                }
                mk::crawlers::ask(card.window.pid, keys);
                mk::feel::tap();
                self.touch(Card::AppCard);
                Refresh::Panes
            }
            Deed::CardClose => self.close_card_soft(),
            Deed::InboxOpen(id) => {
                let opens = self.inbox.iter().any(|it| it.id == id && it.opens);
                mk::inbox::send(if opens { mk::inbox::Command::Open(id) } else { mk::inbox::Command::Dismiss(id) });
                self.inbox.retain(|it| it.id != id);
                mk::feel::tap();
                self.touch_chip_of(Widget::User);
                if opens {
                    return self.close_card();
                }
                self.touch(Card::AppCard);
                Refresh::Panes
            }
            Deed::InboxClear => {
                if !self.inbox.is_empty() {
                    mk::inbox::send(mk::inbox::Command::Clear);
                    self.inbox.clear();
                    mk::feel::tap();
                    self.touch_chip_of(Widget::User);
                    self.touch(Card::AppCard);
                }
                Refresh::Panes
            }
            Deed::Control(key) => {
                mk::feel::tap();
                // Round 64: the screen recording — start a moment after the
                // card has shrunk away (it would film itself), stop at once.
                if key == "recording" {
                    if mk::quick::recording() {
                        mk::process::sh_spawn("exec matrix-capture recording-stop");
                    } else {
                        mk::process::sh_spawn("sleep 0.9; exec matrix-capture recording");
                    }
                    return self.close_card_soft();
                }
                if key == "screenshot" {
                    mk::quick::act(key);
                    return self.close_card_soft();
                }
                let Some(st) = self.controls.clone() else { return Refresh::Calm };
                if st.kind(key) == mk::quick::Kind::Action {
                    mk::quick::act(key);
                    return self.close_card();
                }
                // Show the flip at once; the tool runs off the frame.
                let mut shown = st.clone();
                match key {
                    "motion" => shown.motion_reduced = !shown.motion_reduced,
                    "transparency" => shown.transparency_reduced = !shown.transparency_reduced,
                    "contrast" => shown.contrast_up = !shown.contrast_up,
                    "mute" => shown.volume = shown.volume.map(|(v, m)| (v, !m)),
                    "mic" => shown.mic_muted = shown.mic_muted.map(|m| !m),
                    "wlan" => shown.wlan = Some(!shown.wlan.unwrap_or(false)),
                    "bluetooth" => shown.bluetooth = Some(!shown.bluetooth.unwrap_or(false)),
                    "nightshift" => shown.night_shift = !shown.night_shift,
                    "autobright" => shown.auto_bright = !shown.auto_bright,
                    "rotlock" => shown.rotation_lock = !shown.rotation_lock,
                    "deepsleep" => shown.deep_sleep = !shown.deep_sleep,
                    _ => {}
                }
                self.controls = Some(shown);
                if let Some(loom) = &self.loom {
                    loom.spin(move || {
                        let mut st = st;
                        st.toggle(key);
                        Work::Controls(mk::quick::read())
                    });
                }
                self.touch(Card::AppCard);
                Refresh::Panes
            }
            Deed::Slide(_) => Refresh::Calm,
            // Round 113: picking for the dock — a tap pins (dock-pins; the
            // dock reloads it live) instead of starting.
            Deed::Launch(i) if self.picking => {
                if let Some(e) = self.apps.get(i) {
                    let mut pins: Vec<String> = mk::settings::read("dock-pins").unwrap_or_default().split_whitespace().map(String::from).collect();
                    if !pins.contains(&e.app_id) {
                        pins.push(e.app_id.clone());
                        mk::settings::write("dock-pins", &pins.join(" "));
                        eprintln!("[bar] App «{}» ans Dock geheftet", e.app_id);
                    }
                    mk::feel::select();
                }
                self.picking = false;
                self.close_card_soft()
            }
            Deed::Launch(i) => {
                if let Some(e) = self.apps.get(i) {
                    mk::apps::launch(e);
                    mk::feel::tap();
                }
                self.close_card()
            }
            Deed::Dnd => {
                let on = !Self::dnd();
                mk::settings::write("do-not-disturb", if on { "on" } else { "off" });
                mk::feel::tap();
                self.touch(Card::AppCard);
                Refresh::Panes
            }
        }
    }

    /// The second deed of a chip (right click / long press): a tray icon
    /// opens its own menu, the media chip skips to the next title.
    fn chip_second(&mut self, card: &Card, x: f64, y: f64) -> Refresh {
        self.press = None;
        match self.hit_card(card, x, y) {
            Some(Deed::Tray(i)) => {
                if let Some(it) = self.tray().get(i) {
                    let _ = self.tray_command.send(tray::TrayCommand::Secondary(it.service.clone(), it.path.clone()));
                    mk::feel::hold();
                }
                Refresh::Calm
            }
            Some(Deed::Chip(Widget::Media)) => {
                if let Some(np) = &self.now_playing {
                    crate::media::next(np.player.clone());
                    mk::feel::hold();
                }
                Refresh::Calm
            }
            _ => Refresh::Calm,
        }
    }

    fn hit_card(&self, card: &Card, x: f64, y: f64) -> Option<Deed> {
        // A single-widget chip is ONE target as a whole: its deed does not
        // depend on a (possibly stale) hit list. With the card open the
        // name chip's list was stale and the closing tap found nothing
        // (02.11.2026, tap log: «strike Chip(Focus) → None, card open»).
        // (The tray chip holds several icons: it keeps the hit list.)
        if matches!(card, Card::Chip(_)) && (self.morph > 0.0 || self.morph_dir != 0) {
            return None;
        }
        if let Card::Chip(Chip::Widget(w)) = card {
            if self.pressable(*w) {
                return Some(Deed::Chip(*w));
            }
        }
        let laid = self.laid.iter().find(|l| l.card == *card)?;
        let i = hit(&laid.placed, x as f32, y as f32)?;
        let r = laid.roles.get(i).copied().flatten()?;
        laid.deeds.get(r).copied()
    }

    /// Every surface of the plan, most stable first: the fleet keeps the
    /// common prefix, so chips that come and go (media, tray) and the cards
    /// stand at the end.
    fn plan(&self) -> Vec<Card> {
        let rank = |c: &Chip| match c {
            Chip::Widget(Widget::Battery | Widget::Clock) => 1,
            Chip::Center => 2,
            Chip::Widget(Widget::Focus) => 3,
            Chip::Widget(Widget::Media | Widget::Tray) => 4,
            Chip::Widget(_) => 0,
        };
        let mut cards = Vec::new();
        if self.chips {
            let mut chips: Vec<Chip> = self.row.iter().map(|(c, _, _, _)| *c).collect();
            chips.sort_by_key(|c| rank(c));
            cards.extend(chips.into_iter().map(Card::Chip));
        }
        if let Some(k) = self.menu {
            cards.push(Card::Backdrop);
            cards.push(Card::Menu(k));
        }
        if self.card.is_some() {
            cards.push(Card::CardVeil);
            cards.push(Card::AppCard);
        }
        if self.dialog.is_some() {
            cards.push(Card::DialogVeil);
            cards.push(Card::Dialog);
        }
        // Round 105: the island LAST — the chips keep their surfaces (the
        // fleet keeps the common prefix) and flow underneath it.
        if self.island_up() {
            cards.push(Card::Island);
        }
        // Round 113: the wiggle mode's row («+ Widget», «Fertig»).
        if self.editing && self.chips {
            cards.push(Card::EditBar);
        }
        cards
    }

    fn card_output(&self) -> Option<String> {
        self.card.as_ref().map(|c| match &c.place {
            CardPlace::At(o, ..) | CardPlace::Centered(o, _) => o.clone(),
        })
    }
}

/// Round 105: the icon of the app a notification came from — its desktop
/// entry by name or id, else the name itself as an icon id.
fn island_icon(app: &str, palette: &mk::Palette) -> Option<std::path::PathBuf> {
    matrixkit_icons::for_app(app, palette, 96)
}

/// A small, stable number for a name (signature of a surface's screen).
fn name_code(s: &str) -> u32 {
    s.bytes().fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32))
}

fn widget_code(w: Widget) -> u32 {
    match w {
        Widget::Matrix => 1,
        Widget::Tray => 2,
        Widget::Focus => 3,
        Widget::Clock => 4,
        Widget::Pulse => 5,
        Widget::Bell => 6,
        Widget::Battery => 7,
        Widget::Central => 8,
        Widget::User => 9,
        Widget::Capture => 10,
        Widget::Media => 11,
    }
}

impl shell::Client for Bar {
    type Card = Card;
    type Work = Work;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &Loom<Work>) {
        self.loom = Some(loom.clone());
        // Round 35: the bar's mailbox (the dock asks for the app card).
        if let Some(sock) = mk::bar::listen() {
            let feeder = loom.clone();
            std::thread::spawn(move || {
                let mut buf = [0u8; 256];
                while let Ok(n) = sock.recv(&mut buf) {
                    feeder.feed(Work::Ask(String::from_utf8_lossy(&buf[..n]).trim().to_string()));
                }
            });
        }
        self.backdrop_ask();
        // Focus changes arrive IMMEDIATELY via the compositor's event
        // stream (iced: mkw::canvas_stream) — read on a thread, fed home.
        let feeder = loom.clone();
        // Round 50: the kit's event follow (reconnect, kill AND reap).
        let mut last: Option<Option<String>> = None;
        mk::canvas::follow_events(false, move |line| {
            let Some(line) = line else { return true };
            if !(line.contains("indow") || line.contains("ocus")) {
                return true;
            }
            let now = focus_read();
            if last.as_ref() != Some(&now) {
                last = Some(now.clone());
                feeder.feed(Work::Focus(now));
            }
            true
        });
    }

    fn woven(&mut self, work: Work) -> Refresh {
        match work {
            Work::Backdrop(b) => {
                if b.is_none() || b == self.backdrop {
                    return Refresh::Calm;
                }
                eprintln!("[bar] Hintergrund hinter den Chips: {:.2}", b.as_ref().map(|b| b.1).unwrap_or(0.0));
                self.backdrop = b;
                // Every chip wears the new ink.
                self.stale = self.plan();
                Refresh::Panes
            }
            Work::Sizes(list) => {
                for (o, sz) in list {
                    self.out_sizes.insert(o, sz);
                }
                Refresh::Calm
            }
            Work::Ask(word) => {
                let mut it = word.split_whitespace();
                match (it.next(), it.next()) {
                    (Some(word @ ("apps" | "apps-pick")), out) => {
                        // Round 61: the dock names its tile («x w», logical).
                        let x = it.next().and_then(|v| v.parse::<f32>().ok());
                        let tw = it.next().and_then(|v| v.parse::<f32>().ok());
                        self.apps_from = x.zip(tw);
                        // Round 113: «apps-pick» — the dock's «+» in the wiggle mode.
                        self.picking = word == "apps-pick";
                        self.open_apps_card(out.map(String::from))
                    }
                    // Round 61: the user card on request (probes, automation).
                    (Some("user"), _) => self.open_user_card(),
                    // Round 118: the menus as well (toggle).
                    (Some("menu"), Some(kind)) => {
                        let k = match kind {
                            "matrix" => Some(MenuKind::Matrix),
                            "session" => Some(MenuKind::Session),
                            _ => None,
                        };
                        let next = if self.menu == k { None } else { k };
                        self.set_menu(next)
                    }
                    _ => Refresh::Calm,
                }
            }
            Work::Controls(st) => {
                // A slider under the finger keeps its own value.
                if self.drag.is_some() || self.controls.as_ref() == Some(&st) {
                    return Refresh::Calm;
                }
                self.controls = Some(st);
                if self.card.as_ref().is_some_and(|c| c.kind == CardKind::Central) {
                    self.touch(Card::AppCard);
                    return Refresh::Panes;
                }
                Refresh::Calm
            }
            Work::Focus(f) => {
                if f == self.focus {
                    return Refresh::Calm;
                }
                self.focus = f;
                self.touch_chip_of(Widget::Focus);
                Refresh::Panes
            }
        }
    }

    fn output(&self) -> Option<String> {
        self.output.clone()
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        self.lay_row(engine, w as f32);
        let mut b = SceneBuilder::new();
        self.main_deeds.clear();
        if self.chips {
            // The spine draws nothing — it only reserves the strip.
            return b.finish();
        }
        // The whole bar: one strip, the widgets laid in it.
        let p = self.palette;
        if !self.glass {
            b.quad(0.0, 0.0, w as f32, h as f32, p.surface_container, mk::CORNER_RADIUS + INSET, None, None);
        }
        let mut deeds = Vec::new();
        for (chip, side, off, cw) in self.row.clone() {
            let x = self.x_of(side, off, cw);
            match chip {
                Chip::Widget(wd) => {
                    self.lay_face(&mut b, engine, atlas, &mut deeds, wd, x, 0.0, cw as f32, h as f32);
                    if self.pressable(wd) {
                        let glow = if self.held_open(wd) { self.flash.glow(&wd).max(0.5) } else { self.flash.glow(&wd) };
                        let pressed = self.press.as_ref().is_some_and(|pr| pr.card.is_none() && pr.deed == Some(Deed::Chip(wd)));
                        fam::lay_chip_answer(&mut b, x + INSET, INSET, cw as f32 - 2.0 * INSET, h as f32 - 2.0 * INSET, mk::radius::SMALL, glow, pressed, p);
                    }
                }
                Chip::Center => self.lay_center(&mut b, engine, atlas, &mut deeds, x, 0.0, cw as f32, h as f32),
            }
        }
        self.main_deeds = deeds;
        b.finish()
    }

    fn strike(&mut self, role: Option<usize>, x: f64, y: f64, _source: Source) -> Refresh {
        let deed = role.and_then(|r| self.main_deeds.get(r).copied());
        self.press = Some(Press { card: None, deed, x, y, moved: false, roll0: 0.0, since: std::time::Instant::now(), edited: false });
        if deed.is_some() {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn release(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        let Some(pr) = self.press.take() else { return Refresh::Calm };
        let deed = role.and_then(|r| self.main_deeds.get(r).copied());
        match (pr.card, pr.deed) {
            (None, Some(d)) if deed == Some(d) => {
                let r = self.act(d);
                if r == Refresh::Calm {
                    Refresh::Dirty
                } else {
                    r
                }
            }
            _ => Refresh::Dirty,
        }
    }

    fn key(&mut self, key: Key, _shift: bool) -> Refresh {
        match key {
            Key::Escape if self.dialog.is_some() => self.act(Deed::Answer(false)),
            Key::Enter if self.dialog.is_some() => self.act(Deed::Answer(true)),
            Key::Escape if self.card.is_some() => self.close_card_soft(),
            Key::Escape if self.menu.is_some() => self.set_menu(None),
            _ => Refresh::Calm,
        }
    }

    fn beat(&mut self) -> Refresh {
        // Round 115: the screen film started / ended — the capture chip glows.
        let recording = mk::quick::recording();
        if recording != self.recording {
            self.recording = recording;
            self.touch_chip_of(Widget::Capture);
            if self.chips {
                return Refresh::Panes;
            }
        }
        // Round 113: the dock (or a menu) entered / left the wiggle mode.
        let editing = mk::edit::active();
        if editing != self.editing && self.press.is_none() && self.chips {
            return if editing { self.edit_enter(false) } else { self.edit_leave(false) };
        }
        // Round 105: the inbox on the beat (500 ms, one stat) — the island
        // follows a notification at once, not at the next pulse.
        if !mk::device::display_asleep() {
            let all = self.inbox_check();
            if self.island_try() {
                return Refresh::Panes;
            }
            if all {
                self.stale = self.plan();
                return Refresh::Panes;
            }
        }
        if self.clock_tick() {
            self.touch_chip_of(Widget::Clock);
            return Refresh::Panes;
        }
        Refresh::Calm
    }

    fn pulse(&mut self) -> Refresh {
        // Round 117: refresh the remembered output sizes off the loop (a
        // rotation shows within a few seconds; opening a card asks no one).
        self.sizes_tick = self.sizes_tick.wrapping_add(1);
        if self.sizes_tick % 3 == 0 && !mk::device::display_asleep() {
            if let Some(loom) = &self.loom {
                let mut names: Vec<String> = self.out_sizes.keys().cloned().collect();
                if let Some(o) = &self.output {
                    if !names.contains(o) {
                        names.push(o.clone());
                    }
                }
                loom.spin_once("bar-sizes", move || Work::Sizes(names.into_iter().filter_map(|n| mk::canvas::output_size(&n).map(|s| (n, s))).collect()));
            }
        }
        // Round 83: the duo folded to one panel (or opened again) — the bar
        // moves to the panel in front (its spine is placed at start: anew in
        // place, exec keeps the pid the watchdog knows).
        if mk::device::is_duo() {
            let want = mk::device::duo_panel("bar-output", BAR_PANEL_DUO);
            if self.output.as_deref() != Some(want.as_str()) {
                eprintln!("[bar] Panel wechselt nach «{want}» — Neustart dort");
                let e = mk::process::exec_on_output(None);
                eprintln!("[bar] Neustart misslungen: {e}");
            }
        }
        // Round 44: panels asleep (power key) — nobody sees the bar; skip
        // the readings (battery, network, controls) until they wake. Round
        // 132: the same behind the lock screen (the session is covered).
        if mk::device::display_asleep() || mk::session_lock::active() {
            return Refresh::Calm;
        }
        // A new wallpaper: measure the backdrop again (glass ink).
        if mk::wallpaper::current() != self.backdrop.as_ref().map(|b| b.0.clone()) {
            self.backdrop_ask();
        }
        let mut all = false;
        if self.watcher.changed() {
            self.glass = mk::glass_wanted();
            if let Some(p) = mk::Palette::load() {
                self.palette = p;
            }
            self.app_icons.borrow_mut().clear();
            all = true;
        }
        let theme = matrixkit_icons::look();
        if theme != self.icon_theme {
            self.icon_theme = theme;
            self.app_icons.borrow_mut().clear();
            all = true;
        }
        // Round 34: an open control card follows the device (2 s).
        if self.drag.is_none() && self.card.as_ref().is_some_and(|c| c.kind == CardKind::Central) {
            self.controls_ask();
        }
        if self.inbox_check() {
            all = true;
        }
        // Round 105: a waiting notification takes the bar when it is free.
        if self.island_try() {
            return Refresh::Panes;
        }
        let zones = widgets_read();
        if zones != self.zones {
            self.zones = zones;
            all = true;
        }
        let changed = self.measure();
        if all {
            self.stale = self.plan();
            return Refresh::Panes;
        }
        if changed.is_empty() {
            return Refresh::Calm;
        }
        for w in changed {
            self.touch_chip_of(w);
        }
        if self.chips {
            Refresh::Panes
        } else {
            Refresh::Dirty
        }
    }

    fn animating(&self) -> bool {
        let resting = self.press.as_ref().is_some_and(|p| !p.moved && !p.edited && matches!(p.card, Some(Card::Chip(_))));
        self.editing || resting || self.island.is_some() || self.morph_dir != 0 || self.flash.is_active() || !self.flashing.is_empty() || (self.card.is_some() && !self.grow.is_settled()) || (self.popped_for.is_some() && !self.pop.is_settled())
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        // Round 113: the second stage of a long press on a chip — a second
        // buzz and the wiggle mode (dock and bar together).
        let second = !self.editing
            && self.press.as_ref().is_some_and(|p| {
                !p.moved && !p.edited && matches!(p.card, Some(Card::Chip(_))) && p.since.elapsed().as_secs_f32() >= mk::edit::PRESS_SECS
            });
        if second {
            if let Some(p) = &mut self.press {
                p.edited = true;
                // the release must not tap
                p.deed = None;
            }
            return self.edit_enter(true);
        }
        // Round 113/116: the wiggle runs AFTER every other animation (a card
        // opened from the wiggle mode — the dock's «+» — stayed stuck at its
        // first zoom frame: this step returned early on every frame and the
        // card's spring never moved; only its veil showed, 04.11.2026).
        let r = self.glide_rest(dt);
        if self.editing {
            self.edit_t += dt;
            self.edit_acc += dt;
            if matches!(r, Refresh::Calm) && self.edit_acc >= 0.033 && !mk::motion_reduced() {
                self.edit_acc = 0.0;
                return Refresh::Panes;
            }
        }
        r
    }


    fn fleet(&mut self) -> Vec<shell::fleet::Spec<Card>> {
        // Round 62: a closed menu / dialog pops again when it comes back.
        let still_open = match &self.popped_for {
            Some(Card::Menu(k)) => self.menu == Some(*k),
            Some(Card::Dialog) => self.dialog.is_some(),
            _ => false,
        };
        if !still_open {
            self.popped_for = None;
        }
        let out = self.output.as_deref().map(name_code).unwrap_or(0);
        let card_out = self.card_output().as_deref().map(name_code).unwrap_or(0);
        self.plan()
            .into_iter()
            .map(|card| {
                let (namespace, sig) = match &card {
                    Card::Chip(Chip::Widget(w)) => ("matrix-chip", (1, widget_code(*w) ^ out)),
                    Card::Chip(Chip::Center) => ("matrix-chip", (2, out)),
                    Card::Backdrop => ("matrix-bar-backdrop", (3, out)),
                    Card::Menu(k) => ("matrix-card", (4, *k as u32)),
                    Card::CardVeil => ("matrix-veil", (5, card_out)),
                    // The owner 02.11.2026: the app card WITHOUT glass — its own
                    // namespace matches no glass rule, the card paints its body.
                    // Round 119 (owner: «make the tile with glass too»): the
                    // user card wears the cards' glass (and niri's pop).
                    Card::AppCard if self.card_glass() => ("matrix-card", (11, card_out)),
                    Card::AppCard => ("matrix-app-card", (6, card_out)),
                    Card::DialogVeil => ("matrix-veil", (7, out)),
                    Card::Dialog => ("matrix-card", (8, out)),
                    // Round 111: its own glass rule (like the chips, without
                    // niri's appearance pop — the island grows out of the chips,
                    // a pop made its content jump a few points).
                    Card::Island => ("matrix-island", (9, out)),
                    Card::EditBar => ("matrix-chip", (10, out)),
                };
                let (width, height) = self.pane_place(&card).map(|pl| pl.size).unwrap_or((1, 1));
                shell::fleet::Spec { pane: card, width, height, cross: 0, namespace, keyboard: false, sig }
            })
            .collect()
    }

    fn pane_place(&mut self, card: &Card) -> Option<PanePlace> {
        let bar_out = self.output.clone();
        let top = Anchors { top: true, right: false, bottom: false, left: false };
        Some(match card {
            Card::Chip(chip) if self.morph_rect(chip).is_some() => {
                // Round 105: flowing — every chip anchored left at its moment.
                let r = self.morph_rect(chip)?;
                PanePlace {
                    anchor: Anchors { left: true, ..top },
                    margin: (TOP + r.y.round() as i32, 0, 0, r.x.round() as i32),
                    size: ((r.w.round() as u32).max(1), (r.h.round() as u32).max(1)),
                    output: bar_out,
                    layer: None,
                    keyboard: KeyboardWish::None,
                    exclusive: -1,
                }
            }
            Card::Island => {
                let band = self.island_band();
                PanePlace {
                    anchor: Anchors { left: true, ..top },
                    margin: (TOP, 0, 0, band.x.round() as i32),
                    size: (band.w.round() as u32, HEIGHT),
                    output: bar_out,
                    layer: Some(Layer::Overlay),
                    keyboard: KeyboardWish::None,
                    exclusive: -1,
                }
            }
            Card::EditBar => PanePlace {
                anchor: top,
                margin: (TOP + HEIGHT as i32 + mk::spacing::S as i32, 0, 0, 0),
                size: (EDIT_BAR_W, HEIGHT),
                output: bar_out,
                layer: Some(Layer::Overlay),
                keyboard: KeyboardWish::None,
                exclusive: -1,
            },
            Card::Chip(chip) => {
                let (_, side, off, w) = *self.row.iter().find(|(c, ..)| c == chip)?;
                let anchor = Anchors { left: side == Side::Left, right: side == Side::Right, ..top };
                // Round 113: the wiggle mode moves chips by their margins
                // (nothing repainted); a dragged one rides the finger.
                let (mut dx, mut dy) = (0.0f32, 0.0f32);
                if self.editing {
                    match (&self.edit_drag, chip) {
                        (Some((d, off)), Chip::Widget(w)) if d == w => dx = *off as f32,
                        _ => {
                            let i = self.row.iter().position(|(c, ..)| c == chip).unwrap_or(0);
                            (dx, dy) = mk::edit::wiggle(self.edit_t, i);
                        }
                    }
                }
                let (dx, dy) = (dx.round() as i32, dy.round() as i32);
                PanePlace {
                    anchor,
                    margin: (TOP + dy, if side == Side::Right { off - dx } else { 0 }, 0, if side == Side::Left { off + dx } else { 0 }),
                    size: (w, HEIGHT),
                    output: bar_out,
                    layer: None,
                    keyboard: KeyboardWish::None,
                    exclusive: -1,
                }
            }
            Card::Backdrop => PanePlace { anchor: Anchors::all(), margin: (0, 0, 0, 0), size: (0, 0), output: bar_out, layer: None, keyboard: KeyboardWish::None, exclusive: -1 },
            Card::Menu(kind) => {
                let right = matches!(kind, MenuKind::Session | MenuKind::Widgets);
                let side = if self.chips { CHIP_SIDE } else { TOP };
                PanePlace {
                    anchor: Anchors { left: !right, right, ..top },
                    margin: (TOP + HEIGHT as i32 + mk::spacing::S as i32, if right { side } else { 0 }, 0, if right { 0 } else { side }),
                    size: (MENU_W, self.menu_height(*kind)),
                    output: bar_out,
                    layer: None,
                    keyboard: KeyboardWish::None,
                    exclusive: -1,
                }
            }
            // Not modal: the second tap on the name chip closes the card.
            Card::CardVeil => PanePlace::veil(self.card_output(), Some(Layer::Overlay), false),
            Card::AppCard => {
                let place = self.card.as_ref()?.place.clone();
                match place {
                    // R214: exactly over the window, margins from the output's
                    // edge (the bar's zone is ignored: −1).
                    CardPlace::At(out, x, y, w, h) => PanePlace {
                        anchor: Anchors { left: true, ..top },
                        margin: (y.max(0), 0, 0, x.max(0)),
                        size: (w.max(1), h.max(1)),
                        output: Some(out),
                        layer: Some(Layer::Overlay),
                        keyboard: KeyboardWish::None,
                        exclusive: -1,
                    },
                    CardPlace::Centered(out, h) => PanePlace {
                        anchor: Anchors { top: false, right: false, bottom: false, left: false },
                        margin: (0, 0, 0, 0),
                        size: (CARD_W, h),
                        output: Some(out),
                        layer: Some(Layer::Overlay),
                        keyboard: KeyboardWish::None,
                        exclusive: -1,
                    },
                }
            }
            Card::DialogVeil => PanePlace::veil(bar_out, Some(Layer::Overlay), true),
            Card::Dialog => PanePlace {
                anchor: Anchors { top: false, right: false, bottom: false, left: false },
                margin: (0, 0, 0, 0),
                size: (DIALOG_W, DIALOG_H),
                output: bar_out,
                layer: Some(Layer::Overlay),
                keyboard: KeyboardWish::None,
                exclusive: -1,
            },
        })
    }

    fn pane_scene(&mut self, card: &Card, engine: &mut TextEngine, atlas: &mut ImageAtlas, w: u32, h: u32) -> Option<Scene> {
        let (wf, hf) = (w as f32, h as f32);
        let p = self.palette;
        let mut b = SceneBuilder::new();
        let mut deeds: Vec<Deed> = Vec::new();
        match card {
            // Round 105: flowing chips are bare glass — their faces left at once.
            Card::Chip(_) if self.morph > 0.0 || self.morph_dir != 0 => {
                if !self.glass {
                    b.quad(0.0, 0.0, wf, hf, p.surface_container_high, mk::radius::LARGE, None, None);
                }
            }
            Card::Island => self.lay_island(&mut b, engine, atlas, &mut deeds, wf, hf),
            Card::EditBar => {
                if !self.glass {
                    b.quad(0.0, 0.0, wf, hf, p.surface_container_high, mk::radius::LARGE, None, None);
                }
                let (ink, _) = mk::glass::ink(&p, self.backdrop.as_ref().map(|b| b.1));
                let st = mk::typo::CALLOUT;
                let half = wf / 2.0;
                for (i, (label, deed)) in [("+ Widget", Deed::EditAdd), ("Fertig", Deed::EditDone)].into_iter().enumerate() {
                    deeds.push(deed);
                    let role = Some(deeds.len() - 1);
                    let x0 = i as f32 * half;
                    b.quad(x0, 0.0, half, hf, clear(), mk::radius::LARGE, role, None);
                    if i == 1 {
                        b.quad(x0 + 4.0, 4.0, half - 8.0, hf - 8.0, p.primary.with_alpha(0.85), mk::radius::LARGE - 4.0, role, None);
                    }
                    let (tw, th) = engine.measure_styled(label, st.px(), st.weight);
                    let tint = if i == 1 { p.on_primary } else { ink };
                    b.styled(engine, label.to_string(), st, tint, x0 + (half - tw) / 2.0, (hf - th) / 2.0, role, None);
                }
            }
            Card::Chip(chip) => {
                if !self.glass {
                    b.quad(0.0, 0.0, wf, hf, p.surface_container_high, mk::radius::LARGE, None, None);
                }
                match chip {
                    Chip::Widget(wd) => {
                        // Round 115: a chip that is busy glows inside its own
                        // edge (fam::lay_glow) — the screen film red, playing
                        // media in the accent.
                        let busy = match wd {
                            Widget::Capture if self.recording => Some(p.error),
                            Widget::Media if self.now_playing.as_ref().is_some_and(|n| n.playing) => Some(p.primary),
                            _ => None,
                        };
                        if let Some(tint) = busy {
                            let room = 5.0;
                            fam::lay_glow(&mut b, room, room, wf - 2.0 * room, hf - 2.0 * room, (mk::radius::LARGE - room).max(0.0), room, tint, 0.9, None);
                        }
                        self.lay_face(&mut b, engine, atlas, &mut deeds, *wd, 0.0, 0.0, wf, hf);
                        if self.pressable(*wd) {
                            let mut glow = self.flash.glow(wd);
                            if self.held_open(*wd) {
                                glow = glow.max(0.5);
                            }
                            fam::lay_chip_answer(&mut b, 0.0, 0.0, wf, hf, mk::radius::LARGE, glow, self.pressed_on(card), p);
                        }
                    }
                    Chip::Center => self.lay_center(&mut b, engine, atlas, &mut deeds, 0.0, 0.0, wf, hf),
                }
                // Round 113: the wiggle mode's «–» in the corner.
                if self.editing && matches!(chip, Chip::Widget(_)) {
                    let d = 16.0;
                    b.quad(1.0, 1.0, d, d, p.surface_container_high.with_alpha(0.95), d / 2.0, None, None);
                    b.quad(5.0, 1.0 + d / 2.0 - 1.0, d - 8.0, 2.0, p.on_surface, 1.0, None, None);
                }
            }
            Card::Backdrop => {
                deeds.push(Deed::Close);
                b.quad(0.0, 0.0, wf, hf, clear(), 0.0, Some(0), None);
            }
            Card::Menu(kind) => self.lay_menu(&mut b, engine, &mut deeds, *kind, wf, hf),
            Card::CardVeil | Card::DialogVeil => {
                deeds.push(if *card == Card::CardVeil { Deed::CardClose } else { Deed::Answer(false) });
                b.quad(0.0, 0.0, wf, hf, veil_tint(), 0.0, Some(0), None);
            }
            Card::AppCard => self.lay_card(&mut b, engine, &mut deeds, wf, hf),
            Card::Dialog => {
                if let Some(a) = self.dialog {
                    if !self.glass {
                        b.quad(0.0, 0.0, wf, hf, p.surface_container_high, 22.0, None, None);
                    }
                    let (title, note, action) = a.dialog();
                    deeds.push(Deed::Answer(false));
                    deeds.push(Deed::Answer(true));
                    let pad = mk::spacing::XL;
                    // Measure, then centre vertically.
                    let mut scratch = SceneBuilder::new();
                    let qh = fam::lay_question(&mut scratch, engine, pad, 0.0, wf - 2.0 * pad, title, note, action, 0, 1, p);
                    fam::lay_question(&mut b, engine, pad, ((hf - qh) / 2.0).max(pad / 2.0), wf - 2.0 * pad, title, note, action, 0, 1, p);
                }
            }
        }
        let scene = b.finish();
        self.laid.retain(|l| l.card != *card);
        self.laid.push(Laid { card: card.clone(), placed: scene.placed.clone(), roles: scene.roles.clone(), deeds });
        Some(scene)
    }

    /// Round 61: the card grows out of its chip as a ZOOM of its finished
    /// picture — the house paints it once and only moves/scales it.
    /// Round 117: the app card's content version — its finished zoom
    /// picture then stays (no repaint at the end of the growth, none to
    /// start the closing). Which card, its title and every content change.
    fn pane_version(&self, card: &Card) -> Option<u64> {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        match card {
            Card::AppCard => {
                let c = self.card.as_ref()?;
                format!("{:?}", c.kind).hash(&mut h);
                c.title.hash(&mut h);
            }
            // Round 118: menus and the dialog keep their picture as well.
            Card::Menu(k) => format!("{k:?}").hash(&mut h),
            Card::Dialog => format!("{:?}", self.dialog?).hash(&mut h),
            _ => return None,
        }
        self.card_rev.hash(&mut h);
        self.picking.hash(&mut h);
        Some(h.finish())
    }

    /// Round 117: the cards paint on the fleet GPU — build it ahead.
    fn warm_gpu(&self) -> bool {
        self.chips
    }

    /// Round 107: the island moves every frame — on the GPU.
    fn pane_gpu(&self, card: &Card) -> bool {
        // Round 118: menus and the dialog stay CPU-painted — on the GPU
        // niri's glass vanished beneath them (the island keeps its glass;
        // cause still open).
        *card == Card::Island
    }

    fn pane_zoom(&mut self, card: &Card, w: u32, h: u32) -> Option<shell::Zoom> {
        if matches!(card, Card::Menu(_) | Card::Dialog) {
            return self.pop_zoom(card, w as f32, h as f32);
        }
        if *card != Card::AppCard {
            return None;
        }
        // Round 119: a glass card does not zoom (niri draws the glass at the
        // surface's full size at once — a zoomed picture in it looks torn);
        // niri's own pop brings it in.
        if self.card_glass() {
            return None;
        }
        let open = self.card.clone()?;
        let (wf, hf) = (w as f32, h as f32);
        if self.grown_for != Some(open.kind) {
            self.grown_for = Some(open.kind);
            self.grow_from = self.grow_origin(&open, wf, hf);
            if mk::motion_reduced() {
                self.grow = mk::motion::Spring::with(1.0, GROW_SECONDS, 0.22);
            } else {
                // A spring with a little life: it overshoots and settles.
                self.grow = mk::motion::Spring::with(0.0, GROW_SECONDS, 0.22);
                self.grow.retarget(1.0);
            }
        }
        if self.grow.is_settled() && !self.closing {
            return None;
        }
        // The rectangle the finished picture (the whole pane) is shown in:
        // from the chip to the full pane, a little beyond at the overshoot.
        let r = grow_rect(self.grow_from, self.grow.value(), wf, hf);
        let rest = Rect { x: GROW_INSET, y: GROW_INSET, w: wf - 2.0 * GROW_INSET, h: hf - 2.0 * GROW_INSET };
        // One scale for both axes (no squashed picture), anchored on the
        // chip's side: the body keeps the chip's relative position.
        let s = (r.w / rest.w).min(r.h / rest.h);
        let from = self.grow_from;
        let ax = ((from.x + from.w / 2.0) / wf).clamp(0.0, 1.0);
        let ay = ((from.y + from.h / 2.0) / hf).clamp(0.0, 1.0);
        let bx = r.x + r.w * ax - rest.w * s * ax;
        let by = r.y + r.h * ay - rest.h * s * ay;
        Some(shell::Zoom { x: bx - GROW_INSET * s, y: by - GROW_INSET * s, w: wf * s, h: hf * s })
    }

    fn pane_stale(&mut self, card: &Card) -> bool {
        match self.stale.iter().position(|c| c == card) {
            Some(i) => {
                self.stale.remove(i);
                true
            }
            None => false,
        }
    }

    fn pane_strike(&mut self, card: &Card, x: f64, y: f64, _source: Source) -> Refresh {
        let deed = self.hit_card(card, x, y);
        if let (Card::AppCard, Some(Deed::Slide(key))) = (card, deed) {
            self.drag = Some(key);
            self.press = None;
            return self.slide_to(key, x);
        }
        let roll0 = self.card.as_ref().map_or(0.0, |c| c.roll);
        self.press = Some(Press { card: Some(card.clone()), deed, x, y, moved: false, roll0, since: std::time::Instant::now(), edited: false });
        if deed.is_some() && matches!(card, Card::Chip(_) | Card::Menu(_)) {
            self.touch(card.clone());
            return Refresh::Panes;
        }
        Refresh::Calm
    }

    /// Round 28: a right click (or a long press) on a chip — the tray
    /// icon's own menu, the media chip's «next» (missing in the house bar).
    fn pane_secondary(&mut self, card: &Card, x: f64, y: f64) -> Refresh {
        self.chip_second(card, x, y)
    }

    fn pane_hold(&mut self, card: &Card, x: f64, y: f64, _source: Source) -> Refresh {
        if self.editing {
            return Refresh::Calm;
        }
        // Round 113: the first stage answers with a buzz (the second, on
        // still holding, is the wiggle mode).
        if matches!(card, Card::Chip(_)) {
            mk::feel::hold();
        }
        self.chip_second(card, x, y)
    }

    /// Round 163: the mouse wheel rolls an open card (app list, control centre).
    fn pane_axis(&mut self, card: &Card, _x: f64, _y: f64, value: f64) -> Refresh {
        if *card != Card::AppCard {
            return Refresh::Calm;
        }
        let Some(c) = &mut self.card else { return Refresh::Calm };
        let max = (c.content_h - c.view_h).max(0.0);
        let roll = (c.roll + value as f32).clamp(0.0, max);
        if (roll - c.roll).abs() < 0.5 {
            return Refresh::Calm;
        }
        c.roll = roll;
        self.touch(Card::AppCard);
        Refresh::Panes
    }

    fn pane_travel(&mut self, card: &Card, x: f64, y: f64, _source: Source) -> Refresh {
        if let (Card::AppCard, Some(key)) = (card, self.drag) {
            return self.slide_to(key, x);
        }
        // Round 113: in the wiggle mode a chip follows the finger sideways.
        if self.editing {
            if let (Card::Chip(Chip::Widget(w)), Some(pr)) = (card, &mut self.press) {
                let dx = x - pr.x;
                if self.edit_drag.is_some() || dx.abs() > SLOP {
                    pr.moved = true;
                    self.edit_drag = Some((*w, dx));
                    return Refresh::Panes;
                }
            }
        }
        let Some(pr) = &mut self.press else { return Refresh::Calm };
        if pr.card.as_ref() != Some(card) {
            return Refresh::Calm;
        }
        if !pr.moved && ((y - pr.y).abs() > SLOP || (x - pr.x).abs() > SLOP) {
            pr.moved = true;
        }
        if *card == Card::AppCard && pr.moved {
            let roll = pr.roll0 - (y - pr.y) as f32;
            if let Some(c) = &mut self.card {
                let max = (c.content_h - c.view_h).max(0.0);
                let roll = roll.clamp(0.0, max);
                if (roll - c.roll).abs() >= 1.0 {
                    c.roll = roll;
                    self.touch(Card::AppCard);
                    return Refresh::Panes;
                }
            }
        }
        Refresh::Calm
    }

    fn pane_release(&mut self, card: &Card, x: f64, y: f64, _source: Source) -> Refresh {
        if let (Card::AppCard, Some(key)) = (card, self.drag) {
            let r = self.slide_to(key, x);
            self.drag = None;
            self.slide_commit(key);
            return r;
        }
        // Round 113: the wiggle mode — a drop, or the «–» corner removes.
        if self.editing && matches!(card, Card::Chip(_)) {
            let pr = self.press.take();
            if self.edit_drag.is_some() {
                return self.edit_drop();
            }
            if let (Card::Chip(Chip::Widget(w)), Some(pr)) = (card, pr) {
                if !pr.moved && x < 30.0 && pr.x < 30.0 {
                    return self.widget_remove(*w);
                }
            }
            return Refresh::Calm;
        }
        let Some(pr) = self.press.take() else { return Refresh::Calm };
        // Round 105: swiped up — the island goes (the notification stays in
        // the history).
        // Round 107: the island sits at the screen's top edge — the finger
        // leaves it after a few points, so the direction at release decides:
        // up = away, a still finger = a tap, anything else = nothing.
        if *card == Card::Island {
            let (dx, dy) = (x - pr.x, y - pr.y);
            eprintln!("[bar] Insel losgelassen: dx {dx:.0} dy {dy:.0} bewegt {}", pr.moved);
            if dy < -6.0 {
                return self.island_end();
            }
            if pr.moved || dx.abs() > SLOP || dy.abs() > SLOP {
                return Refresh::Calm;
            }
        }
        let lit = matches!(card, Card::Chip(_) | Card::Menu(_));
        if lit {
            self.touch(card.clone());
        }
        let calm = if lit { Refresh::Panes } else { Refresh::Calm };
        if pr.card.as_ref() != Some(card) || (pr.moved && *card == Card::AppCard) {
            return calm;
        }
        // The deed noted on press counts while the finger stayed on the same
        // surface and did not move: the press repaints the chip (pressed
        // state) and its hit list is rebuilt, so a hit test at release found
        // nothing — the name chip never opened the card (02.11.2026,
        // diagnosed with a runtime file). Elsewhere (menus, the card) the
        // release must still land on the same deed.
        match (pr.deed, self.hit_card(card, x, y)) {
            (Some(d), Some(now)) if d == now => self.act(d),
            (Some(d), _) if !pr.moved && matches!(card, Card::Chip(_)) => self.act(d),
            _ => calm,
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let chips = mk::device::bar_chips();
    // The watchdog's contract (owner rule): it starts us with
    // `--reservieren` and respawns us; we never spawn ourselves.
    let reserve = std::env::args().any(|a| a == "--reserve" || a == "--reservieren");
    eprintln!("[bar] Matrix Bar im Haus (R216), Chips: {chips}, Zone: {reserve}, pid={}", std::process::id());
    shell::run(
        Bar::new(),
        shell::Options {
            mount: shell::Mount::Layer {
                // As chips the spine matches no glass rule (R209).
                namespace: String::from(if chips { "matrix-bar-spine" } else { "matrix-bar" }),
                layer: Layer::Top,
                keyboard: KeyboardWish::None,
                anchor: Anchors { top: true, right: true, bottom: false, left: true },
                size: (0, if chips { 1 } else { HEIGHT }),
                margin: (TOP, if chips { 0 } else { TOP }, 0, if chips { 0 } else { TOP }),
                exclusive: if reserve { reserved_zone() } else { 0 },
            },
            argb: true,
            tag: "bar",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Across,
        },
    )
}

/// Round 59: the card body while it grows (`t` 0 → 1, a little beyond
/// while the spring overshoots) out of `from`, the chip's place in the
/// pane. At rest it keeps `GROW_INSET` from the pane's edge — the room the
/// overshoot swings into.
const GROW_INSET: f32 = 8.0;
/// How long the card takes to grow out of its chip (spring duration, s).
/// 0.42 felt a touch quick (owner, 03.11.2026) — 0.6.
const GROW_SECONDS: f32 = 0.6;

fn grow_rect(from: Rect, t: f32, w: f32, h: f32) -> Rect {
    let rest = Rect { x: GROW_INSET, y: GROW_INSET, w: w - 2.0 * GROW_INSET, h: h - 2.0 * GROW_INSET };
    let t = t.clamp(0.0, 1.0 + GROW_INSET / 40.0);
    let mix = |a: f32, b: f32| a + (b - a) * t;
    let r = Rect { x: mix(from.x, rest.x), y: mix(from.y, rest.y), w: mix(from.w, rest.w), h: mix(from.h, rest.h) };
    // Never outside the pane.
    let x = r.x.max(0.0);
    let y = r.y.max(0.0);
    Rect { x, y, w: (r.x + r.w).min(w) - x, h: (r.y + r.h).min(h) - y }
}

#[cfg(test)]
mod grow_tests {
    use super::*;

    #[test]
    fn the_card_grows_from_its_chip_and_rests_inset() {
        let chip = Rect { x: 250.0, y: 0.0, w: 120.0, h: 40.0 };
        let start = grow_rect(chip, 0.0, 400.0, 600.0);
        assert_eq!((start.x, start.y, start.w, start.h), (250.0, 0.0, 120.0, 40.0));
        let rest = grow_rect(chip, 1.0, 400.0, 600.0);
        assert_eq!((rest.x, rest.y, rest.w, rest.h), (GROW_INSET, GROW_INSET, 400.0 - 2.0 * GROW_INSET, 600.0 - 2.0 * GROW_INSET));
        // The overshoot swings beyond the rest, but never out of the pane.
        let over = grow_rect(chip, 1.2, 400.0, 600.0);
        assert!(over.w > rest.w && over.x >= 0.0 && over.x + over.w <= 400.0);
    }

}
