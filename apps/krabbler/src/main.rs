//! Matrix Krabbler (R211, the owner 02.11.2026): word creatures crawl over a
//! page of text and gather the words they like. They are made of the page
//! itself — every word they pick up rides on their back until it gets too
//! heavy and flies home to its place.
//!
//! Four species, each with its own appetite:
//! * **Langleser** — a centipede, twelve joints; eats nouns (German writes
//!   them with a capital).
//! * **Weberin** — a spider, eight long legs; only long words.
//! * **Füller** — a small beetle; the little words nobody notices.
//! * **Funke** — a firefly without legs; verbs ending in «-en».
//!
//! A tap on an empty spot lures them all there; a tap on a creature makes
//! it drop its load and flee. Untouched for two minutes they fall asleep
//! (no frames, no battery). The mechanics are the Kit's `crawl` family
//! (spine, two-bone legs, gait), the legs are `fam::lay_dotted_line`.
//!
//! `matrix-krabbler <file>` puts any text file under their feet.

use matrixkit_image::ImageAtlas;
use matrixkit_scene::crawl::{self, Critter, Dice, V2};
use matrixkit_scene::scene as fam;
use matrixkit_shell::{self as shell, Refresh, Scene, SceneBuilder, ScenePart, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{Keep, Rect};
use std::collections::VecDeque;

const APP_ID: &str = "matrix-krabbler";
const WIDTH: u32 = 620;
const HEIGHT: u32 = 820;
const CHROME_BASE: usize = 500;
const ROOT_BASE: usize = 700;
/// The page answers to taps (lure, scare).
const PAGE: usize = 1;
/// Untouched this long, the creatures fall asleep.
const SLEEP_AFTER_S: f32 = 120.0;
/// A word that flew home rests before anyone may take it again.
const WORD_REST_S: f32 = 8.0;
const FLY_S: f32 = 0.9;

/// The night page: dark whatever the palette says (neon needs the dark).
const NIGHT: mk::Rgba = mk::night_glass::GROUND;
const INK: mk::Rgba = mk::night_glass::INK;

/// The page's own text: notes about its inhabitants.
const PAGE_TEXT: &str = "# Feldnotizen von der Leinwand

Auf dieser Seite leben Wortwesen. Sie bestehen aus nichts anderem als aus dem Text, über den sie laufen. Jedes Wort, das sie finden, tragen sie auf dem Rücken mit, bis es ihnen zu schwer wird. Dann fliegt es an seinen Platz zurück, und die Seite liest sich wieder ganz.

Der Langleser ist ein Tausendfüssler mit zwölf Gliedern. Er sucht Nomen, also Wörter mit einem grossen Anfang: Haus, Licht, Morgen, Leinwand, Kaffee, Bergwiese. Er läuft ruhig und gründlich, Zeile um Zeile, und vergisst dabei nie, wo er schon gewesen ist.

Die Weberin ist eine Spinne mit acht langen Beinen. Sie interessiert sich nur für lange Wörter, jene Zusammensetzungen, die das Deutsche so gerne baut: Bildschirmhelligkeit, Haltungsdienst, Wiedergabeliste, Untertitelgrösse, Kopfhörerverbindung. Wenige trägt sie, aber dafür grosse.

Der Füller ist ein kleiner Käfer. Er frisst die kurzen Wörter, die niemand beachtet: und, der, mit, im, so, es, an. Ohne ihn fällt die Seite kaum auf. Mit ihm wirkt sie plötzlich löchrig, und man merkt, wie viel die kleinen Wörter tragen.

Der Funke hat keine Beine. Er fliegt in Bögen über die Zeilen und sammelt Tätigkeiten: laufen, tragen, finden, schauen, leuchten, schlafen, warten, suchen, singen. Wo er vorbeizieht, bleiben für einen Moment Lücken zwischen den Dingen.

Tippe auf eine leere Stelle, und alle Wesen kommen neugierig dorthin. Tippe auf ein Wesen, und es lässt seine Beute fallen und flieht. Wer zwei Minuten lang nichts tut, findet sie schlafend, dann ruht auch der Akku.

Am Abend, wenn die Panels dunkler werden, laufen sie langsamer. Nachts sammeln sie kaum noch etwas. Am Morgen aber beginnt die Suche von vorn, und jede Seite wird wieder zu einer Landschaft aus Wörtern, Wegen und Verstecken.";

/// The species live in the Kit (`crawl::Species`) — the same animals walk
/// over any house app (`fam::Crawlers`).
type Kind = crawl::Species;
const KINDS: [Kind; 4] = crawl::SPECIES;

#[derive(Clone, Copy, PartialEq, Debug)]
enum WordState {
    Home,
    /// A creature walks towards it.
    Wanted(usize),
    Carried(usize),
    /// On its way home.
    Flying { from: V2, t: f32 },
}

struct Word {
    text: String,
    clean: String,
    title: bool,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    state: WordState,
    /// Clock seconds before which nobody may take it again.
    rest_until: f32,
}

impl Word {
    fn centre(&self) -> V2 {
        V2::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Goal {
    Wander(V2),
    Word(usize),
}

struct Creature {
    /// Spine, legs, heading, speed — the Kit's body (`crawl::Critter`).
    body: Critter,
    goal: Goal,
    /// When the wander goal is given up for a new one.
    goal_until: f32,
    cargo: VecDeque<usize>,
    flee_until: f32,
}

impl Creature {
    fn new(kind: Kind, at: V2, heading: f32) -> Creature {
        Creature { body: Critter::new(kind, at, heading), goal: Goal::Wander(at), goal_until: 0.0, cargo: VecDeque::new(), flee_until: 0.0 }
    }

    fn kind(&self) -> Kind {
        self.body.species
    }

    fn head(&self) -> V2 {
        self.body.head()
    }
}

struct Krabbler {
    palette: mk::Palette,
    chrome: fam::Chrome,
    root: fam::RootPane,
    paragraphs: Vec<(bool, String)>,
    words: Vec<Word>,
    laid_width: f32,
    page_h: f32,
    /// The page's place in the window (below the chrome) and its scroll.
    area: Rect,
    roll: f32,
    roll_max: f32,
    creatures: Vec<Creature>,
    dice: Dice,
    clock: f32,
    doze: mk::Doze,
    lure: Option<(V2, f32)>,
    press: Option<mk::touch::Press<()>>,
    /// The words' scene part (rebuilt when a word changes or the page moves).
    page_cache: Option<(u64, Vec<Option<ScenePart>>)>,
    rev: u64,
    meter: shell::FrameMeter,
    frames: u64,
    layout_rev: u64,
}

impl Krabbler {
    fn new(text: &str) -> Krabbler {
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7);
        Krabbler {
            palette: mk::Palette::load().unwrap_or_default(),
            chrome: fam::Chrome::new("Matrix Krabbler", CHROME_BASE),
            root: fam::RootPane::new(APP_ID, &[]),
            paragraphs: fam::paragraphs(text),
            words: Vec::new(),
            laid_width: 0.0,
            page_h: 0.0,
            area: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            roll: 0.0,
            roll_max: 0.0,
            creatures: Vec::new(),
            dice: Dice::new(seed),
            clock: 0.0,
            doze: mk::Doze::new(SLEEP_AFTER_S),
            lure: None,
            press: None,
            page_cache: None,
            rev: 0,
            meter: shell::FrameMeter::new("krabbler"),
            frames: 0,
            layout_rev: 0,
        }
    }

    /// Lay the words out for a width (their states stay — same order).
    fn lay_words(&mut self, engine: &mut TextEngine, width: f32) {
        let (boxes, y) = fam::flow_words(engine, &self.paragraphs, width, mk::spacing::L, mk::typo::TITLE, mk::typo::BODY, 1.6);
        let mut fresh: Vec<Word> = boxes
            .into_iter()
            .map(|b| Word { text: b.text, clean: b.clean, title: b.heading, x: b.x, y: b.y, w: b.w, h: b.h, state: WordState::Home, rest_until: 0.0 })
            .collect();
        if fresh.len() == self.words.len() {
            for (new, old) in fresh.iter_mut().zip(&self.words) {
                new.state = old.state;
                new.rest_until = old.rest_until;
            }
        }
        self.words = fresh;
        // The legend below the text.
        self.page_h = y + 4.0 * 34.0 + mk::spacing::L;
        self.laid_width = width;
        self.rev += 1;
        self.layout_rev += 1;
    }

    /// Where creatures may walk (page coordinates).
    fn bounds(&self) -> (V2, V2) {
        let h = self.page_h.max(self.area.h);
        (V2::new(10.0, 10.0), V2::new((self.laid_width - 10.0).max(20.0), (h - 10.0).max(20.0)))
    }

    fn hatch(&mut self) {
        let (lo, hi) = self.bounds();
        // Two centipedes, a spider, two beetles, two fireflies.
        for kind in [Kind::Langleser, Kind::Weberin, Kind::Filler, Kind::Funke, Kind::Langleser, Kind::Filler, Kind::Funke] {
            let at = V2::new(self.dice.range(lo.x + 40.0, hi.x - 40.0), self.dice.range(lo.y + 40.0, (hi.y - 40.0).min(self.area.h.max(200.0))));
            let heading = self.dice.range(-3.1, 3.1);
            self.creatures.push(Creature::new(kind, at, heading));
        }
    }

    fn wander_point(&mut self) -> V2 {
        let (lo, hi) = self.bounds();
        // Mostly where the eye is (the visible part of the page).
        let top = (self.roll - 40.0).max(lo.y);
        let bottom = (self.roll + self.area.h + 40.0).min(hi.y).max(top + 20.0);
        V2::new(self.dice.range(lo.x + 20.0, hi.x - 20.0), self.dice.range(top, bottom))
    }

    /// The nearest word this creature likes and may take.
    fn choose_word(&self, c: usize) -> Option<usize> {
        let kind = self.creatures[c].kind();
        let head = self.creatures[c].head();
        self.words
            .iter()
            .enumerate()
            .filter(|(_, w)| w.state == WordState::Home && self.clock >= w.rest_until && !w.title && kind.likes(&w.clean))
            .map(|(i, w)| (i, w.centre().dist(head)))
            .filter(|(_, d)| *d < 420.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    /// A carried word flies home from where it rode.
    fn let_go(&mut self, word: usize, from: V2) {
        if let Some(w) = self.words.get_mut(word) {
            w.state = WordState::Flying { from, t: 0.0 };
            self.rev += 1;
        }
    }

    /// One step of the world.
    fn step(&mut self, dt: f32) {
        self.clock += dt;
        if self.doze.tick(dt) {
            eprintln!("[krabbler] sie schlafen ein");
        }
        let lure = self.lure.filter(|(_, until)| self.clock < *until).map(|(p, _)| p);
        let (lo, hi) = self.bounds();
        for c in 0..self.creatures.len() {
            // A new aim when the old one is gone.
            let goal = self.creatures[c].goal;
            let need_new = match goal {
                Goal::Word(i) => self.words.get(i).map(|w| w.state) != Some(WordState::Wanted(c)),
                Goal::Wander(p) => self.clock > self.creatures[c].goal_until || self.creatures[c].head().dist(p) < 12.0,
            };
            if need_new && self.clock > self.creatures[c].flee_until {
                let next = match self.choose_word(c) {
                    Some(i) => {
                        self.words[i].state = WordState::Wanted(c);
                        self.rev += 1;
                        Goal::Word(i)
                    }
                    None => Goal::Wander(self.wander_point()),
                };
                let until = self.clock + self.dice.range(3.0, 7.0);
                let cr = &mut self.creatures[c];
                cr.goal = next;
                cr.goal_until = until;
            }
            let target = match (lure, self.creatures[c].goal) {
                (Some(p), _) if self.clock > self.creatures[c].flee_until => p,
                (_, Goal::Word(i)) => self.words[i].centre(),
                (_, Goal::Wander(p)) => p,
            };
            let cr = &mut self.creatures[c];
            let (pace, _) = cr.kind().pace();
            let fleeing = self.clock < cr.flee_until;
            let dist = cr.head().dist(target);
            let want = if self.doze.asleep() {
                0.0
            } else if fleeing {
                pace * 2.4
            } else if lure.is_some() && dist < 40.0 {
                0.0
            } else {
                pace * (dist / 30.0).clamp(0.35, 1.0)
            };
            // The firefly sways on its way.
            let sway = if cr.kind() == Kind::Funke { (self.clock * 3.0 + c as f32).sin() * 0.9 } else { 0.0 };
            if cr.body.steer(target, want, dt, sway, if fleeing { 2.0 } else { 1.0 }, lo, hi) {
                cr.goal_until = 0.0;
            }
            let head = cr.head();
            // A word reached: on its back.
            if let Goal::Word(i) = cr.goal {
                if !fleeing && cr.head().dist(self.words[i].centre()) < 10.0 && self.words[i].state == WordState::Wanted(c) {
                    self.words[i].state = WordState::Carried(c);
                    cr.cargo.push_back(i);
                    cr.goal = Goal::Wander(head);
                    cr.goal_until = 0.0;
                    self.rev += 1;
                    if cr.cargo.len() > cr.kind().cargo() {
                        let from = cr.body.slot(0);
                        if let Some(old) = cr.cargo.pop_front() {
                            // Too heavy: the oldest flies home (fields, not
                            // `let_go` — the creature is still borrowed).
                            self.words[old].state = WordState::Flying { from, t: 0.0 };
                        }
                    }
                }
            }
        }
        // Words in flight come home.
        let clock = self.clock;
        for w in &mut self.words {
            if let WordState::Flying { from, t } = w.state {
                let t = t + dt / FLY_S;
                if t >= 1.0 {
                    w.state = WordState::Home;
                    w.rest_until = clock + WORD_REST_S;
                    self.rev += 1;
                } else {
                    w.state = WordState::Flying { from, t };
                }
            }
        }
    }

    /// A finger tapped the page at `p` (page coordinates).
    fn tap(&mut self, p: V2) {
        if let Some(c) = self.creatures.iter().position(|c| c.body.hit(p)) {
            eprintln!("[krabbler] {} flieht", self.creatures[c].kind().name());
            let cargo: Vec<(usize, V2)> = self.creatures[c].cargo.iter().enumerate().map(|(k, &w)| (w, self.creatures[c].body.slot(k))).collect();
            for (w, from) in cargo {
                self.let_go(w, from);
            }
            let away = self.creatures[c].head().add(self.creatures[c].head().sub(p).unit().mul(260.0));
            let cr = &mut self.creatures[c];
            cr.cargo.clear();
            cr.flee_until = self.clock + 1.6;
            cr.goal = Goal::Wander(away);
            cr.goal_until = self.clock + 1.6;
            cr.body.heading = away.sub(cr.head()).angle();
            mk::feel::tap();
            return;
        }
        // Lured: everyone forgets its word and comes to look.
        for w in &mut self.words {
            if let WordState::Wanted(_) = w.state {
                w.state = WordState::Home;
            }
        }
        for c in &mut self.creatures {
            c.goal = Goal::Wander(p);
            c.goal_until = self.clock + 4.0;
        }
        self.lure = Some((p, self.clock + 4.0));
        self.rev += 1;
    }

    fn wake(&mut self) {
        if self.doze.touch() {
            eprintln!("[krabbler] geweckt");
        }
    }

    /// Is anything still moving?
    fn moving(&self) -> bool {
        !self.doze.asleep()
            || self.creatures.iter().any(|c| c.body.moving())
            || self.words.iter().any(|w| matches!(w.state, WordState::Flying { .. }))
    }

    /// The words (and the legend) as a part, page coordinates shifted by the roll.
    /// Every visible word as its own small part — rebuilt only when the
    /// page scrolls or resizes (rebuilding all at every grab cost up to
    /// 40 ms a frame on the duo); a frame appends the words that are home.
    /// (A night rectangle over a missing word did not hide it: the house
    /// draws text above later rectangles.)
    fn build_page(&self, engine: &mut TextEngine, off: V2) -> Vec<Option<ScenePart>> {
        let clip = Some(self.area);
        self.words
            .iter()
            .map(|w| {
                let (x, y) = (w.x + off.x, w.y + off.y);
                if y + w.h < self.area.y || y > self.area.y + self.area.h {
                    return None;
                }
                let mut b = SceneBuilder::new();
                let style = if w.title { mk::typo::TITLE } else { mk::typo::BODY };
                b.styled(engine, w.text.clone(), style, if w.title { mk::Rgba { a: 0.95, ..INK } } else { INK }, x, y, None, clip);
                Some(b.into_part())
            })
            .collect()
    }

    /// What changes on the page, laid over it each frame: the gap where a
    /// word is missing, the outline of a wanted word, the legend's counts.
    fn build_marks(&self, engine: &mut TextEngine, b: &mut SceneBuilder, off: V2) {
        let clip = Some(self.area);
        for w in &self.words {
            if w.state == WordState::Home {
                continue;
            }
            let (x, y) = (w.x + off.x, w.y + off.y);
            if y + w.h < self.area.y || y > self.area.y + self.area.h {
                continue;
            }
            let style = if w.title { mk::typo::TITLE } else { mk::typo::BODY };
            match w.state {
                WordState::Wanted(c) => {
                    let col = fam::species_color(self.creatures[c].kind());
                    fam::lay_tag(b, engine, &w.text, style, x + w.w / 2.0, y + w.h / 2.0, col, NIGHT, clip);
                }
                // The word is away: a faint gap remains.
                _ => {
                    b.quad(x, y + w.h * 0.25, w.w, w.h * 0.5, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.05 }, 3.0, None, clip);
                }
            }
        }
        // The legend: who eats what, and how much it carries now.
        let mut y = self.page_h - 4.0 * 34.0 - mk::spacing::S + off.y;
        if y > self.area.y + self.area.h {
            return;
        }
        let cap = mk::typo::CAPTION;
        for kind in KINDS {
            let carried: usize = self.creatures.iter().filter(|c| c.kind() == kind).map(|c| c.cargo.len()).sum();
            let line = format!("{} · {} · trägt {}", kind.name(), kind.diet(), carried);
            let (tw, th) = engine.measure_styled(&line, cap.px(), cap.weight);
            let x = mk::spacing::L + off.x;
            b.quad(x, y, tw + 40.0, 28.0, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.05 }, 14.0, None, clip);
            fam::lay_dot(b, x + 15.0, y + 14.0, 9.0, fam::species_color(kind), clip);
            b.styled(engine, line, cap, INK, x + 28.0, y + (28.0 - th) / 2.0, None, clip);
            y += 34.0;
        }
    }

    /// The creatures, on top of the page.
    fn build_creatures(&self, engine: &mut TextEngine, b: &mut SceneBuilder, off: V2) {
        let clip = Some(self.area);
        let dim = if self.doze.asleep() { 0.45 } else { 1.0 };
        // First the loads of all of them — threads and words — so no word
        // ever lies over a creature (Kit: `fam::lay_critter_load`/`lay_critter`).
        for c in &self.creatures {
            let words: Vec<String> = c.cargo.iter().filter_map(|&wi| self.words.get(wi).map(|w| w.clean.clone())).collect();
            fam::lay_critter_load(b, engine, &c.body, &words, off, dim, NIGHT, clip);
        }
        for c in &self.creatures {
            fam::lay_critter(b, &c.body, off, dim, NIGHT, clip);
        }
        // Words in flight, home to their place.
        for w in &self.words {
            if let WordState::Flying { from, t } = w.state {
                let e = 1.0 - (1.0 - t) * (1.0 - t);
                let home = V2::new(w.x, w.y);
                let start = from.sub(V2::new(w.w / 2.0, w.h / 2.0));
                let p = at_page(start.lerp(home, e), off);
                let style = if w.title { mk::typo::TITLE } else { mk::typo::BODY };
                b.styled(engine, w.text.clone(), style, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.9 }, p.x, p.y, None, clip);
            }
        }
    }
    /// The frame: night page, words, creatures, the root pane on top.
    fn view_inner(&mut self, engine: &mut TextEngine, w: u32, h: u32) -> Scene {
        let (wf, hf) = (w as f32, h as f32);
        let mut b = SceneBuilder::new();
        b.quad(0.0, 0.0, wf, hf, NIGHT, 0.0, None, None);
        let top = self.chrome.build(engine, &mut b, w, h, self.palette);
        if self.chrome.rolled {
            return b.finish();
        }
        self.area = Rect { x: 0.0, y: top, w: wf, h: (hf - top).max(0.0) };
        b.quad(self.area.x, self.area.y, self.area.w, self.area.h, mk::Rgba { a: 0.0, ..NIGHT }, 0.0, Some(PAGE), None);
        if (self.laid_width - wf).abs() > 0.5 {
            self.lay_words(engine, wf);
            if self.creatures.is_empty() {
                self.hatch();
            }
        }
        self.roll_max = (self.page_h - self.area.h).max(0.0);
        self.roll = self.roll.clamp(0.0, self.roll_max);
        let off = V2::new(0.0, top - self.roll);
        // The words only move with the page (and a new layout bumps `rev`).
        let key = self.layout_rev ^ (self.roll.to_bits() as u64) << 20 ^ (w as u64) << 52 ^ (top.to_bits() as u64) << 8;
        if self.page_cache.as_ref().map(|c| c.0) != Some(key) {
            let part = self.build_page(engine, off);
            self.page_cache = Some((key, part));
        }
        if let Some((_, parts)) = &self.page_cache {
            // Home words from the cache; wanted and missing ones are marks.
            for (w, part) in self.words.iter().zip(parts) {
                if let (WordState::Home, Some(part)) = (w.state, part) {
                    b.append(part);
                }
            }
        }
        self.build_marks(engine, &mut b, off);
        self.build_creatures(engine, &mut b, off);
        if self.doze.asleep() {
            let cap = mk::typo::CALLOUT;
            let note = "Sie schlafen — tippe, um sie zu wecken.";
            let (tw, th) = engine.measure_styled(note, cap.px(), cap.weight);
            let (x, y) = ((wf - tw) / 2.0 - 14.0, hf - th - 34.0);
            b.quad(x, y, tw + 28.0, th + 14.0, mk::Rgba { r: 0.1, g: 0.11, b: 0.15, a: 0.92 }, (th + 14.0) / 2.0, Some(PAGE), None);
            b.styled(engine, note, cap, INK, x + 14.0, y + 7.0, Some(PAGE), None);
        }
        let mut scene = b.finish();
        if self.root.visible() {
            let mut rb = SceneBuilder::new();
            rb.scene = scene;
            let about = fam::RootAbout {
                name: "Matrix Krabbler",
                version: env!("CARGO_PKG_VERSION"),
                description: "Wortwesen krabbeln über eine Textseite und sammeln ihre Lieblingswörter. Tippen lockt sie an, ein Tipp auf ein Wesen verscheucht es.",
            };
            self.root.lay(engine, &mut rb, &about, wf, hf, self.palette, ROOT_BASE);
            scene = rb.finish();
        }
        scene
    }
}

fn at_page(p: V2, off: V2) -> V2 {
    p.add(off)
}

impl shell::Client for Krabbler {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn base(&mut self, _atlas: &mut ImageAtlas, buf: &mut [u8], _w: u32, _h: u32) {
        mk::night_glass::fill_bgra(buf);
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, _atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let t0 = std::time::Instant::now();
        let scene = self.view_inner(engine, w, h);
        self.meter.view(t0.elapsed().as_secs_f32() * 1000.0);
        scene
    }

    fn animating(&self) -> bool {
        self.moving() || self.chrome.animating() || self.root.animating()
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        let mut dirty = self.chrome.glide(dt);
        dirty |= self.root.glide(dt);
        if self.moving() && !self.creatures.is_empty() {
            // Big gaps (a stalled frame) must not throw them across the page.
            self.step(dt.min(0.05));
            self.frames += 1;
            self.meter.frame(self.frames);
            if let Some(line) = self.meter.report(&format!("; {} Wesen", self.creatures.len())) {
                eprintln!("{line}");
            }
            dirty = true;
        }
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
        let was_asleep = self.doze.asleep();
        self.wake();
        if let Some(deed) = self.root.strike_at(role, ROOT_BASE) {
            return match deed {
                fam::RootDeed::Help => {
                    matrixkit_scene::help_open("Matrix Krabbler");
                    Refresh::Calm
                }
                fam::RootDeed::Calm => Refresh::Calm,
                _ => Refresh::Dirty,
            };
        }
        match self.chrome.strike(role) {
            Some(fam::ChromeDeed::Close) => {
                eprintln!("[krabbler] Ampel × ({})", source.label());
                return Refresh::Close;
            }
            Some(fam::ChromeDeed::Answer(r)) => return r,
            Some(fam::ChromeDeed::Title) => {
                self.root.toggle();
                return Refresh::Dirty;
            }
            None => {}
        }
        // A sleeping page only wakes on the first touch.
        if was_asleep {
            return Refresh::Dirty;
        }
        if role == Some(PAGE) {
            self.press = Some(mk::touch::Press::new(x as f32, y as f32, self.roll, ()));
        }
        Refresh::Dirty
    }

    fn travel(&mut self, x: f64, y: f64, _source: Source) -> Refresh {
        let Some(pr) = &mut self.press else { return Refresh::Calm };
        if let Some(roll) = pr.travel(x as f32, y as f32, self.roll_max) {
            self.roll = roll;
            self.doze.touch();
            return Refresh::Dirty;
        }
        Refresh::Calm
    }

    fn release(&mut self, _role: Option<usize>, x: f64, y: f64, _source: Source) -> Refresh {
        let Some(pr) = self.press.take() else { return Refresh::Calm };
        if pr.release().is_some() {
            let p = V2::new(x as f32, y as f32 - self.area.y + self.roll);
            self.tap(p);
        }
        Refresh::Dirty
    }

    fn axis(&mut self, _x: f64, _y: f64, value: f64, _keep: &mut Keep<f32>) -> Refresh {
        self.wake();
        self.roll = (self.roll + value as f32).clamp(0.0, self.roll_max);
        Refresh::Dirty
    }

    fn root(&mut self) -> Refresh {
        self.root.toggle();
        Refresh::Dirty
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = std::env::args()
        .nth(1)
        .and_then(|p| std::fs::read_to_string(&p).map_err(|e| eprintln!("[krabbler] {p}: {e}")).ok())
        .unwrap_or_else(|| PAGE_TEXT.to_string());
    if !mk::window::single_instance(APP_ID) {
        return Ok(());
    }
    // A thousand dots a frame: the GPU paints them.
    shell::prefer_gpu();
    let app = Krabbler::new(&text);
    eprintln!("[krabbler] Matrix Krabbler, pid={}, {} Absätze", std::process::id(), app.paragraphs.len());
    shell::run(
        app,
        shell::Options {
            mount: shell::Mount::Window { app_id: String::from(APP_ID), title: String::from("Matrix Krabbler"), width: WIDTH, height: HEIGHT },
            argb: false,
            tag: "krabbler",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appetites() {
        assert!(Kind::Langleser.likes("Leinwand"));
        assert!(!Kind::Langleser.likes("und"));
        assert!(Kind::Weberin.likes("Bildschirmhelligkeit"));
        assert!(Kind::Filler.likes("und"));
        assert!(!Kind::Filler.likes("Haus"));
        assert!(Kind::Funke.likes("leuchten"));
        assert!(!Kind::Funke.likes("Garten"));
        assert_eq!(fam::letters("Wörter,"), "Wörter");
    }

    #[test]
    fn page_text_feeds_everyone() {
        let paras = fam::paragraphs(PAGE_TEXT);
        assert!(paras[0].0, "a heading first");
        let words: Vec<String> = paras.iter().filter(|p| !p.0).flat_map(|p| p.1.split_whitespace().map(fam::letters).collect::<Vec<_>>()).collect();
        for kind in KINDS {
            let n = words.iter().filter(|w| kind.likes(w)).count();
            assert!(n >= 10, "{kind:?} finds {n} words");
        }
    }
}
