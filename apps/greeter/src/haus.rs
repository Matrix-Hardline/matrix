//! Matrix login in the window house (round 33, 02.11.2026 — «without iced»).
//!
//! The login screen greetd shows at boot. On the Surface Duo it takes both
//! panels: the upper one (DSI-1) holds the clock, the person and the
//! password; the lower one (DSI-2) is the keyboard — the device has no
//! keys of its own. A single screen shows the keyboard under the field.
//! A physical keyboard types as well.
//!
//! The login itself is the greetd bridge (`crate::greetd::login`), run on
//! the loom. Success ends the process: the kiosk compositor quits with it
//! and greetd starts the session. `--probe` runs it inside a session to
//! look at it: the login is only pretended, Escape ends it.
//!
//! Round 166 (05.11.2026, the owner: «this login screen needs the
//! adjustments of the lock»): ON THE GPU, the chosen person's wallpaper and
//! lock clock as live glass (mk::login_share — their session shares them),
//! glass power keys, and the keyboard as large as the desktop's (the
//! finger pill: full width, `keys::PILL_H` high, one gap from the edges) on
//! glass.

use crate::{greetd, sessions_find, user_find};
use matrixkit_image::ImageAtlas;
use matrixkit_scene::{clock_edit, glass_clock, wall};
use matrixkit_scene::scene as fam;
use matrixkit_scene::scene::keyboard::{self as keys, Act};
use matrixkit_scene::symbol;
use matrixkit_shell::{self as shell, Anchors, Key, KeyboardWish, Layer, Loom, PanePlace, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{hit, Keep, Placed, Rect};

const R_FIELD: usize = 1;
const R_EYE: usize = 2;
const R_LOGIN: usize = 3;
const R_RESTART: usize = 4;
const R_OFF: usize = 5;
/// Morpheus (round 175): the setup's «next», «back» and its text field.
const R_NEXT: usize = 6;
const R_BACK: usize = 7;
const R_TEXT: usize = 8;
/// The setup's language and keyboard choices (06.11.2026).
const R_LANG: usize = 30;
const R_KBD: usize = 40;
/// The two languages and keyboards: (code, label).
const LANGS: [(&str, &str); 2] = [("de-CH", "Deutsch (Schweiz)"), ("en-GB", "English (UK)")];
const KBDS: [(&str, &str, &str); 2] = [("ch", "Schweiz (QWERTZ)", "Swiss (QWERTZ)"), ("gb", "UK (QWERTY)", "UK (QWERTY)")];

/// German or English, by the language chosen (mk::language).
fn l<'a>(de: &'a str, en: &'a str) -> &'a str {
    mk::language::pick(de, en)
}
const KEY_BASE: usize = 100;

/// The field and the pill: a phone-sized column.
const COLUMN: f32 = 380.0;

#[derive(Clone, PartialEq, Debug)]
enum Card {
    /// The keyboard on the lower panel (duo).
    Keys,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Phase {
    Rest,
    Checking,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Power {
    Restart,
    Off,
}

/// MORPHEUS FIRST START (round 175, 06.11.2026): a new device has no
/// account yet — the login screen leads through name and password, has the
/// first-start helper create the account (mk::account), then logs in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    /// Language and keyboard first — every page after speaks it.
    Language,
    Welcome,
    Name,
    Password,
    Again,
    Creating,
}

struct Setup {
    step: Step,
    lang: &'static str,
    kbd: &'static str,
    name: String,
    password: String,
    again: String,
}

enum Work {
    /// The first-start helper created the account (or why not).
    Created(Result<(), String>),
    Answer(Result<(), String>),
    Leave,
    /// The person's wallpaper, decoded at screen size (w, h, RGBA).
    Wall(u32, u32, Vec<u8>),
}

const WHITE: mk::Rgba = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

struct Greeter {
    loom: Option<Loom<Work>>,
    palette: mk::Palette,
    user: String,
    display: String,
    session: (String, String),
    password: String,
    reveal: bool,
    shift: bool,
    page: usize,
    phase: Phase,
    note: Option<(String, bool)>,
    clock: String,
    date: String,
    duo: bool,
    probe: bool,
    /// A power key asks once more before it acts.
    asking: Option<Power>,
    /// The key under the finger (row, column), lit while held.
    held: Option<(usize, usize)>,
    /// Role − KEY_BASE → (deed, row, column), laid with every picture.
    deeds: Vec<(Act, usize, usize)>,
    /// The keyboard pane's hit list (the house leaves panes to us).
    keys_placed: Vec<Placed>,
    keys_roles: Vec<Option<usize>>,
    /// Round 166: what the person shared, their lock clock, the live glass.
    shared: mk::login_share::Shared,
    values: clock_edit::Values,
    glass: f32,
    live: glass_clock::Live,
    fresh: Option<(u32, u32, Vec<u8>)>,
    /// Round 167: the person's wallpaper over both panels (kit family wall).
    wall: wall::Wall,
    /// The upper surface's and the keyboard pane's heights (one wallpaper
    /// over both panels, as the lock).
    main_h: f32,
    pane_h: f32,
    /// Morpheus: Some on a new device (no account yet).
    setup: Option<Setup>,
    /// Morpheus: the installer, in the live system (/etc/morpheus-live).
    installer: Option<crate::installer::Installer>,
}

/// «Montag, 2. November» — local time (libc, no calendar crate).
fn clock_and_date() -> (String, String) {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as libc::time_t).unwrap_or(0);
    // SAFETY: localtime_r writes only into the tm we own.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&now, &mut tm) }.is_null() {
        return (String::from("--:--"), String::new());
    }
    const DAYS: [&str; 7] = ["Sonntag", "Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag"];
    const MONTHS: [&str; 12] = ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"];
    (
        format!("{:02}:{:02}", tm.tm_hour, tm.tm_min),
        if mk::language::is_english() {
            // British: «Thursday 6 November».
            const DAYS_EN: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
            const MONTHS_EN: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
            format!("{} {} {}", DAYS_EN[tm.tm_wday.clamp(0, 6) as usize], tm.tm_mday, MONTHS_EN[tm.tm_mon.clamp(0, 11) as usize])
        } else {
            format!("{}, {}. {}", DAYS[tm.tm_wday.clamp(0, 6) as usize], tm.tm_mday, MONTHS[tm.tm_mon.clamp(0, 11) as usize])
        },
    )
}

impl Greeter {
    fn new(probe: bool) -> Greeter {
        // `--probe` with MATRIX_ERSTSTART=1 shows the setup on a device that
        // has its person (nothing is created in a probe).
        let found = if probe && std::env::var_os("MATRIX_ERSTSTART").is_some() { None } else { user_find() };
        let setup = found.is_none().then(|| {
            // Preselected from the device (Morpheus wrote /etc/matrix), else German.
            let lang = if mk::language::is_english() { "en-GB" } else { "de-CH" };
            let kbd = if mk::language::layout() == "gb" { "gb" } else { "ch" };
            mk::language::choose(lang);
            mk::language::choose_layout(kbd);
            Setup { step: Step::Language, lang, kbd, name: String::new(), password: String::new(), again: String::new() }
        });
        if setup.is_some() {
            eprintln!("[anmeldung] kein Konto — Ersteinrichtung (Morpheus)");
        }
        let (user, display) = found.unwrap_or_else(|| (String::new(), String::new()));
        let sessions = sessions_find();
        // The Matrix session first (the canvas), else the first one found.
        let session = sessions
            .iter()
            .find(|(n, _)| n.contains("Leinwand") || n.contains("Matrix"))
            .or_else(|| sessions.first())
            .cloned()
            .unwrap_or_else(|| (String::from("Matrix Leinwand"), String::from("/usr/local/bin/niri-canvas-session")));
        let (clock, date) = clock_and_date();
        let shared = mk::login_share::read(&user);
        let values = clock_edit::Values::from_lookup(|k| shared.get(k));
        let glass = shared.get("lock-clock-glass").and_then(|v| v.trim().parse().ok()).unwrap_or(0.85f32).clamp(0.0, 1.0);
        eprintln!("[anmeldung] Freigabe von {user}: Wand {}, Farbe {}", shared.wall.is_some(), shared.seed.is_some());
        Greeter {
            loom: None,
            // The person's tint (shared by their session); else — the greeter
            // account has none (olive under greetd, 02.11.2026) — Matrix violet.
            palette: mk::Palette::from_seed(shared.seed.unwrap_or(mk::Rgba { r: 0x5b as f32 / 255.0, g: 0x4b as f32 / 255.0, b: 0xdb as f32 / 255.0, a: 1.0 }), false),
            user,
            display,
            session,
            password: String::new(),
            reveal: false,
            shift: false,
            page: 0,
            phase: Phase::Rest,
            note: None,
            clock,
            date,
            duo: mk::device::panels().is_some(),
            probe,
            asking: None,
            held: None,
            deeds: Vec::new(),
            keys_placed: Vec::new(),
            keys_roles: Vec::new(),
            shared,
            values,
            glass,
            live: glass_clock::Live::new("anmeldung-uhr"),
            fresh: None,
            wall: wall::Wall::new("anmeldung-wand"),
            main_h: 0.0,
            pane_h: 0.0,
            setup,
            installer: crate::installer::live().then(|| {
                eprintln!("[anmeldung] Live-System — Morpheus-Installer");
                crate::installer::Installer::new()
            }),
        }
    }

    /// The text the setup's current step types into.
    fn setup_text(&mut self) -> Option<&mut String> {
        let s = self.setup.as_mut()?;
        match s.step {
            Step::Name => Some(&mut s.name),
            Step::Password => Some(&mut s.password),
            Step::Again => Some(&mut s.again),
            _ => None,
        }
    }

    /// «Weiter»: the next step, once this one is right.
    fn setup_next(&mut self) -> Refresh {
        let Some(s) = self.setup.as_mut() else { return Refresh::Calm };
        self.note = None;
        match s.step {
            Step::Language => s.step = Step::Welcome,
            Step::Welcome => s.step = Step::Name,
            Step::Name => {
                if s.name.trim().is_empty() {
                    self.note = Some((String::from(l("Wie heisst du?", "What is your name?")), true));
                } else {
                    s.step = Step::Password;
                }
            }
            Step::Password => {
                if s.password.chars().count() < mk::account::MIN_PASSWORD {
                    self.note = Some((String::from(l("Mindestens 4 Zeichen", "At least 4 characters")), true));
                } else {
                    s.step = Step::Again;
                }
            }
            Step::Again => match mk::account::password_check(&s.password, &s.again) {
                Err(why) => {
                    let why = if why.starts_with("Mindestens") { l(why, "At least 4 characters") } else { l(why, "The two passwords are not the same") };
                    self.note = Some((String::from(why), true));
                    s.again.clear();
                    mk::feel::warn();
                }
                Ok(()) => {
                    s.step = Step::Creating;
                    self.note = Some((String::from(l("Dein Konto wird angelegt …", "Creating your account …")), false));
                    let (name, pw, lang, kbd) = (s.name.trim().to_string(), s.password.clone(), s.lang, s.kbd);
                    let user = mk::account::username_from(&name);
                    if self.probe {
                        self.note = Some((format!("Probe: Konto «{user}» würde jetzt angelegt"), false));
                        s.step = Step::Again;
                        return Refresh::Dirty;
                    }
                    if let Some(loom) = &self.loom {
                        loom.spin(move || Work::Created(mk::account::create(&name, &user, &pw, lang, kbd)));
                    }
                }
            },
            Step::Creating => {}
        }
        Refresh::Full
    }

    fn setup_back(&mut self) -> Refresh {
        let Some(s) = self.setup.as_mut() else { return Refresh::Calm };
        self.note = None;
        s.step = match s.step {
            Step::Welcome => Step::Language,
            Step::Name => Step::Welcome,
            Step::Password => Step::Name,
            Step::Again => {
                s.again.clear();
                Step::Password
            }
            other => other,
        };
        Refresh::Full
    }

    /// Which part of the one wallpaper this surface shows (round 167).
    fn part(&self, upper: bool) -> wall::Part {
        match (self.duo, upper) {
            (false, _) => wall::Part::Whole,
            (true, true) => wall::Part::Upper,
            (true, false) => wall::Part::Lower,
        }
    }

    fn type_char(&mut self, c: char) {
        if self.phase != Phase::Rest {
            return;
        }
        if self.setup.is_some() {
            if let Some(t) = self.setup_text() {
                if t.chars().count() < 64 {
                    t.push(c);
                }
            }
            self.note = None;
            return;
        }
        if self.password.chars().count() < 128 {
            self.password.push(c);
        }
        self.note = None;
        self.asking = None;
    }

    fn login(&mut self) -> Refresh {
        if self.phase != Phase::Rest {
            return Refresh::Calm;
        }
        self.asking = None;
        if self.probe {
            self.note = Some((format!("Probe: «{}» würde jetzt starten", self.session.0), false));
            self.password.clear();
            return Refresh::Dirty;
        }
        let Some(loom) = &self.loom else { return Refresh::Calm };
        self.phase = Phase::Checking;
        self.note = Some((String::from(l("Wird geprüft …", "Checking …")), false));
        let (user, pw, cmd) = (self.user.clone(), self.password.clone(), self.session.1.clone());
        loom.spin(move || Work::Answer(greetd::login(&user, &pw, &cmd)));
        Refresh::Dirty
    }

    fn power(&mut self, what: Power) -> Refresh {
        if self.asking != Some(what) {
            self.asking = Some(what);
            return Refresh::Dirty;
        }
        self.asking = None;
        if self.probe {
            std::process::exit(0);
        }
        let verb = match what {
            Power::Restart => "reboot",
            Power::Off => "poweroff",
        };
        mk::process::spawn_detached("systemctl", &[verb]);
        Refresh::Dirty
    }

    /// A key types on touch-down, like the Matrix keyboard (R59).
    fn press_key(&mut self, act: Act) -> Refresh {
        mk::feel::tick();
        match act {
            Act::Text(c) => {
                self.type_char(c);
                if self.page == 0 {
                    self.shift = false;
                }
            }
            Act::Shift => self.shift = !self.shift,
            Act::Delete => {
                if self.setup.is_some() {
                    if let Some(t) = self.setup_text() {
                        t.pop();
                    }
                } else {
                    self.password.pop();
                }
            }
            Act::Enter => {
                if self.setup.is_some() {
                    self.setup_next();
                } else {
                    self.login();
                }
            }
            Act::Space => self.type_char(' '),
            Act::Page => {
                self.page = 1 - self.page;
                self.shift = false;
            }
            Act::Close => {}
        }
        Refresh::Panes
    }

    /// MORPHEUS: the setup's page — a title, a line, the step's field,
    /// the note, «Zurück» and «Weiter», the power keys as on the login.
    fn lay_setup(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, w: f32, h: f32, anchor: usize) {
        let p = self.palette;
        let white = WHITE;
        let soft = white.with_alpha(0.78);
        let home = Rect { x: 0.0, y: 0.0, w, h };
        let col = (COLUMN + 60.0).min(w - 2.0 * mk::spacing::L);
        let x0 = (w - col) / 2.0;
        let Some(s) = self.setup.as_ref() else { return };
        let step = s.step;
        let user = mk::account::username_from(&s.name);
        let (title, line): (String, String) = match step {
            Step::Language => (String::from("Sprache · Language"), String::from(l("Wähle die Sprache und die Tastatur deines Duo.", "Choose the language and keyboard of your Duo."))),
            Step::Welcome => (String::from(l("Willkommen bei Matrix", "Welcome to Matrix")), String::from(l("Dein Duo ist bereit. Richten wir es in zwei Schritten für dich ein.", "Your Duo is ready. Let us set it up for you in two steps."))),
            Step::Name => (String::from(l("Wie heisst du?", "What is your name?")), String::from(l("Dein Name erscheint beim Anmelden.", "Your name appears when you log in."))),
            Step::Password => (String::from(l("Wähle ein Passwort", "Choose a password")), format!("{} «{user}» — {}", l("Für dein Konto", "For your account"), l("mindestens 4 Zeichen.", "at least 4 characters."))),
            Step::Again => (String::from(l("Passwort wiederholen", "Repeat the password")), String::from(l("Zur Sicherheit noch einmal.", "Once more, to be sure."))),
            Step::Creating => (String::from(l("Einen Moment", "One moment")), String::from(l("Dein Konto wird angelegt, dann geht es los.", "Your account is being created, then off you go."))),
        };
        let mut y = (h * 0.12).max(mk::spacing::L);
        let t = mk::typo::LARGE_TITLE;
        let (tw, th) = engine.measure_styled(&title, t.px(), t.weight);
        let (t, tw, th) = if tw > col { let t2 = mk::typo::TITLE; let (a, b2) = engine.measure_styled(&title, t2.px(), t2.weight); (t2, a, b2) } else { (t, tw, th) };
        b.styled(engine, title.clone(), t, white, (w - tw) / 2.0, y, None, None);
        y += th + mk::spacing::S;
        for l in mk::wrap_words(&line, ((col / (mk::typo::BODY.px() * 0.5)) as usize).max(20), 3) {
            let (lw, lh) = engine.measure_styled(&l, mk::typo::BODY.px(), mk::typo::BODY.weight);
            b.styled(engine, l, mk::typo::BODY, soft, (w - lw) / 2.0, y, None, None);
            y += lh;
        }
        y += mk::spacing::XL;
        let fh = 52.0;
        let fill = mk::Rgba { r: 0.06, g: 0.06, b: 0.10, a: 0.72 };
        let bad = self.note.as_ref().is_some_and(|(_, bad)| *bad);
        match step {
            Step::Language => {
                let langs: Vec<String> = LANGS.iter().map(|(_, n)| n.to_string()).collect();
                let la = LANGS.iter().position(|(c, _)| *c == s.lang);
                y += fam::lay_choices(b, engine, &langs, la, x0, y, col, R_LANG, p) + mk::spacing::L;
                let (kw, kh) = engine.measure_styled(l("Tastatur", "Keyboard"), mk::typo::CAPTION.px(), mk::typo::CAPTION.weight);
                b.styled(engine, l("Tastatur", "Keyboard"), mk::typo::CAPTION, soft, (w - kw) / 2.0, y, None, None);
                y += kh + mk::spacing::XS;
                let kbds: Vec<String> = KBDS.iter().map(|(_, de, en)| l(de, en).to_string()).collect();
                let ka = KBDS.iter().position(|(c, _, _)| *c == s.kbd);
                y += fam::lay_choices(b, engine, &kbds, ka, x0, y, col, R_KBD, p);
            }
            Step::Name => {
                let mut fp = p;
                fp.surface_container_high = fill;
                fp.on_surface = white;
                fp.on_surface_variant = soft;
                fam::lay_text_field(b, engine, Rect { x: x0, y, w: col, h: fh }, &s.name, true, true, l("Vorname Nachname", "First and last name"), Some(symbol::EDIT), (Some(R_TEXT), None), None, fp);
                y += fh + mk::spacing::S;
                if !s.name.trim().is_empty() {
                    let k = format!("{}: {user}", l("Konto", "Account"));
                    let (kw, _) = engine.measure_styled(&k, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight);
                    b.styled(engine, k, mk::typo::CAPTION, soft, (w - kw) / 2.0, y, None, None);
                }
            }
            Step::Password | Step::Again => {
                let text = if step == Step::Password { &s.password } else { &s.again };
                let hint = if step == Step::Password { l("Passwort", "Password") } else { l("Passwort wiederholen", "Repeat the password") };
                fam::lay_secret_field(b, engine, Rect { x: x0, y, w: col, h: fh }, text, self.reveal, true, bad, hint, (Some(R_TEXT), Some(R_EYE)), white, fill, p);
                y += fh + mk::spacing::S;
            }
            _ => {}
        }
        y += 24.0;
        if let Some((text, bad)) = &self.note {
            let ink = if *bad { mk::Rgba { r: 1.0, g: 0.62, b: 0.60, a: 1.0 } } else { soft };
            let (nw, _) = engine.measure_styled(text, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight);
            b.styled(engine, text.clone(), mk::typo::CAPTION, ink, (w - nw) / 2.0, y, None, None);
        }
        y += 28.0;
        if step != Step::Creating {
            let next = match step {
                Step::Welcome => l("Los geht's", "Let's go"),
                Step::Again => l("Konto anlegen", "Create account"),
                _ => l("Weiter", "Continue"),
            };
            let nw = fam::pill_width(engine, Some(symbol::ARROW_FORWARD), next);
            let back = l("Zurück", "Back");
            let bw = if step == Step::Language { 0.0 } else { fam::pill_width(engine, Some(symbol::ARROW_BACK), back) };
            let gap = if bw > 0.0 { mk::spacing::M } else { 0.0 };
            let mut x = (w - nw - bw - gap) / 2.0;
            if bw > 0.0 {
                fam::lay_pill(b, engine, Some(symbol::ARROW_BACK), back, x, y, None, Some(R_BACK), fam::PillTone::Quiet, p);
                x += bw + gap;
            }
            fam::lay_pill(b, engine, Some(symbol::ARROW_FORWARD), next, x, y, None, Some(R_NEXT), fam::PillTone::Accent, p);
        }
        // The power keys as on the login.
        let key = 48.0;
        let by = h - key - mk::spacing::L;
        let mut bx = w - mk::spacing::L - key;
        for (power, glyph, role) in [(Power::Off, symbol::POWER, R_OFF), (Power::Restart, symbol::RESTART, R_RESTART)] {
            wall::glass_key(b, engine, atlas, &self.wall, glyph, bx, by, key, self.asking == Some(power), role, home, anchor, p);
            bx -= key + mk::spacing::S;
        }
    }

    /// The upper part: clock, person, field, note, pill, power keys.
    fn lay_login(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, w: f32, h: f32, anchor: usize) {
        let p = self.palette;
        let white = WHITE;
        let soft = white.with_alpha(0.72);
        let s = mk::spacing::S;
        let home = Rect { x: 0.0, y: 0.0, w, h };
        // Round 166: as the lock — the date, then the person's glass clock.
        let date_style = mk::typo::HEADLINE;
        let dpx = date_style.px() * 1.15;
        let (dw, dh) = engine.measure_family(matrixkit_text::FAMILY, &self.date, dpx, date_style.weight);
        let mut y = (h * 0.06).max(s);
        b.text(engine, self.date.clone(), dpx, date_style.weight, matrixkit_text::FAMILY, white.with_alpha(0.92), (w - dw) / 2.0, y, None, None);
        y += dh + 6.0;
        let v = self.values;
        let room = h * 0.30;
        let mut digit_w = (room / 1.9).min(w * 0.13) * v.width;
        let probe = glass_clock::Look { digit_w, stretch: v.stretch, glass: self.glass, weight: v.weight, body: v.body, ink: white };
        let (pw0, _) = glass_clock::measure(&self.clock, &probe);
        let room_w = w - 2.0 * mk::spacing::L;
        let tall = digit_w * v.stretch;
        if pw0 > room_w {
            digit_w *= room_w / pw0;
        }
        let digit_w = digit_w.floor();
        let stretch = (tall / digit_w.max(1.0)).clamp(glass_clock::Look::STRETCH_MIN, glass_clock::Look::STRETCH_MAX);
        let look = glass_clock::Look { digit_w, stretch, ..probe };
        let (cw, ch) = glass_clock::measure(&self.clock, &look);
        let (cx, cy) = ((w - cw) / 2.0, y);
        let pic = (self.wall.shown().is_some() && self.glass >= 0.3).then(|| {
            let backdrop = self.wall.path();
            self.live.picture(atlas, &self.clock, &look, cx, cy, shell::screen_scale() as f32, glass_clock::Pace::Still, &backdrop, 1.0)
        });
        match pic.flatten() {
            Some(pic) => b.pict(pic.pict(home, anchor)),
            None => glass_clock::lay(b, &self.clock, cx, cy, &look, home, anchor),
        }
        y += ch + mk::spacing::XL;
        // The person: a round initial and the name.
        let col = COLUMN.min(w - 2.0 * mk::spacing::L);
        let x0 = (w - col) / 2.0;
        let av = 48.0;
        b.quad(x0, y, av, av, p.primary_container, av / 2.0, None, None);
        let initial = self.display.chars().next().unwrap_or('M').to_uppercase().to_string();
        let ist = mk::typo::TITLE;
        let (iw, ih) = engine.measure_styled(&initial, ist.px(), ist.weight);
        b.styled(engine, initial, ist, p.on_primary_container, x0 + (av - iw) / 2.0, y + (av - ih) / 2.0, None, None);
        let (_, nh) = engine.measure_styled(&self.display, mk::typo::TITLE.px(), mk::typo::TITLE.weight);
        b.styled(engine, self.display.clone(), mk::typo::TITLE, white, x0 + av + mk::spacing::M, y + av / 2.0 - nh + 2.0, None, None);
        b.styled(engine, self.session.0.clone(), mk::typo::CAPTION, soft, x0 + av + mk::spacing::M, y + av / 2.0 + 2.0, None, None);
        y += av + mk::spacing::L;
        // The field.
        let fh = 52.0;
        let bad = self.note.as_ref().is_some_and(|(_, bad)| *bad);
        let fill = mk::Rgba { r: 0.06, g: 0.06, b: 0.10, a: 0.72 };
        fam::lay_secret_field(b, engine, Rect { x: x0, y, w: col, h: fh }, &self.password, self.reveal, false, bad, l("Passwort", "Password"), (Some(R_FIELD), Some(R_EYE)), white, fill, p);
        y += fh + s;
        // The note (checking / refused / probe).
        if let Some((text, bad)) = &self.note {
            let ink = if *bad { mk::Rgba { r: 1.0, g: 0.62, b: 0.60, a: 1.0 } } else { soft };
            let (nw, _) = engine.measure_styled(text, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight);
            b.styled(engine, text.clone(), mk::typo::CAPTION, ink, (w - nw) / 2.0, y, None, None);
        }
        y += 22.0;
        // The login pill (the keyboard's enter does the same).
        let label = if self.phase == Phase::Checking { l("Wird geprüft …", "Checking …") } else { l("Anmelden", "Log in") };
        let pw = fam::pill_width(engine, Some(symbol::ARROW_FORWARD), label);
        fam::lay_pill(b, engine, Some(symbol::ARROW_FORWARD), label, (w - pw) / 2.0, y, None, Some(R_LOGIN), fam::PillTone::Accent, p);
        // Power keys, bottom right, as glass (round 166); a first tap asks.
        let key = 48.0;
        let by = h - key - mk::spacing::L;
        let mut bx = w - mk::spacing::L - key;
        for (power, glyph, role, ask) in [(Power::Off, symbol::POWER, R_OFF, l("Nochmals tippen: ausschalten", "Tap again: power off")), (Power::Restart, symbol::RESTART, R_RESTART, l("Nochmals tippen: neu starten", "Tap again: restart"))] {
            wall::glass_key(b, engine, atlas, &self.wall, glyph, bx, by, key, self.asking == Some(power), role, home, anchor, p);
            if self.asking == Some(power) {
                let (aw, ah) = engine.measure_styled(ask, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight);
                b.styled(engine, ask, mk::typo::CAPTION, white, w - mk::spacing::L - aw, by - ah - s, None, None);
            }
            bx -= key + s;
        }
    }

    /// The keyboard as on the desktop (keys::finger_rect) on a glass card
    /// over the wallpaper (kit family wall).
    fn lay_keys_glass(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, atlas: &mut ImageAtlas, rect: Rect, home: Rect, anchor: usize) {
        let _on_glass = wall::glass_ground(b, atlas, &self.wall, rect, 22.0, home, anchor);
        self.lay_keys(b, engine, rect);
    }

    fn lay_keys(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, rect: Rect) {
        let enter = if self.phase == Phase::Checking { "…" } else if self.setup.is_some() { l("Weiter", "Continue") } else { l("Anmelden", "Log in") };
        let look = keys::Look { page: self.page, shift: self.shift, closable: false, glowing: &[], held: self.held, focus: None, enter };
        self.deeds = keys::lay(b, engine, rect, &look, KEY_BASE, self.palette);
    }

    fn key_strike(&mut self, role: Option<usize>) -> Refresh {
        let Some(&(act, row, col)) = role.and_then(|r| r.checked_sub(KEY_BASE)).and_then(|i| self.deeds.get(i)) else { return Refresh::Calm };
        self.held = Some((row, col));
        self.press_key(act)
    }

    fn key_release(&mut self) -> Refresh {
        if self.held.take().is_some() {
            return Refresh::Panes;
        }
        Refresh::Calm
    }
}

/// The night behind it all: the Matrix blue running into violet.
fn paint_night(buf: &mut [u8], w: u32, h: u32) {
    let (top, bottom) = ([0x0b, 0x0d, 0x2a], [0x2a, 0x10, 0x4a]);
    for yy in 0..h {
        let t = yy as f32 / h.max(1) as f32;
        let c: Vec<u8> = (0..3).map(|i| (top[i] as f32 + (bottom[i] as f32 - top[i] as f32) * t) as u8).collect();
        let row = &mut buf[(yy * w * 4) as usize..((yy + 1) * w * 4) as usize];
        for px in row.chunks_exact_mut(4) {
            // The house's buffers are BGRA.
            px[0] = c[2];
            px[1] = c[1];
            px[2] = c[0];
            px[3] = 255;
        }
    }
}

impl shell::Client for Greeter {
    type Card = Card;
    type Work = Work;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &Loom<Work>) {
        self.loom = Some(loom.clone());
        // The person's wallpaper, decoded off the surface — Morpheus: a new
        // device (nobody shared one yet) shows the Matrix standard picture.
        let standard = std::path::PathBuf::from("/usr/share/backgrounds/matrix/matrix-standard.jpg");
        if let Some(path) = self.shared.wall.clone().or_else(|| standard.exists().then_some(standard)) {
            loom.spin(move || {
                let t0 = std::time::Instant::now();
                let got = matrixkit_image::fitted_rgba(&path, mk::device::image_long_side());
                eprintln!("[anmeldung] Wand {}: {} in {} ms", path.display(), if got.is_some() { "dekodiert" } else { "NICHT lesbar" }, t0.elapsed().as_millis());
                let got = got.unwrap_or((0, 0, Vec::new()));
                Work::Wall(got.0, got.1, got.2)
            });
        }
    }

    fn woven(&mut self, work: Work) -> Refresh {
        match work {
            Work::Created(Ok(())) => {
                // The account exists: log in with it — the session starts.
                if let Some(s) = self.setup.take() {
                    let name = s.name.trim().to_string();
                    self.user = mk::account::username_from(&name);
                    self.display = name;
                    self.password = s.password;
                    eprintln!("[anmeldung] Konto «{}» angelegt — erste Anmeldung", self.user);
                    mk::feel::confirm();
                    return self.login();
                }
            }
            Work::Created(Err(e)) => {
                eprintln!("[anmeldung] Konto nicht angelegt: {e}");
                if let Some(s) = self.setup.as_mut() {
                    s.step = Step::Again;
                    s.again.clear();
                }
                self.note = Some((e, true));
                mk::feel::warn();
            }
            Work::Answer(Ok(())) => {
                self.phase = Phase::Done;
                self.note = Some((String::from(l("Willkommen", "Welcome")), false));
                mk::feel::confirm();
                // greetd has the session: we leave (the kiosk compositor quits with us).
                if let Some(loom) = &self.loom {
                    loom.spin(|| {
                        std::thread::sleep(std::time::Duration::from_millis(250));
                        Work::Leave
                    });
                }
            }
            Work::Answer(Err(e)) => {
                eprintln!("[anmeldung] abgelehnt: {e}");
                self.phase = Phase::Rest;
                self.password.clear();
                let text = if e.to_lowercase().contains("auth") || e.contains("abgelehnt") { String::from(l("Passwort falsch — nochmals versuchen", "Wrong password — try again")) } else { e };
                self.note = Some((text, true));
                mk::feel::warn();
            }
            Work::Leave => std::process::exit(0),
            Work::Wall(w, h, rgba) => {
                eprintln!("[anmeldung] Wand angekommen: {w}×{h}");
                if w > 0 {
                    self.fresh = Some((w, h, rgba));
                    return Refresh::Panes;
                }
                return Refresh::Calm;
            }
        }
        Refresh::Full
    }

    fn base(&mut self, _atlas: &mut ImageAtlas, buf: &mut [u8], w: u32, h: u32) {
        paint_night(buf, w, h);
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let (wf, hf) = (w as f32, h as f32);
        self.main_h = hf;
        let mut b = SceneBuilder::new();
        let ground = b.quad(0.0, 0.0, wf, hf, mk::Rgba { r: 0.04, g: 0.05, b: 0.12, a: 1.0 }, 0.0, None, None);
        self.wall.take(atlas, self.fresh.take());
        let (total, shift) = wall::span(self.part(true), hf, self.pane_h);
        self.wall.lay(&mut b, wf, hf, total, shift, ground, 0.22);
        // Everything after lies above the wallpaper picture (round 150).
        let top = b.rise();
        let home = Rect { x: 0.0, y: 0.0, w: wf, h: hf };
        if let Some(inst) = &self.installer {
            inst.lay(&mut b, engine, wf, hf, self.palette);
            return b.finish();
        }
        if self.duo {
            if self.setup.is_some() {
                self.lay_setup(&mut b, engine, atlas, wf, hf, top);
            } else {
                self.lay_login(&mut b, engine, atlas, wf, hf, top);
            }
        } else {
            // One screen: the login above, the keyboard as on the desktop.
            let rect = keys::finger_rect(wf, hf);
            if self.setup.is_some() {
                self.lay_setup(&mut b, engine, atlas, wf, rect.y, top);
            } else {
                self.lay_login(&mut b, engine, atlas, wf, rect.y, top);
            }
            self.lay_keys_glass(&mut b, engine, atlas, rect, home, top);
        }
        b.finish()
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        match role {
            Some(R_EYE) => {
                self.reveal = !self.reveal;
                Refresh::Dirty
            }
            Some(R_LOGIN) => self.login(),
            Some(r) if (200..300).contains(&r) && self.installer.is_some() => {
                let changed = self.installer.as_mut().is_some_and(|i| i.strike(r));
                if changed {
                    Refresh::Full
                } else {
                    Refresh::Calm
                }
            }
            Some(r) if (R_LANG..R_LANG + LANGS.len()).contains(&r) && self.setup.is_some() => {
                let code = LANGS[r - R_LANG].0;
                if let Some(s) = self.setup.as_mut() {
                    s.lang = code;
                    // The keyboard follows the language until chosen otherwise.
                    s.kbd = if code == "en-GB" { "gb" } else { "ch" };
                    mk::language::choose(code);
                    mk::language::choose_layout(s.kbd);
                }
                mk::feel::tap();
                Refresh::Panes
            }
            Some(r) if (R_KBD..R_KBD + KBDS.len()).contains(&r) && self.setup.is_some() => {
                if let Some(s) = self.setup.as_mut() {
                    s.kbd = KBDS[r - R_KBD].0;
                    mk::language::choose_layout(s.kbd);
                }
                mk::feel::tap();
                Refresh::Panes
            }
            Some(R_NEXT) => self.setup_next(),
            Some(R_BACK) => self.setup_back(),
            Some(R_TEXT) => Refresh::Calm,
            Some(R_RESTART) => self.power(Power::Restart),
            Some(R_OFF) => self.power(Power::Off),
            Some(r) if r >= KEY_BASE => self.key_strike(role),
            _ => {
                if self.asking.take().is_some() {
                    return Refresh::Dirty;
                }
                Refresh::Calm
            }
        }
    }

    fn release(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        self.key_release()
    }

    fn key(&mut self, key: Key, _shift: bool) -> Refresh {
        match key {
            Key::Char(s) => {
                for c in s.chars().filter(|c| !c.is_control()) {
                    self.type_char(c);
                }
            }
            Key::Backspace => {
                if self.setup.is_some() {
                    if let Some(t) = self.setup_text() {
                        t.pop();
                    }
                } else {
                    self.password.pop();
                }
            }
            Key::Enter if self.setup.is_some() => return self.setup_next(),
            Key::Enter => return self.login(),
            Key::Escape if self.probe => std::process::exit(0),
            Key::Escape => {
                self.password.clear();
                self.asking = None;
            }
            _ => return Refresh::Calm,
        }
        Refresh::Full
    }

    fn wants_text(&self) -> bool {
        true
    }

    fn beat(&mut self) -> Refresh {
        if let Some(inst) = self.installer.as_mut() {
            if inst.refresh() {
                return Refresh::Full;
            }
        }
        let (clock, date) = clock_and_date();
        if clock != self.clock {
            self.clock = clock;
            self.date = date;
            return Refresh::Dirty;
        }
        Refresh::Calm
    }

    fn output(&self) -> Option<String> {
        self.duo.then(|| String::from("DSI-1"))
    }

    fn fleet(&mut self) -> Vec<shell::fleet::Spec<Card>> {
        // The installer types nothing: no keyboard on the lower panel.
        if !self.duo || self.installer.is_some() {
            return Vec::new();
        }
        vec![shell::fleet::Spec { pane: Card::Keys, width: 0, height: 0, cross: 0, namespace: "matrix-anmeldung-tasten", keyboard: false, sig: (1, 2) }]
    }

    fn pane_place(&mut self, _card: &Card) -> Option<PanePlace> {
        let mut place = PanePlace::veil(Some(String::from("DSI-2")), Some(Layer::Overlay), false);
        place.keyboard = KeyboardWish::None;
        Some(place)
    }

    fn pane_scene(&mut self, _card: &Card, engine: &mut TextEngine, atlas: &mut ImageAtlas, w: u32, h: u32) -> Option<Scene> {
        let (wf, hf) = (w as f32, h as f32);
        self.pane_h = hf;
        let mut b = SceneBuilder::new();
        let ground = b.quad(0.0, 0.0, wf, hf, mk::Rgba { r: 0x12 as f32 / 255.0, g: 0x10 as f32 / 255.0, b: 0x30 as f32 / 255.0, a: 1.0 }, 0.0, None, None);
        // Round 166: the wallpaper's lower part, the keyboard as on the desktop.
        self.wall.take(atlas, self.fresh.take());
        let (total, shift) = wall::span(self.part(false), hf, self.main_h);
        self.wall.lay(&mut b, wf, hf, total, shift, ground, 0.22);
        let top = b.rise();
        let home = Rect { x: 0.0, y: 0.0, w: wf, h: hf };
        self.lay_keys_glass(&mut b, engine, atlas, keys::finger_rect(wf, hf), home, top);
        let scene = b.finish();
        self.keys_placed = scene.placed.clone();
        self.keys_roles = scene.roles.clone();
        Some(scene)
    }

    fn pane_strike(&mut self, _card: &Card, x: f64, y: f64, _source: Source) -> Refresh {
        let role = hit(&self.keys_placed, x as f32, y as f32).and_then(|i| self.keys_roles.get(i).copied().flatten());
        self.key_strike(role)
    }

    fn pane_release(&mut self, _card: &Card, _x: f64, _y: f64, _source: Source) -> Refresh {
        self.key_release()
    }
}

pub fn run(probe: bool) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[anmeldung] Matrix Anmeldung im Haus (round 166), Probe: {probe}");
    // Round 166: the GPU paints (live glass needs it; MATRIX_CPU=1 is the way out).
    shell::prefer_gpu();
    shell::run(
        Greeter::new(probe),
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-anmeldung"),
                layer: Layer::Overlay,
                keyboard: KeyboardWish::Exclusive,
                anchor: Anchors::all(),
                size: (0, 0),
                margin: (0, 0, 0, 0),
                exclusive: -1,
            },
            argb: false,
            tag: "anmeldung",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Across,
        },
    )
}
