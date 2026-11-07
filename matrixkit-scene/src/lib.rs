//! MatrixKit Scene — the families of the scene tree, without iced.
//!
//! R207 (21.09.2026): the kit split. Until Lager 110 the light
//! families (`scene`), the symbol and mono fonts, focus, the button
//! axes and `help_open` lived in `matrixkit-widgets` — next to the
//! iced gate. An extract without iced needed them separate: HERE lies
//! everything a light client needs, and nothing that knows iced.
//! `matrixkit-widgets` re-exports every name at its old place, no
//! caller changes.

use matrixkit_theme as mk;

pub mod glass_clock;
pub mod passcode;
pub mod notices;
pub mod clock_edit;
pub mod stack;
pub mod wall;

pub mod buttons {
    /// How present is the button? (bordered/borderedProminent/borderless)
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Style {
        /// Filled with accent — the ONE primary action of a surface.
        Prominent,
        /// Tinted pill — normal actions (reference "bordered").
        Tinted,
        /// Text/state layer only — quiet actions in rows and corners.
        Borderless,
    }
    /// What does the button mean? Role colors, the dialog decides position.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Role {
        Normal,
        /// Destructive (delete, remove) — error color world.
        Destructive,
    }
    /// Density level (ControlSize: mini/small/regular/large → 3 suffice).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Size {
        Small,
        Normal,
        Large,
    }
}

/// R178: width of the app dashboard. A cap, not a fixed size — the
/// layer grows with the window and stops here, because longer lines
/// read worse (reference measure of the form width).
pub const ROOT_WIDTH: f32 = 560.0;

/// Default magnification on hover (subtle, like touch-reference buttons).
pub const MAGNIFIER_MAX: f32 = 1.07;

pub mod symbol {
    // R210 (Tube's chip row): quality, subtitles, autoplay, fold up.
    pub const HIGH_QUALITY: char = '\u{e024}';
    pub const CLOSED_CAPTION: char = '\u{e01c}';
    pub const REPEAT: char = '\u{e040}';
    pub const TIMER: char = '\u{e425}';
    pub const EXPAND_LESS: char = '\u{e5ce}';
    pub const CLOSE: char = '\u{e5cd}';
    pub const ARROW_BACK: char = '\u{e5c4}';
    pub const CHECK: char = '\u{e5ca}';
    pub const RADIO_CHECKED: char = '\u{e837}';
    pub const RADIO_UNCHECKED: char = '\u{e836}';
    pub const CHECK_BOX: char = '\u{e834}';
    pub const CHECK_BOX_BLANK: char = '\u{e835}';
    pub const SAVE: char = '\u{e161}';
    /// R181: the personal name (petname) and keeping it on the shelf.
    pub const STAR: char = '\u{e838}';
    pub const STAR_OUTLINE: char = '\u{e83a}';
    pub const BOOKMARK: char = '\u{e866}';
    pub const EVENT: char = '\u{e878}';
    /// Input (R92): controller section of the settings.
    pub const SPORTS_ESPORTS: char = '\u{ea28}';
    pub const INFO: char = '\u{e88e}';
    pub const CODE: char = '\u{e86f}';
    pub const PALETTE: char = '\u{e40a}';
    pub const ROCKET_LAUNCH: char = '\u{eb9b}';
    pub const TOUCH_APP: char = '\u{e913}';
    pub const CONTENT_COPY: char = '\u{e14d}';
    pub const SHIELD: char = '\u{e9e0}';
    pub const MONITORING: char = '\u{f190}';
    pub const MENU_BOOK: char = '\u{ea19}';
    pub const PLAY_ARROW: char = '\u{e037}';
    pub const PAUSE: char = '\u{e034}';
    pub const VOLUME_UP: char = '\u{e050}';
    pub const VOLUME_OFF: char = '\u{e04f}';
    pub const GRAPHIC_EQ: char = '\u{e1b8}';
    pub const LOCK: char = '\u{e899}';
    pub const LOCK_OPEN: char = '\u{e898}';
    pub const CHEVRON_RIGHT: char = '\u{e5cc}';
    pub const UNFOLD_LESS: char = '\u{e5d6}';
    pub const UNFOLD_MORE: char = '\u{e5d7}';
    pub const ARROW_UPWARD: char = '\u{e5d8}';
    pub const ARROW_DOWNWARD: char = '\u{e5db}';
    pub const WARNING: char = '\u{e002}';
    pub const USB: char = '\u{e1e0}';
    /// R149: an attached iOS device in the sidebar (the iPhone bridge).
    pub const SMARTPHONE: char = '\u{e32c}';
    /// Morpheus (06.11.2026): the laptop beside the duo — pairing, the companion app.
    pub const LAPTOP: char = '\u{e31e}';
    pub const NOTIFICATIONS: char = '\u{e7f4}';
    pub const TUNE: char = '\u{e429}';
    pub const BATTERY_FULL: char = '\u{e1a4}';
    pub const BATTERY_CHARGING: char = '\u{e1a3}';
    pub const APPS: char = '\u{e5c3}';
    /// R171: share (material `share`) — Decimen shows the code.
    pub const SHARE: char = '\u{e80d}';
    pub const BRIGHTNESS: char = '\u{e1ac}';
    pub const MIC: char = '\u{e029}';
    pub const MIC_OFF: char = '\u{e02b}';
    pub const POWER: char = '\u{e8ac}';
    pub const DARK_MODE: char = '\u{e51c}';
    pub const RESTART: char = '\u{e5d5}';
    pub const LOGOUT: char = '\u{e9ba}';
    pub const WIFI: char = '\u{e63e}';
    pub const BLUETOOTH: char = '\u{e1a7}';
    /// Headphones (Bluetooth pairing, R210).
    pub const HEADPHONES: char = '\u{e310}';
    /// App card rows (R211): settings gear, panel swap, the Krabbler bug.
    pub const SETTINGS: char = '\u{e8b8}';
    pub const SWAP_VERT: char = '\u{e8d5}';
    pub const BUG: char = '\u{e868}';
    /// Playlist steps (R210).
    pub const SKIP_PREVIOUS: char = '\u{e045}';
    pub const SKIP_NEXT: char = '\u{e044}';
    pub const QUEUE_MUSIC: char = '\u{e03d}';
    pub const KEY: char = '\u{e73c}';
    /// auto_awesome — the AI sparkle (the assistant's folder in Dateien).
    pub const SPARK: char = '\u{e65f}';
    /// cloud — sync services (the cloud folder in Dateien).
    pub const CLOUD: char = '\u{e2bd}';
    pub const REMOVE: char = '\u{e15b}';
    pub const ADD: char = '\u{e145}';
    pub const EXPAND_MORE: char = '\u{e5cf}';
    pub const SEARCH: char = '\u{e8b6}';
    /// video_library — a library of kept videos (Tube's compact tab, R172).
    pub const VIDEO_LIBRARY: char = '\u{e04a}';
    pub const VISIBILITY_OFF: char = '\u{e8f5}';
    pub const VISIBILITY: char = '\u{e8f4}';
    pub const BACKSPACE: char = '\u{e14a}';
    pub const KEYBOARD_RETURN: char = '\u{e31b}';
    pub const HELP: char = '\u{e887}';
    pub const SCHEDULE: char = '\u{e8b5}';
    pub const PERSON: char = '\u{e7fd}';
    pub const WIDGETS: char = '\u{e1bd}';
    pub const PUBLIC: char = '\u{e80b}';
    pub const DRAG_INDICATOR: char = '\u{e945}';
    // File world (Matrix Dateien, round 29)
    pub const FOLDER: char = '\u{e2c7}';
    pub const FILE: char = '\u{e24d}';
    pub const IMAGE: char = '\u{e3f4}';
    pub const MOVIE: char = '\u{e02c}';
    pub const MUSIC_NOTE: char = '\u{e405}';
    pub const HOME: char = '\u{e88a}';
    pub const DOWNLOAD: char = '\u{e2c4}';
    pub const DELETE: char = '\u{e872}';
    /// Material «edit» (pencil) — renaming (R210 round 19).
    pub const EDIT: char = '\u{e3c9}';
    pub const STORAGE: char = '\u{e1db}';
    pub const MEMORY: char = '\u{e322}';
    pub const CANVAS: char = '\u{e30c}';
    pub const PDF: char = '\u{e415}';
    pub const ARROW_FORWARD: char = '\u{e5c8}';
    pub const NEW_FOLDER: char = '\u{e2cc}';
    // Control registry (R122 phase 2): accessibility triad glyphs.
    pub const ANIMATION: char = '\u{e71c}';
    pub const CONTRAST: char = '\u{eb37}';
    pub const OPACITY: char = '\u{e91c}';
    /// The reorder handle (reference `line.3.horizontal`, measured
    /// 23.07.2026: three quiet strokes, wider than tall, trailing edge).
    pub const REORDER: char = '\u{e5d2}';
}

// ---------------------------------------------------------------------------
// Symbol font loading (R102). Lesson from the icon carnival after the
// 2026-07-21 rebase: the font rode along as a DMS asset and vanished
// with the package — a foreign fallback font filled the same PUA
// codepoints with alien glyphs. From now on the fonts are OURS
// (/usr/share/fonts/matrix/) and the loader resolves a chain:
//   1. ~/.config/matrix/symbols.ttf        — user override
//   2. symbols-<set>.ttf                   — chosen set (symbol-set)
//   3. symbols-rounded.ttf                 — the default set
//   4. the old DMS ride-along path         — pre-R102 images
// The FAMILY the widgets reference is read from the loaded file itself,
// so a set or user font only has to exist to take over the whole OS.
// ---------------------------------------------------------------------------

const SYMBOL_FONT_DIR: &str = "/usr/share/fonts/matrix";
const SYMBOL_FONT_LEGACY: &str = "/usr/share/quickshell/dms/assets/fonts/material-design-icons/variablefont/MaterialSymbolsRounded[FILL,GRAD,opsz,wght].ttf";

static SYMBOL_FAMILY: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
static SYMBOL_FILL_FAMILY: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
// R123 optical masters: static opsz instances baked by
// tools/bake-symbol-instances.py. Measured on the reference (SDK
// interrogation 23.07.2026): AppKit drives the optical axis with the
// point size — 13 pt text renders from the SMALLEST optical master.
// Our text stack renders only default instances, so the kit loads the
// baked 20/48 masters as separate families and picks by size.
static SYMBOL_20_FAMILY: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
static SYMBOL_48_FAMILY: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
static SYMBOL_FILL_20_FAMILY: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();
static SYMBOL_FILL_48_FAMILY: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();

/// The family of the loaded symbol font — falls back to the Rounded
/// name so tests and font-less builds keep compiling glyph runs.
pub fn symbol_family() -> &'static str {
    SYMBOL_FAMILY.get().copied().unwrap_or("Material Symbols Rounded")
}

/// The family of the FILLED variant; without one, filled symbols
/// degrade to the outline family (old images, user fonts without fill).
pub fn symbol_fill_family() -> &'static str {
    SYMBOL_FILL_FAMILY.get().copied().unwrap_or_else(symbol_family)
}

static MONO_FAMILY: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();

/// Lager 58: the mono family for the scene. The iced version hands
/// the font file on to iced; the text engine wants the FAMILY NAME —
/// it comes from the same TTF name table as for the symbols. Without
/// a loaded file the known name stays, so tests and font-less builds
/// keep running.
pub fn mono_family() -> &'static str {
    MONO_FAMILY.get().copied().unwrap_or("Adwaita Mono")
}

/// Loads the mono font into the text engine (like arm_symbols) and
/// remembers its family name.
pub fn arm_mono(engine: &mut matrixkit_text::TextEngine) {
    let data = load_mono_font().into_owned();
    if let Some(name) = font_family_name(&data) {
        let _ = MONO_FAMILY.set(Box::leak(name.into_boxed_str()));
    }
    engine.load_font(data);
}

/// Family name straight out of the TTF name table (nameID 16 preferred,
/// else 1) — the loaded file decides which family the widgets reference.
fn font_family_name(data: &[u8]) -> Option<String> {
    let u16be = |o: usize| -> Option<u16> {
        Some(u16::from_be_bytes([*data.get(o)?, *data.get(o + 1)?]))
    };
    let table_count = u16be(4)? as usize;
    let mut name_base = None;
    for i in 0..table_count {
        let rec = 12 + i * 16;
        if data.get(rec..rec + 4)? == b"name" {
            let off = u32::from_be_bytes(data.get(rec + 8..rec + 12)?.try_into().ok()?);
            name_base = Some(off as usize);
            break;
        }
    }
    let base = name_base?;
    let count = u16be(base + 2)? as usize;
    let strings = base + u16be(base + 4)? as usize;
    let mut best: Option<(u8, String)> = None;
    for i in 0..count {
        let r = base + 6 + i * 12;
        let (platform, name_id) = (u16be(r)?, u16be(r + 6)?);
        if name_id != 1 && name_id != 16 {
            continue;
        }
        let len = u16be(r + 8)? as usize;
        let off = strings + u16be(r + 10)? as usize;
        let raw = data.get(off..off + len)?;
        let value = match platform {
            // Windows/Unicode records are UTF-16BE.
            0 | 3 => char::decode_utf16(
                raw.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])),
            )
            .collect::<Result<String, _>>()
            .ok()?,
            _ => String::from_utf8_lossy(raw).into_owned(),
        };
        let priority = if name_id == 16 { 2 } else { 1 };
        if best.as_ref().map_or(true, |(b, _)| priority > *b) {
            best = Some((priority, value));
        }
    }
    best.map(|(_, s)| s)
}

/// Resolves the chain to (outline font, filled variant).
fn symbol_font_paths() -> (Option<std::path::PathBuf>, Option<std::path::PathBuf>) {
    use std::path::PathBuf;
    if let Ok(home) = std::env::var("HOME") {
        let user = PathBuf::from(format!("{home}/.config/matrix/symbols.ttf"));
        if user.exists() {
            let fill = PathBuf::from(format!("{home}/.config/matrix/symbols-fill.ttf"));
            return (Some(user), fill.exists().then_some(fill));
        }
    }
    // R207: the symbol font from the AppImage (foreign desktop).
    if let Some(dir) = matrixkit_text::bundled_fonts_dir() {
        let main = dir.join("matrix/symbols-rounded.ttf");
        if main.exists() {
            let fill = dir.join("matrix/symbols-rounded-fill.ttf");
            return (Some(main), fill.exists().then_some(fill));
        }
    }
    let set = match mk::settings::read("symbol-set").as_deref() {
        Some("sharp") => "sharp",
        _ => "rounded",
    };
    for candidate in [set, "rounded"] {
        let main = PathBuf::from(format!("{SYMBOL_FONT_DIR}/symbols-{candidate}.ttf"));
        if main.exists() {
            let fill = PathBuf::from(format!("{SYMBOL_FONT_DIR}/symbols-{candidate}-fill.ttf"));
            return (Some(main), fill.exists().then_some(fill));
        }
    }
    let legacy = PathBuf::from(SYMBOL_FONT_LEGACY);
    if legacy.exists() {
        return (Some(legacy), None);
    }
    (None, None)
}

fn read_symbol_font(
    path: Option<std::path::PathBuf>,
    family: &'static std::sync::OnceLock<&'static str>,
) -> std::borrow::Cow<'static, [u8]> {
    let Some(bytes) = path.and_then(|p| std::fs::read(p).ok()) else {
        return std::borrow::Cow::Borrowed(&[]);
    };
    if let Some(name) = font_family_name(&bytes) {
        let _ = family.set(Box::leak(name.into_boxed_str()));
    }
    std::borrow::Cow::Owned(bytes)
}

/// The outline symbol font — apps pass it to `.font(...)`. Empty bytes
/// if no font resolves (symbols stay empty boxes then — on Matrix
/// systems the chain always ends in the shipped Rounded set).
pub fn symbol_font_main() -> std::borrow::Cow<'static, [u8]> {
    read_symbol_font(symbol_font_paths().0, &SYMBOL_FAMILY)
}

/// The filled variant (FILL axis pinned to 1) for `symbol_fill`.
pub fn symbol_font_fill() -> std::borrow::Cow<'static, [u8]> {
    read_symbol_font(symbol_font_paths().1, &SYMBOL_FILL_FAMILY)
}

/// The optical siblings of a resolved font file: `<stem>-o20.ttf` and
/// `<stem>-o48.ttf`, user dir (~/.config/matrix) first, then next to
/// the base file. Missing files degrade to the default master.
fn optical_paths(
    base: &std::path::Path,
) -> (Option<std::path::PathBuf>, Option<std::path::PathBuf>) {
    let sibling = |suffix: &str| -> Option<std::path::PathBuf> {
        let stem = base.file_stem()?.to_str()?;
        let name = format!("{stem}-{suffix}.ttf");
        if let Ok(home) = std::env::var("HOME") {
            let user = std::path::PathBuf::from(format!("{home}/.config/matrix/{name}"));
            if user.exists() {
                return Some(user);
            }
        }
        let s = base.with_file_name(&name);
        s.exists().then_some(s)
    };
    (sibling("o20"), sibling("o48"))
}

/// All symbol fonts for `fonts:`-style app builders — base masters
/// plus the baked optical instances (R123).
pub fn load_symbol_fonts() -> Vec<std::borrow::Cow<'static, [u8]>> {
    let (main, fill) = symbol_font_paths();
    let mut fonts = Vec::new();
    if let Some(main) = main {
        let (o20, o48) = optical_paths(&main);
        fonts.push(read_symbol_font(Some(main), &SYMBOL_FAMILY));
        fonts.push(read_symbol_font(o20, &SYMBOL_20_FAMILY));
        fonts.push(read_symbol_font(o48, &SYMBOL_48_FAMILY));
    }
    if let Some(fill) = fill {
        let (o20, o48) = optical_paths(&fill);
        fonts.push(read_symbol_font(Some(fill), &SYMBOL_FILL_FAMILY));
        fonts.push(read_symbol_font(o20, &SYMBOL_FILL_20_FAMILY));
        fonts.push(read_symbol_font(o48, &SYMBOL_FILL_48_FAMILY));
    }
    fonts.into_iter().filter(|f| !f.is_empty()).collect()
}

/// The family for a glyph rendered at `size` — the reference clamps
/// the optical axis to the point size, we bucket: ≤20 the small
/// master, ≥33 the large one, in between the default (opsz 24).
pub fn symbol_family_sized(size: f32) -> &'static str {
    if size <= 20.0 {
        if let Some(f) = SYMBOL_20_FAMILY.get() {
            return f;
        }
    } else if size >= 33.0 {
        if let Some(f) = SYMBOL_48_FAMILY.get() {
            return f;
        }
    }
    symbol_family()
}

/// Optical pick for the FILLED variant.
pub fn symbol_fill_family_sized(size: f32) -> &'static str {
    if size <= 20.0 {
        if let Some(f) = SYMBOL_FILL_20_FAMILY.get() {
            return f;
        }
    } else if size >= 33.0 {
        if let Some(f) = SYMBOL_FILL_48_FAMILY.get() {
            return f;
        }
    }
    symbol_fill_family()
}


/// The mono font bytes for `.font(...)` loading in app main — the
/// path lives EXACTLY ONCE in the kit.
pub fn load_mono_font() -> std::borrow::Cow<'static, [u8]> {
    let mut candidates = vec![std::path::PathBuf::from(
        "/usr/share/fonts/adwaita-mono-fonts/AdwaitaMono-Regular.ttf",
    )];
    // R207: the mono font from the AppImage (foreign desktop).
    if let Some(dir) = matrixkit_text::bundled_fonts_dir() {
        candidates.insert(0, dir.join("adwaita-mono-fonts/AdwaitaMono-Regular.ttf"));
    }
    for c in candidates {
        if let Ok(bytes) = std::fs::read(&c) {
            return std::borrow::Cow::Owned(bytes);
        }
    }
    std::borrow::Cow::Borrowed(&[])
}

/// Index-based focus: Tab wanders, Enter activates, the ring shows where.
/// (Homegrown, because iced buttons are not focusable; AccessKit follows
/// as soon as iced offers it upstream.)
#[derive(Debug, Clone, Copy)]
pub struct Focus {
    pos: Option<usize>,
    pub count: usize,
}

impl Focus {
    pub fn new(count: usize) -> Self {
        Self { pos: None, count }
    }
    pub fn set_count(&mut self, n: usize) {
        self.count = n;
        if let Some(p) = self.pos {
            if p >= n {
                self.pos = n.checked_sub(1);
            }
        }
    }
    pub fn onward(&mut self) {
        if self.count == 0 {
            return;
        }
        self.pos = Some(match self.pos {
            None => 0,
            Some(p) => (p + 1) % self.count,
        });
    }
    pub fn back(&mut self) {
        if self.count == 0 {
            return;
        }
        self.pos = Some(match self.pos {
            None | Some(0) => self.count - 1,
            Some(p) => p - 1,
        });
    }
    pub fn delete(&mut self) {
        self.pos = None;
    }
    pub fn is(&self, i: usize) -> bool {
        self.pos == Some(i)
    }
    pub fn current(&self) -> Option<usize> {
        self.pos
    }
}

/// Open Matrix Hilfe with a search term (reference HelpLink, round 11).
/// Looks for the binary in the usual places (dev setup → system → PATH).
/// Fire-and-forget: if the start fails, simply nothing happens —
/// a missing help button must never tear down the app.
pub fn help_open(term: &str) {
    let candidates = [
        std::env::var_os("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".local/bin/matrix-help")),
        Some(std::path::PathBuf::from("/usr/bin/matrix-help")),
    ];
    let program = candidates
        .into_iter()
        .flatten()
        .find(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("matrix-help"));
    let _ = mk::process::spawn_reaped(std::process::Command::new(program).arg("--suche").arg(term)); // round 51
}

/// Glass switches that menus and scene read together (R125).
/// CRAWL (R211 family, 02.11.2026 — Matrix Krabbler): the mechanics of a
/// walking creature, pure and testable. A SPINE of joints follows its head
/// (follow-the-leader, fixed gaps); LEGS hang on spine joints, find their
/// knee by two-bone inverse kinematics (`ik2`) and step when their foot
/// lags too far behind its rest point — in two alternating groups, so the
/// creature walks instead of sliding. No drawing here: apps lay dots,
/// boxes and words on top (`scene::lay_dotted_line`).
pub mod crawl {
    /// A point / direction in logical pixels.
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct V2 {
        pub x: f32,
        pub y: f32,
    }

    impl V2 {
        pub const fn new(x: f32, y: f32) -> V2 {
            V2 { x, y }
        }
        pub fn add(self, o: V2) -> V2 {
            V2::new(self.x + o.x, self.y + o.y)
        }
        pub fn sub(self, o: V2) -> V2 {
            V2::new(self.x - o.x, self.y - o.y)
        }
        pub fn mul(self, k: f32) -> V2 {
            V2::new(self.x * k, self.y * k)
        }
        pub fn len(self) -> f32 {
            (self.x * self.x + self.y * self.y).sqrt()
        }
        pub fn dist(self, o: V2) -> f32 {
            self.sub(o).len()
        }
        /// Unit length (a zero vector stays zero).
        pub fn unit(self) -> V2 {
            let l = self.len();
            if l < 1e-6 {
                V2::default()
            } else {
                self.mul(1.0 / l)
            }
        }
        /// Turned a quarter to the left (screen: y down).
        pub fn normal(self) -> V2 {
            V2::new(self.y, -self.x)
        }
        pub fn angle(self) -> f32 {
            self.y.atan2(self.x)
        }
        pub fn from_angle(a: f32) -> V2 {
            V2::new(a.cos(), a.sin())
        }
        pub fn lerp(self, o: V2, t: f32) -> V2 {
            self.add(o.sub(self).mul(t))
        }
    }

    /// Two-bone inverse kinematics: the knee between `hip` and `foot` with
    /// bones `a` (thigh) and `b` (shin). Of the two solutions the one
    /// farther from `away_from` (the body) — knees bend outward. A foot
    /// out of reach stretches the leg straight towards it.
    pub fn ik2(hip: V2, foot: V2, a: f32, b: f32, away_from: V2) -> V2 {
        let d = hip.dist(foot).clamp((a - b).abs() + 1e-3, a + b - 1e-3);
        let dir = foot.sub(hip).unit();
        let dir = if dir.len() == 0.0 { V2::new(1.0, 0.0) } else { dir };
        // Law of cosines: the angle at the hip between the line and the thigh.
        let cos = ((a * a + d * d - b * b) / (2.0 * a * d)).clamp(-1.0, 1.0);
        let alpha = cos.acos();
        let base = dir.angle();
        let k1 = hip.add(V2::from_angle(base + alpha).mul(a));
        let k2 = hip.add(V2::from_angle(base - alpha).mul(a));
        if k1.dist(away_from) >= k2.dist(away_from) {
            k1
        } else {
            k2
        }
    }

    /// The spine: `joints[0]` is the head.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Spine {
        pub joints: Vec<V2>,
        pub gap: f32,
    }

    impl Spine {
        /// `n` joints behind `head`, laid out against `heading` (radians).
        pub fn new(head: V2, heading: f32, n: usize, gap: f32) -> Spine {
            let back = V2::from_angle(heading).mul(-gap);
            Spine { joints: (0..n.max(1)).map(|i| head.add(back.mul(i as f32))).collect(), gap }
        }

        /// The head moved: every joint follows its predecessor at `gap`.
        pub fn follow(&mut self, head: V2) {
            self.joints[0] = head;
            for i in 1..self.joints.len() {
                let lead = self.joints[i - 1];
                let dir = self.joints[i].sub(lead).unit();
                let dir = if dir.len() == 0.0 { V2::new(-1.0, 0.0) } else { dir };
                self.joints[i] = lead.add(dir.mul(self.gap));
            }
        }

        /// Where joint `i` faces (unit): towards its predecessor; the head
        /// faces like the joint behind it.
        pub fn heading(&self, i: usize) -> V2 {
            let n = self.joints.len();
            if n < 2 {
                return V2::new(1.0, 0.0);
            }
            let i = i.min(n - 1);
            let (front, back) = if i == 0 { (self.joints[0], self.joints[1]) } else { (self.joints[i - 1], self.joints[i]) };
            let h = front.sub(back).unit();
            if h.len() == 0.0 {
                V2::new(1.0, 0.0)
            } else {
                h
            }
        }
    }

    /// One leg: where it hangs, how it reaches, where its foot stands.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Leg {
        /// The spine joint it hangs on.
        pub joint: usize,
        /// +1 left, -1 right.
        pub side: f32,
        /// Along the heading from the joint (hips and feet in front of or
        /// behind it — a spider's eight legs on one joint).
        pub forward: f32,
        /// Sideways reach of the rest point.
        pub reach: f32,
        pub thigh: f32,
        pub shin: f32,
        pub foot: V2,
        from: V2,
        to: V2,
        /// Step progress: 1 = planted.
        t: f32,
        /// This step's duration (shorter when the foot lagged far).
        dur: f32,
    }

    impl Leg {
        pub fn new(joint: usize, side: f32, forward: f32, reach: f32, thigh: f32, shin: f32) -> Leg {
            Leg { joint, side, forward, reach, thigh, shin, foot: V2::default(), from: V2::default(), to: V2::default(), t: 1.0, dur: 0.1 }
        }

        pub fn stepping(&self) -> bool {
            self.t < 1.0
        }

        /// Where the hip sits (`half` = half the body width there).
        pub fn hip(&self, spine: &Spine, half: f32) -> V2 {
            let h = spine.heading(self.joint);
            spine.joints[self.joint.min(spine.joints.len() - 1)].add(h.mul(self.forward * 0.35)).add(h.normal().mul(self.side * half))
        }

        /// Where the foot wants to stand.
        pub fn rest(&self, spine: &Spine) -> V2 {
            let h = spine.heading(self.joint);
            spine.joints[self.joint.min(spine.joints.len() - 1)].add(h.mul(self.forward)).add(h.normal().mul(self.side * self.reach))
        }

        /// Put the foot on its rest point (a new creature).
        pub fn plant(&mut self, spine: &Spine) {
            self.foot = self.rest(spine);
            self.t = 1.0;
        }
    }

    /// Move the legs one frame. A leg steps when its foot lags more than
    /// `stride` behind its rest point and no leg within `near` (rest points)
    /// is in the air — a wave down the body, the most lagging first. A foot
    /// more than 2.5 strides behind steps anyway, and quicker (`step_s`
    /// down to 45 %): no leg starves, also not on curves or in flight.
    /// (The first form with two global groups let one group starve on
    /// curves — feet stayed 175 px behind, 02.11.2026.) True = any leg moves.
    pub fn walk(legs: &mut [Leg], spine: &Spine, stride: f32, step_s: f32, dt: f32, near: f32) -> bool {
        for leg in legs.iter_mut() {
            if leg.stepping() {
                leg.t = (leg.t + dt / leg.dur.max(0.01)).min(1.0);
                let e = leg.t * leg.t * (3.0 - 2.0 * leg.t);
                leg.foot = leg.from.lerp(leg.to, e);
            }
        }
        let mut lagging: Vec<(f32, usize)> = legs
            .iter()
            .enumerate()
            .filter(|(_, l)| !l.stepping())
            .map(|(i, l)| (l.foot.dist(l.rest(spine)), i))
            .filter(|(lag, _)| *lag > stride)
            .collect();
        lagging.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (lag, i) in lagging {
            let rest = legs[i].rest(spine);
            let urgent = lag > stride * 2.5;
            let crowded = legs.iter().enumerate().any(|(k, o)| k != i && o.stepping() && o.rest(spine).dist(rest) < near);
            if crowded && !urgent {
                continue;
            }
            let h = spine.heading(legs[i].joint);
            let leg = &mut legs[i];
            leg.from = leg.foot;
            // Overshoot a little: the next step comes later.
            leg.to = rest.add(h.mul(stride * 0.5));
            leg.t = 0.0;
            leg.dur = step_s * (stride * 2.0 / lag).clamp(0.45, 1.0);
        }
        legs.iter().any(Leg::stepping)
    }

    /// A tiny deterministic random source (xorshift) — creatures wander
    /// without pulling a crate in.
    #[derive(Debug, Clone)]
    pub struct Dice(u64);

    impl Dice {
        pub fn new(seed: u64) -> Dice {
            Dice(seed.max(1))
        }
        /// 0.0 ..< 1.0
        pub fn roll(&mut self) -> f32 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            (x >> 40) as f32 / (1u64 << 24) as f32
        }
        pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
            lo + (hi - lo) * self.roll()
        }
    }

    /// Turn `current` (radians) towards `goal` by at most `max`.
    pub fn turn_towards(current: f32, goal: f32, max: f32) -> f32 {
        let mut d = goal - current;
        while d > std::f32::consts::PI {
            d -= std::f32::consts::TAU;
        }
        while d < -std::f32::consts::PI {
            d += std::f32::consts::TAU;
        }
        current + d.clamp(-max, max)
    }

    /// THE SPECIES (R211): four word creatures with their own appetite,
    /// body and gait — Matrix Krabbler's and every app's (`fam::Crawlers`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Species {
        /// A centipede, twelve joints; eats nouns (German capitalises them).
        Langleser,
        /// A spider, eight long legs; only long words.
        Weberin,
        /// A small beetle; the little words nobody notices.
        Filler,
        /// A firefly without legs; verbs ending in «-en».
        Funke,
    }

    pub const SPECIES: [Species; 4] = [Species::Langleser, Species::Weberin, Species::Filler, Species::Funke];

    impl Species {
        /// The key in words and files (`mk::crawlers`).
        pub fn key(self) -> &'static str {
            match self {
                Species::Langleser => "langleser",
                Species::Weberin => "weberin",
                Species::Filler => "fueller",
                Species::Funke => "funke",
            }
        }

        pub fn from_key(key: &str) -> Option<Species> {
            SPECIES.into_iter().find(|s| s.key() == key)
        }

        pub fn name(self) -> &'static str {
            match self {
                Species::Langleser => "Langleser",
                Species::Weberin => "Weberin",
                Species::Filler => "Füller",
                Species::Funke => "Funke",
            }
        }

        pub fn diet(self) -> &'static str {
            match self {
                Species::Langleser => "frisst Nomen",
                Species::Weberin => "frisst lange Wörter",
                Species::Filler => "frisst kleine Wörter",
                Species::Funke => "frisst Tätigkeiten",
            }
        }

        /// Does it eat this word? (letters only — `scene::letters`)
        pub fn likes(self, word: &str) -> bool {
            let n = word.chars().count();
            let first = word.chars().next().unwrap_or(' ');
            match self {
                Species::Langleser => first.is_uppercase() && (4..11).contains(&n),
                Species::Weberin => n >= 11,
                Species::Filler => (1..=3).contains(&n) && word.chars().all(char::is_alphabetic),
                Species::Funke => first.is_lowercase() && n >= 5 && word.ends_with("en"),
            }
        }

        /// Joints, gap between them, body box size.
        pub fn body(self) -> (usize, f32, f32) {
            match self {
                Species::Langleser => (12, 11.0, 9.0),
                Species::Weberin => (2, 13.0, 13.0),
                Species::Filler => (3, 8.0, 9.0),
                Species::Funke => (2, 7.0, 7.0),
            }
        }

        /// Speed (px/s) and turning (rad/s).
        pub fn pace(self) -> (f32, f32) {
            match self {
                Species::Langleser => (42.0, 2.2),
                Species::Weberin => (58.0, 3.4),
                Species::Filler => (34.0, 3.0),
                Species::Funke => (78.0, 2.6),
            }
        }

        /// How many words it carries.
        pub fn cargo(self) -> usize {
            match self {
                Species::Langleser => 6,
                Species::Weberin => 3,
                Species::Filler => 4,
                Species::Funke => 2,
            }
        }

        /// Stride, step duration and neighbour distance (`walk`).
        pub fn gait(self) -> (f32, f32, f32) {
            match self {
                Species::Langleser => (7.0, 0.11, 16.0),
                Species::Weberin => (14.0, 0.13, 10.0),
                Species::Filler => (6.0, 0.10, 8.0),
                Species::Funke => (1.0, 0.1, 1.0),
            }
        }

        pub fn legs(self) -> Vec<Leg> {
            match self {
                Species::Langleser => (1..12).flat_map(|j| [Leg::new(j, 1.0, 1.5, 15.0, 8.0, 10.0), Leg::new(j, -1.0, 1.5, 15.0, 8.0, 10.0)]).collect(),
                Species::Weberin => [13.0, 5.0, -3.0, -11.0].iter().flat_map(|f| [Leg::new(0, 1.0, *f, 30.0, 19.0, 22.0), Leg::new(0, -1.0, *f, 30.0, 19.0, 22.0)]).collect(),
                Species::Filler => [6.0, 0.0, -6.0].iter().flat_map(|f| [Leg::new(1, 1.0, *f, 12.0, 7.0, 8.0), Leg::new(1, -1.0, *f, 12.0, 7.0, 8.0)]).collect(),
                Species::Funke => Vec::new(),
            }
        }
    }

    /// A creature's body: spine, legs, heading, speed, wing beat. What it
    /// wants and carries is its owner's business (app or overlay).
    #[derive(Debug, Clone)]
    pub struct Critter {
        pub species: Species,
        pub spine: Spine,
        pub legs: Vec<Leg>,
        pub heading: f32,
        pub speed: f32,
        pub wing: f32,
    }

    impl Critter {
        pub fn new(species: Species, at: V2, heading: f32) -> Critter {
            let (n, gap, _) = species.body();
            let spine = Spine::new(at, heading, n, gap);
            let mut legs = species.legs();
            for l in &mut legs {
                l.plant(&spine);
            }
            Critter { species, spine, legs, heading, speed: 0.0, wing: 0.0 }
        }

        pub fn head(&self) -> V2 {
            self.spine.joints[0]
        }

        /// Touched at `p`?
        pub fn hit(&self, p: V2) -> bool {
            self.spine.joints.iter().any(|j| j.dist(p) < 18.0)
        }

        /// The joint the k-th carried word's thread starts at, and its heading.
        pub fn slot_joint(&self, k: usize) -> (usize, V2) {
            let n = self.spine.joints.len();
            let j = if n >= 6 { (2 + 2 * (k / 2) + k % 2).min(n - 1) } else { n - 1 };
            (j, self.spine.heading(j))
        }

        /// Where the k-th carried word hangs: away from the body on a
        /// thread — beside a long body, trailing behind a short one.
        pub fn slot(&self, k: usize) -> V2 {
            let (j, h) = self.slot_joint(k);
            let side = if k % 2 == 0 { 1.0 } else { -1.0 };
            if self.spine.joints.len() >= 6 {
                self.spine.joints[j].add(h.normal().mul(side * 34.0)).sub(h.mul(6.0))
            } else {
                self.spine.joints[j].sub(h.mul(26.0 + 18.0 * k as f32)).add(h.normal().mul(side * 12.0))
            }
        }

        /// One frame towards `target` at up to `want` px/s (eased), turning
        /// at its rate × `turn_boost`, with a `sway` on the aim; kept inside
        /// `lo`..`hi`. Legs walk, wings beat. True when it met the edge.
        #[allow(clippy::too_many_arguments)]
        pub fn steer(&mut self, target: V2, want: f32, dt: f32, sway: f32, turn_boost: f32, lo: V2, hi: V2) -> bool {
            let (_, turn) = self.species.pace();
            self.speed += (want - self.speed) * (dt * 4.0).min(1.0);
            let aim = target.sub(self.head()).angle() + sway;
            self.heading = turn_towards(self.heading, aim, turn * dt * turn_boost);
            let mut head = self.head().add(V2::from_angle(self.heading).mul(self.speed * dt));
            let edge = head.x < lo.x || head.x > hi.x || head.y < lo.y || head.y > hi.y;
            if edge {
                head = V2::new(head.x.clamp(lo.x, hi.x), head.y.clamp(lo.y, hi.y));
            }
            self.spine.follow(head);
            let (stride, step_s, near) = self.species.gait();
            walk(&mut self.legs, &self.spine, stride, step_s, dt, near);
            self.wing += dt * if want <= 0.0 { 2.0 } else { 26.0 };
            edge
        }

        /// Still moving (speed or a leg in the air)?
        pub fn moving(&self) -> bool {
            self.speed > 0.5 || self.legs.iter().any(Leg::stepping)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn ik2_keeps_bone_lengths_and_bends_outward() {
            let hip = V2::new(0.0, 0.0);
            let foot = V2::new(20.0, 0.0);
            let body = V2::new(10.0, -10.0);
            let knee = ik2(hip, foot, 15.0, 15.0, body);
            assert!((knee.dist(hip) - 15.0).abs() < 0.01);
            assert!((knee.dist(foot) - 15.0).abs() < 0.01);
            assert!(knee.y > 0.0, "the knee bends away from the body");
            // Out of reach: straight towards the foot.
            let far = ik2(hip, V2::new(100.0, 0.0), 10.0, 10.0, body);
            assert!((far.x - 10.0).abs() < 0.05 && far.y.abs() < 0.5);
        }

        #[test]
        fn spine_follows_at_its_gap() {
            let mut s = Spine::new(V2::new(0.0, 0.0), 0.0, 5, 10.0);
            assert_eq!(s.joints[4], V2::new(-40.0, 0.0));
            s.follow(V2::new(5.0, 3.0));
            for i in 1..5 {
                assert!((s.joints[i].dist(s.joints[i - 1]) - 10.0).abs() < 0.01);
            }
            assert!(s.heading(0).x > 0.0);
        }

        #[test]
        fn legs_keep_up_on_curves_and_in_flight() {
            // A centipede on a winding path, slow and fleeing.
            for speed in [42.0f32, 140.0] {
                let mut s = Spine::new(V2::new(0.0, 0.0), 0.0, 12, 11.0);
                let mut legs: Vec<Leg> = (1..12).flat_map(|j| [Leg::new(j, 1.0, 1.5, 15.0, 8.0, 10.0), Leg::new(j, -1.0, 1.5, 15.0, 8.0, 10.0)]).collect();
                for l in &mut legs {
                    l.plant(&s);
                }
                let dt = 1.0 / 45.0;
                let mut x = 0.0f32;
                let mut worst = 0.0f32;
                for _ in 0..(6.0 / dt) as usize {
                    x += speed * dt;
                    s.follow(V2::new(x, 40.0 * (x / 50.0).sin()));
                    walk(&mut legs, &s, 7.0, 0.11, dt, 16.0);
                    worst = legs.iter().map(|l| l.foot.dist(l.rest(&s))).fold(worst, f32::max);
                }
                assert!(worst < 30.0, "{speed} px/s: a foot lagged {worst} px");
            }
        }

        #[test]
        fn species_appetites_and_keys() {
            assert!(Species::Langleser.likes("Leinwand") && !Species::Langleser.likes("und"));
            assert!(Species::Weberin.likes("Bildschirmhelligkeit"));
            assert!(Species::Filler.likes("und") && !Species::Filler.likes("Haus"));
            assert!(Species::Funke.likes("leuchten") && !Species::Funke.likes("Garten"));
            for s in SPECIES {
                assert_eq!(Species::from_key(s.key()), Some(s));
            }
            let mut c = Critter::new(Species::Filler, V2::new(50.0, 50.0), 0.0);
            for _ in 0..60 {
                c.steer(V2::new(200.0, 50.0), 34.0, 1.0 / 30.0, 0.0, 1.0, V2::new(0.0, 0.0), V2::new(300.0, 300.0));
            }
            assert!(c.head().x > 60.0, "it walks towards its aim");
        }

        #[test]
        fn turning_takes_the_short_way() {
            let a = turn_towards(3.0, -3.0, 0.1);
            assert!(a > 3.0, "across ±π, not back through 0");
            assert!((turn_towards(0.0, 0.05, 0.1) - 0.05).abs() < 1e-6);
            let mut d = Dice::new(7);
            for _ in 0..100 {
                let r = d.roll();
                assert!((0.0..1.0).contains(&r));
            }
        }
    }
}

pub mod ui {
    use matrixkit_theme as mk;
    /// Liquid glass on? (settings key `glass`, default on) — surfaces
    /// riding on a compositor glass body thin their own paint.
    pub fn glass_on() -> bool {
        mk::settings::read("glass").as_deref() != Some("off")
    }

    /// Menus of THIS process ride on compositor glass cards (R125).
    /// Only card-window hosts (context service, Zentrale) opt in —
    /// in-surface menus (bar) keep their opaque shell.
    static MENU_GLASS: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);

    pub fn menus_on_glass() {
        MENU_GLASS.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Are menus currently lying on compositor glass? The scene path
    /// asks here before it paints a ground.
    pub fn menu_glass_on() -> bool {
        MENU_GLASS.load(std::sync::atomic::Ordering::Relaxed) && glass_on()
    }


    /// This process's raw menu-glass wish (without the setting).
    pub fn menu_glass_flag() -> bool {
        MENU_GLASS.load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub mod scene {
    use crate::buttons::{Role, Size, Style};
    use crate::symbol;
    use matrixkit_text::TextEngine;
    use matrixkit_theme as mk;
    use matrixkit_theme::scene::{Align, Node, Placed, Rect};

    /// Input state of a button — iced's button::Status, without iced.
    /// Disabled buttons (active = false) know no states; the app
    /// passes Idle for them.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Status {
        Idle,
        Hovered,
        Pressed,
    }

    /// A button in the scene tree: label + the three axes +
    /// active (false = disabled/inert, like `on: None` with
    /// mkw::button).
    pub struct Button {
        pub label: String,
        pub style: Style,
        pub role: Role,
        pub size: Size,
        pub active: bool,
        /// Lager 56: an IMPOSED width (iced's
        /// `.width(Length::Fill)`) — the button fills it and carries
        /// its text centred. None = it takes its size from the word.
        pub width: Option<f32>,
    }

    impl Default for Button {
        fn default() -> Self {
            Button {
                label: String::new(),
                style: Style::Prominent,
                role: Role::Normal,
                size: Size::Normal,
                active: true,
                width: None,
            }
        }
    }

    /// What the painter needs after layout: text, size, weight, font
    /// family, text colour. The anchor point comes from the laid-out
    /// scene (label_anchors / tile_label_anchors).
    /// `family` has been there since Lager 9: the toolbars speak the
    /// symbol font, everything else the house font.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Label {
        pub text: String,
        pub px: f32,
        pub weight: mk::typo::Weight,
        pub family: &'static str,
        pub color: mk::Rgba,
    }

    /// How many Placed ONE button lays: ground + text measure — always,
    /// even borderless (the ground is then transparent). The apps' role
    /// bookkeeping rests on this; a test pins it down.
    pub const PLACED_PER_BUTTON: usize = 2;

    /// THE MatrixKit button as a scene node. Literally the same colour
    /// worlds as mkw::button; whoever puts both side by side must see
    /// no difference.
    pub fn button(engine: &mut TextEngine, b: &Button, status: Status, p: mk::Palette) -> (Node, Label) {
        let (height, pad_x, typo) = match b.size {
            Size::Small => (26.0, mk::spacing::M, mk::typo::CAPTION),
            Size::Normal => (mk::control::BUTTON_HEIGHT, mk::spacing::L, mk::typo::BODY),
            Size::Large => (44.0, mk::spacing::XL, mk::typo::BODY),
        };
        // Colour worlds per style × role — all from the palette.
        let (ground, font) = match (b.style, b.role) {
            (Style::Prominent, Role::Normal) => (Some(p.primary), p.on_primary),
            (Style::Prominent, Role::Destructive) => (Some(p.error), p.on_primary),
            (Style::Tinted, Role::Normal) => {
                (Some(p.on_surface.over(p.surface_container_high, 0.08)), p.on_surface)
            }
            (Style::Tinted, Role::Destructive) => {
                (Some(p.error.over(p.surface_container_high, 0.12)), p.error)
            }
            (Style::Borderless, Role::Normal) => (None, p.primary),
            (Style::Borderless, Role::Destructive) => (None, p.error),
        };
        let font_final = if b.active { font } else { p.on_surface_variant };
        let ground = ground.map(|g| {
            let g = if b.active { g } else { g.over(p.surface_container_high, 0.5) };
            match status {
                Status::Hovered if b.active => font.over(g, mk::state_layer::HOVER),
                Status::Pressed if b.active => font.over(g, mk::state_layer::PRESSED),
                _ => g,
            }
        });
        // Borderless: the state layer only appears on contact.
        let ground = ground.or(match status {
            Status::Hovered if b.active => {
                Some(font.over(p.surface_container_high, mk::state_layer::HOVER))
            }
            Status::Pressed if b.active => {
                Some(font.over(p.surface_container_high, mk::state_layer::PRESSED))
            }
            _ => None,
        });
        // Always one ground Placed, transparent if need be: that way
        // the structure stays constant (PLACED_PER_BUTTON) and the
        // apps' bookkeeping simple.
        let ground = ground.unwrap_or(mk::Rgba { a: 0.0, ..p.surface });
        let (tw, th) = engine.measure_styled(&b.label, typo.size, typo.weight);
        let measure = Node::Quad {
            w: tw,
            h: th,
            color: mk::Rgba { a: 0.0, ..p.surface },
            radius: 0.0,
        };
        let node = match b.width {
            // Imposed width: the box carries the ground, the text
            // sits centred in it (Lager 56).
            Some(w) => Node::Slot {
                w: Some(w),
                h: Some(height),
                ax: Align::Center,
                ay: Align::Center,
                background: Some((ground, mk::CORNER_RADIUS)),
                child: Box::new(measure),
            },
            None => Node::Wrap {
                pad_x,
                pad_y: ((height - th) / 2.0).max(0.0),
                background: Some((ground, mk::CORNER_RADIUS)),
                child: Box::new(measure),
            },
        };
        (node, Label { text: b.label.clone(), px: typo.size, weight: typo.weight, family: matrixkit_text::FAMILY, color: font_final })
    }

    /// After layout: the anchor points of the labels. `first` is the
    /// Placed index of the FIRST button ground (behind the app's field
    /// backgrounds), then it continues in steps of PLACED_PER_BUTTON —
    /// each button's text measure lies directly behind its ground.
    pub fn label_anchors(placed: &[Placed], first: usize, count: usize) -> Vec<(f32, f32)> {
        (0..count)
            .map(|i| {
                let t = &placed[first + i * PLACED_PER_BUTTON + 1];
                (t.x, t.y)
            })
            .collect()
    }

    /// Lager 7: the TILE moves in — the shelf face of
    /// app_tile_auto/tile_chrome (R172: "the name leads, the origin
    /// qualifies") including R174j: one caption line per open window.
    /// Free floating, as wide as its words.
    pub struct Tile {
        pub name: String,
        pub maker: String,
        /// One caption per open window (R174j) — a full shelf reads
        /// as a window list.
        pub lines: Vec<String>,
    }

    /// Placed per tile: border + fill + name + origin + lines.
    /// The tile is the first resident of VARIABLE length — the role
    /// bookkeeping asks here instead of guessing.
    pub fn tile_placed_count(t: &Tile) -> usize {
        4 + t.lines.len()
    }

    /// The shelf tile as a scene node. Chrome like tile_chrome: glass
    /// fill (surface_container α 0.62), 1 px border (on_surface α
    /// 0.14), radius::LARGE, air S high / M wide — the same tokens as
    /// the iced version. The border is an outer wrap, the fill runs
    /// CONCENTRIC (radius::inner); so it lies UNDER the fill instead
    /// of beside it — invisible at α 0.14, noted as a reminder for the
    /// Placed contract.
    pub fn tile(engine: &mut TextEngine, t: &Tile, p: mk::Palette) -> (Node, Vec<Label>) {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let head = mk::typo::SUBTITLE;
        let cap = mk::typo::CAPTION;
        let mut labels = Vec::new();
        let mut children = Vec::new();
        let push = |text: &str, style: mk::typo::TextStyle, color: mk::Rgba,
                        engine: &mut TextEngine,
                        children: &mut Vec<Node>,
                        labels: &mut Vec<Label>| {
            let (w, h) = engine.measure_styled(text, style.px(), style.weight);
            children.push(Node::Quad { w, h, color: clear, radius: 0.0 });
            labels.push(Label {
                text: text.to_string(),
                px: style.px(),
                weight: style.weight,
                family: matrixkit_text::FAMILY,
                color,
            });
        };
        // The name leads, the origin qualifies (R172) …
        push(&t.name, head, p.on_surface, engine, &mut children, &mut labels);
        push(&t.maker, cap, p.on_surface_variant, engine, &mut children, &mut labels);
        // … and every open window names itself (R174j).
        for line in &t.lines {
            push(line, cap, p.on_surface_variant, engine, &mut children, &mut labels);
        }
        let column = Node::Column { gap: 1.0, padding: 0.0, background: None, children };
        let fill = Node::Wrap {
            pad_x: mk::spacing::M - 1.0,
            pad_y: mk::spacing::S - 1.0,
            background: Some((
                p.surface_container.with_alpha(0.62),
                mk::radius::inner(mk::radius::LARGE, 1.0),
            )),
            child: Box::new(column),
        };
        let node = Node::Wrap {
            pad_x: 1.0,
            pad_y: 1.0,
            background: Some((p.on_surface.with_alpha(0.14), mk::radius::LARGE)),
            child: Box::new(fill),
        };
        (node, labels)
    }

    /// Anchors of all labels of a tile: behind the two grounds
    /// (border, fill) lie the text measures in label order.
    pub fn tile_label_anchors(placed: &[Placed], first: usize, t: &Tile) -> Vec<(f32, f32)> {
        (0..2 + t.lines.len())
            .map(|i| {
                let q = &placed[first + 2 + i];
                (q.x, q.y)
            })
            .collect()
    }

    /// Lager 8: the FORMS move in. The switch first — the form control
    /// par excellence, entirely without text input: track 52×30 as a
    /// capsule, thumb 24 as a circle (radius::NORMAL), the thumb's
    /// position IS the state. Colour worlds literally like
    /// toggle_guarded; disabled dims via the 0.45 layer.
    pub fn toggle(on: bool, locked: bool, p: mk::Palette) -> Node {
        let mut track =
            if on { p.primary } else { p.on_surface.over(p.surface_container_high, 0.12) };
        let mut thumb = if on { p.on_primary } else { p.outline };
        if locked {
            track = track.over(p.surface_container_high, 0.45);
            thumb = thumb.over(p.surface_container_high, 0.45);
        }
        let pad = 3.0;
        let knob = Node::Quad { w: 24.0, h: 24.0, color: thumb, radius: mk::radius::NORMAL };
        let slack = Node::Quad {
            w: mk::control::TOGGLE_TRACK_W - 24.0 - 2.0 * pad,
            h: 0.0,
            color: mk::Rgba { a: 0.0, ..p.surface },
            radius: 0.0,
        };
        let children = if on { vec![slack, knob] } else { vec![knob, slack] };
        Node::Row {
            gap: 0.0,
            padding: pad,
            background: Some((track, mk::control::TOGGLE_TRACK_H / 2.0)),
            children,
        }
    }

    /// Placed per switch: track + thumb + spacer.
    pub const PLACED_PER_TOGGLE: usize = 3;

    /// Lager 52: the SLIDER family. The reminder from Lager 34
    /// ("formalise when the second customer comes") is overdue: the
    /// control centre and the player each built one — and they have
    /// already drifted apart (knob 18 vs 16 px). The family carries
    /// the iced grammar of `mkw::slider` including its TWO-CHANNEL
    /// state feedback that both hand builds lacked: while dragging the
    /// handle grows AND the remaining track brightens.
    pub struct SliderSpec {
        /// The value, normalised to 0..=1 — the app computes its range
        /// (it knows volume, brightness, seconds).
        pub value: f32,
        pub width: f32,
        pub status: Status,
        /// The ground the slider sits on: the remaining track blends
        /// over it (card = surface_container_high).
        pub ground: mk::Rgba,
    }

    /// Track, fill, knob, handle.
    pub const PLACED_PER_SLIDER: usize = 4;
    /// The track height (iced: rail width 4).
    pub const SLIDER_RAIL: f32 = 4.0;
    /// The invisible handle height: the hit target is more generous
    /// than what is painted — a finger does not hit 4 pixels.
    pub const SLIDER_GRIP: f32 = 24.0;

    pub fn slider(spec: &SliderSpec, p: mk::Palette) -> Node {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let v = spec.value.clamp(0.0, 1.0);
        // Two-channel (iced): the handle grows, the remaining track brightens.
        let (knob, rest_alpha) = match spec.status {
            Status::Pressed => (20.0, 0.24),
            Status::Hovered => (20.0, 0.15),
            Status::Idle => (18.0, 0.15),
        };
        let rail_r = mk::radius::capsule(SLIDER_RAIL);
        Node::Stack {
            children: vec![
                // The resting track.
                Node::Slot {
                    w: Some(spec.width),
                    h: Some(SLIDER_GRIP),
                    ax: Align::Start,
                    ay: Align::Center,
                    background: None,
                    child: Box::new(Node::Quad {
                        w: spec.width,
                        h: SLIDER_RAIL,
                        color: p.on_surface.over(spec.ground, rest_alpha),
                        radius: rail_r,
                    }),
                },
                // The fill up to the value — at least the cap.
                Node::Slot {
                    w: Some(spec.width),
                    h: Some(SLIDER_GRIP),
                    ax: Align::Start,
                    ay: Align::Center,
                    background: None,
                    child: Box::new(Node::Quad {
                        w: (spec.width * v).max(SLIDER_RAIL / 2.0),
                        h: SLIDER_RAIL,
                        color: p.primary,
                        radius: rail_r,
                    }),
                },
                // The knob sits ON the value. No spacer quad needed: a
                // box whose edge ends at the value carries the knob
                // right-aligned — its centre thus lands exactly on
                // width*v, and the clamping keeps it fully on the track
                // at the ends.
                Node::Slot {
                    w: Some((spec.width * v + knob / 2.0).clamp(knob, spec.width)),
                    h: Some(SLIDER_GRIP),
                    ax: Align::End,
                    ay: Align::Center,
                    background: None,
                    child: Box::new(Node::Quad {
                        w: knob,
                        h: knob,
                        color: p.on_primary.over(p.primary, 0.05),
                        radius: mk::radius::capsule(knob),
                    }),
                },
                // The HANDLE lies on top: what is hit here is the
                // slider — not the 4 pixel track beneath.
                Node::Quad { w: spec.width, h: SLIDER_GRIP, color: clear, radius: 0.0 },
            ],
        }
    }

    /// Lager 59: the MAGNIFIER (mkw::magnifier). It is not geometry
    /// but a transformation — and it was the reason `mk::scene` got a
    /// scale. The family pulls the TEXT along: scaling without taking
    /// the font sizes along would give a picture that does not match
    /// what was measured (the lesson of the tile weights from Lager 7).
    ///
    /// `MAGNIFIER` is the owner's measured value: 1.07 on hover, strong
    /// 1.18 for the traffic lights (family 2 of the MatrixUI directive).
    pub const MAGNIFIER: f32 = 1.07;
    pub const MAGNIFIER_STRONG: f32 = 1.18;

    /// Grows IN PLACE: the scale sits in a box of the original size,
    /// so the centre stays the centre — the element grows, it does
    /// not wander.
    pub fn magnify(
        engine: &mut TextEngine,
        node: Node,
        labels: &mut [Label],
        factor: f32,
        p: mk::Palette,
    ) -> Node {
        let _ = (engine, p);
        for l in labels.iter_mut() {
            l.px *= factor;
        }
        let mut probe = Vec::new();
        let (w, h) = matrixkit_theme::scene::lay_out(&node, 0.0, 0.0, &mut probe);
        Node::Slot {
            w: Some(w),
            h: Some(h),
            ax: Align::Center,
            ay: Align::Center,
            background: None,
            child: Box::new(Node::Scale { factor, child: Box::new(node) }),
        }
    }

    /// Lager 58: the PANEL shell (mkw::panel_shell) and the CONSOLE.
    ///
    /// The shell is the floating pill of the bar family: padding S,
    /// ground surface_container, 1 px border, and the radius
    /// CONCENTRIC to the content — inner radius plus distance to the
    /// edge (radius::SMALL + S). That is Apple's
    /// ContainerRelativeShape, here as the rule of the pill.
    ///
    /// HONEST REMINDER: the TRIPLE shadow of the iced version
    /// (panel::shadow_layers — three layers, because iced's single
    /// layer falls off "far too abruptly") is missing. The scene knows
    /// no shadow; it belongs in the renderer storey (reminder since
    /// Lager 11). On the canvas the compositor provides it today.
    pub fn panel_shell(child: Node, width: f32, p: mk::Palette) -> Node {
        let outer = mk::radius::SMALL + mk::spacing::S;
        Node::Slot {
            w: Some(width),
            h: None,
            ax: Align::Start,
            ay: Align::Start,
            background: Some((p.outline.with_alpha(0.25), outer)),
            child: Box::new(Node::Slot {
                w: Some(width - 2.0),
                h: None,
                ax: Align::Center,
                ay: Align::Center,
                background: Some((p.surface_container, mk::radius::inner(outer, 1.0))),
                child: Box::new(Node::Wrap {
                    // Padding S — NOT S − 1. The reference draws its
                    // border INSIDE the box boundary
                    // (container(content).padding(S) with border), the
                    // ring here lies around the outside; the deduction
                    // made the shell one logical pixel too narrow per
                    // side (measured on matrix-mitteilungen: padding 7.5
                    // instead of 9.5, 17.08.2026 — the same class as
                    // with the card family, Lager 75).
                    pad_x: mk::spacing::S,
                    pad_y: mk::spacing::S,
                    background: None,
                    child: Box::new(child),
                }),
            }),
        }
    }

    /// The CONSOLE (mkw::console): mono font in SMALL, second text
    /// level, padding M, ground surface with a 1 px border in
    /// radius::NORMAL — the place where output lives, not text.
    pub fn console(
        engine: &mut TextEngine,
        content: &str,
        width: f32,
        p: mk::Palette,
    ) -> (Node, Label) {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mono = crate::mono_family();
        let (_, h) = engine.measure_family(
            mono,
            content,
            mk::font_size::SMALL,
            mk::typo::Weight::Normal,
        );
        (
            Node::Slot {
                w: Some(width),
                h: None,
                ax: Align::Start,
                ay: Align::Start,
                background: Some((p.outline.with_alpha(0.25), mk::radius::NORMAL)),
                child: Box::new(Node::Slot {
                    w: Some(width - 2.0),
                    h: None,
                    ax: Align::Start,
                    ay: Align::Start,
                    background: Some((p.surface, mk::radius::inner(mk::radius::NORMAL, 1.0))),
                    child: Box::new(Node::Wrap {
                        pad_x: mk::spacing::M - 1.0,
                        pad_y: mk::spacing::M - 1.0,
                        background: None,
                        child: Box::new(Node::Quad {
                            w: width - 2.0 * mk::spacing::M,
                            h,
                            color: clear,
                            radius: 0.0,
                        }),
                    }),
                }),
            },
            Label {
                text: content.to_string(),
                px: mk::font_size::SMALL,
                weight: mk::typo::Weight::Normal,
                family: mono,
                color: p.text_level(2),
            },
        )
    }

    /// Lager 57: SCROLLING. The scene floor has been able to since
    /// Lager 29 (viewport) and Lager 41 (scroll spring) — what was
    /// missing are the SEAMS: how far may one scroll, where is it
    /// clipped, and the SCROLL GHOST that shows the position at the
    /// mouse instead of at the edge (GeistKit law 5).
    pub const GHOST_H: f32 = 56.0;
    pub const GHOST_W: f32 = 6.0;
    /// Distance of the ghost to the pointer and to the screen edge.
    pub const GHOST_GAP: f32 = 26.0;
    pub const GHOST_MARGIN: f32 = 8.0;

    /// How far one can scroll: never negative, and not at all when
    /// the content is shorter than the viewport.
    pub fn roll_max(content_h: f32, view_h: f32) -> f32 {
        (content_h - view_h).max(0.0)
    }

    /// The share of the visible in the whole — the thumb size.
    pub fn roll_portion(content_h: f32, view_h: f32) -> f32 {
        if content_h <= 0.0 {
            return 1.0;
        }
        (view_h / content_h).clamp(0.0, 1.0)
    }

    /// Clips everything from `first` on to the viewport — the ONE
    /// place where scrolling becomes visible (Lager 29: the box holds
    /// for rasteriser, hits AND diff alike).
    pub fn clip_from(scene: &mut [Placed], first: usize, view: matrixkit_theme::scene::Rect) {
        for q in scene.iter_mut().skip(first) {
            q.clip = Some(view);
        }
    }

    /// The SCROLL GHOST (mkw::scroll_ghost): 56×6 as a capsule, the
    /// thumb proportional (at least 12), the opacity from a spring —
    /// it fades away, it does not vanish (GeistKit law 6).
    pub fn scroll_ghost(portion: f32, offset: f32, opacity: f32, p: mk::Palette) -> Node {
        let a = opacity.clamp(0.0, 1.0);
        let thumb_h = (GHOST_H * portion.clamp(0.0, 1.0)).clamp(12.0, GHOST_H);
        let thumb_y = (GHOST_H - thumb_h) * offset.clamp(0.0, 1.0);
        let track = p.on_surface.over(p.surface_container, 0.22);
        let radius = mk::radius::capsule(GHOST_W);
        Node::Stack {
            children: vec![
                Node::Quad {
                    w: GHOST_W,
                    h: GHOST_H,
                    color: mk::Rgba { a: track.a * a, ..track },
                    radius,
                },
                // The thumb sits at its offset: a box that closes at
                // the thumb's end carries it at the bottom.
                Node::Slot {
                    w: Some(GHOST_W),
                    h: Some(thumb_y + thumb_h),
                    ax: Align::Start,
                    ay: Align::End,
                    background: None,
                    child: Box::new(Node::Quad {
                        w: GHOST_W,
                        h: thumb_h,
                        color: mk::Rgba { a: p.primary.a * a, ..p.primary },
                        radius,
                    }),
                },
            ],
        }
    }

    /// Where the ghost belongs: bottom right of the pointer, flipped
    /// to the left at the right edge, clamped up at the bottom edge —
    /// the same grammar as with the window ghost.
    pub fn ghost_place(mouse: (f32, f32), screen: (f32, f32)) -> (f32, f32) {
        let (mx, my) = mouse;
        let (sw, sh) = screen;
        let mut x = mx + GHOST_GAP;
        if x + GHOST_W + GHOST_MARGIN > sw {
            x = mx - GHOST_GAP - GHOST_W;
        }
        let mut y = my + GHOST_GAP;
        if y + GHOST_H + GHOST_MARGIN > sh {
            y = sh - GHOST_H - GHOST_MARGIN;
        }
        (x.max(GHOST_MARGIN), y.max(GHOST_MARGIN))
    }

    /// Lager 56: the REDACTION (mkw::redaction, Apple's
    /// `.redacted(.placeholder)`): a placeholder in the SHAPE of what
    /// is coming — the waiting shows what it will be instead of being empty.
    pub fn redaction(width: f32, height: f32, p: mk::Palette) -> Node {
        Node::Quad {
            w: width,
            h: height,
            color: p.on_surface.over(p.surface_container, 0.10),
            radius: height / 2.0,
        }
    }

    /// The TOOLTIP (mkw::tooltip, Apple's `.help()`): SMALL in
    /// on_surface, padding 4/8, card surface_container_high with a
    /// 1 px border, radius::SMALL. Where it appears is the app's
    /// decision — the family supplies the card.
    pub fn tooltip(engine: &mut TextEngine, hint: &str, p: mk::Palette) -> (Node, Label) {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let (w, h) = engine.measure_styled(hint, mk::font_size::SMALL, mk::typo::Weight::Normal);
        (
            Node::Wrap {
                pad_x: 1.0,
                pad_y: 1.0,
                background: Some((p.outline.with_alpha(0.25), mk::radius::SMALL)),
                child: Box::new(Node::Wrap {
                    pad_x: 8.0 - 1.0,
                    pad_y: 4.0 - 1.0,
                    background: Some((
                        p.surface_container_high,
                        mk::radius::inner(mk::radius::SMALL, 1.0),
                    )),
                    child: Box::new(Node::Quad { w, h, color: clear, radius: 0.0 }),
                }),
            },
            Label {
                text: hint.to_string(),
                px: mk::font_size::SMALL,
                weight: mk::typo::Weight::Normal,
                family: matrixkit_text::FAMILY,
                color: p.on_surface,
            },
        )
    }

    /// The CONFIRMATION DIALOG (mkw::confirmation). It carries a HIG
    /// rule IN THE LAYOUT: "the safe way is the comfortable one" —
    /// cancel comes first and tinted, the dangerous action last and
    /// prominent in the error role; both share the width. The veil
    /// closes on click (cancel), and the entrance comes from below:
    /// `progress` 0..1 moves the card up 80 px and fades in the veil.
    pub const DIALOG_WIDTH: f32 = 360.0;

    pub struct Confirmation {
        pub node: Node,
        pub labels: Vec<Label>,
        pub label_slots: Vec<usize>,
        /// The veil — a click on it means cancel.
        pub veil_slot: usize,
        pub cancel_slot: usize,
        pub confirm_slot: usize,
    }

    pub fn confirmation(
        engine: &mut TextEngine,
        title: &str,
        note: &str,
        action: &str,
        progress: f32,
        screen: (f32, f32),
        p: mk::Palette,
    ) -> Confirmation {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let symbols = crate::symbol_family();
        let f = progress.clamp(0.0, 1.0);
        let (sw, sh) = screen;
        let inner = DIALOG_WIDTH - 2.0 * mk::spacing::XL;
        let half = (inner - mk::spacing::S) / 2.0;

        let warn = symbol::WARNING.to_string();
        let (gw, gh) =
            engine.measure_family(symbols, &warn, mk::font_size::LARGE, mk::typo::Weight::Normal);
        let head = mk::typo::SUBTITLE;
        let (tw, th) = engine.measure_styled(title, head.px(), head.weight);
        let body = mk::typo::BODY;
        let (nw, nh) = engine.measure_styled(note, body.px(), body.weight);
        let head_h = gh.max(th);

        let cancel = Button {
            label: "Abbrechen".into(),
            style: Style::Tinted,
            width: Some(half),
            ..Default::default()
        };
        let confirm = Button {
            label: action.to_string(),
            style: Style::Prominent,
            role: Role::Destructive,
            width: Some(half),
            ..Default::default()
        };
        let (cancel_node, cancel_label) = button(engine, &cancel, Status::Idle, p);
        let (confirm_node, confirm_label) = button(engine, &confirm, Status::Idle, p);

        let center = |child: Node, w: f32, h: f32| Node::Slot {
            w: Some(w),
            h: Some(h),
            ax: Align::Start,
            ay: Align::Center,
            background: None,
            child: Box::new(child),
        };
        let card = Node::Wrap {
            pad_x: mk::spacing::XL,
            pad_y: mk::spacing::XL,
            background: Some((p.surface_container_high, mk::CORNER_RADIUS)),
            child: Box::new(Node::Column {
                gap: 0.0,
                padding: 0.0,
                background: None,
                children: vec![
                    Node::Row {
                        gap: mk::spacing::XS,
                        padding: 0.0,
                        background: None,
                        children: vec![
                            center(Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 }, gw, head_h),
                            center(Node::Quad { w: tw, h: th, color: clear, radius: 0.0 }, tw, head_h),
                        ],
                    },
                    Node::Quad { w: 0.0, h: mk::spacing::S, color: clear, radius: 0.0 },
                    Node::Quad { w: nw, h: nh, color: clear, radius: 0.0 },
                    Node::Quad { w: 0.0, h: mk::spacing::L, color: clear, radius: 0.0 },
                    Node::Row {
                        gap: mk::spacing::S,
                        padding: 0.0,
                        background: None,
                        children: vec![cancel_node, confirm_node],
                    },
                ],
            }),
        };
        // The entrance: the card starts 80 px lower and moves up —
        // as AIR above it, so the centring carries it.
        let rise = (80.0 * (1.0 - f)).max(0.0);
        let node = Node::Stack {
            children: vec![
                Node::Quad {
                    w: sw,
                    h: sh,
                    // One-step veil on the CPU (Lager 104); on the GPU
                    // painting it blends again.
                    color: mk::Rgba {
                        r: 0.08,
                        g: 0.08,
                        b: 0.08,
                        a: if std::env::var_os("MATRIX_GPU").is_some() {
                            0.55 * f
                        } else if f > 0.0 {
                            0.55
                        } else {
                            0.0
                        },
                    },
                    radius: 0.0,
                },
                Node::Slot {
                    w: Some(sw),
                    h: Some(sh),
                    ax: Align::Center,
                    ay: Align::Center,
                    background: None,
                    child: Box::new(Node::Column {
                        gap: 0.0,
                        padding: 0.0,
                        background: None,
                        children: vec![
                            Node::Quad { w: 0.0, h: 2.0 * rise, color: clear, radius: 0.0 },
                            card,
                        ],
                    }),
                },
            ],
        };
        // Placed: 0 veil · 1 entrance air · 2 card · 3 glyph
        // · 4 title · 5 air · 6 note · 7 air · 8/9 cancel ·
        // 10/11 the action.
        debug_assert_eq!(matrixkit_theme::scene::placed_count(&node), 12);
        Confirmation {
            node,
            labels: vec![
                Label {
                    text: warn,
                    px: mk::font_size::LARGE,
                    weight: mk::typo::Weight::Normal,
                    family: symbols,
                    color: p.error,
                },
                Label {
                    text: title.to_string(),
                    px: head.px(),
                    weight: head.weight,
                    family: matrixkit_text::FAMILY,
                    color: p.on_surface,
                },
                Label {
                    text: note.to_string(),
                    px: body.px(),
                    weight: body.weight,
                    family: matrixkit_text::FAMILY,
                    color: p.on_surface_variant,
                },
                cancel_label,
                confirm_label,
            ],
            label_slots: vec![3, 4, 6, 9, 11],
            veil_slot: 0,
            cancel_slot: 8,
            confirm_slot: 10,
        }
    }

    /// Lager 55: the LEVEL (mkw::level_bar) — the OSD's display that
    /// the dock had built for itself in Lager 42. Track 8 px as a
    /// capsule in on_surface over surface_container (0.12), the fill
    /// up to the share; `full` is the mute signal: full speaks
    /// primary, dimmed the variant text at 0.6.
    pub const LEVEL_H: f32 = 8.0;

    pub fn level_bar(portion: f32, full: bool, width: f32, p: mk::Palette) -> Node {
        let portion = portion.clamp(0.0, 1.0);
        let fill = if full { p.primary } else { p.on_surface_variant.with_alpha(0.6) };
        let radius = mk::radius::capsule(LEVEL_H);
        Node::Stack {
            children: vec![
                Node::Quad {
                    w: width,
                    h: LEVEL_H,
                    color: p.on_surface.over(p.surface_container, 0.12),
                    radius,
                },
                Node::Quad { w: width * portion, h: LEVEL_H, color: fill, radius },
            ],
        }
    }

    /// The BAR chart (ChartKind::Bars): one bar per value, growing
    /// from below — the values come normalised (0..=1), the app
    /// computes the axis. The line version needs no family: it is a
    /// curve (matrixkit-canvas, Lager 16) in the frame of the
    /// history card.
    pub fn bars(values: &[f32], w: f32, h: f32, p: mk::Palette) -> Node {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        if values.is_empty() {
            return Node::Quad { w, h, color: clear, radius: 0.0 };
        }
        let gap = 1.0f32;
        let bar_w = ((w - gap * (values.len() as f32 - 1.0)) / values.len() as f32).max(1.0);
        Node::Row {
            gap,
            padding: 0.0,
            background: None,
            children: values
                .iter()
                .map(|v| {
                    let bh = (h * v.clamp(0.0, 1.0)).max(1.0);
                    // Every bar hangs from the FLOOR: a full-height
                    // box, the quad aligned at its end.
                    Node::Slot {
                        w: Some(bar_w),
                        h: Some(h),
                        ax: Align::Start,
                        ay: Align::End,
                        background: None,
                        child: Box::new(Node::Quad {
                            w: bar_w,
                            h: bh,
                            color: p.primary,
                            radius: mk::radius::MINI.min(bar_w / 2.0),
                        }),
                    }
                })
                .collect(),
        }
    }

    /// Lager 54: the SMALL building blocks — badge, status capsule,
    /// card, empty state. Individually inconspicuous and therefore the
    /// first ones every app reinvents; their tokens come literally
    /// from the iced version.
    ///
    /// The badge (mkw::badge): number in SMALL, padding 1/7, ground
    /// on_surface over surface_container (0.08), capsule.
    pub fn badge(engine: &mut TextEngine, count: usize, p: mk::Palette) -> (Node, Label) {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let s = count.to_string();
        let (w, h) = engine.measure_styled(&s, mk::font_size::SMALL, mk::typo::Weight::Normal);
        (
            Node::Wrap {
                pad_x: 7.0,
                pad_y: 1.0,
                background: Some((
                    p.on_surface.over(p.surface_container, 0.08),
                    mk::radius::capsule(h + 2.0),
                )),
                child: Box::new(Node::Quad { w, h, color: clear, radius: 0.0 }),
            },
            Label {
                text: s,
                px: mk::font_size::SMALL,
                weight: mk::typo::Weight::Normal,
                family: matrixkit_text::FAMILY,
                color: p.on_surface_variant,
            },
        )
    }

    /// The status capsule (mkw::status_capsule): LABEL in the second
    /// text level, padding 3/10, thin fill, capsule. It is the ONLY
    /// element with a capsule radius outside the traffic lights —
    /// that is why the family police has a rule on it.
    pub fn status_capsule(engine: &mut TextEngine, content: &str, p: mk::Palette) -> (Node, Label) {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let style = mk::typo::LABEL;
        let (w, h) = engine.measure_styled(content, style.px(), style.weight);
        (
            Node::Wrap {
                pad_x: 10.0,
                pad_y: 3.0,
                background: Some((p.fill(mk::FillLevel::Thin), mk::radius::capsule(h + 6.0))),
                child: Box::new(Node::Quad { w, h, color: clear, radius: 0.0 }),
            },
            Label {
                text: content.to_string(),
                px: style.px(),
                weight: style.weight,
                family: matrixkit_text::FAMILY,
                color: p.text_level(2),
            },
        )
    }

    /// The card (mkw::card): padding M, large fill, radius::NORMAL,
    /// 1 px border in outline (0.25). The border lies as an outer
    /// ground UNDER the fill — the honest reminder from Lager 7: the
    /// Placed contract knows no real borders yet. With an opaque fill
    /// one sees exactly the ring, with a transparent one it shimmers
    /// through.
    ///
    /// The INNER padding is M − 1, not M: iced draws its border INSIDE
    /// the box boundary, the ring here lies around the outside.
    /// Without the deduction every card would be one logical pixel
    /// larger than the reference on every side and its content
    /// indented by one — measurable in the finger probe of Matrix Home
    /// (16.08.2026). This way the total size stays content + 2·M, and
    /// the content sits at M.
    pub fn card(child: Node, p: mk::Palette) -> Node {
        Node::Wrap {
            pad_x: 1.0,
            pad_y: 1.0,
            background: Some((p.outline.with_alpha(0.25), mk::radius::NORMAL)),
            child: Box::new(Node::Wrap {
                pad_x: mk::spacing::M - 1.0,
                pad_y: mk::spacing::M - 1.0,
                background: Some((
                    p.fill(mk::FillLevel::Large),
                    mk::radius::inner(mk::radius::NORMAL, 1.0),
                )),
                child: Box::new(child),
            }),
        }
    }

    /// The EMPTY STATE (mkw::empty_state, Apple's
    /// ContentUnavailableView): glyph 44 in variant text, air M, title
    /// LARGE, air XXS, explanation MEDIUM — all centred. An empty place
    /// explains itself instead of staying silent.
    pub fn empty_state(
        engine: &mut TextEngine,
        glyph: char,
        title: &str,
        description: &str,
        width: f32,
        p: mk::Palette,
    ) -> (Node, Vec<(Label, usize)>) {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let symbols = crate::symbol_family();
        let s = glyph.to_string();
        let (gw, gh) = engine.measure_family(symbols, &s, 44.0, mk::typo::Weight::Normal);
        let (tw, th) = engine.measure_styled(title, mk::font_size::LARGE, mk::typo::Weight::Normal);
        let middle = |w: f32, h: f32| Node::Slot {
            w: Some(width),
            h: Some(h),
            ax: Align::Center,
            ay: Align::Start,
            background: None,
            child: Box::new(Node::Quad { w, h, color: clear, radius: 0.0 }),
        };
        // The labels come as (label, SLOT) — otherwise the caller
        // counted the air quads too: in Matrix Schlüssel the title
        // anchored on the air and the description on the title measure
        // (finger probe 27.08.2026). And the description is laid out
        // LINE BY LINE: the engine paints single-line, iced's text()
        // breaks at \n — otherwise two sentences ran out of the window
        // as one line.
        let mut children = vec![
            middle(gw, gh),
            Node::Quad { w: 0.0, h: mk::spacing::M, color: clear, radius: 0.0 },
            middle(tw, th),
            Node::Quad { w: 0.0, h: mk::spacing::XXS, color: clear, radius: 0.0 },
        ];
        let mut labels = vec![
            (
                Label {
                    text: s,
                    px: 44.0,
                    weight: mk::typo::Weight::Normal,
                    family: symbols,
                    color: p.on_surface_variant,
                },
                0,
            ),
            (
                Label {
                    text: title.to_string(),
                    px: mk::font_size::LARGE,
                    weight: mk::typo::Weight::Normal,
                    family: matrixkit_text::FAMILY,
                    color: p.on_surface,
                },
                2,
            ),
        ];
        let mut slot = 4;
        for raw in description.split('\n') {
            for line in wrap_lines(engine, raw, mk::font_size::MEDIUM, mk::typo::Weight::Normal, width) {
                let (dw, dh) =
                    engine.measure_styled(&line, mk::font_size::MEDIUM, mk::typo::Weight::Normal);
                children.push(middle(dw, dh));
                labels.push((
                    Label {
                        text: line,
                        px: mk::font_size::MEDIUM,
                        weight: mk::typo::Weight::Normal,
                        family: matrixkit_text::FAMILY,
                        color: p.on_surface_variant,
                    },
                    slot,
                ));
                slot += 1;
            }
        }
        let node = Node::Column { gap: 0.0, padding: 0.0, background: None, children };
        (node, labels)
    }

    /// Lager 53: the MENU family. On 8.7. it was the owner's finding that
    /// the session menu, dock context, canvas context and in-app menus
    /// looked different — four hand builds became
    /// `mkw::ui::menu_family`. In First Light it happened again: the
    /// bar built its session menu (Lager 43), the context service its
    /// two (Lager 47/48). This family is the melting-down, with the
    /// tokens of the iced version.
    ///
    /// R122 grammar: A SEPARATOR IS AIR. Groups become their own island
    /// cards, M between them — the reference speaks in floating
    /// islands, not in lines.
    pub const MENU_WIDTH: f32 = 230.0;
    /// Row heights of the survey: with symbol 40, without 33, disabled 30.
    pub const MENU_ROW_GLYPH: f32 = 40.0;
    pub const MENU_ROW_PLAIN: f32 = 33.0;
    pub const MENU_ROW_LOCKED: f32 = 30.0;

    pub enum MenuEntry {
        Item {
            glyph: Option<char>,
            title: String,
            /// Shortcut on the right — ONLY for real binds (reference rule).
            shortcut: Option<String>,
            /// Living badge on the right ("1 Update").
            badge: Option<String>,
            /// Danger: the row speaks the error colour. Carries the
            /// bar's two-click confirmation ("Really?").
            danger: bool,
            /// Disabled = visible but grey: the reference NEVER hides —
            /// the menu keeps its shape, and one learns what would be
            /// available.
            enabled: bool,
        },
        Separator,
    }

    pub struct Menu {
        pub node: Node,
        pub labels: Vec<Label>,
        pub label_slots: Vec<usize>,
        /// PER INPUT ENTRY its hit target — None for separators and
        /// disabled ones. That way the list index meets the action
        /// index without the app counting along.
        pub item_slots: Vec<Option<usize>>,
    }

    /// `header` is the head card (iced: menu_family takes it as its
    /// own element) — it is its OWN island above the groups, as the
    /// R122 grammar demands.
    pub fn menu(
        engine: &mut TextEngine,
        header: Option<&str>,
        entries: &[MenuEntry],
        hovered: Option<usize>,
        p: mk::Palette,
    ) -> Menu {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        // On compositor GLASS the app contributes NOTHING to the
        // silhouette — no ground, no border (mkw::ui::menus_on_glass,
        // R125). The scene family never asked the switch and painted
        // its menu opaque where the reference shows glass (finger probe
        // matrix-kontext, 17.08.2026).
        let on_glass = crate::ui::menu_glass_on();
        let card = if on_glass { clear } else { p.surface_container_high };
        let symbols = crate::symbol_family();
        let body = mk::typo::BODY;
        let mut labels = Vec::new();
        let mut label_slots = Vec::new();
        let mut item_slots: Vec<Option<usize>> = Vec::new();
        let mut offset = 0usize;
        let mut groups: Vec<Node> = Vec::new();
        let mut rows: Vec<Node> = Vec::new();

        if let Some(title) = header {
            let head = mk::typo::HEADLINE;
            let (hw, hh) = engine.measure_styled(title, head.px(), head.weight);
            labels.push(Label {
                text: title.to_string(),
                px: head.px(),
                weight: head.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface,
            });
            // The card ground comes BEFORE the label (Lager 53).
            label_slots.push(offset + 1);
            offset += 2;
            groups.push(Node::Column {
                gap: 0.0,
                padding: mk::spacing::S,
                background: Some((card, mk::CORNER_RADIUS + 6.0)),
                // The head card takes the FULL menu width like the
                // groups below — an island narrower than its neighbours
                // looks like a mistake (finger probe 16.08.).
                children: vec![Node::Slot {
                    w: Some(MENU_WIDTH - 2.0 * mk::spacing::S),
                    h: Some(hh + 8.0),
                    ax: Align::Start,
                    ay: Align::Center,
                    background: None,
                    child: Box::new(Node::Wrap {
                        pad_x: mk::spacing::S,
                        pad_y: 0.0,
                        background: None,
                        child: Box::new(Node::Quad {
                            w: hw.min(MENU_WIDTH - 4.0 * mk::spacing::S),
                            h: hh,
                            color: clear,
                            radius: 0.0,
                        }),
                    }),
                }],
            });
        }
        let mut group_start = offset;

        for (i, entry) in entries.iter().enumerate() {
            match entry {
                MenuEntry::Separator => {
                    item_slots.push(None);
                    if !rows.is_empty() {
                        groups.push(Node::Column {
                            gap: mk::spacing::XXS,
                            padding: mk::spacing::S,
                            background: Some((card, mk::CORNER_RADIUS + 6.0)),
                            children: std::mem::take(&mut rows),
                        });
                        // The card ground is laid BEFORE the rows: all
                        // row slots of the group shift by one.
                        for slot in item_slots.iter_mut().flatten().filter(|s| **s >= group_start) {
                            *slot += 1;
                        }
                        for slot in label_slots.iter_mut().filter(|s| **s >= group_start) {
                            *slot += 1;
                        }
                        offset += 1;
                        group_start = offset;
                    }
                }
                MenuEntry::Item { glyph, title, shortcut, badge, danger, enabled } => {
                    let tint = if !*enabled {
                        p.outline
                    } else if *danger {
                        p.error
                    } else {
                        p.on_surface
                    };
                    let base_h = if !*enabled {
                        MENU_ROW_LOCKED
                    } else if glyph.is_some() {
                        MENU_ROW_GLYPH
                    } else {
                        MENU_ROW_PLAIN
                    };
                    let inner_w = MENU_WIDTH - 2.0 * mk::spacing::S;
                    // The row's ground: hit target AND hover layer in
                    // one — transparent at rest.
                    let ground = if hovered == Some(i) && *enabled {
                        p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER)
                    } else {
                        clear
                    };
                    item_slots.push(if *enabled { Some(offset) } else { None });
                    offset += 1;

                    let mut lead = Vec::new();
                    let mut lead_w = 0.0f32;
                    let mut lead_h = 0.0f32;
                    if let Some(z) = glyph {
                        let s = z.to_string();
                        let (gw, gh) = engine.measure_family(
                            symbols,
                            &s,
                            mk::font_size::MEDIUM,
                            mk::typo::Weight::Normal,
                        );
                        labels.push(Label {
                            text: s,
                            px: mk::font_size::MEDIUM,
                            weight: mk::typo::Weight::Normal,
                            family: symbols,
                            color: tint,
                        });
                        label_slots.push(offset);
                        offset += 1;
                        lead.push(Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 });
                        lead_w = gw + mk::spacing::S;
                        lead_h = gh;
                    }
                    // Lager 69: the TITLE WRAPS. Before, it stood in one
                    // line and stuck out of the card — which was not only
                    // ugly but inflated the measured menu width and thus
                    // shifted the whole panel (finger probe 16.08., 14
                    // logical pixels).
                    let right_room = match (shortcut.as_ref(), badge.as_ref()) {
                        (Some(s), _) | (_, Some(s)) => {
                            engine.measure_styled(s, body.px(), body.weight).0 + mk::spacing::S
                        }
                        _ => 0.0,
                    };
                    let room = (inner_w - lead_w - right_room).max(40.0);
                    let flowing = engine.wrap_styled(title, body.px(), body.weight, room);
                    let (tw, th) = engine.measure_styled(&flowing, body.px(), body.weight);
                    labels.push(Label {
                        text: flowing,
                        px: body.px(),
                        weight: body.weight,
                        family: matrixkit_text::FAMILY,
                        color: tint,
                    });
                    label_slots.push(offset);
                    offset += 1;
                    lead.push(Node::Quad { w: tw, h: th, color: clear, radius: 0.0 });

                    // Right: shortcut OR badge, both in the variant
                    // text — status, not emphasis.
                    let right = shortcut.as_ref().or(badge.as_ref()).map(|s| {
                        let (rw, rh) = engine.measure_styled(s, body.px(), body.weight);
                        labels.push(Label {
                            text: s.clone(),
                            px: body.px(),
                            weight: body.weight,
                            family: matrixkit_text::FAMILY,
                            color: p.on_surface_variant,
                        });
                        label_slots.push(offset);
                        offset += 1;
                        Node::Quad { w: rw, h: rh, color: clear, radius: 0.0 }
                    });

                    // A wrapped row grows — the height is the maximum
                    // of step and content.
                    let row_h = base_h.max(th.max(lead_h) + 2.0 * mk::spacing::XS);
                    let mut stack = vec![Node::Slot {
                        w: Some(inner_w),
                        h: Some(row_h),
                        ax: Align::Start,
                        ay: Align::Center,
                        background: None,
                        child: Box::new(Node::Row {
                            gap: mk::spacing::S,
                            padding: 0.0,
                            background: None,
                            children: lead,
                        }),
                    }];
                    if let Some(r) = right {
                        stack.push(Node::Slot {
                            w: Some(inner_w),
                            h: Some(row_h),
                            ax: Align::End,
                            ay: Align::Center,
                            background: None,
                            child: Box::new(r),
                        });
                    }
                    rows.push(Node::Slot {
                        w: Some(inner_w),
                        h: Some(row_h),
                        ax: Align::Start,
                        ay: Align::Start,
                        background: Some((ground, mk::CORNER_RADIUS)),
                        child: Box::new(Node::Stack { children: stack }),
                    });
                }
            }
        }
        if !rows.is_empty() {
            groups.push(Node::Column {
                gap: mk::spacing::XXS,
                padding: mk::spacing::S,
                background: Some((card, mk::CORNER_RADIUS + 6.0)),
                children: rows,
            });
            for slot in item_slots.iter_mut().flatten().filter(|s| **s >= group_start) {
                *slot += 1;
            }
            for slot in label_slots.iter_mut().filter(|s| **s >= group_start) {
                *slot += 1;
            }
            offset += 1;
        }
        let node = Node::Column {
            gap: mk::spacing::M,
            padding: 0.0,
            background: None,
            children: groups,
        };
        debug_assert_eq!(matrixkit_theme::scene::placed_count(&node), offset);
        Menu { node, labels, label_slots, item_slots }
    }

    /// Track position → value (0..=1). Both hand builds carried this
    /// clamping themselves; now there is ONE.
    pub fn slider_at(x: f32, track_x: f32, track_w: f32) -> f32 {
        if track_w <= 0.0 {
            return 0.0;
        }
        ((x - track_x) / track_w).clamp(0.0, 1.0)
    }

    /// A form row: title (+ description below), the switch on the
    /// right — mkw::row_toggle, only the floor changes.
    pub struct FormRow {
        pub title: String,
        pub description: Option<String>,
        pub on: bool,
        pub locked: bool,
    }

    /// The built form IS a built list (Lager 51b): the name stays so
    /// no caller breaks, the bookkeeping is the same — and the section
    /// gets row_slots in the process, i.e. each row its own hit target.
    pub type Form = List;

    /// The settings section (mkw::section + row_toggle) — since Lager
    /// 51b a SPECIAL CASE of the list family: a list whose rows all
    /// carry a switch. The name and the contract stay, the face too
    /// (LABEL title with S indent, card surface_container_high, 1 px
    /// hairlines with M indent, row air S+2 high / M wide) — only the
    /// anatomy now lives in ONE place. "Whoever needs the pattern
    /// calls the family, never the hand build."
    pub fn form_section(
        engine: &mut TextEngine,
        title: &str,
        rows: &[FormRow],
        width: f32,
        p: mk::Palette,
    ) -> Form {
        let rows: Vec<ListRow> = rows
            .iter()
            .map(|r| ListRow {
                title: r.title.clone(),
                description: r.description.clone(),
                leading: None,
                trailing: Trailing::Toggle { on: r.on, locked: r.locked },
            })
            .collect();
        list(engine, title, &rows, width, p)
    }

    /// Lager 51: the LIST family — the biggest gap of the survey.
    /// It carries the iced grammar of `mkw::row_item` literally:
    /// leading symbol · text column (title BODY, subline CAPTION in
    /// variant text) · trailer, vertically centred, padding M/S+2 —
    /// and the text column TAKES the space (Length::Fill), which since
    /// Lager 50 the box lays out instead of computing.
    ///
    /// The trailer decides which iced row it is: nothing (row_item),
    /// a value (row_value), the chevron (row_nav) or a switch
    /// (row_toggle).
    pub enum Trailing {
        None,
        Value(String),
        Chevron,
        Toggle { on: bool, locked: bool },
        /// Lager 60: the SELECTION (mkw::row_select, iced's pick_list).
        /// The closed state is a trigger — value MEDIUM plus ⇕ in a
        /// capsule with a state layer; the open state IS the menu
        /// (fam::menu). Two families, one control.
        Select { value: String, hovered: bool },
    }

    pub struct ListRow {
        pub title: String,
        pub description: Option<String>,
        /// Leading symbol from the symbol font.
        pub leading: Option<char>,
        pub trailing: Trailing,
    }

    /// The built list including bookkeeping.
    pub struct List {
        pub node: Node,
        pub labels: Vec<Label>,
        pub label_slots: Vec<usize>,
        /// Per row ITS hit target: the transparent ground across the
        /// full row width. The ROW is the target (R98 metric), not the
        /// text in it — that is why it carries it itself.
        pub row_slots: Vec<usize>,
        /// Per switch row its track (step PLACED_PER_TOGGLE).
        pub toggle_slots: Vec<usize>,
        /// Per selection row its trigger (Lager 60) — a click on it
        /// opens the menu.
        pub select_slots: Vec<usize>,
    }

    /// The SHELL of a section (mkw::section): title in LABEL, below it
    /// ONE card whose rows are separated by hairlines — not one pill
    /// per row.
    ///
    /// The row CONTENT stays free. `list()` fills it with the list row;
    /// whoever needs a row of their own (matrix-codes carries ring,
    /// name, code and copy symbol) builds only the row and leaves
    /// title, card and separation to the family. On 16.08.2026 the
    /// finger probe found hand-built pills in three apps where the
    /// reference shows a card.
    ///
    /// `rows` returns the Placed indices of the row STARTS — whoever
    /// must anchor something in their row computes from there.
    pub struct Section {
        pub node: Node,
        pub title: Option<Label>,
        pub rows: Vec<usize>,
    }

    pub fn section(
        engine: &mut TextEngine,
        title: &str,
        rows: Vec<Node>,
        width: f32,
        p: mk::Palette,
    ) -> Section {
        use matrixkit_theme::scene::placed_count;
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mut outer = Vec::new();
        let mut label = None;
        let mut offset = 0usize;
        if !title.is_empty() {
            let style = mk::typo::LABEL;
            let (tw, th) = engine.measure_styled(title, style.px(), style.weight);
            label = Some(Label {
                text: title.to_string(),
                px: style.px(),
                weight: style.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface_variant,
            });
            outer.push(Node::Wrap {
                pad_x: mk::spacing::S,
                pad_y: 0.0,
                background: None,
                child: Box::new(Node::Quad { w: tw, h: th, color: clear, radius: 0.0 }),
            });
            offset += 1;
        }
        // The card lays its ground BEFORE its children (Column
        // insert(slot)) — that is why it counts here, before the loop.
        offset += 1;
        let mut card_children = Vec::new();
        let mut row_slots = Vec::new();
        for (i, row) in rows.into_iter().enumerate() {
            if i > 0 {
                card_children.push(Node::Row {
                    gap: 0.0,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Quad { w: mk::spacing::M, h: 0.0, color: clear, radius: 0.0 },
                        Node::Quad {
                            w: width - mk::spacing::M,
                            h: 1.0,
                            color: p.outline.over(p.surface_container_high, 0.18),
                            radius: 0.0,
                        },
                    ],
                });
                offset += 2;
            }
            row_slots.push(offset);
            offset += placed_count(&row);
            card_children.push(row);
        }
        let card = Node::Column {
            gap: 0.0,
            padding: 0.0,
            background: Some((p.surface_container_high, mk::CORNER_RADIUS)),
            children: card_children,
        };
        outer.push(card);
        let node = Node::Column {
            gap: mk::spacing::XS,
            padding: 0.0,
            background: None,
            children: outer,
        };
        debug_assert_eq!(placed_count(&node), offset);
        Section { node, title: label, rows: row_slots }
    }

    pub fn list(
        engine: &mut TextEngine,
        title: &str,
        rows: &[ListRow],
        width: f32,
        p: mk::Palette,
    ) -> List {
        list_with(engine, title, rows, width, &ListOptions::default(), p)
    }

    /// Lager 108: SELECTION and TREE of the list (mkw::row_select,
    /// mk::list::INDENT). `selected` lifts a row onto
    /// primary_container; `depths` indents rows by INDENT per level
    /// (folder trees in Dateien, chapters in Lernraum).
    #[derive(Default, Clone)]
    pub struct ListOptions<'a> {
        pub selected: Option<usize>,
        pub depths: &'a [u8],
    }

    pub fn list_with(
        engine: &mut TextEngine,
        title: &str,
        rows: &[ListRow],
        width: f32,
        opts: &ListOptions,
        p: mk::Palette,
    ) -> List {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mut labels = Vec::new();
        let mut label_slots = Vec::new();
        let mut row_slots = Vec::new();
        let mut toggle_slots = Vec::new();
        let mut select_slots = Vec::new();
        let mut offset = 0usize;
        let mut outer_children = Vec::new();

        if !title.is_empty() {
            let style = mk::typo::LABEL;
            let (tw, th) = engine.measure_styled(title, style.px(), style.weight);
            labels.push(Label {
                text: title.to_string(),
                px: style.px(),
                weight: style.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface_variant,
            });
            label_slots.push(offset);
            offset += 1;
            outer_children.push(Node::Wrap {
                pad_x: mk::spacing::S,
                pad_y: 0.0,
                background: None,
                child: Box::new(Node::Quad { w: tw, h: th, color: clear, radius: 0.0 }),
            });
        }

        offset += 1; // the card ground
        let symbols = crate::symbol_family();
        let mut card_children = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            if i > 0 {
                card_children.push(Node::Row {
                    gap: 0.0,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Quad { w: mk::spacing::M, h: 0.0, color: clear, radius: 0.0 },
                        Node::Quad {
                            w: width - mk::spacing::M,
                            h: 1.0,
                            color: p.outline.over(p.surface_container_high, 0.18),
                            radius: 0.0,
                        },
                    ],
                });
                offset += 2;
            }
            // The row's ground: invisible but fully there — it is the
            // hit target and is laid FIRST.
            row_slots.push(offset);
            offset += 1;

            let body = mk::typo::BODY;
            let cap = mk::typo::CAPTION;
            let mut lead_children = Vec::new();
            // Tree indent (Lager 108): a spacer BEFORE the symbol, INDENT per level.
            let depth = opts.depths.get(i).copied().unwrap_or(0);
            if depth > 0 {
                lead_children.push(Node::Quad {
                    w: depth as f32 * mk::list::INDENT,
                    h: 0.0,
                    color: clear,
                    radius: 0.0,
                });
                offset += 1;
            }

            if let Some(glyph) = row.leading {
                let s = glyph.to_string();
                let (gw, gh) =
                    engine.measure_family(symbols, &s, mk::font_size::MEDIUM, mk::typo::Weight::Normal);
                labels.push(Label {
                    text: s,
                    px: mk::font_size::MEDIUM,
                    weight: mk::typo::Weight::Normal,
                    family: symbols,
                    color: p.on_surface_variant,
                });
                label_slots.push(offset);
                offset += 1;
                lead_children.push(Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 });
            }

            let (tw, th) = engine.measure_styled(&row.title, body.px(), body.weight);
            labels.push(Label {
                text: row.title.clone(),
                px: body.px(),
                weight: body.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface,
            });
            label_slots.push(offset);
            offset += 1;
            let mut text_children = vec![Node::Quad { w: tw, h: th, color: clear, radius: 0.0 }];
            let mut text_h = th;
            if let Some(d) = &row.description {
                let (dw, dh) = engine.measure_styled(d, cap.px(), cap.weight);
                labels.push(Label {
                    text: d.clone(),
                    px: cap.px(),
                    weight: cap.weight,
                    family: matrixkit_text::FAMILY,
                    color: p.on_surface_variant,
                });
                label_slots.push(offset);
                offset += 1;
                text_children.push(Node::Quad { w: dw, h: dh, color: clear, radius: 0.0 });
                text_h += 2.0 + dh;
            }
            lead_children.push(Node::Column {
                gap: 2.0,
                padding: 0.0,
                background: None,
                children: text_children,
            });

            // The trailer — it determines which iced row this is.
            let (trailing_node, trailing_h) = match &row.trailing {
                Trailing::None => (None, 0.0),
                Trailing::Value(v) => {
                    let (vw, vh) = engine.measure_styled(v, body.px(), body.weight);
                    labels.push(Label {
                        text: v.clone(),
                        px: body.px(),
                        weight: body.weight,
                        family: matrixkit_text::FAMILY,
                        color: p.on_surface_variant,
                    });
                    label_slots.push(offset);
                    offset += 1;
                    (Some(Node::Quad { w: vw, h: vh, color: clear, radius: 0.0 }), vh)
                }
                Trailing::Chevron => {
                    let s = symbol::CHEVRON_RIGHT.to_string();
                    let (cw, ch) = engine.measure_family(
                        symbols,
                        &s,
                        mk::font_size::MEDIUM,
                        mk::typo::Weight::Normal,
                    );
                    labels.push(Label {
                        text: s,
                        px: mk::font_size::MEDIUM,
                        weight: mk::typo::Weight::Normal,
                        family: symbols,
                        color: p.on_surface_variant,
                    });
                    label_slots.push(offset);
                    offset += 1;
                    (Some(Node::Quad { w: cw, h: ch, color: clear, radius: 0.0 }), ch)
                }
                Trailing::Toggle { on, locked } => {
                    toggle_slots.push(offset);
                    offset += PLACED_PER_TOGGLE;
                    (Some(toggle(*on, *locked, p)), mk::control::TOGGLE_TRACK_H)
                }
                Trailing::Select { value, hovered } => {
                    // The trigger: value + ⇕ in a capsule that shows
                    // its state layer on hover.
                    let ground = if *hovered {
                        p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER)
                    } else {
                        clear
                    };
                    select_slots.push(offset);
                    offset += 1;
                    let (vw, vh) =
                        engine.measure_styled(value, mk::font_size::MEDIUM, mk::typo::Weight::Normal);
                    labels.push(Label {
                        text: value.clone(),
                        px: mk::font_size::MEDIUM,
                        weight: mk::typo::Weight::Normal,
                        family: matrixkit_text::FAMILY,
                        color: p.on_surface,
                    });
                    label_slots.push(offset);
                    offset += 1;
                    let unfold = symbol::UNFOLD_MORE.to_string();
                    let (uw, uh) = engine.measure_family(
                        symbols,
                        &unfold,
                        mk::font_size::SMALL,
                        mk::typo::Weight::Normal,
                    );
                    labels.push(Label {
                        text: unfold,
                        px: mk::font_size::SMALL,
                        weight: mk::typo::Weight::Normal,
                        family: symbols,
                        color: p.on_surface_variant,
                    });
                    label_slots.push(offset);
                    offset += 1;
                    let h = vh.max(uh);
                    (
                        Some(Node::Wrap {
                            pad_x: mk::spacing::S,
                            pad_y: 2.0,
                            background: Some((ground, mk::radius::SMALL)),
                            child: Box::new(Node::Row {
                                gap: mk::spacing::XS,
                                padding: 0.0,
                                background: None,
                                children: vec![
                                    Node::Quad { w: vw, h: vh, color: clear, radius: 0.0 },
                                    Node::Quad { w: uw, h: uh, color: clear, radius: 0.0 },
                                ],
                            }),
                        }),
                        h + 4.0,
                    )
                }
            };

            let content_h = text_h.max(trailing_h);
            let inner_w = width - 2.0 * mk::spacing::M;
            let lead = Node::Row {
                gap: mk::spacing::M,
                padding: 0.0,
                background: None,
                children: lead_children,
            };
            let mut stack_children = vec![Node::Slot {
                w: Some(inner_w),
                h: Some(content_h),
                ax: Align::Start,
                ay: Align::Center,
                background: None,
                child: Box::new(lead),
            }];
            if let Some(t) = trailing_node {
                stack_children.push(Node::Slot {
                    w: Some(inner_w),
                    h: Some(content_h),
                    ax: Align::End,
                    ay: Align::Center,
                    background: None,
                    child: Box::new(t),
                });
            }
            let ground = if opts.selected == Some(i) {
                p.primary_container.with_alpha(0.55)
            } else {
                clear
            };
            card_children.push(Node::Slot {
                w: Some(width),
                h: None,
                ax: Align::Start,
                ay: Align::Start,
                background: Some((ground, 0.0)),
                child: Box::new(Node::Wrap {
                    pad_x: mk::spacing::M,
                    pad_y: mk::spacing::S + 2.0,
                    background: None,
                    child: Box::new(Node::Stack { children: stack_children }),
                }),
            });
        }
        let card = Node::Column {
            gap: 0.0,
            padding: 0.0,
            background: Some((p.surface_container_high, mk::CORNER_RADIUS)),
            children: card_children,
        };
        outer_children.push(card);
        let node = Node::Column {
            gap: mk::spacing::XS,
            padding: 0.0,
            background: None,
            children: outer_children,
        };
        debug_assert_eq!(matrixkit_theme::scene::placed_count(&node), offset);
        List { node, labels, label_slots, row_slots, toggle_slots, select_slots }
    }

    /// Lager 9: the TOOLBARS move in — and with them the scene speaks
    /// the SYMBOL FONT for the first time. This arming stores it in the
    /// text engine (the same bytes the iced path loads too;
    /// symbol_family() reads the name from the TTF name table). Once
    /// per engine, before the first bar.
    pub fn arm_symbols(engine: &mut TextEngine) {
        engine.load_font(crate::symbol_font_main().into_owned());
    }

    /// The FOCUS RING on scene floor (Lager 31): the ring is LAYOUT,
    /// not painting (Lager 3 doctrine) — a 2 px mantle in primary
    /// around the child; the child carries its own ground. Without
    /// focus the tree stays unchanged (one Placed MORE only with the
    /// ring — role bookkeeping over ranges stays valid, fixed anchor
    /// offsets shift by one).
    pub fn ring(child: Node, focused: bool, radius: f32, p: mk::Palette) -> Node {
        if !focused {
            return child;
        }
        Node::Wrap {
            pad_x: 2.0,
            pad_y: 2.0,
            background: Some((p.primary, radius + 2.0)),
            child: Box::new(child),
        }
    }

    /// The tool button (mkw::tool_button): a symbol, 18 px, in
    /// on_surface — or outline when it is muted (reference: muted
    /// chevrons stay VISIBLE). Rests without a ground; only contact
    /// lays the panel layer (0.10, like panel::button_style) under
    /// radius::SMALL.
    pub fn tool_button(
        engine: &mut TextEngine,
        glyph: char,
        active: bool,
        status: Status,
        p: mk::Palette,
    ) -> (Node, Label) {
        let tint = if active { p.on_surface } else { p.outline };
        let ground = match status {
            Status::Hovered | Status::Pressed if active => {
                p.on_surface.over(p.surface_container, 0.10)
            }
            _ => mk::Rgba { a: 0.0, ..p.surface },
        };
        let s = glyph.to_string();
        let family = crate::symbol_family();
        let (gw, gh) = engine.measure_family(family, &s, 18.0, mk::typo::Weight::Normal);
        let node = Node::Wrap {
            pad_x: 6.0,
            pad_y: 4.0,
            background: Some((ground, mk::radius::SMALL)),
            child: Box::new(Node::Quad {
                w: gw,
                h: gh,
                color: mk::Rgba { a: 0.0, ..p.surface },
                radius: 0.0,
            }),
        };
        (node, Label { text: s, px: 18.0, weight: mk::typo::Weight::Normal, family, color: tint })
    }

    /// Placed per tool button: ground + symbol measure.
    pub const PLACED_PER_TOOL: usize = 2;

    /// Lager 19: the INPUT FIELD — the start of the hardest piece.
    /// mkw::input_field on scene floor: ground on_surface over
    /// surface_container (0.08), border 1 px (outline over
    /// surface_container, 0.35) — 2 px primary when FOCUSED —,
    /// radius::SMALL, text in font_size::MEDIUM. The VALUE belongs to
    /// the app (controlled field, as with iced); the caret is a 1.5 px
    /// Placed at the measured prefix. Blinking comes with the 500 ms
    /// pulse, selection and IME are the documented keystone.
    pub struct FieldBuild {
        pub node: Node,
        /// Value or placeholder, already coloured.
        pub label: Label,
        /// The leading glyph as a string (symbol font), if the field
        /// carries one — it lies on Placed 2.
        pub leading: Option<String>,
    }

    /// What a field shows (Lager 20: the keystone). The VALUE belongs
    /// to the app; `anchor` is the selection anchor (shift ranges),
    /// `blink` the caret's blink phase (500 ms beat).
    pub struct FieldSpec<'a> {
        pub value: &'a str,
        pub cursor: usize,
        pub anchor: Option<usize>,
        pub focused: bool,
        pub blink: bool,
        pub placeholder: &'a str,
        /// A glyph INSIDE the pill, before the text — the search
        /// field's magnifying glass. The reference (mkw::search_field)
        /// puts it inside the border, not in front of it; a field
        /// without a glyph stays at seven Placed, with a glyph it is
        /// nine (finger probe Matrix Hilfe, 16.08.2026).
        pub leading: Option<char>,
    }

    /// Placed per field: border, ground, selection lead-in, SELECTION,
    /// prefix spacer, caret, width spacer — always seven, the
    /// selection is zero wide without an anchor. The label anchors on
    /// the GROUND (`first + 1`), its x is the prefix spacer
    /// (`first + 4`).
    pub const PLACED_PER_FIELD: usize = 7;
    /// With a leading glyph, glyph and gap are added.
    pub const PLACED_PER_FIELD_LEADING: usize = 9;

    /// How many Placed this field lays — the caller computes its
    /// anchors from here, never from a guessed number.
    pub fn placed_per_field(spec: &FieldSpec) -> usize {
        if spec.leading.is_some() {
            PLACED_PER_FIELD_LEADING
        } else {
            PLACED_PER_FIELD
        }
    }

    /// The GROWING field (Lager 19 of the Werft): wraps the value at
    /// the width and grows line by line — long radio messages used to
    /// run past the field edge (owner, 27.08.). From `max_lines` on
    /// the WINDOW stays on the last lines (the caret types at the
    /// end). Selection is tracked logically (field_edit), but not yet
    /// drawn as an area — the documented keystone of this block.
    pub struct AreaBuild {
        pub node: Node,
        /// Line labels with RELATIVE Placed slots.
        pub labels: Vec<(Label, usize)>,
        /// How many Placed this field lays.
        pub placed: usize,
        /// The finished field height in logical pixels.
        pub height: f32,
    }

    /// Wrap the value HARD (character-preserving, at spaces where
    /// possible) and map the caret onto its line.
    fn area_wrap(
        engine: &mut TextEngine,
        value: &str,
        px: f32,
        inner_w: f32,
        cursor: usize,
    ) -> (Vec<String>, Vec<usize>, usize, f32) {
        let chars: Vec<char> = value.chars().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut starts: Vec<usize> = Vec::new();
        let mut start = 0usize;
        let mut cursor_line = 0usize;
        let mut cursor_x = 0.0f32;
        while start < chars.len() || lines.is_empty() {
            let mut end = start;
            let mut last_space: Option<usize> = None;
            let mut hard = false;
            while end < chars.len() {
                if chars[end] == '\n' {
                    // A DELIBERATE line break (Shift+Enter, P2.4).
                    hard = true;
                    break;
                }
                if chars[end] == ' ' {
                    last_space = Some(end);
                }
                let head: String = chars[start..=end].iter().collect();
                if engine.measure_styled(&head, px, mk::typo::Weight::Normal).0 > inner_w
                    && end > start
                {
                    break;
                }
                end += 1;
            }
            let cut = if hard {
                end
            } else if end < chars.len() {
                match last_space {
                    Some(sp) if sp > start => sp + 1,
                    _ => end,
                }
            } else {
                end
            };
            let at_end = !hard && cut >= chars.len();
            if cursor >= start && (cursor < cut || (hard && cursor == cut) || (at_end && cursor == cut)) {
                cursor_line = lines.len();
                let head: String = chars[start..cursor].iter().collect();
                cursor_x = engine.measure_styled(&head, px, mk::typo::Weight::Normal).0;
            }
            starts.push(start);
            lines.push(chars[start..cut].iter().collect());
            let next = if hard { cut + 1 } else { cut };
            if next >= chars.len() {
                if hard {
                    // After a trailing break an empty line begins —
                    // that is where the caret stands.
                    starts.push(next);
                    lines.push(String::new());
                    if cursor >= next {
                        cursor_line = lines.len() - 1;
                        cursor_x = 0.0;
                    }
                }
                break;
            }
            start = next;
        }
        (lines, starts, cursor_line, cursor_x)
    }

    pub fn input_area(
        engine: &mut TextEngine,
        spec: &FieldSpec,
        width: f32,
        max_lines: usize,
        p: mk::Palette,
    ) -> AreaBuild {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let px = mk::font_size::MEDIUM;
        let line = px * matrixkit_text::LINE_HEIGHT;
        let bw = if spec.focused { 2.0 } else { 1.0 };
        let rim =
            if spec.focused { p.primary } else { p.outline.over(p.surface_container, 0.35) };
        let inner_w = width - 2.0 * mk::spacing::S - 2.0 * bw;
        let empty = spec.value.is_empty();
        let (mut lines, mut starts, mut cursor_line, cursor_x) = if empty {
            (vec![String::new()], vec![0usize], 0usize, 0.0)
        } else {
            area_wrap(engine, spec.value, px, inner_w, spec.cursor)
        };
        // The window: from max_lines on we show the LAST lines.
        if lines.len() > max_lines {
            let drop = lines.len() - max_lines;
            lines.drain(..drop);
            starts.drain(..drop);
            cursor_line = cursor_line.saturating_sub(drop);
        }
        // The SELECTION (P1.3): anchor→caret as a character span, per
        // line translated into a patch — measured, never estimated.
        let sel = spec.anchor.filter(|a| *a != spec.cursor).map(|a| {
            (a.min(spec.cursor), a.max(spec.cursor))
        });
        let value_chars: Vec<char> = spec.value.chars().collect();
        let mut sel_of_line = |li: usize| -> (f32, f32) {
            let Some((lo, hi)) = sel else { return (0.0, 0.0) };
            let start = starts[li];
            let end = start + lines[li].chars().count();
            let a = lo.max(start);
            let b = hi.min(end);
            if b <= a {
                return (0.0, 0.0);
            }
            let head: String = value_chars[start..a].iter().collect();
            let span: String = value_chars[start..b].iter().collect();
            let x = engine.measure_styled(&head, px, mk::typo::Weight::Normal).0;
            let w = engine.measure_styled(&span, px, mk::typo::Weight::Normal).0 - x;
            (x.min(inner_w), w.max(0.0).min(inner_w))
        };
        let caret_w = if spec.focused && spec.blink { 1.5 } else { 0.0 };
        // Half a leading's padding on top — otherwise the text sticks
        // to the top field edge while the line air stays at the bottom
        // (owner: «closer to the edge on top than at the bottom»).
        let breath = (line - px) * 0.5;
        let mut kids: Vec<Node> = vec![Node::Quad {
            w: inner_w,
            h: breath,
            color: clear,
            radius: 0.0,
        }];
        let mut labels: Vec<(Label, usize)> = Vec::new();
        // Placed: border wrap 1 + ground wrap 1 + breath 1, per line 5
        // (selection lead-in, SELECTION, prefix, caret, rest — the
        // selection lies UNDER the label via a negative gap).
        let sel_tint = p.primary.over(p.on_surface.over(p.surface_container, 0.08), 0.35);
        let mut slot = 3usize;
        for (li, text) in lines.iter().enumerate() {
            let is_cursor = li == cursor_line;
            let (sx, sw) = sel_of_line(li);
            let shown = if empty { spec.placeholder } else { text.as_str() };
            let tint = if empty { p.on_surface_variant } else { p.on_surface };
            labels.push((
                Label {
                    text: shown.to_string(),
                    px,
                    weight: mk::typo::Weight::Normal,
                    family: matrixkit_text::FAMILY,
                    color: tint,
                },
                slot + 2,
            ));
            kids.push(Node::Column {
                gap: -line,
                padding: 0.0,
                background: None,
                children: vec![
                    Node::Row {
                        gap: 0.0,
                        padding: 0.0,
                        background: None,
                        children: vec![
                            Node::Quad { w: sx, h: 0.0, color: clear, radius: 0.0 },
                            Node::Quad { w: sw, h: line, color: sel_tint, radius: 2.0 },
                        ],
                    },
                    Node::Row {
                        gap: 0.0,
                        padding: 0.0,
                        background: None,
                        children: vec![
                            Node::Quad {
                                w: if is_cursor { cursor_x.min(inner_w) } else { 0.0 },
                                h: line,
                                color: clear,
                                radius: 0.0,
                            },
                            Node::Quad {
                                w: if is_cursor { caret_w } else { 0.0 },
                                h: line,
                                color: p.on_surface,
                                radius: 0.0,
                            },
                            Node::Quad {
                                w: (inner_w
                                    - if is_cursor {
                                        cursor_x.min(inner_w) + caret_w
                                    } else {
                                        0.0
                                    })
                                .max(0.0),
                                h: line,
                                color: clear,
                                radius: 0.0,
                            },
                        ],
                    },
                ],
            });
            slot += 5;
        }
        let node = Node::Wrap {
            pad_x: bw,
            pad_y: bw,
            background: Some((rim, mk::radius::SMALL)),
            child: Box::new(Node::Wrap {
                pad_x: mk::spacing::S - bw,
                pad_y: 6.0 - bw,
                background: Some((p.on_surface.over(p.surface_container, 0.08), mk::radius::SMALL)),
                child: Box::new(Node::Column {
                    gap: 0.0,
                    padding: 0.0,
                    background: None,
                    children: kids,
                }),
            }),
        };
        let n_lines = lines.len();
        AreaBuild {
            node,
            labels,
            placed: 3 + 5 * n_lines,
            height: 2.0 * bw + 2.0 * (6.0 - bw) + breath + n_lines as f32 * line,
        }
    }

    // ------------------------------------------------------------
    // NEW FAMILIES from the making of the Werft (Lager 25, 27.08.2026):
    // what apps in the light built by hand several times becomes a
    // family — exclusively without iced.
    // ------------------------------------------------------------

    /// The VEIL of an overlay: the window-filling dark under every
    /// floating card. One Placed; the caller assigns the role (close on
    /// press).
    pub fn veil(w: f32, h: f32) -> Placed {
        let mut out = Vec::new();
        matrixkit_theme::scene::lay_out(
            &Node::Quad {
                w,
                h,
                color: mk::Rgba { r: 0.08, g: 0.08, b: 0.08, a: 0.45 },
                radius: 0.0,
            },
            0.0,
            0.0,
            &mut out,
        );
        out.remove(0)
    }

    /// A FLOATING PILL — the small action above the content ("to the
    /// end", "reload"). Wrap (2 Placed: ground + text measure); the
    /// ground gets the floating shadow, the caller assigns the role
    /// across the area.
    pub fn floating_pill(
        engine: &mut TextEngine,
        text: &str,
        p: mk::Palette,
    ) -> (Node, Label) {
        let (n, l) = plain_label(engine, text, mk::font_size::SMALL, mk::typo::Weight::Semibold, p.on_surface);
        (
            Node::Wrap {
                pad_x: mk::spacing::M,
                pad_y: mk::spacing::XS,
                background: Some((p.surface_container_high, mk::radius::capsule(26.0))),
                child: Box::new(n),
            },
            l,
        )
    }

    /// The INK CHIP: colour dot + name in a capsule, selectable
    /// (project and category choice). 3 Placed: capsule, dot, text
    /// measure — the label anchors on +2.
    pub fn tint_chip(
        engine: &mut TextEngine,
        name: &str,
        tint: mk::Rgba,
        selected: bool,
        p: mk::Palette,
    ) -> (Node, Label) {
        let (n, l) = plain_label(
            engine,
            name,
            mk::font_size::SMALL,
            mk::typo::Weight::Normal,
            if selected { p.on_surface } else { p.on_surface_variant },
        );
        (
            Node::Wrap {
                pad_x: mk::spacing::S,
                pad_y: 3.0,
                background: Some((
                    if selected { tint.with_alpha(0.30) } else { p.surface_container },
                    mk::radius::capsule(22.0),
                )),
                child: Box::new(Node::Row {
                    gap: mk::spacing::XS,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Quad { w: 8.0, h: 8.0, color: tint, radius: mk::radius::capsule(8.0) },
                        n,
                    ],
                }),
            },
            l,
        )
    }

    /// The interview's ANSWER CHIP: kind label + text in one line;
    /// the RECOMMENDATION stands out on primary_container.
    /// 3 Placed: ground, label measure, text measure (labels +1, +2).
    pub fn choice_chip(
        engine: &mut TextEngine,
        kind: &str,
        text: &str,
        highlighted: bool,
        p: mk::Palette,
    ) -> (Node, Label, Label) {
        let (kn, kl) = plain_label(
            engine,
            kind,
            mk::font_size::SMALL,
            mk::typo::Weight::Semibold,
            if highlighted { p.primary } else { p.on_surface_variant },
        );
        let (tn, tl) =
            plain_label(engine, text, mk::font_size::MEDIUM, mk::typo::Weight::Normal, p.on_surface);
        (
            Node::Wrap {
                pad_x: mk::spacing::M,
                pad_y: mk::spacing::S,
                background: Some((
                    if highlighted {
                        p.primary_container.with_alpha(0.55)
                    } else {
                        p.surface_container_high
                    },
                    mk::radius::NORMAL,
                )),
                child: Box::new(Node::Row {
                    gap: mk::spacing::M,
                    padding: 0.0,
                    background: None,
                    children: vec![kn, tn],
                }),
            },
            kl,
            tl,
        )
    }

    /// The BLOCK CHIP: choice_chip with FULL text — the lines wrap at
    /// the given text width, nothing is cut off (owner, 27.08.:
    /// »answer and recommendation are partly not shown completely
    /// either«). Placed: ground, kind measure, then one measure per
    /// line (labels +1, +2, +3, …).
    pub fn choice_chip_block(
        engine: &mut TextEngine,
        kind: &str,
        text: &str,
        text_w: f32,
        highlighted: bool,
        p: mk::Palette,
    ) -> (Node, Label, Vec<Label>) {
        let (kn, kl) = plain_label(
            engine,
            kind,
            mk::font_size::SMALL,
            mk::typo::Weight::Semibold,
            if highlighted { p.primary } else { p.on_surface_variant },
        );
        let mut line_nodes = Vec::new();
        let mut line_labels = Vec::new();
        for line in wrap_text(engine, text, mk::font_size::MEDIUM, mk::typo::Weight::Normal, text_w) {
            let (ln, ll) = plain_label(
                engine,
                &line,
                mk::font_size::MEDIUM,
                mk::typo::Weight::Normal,
                p.on_surface,
            );
            line_nodes.push(ln);
            line_labels.push(ll);
        }
        if line_nodes.is_empty() {
            let (ln, ll) = plain_label(engine, "", mk::font_size::MEDIUM, mk::typo::Weight::Normal, p.on_surface);
            line_nodes.push(ln);
            line_labels.push(ll);
        }
        (
            Node::Wrap {
                pad_x: mk::spacing::M,
                pad_y: mk::spacing::S,
                background: Some((
                    if highlighted {
                        p.primary_container.with_alpha(0.55)
                    } else {
                        p.surface_container_high
                    },
                    mk::radius::NORMAL,
                )),
                child: Box::new(Node::Row {
                    gap: mk::spacing::M,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        kn,
                        Node::Column {
                            gap: mk::spacing::XS,
                            padding: 0.0,
                            background: None,
                            children: line_nodes,
                        },
                    ],
                }),
            },
            kl,
            line_labels,
        )
    }

    /// The SEPARATOR WITH A WORD: centred between the times of a
    /// history ("— neuer Tag —"). 1 Placed (text measure); the caller
    /// centres via the returned width.
    pub fn divider_label(
        engine: &mut TextEngine,
        text: &str,
        p: mk::Palette,
    ) -> (Node, Label) {
        plain_label(engine, text, mk::font_size::SMALL, mk::typo::Weight::Semibold, p.on_surface_variant)
    }

    /// The STATISTIC TILE: large number over a quiet caption — the
    /// tiles of the command bridge. 3 Placed: ground, number measure,
    /// caption measure (labels +1, +2).
    pub fn stat_tile(
        engine: &mut TextEngine,
        value: &str,
        caption: &str,
        w: f32,
        p: mk::Palette,
    ) -> (Node, Label, Label) {
        let (vn, vl) = plain_label(engine, value, 26.0, mk::typo::Weight::Semibold, p.on_surface);
        let (cn, cl) = plain_label(
            engine,
            caption,
            mk::font_size::SMALL,
            mk::typo::Weight::Normal,
            p.on_surface_variant,
        );
        (
            Node::Wrap {
                pad_x: mk::spacing::M,
                pad_y: mk::spacing::M,
                background: Some((p.surface_container_high, mk::radius::NORMAL)),
                child: Box::new(Node::Slot {
                    w: Some(w - 2.0 * mk::spacing::M),
                    h: None,
                    ax: Align::Start,
                    ay: Align::Start,
                    background: None,
                    child: Box::new(Node::Column {
                        gap: 2.0,
                        padding: 0.0,
                        background: None,
                        children: vec![vn, cn],
                    }),
                }),
            },
            vl,
            cl,
        )
    }

    /// The TAB CHIP in the light (the iced counterpart follows):
    /// capsule, active on primary_container. 2 Placed (ground + text
    /// measure, label +1).
    pub fn tab_chip(
        engine: &mut TextEngine,
        text: &str,
        active: bool,
        p: mk::Palette,
    ) -> (Node, Label) {
        let (n, l) = plain_label(
            engine,
            text,
            mk::font_size::MEDIUM,
            mk::typo::Weight::Semibold,
            if active { p.on_primary_container } else { p.on_surface_variant },
        );
        (
            Node::Wrap {
                pad_x: mk::spacing::M,
                pad_y: mk::spacing::XS,
                background: Some((
                    if active { p.primary_container } else { p.surface_container },
                    mk::radius::capsule(30.0),
                )),
                child: Box::new(n),
            },
            l,
        )
    }

    /// The TEXT BUTTON in the light: a clickable label without a
    /// ground (the press_area trap taught: text that should be
    /// clickable needs its own Placed with a role). 1 Placed.
    /// Lager 113: the LAID BUTTON. Three apps (tafel, arcade,
    /// hardline) had copied the same wrapper literally (studio lays
    /// onto its own build type and stays for now): build the spec,
    /// call button(), lay the node, label on the node's last slot. Now
    /// ONE call — and ONE truth about the label slot (two apps guessed
    /// `first + 1`, one `first + n - 1`; correct is the last slot).
    /// `role` None = resting (no hit, dimmed text).
    /// Returns (width, height).
    #[allow(clippy::too_many_arguments)]
    pub fn lay_button(
        b: &mut Sheet,
        engine: &mut TextEngine,
        label: &str,
        style: Style,
        status: Status,
        role: Option<usize>,
        x: f32,
        y: f32,
        clip: Option<Rect>,
        p: mk::Palette,
    ) -> (f32, f32) {
        let spec = Button { label: label.to_string(), style, role: Role::Normal, size: Size::Normal, active: role.is_some(), width: None };
        let (node, l) = button(engine, &spec, status, p);
        let (first, n, w, h) = b.node(&node, x, y, role, clip);
        b.label(first + n - 1, &l, 0.0, 0.0, clip);
        (w, h)
    }

    /// Lager 113: NIGHT GLASS — the design language of the Seraph
    /// pages (dark glass, nebula, stars) as a kit family. Before, the
    /// Tafel workshop kept palette, glass card, kicker and sky
    /// private; the owner wants more of it in all apps. The palette is
    /// FIXED (no token switch: night glass is always night), the
    /// contrast is proven (test `night_contrast`).
    pub mod night {
        use super::*;

        pub const GROUND: mk::Rgba = mk::Rgba { r: 0.043, g: 0.055, b: 0.11, a: 1.0 };
        pub const SURFACE: mk::Rgba = mk::Rgba { r: 0.086, g: 0.102, b: 0.18, a: 1.0 };
        pub const SURFACE2: mk::Rgba = mk::Rgba { r: 0.114, g: 0.133, b: 0.22, a: 1.0 };
        pub const INK: mk::Rgba = mk::Rgba { r: 0.933, g: 0.941, b: 1.0, a: 1.0 };
        pub const INK2: mk::Rgba = mk::Rgba { r: 0.663, g: 0.678, b: 0.741, a: 1.0 };
        pub const VIOLET: mk::Rgba = mk::Rgba { r: 0.616, g: 0.549, b: 1.0, a: 1.0 };
        pub const TURQUOISE: mk::Rgba = mk::Rgba { r: 0.373, g: 0.878, b: 0.847, a: 1.0 };
        pub const PINK: mk::Rgba = mk::Rgba { r: 1.0, g: 0.42, b: 0.541, a: 1.0 };
        /// The violet hairline of the glass card.
        pub const EDGE: mk::Rgba = mk::Rgba { r: 0.616, g: 0.549, b: 1.0, a: 0.26 };

        /// An open glass card: border and body are laid, the height
        /// comes with `glass_close` once the content is laid.
        #[derive(Clone, Copy, Debug)]
        pub struct Glass {
            pub edge: usize,
            pub body: usize,
        }

        /// Open the glass card: dark glass (94 %) with a violet
        /// hairline, NORMAL radius. Height 10 px for now.
        pub fn glass_open(b: &mut Sheet, x: f32, y: f32, w: f32, clip: Option<Rect>) -> Glass {
            let edge = b.quad(x - 1.0, y - 1.0, w + 2.0, 12.0, EDGE, mk::radius::NORMAL + 1.0, None, clip);
            let body = b.quad(x, y, w, 10.0, mk::Rgba { a: 0.94, ..SURFACE }, mk::radius::NORMAL, None, clip);
            Glass { edge, body }
        }

        /// Close the glass card: the body gets `h`, the border `h + 2`
        /// (one hairline at the top and bottom).
        pub fn glass_close(b: &mut Sheet, g: Glass, h: f32) {
            b.placed[g.edge].h = h + 2.0;
            b.placed[g.body].h = h;
        }

        /// The KICKER: the small upper-case line in turquoise above a
        /// title (CAPTION). Returns the height.
        pub fn kicker(b: &mut Sheet, engine: &mut TextEngine, text: &str, x: f32, y: f32, w: f32, clip: Option<Rect>) -> f32 {
            let up: String = text.to_uppercase();
            b.paragraph(engine, &up, mk::typo::CAPTION, TURQUOISE, x, y, w, None, clip)
        }

        /// WCAG contrast of two opaque colours (1 … 21).
        pub fn contrast(a: mk::Rgba, b: mk::Rgba) -> f32 {
            fn lum(c: mk::Rgba) -> f32 {
                fn ch(v: f32) -> f32 {
                    if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
                }
                0.2126 * ch(c.r) + 0.7152 * ch(c.g) + 0.0722 * ch(c.b)
            }
            let (la, lb) = (lum(a), lum(b));
            let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
            (hi + 0.05) / (lo + 0.05)
        }

        static SKY: std::sync::Mutex<Option<(u32, u32, std::sync::Arc<Vec<u8>>)>> = std::sync::Mutex::new(None);

        /// The NIGHT SKY as a ground (RGBA, opaque): gradient GROUND →
        /// #131830 (55 %) → #1a1f3a, three nebulae with a soft falloff,
        /// 140 soft stars. Computed once per size and shared (Arc) —
        /// whoever lays it every frame pays nothing.
        pub fn sky(w: u32, h: u32) -> std::sync::Arc<Vec<u8>> {
            if let Ok(g) = SKY.lock() {
                if let Some((cw, ch, data)) = &*g {
                    if *cw == w && *ch == h {
                        return data.clone();
                    }
                }
            }
            let (wf, hf) = (w as f32, h as f32);
            let mut px = vec![0u8; (w * h * 4) as usize];
            let mut seed: u32 = 7;
            let mut next = move || {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (seed >> 8) as f32 / 16777216.0
            };
            let stars: Vec<(f32, f32, f32, f32)> = (0..140).map(|_| { let sx = next() * wf; let sy = next() * hf; let b = 0.25 + next() * 0.75; let r = 0.9 + next() * 1.6; (sx, sy, r, b) }).collect();
            let scale = (wf / 1200.0).max(0.5);
            let fogs = [
                (wf * 0.78, hf * 0.86, 260.0 * scale, 0.16f32, (VIOLET.r, VIOLET.g, VIOLET.b)),
                (wf * 0.18, hf * 0.22, 220.0 * scale, 0.10, (TURQUOISE.r, TURQUOISE.g, TURQUOISE.b)),
                (wf * 0.6, hf * 0.1, 300.0 * scale, 0.06, (PINK.r, PINK.g, PINK.b)),
            ];
            for y in 0..h {
                let f = y as f32 / hf;
                let (r0, g0, b0) = if f < 0.55 {
                    let t = f / 0.55;
                    (0.043 + (0.075 - 0.043) * t, 0.055 + (0.094 - 0.055) * t, 0.11 + (0.188 - 0.11) * t)
                } else {
                    let t = (f - 0.55) / 0.45;
                    (0.075 + (0.102 - 0.075) * t, 0.094 + (0.122 - 0.094) * t, 0.188 + (0.227 - 0.188) * t)
                };
                for x in 0..w {
                    let (mut r, mut g, mut b) = (r0, g0, b0);
                    let (xf, yf) = (x as f32, y as f32);
                    for (fx, fy, fr, fa, (cr, cg, cb)) in &fogs {
                        let d2 = ((xf - fx) * (xf - fx) + (yf - fy) * (yf - fy)) / (fr * fr);
                        if d2 < 6.0 {
                            let a = fa * (-d2 * 1.4).exp();
                            r += (cr - r) * a;
                            g += (cg - g) * a;
                            b += (cb - b) * a;
                        }
                    }
                    let i = ((y * w + x) * 4) as usize;
                    px[i] = (r * 255.0) as u8;
                    px[i + 1] = (g * 255.0) as u8;
                    px[i + 2] = (b * 255.0) as u8;
                    px[i + 3] = 255;
                }
            }
            for (sx, sy, r, bright) in stars {
                let rr = (r * 2.5).ceil() as i32;
                for dy in -rr..=rr {
                    for dx in -rr..=rr {
                        let x = sx as i32 + dx;
                        let y = sy as i32 + dy;
                        if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                            continue;
                        }
                        let d2 = ((x as f32 - sx) * (x as f32 - sx) + (y as f32 - sy) * (y as f32 - sy)) / (r * r);
                        let a = bright * (-d2 * 1.6).exp();
                        if a < 0.01 {
                            continue;
                        }
                        let i = ((y as u32 * w + x as u32) * 4) as usize;
                        for (k, c) in [(0, 0.93f32), (1, 0.94), (2, 1.0)] {
                            let v = px[i + k] as f32 / 255.0;
                            px[i + k] = ((v + (c - v) * a) * 255.0) as u8;
                        }
                    }
                }
            }
            let data = std::sync::Arc::new(px);
            if let Ok(mut g) = SKY.lock() {
                *g = Some((w, h, data.clone()));
            }
            data
        }
    }

    pub fn text_button(
        engine: &mut TextEngine,
        text: &str,
        p: mk::Palette,
    ) -> (Node, Label) {
        plain_label(engine, text, mk::font_size::SMALL, mk::typo::Weight::Semibold, p.primary)
    }

    /// A bare label as (Node, Label) — the common core of the small
    /// families.
    fn plain_label(
        engine: &mut TextEngine,
        text: &str,
        px: f32,
        weight: mk::typo::Weight,
        color: mk::Rgba,
    ) -> (Node, Label) {
        let (tw, th) = engine.measure_styled(text, px, weight);
        (
            Node::Quad { w: tw, h: th, color: mk::Rgba { a: 0.0, ..color }, radius: 0.0 },
            Label { text: text.to_string(), px, weight, family: matrixkit_text::FAMILY, color },
        )
    }

    pub fn input_field(
        engine: &mut TextEngine,
        spec: &FieldSpec,
        width: f32,
        p: mk::Palette,
    ) -> FieldBuild {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let px = mk::font_size::MEDIUM;
        let line = px * matrixkit_text::LINE_HEIGHT;
        let bw = if spec.focused { 2.0 } else { 1.0 };
        let rim =
            if spec.focused { p.primary } else { p.outline.over(p.surface_container, 0.35) };
        // Prefixes up to caret and anchor — MEASURED, never estimated.
        let value = spec.value;
        let shown = if value.is_empty() { spec.placeholder } else { value };
        let mut width_to = |n: usize| {
            let head: String = value.chars().take(n).collect();
            engine.measure_styled(&head, px, mk::typo::Weight::Normal).0
        };
        let prefix_w = width_to(spec.cursor);
        // The selection: from the anchor to the caret, as a patch
        // UNDER the label (primary over the field ground, 0.35 — the
        // iced selection colour).
        let (sel_x, sel_w) = match spec.anchor {
            Some(a) if a != spec.cursor => {
                let (lo, hi) = (width_to(a.min(spec.cursor)), width_to(a.max(spec.cursor)));
                (lo, hi - lo)
            }
            _ => (0.0, 0.0),
        };
        let caret_w = if spec.focused && spec.blink { 1.5 } else { 0.0 };
        // The leading glyph takes space IN the field — the text begins
        // behind it, otherwise it would lie beneath it.
        let lead = spec.leading.map(|ch| {
            let g = ch.to_string();
            let (gw, gh) =
                engine.measure_family(crate::symbol_family(), &g, px, mk::typo::Weight::Normal);
            (g, gw, gh)
        });
        let lead_w = lead.as_ref().map(|(_, gw, _)| gw + mk::spacing::XS).unwrap_or(0.0);
        let inner_w = width - 2.0 * mk::spacing::S - lead_w;
        let core = Node::Column {
                    gap: -line,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        // Row 1: the selection (lies under the text).
                        Node::Row {
                            gap: 0.0,
                            padding: 0.0,
                            background: None,
                            children: vec![
                                Node::Quad { w: sel_x, h: 0.0, color: clear, radius: 0.0 },
                                Node::Quad {
                                    w: sel_w,
                                    h: line,
                                    color: p.primary.over(
                                        p.on_surface.over(p.surface_container, 0.08),
                                        0.35,
                                    ),
                                    radius: 2.0,
                                },
                            ],
                        },
                        // Row 2 (pulled back by one line height):
                        // prefix + caret + width spacer.
                        Node::Row {
                            gap: 0.0,
                            padding: 0.0,
                            background: None,
                            children: vec![
                                Node::Quad {
                                    w: prefix_w.min(inner_w),
                                    h: line,
                                    color: clear,
                                    radius: 0.0,
                                },
                                Node::Quad { w: caret_w, h: line, color: p.on_surface, radius: 0.0 },
                                Node::Quad {
                                    w: (inner_w - prefix_w - caret_w).max(0.0),
                                    h: 0.0,
                                    color: clear,
                                    radius: 0.0,
                                },
                            ],
                        },
                    ],
        };
        // With a glyph the body becomes a row: magnifier, air, text.
        let body = match lead.as_ref() {
            Some((_, gw, gh)) => Node::Row {
                gap: 0.0,
                padding: 0.0,
                background: None,
                children: vec![
                    Node::Quad { w: *gw, h: *gh, color: clear, radius: 0.0 },
                    Node::Quad { w: mk::spacing::XS, h: 0.0, color: clear, radius: 0.0 },
                    core,
                ],
            },
            None => core,
        };
        let node = Node::Wrap {
            pad_x: bw,
            pad_y: bw,
            background: Some((rim, mk::radius::SMALL)),
            child: Box::new(Node::Wrap {
                pad_x: mk::spacing::S - bw,
                pad_y: 6.0 - bw,
                background: Some((
                    p.on_surface.over(p.surface_container, 0.08),
                    mk::radius::inner(mk::radius::SMALL, bw),
                )),
                child: Box::new(body),
            }),
        };
        let label = Label {
            text: shown.to_string(),
            px,
            weight: mk::typo::Weight::Normal,
            family: matrixkit_text::FAMILY,
            color: if value.is_empty() { p.on_surface_variant } else { p.on_surface },
        };
        debug_assert_eq!(matrixkit_theme::scene::placed_count(&node), placed_per_field(spec));
        FieldBuild { node, label, leading: lead.map(|(g, _, _)| g) }
    }

    /// The built bar including bookkeeping: tool_slots are the hit
    /// targets (◀, ▶, then the right-hand tools), label_slots the
    /// labels in label order.
    pub struct Bar {
        pub node: Node,
        pub labels: Vec<Label>,
        pub label_slots: Vec<usize>,
        pub tool_slots: Vec<usize>,
        /// Placed offset of the search field (border), if there is
        /// one — hit target in steps of PLACED_PER_FIELD.
        pub search_slot: Option<usize>,
    }

    /// The toolbar (mkw::toolbar): ◀ ▶ [title] … [right-hand tools],
    /// HEADLINE title, everything vertically centred, fixed width. The
    /// iced version's search field is DELIBERATELY missing — ENTERING
    /// text is stage 4 land (IME); the bar moves in without it, the
    /// contract grows when input comes.
    /// The bar's search field width (reference: 200 px).
    pub const SEARCH_WIDTH: f32 = 200.0;

    #[allow(clippy::too_many_arguments)]
    pub fn toolbar(
        engine: &mut TextEngine,
        title: &str,
        back: (bool, Status),
        forward: (bool, Status),
        right_tools: &[(char, bool, Status)],
        search: Option<&FieldSpec>,
        width: f32,
        p: mk::Palette,
    ) -> Bar {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mut labels = Vec::new();
        let mut label_slots = Vec::new();
        let mut tool_slots = Vec::new();
        let mut offset = 0usize;
        // Lager 50: two groups instead of one row with a fill spacer.
        // The left one is laid, the right one is laid at the edge —
        // nobody computes in between anymore.
        let mut left = Vec::new();
        let mut right = Vec::new();
        let tool_h = 18.0 * matrixkit_text::LINE_HEIGHT + 8.0;
        let push_tool = |glyph: char, active: bool, status: Status,
                             engine: &mut TextEngine,
                             children: &mut Vec<Node>,
                             labels: &mut Vec<Label>,
                             label_slots: &mut Vec<usize>,
                             tool_slots: &mut Vec<usize>,
                             offset: &mut usize| {
            let (node, label) = tool_button(engine, glyph, active, status, p);
            tool_slots.push(*offset);
            *offset += 1;
            label_slots.push(*offset);
            *offset += 1;
            labels.push(label);
            children.push(node);
        };
        push_tool(symbol::ARROW_BACK, back.0, back.1, engine, &mut left, &mut labels, &mut label_slots, &mut tool_slots, &mut offset);
        push_tool(symbol::ARROW_FORWARD, forward.0, forward.1, engine, &mut left, &mut labels, &mut label_slots, &mut tool_slots, &mut offset);
        // Air S between navigation and title (like mkw::toolbar).
        left.push(Node::Quad { w: mk::spacing::S, h: 0.0, color: clear, radius: 0.0 });
        offset += 1;
        let head = mk::typo::HEADLINE;
        let (tw, th) = engine.measure_styled(title, head.px(), head.weight);
        labels.push(Label {
            text: title.to_string(),
            px: head.px(),
            weight: head.weight,
            family: matrixkit_text::FAMILY,
            color: p.on_surface,
        });
        label_slots.push(offset);
        offset += 1;
        left.push(Node::Wrap {
            pad_x: 0.0,
            pad_y: ((tool_h - th) / 2.0).max(0.0),
            background: None,
            child: Box::new(Node::Quad { w: tw, h: th, color: clear, radius: 0.0 }),
        });
        for (glyph, active, status) in right_tools {
            push_tool(*glyph, *active, *status, engine, &mut right, &mut labels, &mut label_slots, &mut tool_slots, &mut offset);
        }
        // The search field (Lager 19): far right, reference width —
        // the contract grew when input came.
        let mut search_slot = None;
        if let Some(spec) = search {
            let field = input_field(engine, spec, SEARCH_WIDTH, p);
            search_slot = Some(offset);
            label_slots.push(offset + 4);
            offset += PLACED_PER_FIELD;
            labels.push(field.label);
            right.push(field.node);
        }
        // The bar TAKES its width (it wishes for none): both groups get
        // the same box across the full width, the left one flush at
        // the start, the right one at the edge. The bookkeeping keeps
        // counting in laying order — the stack lays the left group
        // first, then the right.
        let group = |children: Vec<Node>| Node::Row {
            gap: mk::spacing::XS,
            padding: 0.0,
            background: None,
            children,
        };
        let node = Node::Stack {
            children: vec![
                Node::Slot {
                    w: Some(width),
                    h: None,
                    ax: Align::Start,
                    ay: Align::Start,
                    background: None,
                    child: Box::new(group(left)),
                },
                Node::Slot {
                    w: Some(width),
                    h: None,
                    ax: Align::End,
                    ay: Align::Start,
                    background: None,
                    child: Box::new(group(right)),
                },
            ],
        };
        debug_assert_eq!(matrixkit_theme::scene::placed_count(&node), offset);
        Bar { node, labels, label_slots, tool_slots, search_slot }
    }

    /// Lager 10: the NOTICES move in. The result banner (mkw::notice,
    /// R47): primary = succeeded, error = failed — text AND ground
    /// come from the same ink (ground = ink over surface_container,
    /// 0.12), air S, radius::SMALL, full width. Placed: ground + text
    /// measure + fill spacer (PLACED_PER_NOTICE).
    pub fn notice(
        engine: &mut TextEngine,
        ok: bool,
        text: &str,
        width: f32,
        p: mk::Palette,
    ) -> (Node, Label) {
        let tint = if ok { p.primary } else { p.error };
        let body = mk::typo::BODY;
        let (tw, th) = engine.measure_styled(text, body.px(), body.weight);
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let slack = (width - 2.0 * mk::spacing::S - tw).max(0.0);
        let node = Node::Wrap {
            pad_x: mk::spacing::S,
            pad_y: mk::spacing::S,
            background: Some((tint.over(p.surface_container, 0.12), mk::radius::SMALL)),
            child: Box::new(Node::Row {
                gap: 0.0,
                padding: 0.0,
                background: None,
                children: vec![
                    Node::Quad { w: tw, h: th, color: clear, radius: 0.0 },
                    Node::Quad { w: slack, h: 0.0, color: clear, radius: 0.0 },
                ],
            }),
        };
        (node, Label {
            text: text.to_string(),
            px: body.px(),
            weight: body.weight,
            family: matrixkit_text::FAMILY,
            color: tint,
        })
    }

    /// Placed per notice: ground + text measure + fill spacer.
    pub const PLACED_PER_NOTICE: usize = 3;

    /// The quiet footer (mkw::footer): mode hints, copy feedback,
    /// permission messages — CAPTION in variant text with M air above.
    /// Placed: air spacer + text measure.
    pub fn footer(engine: &mut TextEngine, content: &str, p: mk::Palette) -> (Node, Label) {
        let cap = mk::typo::CAPTION;
        let (tw, th) = engine.measure_styled(content, cap.px(), cap.weight);
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let node = Node::Column {
            gap: 0.0,
            padding: 0.0,
            background: None,
            children: vec![
                Node::Quad { w: 0.0, h: mk::spacing::M, color: clear, radius: 0.0 },
                Node::Quad { w: tw, h: th, color: clear, radius: 0.0 },
            ],
        };
        (node, Label {
            text: content.to_string(),
            px: cap.px(),
            weight: cap.weight,
            family: matrixkit_text::FAMILY,
            color: p.on_surface_variant,
        })
    }

    /// Placed per footer: air + text measure.
    pub const PLACED_PER_FOOTER: usize = 2;

    /// Lager 13: the FRAME moves in — the header with the traffic
    /// lights, the face of every MatrixKit window (theme_header). On
    /// the left the three 14 pt capsules (error/secondary/primary;
    /// R168g: deliberately above the reference's measured 12 pt — a
    /// touch system wants bigger targets), title SUBTITLE CENTRED;
    /// height = SHADE_HEIGHT. Since Lager 50 two boxes in the stack
    /// carry the arrangement (traffic lights start, title center) —
    /// the reference's 64 pt counterweight was a layout makeshift and
    /// has been dropped.
    /// appearsActive: when inactive the traffic lights turn uniformly
    /// grey and the title speaks the variant text. Hovering over the
    /// group shows the glyph in ALL three (× − +, dark on the light) —
    /// reference grammar. Drag gesture, magnifier and tooltip remain
    /// reminders of the effect lagers.
    #[derive(Clone, Copy)]
    pub struct HeaderState {
        pub active: bool,
        pub lights_hover: bool,
        /// Is the window ROLLED UP? The window button on the right then
        /// shows "unroll" instead of "roll up" — like the reference
        /// (mkw::app_window, symbol::UNFOLD_MORE/LESS).
        pub rolled_up: bool,
        /// The MAGNIFIER (Lager 91): the magnification factor per
        /// target — [light 0, light 1, light 2, title, window button].
        /// 1.0 rests.
        ///
        /// It comes from outside because it is SPRUNG and the spring
        /// belongs to the client (it alone knows the frame beat).
        /// [`HeaderMotion`] computes it; whoever does not want it takes
        /// [`HeaderState::calm`].
        pub magnify: [f32; 5],
    }

    impl HeaderState {
        /// A bar without a magnifier — for probes and for surfaces that
        /// never see a pointer.
        pub fn calm(active: bool, lights_hover: bool, rolled_up: bool) -> Self {
            HeaderState { active, lights_hover, rolled_up, magnify: [1.0; 5] }
        }
    }

    /// The largest magnification of the traffic lights (reference:
    /// `magnifier_strong(.., 1.18)`) and that of the remaining buttons
    /// (`MAGNIFIER_MAX`).
    pub const LIGHT_MAGNIFY: f32 = 1.18;

    /// The MAGNIFIER of the window bar (Lager 91).
    ///
    /// The owner: «hover zoom is missing in the harness». It was missing
    /// because the light path held no spring for it — the reference
    /// springs EVERY button of the bar individually under the pointer
    /// (`mkw::magnifier`), and without a spring it would be a jump
    /// instead of a pop-up.
    ///
    /// The piece holds the boxes of the five targets, their springs,
    /// and knows which one currently lies under the pointer. The
    /// client needs four lines: `learn` after laying, `point` in
    /// `motion`, `tick` in `glide`, `running` in `animating`.
    pub struct HeaderMotion {
        boxes: [Rect; 5],
        springs: [mk::motion::Spring; 5],
        hover: Option<usize>,
        reduced: bool,
    }

    impl Default for HeaderMotion {
        fn default() -> Self {
            Self::new()
        }
    }

    impl HeaderMotion {
        pub fn new() -> Self {
            const LEER: Rect = Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
            HeaderMotion {
                boxes: [LEER; 5],
                springs: std::array::from_fn(|_| mk::motion::Spring::snappy(1.0)),
                hover: None,
                // "Reduce motion" switches the magnifier off HARD —
                // like the reference (mkw::magnifier).
                reduced: mk::motion_reduced(),
            }
        }

        /// Learn the boxes from the laid header. `dx`/`dy` is the
        /// offset with which the header was laid into the scene.
        pub fn learn(&mut self, header: &Header, placed: &[Placed], dx: f32, dy: f32) {
            let take = |slot: usize| {
                placed
                    .get(slot)
                    .map(|q| Rect { x: q.x + dx, y: q.y + dy, w: q.w, h: q.h })
                    .unwrap_or(Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 })
            };
            for (i, slot) in header.light_slots.iter().enumerate() {
                self.boxes[i] = take(*slot);
            }
            self.boxes[3] = take(header.title_slot);
            self.boxes[4] = take(header.shade_slot);
        }

        /// Set the boxes directly — for clients whose bar does not lie
        /// at (0,0) and who have already computed them.
        pub fn set_boxes(&mut self, boxes: [Rect; 5]) {
            self.boxes = boxes;
        }

        /// The pointer stands here. Answer: did something change?
        pub fn point(&mut self, x: f64, y: f64) -> bool {
            // The targets are SMALL (14 pt); a button wants some
            // apron, otherwise one hits it only by luck. The reference
            // gives its buttons the same via the padding.
            const REACH: f32 = 3.0;
            let (x, y) = (x as f32, y as f32);
            let hit = self.boxes.iter().position(|r| {
                r.w > 0.0
                    && x >= r.x - REACH
                    && x < r.x + r.w + REACH
                    && y >= r.y - REACH
                    && y < r.y + r.h + REACH
            });
            self.aim(hit)
        }

        /// The pointer is gone (surface left).
        pub fn leave(&mut self) -> bool {
            self.aim(None)
        }

        fn aim(&mut self, hit: Option<usize>) -> bool {
            if hit == self.hover {
                return false;
            }
            self.hover = hit;
            for (i, s) in self.springs.iter_mut().enumerate() {
                let max = if i < 3 { LIGHT_MAGNIFY } else { crate::MAGNIFIER_MAX };
                let target = if self.reduced {
                    1.0
                } else if self.hover == Some(i) {
                    max
                } else {
                    1.0
                };
                s.retarget(target);
            }
            true
        }

        pub fn running(&self) -> bool {
            self.springs.iter().any(|s| !s.is_settled())
        }

        pub fn tick(&mut self, dt: f32) -> bool {
            let mut moved = false;
            for s in self.springs.iter_mut() {
                moved |= s.tick(dt);
            }
            moved
        }

        pub fn factors(&self) -> [f32; 5] {
            std::array::from_fn(|i| self.springs[i].value())
        }

        /// Is the pointer hovering over ONE of the three traffic
        /// lights? Then ALL three show their glyph — reference grammar.
        pub fn lights_hover(&self) -> bool {
            matches!(self.hover, Some(i) if i < 3)
        }
    }

    /// CHROME (R210 family): the whole window skin of a light-shell app —
    /// head with lights, title and shade key (`frame_header`), hover and
    /// magnifier, roll-up, maximise, close, the head strip for flicks and
    /// foreign-desktop moves. Every app spelled these ~150 lines out
    /// (terminal, player, tube …); now it keeps one `Chrome` and asks it.
    ///
    /// Roles: `base + 0..3` the lights (close, shelf, maximise), `base + 3`
    /// the shade key, `base + 4` the title.
    pub struct Chrome {
        pub title: String,
        /// The window is the active one (head in colour).
        pub active: bool,
        pub rolled: bool,
        pub zoomed: bool,
        base: usize,
        lights_hover: bool,
        motion: HeaderMotion,
        last: (u32, u32),
        head_h: u32,
        before: Option<(u32, u32)>,
        lights: Rect,
        /// R210 duo grammar (owner 01.11.2026): no head at all — the
        /// app's name already stands top left in the top bar, and that
        /// chip's menu closes the window or opens its settings (root pane).
        /// No roll-up either. `MATRIX_HEAD=1` brings the head back.
        pub bare: bool,
    }

    /// Does this device run its windows bare (duo grammar)?
    pub fn bare_windows() -> bool {
        mk::device::is_duo() && std::env::var_os("MATRIX_HEAD").is_none()
    }

    /// What a chrome role asks of the app after a strike.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum ChromeDeed {
        /// The × light: the app tidies up and answers `Refresh::Close`.
        Close,
        /// The title was pressed (the app's root pane, if it has one).
        Title,
        /// Done by the chrome itself — answer with this refresh.
        Answer(matrixkit_shell::Refresh),
    }

    impl Chrome {
        pub fn new(title: impl Into<String>, role_base: usize) -> Self {
            Chrome {
                title: title.into(),
                active: true,
                rolled: false,
                zoomed: false,
                base: role_base,
                lights_hover: false,
                motion: HeaderMotion::new(),
                last: (0, 0),
                head_h: 50,
                before: None,
                lights: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
                bare: bare_windows(),
            }
        }

        /// Build the head into `b` (window `w` × `h`); returns where the
        /// content begins. A rolled-up window is only its head — the app
        /// stops building when `rolled` is set.
        pub fn build(&mut self, engine: &mut TextEngine, b: &mut matrixkit_shell::SceneBuilder, w: u32, h: u32, p: mk::Palette) -> f32 {
            if self.bare {
                self.last = (w, h);
                self.head_h = 0;
                self.rolled = false;
                return 0.0;
            }
            let header = frame_header(
                engine,
                &self.title,
                HeaderState { active: self.active, lights_hover: self.lights_hover, rolled_up: self.rolled, magnify: self.motion.factors() },
                w as f32,
                p,
            );
            let base_index = b.len();
            let mut placed = Vec::new();
            matrixkit_theme::scene::lay_out(&header.node, 0.0, 0.0, &mut placed);
            self.motion.learn(&header, &placed, 0.0, 0.0);
            self.last = (w, h);
            self.head_h = header.height.ceil() as u32;
            let mut roles: Vec<Option<usize>> = vec![None; placed.len()];
            for (k, slot) in header.light_slots.iter().enumerate() {
                roles[*slot] = Some(self.base + k);
            }
            roles[header.shade_slot] = Some(self.base + 3);
            roles[header.title_slot] = Some(self.base + 4);
            let ground = Rect::of(&placed[0]);
            let at = |slot: usize| placed[slot];
            let label_on = |l: &Label, slot: usize, home: Rect, dx: f32, dy: f32| matrixkit_shell::Text {
                s: l.text.clone(),
                px: l.px,
                weight: l.weight,
                family: l.family,
                color: l.color,
                x: at(slot).x + dx,
                y: at(slot).y + dy,
                home,
                anchor: base_index + slot,
                clip: None,
            };
            let mut texts = vec![
                label_on(&header.shade, header.shade_slot, Rect::of(&at(header.shade_slot)), 0.0, 0.0),
                label_on(&header.title, header.title_slot, ground, 0.0, 0.0),
            ];
            for (k, (g, dx, dy)) in header.glyphs.iter().enumerate() {
                texts.push(label_on(g, header.light_slots[k], ground, *dx, *dy));
            }
            let (l0, l2) = (at(header.light_slots[0]), at(header.light_slots[2]));
            self.lights = Rect { x: l0.x - 2.0, y: l0.y - 2.0, w: (l2.x + l2.w) - l0.x + 4.0, h: l0.h + 4.0 };
            for (q, role) in placed.into_iter().zip(roles) {
                b.scene.placed.push(q);
                b.scene.roles.push(role);
            }
            b.scene.texts.extend(texts);
            header.height
        }

        /// Is this role one of the chrome's?
        pub fn owns(&self, role: Option<usize>) -> bool {
            role.is_some_and(|r| (self.base..self.base + 5).contains(&r))
        }

        /// A strike on a role: None = not the chrome's.
        pub fn strike(&mut self, role: Option<usize>) -> Option<ChromeDeed> {
            let r = role.filter(|r| (self.base..self.base + 5).contains(r))? - self.base;
            Some(match r {
                0 => ChromeDeed::Close,
                1 => ChromeDeed::Answer(matrixkit_shell::Refresh::Shelve),
                2 => {
                    if self.rolled {
                        self.rolled = false;
                        ChromeDeed::Answer(matrixkit_shell::Refresh::Full)
                    } else {
                        ChromeDeed::Answer(matrixkit_shell::Refresh::Zoom)
                    }
                }
                3 => {
                    if self.zoomed {
                        return Some(ChromeDeed::Answer(matrixkit_shell::Refresh::Zoom));
                    }
                    self.rolled = !self.rolled;
                    if self.rolled {
                        self.before = Some(self.last);
                    }
                    ChromeDeed::Answer(matrixkit_shell::Refresh::Full)
                }
                _ => ChromeDeed::Title,
            })
        }

        /// Pointer over the window (hover glyphs, magnifier).
        pub fn motion(&mut self, x: f64, y: f64) -> matrixkit_shell::Refresh {
            let r = self.lights;
            let inside = r.w > 0.0 && (x as f32) >= r.x && (x as f32) < r.x + r.w && (y as f32) >= r.y && (y as f32) < r.y + r.h;
            let grew = self.motion.point(x, y);
            let changed = inside != self.lights_hover;
            self.lights_hover = inside;
            if changed || grew {
                matrixkit_shell::Refresh::Dirty
            } else {
                matrixkit_shell::Refresh::Calm
            }
        }

        /// Pointer left the window.
        pub fn depart(&mut self) -> matrixkit_shell::Refresh {
            let shrank = self.motion.leave();
            let was = std::mem::take(&mut self.lights_hover);
            if shrank || was {
                matrixkit_shell::Refresh::Dirty
            } else {
                matrixkit_shell::Refresh::Calm
            }
        }

        /// The magnifier springs (part of the app's `animating`).
        pub fn animating(&self) -> bool {
            self.motion.running()
        }

        /// One step of the magnifier — true if it moved (redraw).
        pub fn glide(&mut self, dt: f32) -> bool {
            self.motion.tick(dt)
        }

        /// The house reports maximised / free.
        pub fn set_zoomed(&mut self, on: bool) {
            self.zoomed = on;
        }

        /// The wish size: rolled = only the head; once after unrolling,
        /// the size from before.
        pub fn extent(&mut self) -> Option<(u32, u32)> {
            if self.rolled {
                Some((self.last.0.max(1), self.head_h.max(1)))
            } else {
                self.before.take()
            }
        }

        /// The head strip (logical px) — flick strip and foreign-desktop move.
        pub fn headline(&self) -> f32 {
            self.head_h as f32
        }
    }

    /// How a pill stands out.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum PillTone {
        /// surface_container_high — a quiet action (back, more).
        Quiet,
        /// primary_container — the thing that is going on (now playing).
        Accent,
    }

    /// PILL (R210 family): a capsule target with an optional leading symbol
    /// and a text — `width` None = as wide as its content, Some = that wide
    /// (the text elided to fit). Returns (width, height). Tube's «Zurück zur
    /// Suche» and «▶ Läuft: …».
    #[allow(clippy::too_many_arguments)]
    pub fn lay_pill(
        b: &mut matrixkit_shell::SceneBuilder,
        engine: &mut TextEngine,
        glyph: Option<char>,
        text: &str,
        x: f32,
        y: f32,
        width: Option<f32>,
        role: Option<usize>,
        tone: PillTone,
        p: mk::Palette,
    ) -> (f32, f32) {
        let h = 40.0;
        let m = mk::spacing::M;
        let (fill, ink) = match tone {
            PillTone::Quiet => (p.surface_container_high, p.on_surface),
            PillTone::Accent => (p.primary_container, p.on_primary_container),
        };
        let symfam = crate::symbol_family();
        let gpx = mk::typo::symbol_px(20.0);
        let (gw, gh) = match glyph {
            Some(g) => engine.measure_family(symfam, &g.to_string(), gpx, mk::typo::Weight::Normal),
            None => (0.0, 0.0),
        };
        let lead = if glyph.is_some() { gw + mk::spacing::S } else { 0.0 };
        let st = mk::typo::CALLOUT;
        let mut shown = text.to_string();
        let (mut tw, th) = engine.measure_styled(&shown, st.px(), st.weight);
        let w = match width {
            Some(w) => {
                let room = (w - 2.0 * m - lead).max(10.0);
                let mut kept: Vec<char> = text.chars().collect();
                while tw > room && kept.len() > 1 {
                    kept.pop();
                    shown = format!("{}\u{2026}", kept.iter().collect::<String>().trim_end());
                    tw = engine.measure_styled(&shown, st.px(), st.weight).0;
                }
                w
            }
            None => m + lead + tw + m,
        };
        b.quad(x, y, w, h, fill, h / 2.0, role, None);
        if let Some(g) = glyph {
            b.text(engine, g.to_string(), gpx, mk::typo::Weight::Normal, symfam, ink, x + m, y + (h - gh) / 2.0, role, None);
        }
        b.styled(engine, shown, st, ink, x + m + lead, y + (h - th) / 2.0, role, None);
        (w, h)
    }

    /// IDLE DIM (R210 family, the owner 02.11.2026): a window that steps back
    /// while nobody touches it — a black veil fades in after `after`
    /// seconds of quiet (cinema mode: less light beside the picture). A
    /// touch wakes it (`poke`) and still ACTS — the veil is see-through
    /// (02.11.2026: Tube swallowed every first touch, the app felt dead).
    /// Only a screen whose light was off may swallow the waking touch.
    pub struct IdleDim {
        last: std::time::Instant,
        level: f32,
        after: f32,
    }

    impl IdleDim {
        /// How dark the veil gets.
        pub const DARK: f32 = 0.85;

        pub fn new(after_s: f32) -> Self {
            IdleDim { last: std::time::Instant::now(), level: 0.0, after: after_s }
        }

        /// A touch: true if the window was dimmed (informative — let the touch act).
        pub fn poke(&mut self) -> bool {
            self.last = std::time::Instant::now();
            let was = self.level > 0.05;
            self.level = 0.0;
            was
        }

        /// Advance the fade; `wanted` = dimming allowed now (playing in
        /// cinema mode). True while it moves (redraw).
        pub fn tick(&mut self, dt: f32, wanted: bool) -> bool {
            let target = if wanted && self.last.elapsed().as_secs_f32() >= self.after { 1.0 } else { 0.0 };
            if !wanted {
                self.last = std::time::Instant::now();
            }
            let before = self.level;
            let step = if mk::motion_reduced() { 1.0 } else { dt / 0.8 };
            self.level = if self.level < target { (self.level + step).min(target) } else { (self.level - step * 3.0).max(target) };
            (self.level - before).abs() > 0.0001
        }

        pub fn level(&self) -> f32 {
            self.level
        }

        /// Seconds since the last touch.
        pub fn quiet_s(&self) -> f32 {
            self.last.elapsed().as_secs_f32()
        }

        /// Nothing to animate (the level stands where it should).
        pub fn settled(&self, wanted: bool) -> bool {
            let target = if wanted && self.last.elapsed().as_secs_f32() >= self.after { 1.0 } else { 0.0 };
            (self.level - target).abs() < 0.0001
        }

        /// Lay the veil over everything built so far.
        pub fn lay(&self, b: &mut matrixkit_shell::SceneBuilder, w: f32, h: f32) {
            if self.level > 0.0 {
                b.quad(0.0, 0.0, w, h, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: self.level * Self::DARK }, 0.0, None, None);
            }
        }
    }

    /// A pill's natural width (what `lay_pill` with `width: None` takes).
    pub fn pill_width(engine: &mut TextEngine, glyph: Option<char>, text: &str) -> f32 {
        let m = mk::spacing::M;
        let lead = glyph
            .map(|g| engine.measure_family(crate::symbol_family(), &g.to_string(), mk::typo::symbol_px(20.0), mk::typo::Weight::Normal).0 + mk::spacing::S)
            .unwrap_or(0.0);
        let st = mk::typo::CALLOUT;
        m + lead + engine.measure_styled(text, st.px(), st.weight).0 + m
    }

    // ===================================================================
    // Round 177 (06.11.2026, out of Morpheus, matrix-trust and the
    // greeter's installer/recovery — each had built these by hand).
    // ===================================================================

    /// A CENTRED PARAGRAPH: `text` wrapped to `col` px, every line centred
    /// on `mid_x`, from `y`. Returns the height used. (The trust dialog, the
    /// installer and the recovery page each had their own closure.)
    #[allow(clippy::too_many_arguments)]
    pub fn lay_centred(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, text: &str, style: mk::typo::TextStyle, ink: mk::Rgba, mid_x: f32, col: f32, y: f32) -> f32 {
        let chars = ((col / (style.px() * 0.5)) as usize).max(16);
        let mut yy = y;
        for line in mk::wrap_words(text, chars, 6) {
            let (lw, lh) = engine.measure_styled(&line, style.px(), style.weight);
            b.styled(engine, line, style, ink, mid_x - lw / 2.0, yy, None, None);
            yy += lh;
        }
        yy - y
    }

    /// One pill of a row: glyph, text, role (None = asleep), tone.
    pub type PillSpec<'a> = (Option<char>, &'a str, Option<usize>, PillTone);

    /// A ROW OF PILLS centred on `mid_x` («Nein · Ja», «Zurück · Weiter»).
    /// Returns (width, height) of the row.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_pill_row(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, pills: &[PillSpec], mid_x: f32, y: f32, gap: f32, p: mk::Palette) -> (f32, f32) {
        if pills.is_empty() {
            return (0.0, 0.0);
        }
        let widths: Vec<f32> = pills.iter().map(|(g, t, _, _)| pill_width(engine, *g, t)).collect();
        let total = widths.iter().sum::<f32>() + gap * (pills.len() as f32 - 1.0);
        let mut x = mid_x - total / 2.0;
        let mut h = 0.0f32;
        for ((g, t, role, tone), w) in pills.iter().zip(widths) {
            let (_, ph) = lay_pill(b, engine, *g, t, x, y, None, *role, *tone, p);
            h = h.max(ph);
            x += w + gap;
        }
        (total, h)
    }

    /// A NOTICE CARD: a symbol, a title and a few wrapped lines on the
    /// container — for states that have no page of their own («Kein Duo
    /// verbunden», «Noch nicht gekoppelt»). Returns the y below it.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_notice(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, glyph: char, title: &str, text: &str, x: f32, y: f32, w: f32, p: mk::Palette) -> f32 {
        let body = mk::typo::BODY;
        let lines = mk::wrap_words(text, ((w - 100.0) / (body.px() * 0.5)).max(16.0) as usize, 6);
        let line_h = body.px() * matrixkit_text::LINE_HEIGHT;
        let h = 70.0 + lines.len() as f32 * line_h;
        b.quad(x, y, w, h, p.surface_container, 18.0, None, None);
        b.text(engine, glyph.to_string(), mk::typo::symbol_px(30.0), mk::typo::Weight::Normal, crate::symbol_family(), p.primary, x + 22.0, y + 22.0, None, None);
        b.styled(engine, title, mk::typo::HEADLINE, p.on_surface, x + 72.0, y + 18.0, None, None);
        let mut yy = y + 50.0;
        for line in lines {
            b.styled(engine, line, body, p.on_surface_variant, x + 72.0, yy, None, None);
            yy += line_h;
        }
        y + h + mk::spacing::M
    }

    /// The height of a fact tile.
    pub const FACT_TILE_H: f32 = 86.0;

    /// A FACT TILE: a quiet label above, the value large below, and an
    /// optional bar (0..1) along its foot — storage, battery, any share.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_fact_tile(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, label: &str, value: &str, bar: Option<f32>, x: f32, y: f32, w: f32, p: mk::Palette) {
        let h = FACT_TILE_H;
        b.quad(x, y, w, h, p.surface_container, 16.0, None, None);
        let clip = Some(Rect { x, y, w, h });
        b.styled(engine, label, mk::typo::CAPTION, p.on_surface_variant, x + 16.0, y + 14.0, None, clip);
        b.styled(engine, value, mk::typo::TITLE, p.on_surface, x + 16.0, y + 36.0, None, clip);
        if let Some(frac) = bar {
            b.quad(x + 16.0, y + h - 16.0, w - 32.0, 6.0, p.outline.with_alpha(0.35), 3.0, None, None);
            b.quad(x + 16.0, y + h - 16.0, (w - 32.0) * frac.clamp(0.0, 1.0), 6.0, p.primary, 3.0, None, None);
        }
    }

    /// The geometry of a picture grid: (columns, side of a square tile,
    /// rows) for `n` pictures in `width`, tiles at least `min_side`. Pure.
    pub fn grid_geometry(n: usize, width: f32, min_side: f32, gap: f32) -> (usize, f32, usize) {
        let cols = ((width + gap) / (min_side + gap)).floor().max(1.0) as usize;
        let side = (width - gap * (cols as f32 - 1.0)) / cols as f32;
        (cols, side, n.div_ceil(cols))
    }

    /// A PICTURE GRID: square tiles (cover-fit, rounded) of `picts` in
    /// `area`, rolled by `roll`; tile k carries role `role_base + k`. Only
    /// the visible rows are laid. Returns the roll limit.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_pict_grid(b: &mut matrixkit_shell::SceneBuilder, picts: &[std::path::PathBuf], area: Rect, roll: f32, min_side: f32, gap: f32, role_base: usize, p: mk::Palette) -> f32 {
        let (cols, side, rows) = grid_geometry(picts.len(), area.w, min_side, gap);
        for (k, path) in picts.iter().enumerate() {
            let ty = area.y + (k / cols) as f32 * (side + gap) - roll;
            if ty + side < area.y || ty > area.y + area.h {
                continue;
            }
            let tx = area.x + (k % cols) as f32 * (side + gap);
            let anchor = b.quad(tx, ty, side, side, p.surface_container, 12.0, Some(role_base + k), Some(area));
            b.pict(matrixkit_shell::Pict {
                path: path.clone(),
                box_w: side as u32,
                box_h: side as u32,
                fit: matrixkit_image::Fit::Cover,
                x: tx,
                y: ty,
                radius: 12.0,
                home: Rect { x: tx, y: ty, w: side, h: side },
                anchor,
                clip: Some(area),
            });
        }
        (rows as f32 * (side + gap) - gap - area.h).max(0.0)
    }

    #[cfg(test)]
    mod round177_tests {
        use super::*;

        #[test]
        fn the_grid_fills_its_width() {
            let (cols, side, rows) = grid_geometry(10, 640.0, 150.0, 10.0);
            assert_eq!(cols, 4);
            assert!((cols as f32 * side + 3.0 * 10.0 - 640.0).abs() < 0.01, "tiles + gaps = width");
            assert_eq!(rows, 3);
            assert_eq!(grid_geometry(0, 640.0, 150.0, 10.0).2, 0);
            assert_eq!(grid_geometry(3, 100.0, 150.0, 10.0).0, 1, "never fewer than one column");
        }
    }

    /// ROW FIT (round 173): a row that can be laid at several densities
    /// (`widths`, widest first — full words, short words, symbols only …)
    /// takes the first that fits `room`; none fits: the last (densest).
    /// Tube's player row dropped its words step by step instead of
    /// squeezing them.
    pub fn row_fit(widths: &[f32], room: f32) -> usize {
        widths.iter().position(|w| *w <= room).unwrap_or(widths.len().saturating_sub(1))
    }

    #[cfg(test)]
    mod row_fit_tests {
        use super::*;

        #[test]
        fn the_widest_that_fits() {
            assert_eq!(row_fit(&[500.0, 400.0, 300.0], 600.0), 0);
            assert_eq!(row_fit(&[500.0, 400.0, 300.0], 450.0), 1);
            assert_eq!(row_fit(&[500.0, 400.0, 300.0], 100.0), 2, "none fits: the densest");
            assert_eq!(row_fit(&[], 100.0), 0);
        }
    }

    /// A value chip's natural width (what `lay_value_chip` takes when the
    /// value fits) — to right-align a row of chips.
    pub fn value_chip_width(engine: &mut TextEngine, glyph: char, value: &str, toggle: bool) -> f32 {
        let (pad, gap) = (mk::spacing::S + 2.0, mk::spacing::XS + 2.0);
        let symfam = crate::symbol_family();
        let gpx = mk::typo::symbol_px(18.0);
        let gw = engine.measure_family(symfam, &glyph.to_string(), gpx, mk::typo::Weight::Normal).0;
        let aw = if toggle { 0.0 } else { engine.measure_family(symfam, &crate::symbol::EXPAND_MORE.to_string(), gpx, mk::typo::Weight::Normal).0 + gap };
        let st = mk::typo::CALLOUT;
        pad + gw + gap + engine.measure_styled(value, st.px(), st.weight).0 + aw + pad
    }

    /// VALUE CHIP (R210 family): a compact chip — symbol, the current value
    /// and a fold arrow (down closed, up open) — for a setting whose choices
    /// unfold below on a tap; `toggle` Some(on) makes it a switch instead
    /// (no arrow, lifted when on). Returns (width, height). Tube's player
    /// row: quality, subtitles, autoplay.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_value_chip(
        b: &mut matrixkit_shell::SceneBuilder,
        engine: &mut TextEngine,
        glyph: char,
        value: &str,
        open: bool,
        toggle: Option<bool>,
        x: f32,
        y: f32,
        max_w: f32,
        role: usize,
        p: mk::Palette,
    ) -> (f32, f32) {
        let h = 36.0;
        let (pad, gap) = (mk::spacing::S + 2.0, mk::spacing::XS + 2.0);
        let lifted = open || toggle == Some(true);
        let (fill, ink) = if lifted { (p.primary_container, p.on_primary_container) } else { (p.surface_container_high, p.on_surface) };
        let symfam = crate::symbol_family();
        let gpx = mk::typo::symbol_px(18.0);
        let measure = |engine: &mut TextEngine, c: char| engine.measure_family(symfam, &c.to_string(), gpx, mk::typo::Weight::Normal);
        let (gw, gh) = measure(engine, glyph);
        let arrow = toggle.is_none().then_some(if open { crate::symbol::EXPAND_LESS } else { crate::symbol::EXPAND_MORE });
        let aw = arrow.map(|a| measure(engine, a).0 + gap).unwrap_or(0.0);
        let st = mk::typo::CALLOUT;
        let room = (max_w - 2.0 * pad - gw - gap - aw).max(24.0);
        let mut shown = value.to_string();
        let (mut tw, th) = engine.measure_styled(&shown, st.px(), st.weight);
        let mut kept: Vec<char> = value.chars().collect();
        while tw > room && kept.len() > 1 {
            kept.pop();
            shown = format!("{}\u{2026}", kept.iter().collect::<String>().trim_end());
            tw = engine.measure_styled(&shown, st.px(), st.weight).0;
        }
        let w = pad + gw + gap + tw + aw + pad;
        b.quad(x, y, w, h, fill, h / 2.0, Some(role), None);
        b.text(engine, glyph.to_string(), gpx, mk::typo::Weight::Normal, symfam, ink, x + pad, y + (h - gh) / 2.0, Some(role), None);
        b.styled(engine, shown, st, ink, x + pad + gw + gap, y + (h - th) / 2.0, Some(role), None);
        if let Some(a) = arrow {
            let (_, ah) = measure(engine, a);
            b.text(engine, a.to_string(), gpx, mk::typo::Weight::Normal, symfam, ink, x + w - pad - (aw - gap), y + (h - ah) / 2.0, Some(role), None);
        }
        (w, h)
    }

    /// SUBTITLE (R210 family): the lines of a cue centred at the bottom of a
    /// stage, white on a dark band each — readable over any picture. `role`
    /// lets a tap fall through to the stage (pause).
    pub fn lay_subtitle(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, stage: Rect, text: &str, role: Option<usize>) {
        let st = mk::typo::BODY;
        let white = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let chars = (((stage.w - 4.0 * mk::spacing::M) / (st.px() * 0.5)) as usize).max(16);
        let lines: Vec<String> = text.lines().flat_map(|l| mk::wrap_words(l, chars, 2)).take(3).collect();
        let line_h = st.px() * 1.35;
        let mut y = stage.y + stage.h - mk::spacing::M - line_h * lines.len() as f32;
        for line in lines {
            let (tw, th) = engine.measure_styled(&line, st.px(), st.weight);
            let x = stage.x + (stage.w - tw) / 2.0;
            b.quad(x - 6.0, y - 2.0, tw + 12.0, th + 4.0, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.66 }, 4.0, role, None);
            b.styled(engine, line, st, white, x, y, role, None);
            y += line_h;
        }
    }

    /// CHOICE PILLS (R210 family): a row of pills that wraps when the width
    /// runs out — the active one lifted on primary_container. Each pill is
    /// a target: role `role_base + i`. Returns the height used. (Tube:
    /// quality and subtitles in the player view.)
    #[allow(clippy::too_many_arguments)]
    pub fn lay_choices(
        b: &mut matrixkit_shell::SceneBuilder,
        engine: &mut TextEngine,
        options: &[String],
        active: Option<usize>,
        x: f32,
        y: f32,
        w: f32,
        role_base: usize,
        p: mk::Palette,
    ) -> f32 {
        let style = mk::typo::CALLOUT;
        let (pad_x, h, gap) = (mk::spacing::M, 36.0, mk::spacing::S);
        let (mut cx, mut cy) = (x, y);
        for (i, text) in options.iter().enumerate() {
            let on = active == Some(i);
            let (tw, th) = engine.measure_styled(text, style.px(), style.weight);
            let pw = (tw + 2.0 * pad_x).min(w);
            if cx > x && cx + pw > x + w {
                cx = x;
                cy += h + gap;
            }
            let (fill, ink) = if on { (p.primary_container, p.on_primary_container) } else { (p.surface_container_high, p.on_surface) };
            b.quad(cx, cy, pw, h, fill, h / 2.0, Some(role_base + i), None);
            b.styled(engine, text.clone(), style, ink, cx + (pw - tw.min(pw - 2.0 * pad_x)) / 2.0, cy + (h - th) / 2.0, Some(role_base + i), None);
            cx += pw + gap;
        }
        if options.is_empty() { 0.0 } else { cy - y + h }
    }

    /// PICTURE FIT (R210 family, duo grammar): a video window takes the
    /// shape of its picture — no black bars (owner 01.11.2026). It learns
    /// the area it had (the panel's) once the picture is known and asks for
    /// a fitted, floating, centred window once per aspect; `extra_h` is the
    /// window's own furniture below the picture (Player: its media bar).
    /// Only on devices with bare windows (duo); elsewhere it stays still.
    pub struct PictureFit {
        app_id: &'static str,
        area: Option<(u32, u32)>,
        /// Fitted for: picture size and the room taken below (px).
        fitted_to: Option<(u32, u32, u32)>,
        /// The area seen last and how many builds in a row — the window
        /// opens small and moves to its panel; only a settled area counts.
        seen: ((u32, u32), u32),
        /// Stepped back into the tiled layout (another app in front).
        back: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl PictureFit {
        /// Builds in a row with the same area before it is learnt.
        const SETTLED: u32 = 12;

        pub fn new(app_id: &'static str) -> Self {
            PictureFit { app_id, area: None, fitted_to: None, seen: ((0, 0), 0), back: Default::default() }
        }

        /// The window's focus changed (`Client::focus`). Losing it to ANOTHER
        /// window on the same panel, the fitted (floating) window steps back
        /// into the tiled layout — the new app is in front, the video plays
        /// on (03.11.2026). Getting it back, the window takes its picture
        /// shape again: true = rebuild (the next `follow` refits).
        pub fn focus(&mut self, on: bool) -> bool {
            use std::sync::atomic::Ordering;
            if !bare_windows() || self.fitted_to.is_none() && !self.back.load(Ordering::Relaxed) {
                return false;
            }
            if on {
                if self.back.swap(false, Ordering::Relaxed) {
                    self.fitted_to = None;
                    return true;
                }
                return false;
            }
            let (app, back) = (self.app_id, self.back.clone());
            std::thread::spawn(move || {
                // The newcomer needs a moment to be the focused window.
                std::thread::sleep(std::time::Duration::from_millis(200));
                if mk::canvas::other_window_in_front(app) && mk::canvas::tile_own_window(app) {
                    eprintln!("[{app}] tritt zurück: ein anderes Fenster ist vorne");
                    back.store(true, Ordering::Relaxed);
                }
            });
            false
        }

        /// Call while building: the picture's area now (w × h, logical) and
        /// the picture's size once known; `taken_below` = room something
        /// else claims from the bottom (a subtitle strip taller than the
        /// dock). True when a fit was asked for.
        pub fn follow(&mut self, area_w: f32, area_h: f32, picture: Option<(u32, u32)>, extra_h: f32, taken_below: f32) -> bool {
            if !bare_windows() {
                return false;
            }
            let Some((pw, ph)) = picture.filter(|(w, h)| *w > 0 && *h > 0) else { return false };
            if self.area.is_none() {
                let now = (area_w as u32, area_h as u32);
                self.seen = if self.seen.0 == now { (now, self.seen.1 + 1) } else { (now, 1) };
                if self.seen.1 < Self::SETTLED {
                    return false;
                }
                self.area = Some(now);
            }
            let Some((aw, ah)) = self.area else { return false };
            let below = taken_below.max(0.0) as u32;
            if self.fitted_to == Some((pw, ph, below)) {
                return false;
            }
            let first = self.fitted_to.is_none();
            self.fitted_to = Some((pw, ph, below));
            let room_h = ah.saturating_sub(below).max(40);
            let fit = letterbox(Rect { x: 0.0, y: 0.0, w: aw as f32, h: (room_h as f32 - extra_h).max(20.0) }, pw, ph);
            let (fw, fh) = (fit.w as u32, fit.h as u32);
            // Nearly a fit and nothing below: leave the window as it is.
            if first && below == 0 && fw + 4 >= aw && fh + extra_h as u32 + 4 >= ah {
                return false;
            }
            let (app, total_h) = (self.app_id, fh + extra_h.max(0.0) as u32);
            eprintln!("[{app}] Fenster in Bildform: {fw}×{total_h} (Fläche {aw}×{ah})");
            std::thread::spawn(move || mk::canvas::fit_own_window(app, fw, total_h));
            true
        }
    }

    /// CHOICE SECTION (R210 family): a quiet caption and its choice pills
    /// below — one setting of a settings sheet (Tube: quality, subtitles,
    /// subtitle size). Returns the height used, including the air after it.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_choice_section(
        b: &mut matrixkit_shell::SceneBuilder,
        engine: &mut TextEngine,
        caption: &str,
        options: &[String],
        active: Option<usize>,
        x: f32,
        y: f32,
        w: f32,
        role_base: usize,
        p: mk::Palette,
    ) -> f32 {
        let (_, ch) = b.styled(engine, caption, mk::typo::CAPTION, p.on_surface_variant, x, y, None, None);
        let pills = lay_choices(b, engine, options, active, x, y + ch + mk::spacing::XS, w, role_base, p);
        ch + mk::spacing::XS + pills + mk::spacing::M
    }

    /// A playing time as text: `m:ss`, from an hour on `h:mm:ss`.
    pub fn clock_text(s: f32) -> String {
        let s = s.max(0.0) as u32;
        if s >= 3600 {
            format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
        } else {
            format!("{}:{:02}", s / 60, s % 60)
        }
    }

    /// LETTERBOX (R210 family): the largest box of the picture's aspect
    /// inside `stage`, centred — a video on black velvet.
    pub fn letterbox(stage: Rect, pic_w: u32, pic_h: u32) -> Rect {
        if pic_w == 0 || pic_h == 0 || stage.w < 2.0 || stage.h < 2.0 {
            return Rect { x: stage.x, y: stage.y, w: 0.0, h: 0.0 };
        }
        let scale = (stage.w / pic_w as f32).min(stage.h / pic_h as f32);
        let (w, h) = ((pic_w as f32 * scale).floor(), (pic_h as f32 * scale).floor());
        Rect { x: (stage.x + (stage.w - w) / 2.0).floor(), y: (stage.y + (stage.h - h) / 2.0).floor(), w, h }
    }

    /// A DOTTED LINE (R211 family, 02.11.2026): any slant from round dots —
    /// the house's quads are upright, a line of dots is not (legs, threads,
    /// connectors). `gap` between dot centres, `size` = dot diameter.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_dotted_line(b: &mut matrixkit_shell::SceneBuilder, x0: f32, y0: f32, x1: f32, y1: f32, gap: f32, size: f32, color: mk::Rgba, clip: Option<Rect>) {
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len = (dx * dx + dy * dy).sqrt();
        let n = ((len / gap.max(0.5)).ceil() as usize).max(1);
        for i in 0..=n {
            let t = i as f32 / n as f32;
            b.quad(x0 + dx * t - size / 2.0, y0 + dy * t - size / 2.0, size, size, color, size / 2.0, None, clip);
        }
    }

    /// One round dot centred on (x, y).
    pub fn lay_dot(b: &mut matrixkit_shell::SceneBuilder, x: f32, y: f32, size: f32, color: mk::Rgba, clip: Option<Rect>) {
        b.quad(x - size / 2.0, y - size / 2.0, size, size, color, size / 2.0, None, clip);
    }

    /// The neon of a species (`mk::night_glass`).
    pub fn species_color(s: crate::crawl::Species) -> mk::Rgba {
        use crate::crawl::Species;
        match s {
            Species::Langleser => mk::night_glass::AMBER,
            Species::Weberin => mk::night_glass::ROSE,
            Species::Filler => mk::night_glass::CYAN,
            Species::Funke => mk::night_glass::LIME,
        }
    }

    /// A CRITTER, DRAWN (R211 family): dotted legs with knee and foot dots,
    /// wings and glow (Funke), dotted spine with framed boxes, eyes. `off`
    /// shifts its world into the window, `dim` < 1 when it sleeps.
    pub fn lay_critter(b: &mut matrixkit_shell::SceneBuilder, c: &crate::crawl::Critter, off: crate::crawl::V2, dim: f32, ground: mk::Rgba, clip: Option<Rect>) {
        use crate::crawl::{ik2, Species, V2};
        let col = species_color(c.species);
        let line = mk::Rgba { a: 0.55 * dim, ..col };
        let solid = mk::Rgba { a: dim, ..col };
        let (_, _, size) = c.species.body();
        let at = |p: V2| p.add(off);
        for leg in &c.legs {
            let hip = at(leg.hip(&c.spine, size / 2.0));
            let foot = at(leg.foot);
            let knee = ik2(hip, foot, leg.thigh, leg.shin, at(c.spine.joints[leg.joint]));
            lay_dotted_line(b, hip.x, hip.y, knee.x, knee.y, 3.4, 1.7, line, clip);
            lay_dotted_line(b, knee.x, knee.y, foot.x, foot.y, 3.4, 1.7, line, clip);
            lay_dot(b, knee.x, knee.y, 3.0, solid, clip);
            lay_dot(b, foot.x, foot.y, 3.4, solid, clip);
        }
        if c.species == Species::Funke {
            let h = c.spine.heading(0);
            let beat = c.wing.sin() * 0.6;
            for side in [1.0f32, -1.0] {
                let base = at(c.spine.joints[0].sub(h.mul(3.0)));
                let tip = base.add(V2::from_angle(h.angle() + side * (2.0 + beat)).mul(14.0));
                lay_dotted_line(b, base.x, base.y, tip.x, tip.y, 2.2, 2.0, mk::Rgba { a: 0.7 * dim, ..col }, clip);
            }
            let p = at(c.spine.joints[1]);
            let glow = 0.18 + 0.12 * (c.wing * 0.08).sin();
            lay_dot(b, p.x, p.y, 22.0, mk::Rgba { a: glow * dim, ..col }, clip);
        }
        for w in c.spine.joints.windows(2) {
            let (p, q) = (at(w[0]), at(w[1]));
            lay_dotted_line(b, p.x, p.y, q.x, q.y, 4.0, 1.7, line, clip);
        }
        for (j, p) in c.spine.joints.iter().enumerate().rev() {
            let p = at(*p);
            let s = if j == 0 { size + 2.0 } else if c.species == Species::Weberin && j == 1 { size + 5.0 } else { size };
            b.quad(p.x - s / 2.0, p.y - s / 2.0, s, s, solid, 2.5, None, clip);
            b.quad(p.x - s / 2.0 + 1.5, p.y - s / 2.0 + 1.5, s - 3.0, s - 3.0, ground, 1.5, None, clip);
            let core = if j == 0 { s - 6.0 } else { (s - 7.0).max(2.0) };
            b.quad(p.x - core / 2.0, p.y - core / 2.0, core, core, mk::Rgba { a: 0.8 * dim, ..col }, 1.0, None, clip);
        }
        let h = c.spine.heading(0);
        let head = at(c.spine.joints[0]);
        for side in [1.0f32, -1.0] {
            let e = head.add(h.mul(size * 0.55)).add(h.normal().mul(side * size * 0.35));
            lay_dot(b, e.x, e.y, 2.6, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.9 * dim }, clip);
        }
    }

    /// Its load: each carried word on a dotted thread, as a tag.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_critter_load(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, c: &crate::crawl::Critter, words: &[String], off: crate::crawl::V2, dim: f32, ground: mk::Rgba, clip: Option<Rect>) {
        let col = species_color(c.species);
        for (k, word) in words.iter().enumerate() {
            let p = c.slot(k).add(off);
            let (j, _) = c.slot_joint(k);
            let from = c.spine.joints[j].add(off);
            lay_dotted_line(b, from.x, from.y, p.x, p.y, 3.2, 1.3, mk::Rgba { a: 0.35 * dim, ..col }, clip);
            lay_tag(b, engine, word, mk::typo::CAPTION, p.x, p.y, mk::Rgba { a: 0.9 * dim, ..col }, mk::Rgba { a: 0.92, ..ground }, clip);
        }
    }

    /// WORD SPLITTER (R211 family): the words of a scene text with their
    /// byte range, x offset and width — measured once per string and style,
    /// cached. For anything that touches an app's words one by one: the
    /// Krabbler, a tap-to-look-up, a reading aid.
    #[derive(Default)]
    pub struct WordSplitter {
        cache: std::collections::HashMap<(String, u32, &'static str, String), (Vec<WordSpan>, f32)>,
    }

    /// One word of a text: `text[start..end]`, at `x` (from the text's x), `w` wide.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct WordSpan {
        pub start: usize,
        pub end: usize,
        pub x: f32,
        pub w: f32,
    }

    impl WordSplitter {
        /// The words of `t` and the text's height.
        pub fn split(&mut self, engine: &mut TextEngine, t: &matrixkit_shell::Text) -> (Vec<WordSpan>, f32) {
            let key = (t.s.clone(), t.px.to_bits(), t.family, format!("{:?}", t.weight));
            if let Some(v) = self.cache.get(&key) {
                return v.clone();
            }
            if self.cache.len() > 4000 {
                self.cache.clear();
            }
            let (_, h) = engine.measure_family(t.family, &t.s, t.px, t.weight);
            let mut words = Vec::new();
            for (start, end) in word_ranges(&t.s) {
                let x = if start == 0 { 0.0 } else { engine.measure_family(t.family, &t.s[..start], t.px, t.weight).0 };
                let w = engine.measure_family(t.family, &t.s[start..end], t.px, t.weight).0;
                words.push(WordSpan { start, end, x, w });
            }
            self.cache.insert(key, (words.clone(), h));
            (words, h)
        }
    }

    /// Byte ranges of the whitespace-separated words of `s` (pure — tested).
    pub fn word_ranges(s: &str) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut start = None;
        for (i, ch) in s.char_indices().chain(std::iter::once((s.len(), ' '))) {
            match (start, ch.is_whitespace()) {
                (None, false) => start = Some(i),
                (Some(a), true) => {
                    out.push((a, i));
                    start = None;
                }
                _ => {}
            }
        }
        out
    }

    /// CONTROLS (R211 family): which roles of a scene are controls (button,
    /// chip, field) and which are content (rows, cards) — judged by the
    /// tallest shape of each role: below `CONTROL_MAX` it is a control.
    /// The Krabbler leave controls' words alone; a reading aid could skip
    /// them, an accessibility walk could announce them as buttons.
    pub struct Controls(std::collections::HashMap<usize, f32>);

    pub const CONTROL_MAX: f32 = 56.0;

    impl Controls {
        pub fn of(scene: &matrixkit_shell::Scene) -> Controls {
            let mut tallest: std::collections::HashMap<usize, f32> = Default::default();
            for (q, r) in scene.placed.iter().zip(&scene.roles) {
                if let Some(r) = r {
                    let h = tallest.entry(*r).or_insert(0.0);
                    *h = h.max(q.h);
                }
            }
            Controls(tallest)
        }

        /// Is the shape at `anchor` (a text's anchor) part of a control?
        pub fn is_control(&self, scene: &matrixkit_shell::Scene, anchor: usize) -> bool {
            scene.roles.get(anchor).copied().flatten().is_some_and(|r| self.0.get(&r).is_some_and(|h| *h < CONTROL_MAX))
        }
    }

    /// KRABBLER OVER ANY APP (R211 family, the owner 02.11.2026): an overlay
    /// (`shell::Overlay`) that lets the species walk over the app's own
    /// words. Switched by the top bar's card (`mk::crawlers`): `crawl …`
    /// in the mailbox. A creature takes a word out of the app's text (the
    /// text is split around it), carries it on a thread and lets it fly
    /// home when the load grows too heavy; a tap on a creature shoos it
    /// (and the app does not get that tap). Two quiet minutes and they
    /// sleep — no frames. Apps opt in with `fam::run`.
    pub struct Crawlers {
        on: Vec<crate::crawl::Species>,
        critters: Vec<Roamer>,
        /// The words of the last scene.
        seen: Vec<SeenWord>,
        /// Words away from their text: key -> carrying critter.
        away: std::collections::HashMap<WordKey, usize>,
        /// Words some critter walks to.
        wanted: std::collections::HashMap<WordKey, usize>,
        /// Words resting after their flight home (clock time).
        resting: std::collections::HashMap<WordKey, f32>,
        flights: Vec<Flight>,
        /// The app's texts split into words (`WordSplitter`).
        splits: WordSplitter,
        size: (f32, f32),
        clock: f32,
        dice: crate::crawl::Dice,
        doze: mk::Doze,
        /// Seconds since the last drawn frame (30 a second).
        since_frame: f32,
    }

    /// A word: which text (hash of its string + n-th such text), which word.
    type WordKey = (u64, u32, u16);

    struct SeenWord {
        key: WordKey,
        text: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    }

    struct Roamer {
        body: crate::crawl::Critter,
        goal: Option<WordKey>,
        wander: crate::crawl::V2,
        wander_until: f32,
        flee_until: f32,
        load: std::collections::VecDeque<(WordKey, String)>,
    }

    struct Flight {
        key: WordKey,
        text: String,
        from: crate::crawl::V2,
        t: f32,
    }

    impl Default for Crawlers {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Crawlers {
        pub fn new() -> Crawlers {
            Crawlers {
                on: Vec::new(),
                critters: Vec::new(),
                seen: Vec::new(),
                away: Default::default(),
                wanted: Default::default(),
                resting: Default::default(),
                flights: Vec::new(),
                splits: WordSplitter::default(),
                size: (400.0, 400.0),
                clock: 0.0,
                dice: crate::crawl::Dice::new(std::process::id() as u64 * 7919 + 13),
                doze: mk::Doze::new(120.0),
                since_frame: 0.0,
            }
        }

        /// Switch to these species (empty = all leave, their words fly home).
        pub fn set(&mut self, on: Vec<crate::crawl::Species>) {
            use crate::crawl::{Critter, V2};
            // Leaving creatures drop their load.
            let mut keep = Vec::new();
            for r in std::mem::take(&mut self.critters) {
                if on.contains(&r.body.species) {
                    keep.push(r);
                } else {
                    for (k, (key, text)) in r.load.iter().enumerate() {
                        self.flights.push(Flight { key: *key, text: text.clone(), from: r.body.slot(k), t: 0.0 });
                    }
                }
            }
            self.critters = keep;
            for s in &on {
                if !self.critters.iter().any(|r| r.body.species == *s) {
                    let (w, h) = self.size;
                    let at = V2::new(self.dice.range(40.0, (w - 40.0).max(41.0)), self.dice.range(40.0, (h - 40.0).max(41.0)));
                    let heading = self.dice.range(-3.1, 3.1);
                    self.critters.push(Roamer { body: Critter::new(*s, at, heading), goal: None, wander: at, wander_until: 0.0, flee_until: 0.0, load: Default::default() });
                }
            }
            // Indices changed: rebuild who carries / wants what.
            self.wanted.clear();
            self.away.clear();
            for (i, r) in self.critters.iter_mut().enumerate() {
                r.goal = None;
                for (key, _) in &r.load {
                    self.away.insert(*key, i);
                }
            }
            for f in &self.flights {
                self.away.insert(f.key, usize::MAX);
            }
            self.on = on;
            self.doze.touch();
            mk::crawlers::publish(&self.on.iter().map(|s| s.key().to_string()).collect::<Vec<_>>());
            eprintln!("[krabbler] in dieser App: {:?}", self.on.iter().map(|s| s.name()).collect::<Vec<_>>());
        }

        fn active(&self) -> bool {
            !self.critters.is_empty() || !self.flights.is_empty()
        }

        /// One frame of the world.
        fn step(&mut self, dt: f32) {
            use crate::crawl::V2;
            self.clock += dt;
            self.doze.tick(dt);
            let (w, h) = self.size;
            let (lo, hi) = (V2::new(8.0, 8.0), V2::new((w - 8.0).max(9.0), (h - 8.0).max(9.0)));
            let asleep = self.doze.asleep();
            for i in 0..self.critters.len() {
                let fleeing = self.clock < self.critters[i].flee_until;
                // An aim: the wanted word if it is still there, else a new one.
                let goal_pos = self.critters[i].goal.and_then(|k| self.seen.iter().find(|s| s.key == k)).map(|s| V2::new(s.x + s.w / 2.0, s.y + s.h / 2.0));
                if goal_pos.is_none() {
                    if let Some(k) = self.critters[i].goal.take() {
                        self.wanted.remove(&k);
                    }
                    if !fleeing {
                        let sp = self.critters[i].body.species;
                        let head = self.critters[i].body.head();
                        let pick = self
                            .seen
                            .iter()
                            .filter(|s| sp.likes(&s.text) && !self.away.contains_key(&s.key) && !self.wanted.contains_key(&s.key) && self.resting.get(&s.key).map_or(true, |t| self.clock >= *t))
                            .map(|s| (s.key, V2::new(s.x + s.w / 2.0, s.y + s.h / 2.0).dist(head)))
                            .filter(|(_, d)| *d < 520.0)
                            .min_by(|a, b| a.1.total_cmp(&b.1));
                        if let Some((k, _)) = pick {
                            self.wanted.insert(k, i);
                            self.critters[i].goal = Some(k);
                        }
                    }
                }
                let goal_pos = self.critters[i].goal.and_then(|k| self.seen.iter().find(|s| s.key == k)).map(|s| V2::new(s.x + s.w / 2.0, s.y + s.h / 2.0));
                let r = &mut self.critters[i];
                if goal_pos.is_none() && (self.clock > r.wander_until || r.body.head().dist(r.wander) < 12.0) {
                    r.wander = V2::new(self.dice.range(lo.x, hi.x), self.dice.range(lo.y, hi.y));
                    r.wander_until = self.clock + self.dice.range(3.0, 7.0);
                }
                let target = goal_pos.unwrap_or(r.wander);
                let (pace, _) = r.body.species.pace();
                let dist = r.body.head().dist(target);
                let want = if asleep { 0.0 } else if fleeing { pace * 2.4 } else { pace * (dist / 30.0).clamp(0.35, 1.0) };
                let sway = if r.body.species == crate::crawl::Species::Funke { (self.clock * 3.0 + i as f32).sin() * 0.9 } else { 0.0 };
                if r.body.steer(target, want, dt, sway, if fleeing { 2.0 } else { 1.0 }, lo, hi) {
                    r.wander_until = 0.0;
                }
                // Reached: the word leaves its text and rides along.
                if let (Some(k), Some(p)) = (r.goal, goal_pos) {
                    if !fleeing && r.body.head().dist(p) < 10.0 {
                        let text = self.seen.iter().find(|s| s.key == k).map(|s| s.text.clone()).unwrap_or_default();
                        r.goal = None;
                        self.wanted.remove(&k);
                        self.away.insert(k, i);
                        r.load.push_back((k, text));
                        if r.load.len() > r.body.species.cargo() {
                            let from = r.body.slot(0);
                            if let Some((old, text)) = r.load.pop_front() {
                                self.away.insert(old, usize::MAX);
                                self.flights.push(Flight { key: old, text, from, t: 0.0 });
                            }
                        }
                    }
                }
            }
            // Words flying home land and rest.
            let clock = self.clock;
            let mut landed = Vec::new();
            self.flights.retain_mut(|f| {
                f.t += dt / 0.9;
                if f.t >= 1.0 {
                    landed.push(f.key);
                    false
                } else {
                    true
                }
            });
            for k in landed {
                self.away.remove(&k);
                self.resting.insert(k, clock + 8.0);
            }
            if self.resting.len() > 2000 {
                self.resting.retain(|_, t| *t > clock);
            }
        }
    }

    impl matrixkit_shell::Overlay for Crawlers {
        fn word(&mut self, word: &str) -> bool {
            let Some(keys) = mk::crawlers::parse(word) else { return false };
            self.set(keys.iter().filter_map(|k| crate::crawl::Species::from_key(k)).collect());
            true
        }

        fn closing(&mut self) {
            // The card must not read a state of a window that is gone.
            mk::crawlers::publish(&[]);
        }

        fn moving(&self) -> bool {
            self.active() && (!self.doze.asleep() || !self.flights.is_empty() || self.critters.iter().any(|r| r.body.moving()))
        }

        fn glide(&mut self, dt: f32) -> bool {
            // Moved every tick, drawn 30 times a second: the whole app is
            // rebuilt per frame (Einstellungen: 56 % CPU at 45–60 frames).
            self.step(dt);
            self.since_frame += dt;
            if self.since_frame >= 1.0 / 30.0 {
                self.since_frame = 0.0;
                return true;
            }
            false
        }

        fn strike(&mut self, x: f32, y: f32) -> bool {
            use crate::crawl::V2;
            if !self.active() {
                return false;
            }
            self.doze.touch();
            let p = V2::new(x, y);
            let Some(i) = self.critters.iter().position(|r| r.body.hit(p)) else { return false };
            // Shooed: the load flies home, it runs away.
            let r = &mut self.critters[i];
            for (k, (key, text)) in r.load.iter().enumerate() {
                self.flights.push(Flight { key: *key, text: text.clone(), from: r.body.slot(k), t: 0.0 });
                self.away.insert(*key, usize::MAX);
            }
            r.load.clear();
            if let Some(k) = r.goal.take() {
                self.wanted.remove(&k);
            }
            let away = r.body.head().add(r.body.head().sub(p).unit().mul(260.0));
            r.wander = away;
            r.wander_until = self.clock + 1.6;
            r.flee_until = self.clock + 1.6;
            r.body.heading = away.sub(r.body.head()).angle();
            mk::feel::tap();
            true
        }

        fn lay(&mut self, engine: &mut TextEngine, mut scene: matrixkit_shell::Scene, w: u32, h: u32) -> matrixkit_shell::Scene {
            use crate::crawl::V2;
            self.size = (w as f32, h as f32);
            if !self.active() {
                return scene;
            }
            // 1. The app's words, and the texts split around those away.
            let symbols = crate::symbol_family();
            let mut seen = Vec::new();
            let mut texts = Vec::with_capacity(scene.texts.len());
            let mut count: std::collections::HashMap<u64, u32> = Default::default();
            let controls = Controls::of(&scene);
            for t in std::mem::take(&mut scene.texts) {
                // Words on controls stay (a button, chip or field must keep
                // its label — the first probe ate Tube's tabs and its search
                // placeholder): a text on a pressable shape lower than a row.
                // Measured on the tallest shape of the same role: a row
                // (76 px) is content, a chip (36) or field (44) a control.
                let on_control = controls.is_control(&scene, t.anchor);
                if t.family == symbols || t.px < 9.0 || t.s.trim().is_empty() || on_control {
                    texts.push(t);
                    continue;
                }
                let hash = {
                    use std::hash::{Hash, Hasher};
                    let mut hs = std::collections::hash_map::DefaultHasher::new();
                    t.s.hash(&mut hs);
                    hs.finish()
                };
                let n = count.entry(hash).or_insert(0);
                let nth = *n;
                *n += 1;
                let (words, th) = self.splits.split(engine, &t);
                let visible = |x: f32, y: f32| t.clip.map_or(true, |c| x >= c.x && x <= c.x + c.w && y >= c.y && y <= c.y + c.h);
                let mut gone = Vec::new();
                for (wi, WordSpan { start: a, end: b2, x, w: ww }) in words.iter().enumerate() {
                    let key = (hash, nth, wi as u16);
                    let (wx, wy) = (t.x + x, t.y);
                    if self.away.contains_key(&key) {
                        gone.push(wi);
                    }
                    if visible(wx + ww / 2.0, wy + th / 2.0) && wx >= 0.0 && wy >= 0.0 && wx < w as f32 && wy < h as f32 {
                        seen.push(SeenWord { key, text: letters(&t.s[*a..*b2]), x: wx, y: wy, w: *ww, h: th });
                    }
                }
                if gone.is_empty() {
                    texts.push(t);
                    continue;
                }
                // Runs between the words that are away; a faint gap for each.
                let mut run_start: Option<usize> = None;
                for wi in 0..=words.len() {
                    let missing = wi == words.len() || gone.contains(&wi);
                    match (run_start, missing) {
                        (None, false) => run_start = Some(wi),
                        (Some(a), true) => {
                            let (sa, xa) = (words[a].start, words[a].x);
                            let eb = words[wi - 1].end;
                            let mut piece = t.clone();
                            piece.s = t.s[sa..eb].to_string();
                            piece.x = t.x + xa;
                            texts.push(piece);
                            run_start = None;
                        }
                        _ => {}
                    }
                }
                for wi in gone {
                    let (x, ww) = (words[wi].x, words[wi].w);
                    scene.placed.push(matrixkit_theme::scene::Placed { x: t.x + x, y: t.y + th * 0.25, w: ww, h: th * 0.5, color: mk::night_glass::HAZE, radius: 3.0, clip: t.clip, shadow: None });
                    scene.roles.push(None);
                }
            }
            scene.texts = texts;
            self.seen = seen;
            // 2. On top: wanted frames, loads, critters, flights.
            let mut b = matrixkit_shell::SceneBuilder::new();
            b.scene = scene;
            let ground = mk::night_glass::GROUND;
            let dim = if self.doze.asleep() { 0.45 } else { 1.0 };
            let origin = V2::new(0.0, 0.0);
            for (key, i) in &self.wanted {
                if let (Some(s), Some(r)) = (self.seen.iter().find(|s| s.key == *key), self.critters.get(*i)) {
                    let col = species_color(r.body.species);
                    b.quad(s.x - 3.0, s.y - 1.0, s.w + 6.0, s.h + 2.0, mk::Rgba { a: 0.8, ..col }, 3.0, None, None);
                    b.quad(s.x - 2.0, s.y, s.w + 4.0, s.h, mk::Rgba { a: 0.85, ..ground }, 2.0, None, None);
                }
            }
            for r in &self.critters {
                let words: Vec<String> = r.load.iter().map(|(_, t)| t.clone()).collect();
                lay_critter_load(&mut b, engine, &r.body, &words, origin, dim, ground, None);
            }
            for r in &self.critters {
                lay_critter(&mut b, &r.body, origin, dim, ground, None);
            }
            for f in &self.flights {
                let home = self.seen.iter().find(|s| s.key == f.key).map(|s| V2::new(s.x + s.w / 2.0, s.y + s.h / 2.0)).unwrap_or(f.from);
                let e = 1.0 - (1.0 - f.t) * (1.0 - f.t);
                let p = f.from.lerp(home, e);
                lay_tag(&mut b, engine, &f.text, mk::typo::CAPTION, p.x, p.y, mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.9 }, mk::Rgba { a: 0.85, ..ground }, None);
            }
            b.finish()
        }
    }

    /// Run a house app with Krabbler over it (`Crawlers`, switched from the
    /// top bar's app card) — `shell::run` for every app that wants them.
    pub fn run<C: matrixkit_shell::Client + 'static>(client: C, options: matrixkit_shell::Options) -> Result<(), Box<dyn std::error::Error>> {
        matrixkit_shell::run_with(client, options, Some(Box::new(Crawlers::new())))
    }

    /// A NAVIGATION SIDEBAR with groups (R214 family — out of Matrix
    /// Einstellungen, the owner 02.11.2026: «Verbindungen unfolds as a
    /// group»): entries with a symbol; the chosen one stands on the accent
    /// container; an entry with parts shows ›, and ⌄ with its parts
    /// indented below while it is chosen; `dot` marks an entry (e.g. «not
    /// moved yet»). Roles: `entry_base + i`, `part_base + k`.
    pub struct NavEntry<'a> {
        pub title: &'a str,
        pub glyph: char,
        pub parts: &'a [(&'a str, char)],
        pub dot: bool,
    }

    /// Lay the sidebar into `area`, scrolled by `roll`. `selected` (index
    /// into `entries`) and its chosen `part`; `unfolded`: the selected
    /// group shows its parts (false = folded again — round 25, the owner:
    /// «be able to fold groups in again»). Returns the content height
    /// (for the scroll limit).
    #[allow(clippy::too_many_arguments)]
    pub fn lay_nav(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, entries: &[NavEntry], selected: Option<usize>, part: Option<usize>, unfolded: bool, area: Rect, roll: f32, entry_base: usize, part_base: usize, p: mk::Palette) -> f32 {
        let clip = Some(area);
        let (m, s) = (mk::spacing::M, mk::spacing::S);
        let family = crate::symbol_family();
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let top = area.y + s - roll;
        let mut y = top;
        for (i, e) in entries.iter().enumerate() {
            let chosen = selected == Some(i);
            let open = chosen && unfolded;
            // A group with a chosen part: its head is not the highlight.
            let on = chosen && part.is_none();
            let role = Some(entry_base + i);
            let h = 40.0;
            b.quad(area.x + s, y, area.w - 2.0 * s, h, if on { p.primary_container } else { clear }, mk::radius::SMALL, role, clip);
            let ink = if on { p.on_primary_container } else { p.on_surface };
            let px = mk::typo::symbol_px(20.0);
            let g = e.glyph.to_string();
            let (_, gh) = engine.measure_family(family, &g, px, mk::typo::Weight::Normal);
            b.text(engine, g, px, mk::typo::Weight::Normal, family, if on { ink } else { p.on_surface_variant }, area.x + m + 4.0, y + (h - gh) / 2.0, role, clip);
            let style = if on { mk::typo::HEADLINE } else { mk::typo::BODY };
            let (_, th) = engine.measure_styled(e.title, style.px(), style.weight);
            b.styled(engine, e.title, style, ink, area.x + m + 36.0, y + (h - th) / 2.0, role, clip);
            if !e.parts.is_empty() {
                let chev = if open { '\u{e5cf}' } else { crate::symbol::CHEVRON_RIGHT }.to_string();
                let (cw, ch) = engine.measure_family(family, &chev, px, mk::typo::Weight::Normal);
                b.text(engine, chev, px, mk::typo::Weight::Normal, family, if on { ink } else { p.on_surface_variant }, area.x + area.w - m - cw, y + (h - ch) / 2.0, role, clip);
            } else if e.dot {
                lay_dot(b, area.x + area.w - m - 6.0, y + h / 2.0, 6.0, p.outline.with_alpha(0.5), clip);
            }
            y += h + 2.0;
            if open {
                for (k, (name, glyph)) in e.parts.iter().enumerate() {
                    let sub_on = part == Some(k);
                    let role = Some(part_base + k);
                    let sh = 36.0;
                    let x0 = area.x + s + 22.0;
                    b.quad(x0, y, area.w - 2.0 * s - 22.0, sh, if sub_on { p.primary_container } else { clear }, mk::radius::SMALL, role, clip);
                    let sink = if sub_on { p.on_primary_container } else { p.on_surface };
                    let g = glyph.to_string();
                    let gpx = mk::typo::symbol_px(18.0);
                    let (_, gh) = engine.measure_family(family, &g, gpx, mk::typo::Weight::Normal);
                    b.text(engine, g, gpx, mk::typo::Weight::Normal, family, if sub_on { sink } else { p.on_surface_variant }, x0 + m - 4.0, y + (sh - gh) / 2.0, role, clip);
                    let st = if sub_on { mk::typo::HEADLINE } else { mk::typo::CALLOUT };
                    let (_, th) = engine.measure_styled(name, st.px(), st.weight);
                    b.styled(engine, *name, st, sink, x0 + m + 26.0, y + (sh - th) / 2.0, role, clip);
                    y += sh + 2.0;
                }
            }
        }
        y + s - top
    }

    /// A SWITCH, laid directly (R212 family): track 52×30, thumb 24 —
    /// the colours of `toggle`/`mkw::toggle_guarded`. Its track's index.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_switch(b: &mut matrixkit_shell::SceneBuilder, x: f32, y: f32, on: bool, locked: bool, role: Option<usize>, clip: Option<Rect>, p: mk::Palette) -> usize {
        let (tw, th) = (mk::control::TOGGLE_TRACK_W, mk::control::TOGGLE_TRACK_H);
        let mut track = if on { p.primary } else { p.on_surface.over(p.surface_container_high, 0.12) };
        let mut thumb = if on { p.on_primary } else { p.outline };
        if locked {
            track = track.over(p.surface_container_high, 0.45);
            thumb = thumb.over(p.surface_container_high, 0.45);
        }
        let q = b.quad(x, y, tw, th, track, th / 2.0, role, clip);
        let k = 24.0;
        let kx = if on { x + tw - 3.0 - k } else { x + 3.0 };
        b.quad(kx, y + (th - k) / 2.0, k, k, thumb, k / 2.0, role, clip);
        q
    }

    /// FORM PAGE (R212 family, the owner 02.11.2026: «settings without
    /// iced»): a settings page laid straight onto the scene builder — page
    /// heading, sections (LABEL title, one card, hairlines between rows),
    /// rows with a switch, a choice, a stepper, a slider, a value, a
    /// chevron or a button; notes between. Every row's whole width is its
    /// target (touch). The card's ground is laid first and stretched when
    /// the section ends (z-order without measuring twice).
    pub struct FormPage<'a> {
        pub b: &'a mut matrixkit_shell::SceneBuilder,
        x: f32,
        y: f32,
        w: f32,
        clip: Option<Rect>,
        p: mk::Palette,
        card: Option<(usize, f32, usize)>,
        /// (role, track rectangle) of every slider — a press or drag maps x.
        pub sliders: Vec<(usize, Rect)>,
    }

    /// Row padding and the minimum row height (a finger's target).
    pub const FORM_ROW_MIN: f32 = 48.0;

    impl<'a> FormPage<'a> {
        pub fn new(b: &'a mut matrixkit_shell::SceneBuilder, x: f32, y: f32, w: f32, clip: Option<Rect>, p: mk::Palette) -> FormPage<'a> {
            FormPage { b, x, y, w, clip, p, card: None, sliders: Vec::new() }
        }

        /// Where the next element goes.
        pub fn y(&self) -> f32 {
            self.y
        }

        pub fn gap(&mut self, h: f32) {
            self.y += h;
        }

        /// The page's heading (and a line below).
        pub fn heading(&mut self, engine: &mut TextEngine, title: &str, about: Option<&str>) {
            let (_, th) = self.b.styled(engine, title, mk::typo::LARGE_TITLE, self.p.on_surface, self.x, self.y, None, self.clip);
            self.y += th + 2.0;
            if let Some(a) = about {
                let chars = ((self.w / (mk::typo::CALLOUT.px() * 0.5)) as usize).max(20);
                for line in mk::wrap_words(a, chars, 3) {
                    let (_, lh) = self.b.styled(engine, line, mk::typo::CALLOUT, self.p.on_surface_variant, self.x, self.y, None, self.clip);
                    self.y += lh;
                }
            }
            self.y += mk::spacing::L;
        }

        /// A section: its title, then one card for the rows that follow.
        pub fn section(&mut self, engine: &mut TextEngine, title: &str) {
            self.end();
            if !title.is_empty() {
                let (_, th) = self.b.styled(engine, title.to_uppercase(), mk::typo::LABEL, self.p.on_surface_variant, self.x + mk::spacing::S, self.y, None, self.clip);
                self.y += th + mk::spacing::XS;
            }
            let ground = self.b.quad(self.x, self.y, self.w, 0.0, self.p.surface_container_high, mk::CORNER_RADIUS, None, self.clip);
            self.card = Some((ground, self.y, 0));
        }

        /// Close the open card (stretch its ground) and leave air.
        pub fn end(&mut self) {
            if let Some((ground, top, _)) = self.card.take() {
                if let Some(q) = self.b.scene.placed.get_mut(ground) {
                    q.h = self.y - top;
                }
                self.y += mk::spacing::L;
            }
        }

        /// A caption between sections (or under a card).
        pub fn note(&mut self, engine: &mut TextEngine, text: &str) {
            self.end();
            let cap = mk::typo::CAPTION;
            let chars = ((self.w / (cap.px() * 0.5)) as usize).max(20);
            self.y -= mk::spacing::S;
            for line in mk::wrap_words(text, chars, 4) {
                let (_, lh) = self.b.styled(engine, line, cap, self.p.on_surface_variant, self.x + mk::spacing::S, self.y, None, self.clip);
                self.y += lh;
            }
            self.y += mk::spacing::L;
        }

        /// A row's frame: hairline, whole-width target, leading symbol,
        /// title and description; returns (trailing area right edge, row
        /// top, row height). `trailing_w` keeps room for the right side.
        fn row(&mut self, engine: &mut TextEngine, role: Option<usize>, glyph: Option<char>, title: &str, desc: Option<&str>, trailing_w: f32, ink: mk::Rgba) -> (f32, f32, f32) {
            let p = self.p;
            let m = mk::spacing::M;
            if self.card.is_none() {
                self.section(engine, "");
            }
            if let Some((_, _, n)) = &mut self.card {
                if *n > 0 {
                    self.b.quad(self.x + m, self.y, self.w - m, 1.0, p.outline.over(p.surface_container_high, 0.18), 0.0, None, self.clip);
                }
                *n += 1;
            }
            let body = mk::typo::BODY;
            let cap = mk::typo::CAPTION;
            let lead = if glyph.is_some() { 28.0 + m } else { 0.0 };
            let text_w = (self.w - 2.0 * m - lead - trailing_w - if trailing_w > 0.0 { m } else { 0.0 }).max(60.0);
            let chars = ((text_w / (body.px() * 0.5)) as usize).max(10);
            let dchars = ((text_w / (cap.px() * 0.5)) as usize).max(12);
            let lines: Vec<String> = mk::wrap_words(title, chars, 2);
            let dlines: Vec<String> = desc.map(|d| mk::wrap_words(d, dchars, 3)).unwrap_or_default();
            let (_, lh) = engine.measure_styled("Hg", body.px(), body.weight);
            let (_, ch) = engine.measure_styled("Hg", cap.px(), cap.weight);
            let text_h = lines.len() as f32 * lh + dlines.len() as f32 * ch + if dlines.is_empty() { 0.0 } else { 2.0 };
            let h = (text_h + 2.0 * (mk::spacing::S + 2.0)).max(FORM_ROW_MIN);
            let top = self.y;
            self.b.quad(self.x, top, self.w, h, mk::Rgba { a: 0.0, ..p.surface }, mk::CORNER_RADIUS, role, self.clip);
            let mut tx = self.x + m;
            if let Some(g) = glyph {
                let family = crate::symbol_family();
                let px = mk::typo::symbol_px(22.0);
                let (gw, gh) = engine.measure_family(family, &g.to_string(), px, mk::typo::Weight::Normal);
                self.b.text(engine, g.to_string(), px, mk::typo::Weight::Normal, family, p.on_surface_variant, tx + (28.0 - gw) / 2.0, top + (h - gh) / 2.0, role, self.clip);
                tx += lead;
            }
            let mut ty = top + (h - text_h) / 2.0;
            for l in lines {
                self.b.styled(engine, l, body, ink, tx, ty, role, self.clip);
                ty += lh;
            }
            ty += 2.0;
            for l in dlines {
                self.b.styled(engine, l, cap, p.on_surface_variant, tx, ty, role, self.clip);
                ty += ch;
            }
            self.y += h;
            (self.x + self.w - m, top, h)
        }

        /// A switch row (the whole row switches).
        pub fn toggle(&mut self, engine: &mut TextEngine, role: usize, title: &str, desc: Option<&str>, on: bool, locked: bool) {
            let tw = mk::control::TOGGLE_TRACK_W;
            let (right, top, h) = self.row(engine, (!locked).then_some(role), None, title, desc, tw, self.p.on_surface);
            lay_switch(self.b, right - tw, top + (h - mk::control::TOGGLE_TRACK_H) / 2.0, on, locked, (!locked).then_some(role), self.clip, self.p);
        }

        /// A row with a value on the right (read only).
        pub fn value(&mut self, engine: &mut TextEngine, glyph: Option<char>, title: &str, desc: Option<&str>, value: &str) {
            let cap = mk::typo::CALLOUT;
            let value = mk::elide(value, 34);
            let (vw, vh) = engine.measure_styled(&value, cap.px(), cap.weight);
            let (right, top, h) = self.row(engine, None, glyph, title, desc, vw, self.p.on_surface);
            self.b.styled(engine, value, cap, self.p.on_surface_variant, right - vw, top + (h - vh) / 2.0, None, self.clip);
        }

        /// A row that leads somewhere (›), with an optional value.
        /// Round 95: a navigation row led by a PICTURE (an app's icon file)
        /// instead of a glyph — the bar's app list wears its icons.
        pub fn nav_pict(&mut self, engine: &mut TextEngine, role: usize, pict: Option<&std::path::Path>, title: &str, desc: Option<&str>) {
            let Some(path) = pict else {
                return self.nav(engine, role, None, title, desc, None);
            };
            let family = crate::symbol_family();
            let px = mk::typo::symbol_px(20.0);
            let chev = crate::symbol::CHEVRON_RIGHT.to_string();
            let (cw, chh) = engine.measure_family(family, &chev, px, mk::typo::Weight::Normal);
            // The row leaves the glyph's room (a blank lead), the picture fills it.
            let (right, top, h) = self.row(engine, Some(role), Some(' '), title, desc, cw, self.p.on_surface);
            self.b.text(engine, chev, px, mk::typo::Weight::Normal, family, self.p.on_surface_variant, right - cw, top + (h - chh) / 2.0, Some(role), self.clip);
            let s = 30.0;
            let (ix, iy) = (self.x + mk::spacing::M - 1.0, top + (h - s) / 2.0);
            let anchor = self.b.quad(ix, iy, s, s, mk::Rgba { a: 0.0, ..self.p.surface }, 7.0, Some(role), self.clip);
            let home = Rect::of(&self.b.scene.placed[anchor]);
            self.b.pict(matrixkit_shell::Pict {
                path: path.to_path_buf(),
                box_w: s as u32,
                box_h: s as u32,
                fit: matrixkit_image::Fit::Contain,
                x: ix,
                y: iy,
                radius: 7.0,
                home,
                anchor,
                clip: self.clip,
            });
        }

        pub fn nav(&mut self, engine: &mut TextEngine, role: usize, glyph: Option<char>, title: &str, desc: Option<&str>, value: Option<&str>) {
            let cap = mk::typo::CALLOUT;
            let family = crate::symbol_family();
            let px = mk::typo::symbol_px(20.0);
            let chev = crate::symbol::CHEVRON_RIGHT.to_string();
            let (cw, chh) = engine.measure_family(family, &chev, px, mk::typo::Weight::Normal);
            let vw = value.map(|v| engine.measure_styled(v, cap.px(), cap.weight).0 + mk::spacing::S).unwrap_or(0.0);
            let (right, top, h) = self.row(engine, Some(role), glyph, title, desc, cw + vw, self.p.on_surface);
            self.b.text(engine, chev, px, mk::typo::Weight::Normal, family, self.p.on_surface_variant, right - cw, top + (h - chh) / 2.0, Some(role), self.clip);
            if let Some(v) = value {
                let (_, vh) = engine.measure_styled(v, cap.px(), cap.weight);
                self.b.styled(engine, v, cap, self.p.on_surface_variant, right - cw - vw, top + (h - vh) / 2.0, Some(role), self.clip);
            }
        }

        /// A row with a button on the right (`tone` Accent = the main one).
        pub fn button(&mut self, engine: &mut TextEngine, role: usize, glyph: Option<char>, title: &str, desc: Option<&str>, label: &str, tone: PillTone) {
            let bw = pill_width(engine, None, label);
            let (right, top, h) = self.row(engine, None, glyph, title, desc, bw, self.p.on_surface);
            let y = top + (h - 40.0) / 2.0;
            if tone == PillTone::Quiet {
                // On the card a quiet pill would vanish (same colour as the
                // card): a lighter ground of its own.
                let p = self.p;
                let style = mk::typo::CALLOUT;
                self.b.quad(right - bw, y, bw, 40.0, p.on_surface.over(p.surface_container_high, 0.10), 20.0, Some(role), self.clip);
                let (tw, th) = engine.measure_styled(label, style.px(), style.weight);
                self.b.styled(engine, label.to_string(), style, p.on_surface, right - bw + (bw - tw) / 2.0, y + (40.0 - th) / 2.0, Some(role), self.clip);
            } else {
                lay_pill(self.b, engine, None, label, right - bw, y, Some(bw), Some(role), tone, self.p);
            }
        }

        /// A row in another ink (Schliessen in red, a warning), pressing as a whole.
        pub fn action(&mut self, engine: &mut TextEngine, role: usize, glyph: Option<char>, title: &str, desc: Option<&str>, ink: mk::Rgba) {
            self.row(engine, Some(role), glyph, title, desc, 0.0, ink);
        }

        /// A choice among a few (pills in a track): beside the title when
        /// they fit, else below it. Roles `role_base + i`.
        pub fn choice(&mut self, engine: &mut TextEngine, role_base: usize, title: &str, desc: Option<&str>, options: &[&str], active: usize) {
            let p = self.p;
            let style = mk::typo::CALLOUT;
            let pad = mk::spacing::M;
            let widths: Vec<f32> = options.iter().map(|o| engine.measure_styled(o, style.px(), style.weight).0 + 2.0 * pad).collect();
            let track = widths.iter().sum::<f32>() + 4.0;
            let beside = track < self.w * 0.55;
            let (right, top, h) = self.row(engine, None, None, title, desc, if beside { track } else { 0.0 }, p.on_surface);
            // Round 89 (owner: «the element sticks out — just like under
            // display size»): below the title the options WRAP into as many
            // capsules as the card's width needs; before, one capsule ran
            // past the card's edge.
            let avail = (self.w - 2.0 * mk::spacing::M).max(60.0);
            let mut lines: Vec<std::ops::Range<usize>> = Vec::new();
            if beside {
                lines.push(0..options.len());
            } else {
                let (mut start, mut used) = (0usize, 4.0f32);
                for (i, w) in widths.iter().enumerate() {
                    if i > start && used + w > avail {
                        lines.push(start..i);
                        start = i;
                        used = 4.0;
                    }
                    used += w;
                }
                lines.push(start..options.len());
            }
            let (x0, mut y) = if beside {
                (right - track, top + (h - 34.0) / 2.0)
            } else {
                // Below: the row grows by a line of pills per capsule.
                let y = self.y - mk::spacing::S;
                self.y += lines.len() as f32 * (34.0 + mk::spacing::S);
                (self.x + mk::spacing::M, y)
            };
            for line in lines {
                let tw = widths[line.clone()].iter().sum::<f32>() + 4.0;
                self.b.quad(x0, y, tw, 34.0, p.surface_container, 17.0, None, self.clip);
                let mut x = x0 + 2.0;
                for i in line {
                    let (o, w) = (options[i], widths[i]);
                    let on = i == active;
                    let role = Some(role_base + i);
                    self.b.quad(x, y + 2.0, w, 30.0, if on { p.primary_container } else { mk::Rgba { a: 0.0, ..p.surface } }, 15.0, role, self.clip);
                    let (ow, oh) = engine.measure_styled(o, style.px(), style.weight);
                    self.b.styled(engine, o.to_string(), style, if on { p.on_primary_container } else { p.on_surface_variant }, x + (w - ow) / 2.0, y + (34.0 - oh) / 2.0, role, self.clip);
                    x += w;
                }
                y += 34.0 + mk::spacing::S;
            }
        }

        /// A level bar on the right (0..1, read only): a microphone's
        /// level, a battery, a progress. Green-to-accent fill.
        pub fn meter(&mut self, engine: &mut TextEngine, glyph: Option<char>, title: &str, desc: Option<&str>, value: f32) {
            let p = self.p;
            let mw = (self.w * 0.38).clamp(120.0, 260.0);
            let (right, top, h) = self.row(engine, None, glyph, title, desc, mw, p.on_surface);
            let (x, cy) = (right - mw, top + h / 2.0);
            self.b.quad(x, cy - 4.0, mw, 8.0, p.on_surface.over(p.surface_container_high, 0.12), 4.0, None, self.clip);
            let v = value.clamp(0.0, 1.0);
            if v > 0.0 {
                self.b.quad(x, cy - 4.0, (mw * v).max(8.0), 8.0, p.primary, 4.0, None, self.clip);
            }
        }

        /// A switch row with a round key before the switch (▶ preview of
        /// a sound, ⓘ details). The row switches; the key has `key_role`.
        #[allow(clippy::too_many_arguments)]
        pub fn toggle_with_key(&mut self, engine: &mut TextEngine, role: usize, key: (char, usize), title: &str, desc: Option<&str>, on: bool, locked: bool) {
            let tw = mk::control::TOGGLE_TRACK_W;
            let k = 36.0;
            let (right, top, h) = self.row(engine, (!locked).then_some(role), None, title, desc, tw + k + mk::spacing::M, self.p.on_surface);
            lay_round_icon(self.b, engine, key.0, right - tw - mk::spacing::M - k, top + (h - k) / 2.0, k, false, Some(key.1), self.clip, self.p);
            lay_switch(self.b, right - tw, top + (h - mk::control::TOGGLE_TRACK_H) / 2.0, on, locked, (!locked).then_some(role), self.clip, self.p);
        }

        /// A choice row with a check on the right when chosen (a device
        /// among several: the whole row picks it).
        pub fn pick(&mut self, engine: &mut TextEngine, role: usize, glyph: Option<char>, title: &str, desc: Option<&str>, chosen: bool) {
            let family = crate::symbol_family();
            let px = mk::typo::symbol_px(22.0);
            let check = crate::symbol::CHECK.to_string();
            let (cw, chh) = engine.measure_family(family, &check, px, mk::typo::Weight::Normal);
            let ink = if chosen { self.p.primary } else { self.p.on_surface };
            let (right, top, h) = self.row(engine, Some(role), glyph, title, desc, cw, ink);
            if chosen {
                self.b.text(engine, check, px, mk::typo::Weight::Normal, family, self.p.primary, right - cw, top + (h - chh) / 2.0, Some(role), self.clip);
            }
        }

        /// A DROPDOWN (06.11.2026, the owner: «rather two dropdown groups»):
        /// the row shows the chosen option and ▾; tapped (`role`) it opens IN
        /// PLACE — the options as rows below it, the chosen one with a check
        /// (roles `role_base + i`). No floating layer: on a phone panel the
        /// list scrolls with the page and every option is a full finger row.
        /// The app keeps `open` and closes it after a pick.
        #[allow(clippy::too_many_arguments)]
        pub fn dropdown(&mut self, engine: &mut TextEngine, role: usize, role_base: usize, title: &str, desc: Option<&str>, options: &[&str], active: usize, open: bool) {
            let cap = mk::typo::CALLOUT;
            let family = crate::symbol_family();
            let px = mk::typo::symbol_px(20.0);
            let chev = (if open { crate::symbol::EXPAND_LESS } else { crate::symbol::EXPAND_MORE }).to_string();
            let (cw, chh) = engine.measure_family(family, &chev, px, mk::typo::Weight::Normal);
            let value = mk::elide(options.get(active).copied().unwrap_or(""), 24);
            let (vw, vh) = engine.measure_styled(&value, cap.px(), cap.weight);
            let (right, top, h) = self.row(engine, Some(role), None, title, desc, cw + vw + mk::spacing::S, self.p.on_surface);
            self.b.text(engine, chev, px, mk::typo::Weight::Normal, family, self.p.on_surface_variant, right - cw, top + (h - chh) / 2.0, Some(role), self.clip);
            let ink = if open { self.p.primary } else { self.p.on_surface_variant };
            self.b.styled(engine, value, cap, ink, right - cw - mk::spacing::S - vw, top + (h - vh) / 2.0, Some(role), self.clip);
            if open {
                for (i, o) in options.iter().enumerate() {
                    self.pick(engine, role_base + i, Some(' '), o, None, i == active);
                }
            }
        }

        /// − value + on the right. Roles `role` (minus) and `role + 1` (plus).
        pub fn stepper(&mut self, engine: &mut TextEngine, role: usize, title: &str, desc: Option<&str>, value: &str, can_less: bool, can_more: bool) {
            let p = self.p;
            let cap = mk::typo::CALLOUT;
            let (vw, vh) = engine.measure_styled(value, cap.px(), cap.weight);
            let total = 34.0 * 2.0 + vw.max(44.0) + 2.0 * mk::spacing::S;
            let (right, top, h) = self.row(engine, None, None, title, desc, total, p.on_surface);
            let y = top + (h - 34.0) / 2.0;
            let x0 = right - total;
            lay_round_icon(self.b, engine, '\u{e15b}', x0, y, 34.0, false, can_less.then_some(role), self.clip, p);
            let vx = x0 + 34.0 + mk::spacing::S + (vw.max(44.0) - vw) / 2.0;
            self.b.styled(engine, value.to_string(), cap, p.on_surface, vx, top + (h - vh) / 2.0, None, self.clip);
            lay_round_icon(self.b, engine, crate::symbol::ADD, right - 34.0, y, 34.0, false, can_more.then_some(role + 1), self.clip, p);
        }

        /// A slider on the right (value 0..1); its track is remembered in
        /// `sliders` for presses and drags.
        pub fn slider(&mut self, engine: &mut TextEngine, role: usize, title: &str, desc: Option<&str>, value: f32) {
            let p = self.p;
            let sw = (self.w * 0.38).clamp(120.0, 260.0);
            let (right, top, h) = self.row(engine, None, None, title, desc, sw, p.on_surface);
            let (x, cy) = (right - sw, top + h / 2.0);
            let v = value.clamp(0.0, 1.0);
            // A finger-sized target over the thin rail.
            self.b.quad(x - 6.0, cy - 14.0, sw + 12.0, 28.0, mk::Rgba { a: 0.0, ..p.surface }, 0.0, Some(role), self.clip);
            self.b.quad(x, cy - 2.0, sw, 4.0, p.on_surface.over(p.surface_container_high, 0.15), 2.0, Some(role), self.clip);
            self.b.quad(x, cy - 2.0, sw * v, 4.0, p.primary, 2.0, Some(role), self.clip);
            self.b.quad(x + sw * v - 9.0, cy - 9.0, 18.0, 18.0, p.primary, 9.0, Some(role), self.clip);
            self.sliders.push((role, Rect { x, y: cy - 14.0, w: sw, h: 28.0 }));
        }

        /// Close the page: the height used (from where it started).
        pub fn finish(mut self) -> (f32, Vec<(usize, Rect)>) {
            self.end();
            (self.y, self.sliders)
        }
    }

    /// Where on a slider's track a press at `x` lies (0..1).
    pub fn slider_value(track: Rect, x: f32) -> f32 {
        if track.w <= 0.0 {
            return 0.0;
        }
        ((x - track.x) / track.w).clamp(0.0, 1.0)
    }

    /// PLATE (round 150, from the notices family): a rounded, filled box
    /// as a computed PICTURE — for anything that must lie ON a picture of
    /// its layer (a glass card, the wallpaper), where a quad would lie under
    /// it. See `SceneBuilder::rise` for the other way.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_plate(b: &mut matrixkit_shell::SceneBuilder, r: Rect, radius: f32, color: mk::Rgba, home: Rect, anchor: usize, clip: Option<Rect>) {
        use matrixkit_shell::shapes::{Shape, Style};
        let (w, h) = (r.w.max(1.0).round(), r.h.max(1.0).round());
        let shape = Shape::new(w, h).op(crate::glass_clock::rounded(0.0, 0.0, w, h, radius), Style::Fill(color));
        let mut pict = pict_of(&shape, r.x, r.y, home, anchor);
        pict.clip = clip;
        b.pict(pict);
    }

    /// GLASS SLIDER (round 148, from the lock's clock sheet): a light track
    /// on glass, filled up to `value`, a white capsule knob — on any dark
    /// or glass ground. `clip` for pages that slide (stack family).
    ///
    /// Drawn as ONE shape picture (round 149): plain quads lie under every
    /// picture of their layer — on a glass card (itself a picture) a quad
    /// slider was simply hidden (05.11.2026).
    #[allow(clippy::too_many_arguments)]
    pub fn lay_glass_slider(b: &mut matrixkit_shell::SceneBuilder, track: Rect, value: f32, fade: f32, clip: Option<Rect>, home: Rect, anchor: usize) {
        use matrixkit_shell::shapes::{Shape, Style};
        let white = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let v = value.clamp(0.0, 1.0);
        let (kw, kh) = (38.0, 24.0);
        let (w, h) = ((track.w + kw).round(), kh);
        let (ox, oy) = (track.x - kw / 2.0, track.y + track.h / 2.0 - kh / 2.0);
        let (tx, ty) = (kw / 2.0, (kh - track.h) / 2.0);
        let r = track.h / 2.0;
        let fill_w = (track.w * v).max(track.h);
        let kx = track.w * v;
        let shape = Shape::new(w, h)
            .op(crate::glass_clock::rounded(tx, ty, tx + track.w, ty + track.h, r), Style::Fill(white.with_alpha(0.22 * fade)))
            .op(crate::glass_clock::rounded(tx, ty, tx + fill_w, ty + track.h, r), Style::Fill(white.with_alpha(0.85 * fade)))
            .op(crate::glass_clock::rounded(kx, 0.0, kx + kw, kh, kh / 2.0), Style::Fill(white.with_alpha(fade)));
        let mut pict = pict_of(&shape, ox, oy, home, anchor);
        pict.clip = clip;
        b.pict(pict);
    }

    /// A TAG (R211 family, 02.11.2026): a word in a thin coloured frame on
    /// a dark inside — readable on anything, quieter than a filled chip (the
    /// words a Krabbler carries, a wanted word). Centred on (cx, cy);
    /// returns its rectangle.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_tag(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, text: &str, style: mk::typo::TextStyle, cx: f32, cy: f32, color: mk::Rgba, inside: mk::Rgba, clip: Option<Rect>) -> Rect {
        let (tw, th) = engine.measure_styled(text, style.px(), style.weight);
        let r = Rect { x: cx - tw / 2.0 - 4.0, y: cy - th / 2.0 - 1.0, w: tw + 8.0, h: th + 2.0 };
        b.quad(r.x, r.y, r.w, r.h, mk::Rgba { a: color.a * 0.78, ..color }, 3.0, None, clip);
        b.quad(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0, inside, 2.5, None, clip);
        b.styled(engine, text.to_string(), style, color, r.x + 4.0, r.y + 1.0, None, clip);
        r
    }

    /// WORD FLOW (R211 family, 02.11.2026): paragraphs broken into lines of
    /// single words, each with its own box — the base for anything that
    /// touches words one by one (Krabbler's creatures; highlights, cards).
    #[derive(Debug, Clone, PartialEq)]
    pub struct WordBox {
        /// As written (with punctuation).
        pub text: String,
        /// Its letters only (`letters`).
        pub clean: String,
        /// Index of its paragraph; headings are paragraphs too.
        pub paragraph: usize,
        pub heading: bool,
        pub x: f32,
        pub y: f32,
        pub w: f32,
        pub h: f32,
    }

    /// A word's letters: punctuation off both ends («Haus», → Haus).
    pub fn letters(word: &str) -> String {
        word.trim_matches(|c: char| !c.is_alphanumeric()).to_string()
    }

    /// Paragraphs of a plain or light-markdown text: blank lines separate,
    /// «# …» is a heading, emphasis marks and list dashes go. (heading, text)
    pub fn paragraphs(text: &str) -> Vec<(bool, String)> {
        text.split("\n\n")
            .map(str::trim)
            .filter(|b| !b.is_empty())
            .map(|block| {
                let (heading, body) = match block.strip_prefix('#') {
                    Some(rest) => (true, rest.trim_start_matches('#').trim()),
                    None => (false, block),
                };
                let body = body.replace(['*', '_', '`'], "").lines().map(|l| l.trim_start_matches("- ").trim()).collect::<Vec<_>>().join(" ");
                (heading, body)
            })
            .collect()
    }

    /// Lay `paras` into `width` from (margin, margin): headings in
    /// `heading`, the rest in `body`, lines `line_gap`× their height.
    /// Returns the words and the height used.
    pub fn flow_words(engine: &mut TextEngine, paras: &[(bool, String)], width: f32, margin: f32, heading: mk::typo::TextStyle, body: mk::typo::TextStyle, line_gap: f32) -> (Vec<WordBox>, f32) {
        let mut words = Vec::new();
        let mut y = margin;
        for (pi, (is_heading, text)) in paras.iter().enumerate() {
            let style = if *is_heading { heading } else { body };
            let (space, line) = engine.measure_styled(" ", style.px(), style.weight);
            let line_h = line * if *is_heading { 1.3 } else { line_gap };
            let mut x = margin;
            for word in text.split_whitespace() {
                let (w, h) = engine.measure_styled(word, style.px(), style.weight);
                if x > margin && x + w > width - margin {
                    x = margin;
                    y += line_h;
                }
                words.push(WordBox { text: word.to_string(), clean: letters(word), paragraph: pi, heading: *is_heading, x, y, w, h });
                x += w + space;
            }
            y += line_h + if *is_heading { mk::spacing::S } else { mk::spacing::M };
        }
        (words, y)
    }

    /// A ROUND ICON KEY (R210 family, 02.11.2026): a pill-round key with
    /// one symbol — «+» on a download, the open/close of a row's choices.
    /// `on` lifts it into the accent container. Returns the placed quad.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_round_icon(
        b: &mut matrixkit_shell::SceneBuilder,
        engine: &mut TextEngine,
        glyph: char,
        x: f32,
        y: f32,
        size: f32,
        on: bool,
        role: Option<usize>,
        clip: Option<Rect>,
        p: mk::Palette,
    ) -> usize {
        let (fill, ink) = if on { (p.primary_container, p.on_primary_container) } else { (p.surface_container_high, p.on_surface) };
        let q = b.quad(x, y, size, size, fill, size / 2.0, role, clip);
        let family = crate::symbol_family();
        let px = mk::typo::symbol_px((size * 0.55).max(14.0));
        let (gw, gh) = engine.measure_family(family, &glyph.to_string(), px, mk::typo::Weight::Normal);
        b.text(engine, glyph.to_string(), px, mk::typo::Weight::Normal, family, ink, x + (size - gw) / 2.0, y + (size - gh) / 2.0, role, clip);
        q
    }

    /// A SECRET FIELD (round 44): a pill that holds a password — dots or
    /// clear text (`reveal`), an optional caret, a placeholder when empty, a
    /// red ring on `error`, and an eye key at its end. The login screen and
    /// the WLAN card drew the same field each on their own. `ink` is the
    /// text colour (white on the night of the login), `fill` the inside.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_secret_field(
        b: &mut matrixkit_shell::SceneBuilder,
        engine: &mut TextEngine,
        rect: Rect,
        text: &str,
        reveal: bool,
        caret: bool,
        error: bool,
        placeholder: &str,
        roles: (Option<usize>, Option<usize>),
        ink: mk::Rgba,
        fill: mk::Rgba,
        p: mk::Palette,
    ) {
        let (field, eye_role) = roles;
        let r = rect.h / 2.0;
        let ring = if error { p.error } else { ink.with_alpha(0.35) };
        b.quad(rect.x - 1.5, rect.y - 1.5, rect.w + 3.0, rect.h + 3.0, ring, r + 1.5, field, None);
        b.quad(rect.x, rect.y, rect.w, rect.h, fill, r, field, None);
        let st = mk::typo::BODY;
        let tx = rect.x + mk::spacing::L;
        let room = rect.w - 2.0 * mk::spacing::L - 44.0;
        let (shown, tint) = if text.is_empty() && !caret {
            (placeholder.to_string(), ink.with_alpha(0.45))
        } else {
            let body = if reveal { text.to_string() } else { "\u{25cf}".repeat(text.chars().count().min(24)) };
            (if caret { format!("{body}|") } else { body }, ink)
        };
        let shown = ellipsize(engine, &shown, st.px(), st.weight, room);
        let (_, th) = engine.measure_styled(&shown, st.px(), st.weight);
        b.styled(engine, shown, st, tint, tx, rect.y + (rect.h - th) / 2.0, field, None);
        let eye = if reveal { crate::symbol::VISIBILITY_OFF } else { crate::symbol::VISIBILITY };
        let gp = mk::typo::symbol_px(22.0);
        let family = crate::symbol_family();
        let (gw, gh) = engine.measure_family(family, &eye.to_string(), gp, mk::typo::Weight::Normal);
        b.quad(rect.x + rect.w - 48.0, rect.y + 4.0, 44.0, rect.h - 8.0, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, 20.0, eye_role, None);
        b.text(engine, eye.to_string(), gp, mk::typo::Weight::Normal, family, ink.with_alpha(0.75), rect.x + rect.w - 26.0 - gw / 2.0, rect.y + (rect.h - gh) / 2.0, eye_role, None);
    }

    /// TEXT FIELD (round 81 family, Matrix Einkauf): a capsule field for
    /// plain text — a leading symbol (search, edit), the text or a quiet
    /// placeholder, a caret while focused, and a round ✕ that empties it
    /// once there is text. `roles` = (the field, the ✕). The secret field's
    /// sibling, without dots and eye.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_text_field(
        b: &mut matrixkit_shell::SceneBuilder,
        engine: &mut TextEngine,
        rect: Rect,
        text: &str,
        focused: bool,
        caret: bool,
        placeholder: &str,
        lead: Option<char>,
        roles: (Option<usize>, Option<usize>),
        clip: Option<Rect>,
        p: mk::Palette,
    ) {
        let (field, clear_role) = roles;
        let r = rect.h / 2.0;
        let ring = if focused { p.primary } else { p.outline.with_alpha(0.45) };
        b.quad(rect.x - 1.5, rect.y - 1.5, rect.w + 3.0, rect.h + 3.0, ring, r + 1.5, field, clip);
        b.quad(rect.x, rect.y, rect.w, rect.h, p.surface_container_high, r, field, clip);
        let family = crate::symbol_family();
        let gp = mk::typo::symbol_px(20.0);
        let mut tx = rect.x + mk::spacing::L;
        if let Some(g) = lead {
            let (gw, gh) = engine.measure_family(family, &g.to_string(), gp, mk::typo::Weight::Normal);
            b.text(engine, g.to_string(), gp, mk::typo::Weight::Normal, family, p.on_surface_variant, tx, rect.y + (rect.h - gh) / 2.0, field, clip);
            tx += gw + mk::spacing::S;
        }
        let clearable = !text.is_empty() && clear_role.is_some();
        let room = rect.x + rect.w - tx - mk::spacing::L - if clearable { 40.0 } else { 0.0 };
        let st = mk::typo::BODY;
        let (shown, tint) = if text.is_empty() && !focused {
            (placeholder.to_string(), p.on_surface_variant.with_alpha(0.7))
        } else if focused && caret {
            (format!("{text}|"), p.on_surface)
        } else {
            (text.to_string(), p.on_surface)
        };
        // The end of a long text stays in sight (that is where one types).
        let mut shown = shown;
        while shown.chars().count() > 1 && engine.measure_styled(&shown, st.px(), st.weight).0 > room.max(10.0) {
            shown = shown.chars().skip(1).collect();
        }
        let (_, th) = engine.measure_styled(&shown, st.px(), st.weight);
        b.styled(engine, shown, st, tint, tx, rect.y + (rect.h - th) / 2.0, field, clip);
        if clearable {
            let x = '\u{e5cd}'.to_string();
            let (gw, gh) = engine.measure_family(family, &x, gp, mk::typo::Weight::Normal);
            let cx = rect.x + rect.w - 24.0;
            b.quad(cx - 18.0, rect.y + (rect.h - 36.0) / 2.0, 36.0, 36.0, p.on_surface.with_alpha(0.08), 18.0, clear_role, clip);
            b.text(engine, x, gp, mk::typo::Weight::Normal, family, p.on_surface_variant, cx - gw / 2.0, rect.y + (rect.h - gh) / 2.0, clear_role, clip);
        }
    }

    /// What a key did to a text being typed (round 81 family).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TextKey {
        Changed,
        Enter,
        Escape,
        Ignored,
    }

    /// Edit `text` with one key: letters append (up to `max` characters),
    /// Backspace takes the last one, Enter and Escape are reported.
    /// `accept` filters what may be typed (digits for a number field).
    pub fn text_key(text: &mut String, key: &matrixkit_shell::Key, max: usize, accept: impl Fn(char) -> bool) -> TextKey {
        match key {
            matrixkit_shell::Key::Char(s) => {
                let mut changed = false;
                for c in s.chars().filter(|c| !c.is_control() && accept(*c)) {
                    if text.chars().count() < max {
                        text.push(c);
                        changed = true;
                    }
                }
                if changed { TextKey::Changed } else { TextKey::Ignored }
            }
            matrixkit_shell::Key::Backspace => {
                if text.pop().is_some() { TextKey::Changed } else { TextKey::Ignored }
            }
            matrixkit_shell::Key::Enter => TextKey::Enter,
            matrixkit_shell::Key::Escape => TextKey::Escape,
            _ => TextKey::Ignored,
        }
    }

    #[cfg(test)]
    mod text_key_tests {
        use super::*;
        use matrixkit_shell::Key;

        #[test]
        fn typing_respects_max_and_filter() {
            let mut t = String::from("Me");
            assert_eq!(text_key(&mut t, &Key::Char("hl".into()), 3, |_| true), TextKey::Changed);
            assert_eq!(t, "Meh");
            assert_eq!(text_key(&mut t, &Key::Char("x".into()), 3, |_| true), TextKey::Ignored);
            assert_eq!(text_key(&mut t, &Key::Backspace, 3, |_| true), TextKey::Changed);
            let mut n = String::new();
            text_key(&mut n, &Key::Char("2a,5".into()), 8, |c| c.is_ascii_digit() || c == ',');
            assert_eq!(n, "2,5");
            assert_eq!(text_key(&mut n, &Key::Enter, 8, |_| true), TextKey::Enter);
        }
    }

    /// THE CARD HEAD (round 36): a title, a line about it and a round ✕
    /// that closes the card (`close_role`) — the head of every card of the
    /// top bar (app, user, control centre, app list; before: four copies).
    /// Returns the y below it.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_card_head(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, title: &str, about: &str, close_role: usize, x: f32, y: f32, w: f32, clip: Option<Rect>, p: mk::Palette) -> f32 {
        let key = 40.0;
        let head = mk::typo::HEADLINE;
        let (hx, mut hy) = (x + mk::spacing::M, y + mk::spacing::XS);
        let title = ellipsize(engine, &mk::elide(title, 30), head.px(), head.weight, w - 2.0 * mk::spacing::M - key - mk::spacing::S);
        let (_, th) = b.styled(engine, title, head, p.on_surface, hx, hy, None, clip);
        lay_round_icon(b, engine, crate::symbol::CLOSE, x + w - key, y, key, false, Some(close_role), clip, p);
        hy += th + 2.0;
        let about = ellipsize(engine, about, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight, w - 2.0 * mk::spacing::M - key - mk::spacing::S);
        let (_, ah) = b.styled(engine, about, mk::typo::CAPTION, p.on_surface_variant, hx, hy, None, clip);
        hy + ah + mk::spacing::M
    }

    /// THE MATRIX KEYBOARD (round 33, 02.11.2026 — «without iced»): the
    /// on-screen keyboard as one family — the layout (QWERTZ-CH with
    /// umlauts, symbols on two levels) and its drawing, exactly the look of
    /// the iced keyboard (dark pill, keys a shade lighter, enter in primary,
    /// ⇧ ⌫ ⏎ ✕). The keyboard daemon and the login screen both draw it.
    pub mod keyboard {
        use super::{Rect, TextEngine};
        use matrixkit_theme as mk;

        /// The key unit, key height, gap and the pill's inset (iced R50).
        pub const KEY: f32 = 56.0;
        pub const KEY_H: f32 = 48.0;
        pub const GAP: f32 = 6.0;
        pub const MARGIN: f32 = 12.0;
        /// A floating pill (desktop): 11 units wide, 4 rows high.
        pub const PILL_W: f32 = 11.0 * KEY + 10.0 * GAP + 2.0 * MARGIN;
        pub const PILL_H: f32 = 4.0 * KEY_H + 3.0 * GAP + 2.0 * MARGIN;

        /// Round 167: where the keyboard lies on a finger device — the
        /// desktop's pill (Matrix Tastatur): full width less one window gap
        /// each side, `PILL_H` high, one gap above the floor. The login
        /// screen lays it so too («the same size as on the desktop»).
        pub fn finger_rect(w: f32, h: f32) -> Rect {
            let gap = mk::device::WINDOW_GAP;
            let kh = PILL_H.min((h - 2.0 * gap).max(0.0));
            Rect { x: gap, y: h - gap - kh, w: (w - 2.0 * gap).max(0.0), h: kh }
        }
        /// The pill's fill coverage over glass and its hairline (mkw::panel).
        const COVERAGE: f32 = 0.94;
        const OUTLINE: f32 = 0.25;

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum Act {
            Text(char),
            Shift,
            Delete,
            Enter,
            Space,
            Page,
            Close,
        }

        #[derive(Debug, Clone, PartialEq)]
        pub struct KeyDef {
            pub display: String,
            pub units: f32,
            pub act: Act,
            pub accent: bool,
        }

        fn char_key(c: char) -> KeyDef {
            KeyDef { display: c.to_string(), units: 1.0, act: Act::Text(c), accent: false }
        }

        fn large(c: char) -> char {
            c.to_uppercase().next().unwrap_or(c)
        }

        /// The rows of a page (0 letters, 1 symbols) — `closable` adds the
        /// ✕ at the end of the bottom row (the login screen has none).
        pub fn rows(page: usize, shift: bool, closable: bool) -> Vec<Vec<KeyDef>> {
            let line = |s: &str| -> Vec<KeyDef> { s.chars().map(|c| char_key(if shift { large(c) } else { c })).collect() };
            let delete = || KeyDef { display: "\u{232b}".into(), units: 1.5, act: Act::Delete, accent: false };
            let bottom = |switch: &str| -> Vec<KeyDef> {
                let mut row = vec![
                    KeyDef { display: switch.into(), units: 1.5, act: Act::Page, accent: false },
                    char_key(','),
                    KeyDef { display: String::new(), units: 5.0, act: Act::Space, accent: false },
                    char_key('.'),
                    KeyDef { display: "\u{23ce}".into(), units: 2.0, act: Act::Enter, accent: true },
                ];
                if closable {
                    row.push(KeyDef { display: "\u{2715}".into(), units: 1.0, act: Act::Close, accent: false });
                }
                row
            };
            // Morpheus (06.11.2026): the layout of the device — Swiss QWERTZ
            // with umlauts, or UK QWERTY (en-GB) with the apostrophe.
            let uk = mk::language::layout() == "gb";
            let (l1, l2, l3) = if uk { ("qwertyuiop", "asdfghjkl'", "zxcvbnm") } else { ("qwertzuiopü", "asdfghjklöä", "yxcvbnm") };
            match page {
                0 => vec![
                    line(l1),
                    line(l2),
                    {
                        let mut r = vec![KeyDef { display: "\u{21e7}".into(), units: 1.5, act: Act::Shift, accent: shift }];
                        r.extend(line(l3));
                        r.push(delete());
                        r
                    },
                    bottom("?123"),
                ],
                // Symbols on two levels like a phone (#+= switches): the
                // terminal needs ~ | \ $ ' ! ? < > [ ] { } ^ ` (01.11.2026).
                _ => {
                    let (r1, r2, r3) = if shift {
                        ("[]{}#%^*+=", "_\\|~<>\u{20ac}\u{00a3}`\u{00b0}", "\u{00e9}\u{00e8}\u{00e0}\u{00e7}\u{00a7}")
                    } else {
                        if uk { ("1234567890", "-/:;()\u{00a3}&@\"", "'!?=+*%") } else { ("1234567890", "-/:;()$&@\"", "'!?=+*%") }
                    };
                    let chars = |s: &str| -> Vec<KeyDef> { s.chars().map(char_key).collect() };
                    vec![
                        chars(r1),
                        chars(r2),
                        {
                            let mut r = vec![KeyDef { display: if shift { "123".into() } else { "#+=".into() }, units: 1.5, act: Act::Shift, accent: shift }];
                            r.extend(chars(r3));
                            r.push(delete());
                            r
                        },
                        bottom("ABC"),
                    ]
                }
            }
        }

        /// Every character the keyboard can type (for the injection keymap).
        pub fn all_chars() -> Vec<char> {
            let mut all = vec![' '];
            for page in 0..2 {
                for shift in [false, true] {
                    for row in rows(page, shift, true) {
                        all.extend(row.iter().filter_map(|k| if let Act::Text(c) = k.act { Some(c) } else { None }));
                    }
                }
            }
            all
        }

        /// What one drawing needs to know.
        pub struct Look<'a> {
            pub page: usize,
            pub shift: bool,
            pub closable: bool,
            /// Keys that glow right now (the 150 ms afterglow, by display).
            pub glowing: &'a [String],
            /// The key under a held finger (row, column), lit.
            pub held: Option<(usize, usize)>,
            /// The controller's focus ring (row, column).
            pub focus: Option<(usize, usize)>,
            /// The enter key's label («Anmelden»); empty = ⏎.
            pub enter: &'a str,
        }

        /// Lay the pill into `rect` (it fills it: panel-wide on a finger
        /// device) and its keys; every key answers to `role_base + i`.
        /// Returns (act, row, column) per role. Taller rects get taller keys.
        #[allow(clippy::too_many_arguments)]
        pub fn lay(b: &mut matrixkit_shell::SceneBuilder, engine: &mut TextEngine, rect: Rect, look: &Look, role_base: usize, p: mk::Palette) -> Vec<(Act, usize, usize)> {
            let radius = mk::radius::SMALL + MARGIN;
            let cover = if mk::transparency_reduced() { 1.0 } else { COVERAGE };
            b.quad(rect.x, rect.y, rect.w, rect.h, p.outline.with_alpha(OUTLINE), radius, None, None);
            b.quad(rect.x + 1.0, rect.y + 1.0, rect.w - 2.0, rect.h - 2.0, p.surface_container.with_alpha(cover), radius - 1.0, None, None);
            let grid = rows(look.page, look.shift, look.closable);
            let inner = rect.w - 2.0 * MARGIN;
            // The widest row decides (the bottom row has more than 11 units).
            let units = grid.iter().map(|r| r.iter().map(|k| k.units).sum::<f32>()).fold(11.0_f32, f32::max);
            let unit = ((inner - (units - 1.0) * GAP) / units).clamp(20.0, KEY * 1.6);
            let n = grid.len() as f32;
            let key_h = ((rect.h - 2.0 * MARGIN - (n - 1.0) * GAP) / n).clamp(28.0, KEY_H * 2.0);
            let top = rect.y + (rect.h - (n * key_h + (n - 1.0) * GAP)) / 2.0;
            let body = mk::typo::BODY;
            let mut out = Vec::new();
            for (ri, row) in grid.iter().enumerate() {
                let row_w: f32 = row.iter().map(|k| k.units * unit + (k.units - 1.0) * GAP).sum::<f32>() + GAP * (row.len() as f32 - 1.0);
                let mut x = rect.x + (rect.w - row_w) / 2.0;
                let y = top + ri as f32 * (key_h + GAP);
                for (ci, k) in row.iter().enumerate() {
                    let w = k.units * unit + (k.units - 1.0) * GAP;
                    let glow = look.glowing.iter().any(|g| *g == k.display);
                    let lit = glow || k.accent;
                    let mut ground = if lit { p.primary } else { p.on_surface.over(p.surface_container, 0.08) };
                    if look.held == Some((ri, ci)) {
                        ground = p.on_surface.over(ground, mk::state_layer::PRESSED);
                    }
                    let role = role_base + out.len();
                    if look.focus == Some((ri, ci)) {
                        b.quad(x - 2.0, y - 2.0, w + 4.0, key_h + 4.0, p.primary, mk::radius::SMALL + 2.0, Some(role), None);
                    }
                    b.quad(x, y, w, key_h, ground, mk::radius::SMALL, Some(role), None);
                    let label = if k.act == Act::Enter && !look.enter.is_empty() { look.enter.to_string() } else { k.display.clone() };
                    if !label.is_empty() {
                        let ink = if lit { p.on_primary } else { p.on_surface };
                        let (tw, th) = engine.measure_styled(&label, body.px(), body.weight);
                        b.styled(engine, label, body, ink, x + (w - tw) / 2.0, y + (key_h - th) / 2.0, Some(role), None);
                    }
                    out.push((k.act, ri, ci));
                    x += w + GAP;
                }
            }
            out
        }

        #[cfg(test)]
        mod tests {
            use super::{all_chars, rows, Act};

            #[test]
            fn the_matrix_layout() {
                let letters = rows(0, false, true);
                assert_eq!(letters.len(), 4);
                assert_eq!(letters[0][0].act, Act::Text('q'));
                assert!(letters[1].iter().any(|k| k.act == Act::Text('ä')));
                assert_eq!(letters[3].last().map(|k| k.act), Some(Act::Close));
                assert_eq!(rows(0, false, false)[3].last().map(|k| k.act), Some(Act::Enter));
                assert_eq!(rows(0, true, true)[0][0].act, Act::Text('Q'));
                let all = all_chars();
                for c in ['~', '|', '\\', '@', 'é', '€', ' '] {
                    assert!(all.contains(&c), "{c}");
                }
            }
        }
    }

    /// THE OSD GLYPH (round 28): the symbol of a level change and whether
    /// it is «full» (sound/mic on, light) — one table for every house that
    /// shows a level (the dock's chip, the OSD; before: a copy each).
    pub fn osd_glyph(state: mk::osd::State) -> (char, bool) {
        match (state.kind, state.muted) {
            (mk::osd::Kind::Sound, false) => (crate::symbol::VOLUME_UP, true),
            (mk::osd::Kind::Sound, true) => (crate::symbol::VOLUME_OFF, false),
            (mk::osd::Kind::Mic, false) => (crate::symbol::MIC, true),
            (mk::osd::Kind::Mic, true) => (crate::symbol::MIC_OFF, false),
            (mk::osd::Kind::Light, _) => (crate::symbol::BRIGHTNESS, true),
        }
    }

    /// A SHUTTER (R215 family, 02.11.2026): the camera's release — a light
    /// ring around a disk; `busy` dims the disk (a still is being taken),
    /// `flash` (0..1) lights it white for the moment of release.
    /// Returns the placed outer quad (its role takes the tap).
    #[allow(clippy::too_many_arguments)]
    pub fn lay_shutter(b: &mut matrixkit_shell::SceneBuilder, cx: f32, cy: f32, size: f32, busy: bool, flash: f32, role: Option<usize>, clip: Option<Rect>, p: mk::Palette) -> usize {
        let ring = mk::Rgba { a: 0.95, ..p.on_surface };
        let outer = b.quad(cx - size / 2.0, cy - size / 2.0, size, size, ring, size / 2.0, role, clip);
        let gap = (size * 0.08).max(3.0);
        let inner = size - 2.0 * gap;
        b.quad(cx - inner / 2.0, cy - inner / 2.0, inner, inner, p.surface, inner / 2.0, role, clip);
        let disk = inner - 2.0 * gap;
        let base = if busy { mk::Rgba { a: 0.45, ..p.on_surface } } else { p.on_surface };
        let lit = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let t = flash.clamp(0.0, 1.0);
        let fill = mk::Rgba { r: base.r + (lit.r - base.r) * t, g: base.g + (lit.g - base.g) * t, b: base.b + (lit.b - base.b) * t, a: base.a + (lit.a - base.a) * t };
        b.quad(cx - disk / 2.0, cy - disk / 2.0, disk, disk, fill, disk / 2.0, role, clip);
        outer
    }

    /// A MEDIA ROW (R210 family, 02.11.2026): thumbnail, up to two title
    /// lines, a meta line — Tube built it four times by hand (search hits,
    /// «Als Nächstes», the playlist, the library). `now` marks the playing
    /// entry (accent ground, ▶/⏸ over the picture), `picked` a quiet ground,
    /// `badge` a length on the picture, `trailing` a round key at the end.
    pub struct MediaRow<'a> {
        pub thumb: Option<&'a std::path::Path>,
        pub thumb_w: f32,
        pub thumb_h: f32,
        pub title: &'a str,
        pub meta: &'a str,
        pub style: mk::typo::TextStyle,
        pub badge: Option<&'a str>,
        /// Some(playing): this entry plays.
        pub now: Option<bool>,
        pub picked: bool,
        /// (symbol, on, role) of a round key at the row's end.
        pub trailing: Option<(char, bool, usize)>,
    }

    impl<'a> MediaRow<'a> {
        /// A row with the common sizes (96 × 54, callout).
        pub fn new(title: &'a str, meta: &'a str) -> MediaRow<'a> {
            MediaRow { thumb: None, thumb_w: 96.0, thumb_h: 54.0, title, meta, style: mk::typo::CALLOUT, badge: None, now: None, picked: false, trailing: None }
        }

        /// The row's height with its gap below.
        pub fn height(&self) -> f32 {
            self.thumb_h + mk::spacing::S
        }
    }

    /// Lay a media row at (x, y), `w` wide; the whole row answers to `role`.
    /// Returns the height taken (with the gap below).
    #[allow(clippy::too_many_arguments)]
    pub fn lay_media_row(
        b: &mut matrixkit_shell::SceneBuilder,
        engine: &mut TextEngine,
        row: &MediaRow,
        x: f32,
        y: f32,
        w: f32,
        role: Option<usize>,
        clip: Option<Rect>,
        p: mk::Palette,
    ) -> f32 {
        let m = mk::spacing::M;
        let (tw, th) = (row.thumb_w, row.thumb_h);
        let radius = if tw >= 120.0 { 8.0 } else { 6.0 };
        let now = row.now.is_some();
        let ground = if now {
            p.primary_container.with_alpha(0.55)
        } else if row.picked {
            p.on_surface.over(p.surface, 0.08)
        } else {
            mk::Rgba { a: 0.0, ..p.surface }
        };
        b.quad(x - m / 2.0, y - 2.0, w + m, th + 4.0, ground, mk::radius::NORMAL, role, clip);
        let anchor = b.quad(x, y, tw, th, p.surface_container_high, radius, role, clip);
        if let Some(path) = row.thumb {
            b.pict(matrixkit_shell::Pict {
                path: path.to_path_buf(),
                box_w: tw as u32,
                box_h: th as u32,
                fit: matrixkit_image::Fit::Cover,
                x,
                y,
                radius,
                home: Rect::of(&b.scene.placed[anchor]),
                anchor,
                clip,
            });
        }
        let white = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let cap = mk::typo::CAPTION;
        if let Some(badge) = row.badge.filter(|s| !s.is_empty()) {
            let (lw, lh) = engine.measure_styled(badge, cap.px(), cap.weight);
            b.quad(x + tw - lw - 12.0, y + th - lh - 6.0, lw + 8.0, lh + 2.0, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.72 }, 4.0, role, clip);
            b.styled(engine, badge, cap, white, x + tw - lw - 8.0, y + th - lh - 5.0, role, clip);
        }
        if let Some(playing) = row.now {
            b.quad(x, y, tw, th, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.45 }, radius, role, clip);
            let family = crate::symbol_family();
            let px = mk::typo::symbol_px(26.0);
            let glyph = if playing { crate::symbol::PLAY_ARROW } else { crate::symbol::PAUSE };
            let (gw, gh) = engine.measure_family(family, &glyph.to_string(), px, mk::typo::Weight::Normal);
            b.text(engine, glyph.to_string(), px, mk::typo::Weight::Normal, family, white, x + (tw - gw) / 2.0, y + (th - gh) / 2.0, role, clip);
        }
        let key = if row.trailing.is_some() { 44.0 + m } else { 0.0 };
        let text_x = x + tw + m;
        let text_w = (x + w - key - text_x).max(40.0);
        let chars = ((text_w / (row.style.px() * 0.5)) as usize).max(12);
        let (ink, soft) = if now { (p.on_primary_container, p.on_primary_container) } else { (p.on_surface, p.on_surface_variant) };
        let mut ty = y;
        for line in mk::wrap_words(row.title, chars, 2) {
            let (_, lh) = b.styled(engine, line, row.style, ink, text_x, ty, role, clip);
            ty += lh;
        }
        if !row.meta.is_empty() {
            b.styled(engine, mk::elide(row.meta, chars + 6), cap, soft, text_x, ty + 1.0, role, clip);
        }
        if let Some((glyph, on, key_role)) = row.trailing {
            lay_round_icon(b, engine, glyph, x + w - 44.0, y + (th - 36.0) / 2.0 - 4.0, 44.0, on, Some(key_role), clip, p);
        }
        row.height()
    }

    /// Join the non-empty parts with « · » (meta lines).
    pub fn dot_join(parts: &[&str]) -> String {
        parts.iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" · ")
    }

    /// What a media bar shows this frame.
    pub struct MediaView<'a> {
        /// Seconds heard / shown.
        pub position: f32,
        pub duration: f32,
        pub playing: bool,
        /// Seconds fetched ahead (`Video::stocked_s`; infinite = all).
        pub ahead: Option<f64>,
        /// Marked parts (from, to) in seconds — SponsorBlock, chapters …
        pub marks: &'a [(f32, f32)],
        /// Round 172: a «previous» key at the bar's start (Some(usable)),
        /// a «next» key at its end — a playlist steers in the same row as
        /// the timeline (owner: they had a row of their own below it).
        pub prev: Option<bool>,
        pub next: Option<bool>,
    }

    /// What a media bar asks of the app.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum MediaDeed {
        /// Play or pause.
        Toggle,
        /// A drag on the timeline began (redraw).
        Scrub,
        /// Round 172: the previous / next entry (a playlist).
        Prev,
        Next,
    }

    /// MEDIA BAR (R210 family): play/pause, the timeline with its stock
    /// shadow running ahead (like YouTube), marks and the knob, the time
    /// text. Dragging the timeline previews the target; `release` hands
    /// the time to seek to. Tube and Player drew this by hand.
    ///
    /// Roles: `base` the play key, `base + 1` the timeline, `base + 2` the
    /// previous and `base + 3` the next key (round 172).
    pub struct MediaBar {
        base: usize,
        track: Rect,
        duration: f32,
        /// The target while a finger drags the timeline.
        pub dragging: Option<f32>,
    }

    impl MediaBar {
        pub const HEIGHT: f32 = 48.0;

        pub fn new(role_base: usize) -> Self {
            MediaBar { base: role_base, track: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }, duration: 0.0, dragging: None }
        }

        pub fn owns(&self, role: Option<usize>) -> bool {
            matches!(role, Some(r) if (self.base..=self.base + 3).contains(&r))
        }

        /// Below this width the time shows only the position (round 172).
        pub const NARROW: f32 = 420.0;

        /// Lay the bar at `y` across `w`; returns its height.
        pub fn build(&mut self, engine: &mut TextEngine, b: &mut matrixkit_shell::SceneBuilder, y: f32, w: f32, v: &MediaView, p: mk::Palette) -> f32 {
            let h = Self::HEIGHT;
            let m = mk::spacing::M;
            let clear = mk::Rgba { a: 0.0, ..p.surface };
            let (play, line) = (Some(self.base), Some(self.base + 1));
            self.duration = v.duration.max(0.0);
            b.quad(0.0, y, w, h, p.surface_container, 0.0, None, None);
            let fam = crate::symbol_family();
            let gpx = mk::typo::symbol_px(24.0);
            // A round key of 40 px at x (round 172: play, previous, next).
            let mut key = |b: &mut matrixkit_shell::SceneBuilder, glyph: char, x: f32, role: Option<usize>, on: bool| {
                let (gw, gh) = engine.measure_family(fam, &glyph.to_string(), gpx, mk::typo::Weight::Normal);
                let ink = if on { p.on_surface } else { p.on_surface.over(p.surface_container, 0.38) };
                b.quad(x, y + (h - 40.0) / 2.0, 40.0, 40.0, clear, 20.0, role.filter(|_| on), None);
                b.text(engine, glyph.to_string(), gpx, mk::typo::Weight::Normal, fam, ink, x + (40.0 - gw) / 2.0, y + (h - gh) / 2.0, role.filter(|_| on), None);
            };
            let mut left = m;
            if let Some(on) = v.prev {
                key(b, crate::symbol::SKIP_PREVIOUS, left, Some(self.base + 2), on);
                left += 44.0;
            }
            // Play / pause.
            let glyph = if v.playing { crate::symbol::PAUSE } else { crate::symbol::PLAY_ARROW };
            key(b, glyph, left, play, true);
            left += 52.0;
            let mut right = w - m;
            if let Some(on) = v.next {
                right -= 40.0;
                key(b, crate::symbol::SKIP_NEXT, right, Some(self.base + 3), on);
                right -= 8.0;
            }
            // Time text (narrow: the position only).
            let shown = self.dragging.unwrap_or(v.position).clamp(0.0, self.duration.max(v.position));
            let cap = mk::typo::CAPTION;
            let text = time_text(shown, self.duration, w < Self::NARROW);
            let (lw, lh) = engine.measure_styled(&text, cap.px(), cap.weight);
            b.styled(engine, text, cap, p.on_surface_variant, right - lw, y + (h - lh) / 2.0, None, None);
            // The timeline.
            let t = Rect { x: left, y: y + h / 2.0 - 2.0, w: (right - lw - m - left).max(10.0), h: 4.0 };
            self.track = t;
            let at = |s: f32| if self.duration > 0.0 { (s / self.duration).clamp(0.0, 1.0) } else { 0.0 };
            let ratio = at(shown);
            b.quad(t.x, t.y, t.w, 4.0, p.on_surface.over(p.surface_container, 0.18), 2.0, line, None);
            if let Some(ahead) = v.ahead {
                let reach = if ahead.is_finite() { at(v.position + ahead as f32) } else { 1.0 };
                if reach > ratio {
                    b.quad(t.x + t.w * ratio, t.y, t.w * (reach - ratio), 4.0, p.on_surface.over(p.surface_container, 0.42), 2.0, line, None);
                }
            }
            for (from, to) in v.marks {
                let (x0, x1) = (t.x + t.w * at(*from), t.x + t.w * at(*to));
                b.quad(x0, t.y, (x1 - x0).max(2.0), 4.0, p.tertiary, 2.0, line, None);
            }
            b.quad(t.x, t.y, (t.w * ratio).max(2.0), 4.0, p.primary, 2.0, line, None);
            b.quad(t.x + t.w * ratio - 9.0, t.y - 7.0, 18.0, 18.0, p.primary, 9.0, line, None);
            // A generous hit strip for fingers.
            b.quad(t.x, t.y - 14.0, t.w, 32.0, clear, 0.0, line, None);
            h
        }

        fn time_at(&self, x: f32) -> f32 {
            let t = self.track;
            ((x - t.x) / t.w.max(1.0)).clamp(0.0, 1.0) * self.duration
        }

        /// A press on one of the bar's roles.
        pub fn strike(&mut self, role: Option<usize>, x: f32) -> Option<MediaDeed> {
            match role {
                Some(r) if r == self.base => {
                    mk::feel::tap();
                    Some(MediaDeed::Toggle)
                }
                Some(r) if r == self.base + 1 => {
                    self.dragging = Some(self.time_at(x));
                    Some(MediaDeed::Scrub)
                }
                Some(r) if r == self.base + 2 => {
                    mk::feel::tap();
                    Some(MediaDeed::Prev)
                }
                Some(r) if r == self.base + 3 => {
                    mk::feel::tap();
                    Some(MediaDeed::Next)
                }
                _ => None,
            }
        }

        /// The finger moves: true while it drags the timeline (redraw).
        pub fn travel(&mut self, x: f32) -> bool {
            if self.dragging.is_some() {
                self.dragging = Some(self.time_at(x));
                return true;
            }
            false
        }

        /// The finger lifts: the time to seek to, if it dragged.
        pub fn release(&mut self) -> Option<f32> {
            self.dragging.take()
        }
    }

    /// The bar's time: «1:15 / 3:20», narrow only «1:15» (round 172).
    pub fn time_text(position: f32, duration: f32, narrow: bool) -> String {
        if narrow || duration <= 0.0 {
            clock_text(position)
        } else {
            format!("{} / {}", clock_text(position), clock_text(duration))
        }
    }

    #[cfg(test)]
    mod media_tests {
        use super::*;

        #[test]
        fn the_bar_has_its_skip_keys() {
            let mut bar = MediaBar::new(10);
            assert!(bar.owns(Some(12)) && bar.owns(Some(13)) && !bar.owns(Some(14)));
            assert_eq!(bar.strike(Some(12), 0.0), Some(MediaDeed::Prev));
            assert_eq!(bar.strike(Some(13), 0.0), Some(MediaDeed::Next));
            assert_eq!(time_text(75.0, 200.0, false), "1:15 / 3:20");
            assert_eq!(time_text(75.0, 200.0, true), "1:15");
        }

        #[test]
        fn clock_text_reads_like_a_player() {
            assert_eq!(clock_text(0.0), "0:00");
            assert_eq!(clock_text(75.9), "1:15");
            assert_eq!(clock_text(3725.0), "1:02:05");
        }

        #[test]
        fn paragraphs_and_letters() {
            let p = paragraphs("# Titel\n\nEin *Satz*.\n- und so\n\n\n");
            assert_eq!(p, vec![(true, "Titel".to_string()), (false, "Ein Satz. und so".to_string())]);
            assert_eq!(letters("«Haus»,"), "Haus");
            assert_eq!(letters("..."), "");
        }

        #[test]
        fn word_ranges_cut_at_spaces() {
            assert_eq!(word_ranges("  Matrix  Tube "), vec![(2, 8), (10, 14)]);
            assert_eq!(word_ranges("Grösse äh"), vec![(0, 7), (8, 11)]);
            assert!(word_ranges("   ").is_empty());
        }

        #[test]
        fn dot_join_skips_empty_parts() {
            assert_eq!(dot_join(&["MP3", "", "3:12"]), "MP3 · 3:12");
            assert_eq!(dot_join(&["", ""]), "");
        }

        #[test]
        fn letterbox_keeps_the_aspect_and_centres() {
            let stage = Rect { x: 0.0, y: 100.0, w: 400.0, h: 400.0 };
            let r = letterbox(stage, 1920, 1080);
            assert_eq!((r.w, r.h), (400.0, 225.0));
            assert_eq!(r.y, 100.0 + 87.0);
            assert_eq!(letterbox(stage, 0, 10).w, 0.0);
        }

        #[test]
        fn media_bar_scrubs_and_seeks() {
            let mut bar = MediaBar::new(10);
            bar.track = Rect { x: 100.0, y: 0.0, w: 200.0, h: 4.0 };
            bar.duration = 60.0;
            assert!(bar.owns(Some(11)) && bar.owns(Some(13)) && !bar.owns(Some(14)) && !bar.owns(Some(9)));
            assert_eq!(bar.strike(Some(11), 200.0), Some(MediaDeed::Scrub));
            assert!(bar.travel(250.0));
            assert_eq!(bar.release(), Some(45.0));
            assert!(!bar.travel(250.0));
        }
    }

    /// The built header including bookkeeping: light_slots are the
    /// three traffic lights (close/shelf/maximise), title_slot the
    /// title's text measure; on hover glyphs carries the three light
    /// glyphs with their offset RELATIVE to the respective light.
    pub struct Header {
        pub node: Node,
        pub title: Label,
        pub title_slot: usize,
        pub light_slots: [usize; 3],
        pub glyphs: Vec<(Label, f32, f32)>,
        /// The WINDOW BUTTON on the right (roll up/unroll) and its
        /// Placed. The light path lacked it entirely — the reference
        /// carries it in every window bar (finger probe 16.08.2026).
        pub shade: Label,
        pub shade_slot: usize,
        /// R207: the harness EXTENSION (e.g. Hardline) — sits in the
        /// window button's place when the app names one: glyph and name
        /// with their slots (the wrap is `shade_slot`, the dot
        /// `shade_slot + 1`).
        pub badge_glyph: Option<(Label, usize)>,
        pub badge_name: Option<(Label, usize)>,
        /// The REAL height of this bar. The reference drives it from
        /// the content (title button + padding M), so it grows along
        /// with the font size setting. Whoever builds on below takes
        /// THIS number, not the fixed SHADE_HEIGHT — otherwise with a
        /// large font the content overlaps the bar (finger probe
        /// 16.08.2026).
        pub height: f32,
    }

    /// Traffic light diameter (R168g) and the glyph size inside it.
    pub const LIGHT: f32 = 14.0;
    /// The glyph size of the window button (like the reference: 14).
    pub const SHADE_GLYPH: f32 = 14.0;

    pub fn frame_header(
        engine: &mut TextEngine,
        title: &str,
        state: HeaderState,
        width: f32,
        p: mk::Palette,
    ) -> Header {
        frame_header_with(engine, title, state, width, None, p)
    }

    /// R207 (the owner): «The harness of MatrixKit applications can be
    /// extended with functions, such as Hardline, so that the
    /// application works at all.» A harness EXTENSION shows itself at
    /// the top right in the window button's place: dot (on/off),
    /// glyph, name. A click on its role opens the app's extension
    /// panel (`extension_panel`).
    pub struct ExtensionBadge<'a> {
        pub name: &'a str,
        pub glyph: char,
        pub on: bool,
    }

    /// Placed of an extension in the header: wrap, dot, glyph measure,
    /// name measure.
    pub const PLACED_PER_BADGE: usize = 4;

    pub fn frame_header_with(
        engine: &mut TextEngine,
        title: &str,
        state: HeaderState,
        width: f32,
        badge: Option<&ExtensionBadge<'_>>,
        p: mk::Palette,
    ) -> Header {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let gray = p.outline.over(p.surface_container_high, 0.6);
        let tints = [p.error, p.secondary, p.primary];
        let head = mk::typo::SUBTITLE;
        let (_, th) = engine.measure_styled(title, head.px(), head.weight);
        // In the reference the title sits in a button with 2 px padding
        // above and below; the traffic lights set the lower bound.
        // R209: the device's lights (duo: 24 pt, glyphs always, no shelf
        // light — its slot stays, 0 wide, so every app keeps its three
        // light slots and roles).
        let form = mk::device::lights();
        let light = form.size;
        let core = (th + 4.0).max(light);
        // The window button: the same glyph and size as the reference
        // (14 px, on_surface_variant or outline).
        let symfam = crate::symbol_family();
        let shade_glyph = if state.rolled_up { symbol::UNFOLD_MORE } else { symbol::UNFOLD_LESS };
        let shade_s = shade_glyph.to_string();
        let (sw, sh) =
            engine.measure_family(symfam, &shade_s, mk::typo::symbol_px(SHADE_GLYPH), mk::typo::Weight::Normal);

        // The extension's BADGE (R207): dot, glyph, name in a capsule —
        // four Placed (wrap, dot, glyph, name).
        let cap = mk::typo::CAPTION;
        let mut badge_labels: Option<(Label, Label)> = None;
        let mut badge_nodes: Option<Node> = badge.map(|b| {
            let ink = if b.on { p.on_surface } else { p.on_surface_variant };
            let dot = if b.on { p.primary } else { p.outline };
            let gs = b.glyph.to_string();
            let gpx = mk::typo::symbol_px(SHADE_GLYPH);
            let (gw, gh) = engine.measure_family(symfam, &gs, gpx, mk::typo::Weight::Normal);
            let (nw, nh) = engine.measure_styled(b.name, cap.px(), cap.weight);
            badge_labels = Some((
                Label { text: gs, px: gpx, weight: mk::typo::Weight::Normal, family: symfam, color: ink },
                Label { text: b.name.to_string(), px: cap.px(), weight: cap.weight, family: matrixkit_text::FAMILY, color: ink },
            ));
            Node::Wrap {
                pad_x: mk::spacing::S,
                pad_y: 2.0,
                background: Some((p.on_surface.over(p.surface_container_high, 0.06), mk::radius::capsule(core))),
                child: Box::new(Node::Row {
                    gap: mk::spacing::XS,
                    padding: 0.0,
                    background: None,
                    // Dot and glyph sit vertically CENTRED to the name —
                    // a slot without a ground lays nothing, so it does not count.
                    children: vec![
                        Node::Slot { w: Some(8.0), h: Some(nh), ax: Align::Center, ay: Align::Center, background: None, child: Box::new(Node::Quad { w: 8.0, h: 8.0, color: dot, radius: 4.0 }) },
                        Node::Slot { w: Some(gw), h: Some(nh), ax: Align::Center, ay: Align::Center, background: None, child: Box::new(Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 }) },
                        Node::Quad { w: nw, h: nh, color: clear, radius: 0.0 },
                    ],
                }),
            }
        });

        // R207: the title SHORTENS itself instead of running under the
        // traffic lights or the badge — at phone width a beginning of
        // the page title with an ellipsis remains.
        let badge_w = badge_nodes
            .as_ref()
            .map(|n| {
                let mut v = Vec::new();
                matrixkit_theme::scene::lay_out(n, 0.0, 0.0, &mut v).0
            })
            .unwrap_or(sw + 12.0);
        let lights_w = if form.shelf {
            3.0 * light + 2.0 * mk::spacing::S
        } else {
            2.0 * light + mk::spacing::S
        };
        let side_w = lights_w.max(badge_w);
        let inner_w = width - 2.0 * mk::spacing::L;
        // Centred in the window as long as the title has room there
        // (the canvas sees no difference); only when it gets tight does
        // it move into the GAP between traffic lights and badge and
        // shorten there — so more title remains at phone width.
        let avail_center = (inner_w - 2.0 * side_w - 2.0 * mk::spacing::S).max(0.0);
        let gap_w = (inner_w - lights_w - badge_w - 2.0 * mk::spacing::S).max(40.0);
        let (tw_full, _) = engine.measure_styled(title, head.px(), head.weight);
        let in_gap = tw_full > avail_center;
        let title = if in_gap { ellipsize(engine, title, head.px(), head.weight, gap_w) } else { title.to_string() };
        let (tw, th) = engine.measure_styled(&title, head.px(), head.weight);
        // The MAGNIFIER (Lager 91): every traffic light springs
        // INDIVIDUALLY under the pointer. The box keeps its size — only
        // the content grows, so the arrangement does not jump and the
        // hit area stays the same (like `mkw::magnifier`).
        // The gaps are empty slots (they lay no Placed), so a missing shelf
        // light also takes its gap with it.
        let spacer = |w: f32| Node::Slot {
            w: Some(w),
            h: Some(light),
            ax: Align::Start,
            ay: Align::Center,
            background: None,
            child: Box::new(Node::Row { gap: 0.0, padding: 0.0, background: None, children: Vec::new() }),
        };
        let mut lights_children = Vec::new();
        for (i, t) in tints.iter().enumerate() {
            let size = if i == 1 && !form.shelf { 0.0 } else { light };
            if i > 0 && size > 0.0 {
                lights_children.push(spacer(mk::spacing::S));
            }
            lights_children.push(Node::Slot {
                w: Some(size),
                h: Some(light),
                ax: Align::Center,
                ay: Align::Center,
                background: None,
                child: Box::new(Node::Scale {
                    factor: state.magnify[i],
                    child: Box::new(Node::Quad {
                        w: size,
                        h: size,
                        color: if state.active { *t } else { gray },
                        radius: mk::radius::capsule(light),
                    }),
                }),
            });
        }
        let lights_row = Node::Row { gap: 0.0, padding: 0.0, background: None, children: lights_children };
        // Lager 50: title CENTRED — laid, not computed. Two boxes of the
        // same width in the stack: the traffic lights on the left, the
        // title centred; both vertically centred. Before, three spacer
        // quads and a 64 counterweight did this, whose widths the family
        // had to compute itself (and which jammed when space was tight).
        // Horizontally L, vertically M — like the reference
        // (theme_header: .padding([M, L])). With M on both sides the
        // traffic lights and window button each sat four logical pixels
        // too far inside (finger probe 16.08.2026).
        let inner = width - 2.0 * mk::spacing::L;
        // The REAL height, analytically — the bar needs it already when
        // building, not only after laying.
        let head_h = core + 2.0 * mk::spacing::M;
        let content = Node::Wrap {
            pad_x: mk::spacing::L,
            // Padding M like the reference — NOT stretched to a fixed
            // height.
            pad_y: mk::spacing::M,
            background: None,
            child: Box::new(Node::Stack {
                children: vec![
                    Node::Slot {
                        w: Some(inner),
                        h: Some(core),
                        ax: Align::Start,
                        ay: Align::Center,
                        background: None,
                        child: Box::new(lights_row),
                    },
                    {
                        let title_box = Node::Slot {
                            w: Some(tw),
                            h: Some(th),
                            ax: Align::Center,
                            ay: Align::Center,
                            background: None,
                            child: Box::new(Node::Scale {
                                factor: state.magnify[3],
                                child: Box::new(Node::Quad {
                                    w: tw,
                                    h: th,
                                    color: clear,
                                    radius: 0.0,
                                }),
                            }),
                        };
                        if in_gap {
                            // Empty rows lay no Placed: the slots
                            // (title = 5) stay where they are.
                            Node::Row {
                                gap: 0.0,
                                padding: 0.0,
                                background: None,
                                children: vec![
                                    Node::Slot {
                                        w: Some(lights_w + mk::spacing::S),
                                        h: Some(core),
                                        ax: Align::Start,
                                        ay: Align::Center,
                                        background: None,
                                        child: Box::new(Node::Row { gap: 0.0, padding: 0.0, background: None, children: Vec::new() }),
                                    },
                                    Node::Slot {
                                        w: Some(gap_w),
                                        h: Some(core),
                                        ax: Align::Center,
                                        ay: Align::Center,
                                        background: None,
                                        child: Box::new(title_box),
                                    },
                                ],
                            }
                        } else {
                            Node::Slot {
                                w: Some(inner),
                                h: Some(core),
                                ax: Align::Center,
                                ay: Align::Center,
                                background: None,
                                child: Box::new(title_box),
                            }
                        }
                    },
                    // The WINDOW BUTTON sits on the right, vertically
                    // centred — with its own button padding (the
                    // reference gives the button .padding([2, 6])). If the
                    // app names an EXTENSION, its badge sits here.
                    Node::Slot {
                        w: Some(inner),
                        h: Some(core),
                        ax: Align::End,
                        ay: Align::Center,
                        background: None,
                        child: Box::new(match badge_nodes.take() {
                            Some(b) => b,
                            None => Node::Wrap {
                                pad_x: 6.0,
                                pad_y: 2.0,
                                background: None,
                                child: Box::new(Node::Slot {
                                    w: Some(sw),
                                    h: Some(sh),
                                    ax: Align::Center,
                                    ay: Align::Center,
                                    background: None,
                                    child: Box::new(Node::Scale {
                                        factor: state.magnify[4],
                                        child: Box::new(Node::Quad {
                                            w: sw,
                                            h: sh,
                                            color: clear,
                                            radius: 0.0,
                                        }),
                                    }),
                                }),
                            },
                        }),
                    },
                ],
            }),
        };
        // The STRAIGHT BOTTOM EDGE (Lager 90).
        //
        // A quad has ONE radius, so four equal corners. But the header
        // always sits at the TOP of its container, and at the bottom the
        // window edge continues — nothing may buckle there. Measured,
        // the bar pulled in at the bottom by up to 9 logical pixels: the
        // window side jumped in at the header edge and straight back
        // out, and exactly that bothered the owner about the design
        // (17.08.2026).
        //
        // Two quads solve it without new geometry in the rasteriser: the
        // rounded ground, and above it a STRAIGHT strip of the radius'
        // height at the bottom edge. Both carry the same colour — the
        // overlap is invisible as long as the colour is opaque (guard
        // `header_ground_is_opaque`).
        let node = Node::Stack {
            children: vec![
                Node::Quad {
                    w: width,
                    h: head_h,
                    color: p.surface_container_high,
                    radius: mk::CORNER_RADIUS,
                },
                Node::Slot {
                    w: Some(width),
                    h: Some(head_h),
                    ax: Align::Start,
                    ay: Align::End,
                    background: None,
                    child: Box::new(Node::Quad {
                        w: width,
                        h: mk::CORNER_RADIUS,
                        color: p.surface_container_high,
                        radius: 0.0,
                    }),
                },
                content,
            ],
        };
        // Placed: [0] header ground, [1] straight bottom edge,
        // [2..=4] traffic lights, [5] title measure, [6] window button —
        // or [6..=9] the extension's badge (wrap, dot, glyph, name).
        let with_badge = badge_labels.is_some();
        debug_assert_eq!(matrixkit_theme::scene::placed_count(&node), if with_badge { 10 } else { 7 });
        let title_color = if state.active { p.on_surface } else { p.on_surface_variant };
        let mut glyphs = Vec::new();
        // R209: a finger has no hover — the duo shows the glyphs always.
        if state.lights_hover || form.glyphs {
            let family = crate::symbol_family();
            let ink = mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.55 };
            for (i, g) in [symbol::CLOSE, symbol::REMOVE, symbol::ADD].into_iter().enumerate() {
                // The magnifier takes the glyph ALONG: it lives on the
                // traffic light, so it grows with it (Lager 91). The
                // offset counts in the GROWN box, not the resting one —
                // otherwise it slips out of the centre.
                let f = state.magnify[i].max(0.01);
                // No shelf light: its glyph stays empty (index k = slot k).
                let s = if i == 1 && !form.shelf { String::new() } else { g.to_string() };
                let px = if form.glyphs { 0.75 * light * f } else { 10.5 * f };
                let (gw, gh) = engine.measure_family(family, &s, px, mk::typo::Weight::Normal);
                glyphs.push((
                    Label { text: s, px, weight: mk::typo::Weight::Normal, family, color: ink },
                    (light * f - gw) / 2.0,
                    (light * f - gh) / 2.0,
                ));
            }
        }
        Header {
            node,
            title: Label {
                text: title.to_string(),
                // The title springs with its box (Lager 91): stretching
                // the box alone would leave the text standing.
                px: head.px() * state.magnify[3],
                weight: head.weight,
                family: matrixkit_text::FAMILY,
                color: title_color,
            },
            title_slot: 5,
            light_slots: [2, 3, 4],
            shade: Label {
                text: shade_s,
                px: mk::typo::symbol_px(SHADE_GLYPH) * state.magnify[4],
                weight: mk::typo::Weight::Normal,
                family: symfam,
                color: if state.active { p.on_surface_variant } else { p.outline },
            },
            shade_slot: 6,
            badge_glyph: badge_labels.as_ref().map(|(g, _)| (g.clone(), 8)),
            badge_name: badge_labels.map(|(_, n)| (n, 9)),
            height: head_h,
            glyphs,
        }
    }

    /// The EXTENSION PANEL (R207): the floating card under the badge —
    /// title row (glyph, name, state), rows about the status, and
    /// actions as text buttons. Placed ABSOLUTE (the card lies over the
    /// page), roles = index of the action; `hover` is the index of the
    /// action under the pointer.
    pub struct ExtensionPanel<'a> {
        pub name: &'a str,
        pub glyph: char,
        pub on: bool,
        pub lines: &'a [String],
        pub actions: &'a [&'a str],
        pub hover: Option<usize>,
    }

    pub struct ExtensionPanelBuild {
        pub placed: Vec<Placed>,
        pub roles: Vec<Option<usize>>,
        pub labels: Vec<(Label, usize)>,
        pub height: f32,
    }

    /// The card hangs by its right edge at `right`, starts at `top`, is
    /// `width` wide.
    pub fn extension_panel(
        engine: &mut TextEngine,
        panel: &ExtensionPanel<'_>,
        right: f32,
        top: f32,
        width: f32,
        p: mk::Palette,
    ) -> ExtensionPanelBuild {
        let mut b = Sheet::new();
        let x = right - width;
        let inner = width - 2.0 * mk::spacing::M;
        // First measure the content (height), then the ground before it:
        // the ground must be the FIRST slot (z-order), so lay twice.
        let mut probe = Sheet::new();
        let h = panel_rows(engine, &mut probe, panel, x, top, inner, p);
        let ground = Node::Quad { w: width, h, color: p.surface_container_high, radius: mk::radius::NORMAL };
        let g = b.node(&ground, x, top, None, None).0;
        lift(&mut b.placed, g, floating_shadow());
        let _ = panel_rows(engine, &mut b, panel, x, top, inner, p);
        let labels = b.texts.iter().map(|t| (Label { text: t.s.clone(), px: t.px, weight: t.weight, family: t.family, color: t.color }, t.anchor)).collect();
        ExtensionPanelBuild { placed: b.placed, roles: b.roles, labels, height: h }
    }

    fn panel_rows(engine: &mut TextEngine, b: &mut Sheet, panel: &ExtensionPanel<'_>, x: f32, top: f32, inner: f32, p: mk::Palette) -> f32 {
        let mut y = top + mk::spacing::M;
        let ix = x + mk::spacing::M;
        // Title row: dot, glyph, name, state.
        let dot = if panel.on { p.primary } else { p.outline };
        b.quad(ix, y + 6.0, 8.0, 8.0, dot, 4.0, None, None);
        let (gn, gl) = glyph(engine, panel.glyph, mk::font_size::MEDIUM, p.on_surface);
        let (gf, _, gw, _) = b.node(&gn, ix + 14.0, y, None, None);
        b.label(gf, &gl, 0.0, 0.0, None);
        let sub = mk::typo::SUBTITLE;
        let (nn, nl) = plain_label(engine, panel.name, sub.px(), sub.weight, p.on_surface);
        let (nf, _, _, nh) = b.node(&nn, ix + 14.0 + gw + mk::spacing::XS, y, None, None);
        b.label(nf, &nl, 0.0, 0.0, None);
        let state = if panel.on { "läuft" } else { "aus" };
        let cap = mk::typo::CAPTION;
        let (sn, sl) = plain_label(engine, state, cap.px(), cap.weight, if panel.on { p.primary } else { p.on_surface_variant });
        let (sw, _) = engine.measure_styled(state, cap.px(), cap.weight);
        let (sf, _, _, _) = b.node(&sn, x + inner + mk::spacing::M - sw, y + 2.0, None, None);
        b.label(sf, &sl, 0.0, 0.0, None);
        y += nh + mk::spacing::S;
        for line in panel.lines {
            y += b.paragraph(engine, line, mk::typo::CAPTION, p.on_surface_variant, ix, y, inner, None, None) + mk::spacing::XS;
        }
        y += mk::spacing::XS;
        for (i, a) in panel.actions.iter().enumerate() {
            let status = if panel.hover == Some(i) { Status::Hovered } else { Status::Idle };
            let (an, al) = text_button_with(engine, a, status, p);
            let (af, _, _, ah) = b.node(&an, ix - LINK_PAD.0, y, Some(i), None);
            b.label(af + 1, &al, 0.0, 0.0, None);
            y += ah - 2.0 * LINK_PAD.1 + mk::spacing::S;
        }
        y - top + mk::spacing::S
    }

    /// Lager 23: the ONE editing logic of the field — caret in
    /// characters, selection from the anchor, shift spans, cut before
    /// insert. Born in the probe field (Lager 19/20), lifted here so
    /// every field speaks THE SAME behaviour.
    /// Lager 64: the ELEVATION family. The tokens are literally those
    /// of the iced version (mkw::elevation): the resting card barely
    /// lifts (0.10, 1 px deep, 5 soft), the floating surface clearly
    /// (0.32, 10 px deep, 30 soft). "Shadows communicate level, never
    /// drama."
    ///
    /// It is set AFTER laying — the same seam as with the viewport:
    /// the layout knows only geometry.
    pub fn card_shadow() -> matrixkit_theme::scene::Shadow {
        matrixkit_theme::scene::Shadow {
            color: mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.10 },
            dx: 0.0,
            dy: 1.0,
            blur: 5.0,
        }
    }

    pub fn floating_shadow() -> matrixkit_theme::scene::Shadow {
        matrixkit_theme::scene::Shadow {
            color: mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.32 },
            dx: 0.0,
            dy: 10.0,
            blur: 30.0,
        }
    }

    /// Lifts a laid quad onto a level. Whoever floats casts a shadow —
    /// and FROM THEN ON it belongs to the damage, because Rect::of
    /// takes it into the hull.
    pub fn lift(scene: &mut [Placed], index: usize, shadow: matrixkit_theme::scene::Shadow) {
        if let Some(q) = scene.get_mut(index) {
            q.shadow = Some(shadow);
        }
    }

    /// Lager 63: the MULTI-LINE EDITOR (iced's text_editor) — the
    /// hardest single piece of the ladder, deliberately last.
    ///
    /// Since Lager 19/20 the single-line field can do caret, selection
    /// with shift, repeat and blinking. What is missing is the SECOND
    /// DIMENSION: up and down, selection across line boundaries, and
    /// an enter that SPLITS the line instead of submitting.
    ///
    /// Lines here are LOGICAL lines (\n); wrapping at the column edge
    /// is the display's business (TextEngine::wrap_styled) and a
    /// reminder of its own.
    ///
    /// COLUMN FIDELITY (reference grammar): whoever wanders up and
    /// down keeps their DESIRED column, even across short lines —
    /// otherwise the caret slips left at every jump over a short line
    /// and never comes back.
    #[derive(Default)]
    pub struct EditorState {
        pub cursor: usize,
        pub anchor: Option<usize>,
        /// The desired column for wandering; None means "derive from
        /// the current caret".
        pub wish: Option<usize>,
    }

    /// Line start, line end and column of a caret — the geometry all
    /// movements rest on.
    fn line_at(value: &str, cursor: usize) -> (usize, usize, usize) {
        let chars: Vec<char> = value.chars().collect();
        let cursor = cursor.min(chars.len());
        let start = chars[..cursor].iter().rposition(|c| *c == '\n').map_or(0, |i| i + 1);
        let end = chars[cursor..]
            .iter()
            .position(|c| *c == '\n')
            .map_or(chars.len(), |i| cursor + i);
        (start, end, cursor - start)
    }

    pub fn editor_edit(
        value: &mut String,
        state: &mut EditorState,
        key: &matrixkit_shell::Key,
        shift: bool,
    ) -> FieldDeed {
        use matrixkit_shell::Key;
        let at = |s: &str, i: usize| s.char_indices().nth(i).map(|(b, _)| b).unwrap_or(s.len());
        let count = value.chars().count();
        let mut keep_wish = false;

        let cut = |value: &mut String, state: &mut EditorState| {
            let Some(a) = state.anchor else { return false };
            if a == state.cursor {
                state.anchor = None;
                return false;
            }
            let (lo, hi) = (a.min(state.cursor), a.max(state.cursor));
            let (b0, b1) = (at(value, lo), at(value, hi));
            value.replace_range(b0..b1, "");
            state.cursor = lo;
            state.anchor = None;
            true
        };
        let travel = |to: usize, state: &mut EditorState| {
            if shift {
                if state.anchor.is_none() {
                    state.anchor = Some(state.cursor);
                }
            } else {
                state.anchor = None;
            }
            state.cursor = to;
        };

        match key {
            Key::Char(t) => {
                cut(value, state);
                let b = at(value, state.cursor);
                value.insert_str(b, t);
                state.cursor += t.chars().count();
            }
            // Enter SPLITS the line — the one difference to the field,
            // where it submits.
            Key::Enter => {
                cut(value, state);
                let b = at(value, state.cursor);
                value.insert(b, '\n');
                state.cursor += 1;
            }
            Key::Backspace => {
                if !cut(value, state) && state.cursor > 0 {
                    state.cursor -= 1;
                    let b = at(value, state.cursor);
                    value.remove(b);
                }
            }
            Key::Delete => {
                if !cut(value, state) && state.cursor < count {
                    let b = at(value, state.cursor);
                    value.remove(b);
                }
            }
            Key::Left => {
                let to = state.cursor.saturating_sub(1);
                travel(to, state);
            }
            Key::Right => {
                let to = (state.cursor + 1).min(count);
                travel(to, state);
            }
            // Home and End apply to the LINE, not the text.
            Key::Home => {
                let (start, _, _) = line_at(value, state.cursor);
                travel(start, state);
            }
            Key::End => {
                let (_, end, _) = line_at(value, state.cursor);
                travel(end, state);
            }
            Key::Up | Key::Down => {
                let (start, end, col) = line_at(value, state.cursor);
                let wish = state.wish.unwrap_or(col);
                let to = if matches!(key, Key::Up) {
                    if start == 0 {
                        0
                    } else {
                        let (prev_start, _, _) = line_at(value, start - 1);
                        prev_start + wish.min(start - 1 - prev_start)
                    }
                } else if end >= count {
                    count
                } else {
                    let (_, next_end, _) = line_at(value, end + 1);
                    (end + 1 + wish).min(next_end)
                };
                travel(to, state);
                state.wish = Some(wish);
                keep_wish = true;
            }
            Key::Escape => return FieldDeed::Escaped,
            _ => return FieldDeed::Calm,
        }
        if !keep_wish {
            state.wish = None;
        }
        FieldDeed::Changed
    }

    pub enum FieldDeed {
        /// Value or caret changed — repaint.
        Changed,
        /// Enter: the value stands.
        Submitted,
        /// Escape: the field gives up focus.
        Escaped,
        /// Nothing for the field.
        Calm,
    }

    pub fn field_edit(
        value: &mut String,
        cursor: &mut usize,
        anchor: &mut Option<usize>,
        key: &matrixkit_shell::Key,
        shift: bool,
    ) -> FieldDeed {
        use matrixkit_shell::Key;
        let at = |s: &str, i: usize| s.char_indices().nth(i).map(|(b, _)| b).unwrap_or(s.len());
        let mut cut = || {
            let Some(a) = *anchor else { return false };
            if a == *cursor {
                *anchor = None;
                return false;
            }
            let (lo, hi) = (a.min(*cursor), a.max(*cursor));
            let (b0, b1) = (at(value, lo), at(value, hi));
            value.replace_range(b0..b1, "");
            *cursor = lo;
            *anchor = None;
            true
        };
        let travel = |to: usize, cursor: &mut usize, anchor: &mut Option<usize>| {
            if shift {
                if anchor.is_none() {
                    *anchor = Some(*cursor);
                }
            } else {
                *anchor = None;
            }
            *cursor = to;
        };
        match key {
            Key::Char(t) => {
                cut();
                let b = at(value, *cursor);
                value.insert_str(b, t);
                *cursor += t.chars().count();
            }
            Key::Backspace => {
                if !cut() && *cursor > 0 {
                    *cursor -= 1;
                    let b = at(value, *cursor);
                    value.remove(b);
                }
            }
            Key::Delete => {
                if !cut() && *cursor < value.chars().count() {
                    let b = at(value, *cursor);
                    value.remove(b);
                }
            }
            Key::Left => {
                let to = cursor.saturating_sub(1);
                travel(to, cursor, anchor);
            }
            Key::Right => {
                let to = (*cursor + 1).min(value.chars().count());
                travel(to, cursor, anchor);
            }
            Key::Home => travel(0, cursor, anchor),
            Key::End => {
                let to = value.chars().count();
                travel(to, cursor, anchor);
            }
            Key::Enter => return FieldDeed::Submitted,
            Key::Escape => return FieldDeed::Escaped,
            _ => return FieldDeed::Calm,
        }
        FieldDeed::Changed
    }

    /// Lager 21: the PROGRESS (mkw::progress) on scene floor — quiet
    /// track, accent fill, MINI radius. Two Placed: track, fill.
    pub fn progress(ratio: f32, width: f32, p: mk::Palette) -> Node {
        let ratio = ratio.clamp(0.0, 1.0);
        Node::Row {
            gap: 0.0,
            padding: 0.0,
            background: Some((
                p.on_surface.over(p.surface_container, 0.10),
                mk::radius::MINI,
            )),
            children: vec![Node::Quad {
                w: (width * ratio).max(1.0),
                h: 6.0,
                color: p.primary,
                radius: mk::radius::MINI,
            }, Node::Quad {
                w: (width * (1.0 - ratio)).max(0.0),
                h: 0.0,
                color: mk::Rgba { a: 0.0, ..p.surface },
                radius: 0.0,
            }],
        }
    }

    /// Placed per progress: track + fill + remaining spacer.
    pub const PLACED_PER_PROGRESS: usize = 3;

    /// Lager 21: the HISTORY CARD — born as the probe's diff history
    /// (Lager 16), turned into a family with the monitor move: card in
    /// tile chrome (ring + concentric fill), CAPTION title in variant
    /// text, the curve box below. The CURVE itself the app lays as a
    /// scene citizen onto the box (curve_slot) — the family only lays
    /// the stage.
    pub struct HistoryCard {
        pub node: Node,
        pub label: Label,
        /// Placed offset of the title's text measure.
        pub label_slot: usize,
        /// Placed offset of the curve box.
        pub curve_slot: usize,
    }

    pub fn history_card(
        engine: &mut TextEngine,
        title: &str,
        curve_w: f32,
        curve_h: f32,
        p: mk::Palette,
    ) -> HistoryCard {
        let cap = mk::typo::CAPTION;
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let (tw, th) = engine.measure_styled(title, cap.px(), cap.weight);
        let node = Node::Wrap {
            pad_x: 1.0,
            pad_y: 1.0,
            background: Some((p.outline.with_alpha(0.3), mk::radius::LARGE)),
            child: Box::new(Node::Wrap {
                pad_x: mk::spacing::M - 1.0,
                pad_y: mk::spacing::S - 1.0,
                background: Some((
                    p.surface_container_high,
                    mk::radius::inner(mk::radius::LARGE, 1.0),
                )),
                child: Box::new(Node::Column {
                    gap: mk::spacing::XS,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Quad { w: tw, h: th, color: clear, radius: 0.0 },
                        Node::Quad { w: curve_w, h: curve_h, color: clear, radius: 0.0 },
                    ],
                }),
            }),
        };
        debug_assert_eq!(matrixkit_theme::scene::placed_count(&node), 4);
        HistoryCard {
            node,
            label: Label {
                text: title.to_string(),
                px: cap.px(),
                weight: cap.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface_variant,
            },
            label_slot: 2,
            curve_slot: 3,
        }
    }

    /// Lager 17: the fleet now lives in the window house
    /// (matrixkit-shell) — the familiar name stays here as a re-export,
    /// the contracts are unchanged.
    pub use matrixkit_shell::fleet;


    /// Lager 11: the GLANCE moves in (Quick Look grammar, R98 — so far
    /// hand-rolled in Matrix Dateien, WITH the move it becomes a kit
    /// building block for the first time). The fleeting preview of the
    /// selection: a floating card, content on top, below it centred
    /// "the name leads" (SUBTITLE) and the subline (CAPTION, variant
    /// text).
    ///
    /// Content today: text head or hero glyph. IMAGE follows with the
    /// image atlas (stage 4), the ROLL in the text head with the
    /// state-carrying widgets, the card's SHADOW with the renderer
    /// storey — three honest reminders, no dummies.
    pub enum PeekContent {
        /// The first lines of a readable file (without wrapping —
        /// whoever measures, shortens).
        Text(String),
        /// The fallback glyph in icon_size::HERO and primary.
        Glyph(char),
        /// An image, contain on the 280 stage — fetched and blitted by
        /// the image atlas (Lager 15); the family only lays the box.
        Image(std::path::PathBuf),
    }

    /// The fixed width of the glance card (Matrix Dateien, R98).
    pub const PEEK_WIDTH: f32 = 640.0;

    /// The viewport height of the text head (Lager 14: the roll).
    pub const ROLL_VIEW: f32 = 140.0;

    /// The roll of the text head: where the viewport lies and how tall
    /// the content really is — the app paints the text OFFSET and
    /// CLIPPED (paint_clip) and keeps the offset in the state slot
    /// (mk::scene::Keep).
    pub struct PeekRoll {
        pub viewport_slot: usize,
        pub content_h: f32,
        pub view_h: f32,
    }

    /// The built glance including bookkeeping.
    pub struct PeekSheet {
        pub node: Node,
        pub labels: Vec<Label>,
        pub label_slots: Vec<usize>,
        pub roll: Option<PeekRoll>,
        /// Image content: (Placed slot of the box, path) — the app
        /// fetches and blits via the atlas.
        pub image: Option<(usize, std::path::PathBuf)>,
    }

    pub fn peek(
        engine: &mut TextEngine,
        name: &str,
        subtitle: &str,
        content: &PeekContent,
        roll_offset: f32,
        p: mk::Palette,
    ) -> PeekSheet {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mut labels = Vec::new();
        let mut label_slots = Vec::new();
        let mut roll = None;
        let mut image = None;
        // Placed offsets: [0] border ring, [1] card ground, then the
        // content in build order.
        let mut offset = 2usize;
        let inner_w = PEEK_WIDTH - 2.0 * mk::spacing::L;

        let content_node = match content {
            PeekContent::Text(t) => {
                let cap = mk::typo::CAPTION;
                let (tw, th) = engine.measure_styled(t, cap.px(), cap.weight);
                labels.push(Label {
                    text: t.clone(),
                    px: cap.px(),
                    weight: cap.weight,
                    family: matrixkit_text::FAMILY,
                    color: p.on_surface,
                });
                offset += 1; // the head ground
                if th > ROLL_VIEW {
                    // Lager 14: the content is taller than the box — it
                    // SCROLLS. Viewport on the left, thumb on the right
                    // (proportional, like every honest scroll bar); the
                    // app paints the text: offset by roll_offset, clipped
                    // to the box.
                    let view_h = ROLL_VIEW;
                    let thumb_h = (view_h * view_h / th).max(24.0);
                    let track = view_h - thumb_h;
                    let span = th - view_h;
                    let thumb_off =
                        if span > 0.0 { (roll_offset / span).clamp(0.0, 1.0) * track } else { 0.0 };
                    label_slots.push(offset); // the viewport is the anchor
                    roll = Some(PeekRoll { viewport_slot: offset, content_h: th, view_h });
                    offset += 3; // box + thumb spacer + thumb
                    let _ = tw;
                    Node::Wrap {
                        pad_x: mk::spacing::M,
                        pad_y: mk::spacing::M,
                        background: Some((p.surface_container, mk::radius::NORMAL)),
                        child: Box::new(Node::Row {
                            gap: mk::spacing::XS,
                            padding: 0.0,
                            background: None,
                            children: vec![
                                Node::Quad {
                                    w: inner_w - 2.0 * mk::spacing::M - 4.0 - mk::spacing::XS,
                                    h: view_h,
                                    color: clear,
                                    radius: 0.0,
                                },
                                Node::Column {
                                    gap: 0.0,
                                    padding: 0.0,
                                    background: None,
                                    children: vec![
                                        Node::Quad { w: 4.0, h: thumb_off, color: clear, radius: 0.0 },
                                        Node::Quad {
                                            w: 4.0,
                                            h: thumb_h,
                                            color: p.outline,
                                            radius: mk::radius::capsule(4.0),
                                        },
                                    ],
                                },
                            ],
                        }),
                    }
                } else {
                    // Fits without a roll: text measure + width spacer.
                    label_slots.push(offset);
                    offset += 2;
                    Node::Wrap {
                        pad_x: mk::spacing::M,
                        pad_y: mk::spacing::M,
                        background: Some((p.surface_container, mk::radius::NORMAL)),
                        child: Box::new(Node::Row {
                            gap: 0.0,
                            padding: 0.0,
                            background: None,
                            children: vec![
                                Node::Quad { w: tw, h: th, color: clear, radius: 0.0 },
                                Node::Quad {
                                    w: (inner_w - 2.0 * mk::spacing::M - tw).max(0.0),
                                    h: 0.0,
                                    color: clear,
                                    radius: 0.0,
                                },
                            ],
                        }),
                    }
                }
            }
            PeekContent::Image(path) => {
                // The box of the 280 stage; the image goes in contained,
                // centred — blitted by the app.
                image = Some((offset, path.clone()));
                offset += 1;
                Node::Quad { w: inner_w, h: 280.0, color: clear, radius: 0.0 }
            }
            PeekContent::Glyph(g) => {
                let s = g.to_string();
                let family = crate::symbol_family();
                let (gw, gh) =
                    engine.measure_family(family, &s, mk::icon_size::HERO, mk::typo::Weight::Normal);
                labels.push(Label {
                    text: s,
                    px: mk::icon_size::HERO,
                    weight: mk::typo::Weight::Normal,
                    family,
                    color: p.primary,
                });
                label_slots.push(offset);
                offset += 1;
                // The hero stage: 280 high, glyph centred (Matrix
                // Dateien takes 640×280 for the fallback).
                Node::Wrap {
                    pad_x: ((inner_w - gw) / 2.0).max(0.0),
                    pad_y: ((280.0 - gh) / 2.0).max(0.0),
                    background: None,
                    child: Box::new(Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 }),
                }
            }
        };

        let head = mk::typo::SUBTITLE;
        let cap = mk::typo::CAPTION;
        let (nw, nh) = engine.measure_styled(name, head.px(), head.weight);
        let (sw, sh) = engine.measure_styled(subtitle, cap.px(), cap.weight);
        labels.push(Label {
            text: name.to_string(),
            px: head.px(),
            weight: head.weight,
            family: matrixkit_text::FAMILY,
            color: p.on_surface,
        });
        offset += 1; // the S air spacer before the name
        label_slots.push(offset);
        offset += 1;
        labels.push(Label {
            text: subtitle.to_string(),
            px: cap.px(),
            weight: cap.weight,
            family: matrixkit_text::FAMILY,
            color: p.on_surface_variant,
        });
        label_slots.push(offset);
        offset += 1;

        let column = Node::Column {
            gap: 0.0,
            padding: 0.0,
            background: None,
            children: vec![
                content_node,
                Node::Quad { w: 0.0, h: mk::spacing::S, color: clear, radius: 0.0 },
                // Centred under the content: the name leads.
                Node::Wrap {
                    pad_x: ((inner_w - nw) / 2.0).max(0.0),
                    pad_y: 0.0,
                    background: None,
                    child: Box::new(Node::Quad { w: nw, h: nh, color: clear, radius: 0.0 }),
                },
                Node::Wrap {
                    pad_x: ((inner_w - sw) / 2.0).max(0.0),
                    pad_y: 0.0,
                    background: None,
                    child: Box::new(Node::Quad { w: sw, h: sh, color: clear, radius: 0.0 }),
                },
            ],
        };
        // Card: 1 px ring (outline α 0.3) under the fill
        // (surface_container_high, concentric) — the shadow is still
        // missing, see the reminder above.
        let fill = Node::Wrap {
            pad_x: mk::spacing::L - 1.0,
            pad_y: mk::spacing::L - 1.0,
            background: Some((
                p.surface_container_high,
                mk::radius::inner(mk::radius::NORMAL, 1.0),
            )),
            child: Box::new(column),
        };
        let node = Node::Wrap {
            pad_x: 1.0,
            pad_y: 1.0,
            background: Some((p.outline.with_alpha(0.3), mk::radius::NORMAL)),
            child: Box::new(fill),
        };
        debug_assert_eq!(matrixkit_theme::scene::placed_count(&node), offset);
        PeekSheet { node, labels, label_slots, roll, image }
    }


    // =====================================================================
    // MARKDOWN in the light (Lager 102) — the agents' script.
    //
    // The owner (from the radio): the answers should look like in the
    // Claude terminal — markup, not raw asterisks. The oracle can do
    // that via `mkw::rich`, but that is iced (rich_text); the Werft
    // really runs in the window house. So here the first SCENE
    // markdown: the same grammar as oracle/schrift.rs (bold, italic,
    // `code` chips, # headings, bullets, quotes, [[farbe:…]]/[[laut:…]]
    // expression), but as Placed+labels with MEASURED mixed wrapping —
    // no renderer, only the scene.
    //
    // The engine does not know italic (it lacks a slant axis); italic
    // is set as medium weight with a quiet ink — honestly named
    // instead of secretly left out.
    // =====================================================================

    /// A run of the same markup.
    #[derive(Clone)]
    struct MdRun {
        text: String,
        px: f32,
        weight: mk::typo::Weight,
        color: mk::Rgba,
        /// Code runs speak mono and get a plate.
        code: bool,
    }

    #[derive(Clone, Copy)]
    struct MdPen {
        px: f32,
        weight: mk::typo::Weight,
        color: mk::Rgba,
    }

    fn md_express(tag: &str, pen: &MdPen, p: mk::Palette) -> Option<MdPen> {
        let tint = |r, g, b| mk::Rgba { r, g, b, a: pen.color.a };
        Some(match tag {
            "rot" => MdPen { color: tint(0.85, 0.30, 0.25), ..*pen },
            "gruen" | "grün" => MdPen { color: tint(0.28, 0.63, 0.35), ..*pen },
            "gold" => MdPen { color: tint(0.82, 0.58, 0.10), ..*pen },
            "blau" => MdPen { color: tint(0.30, 0.55, 0.90), ..*pen },
            "violett" => MdPen { color: tint(0.62, 0.44, 0.85), ..*pen },
            "tuerkis" | "türkis" => MdPen { color: tint(0.15, 0.62, 0.68), ..*pen },
            "laut" => MdPen { weight: mk::typo::Weight::Semibold, px: pen.px + 3.0, ..*pen },
            "leise" => MdPen {
                px: (pen.px - 2.0).max(9.0),
                color: mk::Rgba { a: pen.color.a * 0.6, ..pen.color },
                ..*pen
            },
            "puls" => MdPen { color: if pen.color == p.on_surface { p.primary } else { pen.color }, ..*pen },
            _ => return None,
        })
    }

    /// The inline parser — literally the rules of oracle/schrift.rs:
    /// **bold**, *italic*, `code`, [[expression:…]], freely nested;
    /// anything unclosed stays literally.
    fn md_inline(runs: &mut Vec<MdRun>, s: &str, pen: MdPen, p: mk::Palette) {
        let mut rest = s;
        let mut plain = String::new();
        let flush = |runs: &mut Vec<MdRun>, plain: &mut String, pen: &MdPen| {
            if !plain.is_empty() {
                runs.push(MdRun {
                    text: std::mem::take(plain),
                    px: pen.px,
                    weight: pen.weight,
                    color: pen.color,
                    code: false,
                });
            }
        };
        while !rest.is_empty() {
            if let Some(r) = rest.strip_prefix("**") {
                if let Some(end) = r.find("**") {
                    flush(runs, &mut plain, &pen);
                    md_inline(runs, &r[..end], MdPen { weight: mk::typo::Weight::Semibold, ..pen }, p);
                    rest = &r[end + 2..];
                    continue;
                }
            }
            if let Some(r) = rest.strip_prefix("[[") {
                if let (Some(colon), Some(end)) = (r.find(':'), r.find("]]")) {
                    if colon < end {
                        if let Some(pen2) = md_express(&r[..colon].trim().to_lowercase(), &pen, p) {
                            flush(runs, &mut plain, &pen);
                            md_inline(runs, &r[colon + 1..end], pen2, p);
                            rest = &r[end + 2..];
                            continue;
                        }
                    }
                }
            }
            if let Some(r) = rest.strip_prefix('`') {
                if let Some(end) = r.find('`') {
                    flush(runs, &mut plain, &pen);
                    runs.push(MdRun {
                        text: r[..end].to_string(),
                        px: pen.px - 1.0,
                        weight: mk::typo::Weight::Normal,
                        color: p.primary,
                        code: true,
                    });
                    rest = &r[end + 1..];
                    continue;
                }
            }
            if let Some(r) = rest.strip_prefix('*') {
                if let Some(end) = r.find('*') {
                    if end > 0 {
                        flush(runs, &mut plain, &pen);
                        // Italic substitute: medium + quieter ink.
                        md_inline(
                            runs,
                            &r[..end],
                            MdPen {
                                weight: mk::typo::Weight::Medium,
                                color: mk::Rgba { a: pen.color.a * 0.85, ..pen.color },
                                ..pen
                            },
                            p,
                        );
                        rest = &r[end + 1..];
                        continue;
                    }
                }
            }
            let ch = rest.chars().next().unwrap();
            plain.push(ch);
            rest = &rest[ch.len_utf8()..];
        }
        flush(runs, &mut plain, &pen);
    }

    /// The finished markdown build: a column of line rows; every word a
    /// measure quad (code words as plate wraps), the labels know their
    /// SLOT themselves (the lesson from Lager 98).
    pub struct MarkdownBuild {
        pub node: Node,
        pub labels: Vec<(Label, usize)>,
        /// The number of Placed — the caller checks against its lay_out.
        pub placed: usize,
    }

    /// Markdown as a scene: `text` with \n lines, wrapped at `width`,
    /// base font `base_px`/`ink`.
    pub fn markdown(
        engine: &mut TextEngine,
        text: &str,
        base_px: f32,
        ink: mk::Rgba,
        width: f32,
        p: mk::Palette,
    ) -> MarkdownBuild {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mono = crate::mono_family();
        let mut rows: Vec<Node> = Vec::new();
        let mut labels: Vec<(Label, usize)> = Vec::new();
        let mut placed = 0usize;
        let mut gap = true;

        for raw in text.lines() {
            let t = raw.trim_end();
            let tt = t.trim_start();
            if tt.is_empty() {
                if !gap {
                    rows.push(Node::Quad { w: 0.0, h: mk::spacing::XS, color: clear, radius: 0.0 });
                    placed += 1;
                    gap = true;
                }
                continue;
            }
            gap = false;
            let mut pen = MdPen { px: base_px, weight: mk::typo::Weight::Normal, color: ink };
            let mut lead: Option<MdRun> = None;
            let mut indent = 0.0;
            let content = if let Some(h) = tt.strip_prefix("### ") {
                pen.weight = mk::typo::Weight::Semibold;
                h
            } else if let Some(h) = tt.strip_prefix("## ") {
                pen.weight = mk::typo::Weight::Semibold;
                pen.px = base_px + 2.0;
                h
            } else if let Some(h) = tt.strip_prefix("# ") {
                pen.weight = mk::typo::Weight::Semibold;
                pen.px = base_px + 4.0;
                h
            } else if let Some(item) = tt.strip_prefix("- ").or_else(|| tt.strip_prefix("* ")) {
                lead = Some(MdRun {
                    text: String::from("•"),
                    px: base_px,
                    weight: mk::typo::Weight::Normal,
                    color: p.primary,
                    code: false,
                });
                indent = 16.0;
                item
            } else if let Some(q) = tt.strip_prefix("> ") {
                pen.weight = mk::typo::Weight::Medium;
                pen.color = mk::Rgba { a: ink.a * 0.75, ..ink };
                q
            } else {
                tt
            };

            let mut runs: Vec<MdRun> = Vec::new();
            md_inline(&mut runs, content, pen, p);

            // Wrap WORD BY WORD, keeping runs: every word keeps its
            // run's markup; it is measured with the family that
            // carries it.
            let mut words: Vec<MdRun> = Vec::new();
            for r in &runs {
                if r.code {
                    words.push(r.clone());
                    continue;
                }
                for w in r.text.split_inclusive(' ') {
                    words.push(MdRun { text: w.to_string(), ..r.clone() });
                }
            }

            let mut line: Vec<MdRun> = Vec::new();
            let mut line_w = indent;
            let measure = |engine: &mut TextEngine, r: &MdRun| -> f32 {
                if r.code {
                    engine.measure_family(mono, &r.text, r.px, r.weight).0 + 8.0
                } else {
                    engine.measure_styled(&r.text, r.px, r.weight).0
                }
            };
            let mut flush_line =
                |line: &mut Vec<MdRun>, lead: &mut Option<MdRun>, engine: &mut TextEngine| {
                    if line.is_empty() && lead.is_none() {
                        return;
                    }
                    let mut kids: Vec<Node> = Vec::new();
                    let mut slot_in_row = 0usize;
                    if indent > 0.0 {
                        if let Some(b) = lead.take() {
                            let (bw, bh) = engine.measure_styled(&b.text, b.px, b.weight);
                            labels.push((
                                Label { text: b.text, px: b.px, weight: b.weight, family: matrixkit_text::FAMILY, color: b.color },
                                placed + slot_in_row,
                            ));
                            kids.push(Node::Quad { w: bw, h: bh, color: clear, radius: 0.0 });
                            slot_in_row += 1;
                            kids.push(Node::Quad { w: (indent - bw).max(0.0), h: 0.0, color: clear, radius: 0.0 });
                            slot_in_row += 1;
                        } else {
                            kids.push(Node::Quad { w: indent, h: 0.0, color: clear, radius: 0.0 });
                            slot_in_row += 1;
                        }
                    }
                    for r in line.drain(..) {
                        if r.code {
                            let (cw, chh) = engine.measure_family(mono, &r.text, r.px, r.weight);
                            // The chip wrap lays [plate, measure] — the
                            // label anchors on the MEASURE (+1).
                            labels.push((
                                Label { text: r.text, px: r.px, weight: r.weight, family: mono, color: r.color },
                                placed + slot_in_row + 1,
                            ));
                            kids.push(Node::Wrap {
                                pad_x: 4.0,
                                pad_y: 1.0,
                                background: Some((mk::Rgba { a: 0.08, ..ink }, 4.0)),
                                child: Box::new(Node::Quad { w: cw, h: chh, color: clear, radius: 0.0 }),
                            });
                            slot_in_row += 2;
                        } else {
                            let (ww, wh) = engine.measure_styled(&r.text, r.px, r.weight);
                            labels.push((
                                Label { text: r.text, px: r.px, weight: r.weight, family: matrixkit_text::FAMILY, color: r.color },
                                placed + slot_in_row,
                            ));
                            kids.push(Node::Quad { w: ww, h: wh, color: clear, radius: 0.0 });
                            slot_in_row += 1;
                        }
                    }
                    rows.push(Node::Row { gap: 0.0, padding: 0.0, background: None, children: kids });
                    placed += slot_in_row;
                };
            for w in words {
                let ww = measure(engine, &w);
                if line_w + ww > width && !line.is_empty() {
                    flush_line(&mut line, &mut lead, engine);
                    line_w = indent;
                }
                line_w += ww;
                line.push(w);
            }
            flush_line(&mut line, &mut lead, engine);
        }

        MarkdownBuild {
            node: Node::Column { gap: 3.0, padding: 0.0, background: None, children: rows },
            labels,
            placed,
        }
    }

    // =====================================================================
    // The ROOT LAYER in the light (Lager 93) — the app's dashboard.
    //
    // The owner: «in the harness … the root layer … is missing.» In the
    // iced path the window bar's title opens `root_view` (~300 lines):
    // about card, BINDING permission list behind the password bolt,
    // help and done button, veil, spring entrance. Here the same
    // grammar as a scene.
    //
    // [`RootPane`] holds the state and the actions (the client only
    // passes through); [`RootPane::build`] lays the layer OVER the
    // finished scene — anchor is depth (Lager 70), so simply append
    // last. The build's role codes are SMALL (0 = veil …); the client
    // raises them with its own base.
    // =====================================================================

    /// The app's about details — name, version, description.
    /// (The living app icon of the iced version is an open item: the
    /// light path paints images via the atlas, not via an iced handle.)
    pub struct RootAbout<'a> {
        pub name: &'a str,
        pub version: &'a str,
        pub description: &'a str,
    }

    /// What a press in the layer has caused.
    #[derive(PartialEq, Eq, Debug)]
    pub enum RootDeed {
        /// Repaint (switch flipped, field focused …).
        Changed,
        /// The layer closes (veil or "Fertig").
        Closed,
        /// Help should open — the client DOES that (it knows its name
        /// and may start processes, the layer may not).
        Help,
        /// Nothing (press on the card itself — it swallows it so it
        /// does not fall through the veil and close it).
        Calm,
    }

    /// The state of the root layer — literally the grammar of
    /// `mkw::RootState`, plus what iced hides: the thread for the
    /// password check (it runs via `sudo -S -v` and must never stop
    /// the house's loop).
    pub struct RootPane {
        relevant: Vec<mk::permissions::Permission>,
        permissions: mk::permissions::PermissionSet,
        target: bool,
        spring: mk::motion::Spring,
        reduced: bool,
        unlocked: bool,
        password: String,
        cursor: usize,
        anchor: Option<usize>,
        failed: bool,
        /// Is a check running right now? The thread drops its result
        /// here; `poll()` collects it on the loop.
        checking: Option<std::sync::Arc<std::sync::Mutex<Option<bool>>>>,
        wobble: mk::motion::Spring,
        /// Blink phase of the field caret (the house's 500 ms beat).
        blink: bool,
    }

    impl RootPane {
        pub fn new(app_id: &str, relevant: &[mk::permissions::Permission]) -> Self {
            RootPane {
                relevant: relevant.to_vec(),
                permissions: mk::permissions::PermissionSet::load(app_id),
                target: false,
                spring: mk::motion::Spring::snappy(0.0),
                reduced: mk::motion_reduced(),
                unlocked: false,
                password: String::new(),
                cursor: 0,
                anchor: None,
                failed: false,
                checking: None,
                wobble: mk::motion::Spring::new(0.0),
                blink: true,
            }
        }

        /// Open/close via the title — like the reference: LOCKED at
        /// every change (the reference's lock model).
        pub fn toggle(&mut self) {
            self.target = !self.target;
            self.unlocked = false;
            self.password.clear();
            self.cursor = 0;
            self.anchor = None;
            self.failed = false;
            self.checking = None;
            let z = if self.target { 1.0 } else { 0.0 };
            // CPU painting (Lager 104f): the entrance spring cost ~45 %
            // of the window area per round at ~190 ms (~5 frames/s) —
            // there the layer enters in ONE frame. On GPU PAINTING
            // (stage 5) the full frame costs < 5 ms: the spring is back.
            if self.reduced || std::env::var_os("MATRIX_GPU").is_none() {
                self.spring = mk::motion::Spring::snappy(z);
            } else {
                self.spring.retarget(z);
            }
        }

        pub fn is_open(&self) -> bool {
            self.target
        }

        /// Visible also on EXIT — the spring runs out.
        pub fn visible(&self) -> bool {
            self.target || self.spring.value() > 0.01
        }

        pub fn animating(&self) -> bool {
            !self.spring.is_settled() || !self.wobble.is_settled() || self.checking.is_some()
        }

        /// The frame beat. Answer: did something move?
        pub fn glide(&mut self, dt: f32) -> bool {
            let a = self.spring.tick(dt);
            let b = self.wobble.tick(dt);
            a || b || self.poll()
        }

        /// The 500 ms beat: blink the caret + collect the check result.
        pub fn beat(&mut self) -> bool {
            self.blink = !self.blink;
            let field = self.target && !self.unlocked && !self.relevant.is_empty();
            self.poll() || field
        }

        /// Does the layer want TEXT right now? (The password field.)
        pub fn wants_text(&self) -> bool {
            self.target && !self.unlocked && !self.relevant.is_empty()
        }

        /// Collect the result of the check thread.
        fn poll(&mut self) -> bool {
            let Some(cell) = &self.checking else { return false };
            let got = cell.lock().ok().and_then(|mut g| g.take());
            let Some(ok) = got else { return false };
            self.checking = None;
            self.unlocked = ok;
            self.failed = !ok;
            if ok {
                self.password.clear();
                self.cursor = 0;
                self.anchor = None;
            } else if !self.reduced {
                // The lock WOBBLES: stiff spring, little damping —
                // literally the reference's numbers (mkw::RootState).
                self.wobble = mk::motion::Spring {
                    value: 1.0,
                    velocity: 0.0,
                    target: 0.0,
                    stiffness: 900.0,
                    damping: 7.0,
                };
            }
            true
        }

        /// Enter in the field: trigger the check — on a THREAD.
        /// `password_verify` talks to `sudo` and can take hundreds of
        /// milliseconds; the house's loop does not wait for it.
        fn submit(&mut self) {
            if self.password.is_empty() || self.checking.is_some() {
                return;
            }
            let cell = std::sync::Arc::new(std::sync::Mutex::new(None));
            let out = cell.clone();
            let pw = self.password.clone();
            std::thread::spawn(move || {
                let ok = mk::permissions::password_verify(&pw);
                if let Ok(mut g) = out.lock() {
                    *g = Some(ok);
                }
            });
            self.checking = Some(cell);
        }

        /// A key while the layer is open. Answer: WAS it consumed?
        /// (While the layer is open the keyboard belongs to the layer —
        /// as in iced, where the root layer intercepts it.)
        pub fn key(&mut self, key: &matrixkit_shell::Key, shift: bool) -> bool {
            use matrixkit_shell::Key;
            if !self.target {
                return false;
            }
            if matches!(key, Key::Escape) {
                self.toggle();
                return true;
            }
            if self.unlocked || self.relevant.is_empty() {
                if matches!(key, Key::Enter) {
                    self.toggle();
                }
                return true;
            }
            match field_edit(
                &mut self.password,
                &mut self.cursor,
                &mut self.anchor,
                key,
                shift,
            ) {
                FieldDeed::Submitted => self.submit(),
                FieldDeed::Changed => self.failed = false,
                FieldDeed::Escaped | FieldDeed::Calm => {}
            }
            true
        }

        /// A press on one of the build's roles (the SMALL code, without
        /// the client's base).
        pub fn strike(&mut self, code: usize) -> RootDeed {
            match code {
                ROOT_VEIL | ROOT_DONE => {
                    self.toggle();
                    RootDeed::Closed
                }
                ROOT_HELP => RootDeed::Help,
                t if t >= ROOT_TOGGLE && t < ROOT_TOGGLE + self.relevant.len() => {
                    if self.unlocked {
                        let r = self.relevant[t - ROOT_TOGGLE];
                        let allowed = self.permissions.allowed(r);
                        self.permissions.set(r, !allowed);
                        RootDeed::Changed
                    } else {
                        RootDeed::Calm
                    }
                }
                _ => RootDeed::Calm,
            }
        }
    }

    /// The role codes of the root build.
    pub const ROOT_VEIL: usize = 0;
    pub const ROOT_CARD: usize = 1;
    pub const ROOT_FIELD: usize = 2;
    pub const ROOT_HELP: usize = 3;
    pub const ROOT_DONE: usize = 4;
    pub const ROOT_TOGGLE: usize = 10;

    /// The finished build: Placed at ABSOLUTE coordinates (the layer
    /// lies over the whole window), small role codes, and the labels
    /// per slot (the client raises anchors and roles).
    pub struct RootBuild {
        pub placed: Vec<Placed>,
        pub roles: Vec<Option<usize>>,
        pub labels: Vec<(Label, usize)>,
    }

    impl RootPane {
        /// LAY THE ROOT PANE (R210 family): build it over everything in
        /// `b`, its roles shifted to `role_base` — what Player and Tube
        /// spelled out by hand. Nothing while it is hidden.
        #[allow(clippy::too_many_arguments)]
        pub fn lay(&mut self, engine: &mut TextEngine, b: &mut matrixkit_shell::SceneBuilder, about: &RootAbout, w: f32, h: f32, p: mk::Palette, role_base: usize) {
            if !self.visible() {
                return;
            }
            let build = self.build(engine, about, w, h, p);
            let first = b.len();
            for (q, r) in build.placed.into_iter().zip(build.roles) {
                b.scene.placed.push(q);
                b.scene.roles.push(r.map(|k| role_base + k));
            }
            for (l, slot) in build.labels {
                let q = b.scene.placed[first + slot];
                b.text_on(
                    first + slot,
                    matrixkit_shell::Text { s: l.text, px: l.px, weight: l.weight, family: l.family, color: l.color, x: q.x, y: q.y, home: Rect::of(&q), anchor: 0, clip: None },
                );
            }
        }

        /// While open, every press belongs to the pane: its own roles act,
        /// everything else is swallowed. None = closed, the app goes on.
        pub fn strike_at(&mut self, role: Option<usize>, role_base: usize) -> Option<RootDeed> {
            if !self.is_open() {
                return None;
            }
            Some(match role {
                Some(r) if r >= role_base => self.strike(r - role_base),
                _ => RootDeed::Calm,
            })
        }

        /// Open it (the top bar's «Einstellungen»); true if it was closed.
        pub fn open(&mut self) -> bool {
            if self.is_open() {
                return false;
            }
            self.toggle();
            true
        }
    }

    /// Line wrapping, greedy and MEASURED — no estimated character
    /// measure. For the description of the about card.
    /// Public as `wrap_text`: measured line wrapping for clients (the
    /// engine paints single-line; iced's text() wrapped by itself).
    pub fn wrap_text(
        engine: &mut TextEngine,
        text: &str,
        px: f32,
        weight: mk::typo::Weight,
        width: f32,
    ) -> Vec<String> {
        wrap_lines(engine, text, px, weight, width)
    }

    fn wrap_lines(
        engine: &mut TextEngine,
        text: &str,
        px: f32,
        weight: mk::typo::Weight,
        width: f32,
    ) -> Vec<String> {
        // Wrap what the screen shows (the language), not the German key.
        let text = mk::language::shown(text);
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in text.split_whitespace() {
            let probe = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if engine.measure_styled(&probe, px, weight).0 <= width {
                line = probe;
                continue;
            }
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            // R207 (Lager 114): a word that alone is wider than the line
            // (addresses, hashes) is broken at characters — no text ever
            // runs over the edge.
            if engine.measure_styled(word, px, weight).0 <= width {
                line = word.to_string();
            } else {
                for piece in hard_break(engine, word, px, weight, width) {
                    if !line.is_empty() {
                        lines.push(std::mem::take(&mut line));
                    }
                    line = piece;
                }
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines
    }

    /// A word into pieces that each fit in `width` (at least one
    /// character per piece, so it always makes progress).
    fn hard_break(engine: &mut TextEngine, word: &str, px: f32, weight: mk::typo::Weight, width: f32) -> Vec<String> {
        let chars: Vec<char> = word.chars().collect();
        let mut out = Vec::new();
        let mut piece = String::new();
        for c in chars {
            let mut probe = piece.clone();
            probe.push(c);
            if piece.is_empty() || engine.measure_styled(&probe, px, weight).0 <= width {
                piece = probe;
            } else {
                out.push(std::mem::take(&mut piece));
                piece.push(c);
            }
        }
        if !piece.is_empty() {
            out.push(piece);
        }
        out
    }

    /// The card's collecting build: row nodes, counted quads (the
    /// count IS the Placed order — depth-first), labels and roles per
    /// slot.
    struct RootRows {
        rows: Vec<Node>,
        quads: usize,
        labels: Vec<(Label, usize)>,
        roles: Vec<(usize, usize)>,
        clear: mk::Rgba,
    }

    impl RootRows {
        fn gap(&mut self, hgt: f32) {
            self.rows.push(Node::Quad { w: 0.0, h: hgt, color: self.clear, radius: 0.0 });
            self.quads += 1;
        }

        fn text(
            &mut self,
            engine: &mut TextEngine,
            s: &str,
            px: f32,
            family: &'static str,
            color: mk::Rgba,
        ) {
            let weight = mk::typo::Weight::Normal;
            let (tw, th) = if family == matrixkit_text::FAMILY {
                engine.measure_styled(s, px, weight)
            } else {
                engine.measure_family(family, s, px, weight)
            };
            let slot = self.quads;
            self.rows.push(Node::Quad { w: tw, h: th, color: self.clear, radius: 0.0 });
            self.quads += 1;
            self.labels.push((Label { text: s.to_string(), px, weight, family, color }, slot));
        }
    }

    impl RootPane {
        /// Build the layer — over a window of w×h logical pixels.
        /// Goes LAST into the scene: anchor is depth.
        pub fn build(
            &mut self,
            engine: &mut TextEngine,
            about: &RootAbout,
            w: f32,
            h: f32,
            p: mk::Palette,
        ) -> RootBuild {
            let f = self.spring.value().clamp(0.0, 1.2);
            let clear = mk::Rgba { a: 0.0, ..p.surface };
            let symbols = crate::symbol_family();

            // Card size: grows with the window, capped at ROOT_WIDTH
            // (R178) — air XL to the edges.
            let card_w = (w - 2.0 * mk::spacing::XL).min(crate::ROOT_WIDTH);
            let inner = card_w - 2.0 * mk::spacing::XXL;

            let mut b = RootRows {
                rows: Vec::new(),
                quads: 0,
                labels: Vec::new(),
                roles: Vec::new(),
                clear,
            };

            // --- About card -------------------------------------------
            b.text(engine, about.name, 20.0, matrixkit_text::FAMILY, p.on_surface);
            b.gap(mk::spacing::XXS);
            b.text(engine, &format!("Version {} · MatrixKit-App", about.version), mk::font_size::SMALL, matrixkit_text::FAMILY, p.on_surface_variant);
            b.text(engine, "Matrix Hardline · matrixhardline@ik.me", mk::font_size::SMALL, matrixkit_text::FAMILY, p.on_surface_variant);
            b.gap(mk::spacing::S);
            for line in wrap_lines(engine, about.description, mk::font_size::MEDIUM, mk::typo::Weight::Normal, inner) {
                b.text(engine, &line, mk::font_size::MEDIUM, matrixkit_text::FAMILY, p.on_surface_variant);
            }
            b.gap(mk::spacing::L);
            // Separator — the family's hairline.
            b.rows.push(Node::Quad { w: inner, h: 1.0, color: p.outline.with_alpha(0.25), radius: 0.0 });
            b.quads += 1;
            b.gap(mk::spacing::M);

            // --- Berechtigungen --------------------------------------
            {
                let glyph = crate::symbol::SHIELD.to_string();
                let (gw, gh) = engine.measure_family(symbols, &glyph, mk::font_size::MEDIUM, mk::typo::Weight::Normal);
                let (tw, th) = engine.measure_styled("BERECHTIGUNGEN", mk::font_size::SMALL, mk::typo::Weight::Normal);
                let g_slot = b.quads;
                let t_slot = b.quads + 1;
                b.rows.push(Node::Row {
                    gap: mk::spacing::XS,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 },
                        Node::Quad { w: tw, h: th, color: clear, radius: 0.0 },
                    ],
                });
                b.quads += 2;
                b.labels.push((Label { text: glyph, px: mk::font_size::MEDIUM, weight: mk::typo::Weight::Normal, family: symbols, color: p.on_surface_variant }, g_slot));
                b.labels.push((Label { text: String::from("BERECHTIGUNGEN"), px: mk::font_size::SMALL, weight: mk::typo::Weight::Normal, family: matrixkit_text::FAMILY, color: p.on_surface_variant }, t_slot));
            }
            b.gap(mk::spacing::S);

            if self.relevant.is_empty() {
                for line in wrap_lines(engine, "Diese App benötigt keine PermissionSet.", mk::font_size::MEDIUM, mk::typo::Weight::Normal, inner) {
                    b.text(engine, &line, mk::font_size::MEDIUM, matrixkit_text::FAMILY, p.on_surface_variant);
                }
            }
            let relevant = self.relevant.clone();
            for (i, r) in relevant.iter().enumerate() {
                let allowed = self.permissions.allowed(*r);
                let name = r.display_name();
                let ink = if self.unlocked { p.on_surface } else { p.on_surface_variant };
                let (tw, th) = engine.measure_styled(name, mk::font_size::MEDIUM, mk::typo::Weight::Normal);
                let t_slot = b.quads;
                // Text measure + spacer, then the switch's TRACK.
                let toggle_first = b.quads + 2;
                let slack = (inner - tw - mk::control::TOGGLE_TRACK_W).max(0.0);
                b.rows.push(Node::Row {
                    gap: 0.0,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Quad { w: tw, h: th, color: clear, radius: 0.0 },
                        Node::Quad { w: slack, h: 0.0, color: clear, radius: 0.0 },
                        toggle(allowed, !self.unlocked, p),
                    ],
                });
                b.quads += 2 + PLACED_PER_TOGGLE;
                b.labels.push((Label { text: name.to_string(), px: mk::font_size::MEDIUM, weight: mk::typo::Weight::Normal, family: matrixkit_text::FAMILY, color: ink }, t_slot));
                b.roles.push((toggle_first, ROOT_TOGGLE + i));
                b.gap(mk::spacing::XS);
            }

            // --- Der Riegel ------------------------------------------
            if !relevant.is_empty() {
                b.gap(mk::spacing::S);
                if self.unlocked {
                    let glyph = crate::symbol::LOCK_OPEN.to_string();
                    let (gw, gh) = engine.measure_family(symbols, &glyph, mk::font_size::MEDIUM, mk::typo::Weight::Normal);
                    let note = "Entsperrt — Änderungen möglich";
                    let (tw, th) = engine.measure_styled(note, mk::font_size::SMALL, mk::typo::Weight::Normal);
                    let g_slot = b.quads;
                    let t_slot = b.quads + 1;
                    b.rows.push(Node::Row {
                        gap: mk::spacing::XS,
                        padding: 0.0,
                        background: None,
                        children: vec![
                            Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 },
                            Node::Quad { w: tw, h: th, color: clear, radius: 0.0 },
                        ],
                    });
                    b.quads += 2;
                    b.labels.push((Label { text: glyph, px: mk::font_size::MEDIUM, weight: mk::typo::Weight::Normal, family: symbols, color: p.primary }, g_slot));
                    b.labels.push((Label { text: note.to_string(), px: mk::font_size::SMALL, weight: mk::typo::Weight::Normal, family: matrixkit_text::FAMILY, color: p.on_surface_variant }, t_slot));
                } else {
                    // Lock (with wobble deflection) + password field.
                    let deflection = (self.wobble.value() * 6.0).clamp(-6.0, 6.0);
                    let glyph = crate::symbol::LOCK.to_string();
                    let (gw, gh) = engine.measure_family(symbols, &glyph, mk::font_size::MEDIUM, mk::typo::Weight::Normal);
                    // Show the password MASKED: the field knows no
                    // secure mode — so it gets dots. One dot per
                    // character keeps the caret arithmetic matching.
                    let masked: String = "•".repeat(self.password.chars().count());
                    let spec = FieldSpec {
                        value: &masked,
                        cursor: self.cursor.min(masked.chars().count()),
                        anchor: self.anchor,
                        focused: true,
                        blink: self.blink,
                        placeholder: if self.checking.is_some() { "Prüfe …" } else { "Passwort zum Ändern" },
                        leading: None,
                    };
                    let field_w = inner - gw - 12.0 - mk::spacing::XS;
                    let field = input_field(engine, &spec, field_w, p);
                    let g_slot = b.quads;
                    // The wrap around the lock lays NO ground
                    // (background: None) — first the glyph, then the
                    // field.
                    let field_first = b.quads + 1;
                    b.rows.push(Node::Row {
                        gap: mk::spacing::XS,
                        padding: 0.0,
                        background: None,
                        children: vec![
                            Node::Wrap {
                                pad_x: (6.0 + deflection).max(0.0),
                                pad_y: 0.0,
                                background: None,
                                child: Box::new(Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 }),
                            },
                            field.node,
                        ],
                    });
                    b.quads += 1 + placed_per_field(&spec);
                    b.labels.push((Label { text: glyph, px: mk::font_size::MEDIUM, weight: mk::typo::Weight::Normal, family: symbols, color: p.on_surface_variant }, g_slot));
                    // Field convention (PLACED_PER_FIELD): the label
                    // anchors on the GROUND (+1).
                    b.labels.push((field.label, field_first + 1));
                    b.roles.push((field_first + 1, ROOT_FIELD));
                    if self.failed {
                        b.gap(mk::spacing::XXS);
                        b.text(engine, "Passwort falsch", mk::font_size::SMALL, matrixkit_text::FAMILY, p.error);
                    }
                }
                b.gap(mk::spacing::S);
                // The note wraps MEASURED too — on a narrow window it
                // otherwise ran out of the card (finger probe Matrix
                // Farben, 380 px wide).
                for line in wrap_lines(engine, "Bindend: Ohne Berechtigung erhält die App keinen Zugriff.", mk::font_size::SMALL, mk::typo::Weight::Normal, inner) {
                    b.text(engine, &line, mk::font_size::SMALL, matrixkit_text::FAMILY, p.on_surface_variant);
                }
            }

            // --- Hilfe + Fertig --------------------------------------
            b.gap(mk::spacing::L);
            {
                let glyph = crate::symbol::HELP.to_string();
                let (gw, gh) = engine.measure_family(symbols, &glyph, mk::font_size::MEDIUM, mk::typo::Weight::Normal);
                let (tw, th) = engine.measure_styled("Fertig", mk::font_size::MEDIUM, mk::typo::Weight::Normal);
                // A slot WITH a ground lays: ground, then child.
                let help_first = b.quads;
                let done_first = b.quads + 2;
                let done_w = (inner - 40.0 - mk::spacing::S).max(80.0);
                b.rows.push(Node::Row {
                    gap: mk::spacing::S,
                    padding: 0.0,
                    background: None,
                    children: vec![
                        Node::Slot {
                            w: Some(40.0),
                            h: Some(40.0),
                            ax: Align::Center,
                            ay: Align::Center,
                            background: Some((p.surface_container_high, mk::radius::capsule(40.0))),
                            child: Box::new(Node::Quad { w: gw, h: gh, color: clear, radius: 0.0 }),
                        },
                        Node::Slot {
                            w: Some(done_w),
                            h: Some(40.0),
                            ax: Align::Center,
                            ay: Align::Center,
                            background: Some((p.primary, mk::CORNER_RADIUS)),
                            child: Box::new(Node::Quad { w: tw, h: th, color: clear, radius: 0.0 }),
                        },
                    ],
                });
                b.quads += 4;
                b.labels.push((Label { text: glyph, px: mk::font_size::MEDIUM, weight: mk::typo::Weight::Normal, family: symbols, color: p.on_surface_variant }, help_first + 1));
                b.labels.push((Label { text: String::from("Fertig"), px: mk::font_size::MEDIUM, weight: mk::typo::Weight::Normal, family: matrixkit_text::FAMILY, color: p.on_primary }, done_first + 1));
                b.roles.push((help_first, ROOT_HELP));
                b.roles.push((done_first, ROOT_DONE));
            }

            // --- Lay the card -----------------------------------------
            let column = Node::Column { gap: 0.0, padding: 0.0, background: None, children: b.rows };
            let card = Node::Wrap {
                pad_x: mk::spacing::XXL,
                pad_y: mk::spacing::XXL,
                background: Some((p.surface_container_high, mk::CORNER_RADIUS)),
                child: Box::new(Node::Slot {
                    w: Some(inner),
                    h: None,
                    ax: Align::Start,
                    ay: Align::Start,
                    background: None,
                    child: Box::new(column),
                }),
            };
            // First measure at the origin, then lay at the POSITION —
            // with the spring entrance: 80 px from below (emphasized
            // decelerate; the overshoot gives the last millimetres
            // life).
            let mut probe = Vec::new();
            let (cw, ch) = matrixkit_theme::scene::lay_out(&card, 0.0, 0.0, &mut probe);
            let cx = ((w - cw) / 2.0).max(0.0);
            let cy = (((h - ch) / 2.0).max(mk::spacing::XL)) + 80.0 * (1.0 - f.min(1.0));

            let mut placed: Vec<Placed> = Vec::new();
            let mut roles: Vec<Option<usize>> = Vec::new();
            // The veil first — it lies UNDER the card and fades in with
            // the spring (a press on it closes).
            placed.push(Placed {
                x: 0.0,
                y: 0.0,
                w,
                h,
                // One-step veil on the CPU (Lager 104: a fade there is a
                // full frame per frame); on GPU painting it blends again
                // (stage 5).
                color: mk::Rgba {
                    r: 0.08,
                    g: 0.08,
                    b: 0.08,
                    a: if std::env::var_os("MATRIX_GPU").is_some() {
                        0.55 * f.min(1.0)
                    } else if f > 0.0 {
                        0.55
                    } else {
                        0.0
                    },
                },
                radius: 0.0,
                clip: None,
                shadow: None,
            });
            roles.push(Some(ROOT_VEIL));

            let mut card_placed = Vec::new();
            matrixkit_theme::scene::lay_out(&card, cx, cy, &mut card_placed);
            let mut card_roles: Vec<Option<usize>> = vec![None; card_placed.len()];
            // Slot 0 is the wrap ground: it SWALLOWS presses (otherwise
            // they would fall through to the app or the veil) and floats.
            card_roles[0] = Some(ROOT_CARD);
            card_placed[0].shadow = Some(floating_shadow());
            // The wrap lays 1 ground quad BEFORE the column; the slot in
            // it lays none. So all counted slots shift by exactly 1.
            let base = 1;
            let mut labels: Vec<(Label, usize)> = Vec::new();
            // The role falls ACROSS THE AREA onto everything inside the
            // target's box. `hit()` takes the TOPMOST Placed — and above
            // the button ground lies its clear text measure: a click
            // right on "Fertig" otherwise hit the measure (without a
            // role) instead of the button (finger probe 19.08.2026 — the
            // layer simply stayed open).
            for (slot, code) in b.roles {
                card_roles[base + slot] = Some(code);
                let r = card_placed[base + slot];
                for j in 0..card_placed.len() {
                    if card_roles[j].is_some() {
                        continue;
                    }
                    let q = card_placed[j];
                    if q.x >= r.x - 0.5
                        && q.y >= r.y - 0.5
                        && q.x + q.w <= r.x + r.w + 0.5
                        && q.y + q.h <= r.y + r.h + 0.5
                    {
                        card_roles[j] = Some(code);
                    }
                }
            }
            for (label, slot) in b.labels {
                labels.push((label, placed.len() + base + slot));
            }
            placed.extend(card_placed);
            roles.extend(card_roles);

            RootBuild { placed, roles, labels }
        }
    }


    // ------------------------------------------------------------
    // The remainder list of 09.09.2026 — surveyed on reference app (iced),
    // Werft and Studio (light): what every app built for itself or
    // what it lacked in the light. Text and glyph (Werft: label/glyph,
    // Studio: label/text, Terminal: runs), the tick (mkw::tick), the
    // tinted card (mkw::card_tinted), the stepper (mkw::stepper),
    // expander (mkw::expand), breadcrumbs, page picker, numbered rail
    // and ladder (R202 D.3), segments (mkw::segments) and the table
    // (mkw::table).
    // All slot indices are RELATIVE to the family's first Placed.
    // ------------------------------------------------------------

    /// A label as (Node, Label) — the building block that Werft
    /// (`label`), Studio (`text`) and Terminal each wrote themselves.
    /// 1 Placed (transparent measure).
    pub fn label(
        engine: &mut TextEngine,
        text: &str,
        px: f32,
        weight: mk::typo::Weight,
        color: mk::Rgba,
    ) -> (Node, Label) {
        plain_label(engine, text, px, weight, color)
    }

    /// A glyph of the symbol font as (Node, Label). `px` is the text
    /// size, the glyph is set to fit via symbol_px.
    /// 1 Placed.
    pub fn glyph(engine: &mut TextEngine, g: char, px: f32, color: mk::Rgba) -> (Node, Label) {
        let fam = crate::symbol_family();
        let s = g.to_string();
        let spx = mk::typo::symbol_px(px);
        let (w, h) = engine.measure_family(fam, &s, spx, mk::typo::Weight::Normal);
        (
            Node::Quad { w, h, color: mk::Rgba { a: 0.0, ..color }, radius: 0.0 },
            Label { text: s, px: spx, weight: mk::typo::Weight::Normal, family: fam, color },
        )
    }

    /// The PARAGRAPH: a text that wraps at the column edge — a column
    /// of measured lines. Placed = lines (at least 1).
    pub struct Paragraph {
        pub node: Node,
        pub labels: Vec<Label>,
    }

    pub fn paragraph(
        engine: &mut TextEngine,
        text: &str,
        px: f32,
        weight: mk::typo::Weight,
        color: mk::Rgba,
        width: f32,
    ) -> Paragraph {
        let mut lines = wrap_text(engine, text, px, weight, width);
        if lines.is_empty() {
            lines.push(String::new());
        }
        let mut nodes = Vec::with_capacity(lines.len());
        let mut labels = Vec::with_capacity(lines.len());
        for line in &lines {
            let (n, l) = plain_label(engine, line, px, weight, color);
            nodes.push(n);
            labels.push(l);
        }
        Paragraph {
            node: Node::Column { gap: 2.0, padding: 0.0, background: None, children: nodes },
            labels,
        }
    }

    /// The TRUNCATION (R207): if the text does not fit into `max_w`,
    /// its beginning remains with an ellipsis — never a text that runs
    /// through the window. Binary search over the characters, measured
    /// with the UI font.
    pub fn ellipsize(engine: &mut TextEngine, text: &str, px: f32, weight: mk::typo::Weight, max_w: f32) -> String {
        ellipsize_with(text, max_w, |s| engine.measure_styled(s, px, weight).0)
    }

    /// Like `ellipsize`, measured with another family (mono).
    pub fn ellipsize_family(engine: &mut TextEngine, family: &'static str, text: &str, px: f32, weight: mk::typo::Weight, max_w: f32) -> String {
        ellipsize_with(text, max_w, |s| engine.measure_family(family, s, px, weight).0)
    }

    fn ellipsize_with(text: &str, max_w: f32, mut measure: impl FnMut(&str) -> f32) -> String {
        if measure(text) <= max_w {
            return text.to_string();
        }
        let chars: Vec<char> = text.chars().collect();
        let (mut lo, mut hi) = (0usize, chars.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            let mut probe: String = chars[..mid].iter().collect();
            probe.push('…');
            if measure(&probe) <= max_w {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        let mut out: String = chars[..lo].iter().collect::<String>().trim_end().to_string();
        out.push('…');
        out
    }

    /// The FLOW ROW (R207): children side by side as long as they fit
    /// into `width`, then the next line — chips, buttons, menus that
    /// wrap at phone width instead of hanging out. Rows and columns
    /// without a ground lay no Placed: the children keep their order
    /// and their slots.
    pub fn flow(children: Vec<Node>, gap: f32, width: f32) -> Node {
        let mut rows: Vec<Node> = Vec::new();
        let mut row: Vec<Node> = Vec::new();
        let mut used = 0.0f32;
        for child in children {
            let mut v = Vec::new();
            let (cw, _) = matrixkit_theme::scene::lay_out(&child, 0.0, 0.0, &mut v);
            if !row.is_empty() && used + gap + cw > width {
                rows.push(Node::Row { gap, padding: 0.0, background: None, children: std::mem::take(&mut row) });
                used = 0.0;
            }
            used += if row.is_empty() { cw } else { gap + cw };
            row.push(child);
        }
        if !row.is_empty() {
            rows.push(Node::Row { gap, padding: 0.0, background: None, children: row });
        }
        Node::Column { gap, padding: 0.0, background: None, children: rows }
    }

    /// The TINTED CARD (mkw::card_tinted): the same anatomy as `card`,
    /// border and ground carry the tint — project colours, agent
    /// colours, warning colours. 2 Placed before the child.
    pub fn card_tinted(child: Node, tint: mk::Rgba, p: mk::Palette) -> Node {
        Node::Wrap {
            pad_x: 1.0,
            pad_y: 1.0,
            background: Some((tint.with_alpha(0.35), mk::radius::NORMAL)),
            child: Box::new(Node::Wrap {
                pad_x: mk::spacing::M - 1.0,
                pad_y: mk::spacing::M - 1.0,
                background: Some((
                    tint.over(p.fill(mk::FillLevel::Large), 0.12),
                    mk::radius::inner(mk::radius::NORMAL, 1.0),
                )),
                child: Box::new(child),
            }),
        }
    }

    /// The TICK (mkw::tick): box 18 with a check glyph, the text beside
    /// it. The whole ROW is the target (Placed 0, transparent), as in
    /// the list. Placed: row, frame, fill, check measure, text measure.
    pub const PLACED_PER_CHECK: usize = 5;
    pub const CHECK_BOX: f32 = 18.0;

    pub struct Check {
        pub node: Node,
        pub label: Label,
        pub tick: Label,
        /// The hit target: the row (always 0).
        pub slot: usize,
    }

    pub fn check(engine: &mut TextEngine, text: &str, checked: bool, p: mk::Palette) -> Check {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let (tn, mut tl) = glyph(engine, crate::symbol::CHECK, mk::font_size::SMALL, p.on_primary);
        if !checked {
            tl.color = mk::Rgba { a: 0.0, ..tl.color };
        }
        let (ln, ll) = plain_label(engine, text, mk::font_size::MEDIUM, mk::typo::Weight::Normal, p.on_surface);
        let frame = Node::Wrap {
            pad_x: 1.0,
            pad_y: 1.0,
            background: Some((
                if checked { p.primary } else { p.outline.with_alpha(0.6) },
                mk::radius::SMALL / 2.0,
            )),
            child: Box::new(Node::Slot {
                w: Some(CHECK_BOX - 2.0),
                h: Some(CHECK_BOX - 2.0),
                ax: Align::Center,
                ay: Align::Center,
                background: Some((
                    if checked { p.primary } else { p.surface_container_high },
                    mk::radius::inner(mk::radius::SMALL / 2.0, 1.0),
                )),
                child: Box::new(tn),
            }),
        };
        Check {
            node: Node::Row {
                gap: mk::spacing::S,
                padding: 0.0,
                background: Some((clear, 0.0)),
                children: vec![frame, ln],
            },
            label: ll,
            tick: tl,
            slot: 0,
        }
    }

    /// The STEPPER (mkw::stepper): −, value, + — two round targets
    /// around a value. Placed: − ground, − glyph, value, + ground,
    /// + glyph (labels: minus, value, plus).
    pub const PLACED_PER_STEPPER: usize = 5;

    pub struct Stepper {
        pub node: Node,
        pub minus: Label,
        pub value: Label,
        pub plus: Label,
        pub minus_slot: usize,
        pub plus_slot: usize,
    }

    pub fn stepper(engine: &mut TextEngine, value: &str, p: mk::Palette) -> Stepper {
        let knob = |engine: &mut TextEngine, g: char| {
            let (gn, gl) = glyph(engine, g, mk::font_size::MEDIUM, p.on_surface);
            (
                Node::Wrap {
                    pad_x: mk::spacing::S,
                    pad_y: mk::spacing::XS,
                    background: Some((p.surface_container_high, mk::radius::capsule(28.0))),
                    child: Box::new(gn),
                },
                gl,
            )
        };
        let (mn, ml) = knob(engine, crate::symbol::REMOVE);
        let (vn, vl) = plain_label(engine, value, mk::font_size::MEDIUM, mk::typo::Weight::Semibold, p.on_surface);
        let (pn, pl) = knob(engine, crate::symbol::ADD);
        Stepper {
            node: Node::Row { gap: mk::spacing::S, padding: 0.0, background: None, children: vec![mn, vn, pn] },
            minus: ml,
            value: vl,
            plus: pl,
            minus_slot: 0,
            plus_slot: 3,
        }
    }

    /// The EXPANDER (mkw::expand): header row with arrow and title
    /// across the full width (Placed 0 = target), below it the content
    /// when open. Header = 3 Placed (ground, arrow, title).
    pub const PLACED_PER_DISCLOSURE_HEAD: usize = 3;

    pub struct Disclosure {
        pub node: Node,
        pub title: Label,
        pub chevron: Label,
        pub head_slot: usize,
        /// First Placed of the content, if it was laid.
        pub body_first: Option<usize>,
    }

    pub fn disclosure(
        engine: &mut TextEngine,
        title: &str,
        width: f32,
        open: bool,
        body: Option<Node>,
        p: mk::Palette,
    ) -> Disclosure {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let (cn, cl) = glyph(
            engine,
            if open { crate::symbol::EXPAND_MORE } else { crate::symbol::CHEVRON_RIGHT },
            mk::font_size::MEDIUM,
            p.on_surface_variant,
        );
        let (tn, tl) = plain_label(engine, title, mk::font_size::MEDIUM, mk::typo::Weight::Semibold, p.on_surface);
        let head = Node::Slot {
            w: Some(width),
            h: None,
            ax: Align::Start,
            ay: Align::Center,
            background: Some((clear, 0.0)),
            child: Box::new(Node::Row {
                gap: mk::spacing::S,
                padding: 0.0,
                background: None,
                children: vec![cn, tn],
            }),
        };
        let mut children = vec![head];
        let mut body_first = None;
        if open {
            if let Some(b) = body {
                body_first = Some(PLACED_PER_DISCLOSURE_HEAD);
                children.push(b);
            }
        }
        Disclosure {
            node: Node::Column { gap: mk::spacing::S, padding: 0.0, background: None, children },
            title: tl,
            chevron: cl,
            head_slot: 0,
            body_first,
        }
    }

    /// BREADCRUMBS: links as text buttons, the last one quiet and
    /// fixed, › in between. `slots` names each link's Placed.
    pub struct Breadcrumbs {
        pub node: Node,
        pub labels: Vec<Label>,
        pub slots: Vec<usize>,
    }

    pub fn breadcrumbs(engine: &mut TextEngine, parts: &[&str], p: mk::Palette) -> Breadcrumbs {
        let mut nodes = Vec::new();
        let mut labels = Vec::new();
        let mut slots = Vec::new();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                let (sn, sl) = plain_label(engine, "›", mk::font_size::SMALL, mk::typo::Weight::Normal, p.on_surface_variant);
                nodes.push(sn);
                labels.push(sl);
            }
            let last = i + 1 == parts.len();
            slots.push(nodes.len());
            let (n, l) = plain_label(
                engine,
                part,
                mk::font_size::SMALL,
                mk::typo::Weight::Semibold,
                if last { p.on_surface } else { p.primary },
            );
            nodes.push(n);
            labels.push(l);
        }
        Breadcrumbs {
            node: Node::Row { gap: mk::spacing::S, padding: 0.0, background: None, children: nodes },
            labels,
            slots,
        }
    }

    /// The PAGE PICKER: ‹ 01 02 03 › — one chip per page, the active
    /// one on primary_container; from 13 pages on a counter "03/27"
    /// instead of chips. Arrows and chips are targets (ground).
    pub struct Pager {
        pub node: Node,
        pub labels: Vec<Label>,
        pub prev_slot: usize,
        pub next_slot: usize,
        pub page_slots: Vec<usize>,
    }

    pub const PAGER_CHIP_LIMIT: usize = 12;

    pub fn pager(engine: &mut TextEngine, index: usize, count: usize, p: mk::Palette) -> Pager {
        let chip = |engine: &mut TextEngine, text: &str, active: bool, color: mk::Rgba| {
            let (n, l) = plain_label(engine, text, mk::font_size::SMALL, mk::typo::Weight::Semibold, color);
            (
                Node::Wrap {
                    pad_x: mk::spacing::S,
                    pad_y: mk::spacing::XS,
                    background: Some((
                        if active { p.primary_container } else { p.surface_container },
                        mk::radius::capsule(26.0),
                    )),
                    child: Box::new(n),
                },
                l,
            )
        };
        let arrow = |engine: &mut TextEngine, g: char| {
            let (gn, gl) = glyph(engine, g, mk::font_size::SMALL, p.on_surface_variant);
            (
                Node::Wrap {
                    pad_x: mk::spacing::S,
                    pad_y: mk::spacing::XS,
                    background: Some((p.surface_container, mk::radius::capsule(26.0))),
                    child: Box::new(gn),
                },
                gl,
            )
        };
        let mut nodes = Vec::new();
        let mut labels = Vec::new();
        let mut page_slots = Vec::new();
        let (pn, pl) = arrow(engine, '\u{e5cb}'); // CHEVRON_LEFT
        nodes.push(pn);
        labels.push(pl);
        let prev_slot = 0;
        let mut placed = 2;
        if count <= PAGER_CHIP_LIMIT {
            for i in 0..count {
                page_slots.push(placed);
                let (n, l) = chip(
                    engine,
                    &format!("{:02}", i + 1),
                    i == index,
                    if i == index { p.on_primary_container } else { p.on_surface_variant },
                );
                nodes.push(n);
                labels.push(l);
                placed += 2;
            }
        } else {
            let (n, l) = plain_label(
                engine,
                &format!("{:02}/{:02}", index + 1, count),
                mk::font_size::SMALL,
                mk::typo::Weight::Semibold,
                p.on_surface,
            );
            nodes.push(n);
            labels.push(l);
            placed += 1;
        }
        let next_slot = placed;
        let (nn, nl) = arrow(engine, crate::symbol::CHEVRON_RIGHT);
        nodes.push(nn);
        labels.push(nl);
        Pager {
            node: Node::Row { gap: mk::spacing::XS, padding: 0.0, background: None, children: nodes },
            labels,
            prev_slot,
            next_slot,
            page_slots,
        }
    }

    /// The NUMBERED RAIL (R202: "The SOP, written out"): 01… in mono
    /// font, the wrapped text beside it. Labels: per entry the number,
    /// then its lines.
    pub struct Rail {
        pub node: Node,
        pub labels: Vec<Label>,
    }

    pub fn numbered_rail(engine: &mut TextEngine, items: &[&str], width: f32, p: mk::Palette) -> Rail {
        let mono = crate::mono_family();
        let mut rows = Vec::new();
        let mut labels = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let num = format!("{:02}", i + 1);
            let (nw, nh) = engine.measure_family(mono, &num, mk::font_size::SMALL, mk::typo::Weight::Semibold);
            let nn = Node::Quad { w: nw, h: nh, color: mk::Rgba { a: 0.0, ..p.primary }, radius: 0.0 };
            labels.push(Label {
                text: num,
                px: mk::font_size::SMALL,
                weight: mk::typo::Weight::Semibold,
                family: mono,
                color: p.primary,
            });
            let para = paragraph(
                engine,
                item,
                mk::font_size::MEDIUM,
                mk::typo::Weight::Normal,
                p.on_surface,
                (width - nw - mk::spacing::M).max(40.0),
            );
            labels.extend(para.labels);
            rows.push(Node::Row { gap: mk::spacing::M, padding: 0.0, background: None, children: vec![nn, para.node] });
        }
        Rail { node: Node::Column { gap: mk::spacing::S, padding: 0.0, background: None, children: rows }, labels }
    }

    /// The LADDER (R202: human-led → assisted → autonomous): steps one
    /// below the other, per step title and sentence, the active one
    /// with a bright bar. Every step is a target (`slots`).
    pub struct Ladder {
        pub node: Node,
        pub labels: Vec<Label>,
        pub slots: Vec<usize>,
    }

    pub const LADDER_BAR_W: f32 = 4.0;

    pub fn ladder(
        engine: &mut TextEngine,
        steps: &[(&str, &str)],
        active: usize,
        width: f32,
        p: mk::Palette,
    ) -> Ladder {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mut rows = Vec::new();
        let mut labels = Vec::new();
        let mut slots = Vec::new();
        let mut placed = 0;
        for (i, (title, text)) in steps.iter().enumerate() {
            let on = i == active;
            slots.push(placed);
            let (tn, tl) = plain_label(
                engine,
                title,
                mk::font_size::MEDIUM,
                mk::typo::Weight::Semibold,
                if on { p.on_surface } else { p.on_surface_variant },
            );
            labels.push(tl);
            let para = paragraph(
                engine,
                text,
                mk::font_size::SMALL,
                mk::typo::Weight::Normal,
                p.on_surface_variant,
                (width - LADDER_BAR_W - 2.0 * mk::spacing::M).max(40.0),
            );
            let lines = para.labels.len();
            labels.extend(para.labels);
            let bar_h = 18.0 + lines as f32 * (mk::font_size::SMALL * 1.3 + 2.0);
            rows.push(Node::Row {
                gap: mk::spacing::M,
                padding: 0.0,
                background: Some((clear, 0.0)),
                children: vec![
                    Node::Quad {
                        w: LADDER_BAR_W,
                        h: bar_h,
                        color: if on { p.primary } else { p.outline.with_alpha(0.3) },
                        radius: mk::radius::MINI,
                    },
                    Node::Column { gap: 2.0, padding: 0.0, background: None, children: vec![tn, para.node] },
                ],
            });
            placed += 3 + lines; // ground, bar, title, lines
        }
        Ladder { node: Node::Column { gap: mk::spacing::XS, padding: 0.0, background: None, children: rows }, labels, slots }
    }

    /// The SEGMENTS (mkw::segments, Apple's segmented control): a track,
    /// in it one target per option, the active one raised. Placed:
    /// track, then per option ground + text (`slots` = grounds).
    pub struct Segments {
        pub node: Node,
        pub labels: Vec<Label>,
        pub slots: Vec<usize>,
    }

    pub fn segments(engine: &mut TextEngine, options: &[&str], active: usize, p: mk::Palette) -> Segments {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mut nodes = Vec::new();
        let mut labels = Vec::new();
        let mut slots = Vec::new();
        for (i, opt) in options.iter().enumerate() {
            let on = i == active;
            slots.push(1 + 2 * i);
            let (n, l) = plain_label(
                engine,
                opt,
                mk::font_size::SMALL,
                mk::typo::Weight::Semibold,
                if on { p.on_primary_container } else { p.on_surface_variant },
            );
            labels.push(l);
            nodes.push(Node::Wrap {
                pad_x: mk::spacing::M,
                pad_y: mk::spacing::XS,
                background: Some((if on { p.primary_container } else { clear }, mk::radius::capsule(26.0))),
                child: Box::new(n),
            });
        }
        Segments {
            node: Node::Row {
                gap: 2.0,
                padding: 2.0,
                background: Some((p.surface_container, mk::radius::capsule(30.0))),
                children: nodes,
            },
            labels,
            slots,
        }
    }

    /// The TABLE (mkw::table): header row in variant text, hairline,
    /// then rows with fixed column widths — every row a target
    /// (`row_slots`). Cells are truncated to their column (first wrap
    /// line + …). Placed: header cells (1 each), hairline, per row
    /// ground + cells.
    pub struct Table {
        pub node: Node,
        pub labels: Vec<Label>,
        pub row_slots: Vec<usize>,
    }

    pub fn table(
        engine: &mut TextEngine,
        columns: &[(&str, f32)],
        rows: &[Vec<String>],
        p: mk::Palette,
    ) -> Table {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let total: f32 = columns.iter().map(|c| c.1).sum::<f32>()
            + mk::spacing::M * columns.len().saturating_sub(1) as f32
            + 2.0 * mk::spacing::S;
        let mut labels = Vec::new();
        let cells = |engine: &mut TextEngine, texts: &[String], px: f32, weight: mk::typo::Weight, color: mk::Rgba, labels: &mut Vec<Label>| -> Vec<Node> {
            columns
                .iter()
                .enumerate()
                .map(|(i, (_, w))| {
                    let raw = texts.get(i).map(String::as_str).unwrap_or("");
                    let mut lines = wrap_text(engine, raw, px, weight, *w);
                    let mut text = lines.first().cloned().unwrap_or_default();
                    if lines.len() > 1 {
                        text.push('…');
                        lines.truncate(1);
                    }
                    let (n, l) = plain_label(engine, &text, px, weight, color);
                    labels.push(l);
                    Node::Slot { w: Some(*w), h: None, ax: Align::Start, ay: Align::Center, background: None, child: Box::new(n) }
                })
                .collect()
        };
        let head_texts: Vec<String> = columns.iter().map(|c| c.0.to_string()).collect();
        let head = Node::Row {
            gap: mk::spacing::M,
            padding: mk::spacing::S,
            background: None,
            children: cells(engine, &head_texts, mk::font_size::SMALL, mk::typo::Weight::Semibold, p.on_surface_variant, &mut labels),
        };
        let mut children = vec![
            head,
            Node::Quad { w: total, h: 1.0, color: p.outline.with_alpha(0.25), radius: 0.0 },
        ];
        let mut row_slots = Vec::new();
        let mut placed = columns.len() + 1;
        for row in rows {
            row_slots.push(placed);
            children.push(Node::Row {
                gap: mk::spacing::M,
                padding: mk::spacing::S,
                background: Some((clear, 0.0)),
                children: cells(engine, row, mk::font_size::MEDIUM, mk::typo::Weight::Normal, p.on_surface, &mut labels),
            });
            placed += 1 + columns.len();
        }
        Table { node: Node::Column { gap: 0.0, padding: 0.0, background: None, children }, labels, row_slots }
    }


    // ------------------------------------------------------------
    // Lager 106: SHAPE BLOCKS — ring, donut, lines, grid — as
    // registered shapes (matrixkit_shell::shapes) that the client lays
    // into a Pict like an image: `pict_of(shape, x, y, home, anchor)`.
    // Plus the SIDEBAR (mkw::sidebar_entry).
    // ------------------------------------------------------------

    pub use matrixkit_shell::shapes::{Cmd, Shape, Style as ShapeStyle};

    /// A ring gauge: full track, share `portion` (0..1) in `tint`
    /// clockwise from 12 o'clock. Box size×size.
    pub fn ring_gauge(size: f32, thickness: f32, portion: f32, tint: mk::Rgba, track: mk::Rgba) -> Shape {
        let r = (size - thickness) / 2.0;
        let c = size / 2.0;
        let top = -std::f32::consts::FRAC_PI_2;
        let sweep = portion.clamp(0.0, 1.0) * std::f32::consts::TAU;
        let mut s = Shape::new(size, size)
            .op(vec![Cmd::Circle(c, c, r)], ShapeStyle::Stroke { tint: track, width: thickness });
        if sweep > 0.0 {
            s = s.op(
                vec![Cmd::Arc { cx: c, cy: c, r, start: top, sweep, connect: false }],
                ShapeStyle::Stroke { tint, width: thickness },
            );
        }
        s
    }

    /// The DONUT: shares (value, colour) as ring segments with a small
    /// gap between them. Box size×size.
    pub fn donut(size: f32, thickness: f32, parts: &[(f32, mk::Rgba)]) -> Shape {
        let total: f32 = parts.iter().map(|p| p.0.max(0.0)).sum();
        let r = (size - thickness) / 2.0;
        let c = size / 2.0;
        let gap = if parts.len() > 1 { 0.03 } else { 0.0 };
        let mut s = Shape::new(size, size);
        if total <= 0.0 {
            return s;
        }
        let mut a = -std::f32::consts::FRAC_PI_2;
        for (v, tint) in parts {
            let sweep = (v.max(0.0) / total) * std::f32::consts::TAU;
            let drawn = (sweep - gap).max(0.0);
            if drawn > 0.0 {
                s = s.op(
                    vec![Cmd::Arc { cx: c, cy: c, r, start: a + gap / 2.0, sweep: drawn, connect: false }],
                    ShapeStyle::Stroke { tint: *tint, width: thickness },
                );
            }
            a += sweep;
        }
        s
    }

    /// A line or polyline through `points` (in the w×h box).
    pub fn polyline(w: f32, h: f32, points: &[(f32, f32)], width: f32, tint: mk::Rgba, dash: Option<(f32, f32)>) -> Shape {
        let mut path = Vec::with_capacity(points.len());
        for (i, (x, y)) in points.iter().enumerate() {
            path.push(if i == 0 { Cmd::Move(*x, *y) } else { Cmd::Line(*x, *y) });
        }
        let style = match dash {
            Some((on, off)) => ShapeStyle::Dashed { tint, width, on, off, phase: 0.0 },
            None => ShapeStyle::Stroke { tint, width },
        };
        Shape::new(w, h).op(path, style)
    }

    /// The GRID GROUND (R202): dot grid at intervals of `step` — the
    /// quiet stage of a graph.
    pub fn grid_ground(w: f32, h: f32, step: f32, tint: mk::Rgba) -> Shape {
        let step = step.max(4.0);
        let mut path = Vec::new();
        let mut y = step;
        while y < h {
            let mut x = step;
            while x < w {
                path.push(Cmd::Circle(x, y, 0.9));
                x += step;
            }
            y += step;
        }
        Shape::new(w, h).op(path, ShapeStyle::Fill(tint))
    }

    /// The shape as a scene image at a laid box (Placed `anchor` = the
    /// stage's quad): registered, Pict built.
    pub fn pict_of(shape: &Shape, x: f32, y: f32, home: Rect, anchor: usize) -> matrixkit_shell::Pict {
        matrixkit_shell::Pict {
            path: matrixkit_shell::shapes::register(shape),
            box_w: shape.w.round().max(1.0) as u32,
            box_h: shape.h.round().max(1.0) as u32,
            fit: matrixkit_image::Fit::Contain,
            x,
            y,
            radius: 0.0,
            home,
            anchor,
            clip: None,
        }
    }

    /// The SIDEBAR (mkw::sidebar_entry): entries with symbol and title
    /// across the full width, the chosen one on primary_container.
    /// Every entry is a target (`slots`, 3 Placed per entry: ground,
    /// symbol measure, text measure).
    pub const PLACED_PER_SIDEBAR_ENTRY: usize = 3;

    pub struct Sidebar {
        pub node: Node,
        pub labels: Vec<Label>,
        pub slots: Vec<usize>,
    }

    pub fn sidebar(
        engine: &mut TextEngine,
        entries: &[(char, &str)],
        selected: Option<usize>,
        width: f32,
        p: mk::Palette,
    ) -> Sidebar {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mut rows = Vec::new();
        let mut labels = Vec::new();
        let mut slots = Vec::new();
        for (i, (sym, title)) in entries.iter().enumerate() {
            let on = selected == Some(i);
            let ink = if on { p.on_primary_container } else { p.on_surface };
            let (gn, gl) = glyph(engine, *sym, mk::font_size::MEDIUM, if on { ink } else { p.on_surface_variant });
            let (tn, tl) = plain_label(engine, title, mk::font_size::MEDIUM, if on { mk::typo::Weight::Semibold } else { mk::typo::Weight::Normal }, ink);
            labels.push(gl);
            labels.push(tl);
            slots.push(i * PLACED_PER_SIDEBAR_ENTRY);
            rows.push(Node::Slot {
                w: Some(width),
                h: None,
                ax: Align::Start,
                ay: Align::Center,
                background: Some((if on { p.primary_container } else { clear }, mk::radius::SMALL)),
                child: Box::new(Node::Wrap {
                    pad_x: mk::spacing::M,
                    pad_y: mk::spacing::S,
                    background: None,
                    child: Box::new(Node::Row { gap: mk::spacing::M, padding: 0.0, background: None, children: vec![gn, tn] }),
                }),
            });
        }
        Sidebar { node: Node::Column { gap: 2.0, padding: 0.0, background: None, children: rows }, labels, slots }
    }

    // ------------------------------------------------------------
    // Lager 108: the DRAG as a kit helper — the Werft dragged its
    // kanban cards with its own bookkeeping (drag, card_hits, column
    // spans). The house supplies strike/travel/release raw; here a
    // drag is made from it: only after DRAG_SLOP does it live, a ghost
    // follows the pointer, release names origin and target.
    // ------------------------------------------------------------

    /// How far a press must wander before it is a drag (and stops being
    /// a click).
    pub const DRAG_SLOP: f32 = 6.0;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Drag {
        /// The role under the press (what is being dragged).
        pub origin: usize,
        pub start: (f32, f32),
        pub at: (f32, f32),
        /// Only from DRAG_SLOP on: before that it is a click on standby.
        pub live: bool,
    }

    impl Drag {
        pub fn begin(origin: usize, x: f32, y: f32) -> Self {
            Self { origin, start: (x, y), at: (x, y), live: false }
        }

        /// The pointer wanders; `true` as soon as (or as long as) the drag lives.
        pub fn travel(&mut self, x: f32, y: f32) -> bool {
            self.at = (x, y);
            if !self.live
                && ((x - self.start.0).abs() > DRAG_SLOP || (y - self.start.1).abs() > DRAG_SLOP)
            {
                self.live = true;
            }
            self.live
        }

        /// Offset of the ghost relative to the grab point.
        pub fn offset(&self) -> (f32, f32) {
            (self.at.0 - self.start.0, self.at.1 - self.start.1)
        }

        /// Release: if the drag lived and a target lies under the
        /// pointer, (origin, target) comes — otherwise it was a click or
        /// a drag into the void.
        pub fn end(self, target: Option<usize>) -> Option<(usize, usize)> {
            match (self.live, target) {
                (true, Some(t)) => Some((self.origin, t)),
                _ => None,
            }
        }
    }

    /// The drag's GHOST: the dragged piece slightly grown; the caller
    /// lays it topmost in the stack and lifts it with
    /// `lift(.., floating_shadow())`.
    pub fn ghost(node: Node) -> Node {
        Node::Scale { factor: 1.04, child: Box::new(node) }
    }

    /// A drag TARGET: the box the ghost is currently hovering over gets
    /// a ring in the primary colour.
    pub fn drop_target(child: Node, hot: bool, p: mk::Palette) -> Node {
        ring(child, hot, mk::radius::NORMAL, p)
    }

    // ------------------------------------------------------------
    // Lager 109: DIVIDER, PAN-ZOOM VIEW and SWIPE ACTIONS — the last
    // three cards of the remainder list. All three live on
    // Node::Offset (Theme, Lager 109).
    // ------------------------------------------------------------

    /// The DIVIDER (mkw had none; iced's pane_grid): left and right one
    /// box of fixed width each, between them a 6 px handle that is the
    /// drag's target (`divider_slot`). The caller computes the new
    /// ratio with `split_ratio` and clips the halves as with the roll.
    pub const DIVIDER_W: f32 = 6.0;

    pub struct Split {
        pub node: Node,
        pub divider_slot: usize,
        pub left_w: f32,
        pub right_w: f32,
    }

    pub fn split(left: Node, right: Node, ratio: f32, width: f32, height: f32, hot: bool, p: mk::Palette) -> Split {
        let ratio = ratio.clamp(0.15, 0.85);
        let left_w = ((width - DIVIDER_W) * ratio).round();
        let right_w = width - DIVIDER_W - left_w;
        let left_slot = Node::Slot {
            w: Some(left_w),
            h: Some(height),
            ax: Align::Start,
            ay: Align::Start,
            background: None,
            child: Box::new(left),
        };
        let divider_slot = matrixkit_theme::scene::placed_count(&left_slot);
        let node = Node::Row {
            gap: 0.0,
            padding: 0.0,
            background: None,
            children: vec![
                left_slot,
                Node::Quad {
                    w: DIVIDER_W,
                    h: height,
                    color: if hot { p.primary.with_alpha(0.6) } else { p.outline.with_alpha(0.25) },
                    radius: mk::radius::MINI,
                },
                Node::Slot {
                    w: Some(right_w),
                    h: Some(height),
                    ax: Align::Start,
                    ay: Align::Start,
                    background: None,
                    child: Box::new(right),
                },
            ],
        };
        Split { node, divider_slot, left_w, right_w }
    }

    /// The ratio from the pointer x (relative to the divider start).
    pub fn split_ratio(x: f32, width: f32) -> f32 {
        ((x - DIVIDER_W / 2.0) / (width - DIVIDER_W).max(1.0)).clamp(0.15, 0.85)
    }

    /// The PAN-ZOOM VIEW (R202: graph surface): a w×h box, in it the
    /// child stretched by `zoom` and shifted by `pan`. Whatever sticks
    /// out of the box the caller clips with `clip_from` — the roll does
    /// it the same way. Stroke widths scale along; whoever wants them
    /// constant computes the affine in the model (R202 D.4).
    pub fn viewport(child: Node, w: f32, h: f32, pan: (f32, f32), zoom: f32, p: mk::Palette) -> Node {
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        Node::Slot {
            w: Some(w),
            h: Some(h),
            ax: Align::Start,
            ay: Align::Start,
            background: Some((clear, 0.0)),
            child: Box::new(Node::Offset {
                dx: pan.0,
                dy: pan.1,
                child: Box::new(Node::Scale { factor: zoom.max(0.05), child: Box::new(child) }),
            }),
        }
    }

    /// Zoom around the pointer: the model point under the pointer stays
    /// put. `cursor` relative to the box, `pan` the previous offset.
    pub fn zoom_at(cursor: (f32, f32), pan: (f32, f32), old: f32, new: f32) -> (f32, f32) {
        let old = old.max(0.05);
        let model = ((cursor.0 - pan.0) / old, (cursor.1 - pan.1) / old);
        (cursor.0 - model.0 * new, cursor.1 - model.1 * new)
    }

    /// SWIPE ACTIONS of a row (Radar, iOS): behind the row lie action
    /// fields on the right (glyph on colour); the row wanders left by
    /// `reveal` and uncovers them. Placed: per action ground + glyph
    /// (`action_slots`), then the row. `reveal` clamps to the width of
    /// the actions.
    pub const SWIPE_ACTION_W: f32 = 72.0;

    pub struct Swipe {
        pub node: Node,
        pub labels: Vec<Label>,
        pub action_slots: Vec<usize>,
        pub max_reveal: f32,
    }

    pub fn swipe(
        engine: &mut TextEngine,
        row: Node,
        width: f32,
        height: f32,
        reveal: f32,
        actions: &[(char, mk::Rgba)],
        p: mk::Palette,
    ) -> Swipe {
        let max_reveal = actions.len() as f32 * SWIPE_ACTION_W;
        let reveal = reveal.clamp(0.0, max_reveal);
        let mut labels = Vec::new();
        let mut action_slots = Vec::new();
        let mut fields = Vec::new();
        for (i, (g, tint)) in actions.iter().enumerate() {
            action_slots.push(i * 2);
            let (gn, gl) = glyph(engine, *g, mk::font_size::LARGE, p.on_primary);
            labels.push(gl);
            fields.push(Node::Slot {
                w: Some(SWIPE_ACTION_W),
                h: Some(height),
                ax: Align::Center,
                ay: Align::Center,
                background: Some((*tint, 0.0)),
                child: Box::new(gn),
            });
        }
        let actions_row = Node::Row { gap: 0.0, padding: 0.0, background: None, children: fields };
        let node = Node::Stack {
            children: vec![
                Node::Slot {
                    w: Some(width),
                    h: Some(height),
                    ax: Align::End,
                    ay: Align::Start,
                    background: None,
                    child: Box::new(actions_row),
                },
                Node::Offset { dx: -reveal, dy: 0.0, child: Box::new(row) },
            ],
        };
        Swipe { node, labels, action_slots, max_reveal }
    }

    /// The reveal from a horizontal drag: only leftwards counts,
    /// clamped to the maximum; below half a field the row snaps back
    /// (the caller asks `snap`).
    pub fn swipe_reveal(drag: &Drag, max_reveal: f32) -> f32 {
        (-drag.offset().0).clamp(0.0, max_reveal)
    }

    pub fn swipe_snap(reveal: f32, max_reveal: f32) -> f32 {
        if reveal >= SWIPE_ACTION_W / 2.0 { max_reveal } else { 0.0 }
    }


    // ------------------------------------------------------------
    // Lager 111 (R207): what Rabbit built by hand becomes a family —
    // the sheet, the hover guard, text buttons and links with state,
    // the closable tab chip, the suggestion list under a field, the
    // code card and the paragraph card. All slot indices RELATIVE to
    // the first Placed.
    // ------------------------------------------------------------

    /// The SHEET: building a scene from nodes, labels and images — with
    /// the bookkeeping every client otherwise wrote itself (Studio,
    /// Hilfe, Rabbit each had their own `Build`). A node is laid, its
    /// Placed appended with role, a label anchors at a slot; at the end
    /// the scene is made from it.
    #[derive(Default)]
    pub struct Sheet {
        pub placed: Vec<Placed>,
        pub roles: Vec<Option<usize>>,
        pub texts: Vec<matrixkit_shell::Text>,
        pub picts: Vec<matrixkit_shell::Pict>,
    }

    impl Sheet {
        pub fn new() -> Self {
            Self::default()
        }

        /// Lay a node; returns (first slot, count, width, height).
        pub fn node(&mut self, node: &Node, x: f32, y: f32, role: Option<usize>, clip: Option<Rect>) -> (usize, usize, f32, f32) {
            let first = self.placed.len();
            let mut out = Vec::new();
            let (w, h) = matrixkit_theme::scene::lay_out(node, x, y, &mut out);
            let n = out.len();
            if let Some(c) = clip {
                for q in &mut out {
                    q.clip = Some(c);
                }
            }
            self.placed.extend(out);
            self.roles.extend(std::iter::repeat(role).take(n));
            (first, n, w, h)
        }

        /// A label at a laid slot (with offset).
        pub fn label(&mut self, slot: usize, l: &Label, dx: f32, dy: f32, clip: Option<Rect>) {
            let Some(q) = self.placed.get(slot).copied() else { return };
            self.texts.push(matrixkit_shell::Text {
                s: l.text.clone(),
                px: l.px,
                weight: l.weight,
                family: l.family,
                color: l.color,
                x: q.x + dx,
                y: q.y + dy,
                home: clip.unwrap_or(Rect::of(&q)),
                anchor: slot,
                clip,
            });
        }

        /// A quad; returns its slot.
        #[allow(clippy::too_many_arguments)]
        pub fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, color: mk::Rgba, radius: f32, role: Option<usize>, clip: Option<Rect>) -> usize {
            self.node(&Node::Quad { w, h, color, radius }, x, y, role, clip).0
        }

        /// A label on a freshly laid, invisible measuring box.
        #[allow(clippy::too_many_arguments)]
        pub fn text(&mut self, s: String, px: f32, weight: mk::typo::Weight, family: &'static str, color: mk::Rgba, x: f32, y: f32, w: f32, h: f32, role: Option<usize>, clip: Option<Rect>) -> usize {
            let clear = mk::Rgba { a: 0.0, ..color };
            let (first, _, _, _) = self.node(&Node::Quad { w, h, color: clear, radius: 0.0 }, x, y, role, clip);
            self.texts.push(matrixkit_shell::Text {
                s,
                px,
                weight,
                family,
                color,
                x,
                y,
                home: clip.unwrap_or(Rect::of(&self.placed[first])),
                anchor: first,
                clip,
            });
            first
        }

        /// A paragraph (wrapped at `width`); returns the height.
        #[allow(clippy::too_many_arguments)]
        pub fn paragraph(&mut self, engine: &mut TextEngine, text: &str, style: mk::typo::TextStyle, color: mk::Rgba, x: f32, y: f32, width: f32, role: Option<usize>, clip: Option<Rect>) -> f32 {
            let para = paragraph(engine, text, style.px(), style.weight, color, width.max(20.0));
            let (first, _, _, h) = self.node(&para.node, x, y, role, clip);
            for (i, l) in para.labels.iter().enumerate() {
                self.label(first + i, l, 0.0, 0.0, clip);
            }
            h
        }

        /// A card with a paragraph (card_paragraph); returns the height.
        #[allow(clippy::too_many_arguments)]
        pub fn card_paragraph(&mut self, engine: &mut TextEngine, text: &str, style: mk::typo::TextStyle, color: mk::Rgba, x: f32, y: f32, width: f32, role: Option<usize>, clip: Option<Rect>, p: mk::Palette) -> f32 {
            let c = card_paragraph(engine, text, style.px(), style.weight, color, width, p);
            let (first, _, _, h) = self.node(&c.node, x, y, role, clip);
            for (i, l) in c.labels.iter().enumerate() {
                self.label(first + CARD_PLACED + i, l, 0.0, 0.0, clip);
            }
            h
        }

        /// A code card (code_card); returns the height.
        #[allow(clippy::too_many_arguments)]
        pub fn code_card(&mut self, engine: &mut TextEngine, text: &str, style: mk::typo::TextStyle, color: mk::Rgba, x: f32, y: f32, width: f32, clip: Option<Rect>, p: mk::Palette) -> f32 {
            let c = code_card(engine, text, style.px(), style.weight, color, width, p);
            let (first, _, _, h) = self.node(&c.node, x, y, None, clip);
            for (i, l) in c.labels.iter().enumerate() {
                self.label(first + CARD_PLACED + i, l, 0.0, 0.0, clip);
            }
            h
        }

        /// The finished scene.
        pub fn into_scene(self) -> matrixkit_shell::Scene {
            matrixkit_shell::Scene { placed: self.placed, roles: self.roles, texts: self.texts, picts: self.picts, curves: Vec::new() }
        }
    }

    /// Placed that a card lays BEFORE its child (border + ground).
    pub const CARD_PLACED: usize = 2;

    /// The HOVER GUARD: remembers the laid scene and in motion() hits
    /// the role under the pointer. Roles from `ceiling` up (traffic
    /// lights, root layer) do not count — they have their own care.
    #[derive(Default)]
    pub struct Hover {
        placed: Vec<Placed>,
        roles: Vec<Option<usize>>,
        role: Option<usize>,
        ceiling: usize,
    }

    impl Hover {
        pub fn new(ceiling: usize) -> Self {
            Hover { placed: Vec::new(), roles: Vec::new(), role: None, ceiling }
        }

        /// After every view(): remember the scene (before handing it over).
        pub fn learn(&mut self, placed: &[Placed], roles: &[Option<usize>]) {
            self.placed = placed.to_vec();
            self.roles = roles.to_vec();
        }

        /// The pointer stands at (x, y): did the role change?
        pub fn point(&mut self, x: f64, y: f64) -> bool {
            let role = matrixkit_theme::scene::hit(&self.placed, x as f32, y as f32)
                .and_then(|i| self.roles.get(i).copied().flatten())
                .filter(|r| *r < self.ceiling);
            let changed = role != self.role;
            self.role = role;
            changed
        }

        /// The pointer leaves the surface: was something raised?
        pub fn leave(&mut self) -> bool {
            self.role.take().is_some()
        }

        pub fn role(&self) -> Option<usize> {
            self.role
        }

        pub fn is(&self, role: Option<usize>) -> bool {
            role.is_some() && role == self.role
        }

        /// The state of a button with this role.
        pub fn status(&self, role: Option<usize>) -> Status {
            if self.is(role) { Status::Hovered } else { Status::Idle }
        }
    }

    /// The tint of a label per state: under the pointer it brightens
    /// (on_surface over the colour, 0.25).
    pub fn hover_tint(color: mk::Rgba, status: Status, p: mk::Palette) -> mk::Rgba {
        match status {
            Status::Hovered | Status::Pressed => p.on_surface.over(color, 0.25),
            Status::Idle => color,
        }
    }

    /// Placed of a link/text button with state: state layer + text
    /// measure — always two, the layer is transparent at rest.
    pub const PLACED_PER_LINK: usize = 2;
    /// The margin of the state layer around the label.
    pub const LINK_PAD: (f32, f32) = (6.0, 3.0);

    /// A LINK (clickable label) with state: under the pointer it carries
    /// the state layer (state_layer::HOVER) and brightens. Laid at
    /// (x − LINK_PAD.0, y − LINK_PAD.1) so the text itself stays at
    /// (x, y). Label on `first + 1`.
    pub fn link(
        engine: &mut TextEngine,
        text: &str,
        px: f32,
        weight: mk::typo::Weight,
        color: mk::Rgba,
        status: Status,
        p: mk::Palette,
    ) -> (Node, Label) {
        let (node, l) = plain_label(engine, text, px, weight, hover_tint(color, status, p));
        let ground = match status {
            Status::Hovered | Status::Pressed => p.on_surface.with_alpha(0.10),
            Status::Idle => mk::Rgba { a: 0.0, ..p.surface },
        };
        (
            Node::Wrap {
                pad_x: LINK_PAD.0,
                pad_y: LINK_PAD.1,
                background: Some((ground, mk::radius::SMALL)),
                child: Box::new(node),
            },
            l,
        )
    }

    /// The TEXT BUTTON with state — like `text_button`, but with a
    /// state layer (PLACED_PER_LINK, label on `first + 1`).
    pub fn text_button_with(engine: &mut TextEngine, text: &str, status: Status, p: mk::Palette) -> (Node, Label) {
        link(engine, text, mk::font_size::SMALL, mk::typo::Weight::Semibold, p.primary, status, p)
    }

    /// The closable TAB CHIP — literally the iced counterpart
    /// (mkw::tab_chip): label and × in ONE capsule with radius::SMALL,
    /// air S/4, CAPTION; active on primary_container, otherwise
    /// on_surface over surface_container (0.06). Placed: ground, label,
    /// × — `close_slot` = 2, label on 1.
    pub struct TabChipBuild {
        pub node: Node,
        pub label: Label,
        pub close: Label,
        pub close_slot: usize,
    }

    pub fn tab_chip_closable(engine: &mut TextEngine, text: &str, active: bool, p: mk::Palette) -> TabChipBuild {
        let (bg, ink) = if active {
            (p.primary_container, p.on_primary_container)
        } else {
            (p.on_surface.over(p.surface_container, 0.06), p.on_surface_variant)
        };
        let cap = mk::typo::CAPTION;
        let (ln, label) = plain_label(engine, text, cap.px(), cap.weight, ink);
        let (xn, close) = plain_label(engine, "×", cap.px(), cap.weight, ink);
        let node = Node::Wrap {
            pad_x: mk::spacing::S,
            pad_y: 4.0,
            background: Some((bg, mk::radius::SMALL)),
            child: Box::new(Node::Row { gap: mk::spacing::S, padding: 0.0, background: None, children: vec![ln, xn] }),
        };
        TabChipBuild { node, label, close, close_slot: 2 }
    }

    /// An entry of the suggestion list: label and origin.
    pub struct Suggestion<'a> {
        pub label: &'a str,
        pub origin: &'a str,
    }

    /// The build of the suggestion list: Placed at ABSOLUTE coordinates
    /// (it lies over the page), roles = row index, labels per slot. The
    /// caller raises roles and anchors.
    pub struct SuggestionsBuild {
        pub placed: Vec<Placed>,
        pub roles: Vec<Option<usize>>,
        pub labels: Vec<(Label, usize)>,
        /// The height of the list under the field.
        pub height: f32,
    }

    /// The SUGGESTION LIST under an input field (owner: «like a
    /// classic search»): same width as the field, reaching under the
    /// pill at the top, round at the bottom, floating; every row a target
    /// with a state layer under the pointer (`hover` = row index).
    pub fn suggestions(
        engine: &mut TextEngine,
        field: Rect,
        entries: &[Suggestion<'_>],
        hover: Option<usize>,
        p: mk::Palette,
    ) -> SuggestionsBuild {
        let body = mk::typo::BODY;
        let cap = mk::typo::CAPTION;
        let (pad_x, pad_y) = (mk::spacing::M, mk::spacing::S);
        let mut b = Sheet::new();
        let mut rows: Vec<(Label, Label, f32)> = Vec::new();
        let mut list_h = 0.0f32;
        for e in entries {
            let (_, ll) = plain_label(engine, e.label, body.px(), body.weight, p.on_surface);
            let (_, ol) = plain_label(engine, e.origin, cap.px(), cap.weight, p.on_surface_variant);
            let (_, lh) = engine.measure_styled(e.label, body.px(), body.weight);
            let rh = lh + 2.0 * pad_y;
            rows.push((ll, ol, rh));
            list_h += rh;
        }
        let top = field.y + field.h - mk::radius::SMALL;
        let height = list_h + mk::radius::SMALL + pad_y;
        let ground = Node::Quad { w: field.w, h: height, color: p.surface_container_high, radius: mk::radius::SMALL };
        let gslot = b.node(&ground, field.x, top, None, None).0;
        lift(&mut b.placed, gslot, floating_shadow());
        let mut labels: Vec<(Label, usize)> = Vec::new();
        let mut ry = field.y + field.h + pad_y * 0.5;
        for (i, e) in entries.iter().enumerate() {
            let (ll, ol, rh) = &rows[i];
            let hovered = hover == Some(i);
            let row_bg = if hovered { p.on_surface.with_alpha(0.10) } else { mk::Rgba { a: 0.0, ..p.surface } };
            b.quad(field.x + 4.0, ry, field.w - 8.0, *rh, row_bg, mk::radius::SMALL, Some(i), None);
            let mut ll = ll.clone();
            ll.color = hover_tint(ll.color, if hovered { Status::Hovered } else { Status::Idle }, p);
            let (ln, _) = plain_label(engine, e.label, body.px(), body.weight, ll.color);
            let (lf, _, lw, _) = b.node(&ln, field.x + pad_x, ry + pad_y, Some(i), None);
            labels.push((ll, lf));
            let (on, _) = plain_label(engine, e.origin, cap.px(), cap.weight, ol.color);
            let (of, _, _, _) = b.node(&on, field.x + pad_x + lw + mk::spacing::S, ry + pad_y + 2.0, Some(i), None);
            labels.push((ol.clone(), of));
            ry += rh;
        }
        SuggestionsBuild { placed: b.placed, roles: b.roles, labels, height: height - mk::radius::SMALL }
    }

    /// The PARAGRAPH CARD: a card around a wrapped paragraph that fills
    /// the column (quote, hint, module). Labels on
    /// `first + CARD_PLACED + i`.
    pub fn card_paragraph(
        engine: &mut TextEngine,
        text: &str,
        px: f32,
        weight: mk::typo::Weight,
        color: mk::Rgba,
        width: f32,
        p: mk::Palette,
    ) -> Paragraph {
        let inner = width - 2.0 * mk::spacing::M;
        let para = paragraph(engine, text, px, weight, color, inner.max(20.0));
        let mut lines = para.node;
        if let Node::Column { children, .. } = &mut lines {
            children.push(Node::Quad { w: inner, h: 0.0, color: mk::Rgba { a: 0.0, ..color }, radius: 0.0 });
        }
        Paragraph { node: card(lines, p), labels: para.labels }
    }

    /// The CODE CARD: lines instead of running text, in the mono font —
    /// every line a measure, long lines wrap at the edge (finger probe
    /// Rabbit 21.09.2026: the paragraph wrapping merged the lines).
    /// Labels on `first + CARD_PLACED + i`. `arm_mono` beforehand.
    pub fn code_card(
        engine: &mut TextEngine,
        text: &str,
        px: f32,
        weight: mk::typo::Weight,
        color: mk::Rgba,
        width: f32,
        p: mk::Palette,
    ) -> Paragraph {
        let inner = width - 2.0 * mk::spacing::M;
        let mono = crate::mono_family();
        let mut nodes: Vec<Node> = Vec::new();
        let mut labels: Vec<Label> = Vec::new();
        for line in text.lines() {
            let shown = if line.is_empty() { " " } else { line };
            let wrapped = engine.wrap_family(mono, shown, px, weight, inner.max(40.0));
            for l in wrapped.lines() {
                let (w, h) = engine.measure_family(mono, l, px, weight);
                nodes.push(Node::Quad { w, h, color: mk::Rgba { a: 0.0, ..color }, radius: 0.0 });
                labels.push(Label { text: l.to_string(), px, weight, family: mono, color });
            }
        }
        nodes.push(Node::Quad { w: inner, h: 0.0, color: mk::Rgba { a: 0.0, ..color }, radius: 0.0 });
        Paragraph { node: card(Node::Column { gap: 2.0, padding: 0.0, background: None, children: nodes }, p), labels }
    }

    #[cfg(test)]
    mod tests {

        /// Lager 114: a word without spaces breaks at characters instead
        /// of running over the edge.
        #[test]
        fn overlong_words_break_at_characters() {
            let mut e = TextEngine::new();
            let px = mk::typo::BODY.px();
            let url = "https://www.example.org/inhalt.jagd-auf-marder-und-fuechse-stadt-jagt-kaninchen-mit-greifvoegeln-page1.0834";
            let lines = wrap_text(&mut e, &format!("Quelle {url} Ende"), px, mk::typo::Weight::Normal, 200.0);
            assert!(lines.len() >= 4, "{lines:?}");
            for l in &lines {
                assert!(e.measure_styled(l, px, mk::typo::Weight::Normal).0 <= 200.0, "{l}");
            }
            let joined: String = lines.concat();
            assert!(joined.contains("Quelle") && joined.ends_with("Ende"), "{joined}");
        }

        /// R207: the truncation ends with an ellipsis and fits.
        #[test]
        fn ellipsize_fits_and_keeps_the_start() {
            let mut e = TextEngine::new();
            let px = mk::typo::BODY.px();
            let long = "Willkommen bei Rabbit — der Gastgeber für das Hardline-Netz";
            let short = ellipsize(&mut e, long, px, mk::typo::Weight::Normal, 120.0);
            assert!(short.ends_with('…'), "{short}");
            assert!(short.starts_with("Will"), "{short}");
            let (w, _) = e.measure_styled(&short, px, mk::typo::Weight::Normal);
            assert!(w <= 120.0, "gekürzt {w} > 120");
            assert_eq!(ellipsize(&mut e, "kurz", px, mk::typo::Weight::Normal, 200.0), "kurz");
        }

        /// R207: the flow row wraps and lays no Placed itself.
        #[test]
        fn flow_wraps_without_extra_placed() {
            let q = |w: f32| Node::Quad { w, h: 10.0, color: mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, radius: 0.0 };
            let node = flow(vec![q(100.0), q(100.0), q(100.0)], 8.0, 220.0);
            let mut placed = Vec::new();
            let (w, h) = matrixkit_theme::scene::lay_out(&node, 0.0, 0.0, &mut placed);
            assert_eq!(placed.len(), 3, "nur die Kinder legen Placed");
            assert_eq!(placed[2].y, 18.0, "das dritte Kind steht in der zweiten Zeile");
            assert!(w <= 220.0 && h == 28.0, "{w}×{h}");
        }

        /// R207: the header title shortens at phone width.
        #[test]
        fn the_header_title_shortens_at_phone_width() {
            let mut e = TextEngine::new();
            let long = "Eine sehr lange Seitenüberschrift, die auf keinem Handy in den Kopf passt";
            let st = HeaderState { active: true, lights_hover: false, rolled_up: false, magnify: [1.0; 5] };
            let h = frame_header_with(&mut e, long, st, mk::form::PHONE_W, None, mk::Palette::default());
            assert!(h.title.text.ends_with('…'), "{}", h.title.text);
            let wide = frame_header_with(&mut e, "Kurz", st, 900.0, None, mk::Palette::default());
            assert_eq!(wide.title.text, "Kurz");
            // With a wide badge a short title moves into the gap instead
            // of shortening — and the slots stay.
            let badge = ExtensionBadge { name: "Dienst läuft", glyph: symbol::PUBLIC, on: true };
            let hb = frame_header_with(&mut e, "Hardline", st, mk::form::PHONE_W, Some(&badge), mk::Palette::default());
            assert_eq!(hb.title.text, "Hardline");
            let mut placed = Vec::new();
            matrixkit_theme::scene::lay_out(&hb.node, 0.0, 0.0, &mut placed);
            assert_eq!(placed.len(), 10);
            let t = placed[hb.title_slot];
            let lights_end = placed[hb.light_slots[2]].x + placed[hb.light_slots[2]].w;
            assert!(t.x > lights_end, "Titel {} rechts der Ampeln {}", t.x, lights_end);
        }

        /// The block chip cuts nothing off: every wrapped line gets its
        /// own measure, Placed = ground + kind + lines.
        #[test]
        fn the_block_chip_keeps_every_line() {
            let mut e = TextEngine::new();
            let long = "Empfehlung Faden: lokal beginnen, aber Cursor- und Auswahl-Zustand ohne Umwege in eine eigene Struktur legen, damit der Umzug ins Kit später klein bleibt.";
            let (node, _kind, lines) = choice_chip_block(
                &mut e, "EMPFEHLUNG", long, 300.0, true, mk::Palette::default());
            assert!(lines.len() >= 2, "langer Text muss mehrzeilig werden, kam {}", lines.len());
            let joined: String = lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join(" ");
            assert!(joined.replace(' ', "").contains("kleinbleibt"), "das Satzende fehlt: {joined}");
            let mut placed = Vec::new();
            matrixkit_theme::scene::lay_out(&node, 0.0, 0.0, &mut placed);
            assert_eq!(placed.len(), 2 + lines.len(), "Placed = Grund + Art + Zeilen");
        }

        /// Guard of the growing field (P1.3): wrapping grows, the caret
        /// sits on its line, the selection is a measured patch, and the
        /// Placed count is right (5 per line).
        #[test]
        fn input_area_grows_wraps_and_selects() {
            let mut e = TextEngine::new();
            let p = mk::Palette::default();
            let long = "die werft waechst mit dem funkspruch und bricht die zeilen sauber um";
            let spec = FieldSpec {
                value: long,
                cursor: long.chars().count(),
                anchor: Some(4),
                focused: true,
                blink: true,
                placeholder: "…",
                leading: None,
            };
            let a = input_area(&mut e, &spec, 220.0, 6, p);
            let mut placed = Vec::new();
            matrixkit_theme::scene::lay_out(&a.node, 0.0, 0.0, &mut placed);
            assert_eq!(placed.len(), a.placed, "Placed-Zahl hält Wort");
            assert!(a.labels.len() >= 2, "lange Sprüche brechen um");
            assert_eq!(a.placed, 3 + 5 * a.labels.len());
            // The selection (anchor 4 → end) has size on line 1 and
            // begins behind the measured prefix.
            let sel1 = &placed[4];
            assert!(sel1.w > 10.0, "Auswahlfleck der ersten Zeile trägt");
            assert!(placed[3].w > 0.0, "Auswahl-Vorlauf ist gemessen");
            // Height grows with the lines.
            let one = input_area(
                &mut e,
                &FieldSpec { value: "kurz", cursor: 4, anchor: None, ..spec },
                220.0,
                6,
                p,
            );
            assert!(a.height > one.height + 10.0, "das Feld wächst");
            assert_eq!(one.placed, 3 + 5, "eine Zeile, fünf Plätze");
        }

        use super::*;
        use matrixkit_theme::scene::lay_out;
        /// Markdown in the light (Lager 102): runs, chips and wrapping
        /// with self-named slots — and the Placed count matches the
        /// laying (the slot lesson from Lager 98).
        #[test]
        fn markdown_runs_anchor_on_their_own_slots() {
            let p = palette();
            let mut e = TextEngine::new();
            crate::arm_mono(&mut e);
            let text = "# Titel\nEin **fetter** Lauf mit `code` und *leiser* Stimme.\n- Punkt eins\n- Punkt zwei mit einem sehr langen Text der sicher einmal umbrechen muss auf schmaler Breite";
            let b = markdown(&mut e, text, 14.0, p.on_surface, 260.0, p);
            let mut out = Vec::new();
            lay_out(&b.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), b.placed, "gezählt = gelegt");
            for (l, slot) in &b.labels {
                assert!(*slot < out.len(), "Label-Anker ausserhalb: {}", l.text);
            }
            // The bold run is its OWN label in semibold.
            assert!(b.labels.iter().any(|(l, _)| l.text.contains("fetter") && l.weight == mk::typo::Weight::Semibold));
            // The code chip speaks mono and lies on a plate (its
            // measure quad has a wrap ground in front of it).
            let chip = b.labels.iter().find(|(l, _)| l.text == "code").expect("Chip fehlt");
            assert_eq!(chip.0.family, crate::mono_family());
            let mass = out[chip.1];
            let plate = out[chip.1 - 1];
            assert!(plate.x <= mass.x && plate.w >= mass.w, "die Platte liegt hinter dem Chip");
            // The bullet carries the primary colour, the long line wraps.
            assert!(b.labels.iter().any(|(l, _)| l.text == "•" && l.color == p.primary));
            assert!(out.iter().map(|q| q.y).fold(0.0f32, f32::max) > 40.0, "es gibt mehrere Zeilen");
            // Nothing runs out on the right.
            for q in &out {
                assert!(q.x + q.w <= 260.0 + 0.5, "läuft hinaus: {}", q.x + q.w);
            }
        }

        /// The root layer (Lager 93): every role must lie ACROSS THE
        /// AREA of its target. `hit()` takes the TOPMOST Placed — and
        /// above the Fertig ground lies its clear text measure: without
        /// area-wide roles a click right on "Fertig" hit the measure and
        /// the layer stayed open (finger probe 19.08.2026).
        #[test]
        fn the_root_layer_covers_its_buttons_with_roles() {
            let p = palette();
            let mut e = TextEngine::new();
            arm_symbols(&mut e);
            let mut root = RootPane::new(
                "matrix-probe-root",
                &[mk::permissions::Permission::Clipboard],
            );
            root.toggle();
            let about = RootAbout {
                name: "Erstes Licht",
                version: "1.0",
                description: "Eine Probe der Root-Ebene mit einem längeren Satz, der umbrechen muss.",
            };
            let build = root.build(&mut e, &about, 840.0, 560.0, p);
            assert_eq!(build.placed.len(), build.roles.len());
            // The veil covers the whole window and closes.
            assert_eq!(build.roles[0], Some(ROOT_VEIL));
            assert!((build.placed[0].w - 840.0).abs() < 0.01);
            assert!((build.placed[0].h - 560.0).abs() < 0.01);
            // The Fertig button: its ground AND everything in it carries DONE.
            let done = build
                .placed
                .iter()
                .position(|q| q.color == p.primary && (q.h - 40.0).abs() < 0.01)
                .expect("kein Fertig-Grund");
            assert_eq!(build.roles[done], Some(ROOT_DONE));
            let r = build.placed[done];
            for (j, q) in build.placed.iter().enumerate() {
                if j != done
                    && q.x >= r.x - 0.5
                    && q.y >= r.y - 0.5
                    && q.x + q.w <= r.x + r.w + 0.5
                    && q.y + q.h <= r.y + r.h + 0.5
                {
                    assert_eq!(
                        build.roles[j],
                        Some(ROOT_DONE),
                        "Quad {j} im Fertig-Knopf ohne Rolle"
                    );
                }
            }
            // The switch carries its role; the labels anchor validly.
            assert!(build.roles.iter().any(|r| *r == Some(ROOT_TOGGLE)));
            for (_, slot) in &build.labels {
                assert!(*slot < build.placed.len(), "Label-Anker ausserhalb");
            }
            // No line runs out of the card: all Placed lie in the window
            // (the wrapping is measured, not estimated).
            for q in &build.placed {
                assert!(q.x + q.w <= 840.0 + 0.5, "läuft rechts hinaus: {}", q.x + q.w);
            }
        }

        /// When locked NO switch flips — and the veil closes, the card
        /// swallows.
        #[test]
        fn the_root_lock_is_binding() {
            let mut root = RootPane::new(
                "matrix-probe-root",
                &[mk::permissions::Permission::Clipboard],
            );
            root.toggle();
            assert!(root.is_open());
            assert_eq!(root.strike(ROOT_TOGGLE), RootDeed::Calm, "verriegelt bleibt verriegelt");
            assert_eq!(root.strike(ROOT_CARD), RootDeed::Calm, "die Karte schluckt");
            assert_eq!(root.strike(ROOT_HELP), RootDeed::Help);
            assert_eq!(root.strike(ROOT_VEIL), RootDeed::Closed, "der Schleier schliesst");
            assert!(!root.is_open());
        }


        fn palette() -> mk::Palette {
            mk::Palette::default()
        }

        #[test]
        fn the_family_keeps_its_face() {
            // Colour worlds literally like mkw::button: Prominent carries
            // primary/on_primary, pressed the state layer lies over it,
            // disabled speaks the variant text.
            let p = palette();
            let mut e = TextEngine::new();
            let b = Button {
                label: "Sichern".into(),
                style: Style::Prominent,
                role: Role::Normal,
                size: Size::Normal,
                active: true, ..Default::default() };
            let (node, label) = button(&mut e, &b, Status::Idle, p);
            let ground = match node {
                Node::Wrap { background: Some((g, r)), .. } => {
                    assert_eq!(r, mk::CORNER_RADIUS);
                    g
                }
                _ => panic!("ein Knopf ist ein Wrap mit Grund"),
            };
            assert_eq!(ground, p.primary);
            assert_eq!(label.color, p.on_primary);

            let (pressed, _) = button(&mut e, &b, Status::Pressed, p);
            if let Node::Wrap { background: Some((g, _)), .. } = pressed {
                assert_eq!(g, p.on_primary.over(p.primary, mk::state_layer::PRESSED));
            }

            let locked = Button { active: false, label: "Gesperrt".into(), ..b };
            let (node, label) = button(&mut e, &locked, Status::Idle, p);
            assert_eq!(label.color, p.on_surface_variant);
            if let Node::Wrap { background: Some((g, _)), .. } = node {
                assert_eq!(g, p.primary.over(p.surface_container_high, 0.5));
            }
        }

        #[test]
        fn a_button_lays_two_placed_at_token_height() {
            let p = palette();
            let mut e = TextEngine::new();
            let b = Button {
                label: "Optionen".into(),
                style: Style::Tinted,
                role: Role::Normal,
                size: Size::Normal,
                active: true, ..Default::default() };
            let (node, _) = button(&mut e, &b, Status::Idle, p);
            let mut out = Vec::new();
            let (_, h) = lay_out(&node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), PLACED_PER_BUTTON);
            assert!((h - mk::control::BUTTON_HEIGHT).abs() < 0.5, "Token-Höhe, kein Augenmaß");
            let (g, t) = (out[0], out[1]);
            // Text centred in the step, breath = spacing::L.
            assert!(((t.y - g.y) - (g.h - t.h) / 2.0).abs() < 0.75);
            assert!((t.x - g.x - mk::spacing::L).abs() < 0.01);
            // And the anchor bookkeeping finds exactly this text measure.
            assert_eq!(label_anchors(&out, 0, 1), vec![(t.x, t.y)]);
        }

        #[test]
        fn the_tile_wears_its_chrome() {
            // Chrome tokens literally like tile_chrome: glass fill
            // α 0.62, 1 px border α 0.14, radius::LARGE, concentric fill
            // — and the name leads above the origin.
            let p = palette();
            let mut e = TextEngine::new();
            let t = Tile { name: "Dateien".into(), maker: "Matrix".into(), lines: vec![] };
            let (node, labels) = tile(&mut e, &t, p);
            let mut out = Vec::new();
            lay_out(&node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), tile_placed_count(&t));
            let (rim, fill, name, maker) = (out[0], out[1], out[2], out[3]);
            assert_eq!(rim.color, p.on_surface.with_alpha(0.14));
            assert_eq!(rim.radius, mk::radius::LARGE);
            assert_eq!(fill.color, p.surface_container.with_alpha(0.62));
            assert_eq!(fill.radius, mk::radius::inner(mk::radius::LARGE, 1.0));
            // Concentric: the fill sits 1 px inside the border.
            assert_eq!((fill.x - rim.x, fill.y - rim.y), (1.0, 1.0));
            // The name leads (SUBTITLE, on_surface), the origin
            // qualifies below (CAPTION, on_surface_variant).
            assert!(name.y < maker.y);
            assert_eq!(labels[0].weight, mk::typo::SUBTITLE.weight);
            assert_eq!(labels[0].color, p.on_surface);
            assert_eq!(labels[1].color, p.on_surface_variant);
            // And the anchor bookkeeping finds both labels.
            assert_eq!(tile_label_anchors(&out, 0, &t), vec![(name.x, name.y), (maker.x, maker.y)]);
        }

        #[test]
        fn open_windows_grow_the_tile() {
            // R174j: one caption line per open window — the tile grows,
            // the Placed budget grows with it.
            let p = palette();
            let mut e = TextEngine::new();
            let closed = Tile { name: "Terminal".into(), maker: "Matrix".into(), lines: vec![] };
            let open =
                Tile { name: "Terminal".into(), maker: "Matrix".into(), lines: vec!["1 Fenster".into()] };
            let (a, _) = tile(&mut e, &closed, p);
            let (b, labels) = tile(&mut e, &open, p);
            let (mut oa, mut ob) = (Vec::new(), Vec::new());
            let (_, ha) = lay_out(&a, 0.0, 0.0, &mut oa);
            let (_, hb) = lay_out(&b, 0.0, 0.0, &mut ob);
            assert_eq!(ob.len(), oa.len() + 1);
            assert_eq!(ob.len(), tile_placed_count(&open));
            assert!(hb > ha, "ein offenes Fenster macht die Kachel höher");
            assert_eq!(labels.len(), 3);
            assert_eq!(labels[2].text, "1 Fenster");
        }

        #[test]
        fn the_toggle_slides_its_thumb() {
            // Track 52×30 as a capsule, thumb a 24 circle; the thumb's
            // position IS the state — and disabled dims both.
            let p = palette();
            let (mut off, mut on) = (Vec::new(), Vec::new());
            lay_out(&toggle(false, false, p), 0.0, 0.0, &mut off);
            lay_out(&toggle(true, false, p), 0.0, 0.0, &mut on);
            assert_eq!(off.len(), PLACED_PER_TOGGLE);
            let track = off[0];
            assert_eq!((track.w, track.h), (mk::control::TOGGLE_TRACK_W, mk::control::TOGGLE_TRACK_H));
            assert_eq!(track.radius, mk::control::TOGGLE_TRACK_H / 2.0);
            assert_eq!(track.color, p.on_surface.over(p.surface_container_high, 0.12));
            assert_eq!(on[0].color, p.primary);
            // Off: thumb at the left edge; on: at the right edge.
            let knob_off = off[1];
            let knob_on = on[2];
            assert_eq!((knob_off.w, knob_off.radius), (24.0, mk::radius::NORMAL));
            assert_eq!(knob_off.x, 3.0);
            assert_eq!(knob_on.x, mk::control::TOGGLE_TRACK_W - 3.0 - 24.0);
            assert_eq!(knob_off.color, p.outline);
            assert_eq!(knob_on.color, p.on_primary);
            // Disabled: the 0.45 layer lies over track and thumb.
            let mut locked = Vec::new();
            lay_out(&toggle(true, true, p), 0.0, 0.0, &mut locked);
            assert_eq!(locked[0].color, p.primary.over(p.surface_container_high, 0.45));
        }

        #[test]
        fn the_form_knows_where_everything_lives() {
            let p = palette();
            let mut e = TextEngine::new();
            let rows = [
                FormRow { title: "Funknetz".into(), description: None, on: true, locked: false },
                FormRow {
                    title: "Bluetooth".into(),
                    description: Some("Sichtbar als Matrix".into()),
                    on: false,
                    locked: false,
                },
            ];
            let f = form_section(&mut e, "Verbindungen", &rows, 340.0, p);
            let mut out = Vec::new();
            let (w, _) = lay_out(&f.node, 0.0, 0.0, &mut out);
            // The bookkeeping IS the contract: count = laying.
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&f.node));
            assert!((w - 340.0).abs() < 0.01, "der Abschnitt nimmt seine Breite");
            // Every label slot points to a text measure (transparent).
            assert_eq!(f.labels.len(), f.label_slots.len());
            for slot in &f.label_slots {
                assert_eq!(out[*slot].color.a, 0.0);
            }
            // The switch tracks carry the state of their row.
            assert_eq!(f.toggle_slots.len(), 2);
            assert_eq!(out[f.toggle_slots[0]].color, p.primary);
            assert_eq!(
                out[f.toggle_slots[1]].color,
                p.on_surface.over(p.surface_container_high, 0.12)
            );
            // A hairline with M indent separates the rows.
            assert!(out.iter().any(|q| q.h == 1.0 && q.x == mk::spacing::M));
            // Row title and switch share the centre.
            let title_q = out[f.label_slots[1]];
            let track = out[f.toggle_slots[0]];
            let mid_text = title_q.y + title_q.h / 2.0;
            let mid_track = track.y + track.h / 2.0;
            assert!((mid_text - mid_track).abs() < 0.75, "die Zeile zentriert vertikal");
            // Lager 50b: the switch sits at the inner edge — laid
            // (Align::End in the box), not computed via a spacer from
            // four sizes.
            assert!(
                (track.x + track.w - (340.0 - mk::spacing::M)).abs() < 0.01,
                "der Schalter endet an der Innenkante"
            );
            assert!((title_q.x - mk::spacing::M).abs() < 0.01, "der Titel beginnt am Einzug");
        }

        #[test]
        fn the_magnifier_grows_in_place_and_takes_the_type_along() {
            let p = palette();
            let mut e = TextEngine::new();
            let b = Button { label: "Sichern".into(), ..Default::default() };
            let (node, label) = button(&mut e, &b, Status::Idle, p);
            let mut rest = Vec::new();
            let (rw, rh) = lay_out(&node, 0.0, 0.0, &mut rest);
            let px_before = label.px;

            let (node, label) = button(&mut e, &b, Status::Hovered, p);
            let mut labels = vec![label];
            let big = magnify(&mut e, node, &mut labels, MAGNIFIER, p);
            let mut out = Vec::new();
            let (w, h) = lay_out(&big, 0.0, 0.0, &mut out);
            // The box stays: the magnifier grows in place.
            assert!((w - rw).abs() < 0.01 && (h - rh).abs() < 0.01, "der Platz bleibt derselbe");
            // The ground is larger than the resting one — and centred.
            assert!(out[0].w > rest[0].w, "der Knopf wird groesser");
            assert!((out[0].w - rest[0].w * MAGNIFIER).abs() < 0.01);
            assert!(((out[0].x + out[0].w / 2.0) - rw / 2.0).abs() < 0.01, "die Mitte bleibt die Mitte");
            assert!(out[0].x < 0.01, "er waechst nach beiden Seiten");
            // And the TEXT grows along — otherwise the picture does not
            // match what was measured.
            assert!((labels[0].px - px_before * MAGNIFIER).abs() < 0.001);
        }

        #[test]
        fn the_panel_and_the_console_ring_concentric() {
            let p = palette();
            let mut e = TextEngine::new();
            // The panel shell: border outside, ground concentric inside,
            // radius = inner radius + distance to the edge.
            let mut out = Vec::new();
            let child = Node::Quad { w: 100.0, h: 20.0, color: p.primary, radius: 0.0 };
            let (w, _) = lay_out(&panel_shell(child, 240.0, p), 0.0, 0.0, &mut out);
            assert!((w - 240.0).abs() < 0.01, "die Hülle nimmt ihre Breite");
            let outer = mk::radius::SMALL + mk::spacing::S;
            assert_eq!(out[0].radius, outer);
            assert_eq!(out[1].radius, mk::radius::inner(outer, 1.0));
            assert!((out[1].w - 238.0).abs() < 0.01, "der Grund sitzt im Rand");
            assert_eq!(out[0].color, p.outline.with_alpha(0.25));
            // The console speaks mono in the second text level.
            let (node, label) = console(&mut e, "matrix build --offline", 300.0, p);
            let mut c = Vec::new();
            let (cw, _) = lay_out(&node, 0.0, 0.0, &mut c);
            assert!((cw - 300.0).abs() < 0.01);
            assert_eq!(c[0].radius, mk::radius::NORMAL);
            assert_eq!(c[1].color, p.surface, "Ausgabe wohnt auf surface");
            assert_eq!(label.family, crate::mono_family());
            assert_eq!(label.color, p.text_level(2));
            assert_eq!(label.px, mk::font_size::SMALL);
        }

        #[test]
        fn the_ghost_shows_the_place_beside_the_hand() {
            let p = palette();
            // Short content: nothing to scroll, and the thumb is full.
            assert_eq!(roll_max(100.0, 300.0), 0.0);
            assert_eq!(roll_portion(100.0, 300.0), 1.0);
            assert!((roll_max(900.0, 300.0) - 600.0).abs() < 0.01);
            assert!((roll_portion(900.0, 300.0) - 1.0 / 3.0).abs() < 0.01);
            // The ghost: track whole, thumb proportional, at the top at the start.
            let mut out = Vec::new();
            let (w, h) = lay_out(&scroll_ghost(1.0 / 3.0, 0.0, 1.0, p), 0.0, 0.0, &mut out);
            assert_eq!((w, h), (GHOST_W, GHOST_H));
            assert_eq!(out[0].h, GHOST_H);
            assert!((out[1].h - GHOST_H / 3.0).abs() < 0.01);
            assert_eq!(out[1].y, 0.0, "ganz oben heißt ganz oben");
            // At the end the thumb sits at the bottom of the track.
            let mut end = Vec::new();
            lay_out(&scroll_ghost(1.0 / 3.0, 1.0, 1.0, p), 0.0, 0.0, &mut end);
            assert!(((end[1].y + end[1].h) - GHOST_H).abs() < 0.01);
            // It fades away: half spring = half opacity, both parts.
            let mut faded = Vec::new();
            lay_out(&scroll_ghost(1.0 / 3.0, 0.0, 0.5, p), 0.0, 0.0, &mut faded);
            assert!((faded[1].color.a - p.primary.a * 0.5).abs() < 0.001);
            assert!(faded[0].color.a < out[0].color.a);
            // And it lives near the hand — with edge grammar.
            let (x, y) = ghost_place((400.0, 300.0), (1920.0, 1080.0));
            assert_eq!((x, y), (400.0 + GHOST_GAP, 300.0 + GHOST_GAP));
            let (x, _) = ghost_place((1910.0, 300.0), (1920.0, 1080.0));
            assert!(x < 1910.0, "an der rechten Kante klappt er nach links");
            let (_, y) = ghost_place((400.0, 1070.0), (1920.0, 1080.0));
            assert!(y + GHOST_H + GHOST_MARGIN <= 1080.01, "unten klemmt er hoch");
        }

        #[test]
        fn the_dialog_puts_the_safe_way_first() {
            let p = palette();
            let mut e = TextEngine::new();
            arm_symbols(&mut e);
            let d = confirmation(&mut e, "Wirklich löschen?", "Das lässt sich nicht rückgängig machen.", "Löschen", 1.0, (800.0, 600.0), p);
            let mut out = Vec::new();
            lay_out(&d.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&d.node));
            // The veil covers the whole screen and takes the click.
            let veil = out[d.veil_slot];
            assert_eq!((veil.w, veil.h), (800.0, 600.0));
            assert_eq!(
                matrixkit_theme::scene::hit(&out, 40.0, 40.0),
                Some(d.veil_slot),
                "daneben heißt abbrechen"
            );
            // HIG: cancel on the LEFT and tinted, the danger on the RIGHT
            // and prominent — both equally wide.
            let cancel = out[d.cancel_slot];
            let confirm = out[d.confirm_slot];
            assert!(cancel.x < confirm.x, "der sichere Weg liegt zuerst");
            assert!((cancel.w - confirm.w).abs() < 0.01, "beide teilen die Breite");
            assert_eq!(confirm.color, p.error, "die Gefahr trägt die Fehlerfarbe");
            assert_ne!(cancel.color, p.error);
            // The card is centred on the screen.
            let card = out[2];
            assert!((card.w - DIALOG_WIDTH).abs() < 0.01);
            assert!(((card.x + card.w / 2.0) - 400.0).abs() < 0.01, "waagrecht mittig");
            // The entrance: at half progress the card stands LOWER and
            // the veil is paler.
            let half = confirmation(&mut e, "Wirklich löschen?", "…", "Löschen", 0.5, (800.0, 600.0), p);
            let mut h = Vec::new();
            lay_out(&half.node, 0.0, 0.0, &mut h);
            assert!(h[2].y > card.y, "die Karte fährt von unten herauf");
            // Lager 104: the veil stands IMMEDIATELY (fade = full frame
            // per frame on the CPU) — same coverage at half the run.
            assert!((h[half.veil_slot].color.a - veil.color.a).abs() < 0.001, "der Schleier steht sofort");
            // Tooltip and redaction carry their tokens.
            let (node, label) = tooltip(&mut e, "Schließen", p);
            let mut t = Vec::new();
            lay_out(&node, 0.0, 0.0, &mut t);
            assert_eq!(t[0].radius, mk::radius::SMALL);
            assert_eq!(t[1].color, p.surface_container_high);
            assert_eq!(label.px, mk::font_size::SMALL);
            let mut r = Vec::new();
            lay_out(&redaction(120.0, 12.0, p), 0.0, 0.0, &mut r);
            assert_eq!(r[0].radius, 6.0, "der Platzhalter ist eine Kapsel");
            assert_eq!(r[0].color, p.on_surface.over(p.surface_container, 0.10));
        }

        #[test]
        fn the_level_speaks_its_mute_and_the_bars_hang_from_the_floor() {
            let p = palette();
            // The level: track whole, fill proportional, both capsule.
            let mut out = Vec::new();
            let (w, h) = lay_out(&level_bar(0.4, true, 100.0, p), 0.0, 0.0, &mut out);
            assert_eq!((w, h), (100.0, LEVEL_H));
            assert_eq!(out.len(), 2);
            assert_eq!(out[0].w, 100.0);
            assert!((out[1].w - 40.0).abs() < 0.01);
            assert_eq!(out[1].color, p.primary, "voll spricht primary");
            assert_eq!(out[0].radius, mk::radius::capsule(LEVEL_H));
            // Mute is a signal of its own, not merely a different value.
            let mut muted = Vec::new();
            lay_out(&level_bar(0.4, false, 100.0, p), 0.0, 0.0, &mut muted);
            assert_eq!(muted[1].color, p.on_surface_variant.with_alpha(0.6));
            assert_eq!(muted[1].w, out[1].w, "derselbe Wert, andere Stimme");
            // The bars: equally wide, aligned at the FLOOR, and the
            // highest reaches the top.
            let mut b = Vec::new();
            let (bw, bh) = lay_out(&bars(&[0.25, 1.0, 0.5], 100.0, 40.0, p), 0.0, 0.0, &mut b);
            assert!((bw - 100.0).abs() < 0.01);
            assert_eq!(bh, 40.0);
            assert_eq!(b.len(), 3);
            let floor = b[0].y + b[0].h;
            for q in &b {
                assert!(((q.y + q.h) - floor).abs() < 0.01, "alle stehen auf dem Boden");
            }
            assert!((b[1].h - 40.0).abs() < 0.01, "der volle Balken reicht hinauf");
            assert!((b[0].h - 10.0).abs() < 0.01);
        }

        #[test]
        fn the_small_blocks_wear_their_tokens() {
            let p = palette();
            let mut e = TextEngine::new();
            arm_symbols(&mut e);
            // Badge: capsule, number in the variant text.
            let (node, label) = badge(&mut e, 3, p);
            let mut out = Vec::new();
            let (_, h) = lay_out(&node, 0.0, 0.0, &mut out);
            assert_eq!(out[0].radius, mk::radius::capsule(h), "Kapsel, nicht Ecke");
            assert_eq!(out[0].color, p.on_surface.over(p.surface_container, 0.08));
            assert_eq!(label.color, p.on_surface_variant);
            assert_eq!(label.px, mk::font_size::SMALL);
            // Status capsule: thin fill, second text level.
            let (node, label) = status_capsule(&mut e, "bereit", p);
            let mut out = Vec::new();
            let (_, h) = lay_out(&node, 0.0, 0.0, &mut out);
            assert_eq!(out[0].color, p.fill(mk::FillLevel::Thin));
            assert_eq!(out[0].radius, mk::radius::capsule(h));
            assert_eq!(label.color, p.text_level(2));
            // Card: border outside, fill CONCENTRIC inside.
            let mut out = Vec::new();
            let inner = Node::Quad { w: 40.0, h: 20.0, color: p.primary, radius: 0.0 };
            let (cw, ch) = lay_out(&card(inner, p), 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 3, "Rand, Füllung, Inhalt");
            assert_eq!(out[0].color, p.outline.with_alpha(0.25));
            assert_eq!(out[0].radius, mk::radius::NORMAL);
            assert_eq!(out[1].radius, mk::radius::inner(mk::radius::NORMAL, 1.0));
            assert!((out[1].w - (cw - 2.0)).abs() < 0.01, "die Füllung sitzt im Rand");
            // The outer size is content + 2·M — EXACTLY as with iced,
            // which draws its border inside the box boundary. The ring
            // here lies around the outside, that is why the inner
            // padding is M − 1 (finger probe Matrix Home, 16.08.).
            assert!((ch - (20.0 + 2.0 * mk::spacing::M)).abs() < 0.01, "Aussenmass = Inhalt + 2·M");
            assert!((cw - (40.0 + 2.0 * mk::spacing::M)).abs() < 0.01, "auch in der Breite");
            // And the content sits at M, not at M + 1.
            assert!((out[2].x - mk::spacing::M).abs() < 0.01, "der Inhalt sitzt bei M");
            assert!((out[2].y - mk::spacing::M).abs() < 0.01, "der Inhalt sitzt bei M");
            // Empty state: everything centred, in the reference's
            // order — glyph, title, explanation.
            let (node, labels) = empty_state(&mut e, symbol::FOLDER, "Nichts hier", "Lege etwas ab", 300.0, p);
            let mut out = Vec::new();
            let (w, _) = lay_out(&node, 0.0, 0.0, &mut out);
            assert!((w - 300.0).abs() < 0.01);
            for q in out.iter().filter(|q| q.w > 0.0) {
                let mid = q.x + q.w / 2.0;
                assert!((mid - 150.0).abs() < 0.01, "jede Zeile mittig");
            }
            // Since Lager 98 the labels come as (label, SLOT).
            assert_eq!(labels[0].0.family, crate::symbol_family());
            assert_eq!(labels[0].0.px, 44.0);
            assert_eq!(labels[0].1, 0, "Glyphe auf Platz 0");
            assert_eq!(labels[1].0.px, mk::font_size::LARGE);
            assert_eq!(labels[1].1, 2, "Titel auf Platz 2, nicht auf der Luft");
            assert_eq!(labels[2].0.color, p.on_surface_variant);
        }

        #[test]
        fn a_separator_is_air_and_a_locked_row_is_no_target() {
            let p = palette();
            let mut e = TextEngine::new();
            arm_symbols(&mut e);
            let entries = [
                MenuEntry::Item {
                    glyph: Some(symbol::FOLDER),
                    title: "Öffnen".into(),
                    shortcut: Some("⌘O".into()),
                    badge: None,
                    danger: false,
                    enabled: true,
                },
                MenuEntry::Item {
                    glyph: None,
                    title: "Einfügen".into(),
                    shortcut: None,
                    badge: None,
                    danger: false,
                    enabled: false,
                },
                MenuEntry::Separator,
                MenuEntry::Item {
                    glyph: None,
                    title: "Herunterfahren".into(),
                    shortcut: None,
                    badge: None,
                    danger: true,
                    enabled: true,
                },
            ];
            let m = menu(&mut e, Some("Aufnahme"), &entries, Some(0), p);
            let mut out = Vec::new();
            let (w, _) = lay_out(&m.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&m.node));
            assert!((w - MENU_WIDTH).abs() < 0.01, "die Familien-Norm 230");
            // The separator IS air: two island cards, no line.
            let cards: Vec<_> = out
                .iter()
                .filter(|q| q.radius == mk::CORNER_RADIUS + 6.0)
                .collect();
            // The header is its OWN island above the groups (R122) —
            // with it there are three cards.
            assert_eq!(cards.len(), 3, "Kopf + zwei Gruppen = drei Inseln");
            assert!(
                m.labels.iter().any(|l| l.text == "Aufnahme" && l.px == mk::typo::HEADLINE.px()),
                "die Kopfzeile spricht HEADLINE"
            );
            // All islands are equally wide — a narrower one looks like
            // a mistake (finger probe 16.08.).
            let breiten: Vec<f32> = cards.iter().map(|q| q.w).collect();
            assert!(
                breiten.windows(2).all(|w| (w[0] - w[1]).abs() < 0.01),
                "alle Karten gleich breit: {breiten:?}"
            );
            // Without a header there remain two.
            let ohne = menu(&mut e, None, &entries, None, p);
            let mut oo = Vec::new();
            lay_out(&ohne.node, 0.0, 0.0, &mut oo);
            assert_eq!(
                oo.iter().filter(|q| q.radius == mk::CORNER_RADIUS + 6.0).count(),
                2,
                "ohne Kopf zwei Inseln"
            );
            // And the long title WRAPS instead of sticking out of the
            // card (finger probe 16.08.): no label is wider than the row
            // that carries it.
            let lang = [MenuEntry::Item {
                glyph: Some(symbol::FOLDER),
                title: "Ganzen Bildschirm fotografieren".into(),
                shortcut: None,
                badge: None,
                danger: false,
                enabled: true,
            }];
            let breit = menu(&mut e, None, &lang, None, p);
            let mut bo = Vec::new();
            lay_out(&breit.node, 0.0, 0.0, &mut bo);
            let titel = bo[breit.label_slots[1]];
            assert!(
                titel.x + titel.w <= MENU_WIDTH - mk::spacing::S + 0.01,
                "der Titel bleibt in der Karte ({} > {})",
                titel.x + titel.w,
                MENU_WIDTH - mk::spacing::S
            );
            assert!(titel.h > mk::typo::BODY.px(), "und er tut es, indem er umbricht");
            assert!(out.iter().all(|q| q.h != 1.0), "keine Haarlinie im Menü");
            // One target per input entry — except separators and disabled ones.
            assert_eq!(m.item_slots.len(), entries.len());
            assert!(m.item_slots[0].is_some());
            assert!(m.item_slots[1].is_none(), "gesperrt ist kein Ziel");
            assert!(m.item_slots[2].is_none(), "ein Trenner erst recht nicht");
            assert!(m.item_slots[3].is_some());
            // The hovered row carries the state layer, the resting one
            // nothing — and danger speaks the error colour.
            let hot = out[m.item_slots[0].unwrap()];
            assert_eq!(
                hot.color,
                p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER)
            );
            assert_eq!(out[m.item_slots[3].unwrap()].color.a, 0.0);
            let danger = m.labels.iter().find(|l| l.text == "Herunterfahren").unwrap();
            assert_eq!(danger.color, p.error);
            let locked = m.labels.iter().find(|l| l.text == "Einfügen").unwrap();
            assert_eq!(locked.color, p.outline, "gesperrt bleibt SICHTBAR, nur grau");
            // The shortcut stands on the right at the inner edge.
            let si = m.labels.iter().position(|l| l.text == "⌘O").unwrap();
            let sq = out[m.label_slots[si]];
            // The row is MENU_WIDTH − 2·S wide but sits IN the card's
            // padding: its edge lies at MENU_WIDTH − S.
            let inner_right = MENU_WIDTH - mk::spacing::S;
            assert!(
                (sq.x + sq.w - inner_right).abs() < 0.01,
                "das Kürzel steht an der Innenkante"
            );
        }

        #[test]
        fn the_slider_carries_its_value_and_answers_the_drag() {
            let p = palette();
            let ground = p.surface_container_high;
            let spec = SliderSpec { value: 0.25, width: 200.0, status: Status::Idle, ground };
            let mut out = Vec::new();
            let (w, h) = lay_out(&slider(&spec, p), 0.0, 0.0, &mut out);
            assert_eq!(out.len(), PLACED_PER_SLIDER, "Bahn, Füllung, Knopf, Griff");
            assert_eq!((w, h), (200.0, SLIDER_GRIP));
            // The fill reaches the value, the track is fully there.
            assert!((out[0].w - 200.0).abs() < 0.01);
            assert!((out[1].w - 50.0).abs() < 0.01, "25 % von 200");
            assert_eq!(out[1].color, p.primary);
            // The knob sits WITH ITS CENTRE on the value.
            let knob = out[2];
            assert!(((knob.x + knob.w / 2.0) - 50.0).abs() < 0.01);
            assert_eq!(knob.w, 18.0);
            assert_eq!(knob.radius, mk::radius::capsule(18.0));
            // The handle lies on top and takes the hit — even far beside
            // the 4 px track.
            assert_eq!(
                matrixkit_theme::scene::hit(&out, 120.0, 2.0),
                Some(3),
                "der Griff gewinnt"
            );
            assert_eq!(out[3].color.a, 0.0, "und ist unsichtbar");
            // Two-channel: while dragging the knob grows AND the
            // remaining track brightens (iced grammar that both hand
            // builds lacked).
            let dragged =
                SliderSpec { value: 0.25, width: 200.0, status: Status::Pressed, ground };
            let mut d = Vec::new();
            lay_out(&slider(&dragged, p), 0.0, 0.0, &mut d);
            assert_eq!(d[2].w, 20.0, "der Griff wächst");
            assert_ne!(d[0].color, out[0].color, "und die Rest-Bahn hellt auf");
            // At the ends the knob stays fully on the track.
            for v in [0.0, 1.0] {
                let mut e2 = Vec::new();
                let s = SliderSpec { value: v, width: 200.0, status: Status::Idle, ground };
                lay_out(&slider(&s, p), 0.0, 0.0, &mut e2);
                assert!(e2[2].x >= -0.01 && e2[2].x + e2[2].w <= 200.01, "Knopf bleibt drin");
            }
            // And the ONE clamping that both apps carried themselves.
            assert!((slider_at(50.0, 0.0, 200.0) - 0.25).abs() < 0.001);
            assert_eq!(slider_at(-30.0, 0.0, 200.0), 0.0);
            assert_eq!(slider_at(400.0, 0.0, 200.0), 1.0);
        }

        #[test]
        fn the_list_makes_every_row_a_target() {
            let p = palette();
            let mut e = TextEngine::new();
            arm_symbols(&mut e);
            let rows = [
                ListRow {
                    title: "Bilder".into(),
                    description: None,
                    leading: Some(symbol::FOLDER),
                    trailing: Trailing::Chevron,
                },
                ListRow {
                    title: "Speicher".into(),
                    description: Some("237 GB frei".into()),
                    leading: None,
                    trailing: Trailing::Value("1 TB".into()),
                },
                ListRow {
                    title: "Sichtbar".into(),
                    description: None,
                    leading: None,
                    trailing: Trailing::Toggle { on: true, locked: false },
                },
            ];
            let l = list(&mut e, "Orte", &rows, 340.0, p);
            let mut out = Vec::new();
            let (w, _) = lay_out(&l.node, 0.0, 0.0, &mut out);
            // The count IS the contract.
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&l.node));
            assert!((w - 340.0).abs() < 0.01, "die Liste nimmt ihre Breite");
            // Every row carries its own hit target: full width,
            // invisible — and a tap anywhere in it hits IT.
            assert_eq!(l.row_slots.len(), 3);
            for slot in &l.row_slots {
                let q = out[*slot];
                assert_eq!(q.color.a, 0.0, "der Zeilengrund ist unsichtbar");
                assert!((q.w - 340.0).abs() < 0.01, "und doch volle Breite");
            }
            let first = out[l.row_slots[0]];
            assert_eq!(
                matrixkit_theme::scene::hit(&out, 300.0, first.y + first.h / 2.0),
                Some(l.row_slots[0]),
                "weit rechts in der Zeile trifft die ZEILE"
            );
            // The trailer sits at the inner edge — laid, not computed:
            // chevron, value and switch track alike.
            let inner_right = 340.0 - mk::spacing::M;
            let ci = l
                .labels
                .iter()
                .position(|x| x.text == symbol::CHEVRON_RIGHT.to_string())
                .expect("die Chevron-Zeile trägt ihr Zeichen");
            let chevron = out[l.label_slots[ci]];
            assert!((chevron.x + chevron.w - inner_right).abs() < 0.01);
            let track = out[l.toggle_slots[0]];
            assert!((track.x + track.w - inner_right).abs() < 0.01);
            assert_eq!(track.color, p.primary, "der Schalter trägt seinen Zustand");
            // The leading symbol speaks the symbol font, the title the
            // house font — and the subline the variant text.
            assert_eq!(l.labels[1].family, crate::symbol_family());
            assert_eq!(l.labels[2].family, matrixkit_text::FAMILY);
            assert_eq!(l.labels[2].color, p.on_surface);
            let sub = l.labels.iter().find(|x| x.text == "237 GB frei").expect("Unterzeile");
            assert_eq!(sub.color, p.on_surface_variant);
            assert_eq!(sub.px, mk::typo::CAPTION.px());
        }

        #[test]
        fn what_floats_casts_a_shadow_and_owns_the_damage() {
            let p = palette();
            let mut e = TextEngine::new();
            let (node, _) = button(&mut e, &Button { label: "Sichern".into(), ..Default::default() }, Status::Idle, p);
            let mut out = Vec::new();
            lay_out(&node, 40.0, 40.0, &mut out);
            // Without elevation: the hull is the quad plus smoothing.
            let bare = matrixkit_theme::scene::Rect::of(&out[0]);
            assert!((bare.h - (out[0].h + 2.0)).abs() < 0.01);
            // With elevation: the shadow belongs to the damage, otherwise
            // it would stay behind in the diff and smear.
            lift(&mut out, 0, floating_shadow());
            let lifted = matrixkit_theme::scene::Rect::of(&out[0]);
            assert!(lifted.h > bare.h + 50.0, "die Huelle waechst um die Weichheit");
            assert!(lifted.y < bare.y, "auch nach oben — die Weichheit ist rundum");
            assert!(
                lifted.y + lifted.h > bare.y + bare.h + 30.0,
                "und nach unten um Versatz plus Weichheit"
            );
            // The tokens are those of the iced version.
            assert_eq!(card_shadow().dy, 1.0);
            assert_eq!(card_shadow().blur, 5.0);
            assert_eq!(floating_shadow().dy, 10.0);
            assert_eq!(floating_shadow().blur, 30.0);
        }

        #[test]
        fn the_editor_keeps_its_column_across_short_lines() {
            use matrixkit_shell::Key;
            let mut v = String::from("lang genug\nkurz\nwieder lang genug");
            let mut st = EditorState::default();
            // On the first line at column 9.
            st.cursor = 9;
            editor_edit(&mut v, &mut st, &Key::Down, false);
            let (_, _, col) = line_at(&v, st.cursor);
            assert_eq!(col, 4, "die kurze Zeile ist zu Ende");
            // DECISIVE: the second jump finds column 9 again.
            editor_edit(&mut v, &mut st, &Key::Down, false);
            let (_, _, col) = line_at(&v, st.cursor);
            assert_eq!(col, 9, "die Wunsch-Spalte ueberlebt die kurze Zeile");
            // A sideways movement gives up the wish.
            editor_edit(&mut v, &mut st, &Key::Left, false);
            assert!(st.wish.is_none());
        }

        #[test]
        fn enter_splits_the_line_and_shift_selects_across_it() {
            use matrixkit_shell::Key;
            let mut v = String::from("abcdef");
            let mut st = EditorState { cursor: 3, ..Default::default() };
            editor_edit(&mut v, &mut st, &Key::Enter, false);
            assert_eq!(v, "abc\ndef", "Enter teilt, es schickt nicht ab");
            assert_eq!(st.cursor, 4);
            // Home and End apply to the LINE.
            editor_edit(&mut v, &mut st, &Key::End, false);
            assert_eq!(st.cursor, 7);
            editor_edit(&mut v, &mut st, &Key::Home, false);
            assert_eq!(st.cursor, 4);
            // Selection ACROSS the line boundary: from column 0 of the
            // second line Shift+Up leads to the start of the first —
            // selected is "abc\n", and exactly that falls away.
            editor_edit(&mut v, &mut st, &Key::Up, true);
            assert_eq!(st.anchor, Some(4), "Shift spannt vom Ausgangspunkt");
            assert_eq!(st.cursor, 0, "und zwar bis zum Zeilenanfang darueber");
            editor_edit(&mut v, &mut st, &Key::Backspace, false);
            assert_eq!(v, "def", "die Auswahl faellt ganz, samt Umbruch");
            assert!(st.anchor.is_none());

            // And the narrow case: ONLY the line break, then the two
            // lines are one again.
            let mut v = String::from("abc\ndef");
            let mut st = EditorState { cursor: 4, ..Default::default() };
            editor_edit(&mut v, &mut st, &Key::Left, true);
            editor_edit(&mut v, &mut st, &Key::Backspace, false);
            assert_eq!(v, "abcdef", "der Umbruch faellt, die Zeilen wachsen zusammen");
            assert_eq!(st.cursor, 3);
        }

        #[test]
        fn the_select_row_is_a_trigger_that_wears_its_value() {
            let p = palette();
            let mut e = TextEngine::new();
            arm_symbols(&mut e);
            let rows = [ListRow {
                title: "Erscheinungsbild".into(),
                description: None,
                leading: None,
                trailing: Trailing::Select { value: "Automatisch".into(), hovered: true },
            }];
            let l = list(&mut e, "", &rows, 340.0, p);
            let mut out = Vec::new();
            lay_out(&l.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&l.node));
            // The trigger is a target of its own — not the row.
            assert_eq!(l.select_slots.len(), 1);
            let trigger = out[l.select_slots[0]];
            assert_eq!(trigger.radius, mk::radius::SMALL);
            assert_eq!(
                trigger.color,
                p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER),
                "schwebend zeigt er seine Zustandsschicht"
            );
            // It sits at the inner edge, and ⇕ stands to its right.
            assert!((trigger.x + trigger.w - (340.0 - mk::spacing::M)).abs() < 0.01);
            let ui = l.labels.iter().position(|x| x.text == symbol::UNFOLD_MORE.to_string()).unwrap();
            let vi = l.labels.iter().position(|x| x.text == "Automatisch").unwrap();
            assert!(out[l.label_slots[vi]].x < out[l.label_slots[ui]].x, "erst der Wert, dann ⇕");
            assert_eq!(l.labels[ui].family, crate::symbol_family());
            // At rest it carries no ground.
            let calm = [ListRow {
                title: "Erscheinungsbild".into(),
                description: None,
                leading: None,
                trailing: Trailing::Select { value: "Automatisch".into(), hovered: false },
            }];
            let c = list(&mut e, "", &calm, 340.0, p);
            let mut co = Vec::new();
            lay_out(&c.node, 0.0, 0.0, &mut co);
            assert_eq!(co[c.select_slots[0]].color.a, 0.0);
        }

        /// The header must end STRAIGHT at the bottom (Lager 90).
        ///
        /// A quad has one radius and thus four equal corners. But the
        /// bar sits at the top of its container, and the window edge
        /// continues below it — if it pulls inward there, the side
        /// jumps in and straight back out. Measured, that was up to 9
        /// logical pixels.
        /// The bar's MAGNIFIER (Lager 91): the pointer on a traffic
        /// light must make it GROW — and in place, without shifting
        /// the arrangement.
        #[test]
        fn the_pointer_magnifies_the_light_it_rests_on() {
            let p = palette();
            let mut e = TextEngine::new();
            let mut zoom = HeaderMotion::new();
            let h = frame_header(&mut e, "Erstes Licht", HeaderState::calm(true, false, false), 840.0, p);
            let mut out = Vec::new();
            lay_out(&h.node, 0.0, 0.0, &mut out);
            zoom.learn(&h, &out, 0.0, 0.0);

            let first = out[h.light_slots[0]];
            let (cx, cy) = (first.x + first.w / 2.0, first.y + first.h / 2.0);
            assert!(zoom.point(cx as f64, cy as f64), "der Zeiger auf der Ampel ist eine Aenderung");
            assert!(zoom.lights_hover(), "und ALLE drei zeigen dann ihre Glyphe");
            assert!(zoom.running(), "die Feder laeuft los");

            // A few frame beats, then it must have grown.
            for _ in 0..60 {
                zoom.tick(1.0 / 60.0);
            }
            let f = zoom.factors();
            assert!(f[0] > 1.10, "die Ampel unterm Zeiger waechst: {}", f[0]);
            assert!(f[1] < 1.01 && f[2] < 1.01, "die anderen ruhen: {:?}", f);

            // And the box stays where it was — only the content grows.
            let mut grown = Vec::new();
            let state = HeaderState { active: true, lights_hover: true, rolled_up: false, magnify: f };
            let h2 = frame_header(&mut e, "Erstes Licht", state, 840.0, p);
            lay_out(&h2.node, 0.0, 0.0, &mut grown);
            let big = grown[h2.light_slots[0]];
            assert!(big.w > first.w + 1.0, "die Ampel ist groesser: {} → {}", first.w, big.w);
            let (bx, by) = (big.x + big.w / 2.0, big.y + big.h / 2.0);
            assert!((bx - cx).abs() < 0.01 && (by - cy).abs() < 0.01,
                    "und sie waechst um ihre MITTE, verschiebt also nichts");
            // The second traffic light must not have wandered along.
            let zwei_ruhe = out[h.light_slots[1]];
            let zwei_gross = grown[h2.light_slots[1]];
            assert!((zwei_ruhe.x - zwei_gross.x).abs() < 0.01, "die Nachbarin bleibt stehen");
        }

        /// The pointer BESIDE the bar leaves everything at rest.
        #[test]
        fn the_pointer_elsewhere_leaves_the_header_calm() {
            let p = palette();
            let mut e = TextEngine::new();
            let mut zoom = HeaderMotion::new();
            let h = frame_header(&mut e, "Erstes Licht", HeaderState::calm(true, false, false), 840.0, p);
            let mut out = Vec::new();
            let (_, hh) = lay_out(&h.node, 0.0, 0.0, &mut out);
            zoom.learn(&h, &out, 0.0, 0.0);
            assert!(!zoom.point(400.0, (hh + 200.0) as f64), "weit unterhalb: keine Aenderung");
            assert!(!zoom.lights_hover());
            assert_eq!(zoom.factors(), [1.0; 5]);
        }

        #[test]
        fn the_header_ends_straight_at_the_bottom() {
            let p = palette();
            let mut e = TextEngine::new();
            let state = HeaderState::calm(true, false, false);
            let h = frame_header(&mut e, "Erstes Licht", state, 456.0, p);
            let mut out = Vec::new();
            let (w, hh) = lay_out(&h.node, 0.0, 0.0, &mut out);
            // On the bottom edge there must lie a quad that has the FULL
            // width and is NOT rounded.
            let base_strip = out
                .iter()
                .find(|q| {
                    q.radius == 0.0
                        && (q.w - w).abs() < 0.01
                        && ((q.y + q.h) - hh).abs() < 0.01
                })
                .expect("kein gerader Streifen an der Unterkante");
            assert!(
                base_strip.h >= mk::CORNER_RADIUS - 0.01,
                "der Streifen muss die ganze Rundung abdecken: {} < {}",
                base_strip.h,
                mk::CORNER_RADIUS
            );
            assert_eq!(base_strip.x, 0.0, "und von Kante zu Kante laufen");
            // At the top it may still be round — only not at the bottom.
            let rounded = out[0];
            assert!(rounded.radius > 0.0, "die obere Rundung bleibt");
        }

        /// The straight strip lies OVER the rounded ground. That is only
        /// invisible as long as the colour is opaque — otherwise two
        /// half-coverages add up to a dark band.
        #[test]
        fn header_ground_is_opaque() {
            let p = palette();
            assert!(
                p.surface_container_high.a >= 0.999,
                "der Kopfgrund ist durchscheinend geworden ({}): der \
                 gerade Streifen von Lager 90 wuerde sich als Band \
                 abzeichnen — dann braucht der Rasterer echte \
                 Einzelecken statt zweier Vierecke",
                p.surface_container_high.a
            );
        }

        #[test]
        fn the_header_wears_its_lights() {
            let p = palette();
            let mut e = TextEngine::new();
            let state = HeaderState::calm(true, false, false);
            let h = frame_header(&mut e, "Erstes Licht", state, 456.0, p);
            let mut out = Vec::new();
            let (w, hh) = lay_out(&h.node, 0.0, 0.0, &mut out);
            // Lager 50: ground + three traffic lights + title measure.
            // The three spacer quads and the 64 counterweight have been
            // dropped — the box lays what they computed.
            // Ground, three traffic lights, title measure — and the
            // WINDOW BUTTON on the right, which the light path lacked
            // entirely (16.08.2026). Since Lager 90 the bar carries a
            // second ground: the STRAIGHT strip at its bottom edge.
            assert_eq!(out.len(), 7);
            let shade = out[h.shade_slot];
            assert!(
                shade.x + shade.w <= 456.0 - mk::spacing::M + 0.01,
                "die Taste sitzt INNERHALB der Leiste"
            );
            assert!(
                shade.x > 456.0 / 2.0,
                "und rechts, nicht links"
            );
            assert!(!h.shade.text.is_empty(), "sie traegt ihre Glyphe");
            assert!((w - 456.0).abs() < 0.01);
            // The bar is driven by the CONTENT as in the reference
            // (title button + padding M), not stretched to a fixed
            // height — only that way does it grow along with the font
            // size setting (finger probe 16.08.2026). It reports its
            // height itself; whoever builds below takes this number.
            assert!((hh - h.height).abs() < 0.01, "die Leiste meldet ihre eigene Höhe");
            let (tw, th) = e.measure_styled("Erstes Licht", mk::typo::SUBTITLE.px(), mk::typo::SUBTITLE.weight);
            let _ = tw;
            let core = (th + 4.0).max(LIGHT);
            assert!((hh - (core + 2.0 * mk::spacing::M)).abs() < 0.01, "Inhalt + 2·M");
            // Drei Ampeln in Leitbild-Farben, als Kapseln.
            let tints = [p.error, p.secondary, p.primary];
            for (slot, tint) in h.light_slots.iter().zip(tints) {
                let q = out[*slot];
                assert_eq!((q.w, q.h), (LIGHT, LIGHT));
                assert_eq!(q.radius, mk::radius::capsule(LIGHT));
                assert_eq!(q.color, tint);
            }
            // The title sits CENTRED in the total width.
            let t = out[h.title_slot];
            assert!(((t.x + t.w / 2.0) - 228.0).abs() < 1.0, "Titel mittig");
            // And the traffic lights sit flush at the left inner edge —
            // Align::Start, not computed.
            // The inner edge is L, not M — the reference pads its bar
            // .padding([M, L]) (16.08.2026).
            assert!((out[h.light_slots[0]].x - mk::spacing::L).abs() < 0.01,
                "die erste Ampel am Innenrand");
            assert_eq!(h.title.color, p.on_surface);
            assert!(h.glyphs.is_empty(), "ohne Hover keine Glyphen");
        }

        #[test]
        fn the_header_dims_when_the_window_sleeps() {
            // appearsActive: inactive = uniformly grey traffic lights,
            // title in variant text — the eye knows at once who has the
            // input.
            let p = palette();
            let mut e = TextEngine::new();
            let state = HeaderState::calm(false, false, false);
            let h = frame_header(&mut e, "Erstes Licht", state, 456.0, p);
            let mut out = Vec::new();
            lay_out(&h.node, 0.0, 0.0, &mut out);
            let gray = p.outline.over(p.surface_container_high, 0.6);
            for slot in h.light_slots {
                assert_eq!(out[slot].color, gray);
            }
            assert_eq!(h.title.color, p.on_surface_variant);
        }

        #[test]
        fn hover_summons_all_three_glyphs() {
            let p = palette();
            let mut e = TextEngine::new();
            let state = HeaderState::calm(true, true, false);
            let h = frame_header(&mut e, "Erstes Licht", state, 456.0, p);
            assert_eq!(h.glyphs.len(), 3, "die Gruppe zeigt ALLE Glyphen");
            for (g, _, _) in &h.glyphs {
                assert_eq!(g.family, crate::symbol_family());
                assert_eq!(g.px, 10.5);
                assert!(g.color.a < 1.0, "dunkel AUF der Ampel, nie deckend");
            }
        }

        #[test]
        fn the_peek_floats_and_the_name_leads_centered() {
            let p = palette();
            let mut e = TextEngine::new();
            let sheet = peek(
                &mut e,
                "Erstes Licht",
                "Stufe-3-Probefeld",
                &PeekContent::Glyph(symbol::STAR),
                0.0,
                p,
            );
            let mut out = Vec::new();
            let (w, _) = lay_out(&sheet.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&sheet.node));
            assert!((w - PEEK_WIDTH).abs() < 0.01, "die Karte nimmt ihre Leitbild-Breite");
            // Ring + concentric fill, like the tile chrome.
            assert_eq!(out[0].color, p.outline.with_alpha(0.3));
            assert_eq!(out[0].radius, mk::radius::NORMAL);
            assert_eq!(out[1].color, p.surface_container_high);
            // Three labels: hero glyph (symbol font, primary), name
            // (SUBTITLE), subline (CAPTION, variant text).
            assert_eq!(sheet.labels.len(), 3);
            assert_eq!(sheet.labels[0].family, crate::symbol_family());
            assert_eq!(sheet.labels[0].px, mk::icon_size::HERO);
            assert_eq!(sheet.labels[0].color, p.primary);
            assert_eq!(sheet.labels[1].weight, mk::typo::SUBTITLE.weight);
            assert_eq!(sheet.labels[2].color, p.on_surface_variant);
            // Name and subline sit centred under the content.
            let card_mid = out[0].x + out[0].w / 2.0;
            for slot in &sheet.label_slots[1..] {
                let q = out[*slot];
                assert!(q.color.a == 0.0, "Label-Slots sind Textmaße");
                assert!(((q.x + q.w / 2.0) - card_mid).abs() < 1.0, "mittig ist mittig");
            }
        }

        #[test]
        fn the_peek_text_head_wears_its_own_card() {
            let p = palette();
            let mut e = TextEngine::new();
            let sheet = peek(
                &mut e,
                "notizen.txt",
                "Text · 3 Zeilen",
                &PeekContent::Text("erste Zeile\nzweite Zeile\ndritte Zeile".into()),
                0.0,
                p,
            );
            let mut out = Vec::new();
            lay_out(&sheet.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&sheet.node));
            // The text head carries its own card (surface_container).
            assert_eq!(out[2].color, p.surface_container);
            assert_eq!(out[2].radius, mk::radius::NORMAL);
            // Three lines measure taller than one.
            let one = e.measure("erste Zeile", mk::typo::CAPTION.px());
            let q = out[sheet.label_slots[0]];
            assert!(q.h > 2.0 * one.1, "der Kopf trägt alle Zeilen");
        }

        #[test]
        fn the_image_peek_lays_its_stage() {
            // Lager 15: the family lays the box (contain stage), the
            // atlas fetches and blits — separation as with text: whoever
            // measures, lays; whoever has laid, paints.
            let p = palette();
            let mut e = TextEngine::new();
            let sheet = peek(
                &mut e,
                "wand.jpg",
                "Bild · aus dem Atlas",
                &PeekContent::Image(std::path::PathBuf::from("/tmp/wand.jpg")),
                0.0,
                p,
            );
            let (slot, path) = sheet.image.as_ref().expect("Bild-Inhalt trägt den Kasten");
            assert_eq!(path, &std::path::PathBuf::from("/tmp/wand.jpg"));
            let mut out = Vec::new();
            lay_out(&sheet.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&sheet.node));
            let q = out[*slot];
            assert_eq!((q.w, q.h), (PEEK_WIDTH - 2.0 * mk::spacing::L, 280.0));
            assert_eq!(q.color.a, 0.0, "die Bühne ist durchsichtig — das Bild malt der Atlas");
            assert!(sheet.roll.is_none());
        }

        #[test]
        fn tall_content_rolls_behind_a_thumb() {
            // Lager 14: the viewport is fixed, the thumb honestly
            // proportional — and it wanders with the offset.
            let p = palette();
            let mut e = TextEngine::new();
            let long: String =
                (1..=30).map(|i| format!("Zeile {i}\n")).collect();
            let sheet = peek(&mut e, "log.txt", "30 Zeilen", &PeekContent::Text(long.clone()), 0.0, p);
            let roll = sheet.roll.as_ref().expect("hoher Inhalt rollt");
            assert_eq!(roll.view_h, ROLL_VIEW);
            assert!(roll.content_h > ROLL_VIEW);
            let mut out = Vec::new();
            lay_out(&sheet.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&sheet.node));
            let vq = out[roll.viewport_slot];
            assert_eq!(vq.h, ROLL_VIEW, "der Kasten haelt seine Hoehe");
            // Thumb: proportionally small, at the top at offset 0 …
            let thumb0 = out[roll.viewport_slot + 2];
            assert!(thumb0.h < ROLL_VIEW / 2.0);
            assert_eq!(thumb0.radius, mk::radius::capsule(4.0));
            // … and at the bottom at full offset.
            let full = roll.content_h - roll.view_h;
            let sheet2 = peek(&mut e, "log.txt", "30 Zeilen", &PeekContent::Text(long), full, p);
            let mut out2 = Vec::new();
            lay_out(&sheet2.node, 0.0, 0.0, &mut out2);
            let thumb1 = out2[roll.viewport_slot + 2];
            assert!(thumb1.y > thumb0.y, "der Daumen wandert mit");
            assert!((thumb1.y + thumb1.h) - (vq.y + vq.h) < 0.75, "voll = am Boden");
        }

        #[test]
        fn the_notice_speaks_in_one_tint() {
            // R47: text and ground from the same ink — primary for
            // succeeded, error for failed.
            let p = palette();
            let mut e = TextEngine::new();
            let (node, label) = notice(&mut e, true, "Gespeichert", 340.0, p);
            let mut out = Vec::new();
            let (w, _) = lay_out(&node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), PLACED_PER_NOTICE);
            assert!((w - 340.0).abs() < 0.01, "das Banner nimmt die volle Breite");
            assert_eq!(out[0].color, p.primary.over(p.surface_container, 0.12));
            assert_eq!(out[0].radius, mk::radius::SMALL);
            assert_eq!(label.color, p.primary);
            let (node, label) = notice(&mut e, false, "Fehlgeschlagen", 340.0, p);
            let mut out = Vec::new();
            lay_out(&node, 0.0, 0.0, &mut out);
            assert_eq!(out[0].color, p.error.over(p.surface_container, 0.12));
            assert_eq!(label.color, p.error);
            // Text sits with S air in the banner.
            assert_eq!((out[1].x, out[1].y), (mk::spacing::S, mk::spacing::S));
        }

        #[test]
        fn the_footer_stays_quiet_below() {
            let p = palette();
            let mut e = TextEngine::new();
            let (node, label) = footer(&mut e, "Bereit", p);
            let mut out = Vec::new();
            lay_out(&node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), PLACED_PER_FOOTER);
            // M air above, then the variant text in CAPTION.
            assert_eq!(out[1].y, mk::spacing::M);
            assert_eq!(label.px, mk::typo::CAPTION.px());
            assert_eq!(label.color, p.on_surface_variant);
        }

        #[test]
        fn the_tool_button_rests_bare_and_dims_when_mute() {
            let p = palette();
            let mut e = TextEngine::new();
            let (node, label) = tool_button(&mut e, symbol::ARROW_BACK, true, Status::Idle, p);
            if let Node::Wrap { background: Some((g, r)), .. } = &node {
                assert_eq!(g.a, 0.0, "ruhend ohne Grund");
                assert_eq!(*r, mk::radius::SMALL);
            } else {
                panic!("ein Werkzeugknopf ist ein Wrap mit Grund");
            }
            assert_eq!(label.color, p.on_surface);
            assert_eq!(label.family, crate::symbol_family(), "Werkzeuge sprechen die Symbolschrift");
            // Muted stays visible, only dimmed (reference chevrons).
            let (_, mute) = tool_button(&mut e, symbol::ARROW_FORWARD, false, Status::Idle, p);
            assert_eq!(mute.color, p.outline);
            // Contact lays the panel layer (0.10).
            let (hot, _) = tool_button(&mut e, symbol::ARROW_BACK, true, Status::Hovered, p);
            if let Node::Wrap { background: Some((g, _)), .. } = hot {
                assert_eq!(g, p.on_surface.over(p.surface_container, 0.10));
            }
            let mut out = Vec::new();
            lay_out(&node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), PLACED_PER_TOOL);
        }

        #[test]
        fn the_progress_fills_its_track() {
            let p = palette();
            let mut out = Vec::new();
            let (w, h) = lay_out(&progress(0.5, 200.0, p), 0.0, 0.0, &mut out);
            assert_eq!(out.len(), PLACED_PER_PROGRESS);
            assert!((w - 200.0).abs() < 0.01);
            assert_eq!(h, 6.0);
            assert_eq!(out[0].radius, mk::radius::MINI);
            assert_eq!(out[1].color, p.primary);
            assert!((out[1].w - 100.0).abs() < 0.01, "halbe Spur = halbe Füllung");
        }

        #[test]
        fn the_history_card_lays_its_stage() {
            let p = palette();
            let mut e = TextEngine::new();
            let card = history_card(&mut e, "Prozessor · Verlauf", 300.0, 90.0, p);
            let mut out = Vec::new();
            lay_out(&card.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 4);
            assert_eq!(out[0].color, p.outline.with_alpha(0.3));
            let bq = out[card.curve_slot];
            assert_eq!((bq.w, bq.h), (300.0, 90.0));
            assert_eq!(bq.color.a, 0.0, "die Bühne ist durchsichtig — die Kurve legt die App");
            assert_eq!(card.label.color, p.on_surface_variant);
        }

        #[test]
        fn the_field_holds_value_caret_and_focus() {
            let p = palette();
            let mut e = TextEngine::new();
            // Focused: 2 px primary border, caret at the prefix.
            let spec = FieldSpec {
                value: "matrix",
                cursor: 3,
                anchor: None,
                focused: true,
                blink: true,
                placeholder: "Suchen",
                leading: None,
            };
            let f = input_field(&mut e, &spec, 200.0, p);
            let mut out = Vec::new();
            let (w, _) = lay_out(&f.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), PLACED_PER_FIELD);
            assert!((w - 200.0).abs() < 0.01);
            assert_eq!(out[0].color, p.primary, "Fokus trägt primary");
            assert_eq!(out[3].w, 0.0, "ohne Anker keine Auswahl");
            let caret = out[5];
            assert_eq!(caret.w, 1.5);
            let (head_w, _) = e.measure_styled("mat", mk::font_size::MEDIUM, mk::typo::Weight::Normal);
            assert!((caret.x - (out[4].x + head_w)).abs() < 0.75, "die Marke sitzt am Präfix");
            assert_eq!(f.label.color, p.on_surface);
            // Blink phase off: the caret disappears, nothing else.
            let f = input_field(&mut e, &FieldSpec { blink: false, ..spec }, 200.0, p);
            let mut out2 = Vec::new();
            lay_out(&f.node, 0.0, 0.0, &mut out2);
            assert_eq!(out2[5].w, 0.0, "Blinkphase aus = Marke aus");
            // Resting: 1 px outline border, NO caret, placeholder.
            let spec = FieldSpec {
                value: "",
                cursor: 0,
                anchor: None,
                focused: false,
                blink: true,
                placeholder: "Suchen",
                leading: None,
            };
            let f = input_field(&mut e, &spec, 200.0, p);
            let mut out = Vec::new();
            lay_out(&f.node, 0.0, 0.0, &mut out);
            assert_eq!(out[0].color, p.outline.over(p.surface_container, 0.35));
            assert_eq!(out[5].w, 0.0, "ohne Fokus keine Marke");
            assert_eq!(f.label.text, "Suchen");
            assert_eq!(f.label.color, p.on_surface_variant);
        }

        #[test]
        fn the_selection_spans_anchor_to_cursor() {
            // Lager 20: the selection — from the anchor to the caret, measured.
            let p = palette();
            let mut e = TextEngine::new();
            let spec = FieldSpec {
                value: "matrix",
                cursor: 5,
                anchor: Some(2),
                focused: true,
                blink: true,
                placeholder: "",
                leading: None,
            };
            let f = input_field(&mut e, &spec, 200.0, p);
            let mut out = Vec::new();
            lay_out(&f.node, 0.0, 0.0, &mut out);
            let sel = out[3];
            let m = mk::font_size::MEDIUM;
            let w2 = e.measure_styled("ma", m, mk::typo::Weight::Normal).0;
            let w5 = e.measure_styled("matri", m, mk::typo::Weight::Normal).0;
            assert!((sel.w - (w5 - w2)).abs() < 0.75, "die Auswahl misst ihre Zeichen");
            assert!((sel.x - (out[4].x + w2)).abs() < 1.0, "…und beginnt am Anker");
            assert!(sel.color.a > 0.9, "die Auswahl ist deckend gemischt");
        }

        #[test]
        fn the_toolbar_carries_its_search_field() {
            let p = palette();
            let mut e = TextEngine::new();
            let bar = toolbar(
                &mut e,
                "Seite 2",
                (true, Status::Idle),
                (false, Status::Idle),
                &[],
                Some(&FieldSpec {
                    value: "",
                    cursor: 0,
                    anchor: None,
                    focused: false,
                    blink: true,
                    placeholder: "Suchen",
                    leading: None,
                }),
                456.0,
                p,
            );
            let mut out = Vec::new();
            let (w, _) = lay_out(&bar.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&bar.node));
            assert!((w - 456.0).abs() < 0.01);
            let slot = bar.search_slot.expect("die Leiste trägt ihr Feld");
            // The field ends at the edge, reference width.
            let rim = out[slot];
            assert!((rim.x + rim.w - 456.0).abs() < 0.01);
            assert!((rim.w - SEARCH_WIDTH).abs() < 0.01);
        }

        #[test]
        fn the_toolbar_keeps_its_books() {
            let p = palette();
            let mut e = TextEngine::new();
            let bar = toolbar(
                &mut e,
                "Seite 2",
                (true, Status::Idle),
                (false, Status::Idle),
                &[(symbol::STAR_OUTLINE, true, Status::Idle)],
                None,
                456.0,
                p,
            );
            let mut out = Vec::new();
            let (w, _) = lay_out(&bar.node, 0.0, 0.0, &mut out);
            // Count = laying, and the bar takes its width.
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&bar.node));
            assert!((w - 456.0).abs() < 0.01);
            // Three tools (◀ ▶ ★), four labels (+ title).
            assert_eq!(bar.tool_slots.len(), 3);
            assert_eq!(bar.labels.len(), 4);
            assert_eq!(bar.labels.len(), bar.label_slots.len());
            // The title speaks the house font in HEADLINE, the tools the
            // symbol font; the muted ▶ carries outline.
            assert_eq!(bar.labels[2].family, matrixkit_text::FAMILY);
            assert_eq!(bar.labels[2].weight, mk::typo::HEADLINE.weight);
            assert_eq!(bar.labels[1].color, p.outline);
            // All label slots point to text measures (transparent), all
            // tool slots to grounds in radius::SMALL.
            for slot in &bar.label_slots {
                assert_eq!(out[*slot].color.a, 0.0);
            }
            for slot in &bar.tool_slots {
                assert_eq!(out[*slot].radius, mk::radius::SMALL);
            }
            // Title and tools share the centre; the right-hand tool ends
            // at the edge.
            let title_q = out[bar.label_slots[2]];
            let back_q = out[bar.tool_slots[0]];
            assert!(((title_q.y + title_q.h / 2.0) - (back_q.y + back_q.h / 2.0)).abs() < 0.75);
            let star_q = out[bar.tool_slots[2]];
            assert!((star_q.x + star_q.w - 456.0).abs() < 0.01);
        }

        #[test]
        fn borderless_idle_is_pure_text() {
            let p = palette();
            let mut e = TextEngine::new();
            let b = Button {
                label: "Leise".into(),
                style: Style::Borderless,
                role: Role::Normal,
                size: Size::Normal,
                active: true, ..Default::default() };
            let (node, label) = button(&mut e, &b, Status::Idle, p);
            if let Node::Wrap { background: Some((g, _)), .. } = &node {
                assert_eq!(g.a, 0.0, "randlos ruht ohne Grund");
            }
            assert_eq!(label.color, p.primary);
            // Only contact brings the state layer.
            let (hot, _) = button(&mut e, &b, Status::Hovered, p);
            if let Node::Wrap { background: Some((g, _)), .. } = hot {
                assert!(g.a > 0.0, "Berührung zeigt die Schicht");
            }
        }

        /// The section shell: ONE card, hairlines between, and the
        /// reported row starts must be right — otherwise the caller
        /// anchors its text into the void.
        #[test]
        fn the_section_wraps_its_rows_in_one_card() {
            let mut e = TextEngine::new();
            let p = mk::Palette::default();
            let bar = |w: f32| Node::Quad { w, h: 30.0, color: p.primary, radius: 0.0 };
            let s = section(&mut e, "KONTEN", vec![bar(100.0), bar(120.0)], 200.0, p);
            let mut out = Vec::new();
            lay_out(&s.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), matrixkit_theme::scene::placed_count(&s.node));
            assert!(s.title.is_some(), "der Titel traegt sein Label");
            assert_eq!(s.rows.len(), 2);
            // Rows 0 and 1 point to their OWN quads.
            assert!((out[s.rows[0]].w - 100.0).abs() < 0.01);
            assert!((out[s.rows[1]].w - 120.0).abs() < 0.01);
            // Exactly ONE card in surface_container_high — no pill per
            // row.
            let cards = out.iter().filter(|q| q.color == p.surface_container_high).count();
            assert_eq!(cards, 1, "eine Karte, nicht eine pro Zeile");
            // Between the rows lies a hairline.
            assert_eq!(out.iter().filter(|q| (q.h - 1.0).abs() < 0.01).count(), 1);
        }

        // --- The remainder list of 09.09.2026 ---------------------------

        #[test]
        fn the_check_marks_only_when_checked() {
            let p = palette();
            let mut e = TextEngine::new();
            let on = check(&mut e, "Bereit", true, p);
            let off = check(&mut e, "Bereit", false, p);
            let mut out = Vec::new();
            lay_out(&on.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), PLACED_PER_CHECK);
            assert_eq!(out[0].color.a, 0.0, "die Zeile ist das durchsichtige Ziel");
            assert_eq!(out[1].color, p.primary, "der Rahmen leuchtet, wenn gesetzt");
            assert!(on.tick.color.a > 0.0 && off.tick.color.a == 0.0, "der Haken erscheint nur gesetzt");
            assert_eq!(on.slot, 0);
        }

        #[test]
        fn the_stepper_targets_sit_around_the_value() {
            let p = palette();
            let mut e = TextEngine::new();
            let s = stepper(&mut e, "12", p);
            let mut out = Vec::new();
            lay_out(&s.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), PLACED_PER_STEPPER);
            assert!(out[s.minus_slot].x < out[2].x && out[2].x < out[s.plus_slot].x, "−, Wert, + von links nach rechts");
            assert_eq!(s.value.text, "12");
        }

        #[test]
        fn the_disclosure_hides_its_body_when_closed() {
            let p = palette();
            let mut e = TextEngine::new();
            let body = || Node::Quad { w: 100.0, h: 40.0, color: palette().primary, radius: 0.0 };
            let open = disclosure(&mut e, "Details", 240.0, true, Some(body()), p);
            let closed = disclosure(&mut e, "Details", 240.0, false, Some(body()), p);
            let mut a = Vec::new();
            let mut b = Vec::new();
            lay_out(&open.node, 0.0, 0.0, &mut a);
            lay_out(&closed.node, 0.0, 0.0, &mut b);
            assert_eq!(a.len(), PLACED_PER_DISCLOSURE_HEAD + 1);
            assert_eq!(b.len(), PLACED_PER_DISCLOSURE_HEAD);
            assert_eq!(open.body_first, Some(PLACED_PER_DISCLOSURE_HEAD));
            assert!((a[0].w - 240.0).abs() < 0.01, "der Kopf reicht über die volle Breite");
            assert_ne!(open.chevron.text, closed.chevron.text, "der Pfeil zeigt den Zustand");
        }

        #[test]
        fn the_breadcrumbs_name_their_links() {
            let p = palette();
            let mut e = TextEngine::new();
            let b = breadcrumbs(&mut e, &["Werft", "Projekte", "Studio"], p);
            let mut out = Vec::new();
            lay_out(&b.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 5, "drei Glieder, zwei Trenner");
            assert_eq!(b.slots, vec![0, 2, 4]);
            assert_eq!(b.labels[4].color, p.on_surface, "das letzte Glied ist der Ort, kein Link");
            assert_eq!(b.labels[0].color, p.primary);
        }

        #[test]
        fn the_pager_marks_the_current_page() {
            let p = palette();
            let mut e = TextEngine::new();
            let g = pager(&mut e, 1, 3, p);
            let mut out = Vec::new();
            lay_out(&g.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 2 + 3 * 2 + 2);
            assert_eq!(g.page_slots, vec![2, 4, 6]);
            assert_eq!(g.next_slot, 8);
            assert_eq!(out[g.page_slots[1]].color, p.primary_container);
            assert_eq!(out[g.page_slots[0]].color, p.surface_container);
            let many = pager(&mut e, 2, 27, p);
            let mut out = Vec::new();
            lay_out(&many.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 5, "ab 13 Seiten ein Zähler statt Chips");
            assert!(many.page_slots.is_empty());
            assert_eq!(many.labels[1].text, "03/27");
        }

        #[test]
        fn the_rail_numbers_in_mono_and_wraps() {
            let p = palette();
            let mut e = TextEngine::new();
            crate::arm_mono(&mut e);
            let r = numbered_rail(&mut e, &["Kanal lesen", "Erwähnungen ziehen, verdichten, an the owner melden — jeden Morgen"], 220.0, p);
            assert_eq!(r.labels[0].text, "01");
            assert_eq!(r.labels[0].family, crate::mono_family());
            let mut out = Vec::new();
            lay_out(&r.node, 0.0, 0.0, &mut out);
            assert!(out.len() > 4, "der lange Eintrag bricht um");
            assert_eq!(out.len(), r.labels.len(), "jedes Label hat sein Mass");
        }

        #[test]
        fn the_ladder_lights_the_active_step() {
            let p = palette();
            let mut e = TextEngine::new();
            let l = ladder(&mut e, &[("Geführt", "Du führst."), ("Assistiert", "Es hilft."), ("Autonom", "Es läuft.")], 1, 260.0, p);
            let mut out = Vec::new();
            lay_out(&l.node, 0.0, 0.0, &mut out);
            assert_eq!(l.slots.len(), 3);
            assert_eq!(out.len(), 3 * 4, "je Stufe Grund, Balken, Titel, eine Zeile");
            assert_eq!(out[l.slots[1] + 1].color, p.primary, "der Balken der aktiven Stufe");
            assert_ne!(out[l.slots[0] + 1].color, p.primary);
            assert_eq!(out[l.slots[2]].color.a, 0.0, "jede Stufe ist ein durchsichtiges Ziel");
        }

        #[test]
        fn the_segments_lift_the_active_one() {
            let p = palette();
            let mut e = TextEngine::new();
            let s = segments(&mut e, &["Tag", "Woche", "Monat"], 2, p);
            let mut out = Vec::new();
            lay_out(&s.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 1 + 3 * 2);
            assert_eq!(s.slots, vec![1, 3, 5]);
            assert_eq!(out[0].color, p.surface_container, "die Spur");
            assert_eq!(out[5].color, p.primary_container);
            assert_eq!(out[1].color.a, 0.0, "ruhende Segmente sind durchsichtige Ziele");
        }

        #[test]
        fn the_table_gives_every_row_a_target() {
            let p = palette();
            let mut e = TextEngine::new();
            let rows = vec![
                vec!["Studio".to_string(), "offen".to_string()],
                vec!["Ein sehr langer Projektname, der niemals in seine Spalte passt".to_string(), "fertig".to_string()],
            ];
            let t = table(&mut e, &[("Projekt", 120.0), ("Stand", 60.0)], &rows, p);
            let mut out = Vec::new();
            lay_out(&t.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 2 + 1 + 2 * 3, "Kopf, Haarlinie, je Zeile Grund + 2 Zellen");
            assert_eq!(t.row_slots, vec![3, 6]);
            assert_eq!(out[3].color.a, 0.0);
            assert!((out[2].h - 1.0).abs() < 0.01, "die Haarlinie");
            assert!(t.labels[2 + 2].text.ends_with('…'), "die lange Zelle wird gekürzt: {}", t.labels[4].text);
            assert!(out[4].x < out[5].x, "Zellen stehen nebeneinander");
        }

        #[test]
        fn the_paragraph_and_tinted_card_measure_up() {
            let p = palette();
            let mut e = TextEngine::new();
            let para = paragraph(&mut e, "Ein Absatz, der an seiner Spalte umbricht, weil er länger ist als sie.", mk::font_size::MEDIUM, mk::typo::Weight::Normal, p.on_surface, 140.0);
            assert!(para.labels.len() >= 2);
            let mut out = Vec::new();
            let (w, _) = lay_out(&card_tinted(para.node, p.primary, p), 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 2 + para.labels.len());
            assert!(w <= 140.0 + 2.0 * mk::spacing::M + 0.5, "die Karte ist so breit wie ihr Absatz plus Polster");
            assert!(out[0].color.a < 1.0 && out[0].color.a > 0.0, "der Rand trägt die Tönung");
        }

        // --- Lager 106: shapes and sidebar ------------------

        #[test]
        fn the_ring_gauge_and_donut_raster_into_pixels() {
            let p = palette();
            let g = ring_gauge(40.0, 6.0, 0.5, p.primary, p.surface_container);
            assert_eq!(g.ops.len(), 2, "Spur und Anteil");
            let (w, h, rgba) = matrixkit_shell::shapes::raster(&g, 80, 80).expect("gerastert");
            assert_eq!((w, h), (80, 80));
            let a = |x: usize, y: usize| rgba[(y * 80 + x) * 4 + 3];
            assert!(a(74, 40) > 200, "rechts liegt der Ring (Anteil 0..0,5 ab 12 Uhr)");
            assert!(a(40, 40) == 0, "die Mitte ist leer");
            let d = donut(40.0, 8.0, &[(1.0, p.primary), (1.0, p.error), (2.0, p.tertiary)]);
            assert_eq!(d.ops.len(), 3);
            assert_ne!(g.key(), d.key(), "verschiedene Formen, verschiedene Schlüssel");
            assert_eq!(g.key(), ring_gauge(40.0, 6.0, 0.5, p.primary, p.surface_container).key(), "gleiche Form, gleicher Schlüssel");
            let empty = donut(40.0, 8.0, &[]);
            assert!(empty.ops.is_empty());
        }

        #[test]
        fn the_grid_and_polyline_build_and_pict_registers() {
            let p = palette();
            let grid = grid_ground(100.0, 50.0, 10.0, p.outline);
            assert_eq!(grid.ops[0].path.len(), 9 * 4, "9 Spalten × 4 Zeilen Punkte");
            let line = polyline(100.0, 20.0, &[(0.0, 10.0), (50.0, 5.0), (100.0, 15.0)], 2.0, p.primary, Some((4.0, 4.0)));
            assert!(matches!(line.ops[0].style, ShapeStyle::Dashed { .. }));
            let pict = pict_of(&line, 5.0, 7.0, Rect { x: 5.0, y: 7.0, w: 100.0, h: 20.0 }, 3);
            assert!(matrixkit_shell::shapes::is_shape_key(&pict.path));
            assert_eq!((pict.box_w, pict.box_h), (100, 20));
            let (_, _, rgba) = matrixkit_shell::shapes::resolve(&pict.path, 100, 20).expect("der Auflöser kennt die Form");
            assert!(rgba.iter().skip(3).step_by(4).any(|a| *a > 0), "es wurde gemalt");
            assert!(matrixkit_shell::shapes::resolve(std::path::Path::new("/tmp/foo.png"), 10, 10).is_none());
        }

        #[test]
        fn the_sidebar_lifts_the_selected_entry() {
            let p = palette();
            let mut e = TextEngine::new();
            let s = sidebar(&mut e, &[('\u{e88a}', "Heute"), ('\u{e8b8}', "Einstellungen"), ('\u{e0b7}', "Rat")], Some(1), 200.0, p);
            let mut out = Vec::new();
            lay_out(&s.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 3 * PLACED_PER_SIDEBAR_ENTRY);
            assert_eq!(s.slots, vec![0, 3, 6]);
            assert_eq!(out[3].color, p.primary_container, "der gewählte Eintrag ist gehoben");
            assert_eq!(out[0].color.a, 0.0, "ruhende Einträge sind durchsichtige Ziele");
            assert!((out[0].w - 200.0).abs() < 0.01, "jeder Eintrag reicht über die volle Breite");
            assert_eq!(s.labels[3].weight, mk::typo::Weight::Semibold);
        }

        // --- Lager 108: drag, tree list ----------------------------

        #[test]
        fn the_drag_lives_only_after_the_slop() {
            let mut d = Drag::begin(7, 10.0, 10.0);
            assert!(!d.travel(12.0, 13.0), "drei Pixel sind noch ein Klick");
            assert_eq!(d.end(Some(3)), None);
            let mut d = Drag::begin(7, 10.0, 10.0);
            assert!(d.travel(30.0, 10.0));
            assert_eq!(d.offset(), (20.0, 0.0));
            assert!(d.travel(31.0, 10.0), "einmal lebendig bleibt er lebendig");
            assert_eq!(d.end(Some(3)), Some((7, 3)));
            assert_eq!(Drag { live: true, ..Drag::begin(1, 0.0, 0.0) }.end(None), None, "ins Leere gezogen");
        }

        #[test]
        fn the_tree_list_indents_and_selects() {
            let p = palette();
            let mut e = TextEngine::new();
            let rows = vec![
                ListRow { title: "Matrix".into(), description: None, leading: Some('\u{e2c7}'), trailing: Trailing::None },
                ListRow { title: "matrixkit".into(), description: None, leading: Some('\u{e2c7}'), trailing: Trailing::None },
                ListRow { title: "apps".into(), description: None, leading: Some('\u{e2c7}'), trailing: Trailing::None },
            ];
            let flat = list(&mut e, "", &rows, 300.0, p);
            let tree = list_with(&mut e, "", &rows, 300.0, &ListOptions { selected: Some(1), depths: &[0, 1, 2] }, p);
            let mut a = Vec::new();
            let mut b = Vec::new();
            lay_out(&flat.node, 0.0, 0.0, &mut a);
            lay_out(&tree.node, 0.0, 0.0, &mut b);
            assert_eq!(b.len(), a.len() + 2, "zwei eingerückte Zeilen tragen je einen Leerlauf");
            assert_eq!(tree.row_slots.len(), 3);
            let title_x = |l: &List, out: &[Placed], i: usize| out[l.label_slots[i * 2 + 1]].x;
            assert!(title_x(&tree, &b, 1) > title_x(&tree, &b, 0) + 10.0, "Stufe 1 rückt ein");
            assert!(title_x(&tree, &b, 2) > title_x(&tree, &b, 1) + 10.0, "Stufe 2 rückt weiter ein");
            assert!(b[tree.row_slots[1]].color.a > 0.0, "die gewählte Zeile ist gehoben");
            assert_eq!(b[tree.row_slots[0]].color.a, 0.0);
        }

        // --- Lager 109: divider, view, swipe ----------------------

        #[test]
        fn the_split_puts_its_divider_between_the_halves() {
            let p = palette();
            let q = |w: f32| Node::Quad { w, h: 10.0, color: palette().primary, radius: 0.0 };
            let s = split(q(50.0), q(50.0), 0.25, 400.0, 200.0, false, p);
            let mut out = Vec::new();
            let (w, h) = lay_out(&s.node, 0.0, 0.0, &mut out);
            assert_eq!((w, h), (400.0, 200.0));
            assert_eq!(s.divider_slot, 1, "links ein Placed, dann der Griff");
            assert!((out[s.divider_slot].x - s.left_w).abs() < 0.01);
            assert_eq!(out[s.divider_slot].w, DIVIDER_W);
            assert!((s.left_w + DIVIDER_W + s.right_w - 400.0).abs() < 0.01);
            assert!((split_ratio(203.0, 400.0) - 0.5077).abs() < 0.01);
            assert_eq!(split_ratio(-50.0, 400.0), 0.15, "geklemmt");
        }

        #[test]
        fn the_viewport_keeps_the_point_under_the_cursor() {
            let p = palette();
            let q = || Node::Quad { w: 100.0, h: 100.0, color: palette().primary, radius: 0.0 };
            let pan = (10.0, 20.0);
            let cursor = (60.0, 70.0);
            let old = 1.0;
            let new = 2.0;
            let pan2 = zoom_at(cursor, pan, old, new);
            let mut a = Vec::new();
            let mut b = Vec::new();
            lay_out(&viewport(q(), 200.0, 200.0, pan, old, p), 0.0, 0.0, &mut a);
            lay_out(&viewport(q(), 200.0, 200.0, pan2, new, p), 0.0, 0.0, &mut b);
            // Model point under the pointer before: (60-10)/1 = 50 → afterwards at 10+50*2? no: pan2 such that 50*2+pan2 = 60
            let model = (cursor.0 - pan.0) / old;
            assert!(((b[1].x + model * new) - cursor.0).abs() < 0.01, "der Punkt unter dem Zeiger bleibt liegen");
            assert_eq!(a[0].w, 200.0, "der Kasten bleibt");
            assert_eq!(b[1].w, 200.0, "das Kind ist doppelt so gross");
        }

        #[test]
        fn the_swipe_reveals_its_actions_from_the_right() {
            let p = palette();
            let mut e = TextEngine::new();
            let row = Node::Quad { w: 300.0, h: 44.0, color: p.surface_container_high, radius: 0.0 };
            let s = swipe(&mut e, row, 300.0, 44.0, 40.0, &[('\u{e872}', p.error), ('\u{e3c9}', p.primary)], p);
            let mut out = Vec::new();
            lay_out(&s.node, 0.0, 0.0, &mut out);
            assert_eq!(s.action_slots, vec![0, 2]);
            assert_eq!(out.len(), 5, "zwei Aktionen à zwei Placed, dann die Zeile");
            assert!((out[2].x + SWIPE_ACTION_W - 300.0).abs() < 0.01, "die letzte Aktion endet an der Kante");
            assert_eq!(out[4].x, -40.0, "die Zeile ist um die Freigabe nach links gewandert");
            assert_eq!(s.max_reveal, 2.0 * SWIPE_ACTION_W);
            let mut d = Drag::begin(0, 200.0, 10.0);
            d.travel(120.0, 10.0);
            assert_eq!(swipe_reveal(&d, s.max_reveal), 80.0);
            assert_eq!(swipe_snap(20.0, s.max_reveal), 0.0);
            assert_eq!(swipe_snap(80.0, s.max_reveal), s.max_reveal);
        }

        #[test]
        fn a_badge_takes_the_shade_keys_place() {
            let p = palette();
            let mut e = TextEngine::new();
            let badge = ExtensionBadge { name: "Hardline", glyph: '\u{e80b}', on: true };
            let h = frame_header_with(&mut e, "Rabbit", HeaderState::calm(true, false, false), 600.0, Some(&badge), p);
            let mut out = Vec::new();
            lay_out(&h.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 7 - 1 + PLACED_PER_BADGE, "Fenstertaste (1) → Abzeichen (4)");
            assert_eq!(h.shade_slot, 6);
            assert_eq!(h.badge_glyph.as_ref().map(|(_, s)| *s), Some(8));
            assert_eq!(h.badge_name.as_ref().map(|(l, s)| (l.text.as_str(), *s)), Some(("Hardline", 9)));
            assert_eq!(out[7].color, p.primary, "der Punkt leuchtet, wenn die Erweiterung läuft");
            let plain = frame_header(&mut e, "Rabbit", HeaderState::calm(true, false, false), 600.0, p);
            let mut out2 = Vec::new();
            lay_out(&plain.node, 0.0, 0.0, &mut out2);
            assert_eq!(out2.len(), 7, "ohne Erweiterung bleibt alles beim Alten");
        }

        #[test]
        fn the_extension_panel_lists_its_actions() {
            let p = palette();
            let mut e = TextEngine::new();
            let lines = [String::from("hardline://abc"), String::from("Regal: 3")];
            let panel = ExtensionPanel { name: "Hardline", glyph: '\u{e80b}', on: false, lines: &lines, actions: &["Knoten starten", "Nochmal prüfen"], hover: Some(1) };
            let b = extension_panel(&mut e, &panel, 600.0, 50.0, 320.0, p);
            assert_eq!(b.placed[0].x, 280.0, "die Karte hängt rechts");
            assert_eq!(b.placed[0].w, 320.0);
            assert!((b.placed[0].h - b.height).abs() < 0.01);
            let actions: Vec<usize> = b.roles.iter().filter_map(|r| *r).collect();
            assert_eq!(actions, vec![0, 0, 1, 1], "je Handlung Schicht + Text");
            assert!(b.labels.iter().any(|(l, _)| l.text == "Knoten starten"));
        }

        #[test]
        fn the_sheet_keeps_the_books() {
            let p = palette();
            let mut e = TextEngine::new();
            let mut b = Sheet::new();
            let (first, n, _, _) = b.node(&Node::Quad { w: 10.0, h: 10.0, color: p.primary, radius: 0.0 }, 5.0, 6.0, Some(7), None);
            assert_eq!((first, n), (0, 1));
            assert_eq!(b.roles, vec![Some(7)]);
            let h = b.paragraph(&mut e, "Ein Absatz", mk::typo::BODY, p.on_surface, 0.0, 20.0, 200.0, None, None);
            assert!(h > 0.0);
            assert_eq!(b.texts.len(), 1);
            assert_eq!(b.texts[0].anchor, 1, "der Absatz ankert auf seiner Zeile");
            let scene = b.into_scene();
            assert_eq!(scene.placed.len(), scene.roles.len());
        }

        #[test]
        fn the_hover_finds_the_role_and_respects_the_ceiling() {
            let p = palette();
            let mut b = Sheet::new();
            b.quad(0.0, 0.0, 50.0, 50.0, p.primary, 0.0, Some(3), None);
            b.quad(100.0, 0.0, 50.0, 50.0, p.primary, 0.0, Some(900), None);
            let mut h = Hover::new(900);
            h.learn(&b.placed, &b.roles);
            assert!(h.point(10.0, 10.0));
            assert!(h.is(Some(3)));
            assert!(!h.point(12.0, 12.0), "gleiche Rolle — keine Änderung");
            assert!(h.point(110.0, 10.0));
            assert_eq!(h.role(), None, "Rollen ab der Decke zählen nicht");
            h.point(10.0, 10.0);
            assert!(h.leave());
            assert_eq!(h.role(), None);
        }

        #[test]
        fn a_link_carries_its_state_layer() {
            let p = palette();
            let mut e = TextEngine::new();
            let (idle, _) = link(&mut e, "Erlauben", 14.0, mk::typo::Weight::Normal, p.primary, Status::Idle, p);
            let (hot, l) = link(&mut e, "Erlauben", 14.0, mk::typo::Weight::Normal, p.primary, Status::Hovered, p);
            let mut a = Vec::new();
            let mut b = Vec::new();
            lay_out(&idle, 0.0, 0.0, &mut a);
            lay_out(&hot, 0.0, 0.0, &mut b);
            assert_eq!(a.len(), PLACED_PER_LINK);
            assert_eq!(b.len(), PLACED_PER_LINK);
            assert_eq!(a[0].color.a, 0.0, "im Ruhezustand ist die Schicht durchsichtig");
            assert!(b[0].color.a > 0.0, "unter dem Zeiger trägt sie Deckung");
            assert_eq!(a[1].x, LINK_PAD.0, "der Text sitzt um den Rand versetzt");
            assert_ne!(l.color, p.primary, "der Schriftzug hellt sich auf");
        }

        #[test]
        fn the_closable_chip_has_its_glyph_slot() {
            let p = palette();
            let mut e = TextEngine::new();
            let t = tab_chip_closable(&mut e, "Neuer Tab", true, p);
            let mut out = Vec::new();
            lay_out(&t.node, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), 3, "Grund, Beschriftung, ×");
            assert_eq!(t.close_slot, 2);
            assert_eq!(t.close.text, "×");
            assert_eq!(out[0].radius, mk::radius::SMALL, "die Form der alten Kapsel");
        }

        #[test]
        fn the_suggestions_hang_under_the_field() {
            let p = palette();
            let mut e = TextEngine::new();
            let field = Rect { x: 40.0, y: 60.0, w: 300.0, h: 30.0 };
            let entries = [Suggestion { label: "Nachtkiosk", origin: "Regal" }, Suggestion { label: "Notiz", origin: "Gespeichert" }];
            let s = suggestions(&mut e, field, &entries, Some(1), p);
            assert_eq!(s.placed.len(), 1 + 3 * entries.len(), "Grund + je Zeile (Ziel, Text, Herkunft)");
            assert_eq!(s.placed[0].x, field.x);
            assert_eq!(s.placed[0].w, field.w, "gleiche Breite wie das Feld");
            assert!(s.placed[0].y < field.y + field.h, "greift unter die Pille");
            assert_eq!(s.roles[1], Some(0));
            assert_eq!(s.roles[4], Some(1));
            assert!(s.placed[4].color.a > 0.0, "die zweite Zeile schwebt");
            assert_eq!(s.placed[1].color.a, 0.0, "die erste nicht");
            assert_eq!(s.labels.len(), 2 * entries.len());
        }

        #[test]
        fn the_code_card_keeps_its_lines() {
            let p = palette();
            let mut e = TextEngine::new();
            let c = code_card(&mut e, "#hardtext 1\n# Und so sieht die Quelle aus", 12.0, mk::typo::Weight::Normal, p.on_surface, 400.0, p);
            let mut out = Vec::new();
            lay_out(&c.node, 0.0, 0.0, &mut out);
            assert_eq!(c.labels.len(), 2, "zwei Zeilen bleiben zwei");
            assert_eq!(out.len(), CARD_PLACED + 2 + 1, "Karte + Zeilen + Füllzeile");
            let q = card_paragraph(&mut e, "Ein Zitat", 14.0, mk::typo::Weight::Normal, p.on_surface_variant, 400.0, p);
            let mut out2 = Vec::new();
            lay_out(&q.node, 0.0, 0.0, &mut out2);
            assert_eq!(out2[0].w, 400.0, "die Karte füllt die Spalte");
        }
    }

    // ------------------------------------------------------------------
    // SHELL CHIPS (R216 family, 02.11.2026): the top bar's glass chips and
    // shell cards in the house — what `mkw::chip_button`, `mkw::row_mark`
    // and `mkw::confirmation_card` were on iced.

    /// A rounded rectangle as a vector path (rings, strokes); y points down.
    pub fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<Cmd> {
        use std::f32::consts::{FRAC_PI_2, PI};
        let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
        vec![
            Cmd::Move(x + r, y),
            Cmd::Line(x + w - r, y),
            Cmd::Arc { cx: x + w - r, cy: y + r, r, start: -FRAC_PI_2, sweep: FRAC_PI_2, connect: true },
            Cmd::Line(x + w, y + h - r),
            Cmd::Arc { cx: x + w - r, cy: y + h - r, r, start: 0.0, sweep: FRAC_PI_2, connect: true },
            Cmd::Line(x + r, y + h),
            Cmd::Arc { cx: x + r, cy: y + h - r, r, start: FRAC_PI_2, sweep: FRAC_PI_2, connect: true },
            Cmd::Line(x, y + r),
            Cmd::Arc { cx: x + r, cy: y + r, r, start: PI, sweep: FRAC_PI_2, connect: true },
            Cmd::Close,
        ]
    }

    /// CHIP ANSWER (R216 family): a chip that IS its button answers a touch
    /// like a dock tile — darker while pressed, its edge lit by `glow` (0..1,
    /// `mk::motion::TapFlash::glow`). Laid OVER the chip's face at (x, y, w,
    /// h); nothing is laid when neither is on.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_chip_answer(b: &mut matrixkit_shell::SceneBuilder, x: f32, y: f32, w: f32, h: f32, radius: f32, glow: f32, pressed: bool, p: mk::Palette) {
        if pressed {
            b.quad(x, y, w, h, p.on_surface.with_alpha(0.14), radius, None, None);
        }
        let glow = glow.clamp(0.0, 1.0);
        if glow > 0.01 {
            let width = 1.0 + glow;
            let half = width / 2.0;
            let shape = Shape::new(w, h).op(
                rounded_rect_path(half, half, w - width, h - width, (radius - half).max(0.0)),
                ShapeStyle::Stroke { tint: p.primary.with_alpha(0.3 + 0.6 * glow), width },
            );
            let anchor = b.quad(x, y, w, h, mk::Rgba { a: 0.0, ..p.surface }, radius, None, None);
            b.pict(pict_of(&shape, x, y, Rect { x, y, w, h }, anchor));
        }
    }

    /// THE GLOW (round 115, from the dock's running apps, round 114): soft
    /// light rings around a body at (x, y, w, h) with `radius`, fading out
    /// over `room` points. `tint` is mixed half with white so it stands out
    /// on any wallpaper (the accent itself comes FROM the wallpaper and
    /// vanished on it); `strength` 0..1. Laid UNDER the body — call it
    /// first. Rings instead of a shadow: they stay inside the surface's own
    /// margin (a pane cuts off everything beyond its edge). Returns how
    /// many rings were laid.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_glow(b: &mut matrixkit_shell::SceneBuilder, x: f32, y: f32, w: f32, h: f32, radius: f32, room: f32, tint: mk::Rgba, strength: f32, role: Option<usize>) -> usize {
        let strength = strength.clamp(0.0, 1.0);
        if strength <= 0.01 || room < 0.5 {
            return 0;
        }
        let light = glow_light(tint);
        let rings = (room.round() as usize).clamp(2, 6);
        for k in (1..=rings).rev() {
            let e = room * k as f32 / rings as f32;
            let a = strength * 0.30 * (1.0 - (k - 1) as f32 / rings as f32);
            b.quad(x - e, y - e, w + 2.0 * e, h + 2.0 * e, light.with_alpha(a), radius + e, role, None);
        }
        rings
    }

    /// The glow's colour: `tint` mixed with white (0.45 : 0.55).
    pub fn glow_light(tint: mk::Rgba) -> mk::Rgba {
        mk::Rgba { r: 0.55 + 0.45 * tint.r, g: 0.55 + 0.45 * tint.g, b: 0.55 + 0.45 * tint.b, a: 1.0 }
    }

    #[cfg(test)]
    mod glow_tests {
        #[test]
        fn the_glow_fades_outward_and_stays_in_its_room() {
            let mut b = matrixkit_shell::SceneBuilder::new();
            let tint = super::mk::Rgba { r: 0.0, g: 0.5, b: 0.5, a: 1.0 };
            let n = super::lay_glow(&mut b, 10.0, 10.0, 40.0, 40.0, 9.0, 4.0, tint, 1.0, None);
            assert_eq!(n, 4);
            let sc = b.finish();
            // outermost first, at most `room` beyond the body, fading outward
            assert!((sc.placed[0].x - 6.0).abs() < 0.01);
            for pair in sc.placed.windows(2) {
                assert!(pair[0].color.a < pair[1].color.a);
            }
            assert_eq!(super::lay_glow(&mut matrixkit_shell::SceneBuilder::new(), 0.0, 0.0, 1.0, 1.0, 0.0, 4.0, tint, 0.0, None), 0);
        }
    }

    /// Snap a surface width UP to `step` px: a ticking value (a clock, a
    /// battery) rarely changes a chip's size, so the surface rarely moves.
    pub fn snap_up(width: f32, step: u32) -> u32 {
        let step = step.max(1) as f32;
        ((width.max(0.0) / step).ceil() * step) as u32
    }

    /// Where chips sit along a panel edge (R216 family, the top bar's row):
    /// each chip's distance from its anchoring edge — `side` px air before
    /// the first, `gap` between them. In the order given (the first sits at
    /// the edge).
    pub fn chip_offsets(widths: &[u32], side: i32, gap: i32) -> Vec<i32> {
        let mut at = side;
        widths
            .iter()
            .map(|w| {
                let here = at;
                at += *w as i32 + gap;
                here
            })
            .collect()
    }

    /// QUESTION CARD (R216 family — `mkw::confirmation_card` in the house):
    /// warning sign and title, the note, then «Abbrechen» and the red action
    /// side by side (the safe path first). No ground of its own — it lies
    /// on glass or on the caller's card. Returns its height.
    #[allow(clippy::too_many_arguments)]
    pub fn lay_question(
        b: &mut matrixkit_shell::SceneBuilder,
        engine: &mut TextEngine,
        x: f32,
        y: f32,
        w: f32,
        title: &str,
        note: &str,
        action: &str,
        cancel_role: usize,
        confirm_role: usize,
        p: mk::Palette,
    ) -> f32 {
        let symfam = crate::symbol_family();
        let warn = symbol::WARNING.to_string();
        let (gw, gh) = engine.measure_family(symfam, &warn, mk::font_size::LARGE, mk::typo::Weight::Normal);
        let head = mk::typo::SUBTITLE;
        let (_, th) = engine.measure_styled(title, head.px(), head.weight);
        let line = gh.max(th);
        b.text(engine, warn, mk::font_size::LARGE, mk::typo::Weight::Normal, symfam, p.error, x, y + (line - gh) / 2.0, None, None);
        b.styled(engine, title, head, p.on_surface, x + gw + mk::spacing::XS, y + (line - th) / 2.0, None, None);
        let mut cy = y + line + mk::spacing::S;
        let body = mk::typo::BODY;
        let chars = ((w / (body.px() * 0.5)) as usize).max(16);
        for l in mk::wrap_words(note, chars, 4) {
            let (_, lh) = b.styled(engine, l, body, p.on_surface_variant, x, cy, None, None);
            cy += lh;
        }
        cy += mk::spacing::L;
        let half = (w - mk::spacing::S) / 2.0;
        let bh = 44.0;
        let st = mk::typo::CALLOUT;
        let white = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        for (i, (label, role, fill, ink)) in [
            ("Abbrechen", cancel_role, p.on_surface.over(p.surface_container_high, 0.10), p.on_surface),
            (action, confirm_role, p.error, white),
        ]
        .into_iter()
        .enumerate()
        {
            let bx = x + i as f32 * (half + mk::spacing::S);
            b.quad(bx, cy, half, bh, fill, bh / 2.0, Some(role), None);
            let shown = ellipsize(engine, label, st.px(), st.weight, half - 2.0 * mk::spacing::M);
            let (lw, lh) = engine.measure_styled(&shown, st.px(), st.weight);
            b.styled(engine, shown, st, ink, bx + (half - lw) / 2.0, cy + (bh - lh) / 2.0, Some(role), None);
        }
        cy + bh - y
    }

    impl FormPage<'_> {
        /// A MARK row (R216 family — `mkw::row_mark`): a colour dot, title
        /// and description, a check box on the right — choosing several of
        /// a kind (the app card's Krabbler species). The whole row presses.
        pub fn mark(&mut self, engine: &mut TextEngine, role: usize, tint: mk::Rgba, title: &str, desc: Option<&str>, marked: bool) {
            let p = self.p;
            let family = crate::symbol_family();
            let px = mk::typo::symbol_px(22.0);
            let tick = if marked { symbol::CHECK_BOX } else { symbol::CHECK_BOX_BLANK }.to_string();
            let (cw, chh) = engine.measure_family(family, &tick, px, mk::typo::Weight::Normal);
            // A blank symbol keeps the row's 28 px lead free for the dot.
            let (right, top, h) = self.row(engine, Some(role), Some(' '), title, desc, cw, p.on_surface);
            let d = 10.0;
            let dot = if marked { tint } else { tint.with_alpha(0.35) };
            self.b.quad(self.x + mk::spacing::M + (28.0 - d) / 2.0, top + (h - d) / 2.0, d, d, dot, d / 2.0, Some(role), self.clip);
            self.b.text(engine, tick, px, mk::typo::Weight::Normal, family, if marked { p.primary } else { p.on_surface_variant }, right - cw, top + (h - chh) / 2.0, Some(role), self.clip);
        }
    }

    #[cfg(test)]
    mod shell_chip_tests {
        use super::*;

        #[test]
        fn chips_snap_up_and_sit_in_a_row() {
            assert_eq!(snap_up(81.0, 8), 88);
            assert_eq!(snap_up(88.0, 8), 88);
            assert_eq!(snap_up(-3.0, 8), 0);
            // The duo's right group: 10 px from the edge, 4 px between.
            assert_eq!(chip_offsets(&[64, 56, 120], 10, 4), vec![10, 78, 138]);
            assert!(chip_offsets(&[], 10, 4).is_empty());
        }

        #[test]
        fn the_question_has_two_answers() {
            let mut e = TextEngine::new();
            let mut b = matrixkit_shell::SceneBuilder::new();
            let p = mk::Palette::default();
            let h = lay_question(&mut b, &mut e, 0.0, 0.0, 300.0, "Ausschalten?", "Ungesicherte Arbeit geht verloren.", "Ausschalten", 1, 2, p);
            assert!(h > 44.0 + 20.0, "{h}");
            let s = b.finish();
            assert!(s.roles.contains(&Some(1)) && s.roles.contains(&Some(2)));
            assert!(s.texts.iter().any(|t| t.s == "Abbrechen"));
        }

        #[test]
        fn the_ring_path_closes() {
            let path = rounded_rect_path(1.0, 1.0, 40.0, 20.0, 50.0);
            assert!(matches!(path.first(), Some(Cmd::Move(..))));
            assert!(matches!(path.last(), Some(Cmd::Close)));
            // The radius never exceeds half the short side.
            if let Some(Cmd::Arc { r, .. }) = path.get(2) {
                assert_eq!(*r, 10.0);
            }
        }

        #[test]
        fn a_mark_row_presses_as_a_whole() {
            let mut e = TextEngine::new();
            let mut b = matrixkit_shell::SceneBuilder::new();
            let p = mk::Palette::default();
            let mut page = FormPage::new(&mut b, 0.0, 0.0, 320.0, None, p);
            page.mark(&mut e, 5, p.primary, "Ameise", Some("frisst Verben"), true);
            let (h, _) = page.finish();
            assert!(h >= FORM_ROW_MIN);
            let s = b.finish();
            assert!(s.roles.iter().filter(|r| **r == Some(5)).count() >= 3);
        }
    }
}

#[cfg(test)]
mod family_tests {
    use crate::scene::{self as fam, night};
    use matrixkit_text::TextEngine;
    use matrixkit_theme as mk;

    /// Contrast lesson: text on night glass is provably readable.
    #[test]
    fn night_contrast() {
        assert!(night::contrast(night::INK, night::SURFACE) >= 7.0, "INK auf SURFACE {}", night::contrast(night::INK, night::SURFACE));
        assert!(night::contrast(night::INK2, night::SURFACE) >= 4.5, "INK2 auf SURFACE {}", night::contrast(night::INK2, night::SURFACE));
        assert!(night::contrast(night::TURQUOISE, night::GROUND) >= 4.5, "TURQUOISE auf GROUND {}", night::contrast(night::TURQUOISE, night::GROUND));
        assert!(night::contrast(night::INK, night::GROUND) >= 7.0);
        assert!((night::contrast(mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }) - 21.0).abs() < 0.01);
    }

    /// The glass card: open 10 px, closed h, the border one hairline beyond it.
    #[test]
    fn night_glass_closes_to_height() {
        let mut b = fam::Sheet::new();
        let g = night::glass_open(&mut b, 10.0, 20.0, 300.0, None);
        assert_eq!(b.placed[g.body].h, 10.0);
        night::glass_close(&mut b, g, 123.0);
        assert_eq!(b.placed[g.body].h, 123.0);
        assert_eq!(b.placed[g.edge].h, 125.0);
        assert_eq!(b.placed[g.edge].w, 302.0);
    }

    /// The sky: full size, opaque, computed once per size (shared).
    #[test]
    fn night_sky_is_cached_per_size() {
        let a = night::sky(64, 32);
        assert_eq!(a.len(), 64 * 32 * 4);
        assert!(a.iter().skip(3).step_by(4).all(|&alpha| alpha == 255));
        let b = night::sky(64, 32);
        assert!(std::sync::Arc::ptr_eq(&a, &b), "dasselbe Mass teilt den Puffer");
        let c = night::sky(32, 32);
        assert!(!std::sync::Arc::ptr_eq(&a, &c));
    }

    /// The laid button: label on the last slot, measure positive, role at the node.
    #[test]
    fn lay_button_labels_last_slot() {
        let mut e = TextEngine::new();
        let mut b = fam::Sheet::new();
        let p = mk::Palette::default();
        let (w, h) = fam::lay_button(&mut b, &mut e, "Ansicht zurück", crate::buttons::Style::Borderless, fam::Status::Idle, Some(7), 5.0, 6.0, None, p);
        assert!(w > 20.0 && h > 10.0, "{w}×{h}");
        assert_eq!(b.roles[0], Some(7));
        assert_eq!(b.texts.len(), 1);
        assert_eq!(b.texts[0].s, "Ansicht zurück");
        let last = b.placed.len() - 1;
        assert_eq!(b.texts[0].anchor, last, "das Label liegt auf dem letzten Platz des Knotens");
    }
}

/// The key-list registry family (R134): editable, persisted control
/// sets with healing — unknown keys drop silently (configs written by
/// newer images stay loadable), fixed keys heal back in at their
/// default spot. Extracted from the Zentrale; Anpassen's bar and dock
/// lists are the next migration candidates.
pub mod registry {
    use matrixkit_theme as mk;

    pub struct Store {
        /// Settings file name (mk::settings).
        pub file: &'static str,
        /// Space-separated default key list.
        pub default: &'static str,
    }

    impl Store {
        pub fn load(
            &self,
            known: impl Fn(&str) -> bool,
            fixed: impl Fn(&str) -> bool,
        ) -> Vec<String> {
            let mut keys: Vec<String> = mk::settings::read(self.file)
                .unwrap_or_else(|| self.default.into())
                .split_whitespace()
                .filter(|k| known(k))
                .map(String::from)
                .collect();
            for (pos, k) in self.default.split_whitespace().enumerate() {
                if fixed(k) && !keys.iter().any(|q| q == k) {
                    keys.insert(pos.min(keys.len()), k.to_string());
                }
            }
            keys
        }

        pub fn save(&self, keys: &[String]) {
            mk::settings::write(self.file, &keys.join(" "));
        }
    }
}

pub mod central {

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Category {
        Suggest,
        Sound,
        Radio,
        Display,
        Access,
        System,
    }

    impl Category {
        /// Gallery order: suggestions first, like the reference.
        pub const ALL: [Category; 6] = [
            Category::Suggest,
            Category::Sound,
            Category::Radio,
            Category::Display,
            Category::Access,
            Category::System,
        ];

        pub fn title(self) -> &'static str {
            match self {
                Category::Suggest => "VORSCHLÄGE",
                Category::Sound => "KLANG",
                Category::Radio => "FUNK",
                Category::Display => "BILDSCHIRM",
                Category::Access => "BEDIENUNGSHILFEN",
                Category::System => "SYSTEM",
            }
        }
    }

    pub struct Control {
        pub key: &'static str,
        pub title: &'static str,
        pub subtitle: &'static str,
        pub category: Category,
        pub glyph: char,
        /// Fixed modules (reference: the sound and display sliders are
        /// permanent) — always in the panel, invisible to the edit flow.
        pub fixed: bool,
    }

    use crate::symbol as sy;

    /// Everything the Zentrale can host. Order inside a category is the
    /// gallery order; the PANEL order is the user's list.
    pub const REGISTRY: &[Control] = &[
        Control { key: "capture", title: "Aufnahme", subtitle: "Bildschirmfoto & -film", category: Category::Suggest, glyph: sy::IMAGE, fixed: false },
        // Round 64: straight from the control centre (matrix-capture).
        Control { key: "screenshot", title: "Bildschirmfoto", subtitle: "Ganzer Bildschirm, sofort gespeichert", category: Category::Suggest, glyph: sy::IMAGE, fixed: false },
        Control { key: "recording", title: "Bildschirmaufnahme", subtitle: "Bildschirm filmen — nochmals tippen beendet", category: Category::Suggest, glyph: sy::MOVIE, fixed: false },
        Control { key: "lock", title: "Sperren", subtitle: "Bildschirm sofort sperren", category: Category::Suggest, glyph: sy::LOCK, fixed: false },
        Control { key: "volume", title: "Lautstärke", subtitle: "Regler für den Ton", category: Category::Sound, glyph: sy::VOLUME_UP, fixed: true },
        Control { key: "mute", title: "Ton aus", subtitle: "Stummschalter", category: Category::Sound, glyph: sy::VOLUME_OFF, fixed: false },
        Control { key: "mic", title: "Mikrofon aus", subtitle: "Mikrofon stummschalten", category: Category::Sound, glyph: sy::MIC_OFF, fixed: false },
        Control { key: "wlan", title: "WLAN", subtitle: "Funknetz an und aus", category: Category::Radio, glyph: sy::WIFI, fixed: false },
        Control { key: "bluetooth", title: "Bluetooth", subtitle: "Geräte-Funk an und aus", category: Category::Radio, glyph: sy::BLUETOOTH, fixed: false },
        Control { key: "brightness", title: "Helligkeit", subtitle: "Bildschirm-Regler", category: Category::Display, glyph: sy::BRIGHTNESS, fixed: true },
        Control { key: "motion", title: "Bewegung reduzieren", subtitle: "Federn und Übergänge stillhalten", category: Category::Access, glyph: sy::ANIMATION, fixed: false },
        Control { key: "transparency", title: "Transparenz reduzieren", subtitle: "Leisten und Schleier deckend", category: Category::Access, glyph: sy::OPACITY, fixed: false },
        // Round 73: rotation lock (duo-haltung holds the orientation).
        Control { key: "rotlock", title: "Drehsperre", subtitle: "Bildschirm dreht sich nicht mit", category: Category::Display, glyph: sy::LOCK, fixed: false },
        // Round 79: deep sleep (duo-schlaf: dark panels, a minute, nothing playing).
        Control { key: "deepsleep", title: "Tiefschlaf", subtitle: "Schläft tief, wenn die Bildschirme dunkel sind", category: Category::System, glyph: sy::POWER, fixed: false },
        // Round 72: automatic brightness (the light sensor, learning).
        Control { key: "autobright", title: "Auto-Helligkeit", subtitle: "Folgt dem Umgebungslicht", category: Category::Display, glyph: sy::BRIGHTNESS, fixed: false },
        // Round 71: Night Shift (matrix-nightshift follows the setting).
        Control { key: "nightshift", title: "Night Shift", subtitle: "Wärmere Farben am Abend", category: Category::Display, glyph: sy::DARK_MODE, fixed: false },
        Control { key: "contrast", title: "Erhöhter Kontrast", subtitle: "Kräftigere Text- und Konturfarben", category: Category::Access, glyph: sy::CONTRAST, fixed: false },
        Control { key: "appearance", title: "Erscheinungsbild", subtitle: "Hell und Dunkel wechseln", category: Category::System, glyph: sy::DARK_MODE, fixed: false },
    ];

    /// The layout the Zentrale carried before it became editable.
    pub const DEFAULT: &str =
        "screenshot recording volume mute mic wlan bluetooth brightness autobright nightshift rotlock deepsleep motion transparency contrast appearance";

    pub fn find(key: &str) -> Option<&'static Control> {
        REGISTRY.iter().find(|c| c.key == key)
    }

    /// The user's list — unknown keys drop silently (forward-compatible
    /// with configs written by newer images). Fixed modules are healed
    /// back in at their default spot: they cannot be removed, so an
    /// older config without them must not lose the sliders.
    /// The Zentrale's store, on the registry family (R134).
    const STORE: crate::registry::Store = crate::registry::Store {
        file: "central-controls",
        default: DEFAULT,
    };

    pub fn load() -> Vec<String> {
        STORE.load(
            |k| find(k).is_some(),
            |k| find(k).is_some_and(|c| c.fixed),
        )
    }

    pub fn save(keys: &[String]) {
        STORE.save(keys);
    }


    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_keys_are_registered() {
            for k in DEFAULT.split_whitespace() {
                assert!(find(k).is_some(), "unregistered default key {k}");
            }
        }

        #[test]
        fn keys_are_unique() {
            for (n, c) in REGISTRY.iter().enumerate() {
                assert!(
                    REGISTRY.iter().skip(n + 1).all(|q| q.key != c.key),
                    "duplicate key {}",
                    c.key
                );
            }
        }
    }
}
