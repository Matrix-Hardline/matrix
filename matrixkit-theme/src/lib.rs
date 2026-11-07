//! matrixkit-theme — the foundation of the Matrix UI.
//!
//! Contains: design tokens (extracted from the DMS source, see DESIGNSYSTEM.md),
//! the live palette (reads matugen/DMS colors, follows wallpaper & light/dark)
//! and the motion core (Material curves + spring).

use serde::Deserialize;
use std::path::PathBuf;
use std::time::SystemTime;

// ============================================================================
// Tokens — spacing, radii, sizes (source: DMS Theme.qml)
// ============================================================================

pub mod spacing {
    pub const XXS: f32 = 2.0;
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 40.0;
}

pub const CORNER_RADIUS: f32 = radius::NORMAL;

/// Corner radii — Matrix' corner DNA, distilled from the reference system.
///
/// The reference teaches three things about corners:
/// 1. **Continuous curvature** (`RoundedCornerStyle.continuous` is the
///    reference default): corners are superellipses (squircles), not
///    circular arcs — the reason reference corners feel "softer". Our
///    ICONS already have this (tiny-skia superellipse, n=4.6). UI surfaces
///    use iced's circular arc (true squircle surfaces would need a
///    custom-shape rework — documented, for later).
/// 2. **ONE base radius** for the whole system (a coherence signal) — ours
///    is NORMAL. Every other radius derives from it, never arbitrary.
/// 3. **Concentricity** (`ContainerRelativeShape`/`ConcentricRectangle`):
///    an element INSIDE a rounded container rounds so that the corners
///    run concentrically — inner = outer − inset. That lets nested
///    surfaces mesh cleanly instead of "biting" each other.
/// SDK survey R71: the control height ladder of reference 27, measured
/// on a living NSButton — 16/20/24/28, clean steps of four — and the
/// NSSwitch as a 54×24 capsule (pixel render: r = height/2; our
/// radius::capsule is thereby empirically sealed). New controls reach
/// for THESE values instead of inventing their own heights.
pub mod controls {
    pub const MINI: f32 = 16.0;
    pub const SMALL: f32 = 20.0;
    pub const NORMAL: f32 = 24.0;
    pub const LARGE: f32 = 28.0;
    pub const TOGGLE_WIDTH: f32 = 54.0;
    pub const TOGGLE_HEIGHT: f32 = 24.0;
}

pub mod radius {
    /// The ONE base radius: cards, buttons, sections, window corners.
    pub const NORMAL: f32 = 12.0;
    /// Small surfaces: chips, pills, tooltips, selection buttons.
    pub const SMALL: f32 = 8.0;
    /// Tiny elements: scroll thumb, progress bars, placeholders.
    pub const MINI: f32 = 4.0;
    /// Large floating sheets (dialogs, panels): a touch softer.
    pub const LARGE: f32 = 16.0;

    /// Capsule/circle: a surface of height `h` gets fully rounded (traffic
    /// lights, color swatches, round indicators) — width/2 or height/2.
    pub fn capsule(height: f32) -> f32 {
        height / 2.0
    }

    /// CONCENTRIC inner radius: the radius of an element INSIDE a rounded
    /// container (outer radius `aussen`, inset `abstand`) so the corners
    /// run concentrically. Never below MINI — a too-small inner radius
    /// reads as awkwardly sharp.
    pub fn inner(outer: f32, inset: f32) -> f32 {
        (outer - inset).max(MINI)
    }
}

pub mod font_size {
    pub const SMALL: f32 = 12.0;
    pub const MEDIUM: f32 = 14.0;
    pub const LARGE: f32 = 16.0;
    pub const XLARGE: f32 = 20.0;
    /// Hero numbers (dashboard percentages, gauges) — the one display
    /// size above the text ladder (R103c: sysmon carried a raw 34).
    pub const DISPLAY: f32 = 34.0;
}

/// Semantic typography — the core UI knowledge from the reference (Font.TextStyle).
///
/// The reference's actual lesson: hierarchy is SEMANTIC, not metric. You
/// never write "size 14" — you say "this is a `body`", and size,
/// weight and line rhythm arrive as a BUNDLED role. That keeps the
/// typography coherent across the whole surface, no matter who builds it.
///
/// Carried over to Matrix (typeface: Inter Variable, aligned with the
/// existing DMS scale 12/14/16/20). Color deliberately stays SEPARATE
/// (it comes from the palette) — shape and meaning are two axes.
pub mod typo {
    /// Font weight, framework-neutral (like Rgba). Only the three steps
    /// the reference text styles actually distinguish.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Weight {
        Normal,
        Medium,
        Semibold,
    }

    /// A text style = size + weight, named after its ROLE.
    #[derive(Clone, Copy, Debug)]
    pub struct TextStyle {
        pub size: f32,
        pub weight: Weight,
    }

    impl TextStyle {
        /// The LIVED size of this role: `size` times the live
        /// factor from `~/.config/matrix/text-size`.
        ///
        /// `mkw::txt()` has always included the factor for the iced path
        /// (`.size(style.size * factor())`). The
        /// first-light path, on the other hand, used `style.size` raw and
        /// thereby swallowed the font size setting
        /// completely — with `text-size: large` every font appeared
        /// 13 % too small (finger probe Matrix Home, 16.08.2026).
        /// Whoever measures AND whoever paints takes this grip; only then do
        /// size and image stay together.
        pub fn px(self) -> f32 {
            self.size * factor()
        }
    }

    /// A SYMBOL size in the lived measure.
    ///
    /// `mkw::symbol()` has always included the factor
    /// (`.size(size * factor())`) — so EVERY glyph of the
    /// iced path grows with the font size setting. The
    /// first-light path passed raw numbers (18.0, font_size::LARGE)
    /// to the text machine and therefore painted its symbols at
    /// `text-size: large` an eighth too small — the same class
    /// as [`TextStyle::px`], only one level deeper (finger probe of the
    /// window key, 16.08.2026).
    pub fn symbol_px(px: f32) -> f32 {
        px * factor()
    }

    // Reference Font.TextStyle → Matrix role (the subset of the 13
    // reference styles a desktop app actually needs):
    //
    // SDK survey R71 (living AppKit, 2026-07-19): largeTitle 26 ·
    // title1 22 · title2 17 · title3 15 · headline 13 semibold · body 13 ·
    // callout 12 · subheadline 11 · footnote/caption 10. Our ladder runs
    // deliberately 1–2 pt larger (Inter on 4K without scaling), but the
    // RATIOS match as measured: LARGE_TITLE = 2.0×BODY (reference 2.0×),
    // SUBTITLE 1.14× (title3 1.15×), HEADLINE = BODY size in
    // semibold (headline rule exact), CALLOUT 0.93× (callout 0.92×).
    // The one deliberate deviation: TITLE 1.43× sits between title1
    // (1.69×) and title2 (1.31×) — we run ONE title step instead of two.

    /// `largeTitle` — the one big moment (recovery countdown, welcome).
    pub const LARGE_TITLE: TextStyle = TextStyle { size: 28.0, weight: Weight::Semibold };
    /// `title` — app name in the About panel, step headings.
    pub const TITLE: TextStyle = TextStyle { size: 20.0, weight: Weight::Semibold };
    /// `title3` — section heading within a view.
    pub const SUBTITLE: TextStyle = TextStyle { size: 16.0, weight: Weight::Semibold };
    /// `headline` — emphasized row (highlighted entry, active selection).
    pub const HEADLINE: TextStyle = TextStyle { size: 14.0, weight: Weight::Semibold };
    /// `body` — the default running text. Most of the UI.
    pub const BODY: TextStyle = TextStyle { size: 14.0, weight: Weight::Normal };
    /// `callout` — slightly smaller companion text next to the main content.
    pub const CALLOUT: TextStyle = TextStyle { size: 13.0, weight: Weight::Normal };
    /// `footnote`/`subheadline` — descriptions, footer, secondary content.
    pub const CAPTION: TextStyle = TextStyle { size: 12.0, weight: Weight::Normal };
    /// `caption` — ALL-CAPS section headings, tiny labels (slightly
    /// emphasized so they stay readable as structure despite the small size).
    pub const LABEL: TextStyle = TextStyle { size: 12.0, weight: Weight::Medium };

    /// The dynamicTypeSize extract (reference round 20): ONE living
    /// Factor scales the whole semantic ladder. Steps like the reference:
    /// small 0.9 · normal 1.0 · large 1.15 · xlarge 1.3 —
    /// ~/.config/matrix/text-size. Cached (1 s freshness) because txt()
    /// runs many times in every view build.
    pub fn factor() -> f32 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CACHE: AtomicU64 = AtomicU64::new(0);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let packed = CACHE.load(Ordering::Relaxed);
        let (state, value_bits) = (packed >> 32, (packed & 0xffff_ffff) as u32);
        if state == now && value_bits != 0 {
            return f32::from_bits(value_bits);
        }
        let f: f32 = match super::settings::read("text-size").as_deref() {
            Some("small") | Some("klein") => 0.9,
            Some("large") | Some("gross") => 1.15,
            Some("xlarge") | Some("sehr-gross") => 1.3,
            _ => 1.0,
        };
        CACHE.store((now << 32) | u64::from(f.to_bits()), Ordering::Relaxed);
        f
    }

    /// The reference steps in display order (for ± rows).
    pub const SIZES: [&str; 4] = ["small", "normal", "large", "xlarge"];
}

pub mod icon_size {
    pub const XSMALL: f32 = 13.0;
    pub const SMALL: f32 = 16.0;
    /// Row symbols (lists, toolbar glyphs).
    pub const MEDIUM: f32 = 18.0;
    pub const NORMAL: f32 = 24.0;
    pub const LARGE: f32 = 32.0;
    /// Placeholder symbols (empty tiles, previews).
    pub const XLARGE: f32 = 40.0;
    /// Hero symbols (greeter, dialogs, empty states).
    pub const HERO: f32 = 48.0;
}

/// Settings culture (reference-UI AppStorage, round 9): ONE source for
/// user settings — simple readable files under
/// ~/.config/matrix/<name>. Apps read and write only through here;
/// nothing is hidden — everything can be edited or backed up by hand.
pub use settings as setting;
pub mod settings {
    /// HOT FLAG (R209 family): a settings switch asked while DRAWING —
    /// `motion_reduced` is asked by every spring and every widget of every
    /// frame. Read fresh it cost the duo's settings app 600 file opens a
    /// second (strace, 01.11.2026). The answer is kept for 500 ms per key:
    /// a switch flipped in the settings is seen within half a second.
    /// `on` lists the values that mean «on» (English contract + German).
    pub fn flag(key: &'static str, on: &[&str]) -> bool {
        use std::collections::HashMap;
        use std::sync::Mutex;
        use std::time::{Duration, Instant};
        static CACHE: Mutex<Option<HashMap<&'static str, (Instant, Option<String>)>>> = Mutex::new(None);
        let now = Instant::now();
        let value = {
            let mut guard = CACHE.lock().unwrap_or_else(|e| e.into_inner());
            let map = guard.get_or_insert_with(HashMap::new);
            match map.get(key) {
                Some((at, v)) if now.duration_since(*at) < Duration::from_millis(500) => v.clone(),
                _ => {
                    let v = read(key);
                    map.insert(key, (now, v.clone()));
                    v
                }
            }
        };
        value.is_some_and(|v| on.contains(&v.as_str()))
    }

    use std::path::PathBuf;

    fn path(name: &str) -> PathBuf {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config")
            });
        base.join("matrix").join(name)
    }

    /// Legacy → current key renames (R101e English migration). When the
    /// current key has no file yet, `read` falls back to the old German
    /// name so a running system keeps its settings across the update.
    const RENAMES: &[(&str, &str)] = &[
        ("canvas-ghost", "canvas-ghost"),
        ("canvas-paging", "canvas-paging"),
        ("canvas-click-fetch", "canvas-click-fetch"),
        ("canvas-homeward", "canvas-homeward"),
        ("typing-clicks", "typing-clicks"),
        ("do-not-disturb", "do-not-disturb"),
        ("natural-scrolling", "natural-scrolling"),
        ("wallpaper-mode", "wallpaper-mode"),
        ("mouse-speed", "mouse-speed"),
        ("corner-radius", "corner-radius"),
        ("appearance", "appearance"),
        ("rotation-lock", "rotation-lock"),
        ("greeter-session", "greeter-session"),
        ("capture-resolution", "capture-resolution"),
        ("keyboard-layout", "keyboard-layout"),
        ("text-size", "text-size"),
        ("help-state", "help-state"),
        ("languages", "languages"),
        ("scaling", "scaling"),
        ("motion", "motion"),
        ("reduce-motion", "reduce-motion"),
    ];

    /// Read a value (trimmed); None = never set.
    pub fn read(name: &str) -> Option<String> {
        if let Ok(s) = std::fs::read_to_string(path(name)) {
            return Some(s.trim().to_string());
        }
        // Migration fallback: adopt the value from the old German key once.
        if let Some((_, old)) = RENAMES.iter().find(|(new, _)| *new == name) {
            if let Ok(s) = std::fs::read_to_string(path(old)) {
                let v = s.trim().to_string();
                write(name, &v);
                return Some(v);
            }
        }
        None
    }

    /// Write a value (creates ~/.config/matrix; best effort like AppStorage).
    pub fn write(name: &str, value: &str) {
        // R148: ATOMIC (tmp + rename) — these files are live-watched
        // (bar-widgets, dock-pins, …); a half-written file would feed
        // the watcher parse garbage. Same doctrine as .desktop writes.
        let p = path(name);
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = p.with_extension("new");
        if std::fs::write(&tmp, value).is_ok() {
            let _ = std::fs::rename(&tmp, &p);
        }
    }
}

/// Formatter culture (Foundation ByteCountFormatStyle & co., round 8):
/// values are formatted the SAME system-wide — apps never format themselves.
/// de_CH (R65c): Switzerland separates decimals with a POINT ("4.5 GB") —
/// the R35 finding from Matrix Files now applies to the whole family
/// (previously it spoke German comma and Files had to break ranks).
pub mod format {
    /// R100: the ONE place for decimal culture — regional now.
    /// de_CH/international: the point stays a point; region Germany
    /// gets its comma (measured live: 1.234.567,89 vs 1'234'567.89).
    fn comma(s: String) -> String {
        if super::region::decimal_sep() == ',' {
            s.replace('.', ",")
        } else {
            s
        }
    }

    /// A byte value in the file-manager reference style: "512 Byte", "980 KB", "610.7 MB",
    /// "512 GB". R68 empirical correction: the REAL reference file manager does NOT round below
    /// 100 (R35 observation: "610.7 MB") — it adapts like
    /// ByteCountFormatter: MB one decimal, GB/TB up to two, and
    /// superfluous zeros are dropped ("512 GB", never "512.0 GB").
    pub fn bytes(b: u64) -> String {
        fn num(w: f64, max_dez: usize) -> String {
            let mut s = format!("{w:.*}", max_dez);
            if s.contains('.') {
                while s.ends_with('0') {
                    s.pop();
                }
                if s.ends_with('.') {
                    s.pop();
                }
            }
            s
        }
        const STEPS: [(f64, &str, usize); 4] =
            [(1e12, "TB", 2), (1e9, "GB", 2), (1e6, "MB", 1), (1e3, "KB", 0)];
        for (threshold, unit, dez) in STEPS {
            if (b as f64) >= threshold {
                let w = b as f64 / threshold;
                return comma(format!("{} {unit}", num(w, dez)));
            }
        }
        format!("{b} Byte")
    }

    /// Value pair with a SHARED unit (memory displays): "2.4 / 15.5 GB".
    /// The larger value picks the unit — keeps the pair comparable.
    pub fn bytes_paar(part: u64, total: u64) -> String {
        const STEPS: [(f64, &str); 4] = [(1e12, "TB"), (1e9, "GB"), (1e6, "MB"), (1e3, "KB")];
        let max = part.max(total) as f64;
        for (threshold, unit) in STEPS {
            if max >= threshold {
                let (a, b) = (part as f64 / threshold, total as f64 / threshold);
                let num = |w: f64| {
                    if b >= 100.0 { format!("{w:.0}") } else { comma(format!("{w:.1}")) }
                };
                return format!("{} / {} {unit}", num(a), num(b));
            }
        }
        format!("{part} / {total} Byte")
    }

    /// Memory TextStyle (ByteCountFormatter ".memory"): binary (1024-based),
    /// colloquial units — 16 GiB reads "16 GB" like the reference.
    pub fn bytes_decimal(b: u64) -> String {
        let gib = b as f64 / 1_073_741_824.0;
        if gib >= 100.0 {
            format!("{gib:.0} GB")
        } else if gib >= 1.0 {
            comma(format!("{gib:.1} GB"))
        } else {
            // Below 1 GiB the formatter picks the unit itself (reference
            // ByteCountFormatter) — process RSS would read "0.0 GB" everywhere otherwise.
            let mib = b as f64 / 1_048_576.0;
            format!("{mib:.0} MB")
        }
    }

    /// Memory pair with a shared binary unit: "2.4 / 15.5 GB".
    pub fn bytes_decimal_pair(part: u64, total: u64) -> String {
        let g = |b: u64| b as f64 / 1_073_741_824.0;
        let (a, b) = (g(part), g(total));
        let num = |w: f64| {
            if b >= 100.0 { format!("{w:.0}") } else { comma(format!("{w:.1}")) }
        };
        format!("{} / {} GB", num(a), num(b))
    }

    /// Data rate: "1.2 MB/s", "830 kB/s", "0 B/s" — decimal like bytes().
    pub fn rate(bytes_pro_s: f64) -> String {
        let b = bytes_pro_s.max(0.0);
        if b < 1e3 {
            return format!("{b:.0} B/s");
        }
        format!("{}/s", bytes(b as u64))
    }

    /// Countdown/remaining time as MM:SS: "28:48", "0:07".
    pub fn duration_mmss(seconds: u64) -> String {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    }

    /// Time spans the way the reference tells them (uptime): terse,
    /// largest unit first — migrated in from the overview (R65b).
    pub fn duration(seconds: u64) -> String {
        let (t, h, m) = (
            seconds / 86_400,
            (seconds % 86_400) / 3600,
            (seconds % 3600) / 60,
        );
        match (t, h) {
            (0, 0) => format!("{m} Min."),
            (0, _) => format!("{h} Std. {m} Min."),
            _ => format!("{t} T. {h} Std."),
        }
    }

    /// How long ago, the way a status line says it (R214 family):
    /// «gerade eben», «vor 40 s», «vor 5 min», «vor 2 h», «vor 3 Tagen».
    pub fn ago_secs(seconds: u64) -> String {
        match seconds {
            0..=4 => String::from("gerade eben"),
            5..=59 => format!("vor {seconds} s"),
            60..=3599 => format!("vor {} min", seconds / 60),
            3600..=86_399 => format!("vor {} h", seconds / 3600),
            86_400..=172_799 => String::from("vor einem Tag"),
            _ => format!("vor {} Tagen", seconds / 86_400),
        }
    }

    /// `ago_secs` from a unix time.
    pub fn ago(unix: u64) -> String {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(unix);
        ago_secs(now.saturating_sub(unix))
    }

    #[cfg(test)]
    mod ago_tests {
        #[test]
        fn says_how_long_ago() {
            assert_eq!(super::ago_secs(2), "gerade eben");
            assert_eq!(super::ago_secs(40), "vor 40 s");
            assert_eq!(super::ago_secs(300), "vor 5 min");
            assert_eq!(super::ago_secs(7200), "vor 2 h");
            assert_eq!(super::ago_secs(3 * 86_400), "vor 3 Tagen");
        }
    }
}

/// AUDIO (R214 family, 02.11.2026): PipeWire for apps — the endpoints
/// (outputs, inputs) with the default marked, volumes, the default device,
/// a test tone, playing a sound, a microphone level meter; and `pactl`
/// in a language one can parse. Gathered out of Matrix Einstellungen (Ton)
/// and the hearing guard, where each kept its own.
pub mod audio {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Arc;

    /// An output (sink) or input (source).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Endpoint {
        pub id: u32,
        pub name: String,
        /// The default one.
        pub standard: bool,
    }

    /// What kind of device an endpoint is (round 157) — read from its name,
    /// as PipeWire's names are what the house has. A TV on USB-C
    /// (duo-bildton's «Fernseher», an HDMI/DisplayPort sink) is a Display.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Speaker,
        Headphones,
        Bluetooth,
        Display,
        /// Round 169: a built-in microphone (the duo's DMIC, 05.11.2026).
        Microphone,
        /// Round 169: the microphone of a headset (cable or USB).
        Headset,
    }

    pub fn kind_of(name: &str) -> Kind {
        let n = name.to_lowercase();
        let has = |words: &[&str]| words.iter().any(|w| n.contains(w));
        if has(&["fernseher", "hdmi", "displayport", "display port", " tv", "tv ", "monitor", "bildschirm"]) || n == "tv" {
            Kind::Display
        } else if has(&["bluetooth", "bluez", "airpods", "buds"]) {
            Kind::Bluetooth
        } else if has(&["kopfhörer", "headphone", "headset", "earphone"]) {
            Kind::Headphones
        } else {
            Kind::Speaker
        }
    }

    /// What kind of INPUT (source) an endpoint is (round 169): a built-in
    /// microphone, a headset's or a Bluetooth one — the same name rules as
    /// the outputs, but an input is never a speaker or a display.
    pub fn kind_of_input(name: &str) -> Kind {
        match kind_of(name) {
            Kind::Bluetooth => Kind::Bluetooth,
            Kind::Headphones => Kind::Headset,
            _ => {
                let n = name.to_lowercase();
                if n.contains("usb") || n.contains("headset") {
                    Kind::Headset
                } else {
                    Kind::Microphone
                }
            }
        }
    }

    impl Kind {
        /// Its symbol in the Matrix symbol font (Material Symbols code points).
        pub fn glyph(self) -> char {
            match self {
                Kind::Speaker => '\u{e050}',
                Kind::Headphones => '\u{e310}',
                Kind::Bluetooth => '\u{e1a7}',
                Kind::Display => '\u{e333}',
                Kind::Microphone => '\u{e029}',
                Kind::Headset => '\u{e311}',
            }
        }
    }

    impl Endpoint {
        /// The kind of an OUTPUT; for an input use `input_kind`.
        pub fn kind(&self) -> Kind {
            kind_of(&self.name)
        }

        /// The kind of an INPUT (round 169).
        pub fn input_kind(&self) -> Kind {
            kind_of_input(&self.name)
        }
    }

    /// The output sound goes to now.
    pub fn default_output() -> Option<Endpoint> {
        endpoints().0.into_iter().find(|e| e.standard)
    }

    /// The input that listens now (round 169).
    pub fn default_input() -> Option<Endpoint> {
        endpoints().1.into_iter().find(|e| e.standard)
    }

    /// LOUDNESS on a meter (round 169): a linear peak 0..1 as 0..1 over
    /// the decibels from −60 dBFS (empty) to 0 dBFS (full). A linear bar
    /// barely moved for the duo's microphone (speech peaks −19 dBFS = 11 %);
    /// in decibels the same voice fills two thirds — as a meter should.
    pub fn loudness(peak: f32) -> f32 {
        if peak <= 0.001 {
            return 0.0;
        }
        ((20.0 * peak.log10() + 60.0) / 60.0).clamp(0.0, 1.0)
    }

    #[cfg(test)]
    mod kind_tests {
        use super::*;

        #[test]
        fn a_tv_is_a_display_and_the_duo_a_speaker() {
            assert_eq!(kind_of("Fernseher"), Kind::Display);
            assert_eq!(kind_of("HDA Intel HDMI 0"), Kind::Display);
            assert_eq!(kind_of("Surface Duo Lautsprecher"), Kind::Speaker);
            assert_eq!(kind_of("Sony WH-1000XM4 (bluez)"), Kind::Bluetooth);
            assert_eq!(kind_of("USB-Kopfhörer"), Kind::Headphones);
            assert_ne!(Kind::Display.glyph(), Kind::Speaker.glyph());
        }

        #[test]
        fn inputs_are_microphones() {
            assert_eq!(kind_of_input("Surface Duo Mikrofon"), Kind::Microphone);
            assert_eq!(kind_of_input("USB Audio Analog Stereo"), Kind::Headset);
            assert_eq!(kind_of_input("Headset-Mikrofon"), Kind::Headset);
            assert_eq!(kind_of_input("Sony WH-1000XM4 (bluez)"), Kind::Bluetooth);
            assert_eq!(kind_of_input("HDA Intel HDMI"), Kind::Microphone, "an input is never a display");
        }

        #[test]
        fn loudness_is_decibels() {
            assert_eq!(loudness(0.0), 0.0);
            assert!((loudness(1.0) - 1.0).abs() < 1e-6);
            // The duo's speech peaks (−19 dBFS) fill about two thirds.
            let speech = loudness(10f32.powf(-19.0 / 20.0));
            assert!(speech > 0.6 && speech < 0.75, "{speech}");
            assert!(loudness(0.0005) == 0.0, "below −60 dBFS is silence");
        }
    }

    /// Output/input targets for `volume`/`set_volume`.
    pub const OUTPUT: &str = "@DEFAULT_AUDIO_SINK@";
    pub const INPUT: &str = "@DEFAULT_AUDIO_SOURCE@";

    /// `wpctl` (English words — its status is parsed).
    pub fn wpctl(args: &[&str]) -> Option<String> {
        let out = std::process::Command::new("wpctl").args(args).env("LC_ALL", "C.UTF-8").env("LANGUAGE", "C").output().ok()?;
        out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// WHAT CHANGED IN SOUND (round 46): a counter that grows whenever the
    /// sound server reports a change on a sink, a source or the server
    /// (default device) — one `pactl subscribe` per process, started on the
    /// first ask. Read the volume only when it moved: the sound hooks
    /// spawned `wpctl` four times a second (≈ 0.2 s CPU each — nearly a
    /// whole core on the Duo, measured 03.11.2026). Without pactl the
    /// counter ticks every 3 s, so readers still refresh.
    pub fn generation() -> u64 {
        use std::sync::atomic::AtomicU64;
        static GEN: AtomicU64 = AtomicU64::new(1);
        static STARTED: std::sync::Once = std::sync::Once::new();
        STARTED.call_once(|| {
            // Round 49: the kit's follow loop (reconnect + reaping).
            crate::process::follow(
                || {
                    let mut c = pactl();
                    c.arg("subscribe");
                    c
                },
                std::time::Duration::from_secs(3),
                |line| {
                    if line.is_none_or(event_counts) {
                        GEN.fetch_add(1, Ordering::Relaxed);
                    }
                    true
                },
            );
        });
        GEN.load(Ordering::Relaxed)
    }

    /// A `pactl subscribe` line (run through `pactl()`, English words):
    /// «Event 'new' on sink-input #45» → ("new", "sink-input", 45). Round
    /// 47: the hearing guard and the sound generation read the same lines.
    pub fn parse_event(line: &str) -> Option<(&str, &str, u64)> {
        let rest = line.strip_prefix("Event '")?;
        let (kind, rest) = rest.split_once("' on ")?;
        let (what, idx) = rest.split_once(" #")?;
        Some((kind, what, idx.trim().parse().ok()?))
    }

    /// A `pactl subscribe` line that may move a volume (pure — tested).
    pub fn event_counts(line: &str) -> bool {
        parse_event(line).is_some_and(|(_, what, _)| matches!(what, "sink" | "source" | "server" | "card"))
    }

    #[cfg(test)]
    mod generation_tests {
        #[test]
        fn sink_and_server_count_clients_do_not() {
            assert!(super::event_counts("Event 'change' on sink #55"));
            assert!(super::event_counts("Event 'change' on server #4294967295"));
            assert!(super::event_counts("Event 'new' on source #12"));
            assert!(!super::event_counts("Event 'new' on client #262740"));
            assert!(!super::event_counts("Event 'change' on sink-input #262748"));
            assert_eq!(super::parse_event("Event 'new' on sink-input #45"), Some(("new", "sink-input", 45)));
            assert_eq!(super::parse_event("Hallo"), None);
        }
    }

    /// `pactl` in English but with UTF-8 names (the session speaks German:
    /// «Ereignis »neu« auf …»; plain C refuses umlauts) — the hearing
    /// guard's lesson of 02.11.2026.
    pub fn pactl() -> std::process::Command {
        let mut c = std::process::Command::new("pactl");
        c.env("LC_ALL", "C.UTF-8").env("LANG", "C.UTF-8").env("LANGUAGE", "C");
        c
    }

    /// `pactl -f json list <what>` (sinks, sources, sink-inputs …).
    pub fn pactl_json(what: &str) -> Vec<serde_json::Value> {
        pactl()
            .args(["-f", "json", "list", what])
            .output()
            .ok()
            .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    }

    /// (outputs, inputs) from a `wpctl status` text (pure — tested).
    pub fn endpoints_from(text: &str) -> (Vec<Endpoint>, Vec<Endpoint>) {
        let (mut sinks, mut sources) = (Vec::new(), Vec::new());
        let mut block = "";
        for line in text.lines() {
            if line.contains("Sinks:") {
                block = "sinks";
                continue;
            } else if line.contains("Sources:") {
                block = "sources";
                continue;
            } else if line.trim() == "Video" {
                // Video has its own «Sources:» (cameras) — no audio after.
                break;
            } else if line.contains("Filters:") || line.contains("Streams:") || line.contains("Devices:") {
                block = "";
                continue;
            }
            if block.is_empty() {
                continue;
            }
            let core = line.trim_start_matches([' ', '\u{2502}', '\u{251c}', '\u{2514}', '\u{2500}']);
            let standard = core.starts_with('*');
            let core = core.trim_start_matches('*').trim_start();
            let Some((id_part, rest)) = core.split_once('.') else { continue };
            let Ok(id) = id_part.trim().parse::<u32>() else { continue };
            let name = rest.split("[vol:").next().unwrap_or(rest).trim().to_string();
            if name.is_empty() {
                continue;
            }
            let e = Endpoint { id, name, standard };
            if block == "sinks" {
                sinks.push(e);
            } else {
                sources.push(e);
            }
        }
        (sinks, sources)
    }

    /// (outputs, inputs) as PipeWire has them now.
    pub fn endpoints() -> (Vec<Endpoint>, Vec<Endpoint>) {
        wpctl(&["status"]).map(|t| endpoints_from(&t)).unwrap_or_default()
    }

    /// A target's volume, 0..1 (1.0 = 100 %).
    pub fn volume(target: &str) -> f32 {
        wpctl(&["get-volume", target]).and_then(|s| s.split_whitespace().nth(1).and_then(|v| v.parse().ok())).unwrap_or(0.0)
    }

    pub fn set_volume(target: &str, v: f32) {
        wpctl(&["set-volume", target, &format!("{:.2}", v.clamp(0.0, 1.5))]);
    }

    /// Make an endpoint the default (output or input, by its id).
    pub fn set_default(id: u32) {
        wpctl(&["set-default", &id.to_string()]);
    }

    /// Play a sound file (pw-play), without leaving a zombie.
    pub fn play(path: &Path) {
        let mut cmd = std::process::Command::new("pw-play");
        cmd.arg(path);
        crate::process::spawn_reaped(&mut cmd);
    }

    /// A test tone (0.5 s, 440 Hz) on one channel (0 left, 1 right) as a
    /// WAV in the runtime directory — for «Lautsprecher prüfen».
    pub fn test_tone(channel: usize) -> Option<PathBuf> {
        let path = crate::runtime::path(&format!("matrix-test-tone-{channel}.wav"));
        let rate: u32 = 44_100;
        let n = rate / 2;
        let data_len = n * 4;
        let mut wav = Vec::with_capacity(44 + data_len as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_len).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&rate.to_le_bytes());
        wav.extend_from_slice(&(rate * 4).to_le_bytes());
        wav.extend_from_slice(&4u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_len.to_le_bytes());
        for i in 0..n {
            // A gentle envelope: no click at the ends.
            let hull = (i.min(n - i) as f32 / 2000.0).min(1.0);
            let s = (i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 0.4 * hull;
            let v = (s * i16::MAX as f32) as i16;
            for w in [if channel == 0 { v } else { 0 }, if channel == 1 { v } else { 0 }] {
                wav.extend_from_slice(&w.to_le_bytes());
            }
        }
        std::fs::write(&path, wav).ok()?;
        Some(path)
    }

    /// A MICROPHONE LEVEL METER: pw-record's peak per 100 ms as 0..1 —
    /// `start`, read `level`, `stop` (also when dropped).
    #[derive(Default)]
    pub struct MicMeter {
        level: Arc<AtomicU32>,
        on: Arc<AtomicBool>,
    }

    impl MicMeter {
        pub fn running(&self) -> bool {
            self.on.load(Ordering::SeqCst)
        }

        /// 0..1, the loudest of the last tenth of a second (linear).
        pub fn level(&self) -> f32 {
            self.level.load(Ordering::SeqCst) as f32 / 1000.0
        }

        /// The same peak in decibels for a meter (round 169, `loudness`).
        pub fn loudness(&self) -> f32 {
            loudness(self.level())
        }

        pub fn start(&self) {
            if self.on.swap(true, Ordering::SeqCst) {
                return;
            }
            let (level, on) = (Arc::clone(&self.level), Arc::clone(&self.on));
            std::thread::spawn(move || {
                use std::io::Read;
                let child = std::process::Command::new("pw-record").args(["--rate", "8000", "--channels", "1", "--format", "s16", "-"]).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null()).spawn();
                let Ok(mut child) = child else {
                    on.store(false, Ordering::SeqCst);
                    return;
                };
                if let Some(mut pipe) = child.stdout.take() {
                    let mut buf = [0u8; 1600];
                    while on.load(Ordering::SeqCst) && pipe.read_exact(&mut buf).is_ok() {
                        let peak = buf.chunks_exact(2).map(|p| (i16::from_le_bytes([p[0], p[1]]) as i32).abs()).max().unwrap_or(0);
                        // Round 169: per mille — per cent swallowed quiet microphones (< 1 %).
                        level.store((peak * 1000 / i16::MAX as i32) as u32, Ordering::SeqCst);
                    }
                }
                on.store(false, Ordering::SeqCst);
                let _ = child.kill();
                let _ = child.wait();
                level.store(0, Ordering::SeqCst);
            });
        }

        pub fn stop(&self) {
            self.on.store(false, Ordering::SeqCst);
        }
    }

    impl Drop for MicMeter {
        fn drop(&mut self) {
            self.stop();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn reads_wpctl_status() {
            let text = "Audio\n ├─ Devices:\n │      51. Internes Audio  [alsa:pcm]\n ├─ Sinks:\n │      53. Surface Duo Lautsprecher [vol: 0.35]\n │  *   88. Buds3 Pro [vol: 0.40]\n ├─ Sources:\n │  *   87. Buds3 Pro [vol: 1.00]\n ├─ Filters:\n │      57. loop\nVideo\n ├─ Sources:\n │  *   99. Kamera\n";
            let (sinks, sources) = endpoints_from(text);
            assert_eq!(sinks.len(), 2);
            assert!(sinks[1].standard && sinks[1].name == "Buds3 Pro" && sinks[1].id == 88);
            assert_eq!(sources, vec![Endpoint { id: 87, name: "Buds3 Pro".into(), standard: true }]);
        }
    }
}

pub mod control {
    pub const BAR_HEIGHT: f32 = 48.0;
    pub const BUTTON_HEIGHT: f32 = 40.0;
    pub const BUTTON_MIN_WIDTH: f32 = 64.0;
    pub const SLIDER_HEIGHT: f32 = 48.0;
    pub const TOGGLE_TRACK_W: f32 = 52.0;
    pub const TOGGLE_TRACK_H: f32 = 30.0;
    pub const TOGGLE_THUMB: f32 = 24.0;
}

/// Material state layers: opacity of the text color OVER the surface.
pub mod state_layer {
    pub const HOVER: f32 = 0.12;
    pub const PRESSED: f32 = 0.20;
}

// ============================================================================
// Bewegung — Dauern, Material-Kurven, Feder (Quelle: DMS Anims.qml)
// ============================================================================

pub mod motion {
    /// How long the ring of a tap answer glows (seconds) — the dock's flash.
    pub const TAP_FLASH_S: f32 = 0.45;

    /// TAP FLASH (R209 family, iced-free since 01.11.2026): the touch answer
    /// of elements that are their own surfaces (dock tiles, bar chips) — a
    /// short buzz and an edge that lights up and fades over `TAP_FLASH_S`.
    /// Keep one, call `tap` on a tap, `glow` while drawing, `tick` from a
    /// ~60 ms timer while `is_active()`.
    pub struct TapFlash<K> {
        taps: Vec<(K, std::time::Instant)>,
    }

    impl<K> Default for TapFlash<K> {
        fn default() -> Self {
            TapFlash { taps: Vec::new() }
        }
    }

    impl<K: PartialEq + Clone> TapFlash<K> {
        /// A tap on `key`: buzz once, light its edge.
        pub fn tap(&mut self, key: K) {
            crate::feel::tap();
            self.taps.retain(|(k, _)| *k != key);
            self.taps.push((key, std::time::Instant::now()));
        }

        /// When `key` was tapped, while its glow still runs.
        pub fn since(&self, key: &K) -> Option<std::time::Instant> {
            self.taps.iter().find(|(k, _)| k == key).map(|(_, t)| *t)
        }

        /// Is `key` still answering a tap?
        pub fn contains(&self, key: &K) -> bool {
            self.since(key).is_some()
        }

        /// How brightly `key`'s edge glows now (0..1).
        pub fn glow(&self, key: &K) -> f32 {
            tap_glow(self.since(key), TAP_FLASH_S, None)
        }

        /// Drop the glows that have faded — call from the timer.
        pub fn tick(&mut self) {
            self.taps.retain(|(_, t)| t.elapsed().as_secs_f32() < TAP_FLASH_S);
        }

        /// Does a glow still run (the timer is needed)?
        pub fn is_active(&self) -> bool {
            !self.taps.is_empty()
        }
    }

    /// TAP ANSWER (R209): how brightly an element answers a tap — a fading
    /// flash for `flash_s` seconds after `tapped`, a calm pulse (1.2 s)
    /// while `starting` (an app on its way). 0..1; reduced motion = 0.
    pub fn tap_glow(
        tapped: Option<std::time::Instant>,
        flash_s: f32,
        starting: Option<std::time::Instant>,
    ) -> f32 {
        if crate::motion_reduced() {
            return 0.0;
        }
        let flash = tapped.map_or(0.0, |t| (1.0 - t.elapsed().as_secs_f32() / flash_s).max(0.0));
        let start = starting.map_or(0.0, |t| {
            0.55 + 0.45 * (t.elapsed().as_secs_f32() * std::f32::consts::TAU / 1.2).cos()
        });
        flash.max(start)
    }

    /// Dauern in Millisekunden.
    pub const DUR_SHORT: u64 = 200;
    pub const DUR_MED: u64 = 450;
    pub const DUR_LONG: u64 = 600;
    /// Default distance for slide-in (px).
    pub const SLIDE_PX: f32 = 80.0;

    /// Cubic bezier curve (x1, y1, x2, y2) — like CSS cubic-bezier.
    #[derive(Clone, Copy, Debug)]
    pub struct Bezier(pub f32, pub f32, pub f32, pub f32);

    pub const STANDARD: Bezier = Bezier(0.20, 0.00, 0.00, 1.00);
    pub const STANDARD_DECEL: Bezier = Bezier(0.00, 0.00, 0.00, 1.00);
    pub const STANDARD_ACCEL: Bezier = Bezier(0.30, 0.00, 1.00, 1.00);
    pub const EMPH_DECEL: Bezier = Bezier(0.05, 0.70, 0.10, 1.00);
    pub const EMPH_ACCEL: Bezier = Bezier(0.30, 0.00, 0.80, 0.15);
    /// Spatial motion with overshoot (y1 > 1!) — popouts, dock hover.
    pub const EXPRESSIVE_SPATIAL: Bezier = Bezier(0.38, 1.21, 0.22, 1.00);
    pub const EXPRESSIVE_FAST: Bezier = Bezier(0.34, 1.50, 0.20, 1.00);
    pub const EXPRESSIVE_FX: Bezier = Bezier(0.34, 0.80, 0.34, 1.00);

    impl Bezier {
        /// Evaluates the curve at progress t ∈ [0,1] (Newton iteration over x).
        pub fn eval(&self, t: f32) -> f32 {
            if t <= 0.0 { return 0.0; }
            if t >= 1.0 { return 1.0; }
            let (x1, y1, x2, y2) = (self.0, self.1, self.2, self.3);
            let cubic = |a: f32, b: f32, s: f32| {
                // Bezier with P0=0, P3=1: 3(1-s)²s·a + 3(1-s)s²·b + s³
                3.0 * (1.0 - s) * (1.0 - s) * s * a + 3.0 * (1.0 - s) * s * s * b + s * s * s
            };
            // find s such that x(s) = t
            let mut s = t;
            for _ in 0..8 {
                let x = cubic(x1, x2, s) - t;
                let dx = 3.0 * (1.0 - s) * (1.0 - s) * x1
                    + 6.0 * (1.0 - s) * s * (x2 - x1)
                    + 3.0 * s * s * (1.0 - x2);
                if dx.abs() < 1e-6 { break; }
                s = (s - x / dx).clamp(0.0, 1.0);
            }
            cubic(y1, y2, s)
        }
    }

    /// The TWEEN (R202 D.2): a synchronous flight from `from` to `to`
    /// in a fixed duration on a curve — unlike the spring, a
    /// hundred tweens of the same duration arrive in the SAME frame (radial → tree,
    /// staggered entrance). `retarget` starts from the current value
    /// without a jump; `staggered` delays per index.
    #[derive(Clone, Copy, Debug)]
    pub struct Tween {
        pub from: f32,
        pub to: f32,
        /// Progress of the flight 0..1 (1 = arrived).
        pub t: f32,
        /// Dauer in Sekunden.
        pub duration: f32,
        /// Remaining delay in seconds before the start.
        pub delay: f32,
        pub curve: Bezier,
    }

    impl Tween {
        /// Rests at `value` — no flight.
        pub fn at(value: f32) -> Self {
            Self { from: value, to: value, t: 1.0, duration: 0.0, delay: 0.0, curve: STANDARD }
        }

        /// A flight from `from` to `to` in `duration` seconds.
        pub fn fly(from: f32, to: f32, duration: f32, curve: Bezier) -> Self {
            Self { from, to, t: 0.0, duration: duration.max(0.001), delay: 0.0, curve }
        }

        /// Staggered: the i-th flight starts i·step seconds later.
        pub fn staggered(mut self, index: usize, step: f32) -> Self {
            self.delay = index as f32 * step;
            self
        }

        /// New target mid-flight: continue from the current value, without a jump.
        pub fn retarget(&mut self, to: f32, duration: f32) {
            let now = self.value();
            self.from = now;
            self.to = to;
            self.t = 0.0;
            self.duration = duration.max(0.001);
            self.delay = 0.0;
        }

        /// One time step; `true` as long as the flight is still running.
        pub fn tick(&mut self, dt: f32) -> bool {
            let mut dt = dt.max(0.0);
            if self.delay > 0.0 {
                self.delay -= dt;
                if self.delay > 0.0 {
                    return true;
                }
                dt = -self.delay;
                self.delay = 0.0;
            }
            if self.t >= 1.0 {
                return false;
            }
            self.t = (self.t + dt / self.duration).min(1.0);
            true
        }

        pub fn value(&self) -> f32 {
            if self.t >= 1.0 {
                return self.to;
            }
            self.from + (self.to - self.from) * self.curve.eval(self.t)
        }

        pub fn is_settled(&self) -> bool {
            self.t >= 1.0 && self.delay <= 0.0
        }
    }

    /// Interruptible spring — the core of the reference feel.
    /// The target can change at any time; motion continues seamlessly from
    /// the current state (position AND velocity).
    #[derive(Clone, Copy, Debug)]
    pub struct Spring {
        pub value: f32,
        pub velocity: f32,
        pub target: f32,
        pub stiffness: f32,
        pub damping: f32,
    }

    impl Spring {
        pub fn new(value: f32) -> Self {
            Self { value, velocity: 0.0, target: value, stiffness: 170.0, damping: 22.0 }
        }
        /// Leitbild-UI-Parametrisierung (Referenz-SDK, swiftinterface-verifiziert):
        /// `dauer` = wahrgenommene Dauer in s, `sprung` ∈ [-1..1] = Nachschwung.
        /// stiffness = (2π/duration)², damping = (1−bounce)·4π/duration for bounce ≥ 0,
        /// else 4π/(duration·(1+bounce)).
        pub fn with(value: f32, duration: f32, bounce: f32) -> Self {
            let omega = 2.0 * std::f32::consts::PI / duration;
            let damping = if bounce >= 0.0 {
                (1.0 - bounce) * 4.0 * std::f32::consts::PI / duration
            } else {
                4.0 * std::f32::consts::PI / (duration * (1.0 + bounce))
            };
            Self { value, velocity: 0.0, target: value, stiffness: omega * omega, damping }
        }
        /// the reference `smooth`: no overshoot (0.5 s / bounce 0).
        pub fn smooth(value: f32) -> Self { Self::with(value, 0.5, 0.0) }
        /// the reference `snappy`: slight overshoot (0.5 s / bounce 0.15).
        pub fn snappy(value: f32) -> Self { Self::with(value, 0.5, 0.15) }
        /// the reference `bouncy`: clear overshoot (0.5 s / bounce 0.3).
        pub fn springy(value: f32) -> Self { Self::with(value, 0.5, 0.3) }
        pub fn retarget(&mut self, target: f32) { self.target = target; }
        /// The current value — reader for everything that derives an
        /// effect from the spring (R192: the entrance in space).
        pub fn value(&self) -> f32 { self.value }
        /// true once the spring has reached its target and rests.
        pub fn is_settled(&self) -> bool {
            self.velocity.abs() < 0.001 && (self.target - self.value).abs() < 0.001
        }
        /// Integrate one time step (dt in seconds). true = still moving.
        pub fn tick(&mut self, dt: f32) -> bool {
            // Explicit Euler is only stable for SMALL steps. Wayland pauses
            // frame callbacks for occluded/idle surfaces; the first tick
            // after such a pause arrives with a huge dt and the naive
            // integration diverges — the header magnifier then grows
            // without bound until the glyph rasterizer aborts (the
            // Briefing "Lupe" crash, 26.08.2026). Cap the step and
            // integrate in fixed sub-steps; excess time beyond the cap is
            // dropped on purpose (after a long pause the spring should
            // simply be near its target, not accumulate fantasy energy).
            const MAX_DT: f32 = 0.25;
            const H: f32 = 1.0 / 240.0;
            let mut rest = if dt.is_finite() { dt.clamp(0.0, MAX_DT) } else { 0.0 };
            while rest > 0.0 {
                let h = rest.min(H);
                let force =
                    self.stiffness * (self.target - self.value) - self.damping * self.velocity;
                self.velocity += force * h;
                self.value += self.velocity * h;
                rest -= h;
            }
            if self.velocity.abs() < 0.001 && (self.target - self.value).abs() < 0.001 {
                self.value = self.target;
                self.velocity = 0.0;
                false
            } else {
                true
            }
        }
    }
}

// ============================================================================
// Live palette — follows the system wallpaper & light/dark mode
// ============================================================================

/// Color as 0..1 RGBA, framework-neutral.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    /// The same color, partially translucent (multiplies the
    /// existing alpha) — color operations live in the theme, not in apps.
    /// Linear blend (for the palette fade).
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        let m = |x: f32, y: f32| x + (y - x) * t;
        Self { r: m(a.r, b.r), g: m(a.g, b.g), b: m(a.b, b.b), a: m(a.a, b.a) }
    }

    pub fn with_alpha(self, a: f32) -> Self {
        Rgba { a: self.a * a, ..self }
    }

    /// R207: hue (0–360), saturation and lightness (0–1) — for
    /// deriving a palette from a seed in the host land.
    pub fn to_hsl(self) -> (f32, f32, f32) {
        let (r, g, b) = (self.r, self.g, self.b);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < 1e-6 {
            return (0.0, 0.0, l);
        }
        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        let h = if max == r {
            ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0
        } else if max == g {
            ((b - r) / d + 2.0) * 60.0
        } else {
            ((r - g) / d + 4.0) * 60.0
        };
        (h, s, l)
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = ((h % 360.0) + 360.0) % 360.0 / 360.0;
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);
        if s < 1e-6 {
            return Rgba { r: l, g: l, b: l, a: 1.0 };
        }
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        let f = |mut t: f32| {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        };
        Rgba { r: f(h + 1.0 / 3.0), g: f(h), b: f(h - 1.0 / 3.0), a: 1.0 }
    }

    /// «#rrggbb» (round 166: the login share writes the seed so).
    pub fn to_hex(self) -> String {
        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", c(self.r), c(self.g), c(self.b))
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let h = hex.trim_start_matches('#');
        let (r, g, b, a) = match h.len() {
            6 => (
                u8::from_str_radix(&h[0..2], 16).ok()?,
                u8::from_str_radix(&h[2..4], 16).ok()?,
                u8::from_str_radix(&h[4..6], 16).ok()?,
                255u8,
            ),
            8 => (
                u8::from_str_radix(&h[0..2], 16).ok()?,
                u8::from_str_radix(&h[2..4], 16).ok()?,
                u8::from_str_radix(&h[4..6], 16).ok()?,
                u8::from_str_radix(&h[6..8], 16).ok()?,
            ),
            _ => return None,
        };
        Some(Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        })
    }

    /// Hex notation (#rrggbb) — e.g. for display and the clipboard.
    pub fn hex(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8
        )
    }

    /// State layer: lay this color over a base at the given opacity.
    pub fn over(&self, base: Rgba, alpha: f32) -> Rgba {
        Rgba {
            r: self.r * alpha + base.r * (1.0 - alpha),
            g: self.g * alpha + base.g * (1.0 - alpha),
            b: self.b * alpha + base.b * (1.0 - alpha),
            a: 1.0,
        }
    }
}

/// The Material 3 roles MatrixKit apps need.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub is_light: bool,
    pub primary: Rgba,
    pub on_primary: Rgba,
    pub primary_container: Rgba,
    pub on_primary_container: Rgba,
    pub secondary: Rgba,
    pub tertiary: Rgba,
    pub surface: Rgba,
    pub on_surface: Rgba,
    pub on_surface_variant: Rgba,
    pub surface_container: Rgba,
    pub surface_container_high: Rgba,
    pub outline: Rgba,
    pub error: Rgba,
}

impl Default for Palette {
    /// Fallback (dark, neutral tones) if DMS data is missing.
    fn default() -> Self {
        let hex = |s| Rgba::from_hex(s).unwrap();
        Self {
            is_light: false,
            primary: hex("#cbcb76"),
            on_primary: hex("#323200"),
            primary_container: hex("#4a4a21"),
            on_primary_container: hex("#e8e78f"),
            // Fallback like the real case: secondary/tertiary follow primary
            secondary: hex("#cbcb76"),
            tertiary: hex("#cbcb76"),
            surface: hex("#14140c"),
            on_surface: hex("#e6e3d5"),
            on_surface_variant: hex("#cac7b6"),
            surface_container: hex("#202018"),
            surface_container_high: hex("#2b2b22"),
            outline: hex("#939182"),
            error: hex("#ffb4ab"),
        }
    }
}

#[derive(Deserialize)]
struct DmsColorsFile {
    colors: DmsModes,
}
#[derive(Deserialize)]
struct DmsModes {
    dark: serde_json::Map<String, serde_json::Value>,
    light: serde_json::Map<String, serde_json::Value>,
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/root".into()))
}

pub fn colors_path() -> PathBuf {
    home().join(".cache/DankMaterialShell/dms-colors.json")
}

/// R107: the wallpaper-path contract. The stored image path per mode
/// lives in ~/.config/matrix/wallpaper-light|dark; pre-R107 files
/// (hintergrund-hell|dunkel) are read as legacy and rewritten English.
pub mod wallpaper {
    use super::home;

    pub fn key(light: bool) -> &'static str {
        if light { "wallpaper-light" } else { "wallpaper-dark" }
    }

    /// Is the session in light mode? (session.json, the same flag the
    /// toggle writes.)
    pub fn light_mode() -> bool {
        std::fs::read_to_string(super::session_path())
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v.get("isLightMode").and_then(|b| b.as_bool()))
            .unwrap_or(false)
    }

    /// THE image the canvas currently wears — one definition for the
    /// whole family (R152). The lock screen must never show a different
    /// picture than the desktop behind it, so it asks here instead of
    /// carrying its own cascade: own contract first, the pre-Matrix
    /// session field second, the shipped default last.
    pub fn current() -> Option<String> {
        let exists = |p: &str| !p.is_empty() && std::path::Path::new(p).exists();
        if let Some(p) = read(light_mode()) {
            if exists(&p) {
                return Some(p);
            }
        }
        if let Ok(raw) = std::fs::read_to_string(super::session_path()) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(p) = v.get("wallpaperPath").and_then(|s| s.as_str()) {
                    if exists(p) {
                        return Some(p.to_string());
                    }
                }
            }
        }
        let shipped = "/usr/share/backgrounds/matrix/matrix-standard.jpg";
        exists(shipped).then(|| shipped.to_string())
    }
    fn legacy_key(light: bool) -> &'static str {
        if light { "hintergrund-hell" } else { "hintergrund-dunkel" }
    }

    /// The stored image path for the mode — English file first, the
    /// German legacy file as fallback. None if neither is set.
    pub fn read(light: bool) -> Option<String> {
        let dir = home().join(".config/matrix");
        for k in [key(light), legacy_key(light)] {
            if let Ok(s) = std::fs::read_to_string(dir.join(k)) {
                let t = s.trim().to_string();
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
        None
    }

    /// Write the image path — always to the English file.
    pub fn write(light: bool, path: &str) {
        // R148: atomic like settings::write — watchers read these live.
        let dir = home().join(".config/matrix");
        let _ = std::fs::create_dir_all(&dir);
        let target = dir.join(key(light));
        let tmp = target.with_extension("new");
        if std::fs::write(&tmp, path).is_ok() {
            let _ = std::fs::rename(&tmp, &target);
        }
    }
}

pub fn session_path() -> PathBuf {
    home().join(".local/state/DankMaterialShell/session.json")
}

/// Toggle light/dark — WITHOUT DMS: flip isLightMode in session.json.
/// The palette carries both modes; all watchers pick it up instantly.
pub fn light_toggle() {
    light_set(!is_light_mode());
}

/// Set light/dark explicitly (R116: the settings' fixed appearance
/// modes use this — same session.json contract as the toggle).
pub fn light_set(light: bool) {
    let path = session_path();
    let mut v: serde_json::Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    v["isLightMode"] = serde_json::Value::Bool(light);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, v.to_string());
}

fn is_light_mode() -> bool {
    if let Some(v) = std::fs::read_to_string(session_path())
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("isLightMode").and_then(|b| b.as_bool()))
    {
        return v;
    }
    if habitat::is_matrix() {
        return false;
    }
    seed::desktop_prefers_light()
}

/// R207: the SEED in the host land — the colour the palette grows from
/// when matugen is missing. Order: the window house's seed file (computed from
/// the wallpaper), the desktop's accent colour (GNOME,
/// KDE), otherwise nothing (default palette).
pub mod seed {
    use super::Rgba;

    pub fn seed_path() -> std::path::PathBuf {
        let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
        std::path::PathBuf::from(dir).join("matrix-seed")
    }

    pub fn seed() -> Option<Rgba> {
        if let Ok(hex) = std::fs::read_to_string(seed_path()) {
            if let Some(c) = Rgba::from_hex(hex.trim()) {
                return Some(c);
            }
        }
        if let Ok(hex) = std::env::var("MATRIX_SEED") {
            if let Some(c) = Rgba::from_hex(hex.trim()) {
                return Some(c);
            }
        }
        accent_color()
    }

    /// The foreign desktop's accent colour: GNOME (gsettings
    /// accent-color), KDE (kdeglobals AccentColor).
    pub fn accent_color() -> Option<Rgba> {
        if let Some(name) = gsettings("org.gnome.desktop.interface", "accent-color") {
            let hex = match name.trim_matches('\'').as_ref() {
                "blue" => "#3584e4",
                "teal" => "#2190a4",
                "green" => "#3a944a",
                "yellow" => "#c88800",
                "orange" => "#ed5b00",
                "red" => "#e62d42",
                "pink" => "#d56199",
                "purple" => "#9141ac",
                "slate" => "#6f8396",
                _ => "",
            };
            if let Some(c) = Rgba::from_hex(hex) {
                return Some(c);
            }
        }
        let home = std::env::var("HOME").ok()?;
        let kde = std::fs::read_to_string(format!("{home}/.config/kdeglobals")).ok()?;
        for line in kde.lines() {
            if let Some(v) = line.strip_prefix("AccentColor=") {
                let parts: Vec<f32> = v.split(',').filter_map(|t| t.trim().parse::<f32>().ok()).collect();
                if parts.len() == 3 {
                    return Some(Rgba { r: parts[0] / 255.0, g: parts[1] / 255.0, b: parts[2] / 255.0, a: 1.0 });
                }
            }
        }
        None
    }

    /// Light or dark in the host land: GNOME color-scheme, otherwise KDE
    /// (ColorScheme with «Light»/«Dark» in the name), otherwise dark.
    pub fn desktop_prefers_light() -> bool {
        static CACHE: std::sync::Mutex<Option<(std::time::Instant, bool)>> = std::sync::Mutex::new(None);
        if let Ok(c) = CACHE.lock() {
            if let Some((t, v)) = *c {
                if t.elapsed().as_secs() < 5 {
                    return v;
                }
            }
        }
        let v = desktop_prefers_light_now();
        if let Ok(mut c) = CACHE.lock() {
            *c = Some((std::time::Instant::now(), v));
        }
        v
    }

    fn desktop_prefers_light_now() -> bool {
        if let Some(v) = gsettings("org.gnome.desktop.interface", "color-scheme") {
            return v.contains("prefer-light") || v.contains("default");
        }
        if let Ok(home) = std::env::var("HOME") {
            if let Ok(kde) = std::fs::read_to_string(format!("{home}/.config/kdeglobals")) {
                for line in kde.lines() {
                    if let Some(v) = line.strip_prefix("ColorScheme=") {
                        return v.to_lowercase().contains("light");
                    }
                }
            }
        }
        false
    }

    /// The wallpaper path in the host land (GNOME, KDE) — the
    /// image machine computes the seed from it.
    pub fn wallpaper_path() -> Option<std::path::PathBuf> {
        let light = desktop_prefers_light();
        let keys: &[&str] = if light { &["picture-uri", "picture-uri-dark"] } else { &["picture-uri-dark", "picture-uri"] };
        for key in keys {
            if let Some(uri) = gsettings("org.gnome.desktop.background", key) {
                let uri = uri.trim().trim_matches('\'');
                let path = uri.strip_prefix("file://").unwrap_or(uri);
                let path = percent_decode(path);
                let p = std::path::PathBuf::from(path);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        let home = std::env::var("HOME").ok()?;
        let plasma = std::fs::read_to_string(format!("{home}/.config/plasma-org.kde.plasma.desktop-appletsrc")).ok()?;
        for line in plasma.lines() {
            if let Some(v) = line.strip_prefix("Image=") {
                let path = v.strip_prefix("file://").unwrap_or(v);
                let p = std::path::PathBuf::from(percent_decode(path));
                if p.is_file() {
                    return Some(p);
                }
                // A KDE image package: contents/images/<largest>.
                let images = p.join("contents/images");
                if let Ok(rd) = std::fs::read_dir(&images) {
                    let mut best: Option<(u64, std::path::PathBuf)> = None;
                    for e in rd.flatten() {
                        let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                        if best.as_ref().map_or(true, |(s, _)| size > *s) {
                            best = Some((size, e.path()));
                        }
                    }
                    if let Some((_, p)) = best {
                        return Some(p);
                    }
                }
            }
        }
        None
    }

    fn gsettings(schema: &str, key: &str) -> Option<String> {
        let out = std::process::Command::new("gsettings").args(["get", schema, key]).output().ok()?;
        if !out.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    fn percent_decode(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            // Round 178: parse the bytes, never slice `s` (`%ä` in a KDE wallpaper path panicked).
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                if let Some(v) = std::str::from_utf8(&bytes[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    out.push(v);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    #[cfg(test)]
    mod percent_tests {
        #[test]
        fn percent_codes_beside_non_ascii_letters_decode_safely() {
            assert_eq!(super::percent_decode("/home/a/B%C3%A4ume.jpg"), "/home/a/Bäume.jpg");
            assert_eq!(super::percent_decode("/home/a/100%äpfel.jpg"), "/home/a/100%äpfel.jpg");
            assert_eq!(super::percent_decode("ende%"), "ende%");
        }
    }
}

/// The palette fade (user, 2026-07-08): on wallpaper change the
/// system does not jump to the new colors, it GLIDES (~450 ms).
/// As a filter in Palette::load(): the static state remembers
/// from/target/start; while the transition runs, load() returns
/// intermediate palettes and PaletteWatcher::changed() stays true — every
/// polling surface fades automatically. bewegung=reduziert switches hard.
pub mod transition {
    use super::*;
    use std::sync::{OnceLock, RwLock};

    pub const DURATION_MS: u128 = 450;

    struct Fade {
        from: Palette,
        target: Palette,
        start: std::time::Instant,
    }
    fn cell() -> &'static RwLock<Option<Fade>> {
        static Z: OnceLock<RwLock<Option<Fade>>> = OnceLock::new();
        Z.get_or_init(|| RwLock::new(None))
    }

    /// Is a fade running? (Drives watchers + fast ticks.)
    pub fn active() -> bool {
        // +80 ms buffer: the LAST tick after t=1 still has to fire
        // so the surface lands exactly on the target (otherwise it would
        // hang on the 97 % intermediate palette).
        cell()
            .read()
            .ok()
            .and_then(|g| g.as_ref().map(|u| u.start.elapsed().as_millis() < DURATION_MS + 80))
            .unwrap_or(false)
    }

    pub(super) fn filter(raw: Palette) -> Palette {
        if super::motion_reduced() {
            *cell().write().unwrap() = None;
            return raw;
        }
        let mut g = cell().write().unwrap();
        match g.as_mut() {
            None => {
                // First contact: adopt silently (app start does not fade).
                *g = Some(Fade { from: raw, target: raw, start: std::time::Instant::now() });
                raw
            }
            Some(u) => {
                let t = (u.start.elapsed().as_millis() as f32 / DURATION_MS as f32).min(1.0);
                let is = Palette::lerp(u.from, u.target, t);
                if raw != u.target {
                    // New target: keep gliding from the current actual value.
                    u.from = is;
                    u.target = raw;
                    u.start = std::time::Instant::now();
                    return is;
                }
                if t >= 1.0 { u.target } else { is }
            }
        }
    }
}

impl Palette {
    /// Linear blend — is_light snaps hard at the midpoint (bool).
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        let m = |x: Rgba, y: Rgba| Rgba::lerp(x, y, t);
        Palette {
            is_light: if t < 0.5 { a.is_light } else { b.is_light },
            primary: m(a.primary, b.primary),
            on_primary: m(a.on_primary, b.on_primary),
            primary_container: m(a.primary_container, b.primary_container),
            on_primary_container: m(a.on_primary_container, b.on_primary_container),
            secondary: m(a.secondary, b.secondary),
            tertiary: m(a.tertiary, b.tertiary),
            surface: m(a.surface, b.surface),
            on_surface: m(a.on_surface, b.on_surface),
            on_surface_variant: m(a.on_surface_variant, b.on_surface_variant),
            surface_container: m(a.surface_container, b.surface_container),
            surface_container_high: m(a.surface_container_high, b.surface_container_high),
            outline: m(a.outline, b.outline),
            error: m(a.error, b.error),
        }
    }

    /// Loads the current system palette (DMS/matugen). None if files are missing.
    /// R207: in the host land (foreign desktop) there is no matugen file —
    /// then the palette comes from the SEED of the wallpaper or the
    /// desktop's accent colour (`seed::seed()`), derived like Material.
    pub fn load() -> Option<Self> {
        let raw = match std::fs::read_to_string(colors_path()) {
            Ok(raw) => raw,
            Err(_) => {
                if habitat::is_matrix() {
                    return None;
                }
                let light = is_light_mode();
                let seed = seed::seed()?;
                return Some(transition::filter(Self::with_contrast(Self::from_seed(seed, light))));
            }
        };
        let file: DmsColorsFile = serde_json::from_str(&raw).ok()?;
        let light = is_light_mode();
        let mode = if light { &file.colors.light } else { &file.colors.dark };
        let get = |key: &str| -> Option<Rgba> {
            mode.get(key).and_then(|v| v.as_str()).and_then(Rgba::from_hex)
        };
        let d = Palette { is_light: light, ..Default::default() };
        Some(transition::filter(Self::with_contrast(Palette {
            is_light: light,
            primary: get("primary").unwrap_or(d.primary),
            on_primary: get("on_primary").unwrap_or(d.on_primary),
            primary_container: get("primary_container").unwrap_or(d.primary_container),
            on_primary_container: get("on_primary_container").unwrap_or(d.on_primary_container),
            secondary: get("secondary").unwrap_or(d.secondary),
            tertiary: get("tertiary").unwrap_or(d.tertiary),
            surface: get("surface").unwrap_or(d.surface),
            on_surface: get("on_surface").unwrap_or(d.on_surface),
            on_surface_variant: get("on_surface_variant").unwrap_or(d.on_surface_variant),
            surface_container: get("surface_container").unwrap_or(d.surface_container),
            surface_container_high: get("surface_container_high").unwrap_or(d.surface_container_high),
            outline: get("outline").unwrap_or(d.outline),
            error: get("error").unwrap_or(d.error),
        })))
    }

    /// R207: a palette from ONE seed, derived like Material You —
    /// the hue stays, saturation and lightness are set per role
    /// (OKLCH-like approximation via HSL, without HCT tables). Host-land path;
    /// on the canvas matugen computes it.
    pub fn from_seed(seed: Rgba, light: bool) -> Palette {
        let (h, s0, _) = seed.to_hsl();
        let s = s0.clamp(0.35, 0.85);
        let tone = |hue: f32, sat: f32, l: f32| Rgba::from_hsl(hue, sat, l);
        let h2 = (h + 60.0) % 360.0;
        if light {
            Palette {
                is_light: true,
                primary: tone(h, s, 0.40),
                on_primary: tone(h, s, 0.98),
                primary_container: tone(h, s, 0.90),
                on_primary_container: tone(h, s, 0.12),
                secondary: tone(h, s * 0.45, 0.42),
                tertiary: tone(h2, s * 0.7, 0.40),
                surface: tone(h, s * 0.15, 0.985),
                on_surface: tone(h, s * 0.2, 0.11),
                on_surface_variant: tone(h, s * 0.2, 0.30),
                surface_container: tone(h, s * 0.15, 0.94),
                surface_container_high: tone(h, s * 0.15, 0.91),
                outline: tone(h, s * 0.2, 0.50),
                error: Rgba::from_hex("#ba1a1a").unwrap(),
            }
        } else {
            Palette {
                is_light: false,
                primary: tone(h, s, 0.78),
                on_primary: tone(h, s, 0.18),
                primary_container: tone(h, s, 0.30),
                on_primary_container: tone(h, s, 0.90),
                secondary: tone(h, s * 0.45, 0.76),
                tertiary: tone(h2, s * 0.7, 0.78),
                surface: tone(h, s * 0.15, 0.07),
                on_surface: tone(h, s * 0.2, 0.90),
                on_surface_variant: tone(h, s * 0.2, 0.78),
                surface_container: tone(h, s * 0.15, 0.12),
                surface_container_high: tone(h, s * 0.15, 0.17),
                outline: tone(h, s * 0.2, 0.58),
                error: Rgba::from_hex("#ffb4ab").unwrap(),
            }
        }
    }

    /// ColorSchemeContrast: with kontrast=hoch the secondary tones move toward
    /// the primaries and outlines get stronger — system-wide, because ALL
    /// Apps ihre Farben hierdurch beziehen.
    fn with_contrast(mut p: Palette) -> Palette {
        if !contrast_up() {
            return p;
        }
        p.on_surface_variant = p.on_surface.over(p.on_surface_variant, 0.45);
        p.outline = p.on_surface.over(p.outline, 0.35);
        p
    }
}

impl Palette {
    /// Like `load()`, but waits out write races: the color sync fires
    /// WHILE matugen is still writing the JSON — half-written files
    /// don't parse and used to silently deliver fallback colors.
    pub fn load_settled(max_wait: std::time::Duration) -> Option<Self> {
        let start = std::time::Instant::now();
        loop {
            if let Some(p) = Self::load() {
                return Some(p);
            }
            if start.elapsed() >= max_wait {
                return None;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
}

/// Input feel (R66, measured empirically on the reference system, defaults read -g):
/// the numbers the reference documents nowhere and keeps identical everywhere.
pub mod input {
    /// Double-click window: reference default 0.5 s (threshold unset =
    /// default; matches the R33 empirics from Matrix Files).
    pub const DOUBLE_CLICK_MS: u128 = 500;
    /// Tastenwiederholung: InitialKeyRepeat 25×15 ms, KeyRepeat 6×15 ms —
    /// 375 ms delay, ~11/s. Set in niri and CONFIRMED via wev on the PC
    /// on 2026-07-16 (repeat_info: rate 11, delay 375).
    pub const KEY_REPEAT_DELAY_MS: u64 = 375;
    pub const KEY_REPEAT_RATE: u8 = 11;
    /// Menu flash: the chosen item blinks ~0.2 s BEFORE the menu
    /// closes — the click receipt of every reference menu since 1984.
    // R120 calibration: the reference blink is quick — 200 ms read as
    // lag, 120 ms keeps the off-ON-off visible and feels immediate.
    pub const MENU_FLASH_MS: u64 = 120;
    /// Drag grammar (R68): 4 pt threshold before a press becomes a drag
    /// (Leitbild-Toolkit-Konstante); Spring-Loading 0,5 s (empirisch:
    /// springing.delay = 0.5, read from the reference system); the drag ghost
    /// floats at ~0.7 opacity.
    pub const DRAG_THRESHOLD: f32 = 4.0;
    pub const SPRING_LOAD_MS: u64 = 500;
    pub const DRAG_GHOST_ALPHA: f32 = 0.7;

    /// THE MOUSE WHEEL (round 161, 05.11.2026, the duo at the TV): one notch
    /// came as 15 points — a long list needed «quite a lot of scrolling». A
    /// wheel's steps are multiplied (a touchpad's smooth scroll is not).
    /// Setting `wheel-speed`: one of `WHEEL_SPEEDS`.
    pub const WHEEL_SPEEDS: [(f32, &str); 3] = [(2.0, "Normal"), (3.5, "Schnell"), (6.0, "Sehr schnell")];
    pub const WHEEL_DEFAULT: usize = 1;

    /// What a LONG PRESS means (round 165) — one rule for every house app.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LongPress {
        /// The app answered it (a menu, the edit mode …).
        App,
        /// Nothing there wanted it: the window wiggles and the compositor
        /// drags it (round 164).
        MoveWindow,
        /// Nothing happens (no window to move, or switched off).
        Nothing,
    }

    /// `on_target`: the press lies on something with a role; `answered`: the
    /// app's hold did something; `window`: a movable toplevel (pure — tested).
    pub fn long_press(on_target: bool, answered: bool, window: bool, allowed: bool) -> LongPress {
        if answered {
            LongPress::App
        } else if !on_target && window && allowed {
            LongPress::MoveWindow
        } else {
            LongPress::Nothing
        }
    }

    #[cfg(test)]
    mod long_press_tests {
        use super::*;

        #[test]
        fn only_a_free_spot_moves_the_window() {
            assert_eq!(long_press(true, true, true, true), LongPress::App, "a chip's menu wins");
            assert_eq!(long_press(false, true, true, true), LongPress::App, "the app used the free spot");
            assert_eq!(long_press(false, false, true, true), LongPress::MoveWindow);
            assert_eq!(long_press(true, false, true, true), LongPress::Nothing, "a button without a menu: no drag");
            assert_eq!(long_press(false, false, false, true), LongPress::Nothing, "a layer or the lock");
            assert_eq!(long_press(false, false, true, false), LongPress::Nothing, "switched off");
        }
    }

    /// Round 164: a long press on nothing moves the window (wiggle, the
    /// compositor drags). Setting `window-move` («off» = never).
    pub fn window_move() -> bool {
        crate::settings::read("window-move").map_or(true, |v| v.trim() != "off")
    }

    pub fn set_window_move(on: bool) {
        crate::settings::write("window-move", if on { "on" } else { "off" });
    }

    /// What the finger does after the long press (round 171).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FlingStep {
        /// Not decided yet (still within the slop).
        Wait,
        /// A quick start: a FLICK — keep reading, the release decides.
        Track,
        /// A slow start: a drag — hand the window to the compositor.
        HandOver,
    }

    /// Below this, the finger only trembles (logical px).
    pub const FLING_SLOP: f64 = 22.0;
    /// A start this fast (px/s) is a flick, slower a drag.
    pub const FLING_START: f64 = 450.0;
    /// A release this fast (px/s, last 100 ms) or this far (px) flings.
    pub const FLING_RELEASE: f64 = 300.0;
    pub const FLING_REACH: f64 = 140.0;

    /// THE FINGER AFTER THE LONG PRESS (round 171): a finger cannot carry a
    /// window over the hinge or onto a TV — a quick start becomes a FLICK,
    /// whose release sends the window to the screen in its direction
    /// (mk::canvas::window_fling); a slow start stays the compositor's drag
    /// within the panel (as the mouse always does). Pure: times in seconds.
    #[derive(Debug, Clone, Default)]
    pub struct Fling {
        start: (f64, f64),
        /// (t, x, y) since the press.
        samples: Vec<(f64, f64, f64)>,
        /// When the finger began to move (left 4 px).
        moved_at: Option<f64>,
        step: Option<FlingStep>,
    }

    impl Fling {
        pub fn new(x: f64, y: f64) -> Fling {
            Fling { start: (x, y), ..Fling::default() }
        }

        /// The finger at (x, y), `t` seconds after the long press.
        pub fn push(&mut self, t: f64, x: f64, y: f64) -> FlingStep {
            self.samples.push((t, x, y));
            if self.samples.len() > 64 {
                self.samples.remove(0);
            }
            let away = (x - self.start.0).hypot(y - self.start.1);
            if self.moved_at.is_none() && away > 4.0 {
                self.moved_at = Some(t);
            }
            if let Some(step) = self.step {
                return step;
            }
            if away < FLING_SLOP {
                return FlingStep::Wait;
            }
            let took = (t - self.moved_at.unwrap_or(t)).max(0.008);
            let step = if away / took >= FLING_START { FlingStep::Track } else { FlingStep::HandOver };
            self.step = Some(step);
            step
        }

        /// At the release: the flick's direction (px/s), or None (no fling).
        pub fn release(&self) -> Option<(f32, f32)> {
            if self.step != Some(FlingStep::Track) {
                return None;
            }
            let &(t1, x1, y1) = self.samples.last()?;
            let (t0, x0, y0) = self.samples.iter().copied().find(|s| t1 - s.0 <= 0.1).unwrap_or((t1, x1, y1));
            let dt = (t1 - t0).max(0.016);
            let v = ((x1 - x0) / dt, (y1 - y0) / dt);
            let whole = (x1 - self.start.0, y1 - self.start.1);
            if v.0.hypot(v.1) >= FLING_RELEASE {
                Some((v.0 as f32, v.1 as f32))
            } else if whole.0.hypot(whole.1) >= FLING_REACH {
                Some((whole.0 as f32 * 4.0, whole.1 as f32 * 4.0))
            } else {
                None
            }
        }
    }

    #[cfg(test)]
    mod fling_tests {
        use super::*;

        #[test]
        fn a_quick_flick_down_flings_down() {
            let mut f = Fling::new(300.0, 200.0);
            assert_eq!(f.push(0.00, 300.0, 202.0), FlingStep::Wait);
            assert_eq!(f.push(0.02, 300.0, 215.0), FlingStep::Wait);
            assert_eq!(f.push(0.04, 301.0, 240.0), FlingStep::Track, "38 px in 30 ms");
            f.push(0.06, 302.0, 290.0);
            f.push(0.08, 302.0, 340.0);
            let (vx, vy) = f.release().expect("a fling");
            assert!(vy > 1000.0 && vx.abs() < 200.0, "{vx} {vy}");
        }

        #[test]
        fn a_slow_drag_is_handed_over() {
            let mut f = Fling::new(100.0, 100.0);
            assert_eq!(f.push(0.1, 105.0, 100.0), FlingStep::Wait);
            assert_eq!(f.push(0.3, 115.0, 100.0), FlingStep::Wait);
            assert_eq!(f.push(0.5, 125.0, 100.0), FlingStep::HandOver, "25 px in 0.4 s");
            assert_eq!(f.release(), None);
        }

        #[test]
        fn a_flick_that_stops_short_does_not_fling() {
            let mut f = Fling::new(0.0, 0.0);
            f.push(0.00, 5.0, 0.0);
            assert_eq!(f.push(0.02, 30.0, 0.0), FlingStep::Track);
            f.push(0.20, 40.0, 0.0);
            f.push(0.40, 41.0, 0.0);
            assert_eq!(f.release(), None, "stopped, and only 41 px");
        }
    }

    pub fn wheel_choice() -> usize {
        crate::settings::read("wheel-speed").and_then(|v| WHEEL_SPEEDS.iter().position(|(_, n)| *n == v.trim())).unwrap_or(WHEEL_DEFAULT)
    }

    pub fn set_wheel_choice(index: usize) {
        if let Some((_, name)) = WHEEL_SPEEDS.get(index) {
            crate::settings::write("wheel-speed", name);
        }
    }

    /// The scroll a compositor `value` becomes (pure — tested): a wheel's
    /// step × the factor, anything smooth (touchpad, finger) as it came.
    pub fn wheel_scaled(value: f64, wheel: bool, factor: f32) -> f64 {
        if wheel {
            value * factor as f64
        } else {
            value
        }
    }

    #[cfg(test)]
    mod wheel_tests {
        use super::*;

        #[test]
        fn a_wheel_notch_goes_further_a_touchpad_stays() {
            let f = WHEEL_SPEEDS[WHEEL_DEFAULT].0;
            assert_eq!(wheel_scaled(15.0, true, f), 52.5);
            assert_eq!(wheel_scaled(-15.0, true, f), -52.5);
            assert_eq!(wheel_scaled(3.2, false, f), 3.2, "a touchpad scrolls 1:1");
        }
    }
}

/// Sprach-Architektur (R100-Vermessung, 20.7.2026, lebendes Referenz-
/// macOS): the UI language there is an ordered CHAIN, not a single
/// choice (measured: ["de-CH", "en-GB", "ja-CH"]) — if a translation
/// is missing, lookup walks down the chain. Also: language ≠
/// region! Formats follow the region (AppleLocale de_CH), the
/// surface follows the language chain — both configurable separately.
/// A language switch there only applies to newly started apps: same here.
/// Touch grammar (R168) — dug from the living UIKit headers
/// (CLT SDK 26.2, iOSSupport, UILongPressGestureRecognizer.h):
/// a long press is 0.5 s with at most 10 pt of drift; once
/// recognized, movement is free. The lift values are kit decisions
/// in the reference's spirit: the element rises while the press
/// charges, so the hand learns a menu is coming.
pub mod touch {
    /// minimumPressDuration (measured: header default 0.5).
    pub const LONG_PRESS_SECS: f32 = 0.5;
    /// allowableMovement (measured: header default 10 pt).
    pub const SLOP: f32 = 10.0;
    /// The press must have settled before the element starts to rise —
    /// a plain tap never lifts (kit decision).
    pub const LIFT_DELAY_SECS: f32 = 0.15;
    /// How far the element rises while charging (kit decision).
    pub const LIFT_SCALE: f32 = 1.03;

    /// A finger that moved less than this stayed a TAP; more, and it
    /// scrolls (R210 round 20: three apps carried their own 12 px).
    pub const TAP_SLOP: f32 = 12.0;

    /// TAP OR SCROLL (R210 round 20, 02.11.2026): one press on a scrolling
    /// list. Keys and rows act on RELEASE, and only when the finger stayed
    /// put — a finger that moves scrolls instead (Tube's playlists played on
    /// touch-down, «I can't scroll»). `what` is what a tap means.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Press<T> {
        pub x: f32,
        pub y: f32,
        /// The list's scroll offset when the finger landed.
        pub roll0: f32,
        /// The farthest the finger got from where it landed.
        pub moved: f32,
        pub what: T,
    }

    impl<T: Copy> Press<T> {
        pub fn new(x: f32, y: f32, roll0: f32, what: T) -> Press<T> {
            Press { x, y, roll0, moved: 0.0, what }
        }

        /// The finger moved: the new scroll offset once it is a scroll
        /// (clamped to 0..=max), None while it is still a tap.
        pub fn travel(&mut self, x: f32, y: f32, max: f32) -> Option<f32> {
            let (dx, dy) = (x - self.x, y - self.y);
            self.moved = self.moved.max((dx * dx + dy * dy).sqrt());
            (self.scrolling() && max > 0.0).then(|| (self.roll0 - dy).clamp(0.0, max))
        }

        pub fn scrolling(&self) -> bool {
            self.moved > TAP_SLOP
        }

        /// The finger lifted: what the tap meant, or None after a scroll.
        pub fn release(self) -> Option<T> {
            (!self.scrolling()).then_some(self.what)
        }

        /// The finger lifted over `now` (the hit test at release). A WHOLE
        /// target (a chip, a key — one deed for its whole surface) takes the
        /// press's deed whenever the finger stayed still: the press itself
        /// repaints the target, and a hit list rebuilt in between can miss
        /// (the bar's name chip never opened its card, 02.11.2026). A PART
        /// target (rows of a menu) needs the release on the same deed.
        pub fn settle(self, now: Option<T>, whole: bool) -> Option<T>
        where
            T: PartialEq,
        {
            let what = self.release()?;
            (whole || now == Some(what)).then_some(what)
        }
    }

    #[cfg(test)]
    mod press_tests {
        use super::*;

        #[test]
        fn a_whole_target_survives_a_stale_hit_list() {
            let tap = Press::new(10.0, 10.0, 0.0, 3usize);
            assert_eq!(tap.settle(None, true), Some(3));
            assert_eq!(tap.settle(None, false), None);
            assert_eq!(tap.settle(Some(4), false), None);
            assert_eq!(tap.settle(Some(3), false), Some(3));
            let mut swipe = tap;
            swipe.travel(10.0, 60.0, 100.0);
            assert_eq!(swipe.settle(Some(3), true), None);
        }

        #[test]
        fn a_still_finger_taps_a_moving_one_scrolls() {
            let mut tap = Press::new(100.0, 200.0, 40.0, 7usize);
            assert_eq!(tap.travel(104.0, 205.0, 500.0), None);
            assert_eq!(tap.release(), Some(7));
            let mut drag = Press::new(100.0, 200.0, 40.0, 7usize);
            assert_eq!(drag.travel(100.0, 170.0, 500.0), Some(70.0)); // finger up 30 → roll down
            assert_eq!(drag.travel(100.0, 230.0, 500.0), Some(10.0));
            assert_eq!(drag.travel(100.0, 400.0, 500.0), Some(0.0), "clamped at the top");
            // Back where it started: still a scroll, no tap on release.
            assert_eq!(drag.travel(100.0, 200.0, 500.0), Some(40.0));
            assert_eq!(drag.release(), None);
            // Nothing to scroll: no offset, but no tap either.
            let mut flat = Press::new(0.0, 0.0, 0.0, ());
            assert_eq!(flat.travel(0.0, 50.0, 0.0), None);
            assert_eq!(flat.release(), None);
        }
    }

    /// FOLLOW (round 140, 05.11.2026): a sheet that HANGS ON THE FINGER — the
    /// lock's glass face, a drawer, a card pulled up. Learnt in the lock's
    /// fight with the glass and the swipe:
    ///
    /// - 1:1: the value moves exactly as far as the finger (`way` = the
    ///   pixels the sheet travels from 0 to 1 — the SAME number the drawing
    ///   uses, never a second guess).
    /// - The way comes from the surface the finger is ON. A surface that was
    ///   never drawn has height 0 (the folded duo's hidden main surface): a
    ///   1-px way flung the glass to 18× its end at the slightest move.
    ///   `new` refuses a way under `MIN_WAY` — the caller must name a real one.
    /// - Only a real swipe opens: past `OPEN_AT`, or a firm flick
    ///   (`FLICK_SPEED` of the way per second) past `FLICK_MIN`. A twitch
    ///   springs back.
    /// - A little resistance past either end (`RUBBER`).
    ///
    /// ```ignore
    /// self.follow = touch::Follow::new(y, self.lift.value, hf * 0.55);
    /// // move:    self.lift.value = f.travel(y);
    /// // release: match f.release() { Settle::Open { velocity } => … }
    /// ```
    #[derive(Debug, Clone, Copy)]
    pub struct Follow {
        y0: f32,
        v0: f32,
        way: f32,
        last: (f32, std::time::Instant),
        speed: f32,
        moved: f32,
        value: f32,
    }

    /// What a let-go means.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Settle {
        /// The finger hardly moved: a tap on the sheet.
        Tap,
        /// Spring to 1 (open), with the finger's speed (way per second).
        Open { velocity: f32 },
        /// Spring back to 0.
        Back { velocity: f32 },
    }

    impl Follow {
        /// The shortest way that is believed (px): a surface of height 0
        /// (never drawn) gives none.
        pub const MIN_WAY: f32 = 40.0;
        pub const OPEN_AT: f32 = 0.5;
        pub const FLICK_SPEED: f32 = 4.0;
        pub const FLICK_MIN: f32 = 0.3;
        pub const RUBBER: f32 = 0.25;

        /// A finger lands at `y` while the sheet stands at `value`; `way` px
        /// carry it from 0 to 1 (finger UP raises it). None: no real way.
        pub fn new(y: f32, value: f32, way: f32) -> Option<Follow> {
            Self::new_at(y, value, way, std::time::Instant::now())
        }

        pub fn new_at(y: f32, value: f32, way: f32, now: std::time::Instant) -> Option<Follow> {
            if !(way >= Self::MIN_WAY) {
                eprintln!("[touch] Follow: Weg {way} px verworfen (Fläche nie gezeichnet?)");
                return None;
            }
            Some(Follow { y0: y, v0: value, way, last: (y, now), speed: 0.0, moved: 0.0, value })
        }

        /// The finger is at `y`: the sheet's value (rubber past 0 and 1).
        pub fn travel(&mut self, y: f32) -> f32 {
            self.travel_at(y, std::time::Instant::now())
        }

        pub fn travel_at(&mut self, y: f32, now: std::time::Instant) -> f32 {
            let dt = now.duration_since(self.last.1).as_secs_f32().max(0.004);
            self.speed = 0.6 * self.speed + 0.4 * ((self.last.0 - y) / self.way / dt);
            self.last = (y, now);
            self.moved = self.moved.max((self.y0 - y).abs());
            let v = self.v0 + (self.y0 - y) / self.way;
            self.value = if v < 0.0 {
                v * Self::RUBBER
            } else if v > 1.0 {
                1.0 + (v - 1.0) * Self::RUBBER
            } else {
                v
            };
            self.value
        }

        /// The value now (without a new point).
        pub fn value(&self) -> f32 {
            self.value
        }

        /// The finger's speed (way per second, up = positive).
        pub fn speed(&self) -> f32 {
            self.speed
        }

        /// The finger lifted.
        pub fn release(self) -> Settle {
            if self.moved < TAP_SLOP {
                return Settle::Tap;
            }
            let velocity = self.speed.clamp(-6.0, 6.0);
            let v = self.value;
            if v > Self::OPEN_AT || (self.speed > Self::FLICK_SPEED && v > Self::FLICK_MIN) {
                Settle::Open { velocity }
            } else {
                Settle::Back { velocity }
            }
        }
    }

    #[cfg(test)]
    mod follow_tests {
        use super::*;
        use std::time::{Duration, Instant};

        #[test]
        fn a_surface_never_drawn_gives_no_way() {
            // The lock's bug of 05.11.2026: size 0 → way max(1 px).
            assert!(Follow::new(500.0, 0.0, 0.0).is_none());
            assert!(Follow::new(500.0, 0.0, 1.0).is_none());
            assert!(Follow::new(500.0, 0.0, f32::NAN).is_none());
            assert!(Follow::new(500.0, 0.0, 359.0).is_some());
        }

        #[test]
        fn the_sheet_moves_one_to_one() {
            let t = Instant::now();
            let mut f = Follow::new_at(600.0, 0.0, 300.0, t).unwrap();
            assert_eq!(f.travel_at(450.0, t + Duration::from_millis(200)), 0.5);
            assert_eq!(f.travel_at(300.0, t + Duration::from_millis(400)), 1.0);
            // Past the end: resistance, never 18×.
            let past = f.travel_at(0.0, t + Duration::from_millis(600));
            assert!((past - 1.25).abs() < 1e-4, "{past}");
            // Below the start: resistance too.
            let below = f.travel_at(900.0, t + Duration::from_millis(800));
            assert!((below + 0.25).abs() < 1e-4, "{below}");
        }

        #[test]
        fn only_a_real_swipe_opens() {
            let t = Instant::now();
            // A twitch: a short slow move springs back.
            let mut f = Follow::new_at(600.0, 0.0, 300.0, t).unwrap();
            f.travel_at(540.0, t + Duration::from_millis(300));
            assert!(matches!(f.release(), Settle::Back { .. }));
            // Past half the way: opens even slowly.
            let mut f = Follow::new_at(600.0, 0.0, 300.0, t).unwrap();
            f.travel_at(420.0, t + Duration::from_millis(900));
            assert!(matches!(f.release(), Settle::Open { .. }));
            // A firm flick past a third: opens.
            let mut f = Follow::new_at(600.0, 0.0, 300.0, t).unwrap();
            f.travel_at(560.0, t + Duration::from_millis(16));
            f.travel_at(490.0, t + Duration::from_millis(32));
            assert!(matches!(f.release(), Settle::Open { .. }), "speed {}", f.speed());
            // A still finger: a tap.
            let mut f = Follow::new_at(600.0, 0.0, 300.0, t).unwrap();
            f.travel_at(596.0, t + Duration::from_millis(50));
            assert_eq!(f.release(), Settle::Tap);
        }
    }

    /// TOUCH LOG (round 140): every finger point of an app with its time —
    /// how the lock's swipe was finally SEEN instead of guessed (the log
    /// showed size 0 and a value of 18). Off unless `MATRIX_TOUCH_LOG` is set
    /// or `~/.config/matrix/touch-log` exists; writes
    /// `$XDG_RUNTIME_DIR/matrix-touch-<app>.log` (tmpfs), restarted at 1 MB.
    pub fn log(app: &str, line: &str) {
        use std::io::Write;
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let on = *ON.get_or_init(|| {
            std::env::var_os("MATRIX_TOUCH_LOG").is_some()
                || std::env::var_os("HOME").is_some_and(|h| std::path::Path::new(&h).join(".config/matrix/touch-log").exists())
        });
        if !on {
            return;
        }
        let t = START.get_or_init(std::time::Instant::now).elapsed().as_millis();
        let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
        let path = format!("{dir}/matrix-touch-{app}.log");
        let big = std::fs::metadata(&path).map(|m| m.len() > 1 << 20).unwrap_or(false);
        let file = if big {
            std::fs::File::create(&path)
        } else {
            std::fs::OpenOptions::new().create(true).append(true).open(&path)
        };
        if let Ok(mut f) = file {
            let _ = writeln!(f, "{t:>8} {line}");
        }
    }
}

/// The installed-apps index (R169): ONE source for every launcher —
/// the home screen first, matrix-start to follow. Reads the .desktop
/// world plus ~/Programme (R48: an AppImage in the folder IS the
/// installation, like the reference's /Applications).
pub mod apps {
    use std::path::{Path, PathBuf};

    /// THE application dirs (R215 family, 02.11.2026), lowest priority
    /// first — a later dir's entry wins. One list for the index, the bar's
    /// title, the start view and the icon painter: before, three copies
    /// disagreed and /usr/local/share (the duo's house apps) was missing
    /// from the index (dock labels) and the bar («Matrix Camera»).
    pub fn data_dirs() -> Vec<PathBuf> {
        let home = std::env::var("HOME").unwrap_or_default();
        vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
            PathBuf::from(format!("{home}/.local/share/flatpak/exports/share/applications")),
            PathBuf::from(format!("{home}/.local/share/applications")),
        ]
    }

    /// The desktop file that counts for `app_id` (the highest-priority dir).
    pub fn desktop_path(app_id: &str) -> Option<PathBuf> {
        desktop_path_in(&data_dirs(), app_id)
    }

    pub fn desktop_path_in(dirs: &[PathBuf], app_id: &str) -> Option<PathBuf> {
        dirs.iter().rev().map(|d| d.join(format!("{app_id}.desktop"))).find(|p| p.is_file())
    }

    /// The name an app shows (Name[de] before Name), from its desktop file.
    pub fn display_name(app_id: &str) -> Option<String> {
        desktop_path(app_id).and_then(|p| name_in(&p))
    }

    fn name_in(path: &Path) -> Option<String> {
        let text = std::fs::read_to_string(path).ok()?;
        let mut name = None;
        let mut local: [Option<String>; 2] = [None, None];
        let mut in_entry = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_entry = line == "[Desktop Entry]";
                continue;
            }
            if !in_entry {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else { continue };
            if let Some(slot) = crate::language::desktop_name_key(k.trim()) {
                local[slot].get_or_insert_with(|| v.trim().to_string());
            } else if k.trim() == "Name" {
                name = Some(v.trim().to_string());
            }
        }
        let [de, en] = local;
        crate::language::desktop_name(name, de, en)
    }

    #[cfg(test)]
    mod dir_tests {
        use super::*;

        #[test]
        fn later_dir_wins_and_german_name_first() {
            let base = std::env::temp_dir().join(format!("mk-apps-{}", std::process::id()));
            let (low, high) = (base.join("low"), base.join("high"));
            std::fs::create_dir_all(&low).unwrap();
            std::fs::create_dir_all(&high).unwrap();
            std::fs::write(low.join("x.desktop"), "[Desktop Entry]\nName=Low\n").unwrap();
            std::fs::write(high.join("x.desktop"), "[Desktop Entry]\nName=Camera\nName[de]=Kamera\n[Desktop Action a]\nName=Nope\n").unwrap();
            let dirs = vec![low.clone(), high.clone()];
            let p = desktop_path_in(&dirs, "x").unwrap();
            assert_eq!(p, high.join("x.desktop"));
            assert_eq!(name_in(&p).as_deref(), Some("Kamera"));
            assert_eq!(name_in(&low.join("x.desktop")).as_deref(), Some("Low"));
            assert!(desktop_path_in(&dirs, "y").is_none());
            let _ = std::fs::remove_dir_all(&base);
        }
    }

    /// A launchable app.
    #[derive(Debug, Clone)]
    pub struct Entry {
        pub app_id: String,
        pub name: String,
        pub exec: String,
        pub terminal: bool,
        /// Icon= field — theme name or absolute path.
        pub icon: Option<String>,
    }

    fn desktop_read(path: &std::path::Path, stem: &str) -> Option<Entry> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut in_entry = false;
        let (mut name, mut exec, mut kind) = (None, None, None);
        let mut local: [Option<String>; 2] = [None, None];
        let mut icon = None;
        let (mut hidden, mut terminal) = (false, false);
        for line in content.lines() {
            let z = line.trim();
            if z.starts_with('[') {
                in_entry = z == "[Desktop Entry]";
                continue;
            }
            if !in_entry {
                continue;
            }
            let Some((k, v)) = z.split_once('=') else { continue };
            match k.trim() {
                "Name" => name = Some(v.trim().to_string()),
                key if crate::language::desktop_name_key(key).is_some() => {
                    let slot = crate::language::desktop_name_key(key).unwrap_or(0);
                    local[slot].get_or_insert_with(|| v.trim().to_string());
                }
                "Exec" => exec = Some(v.trim().to_string()),
                "Icon" => icon = Some(v.trim().to_string()),
                "Type" => kind = Some(v.trim().to_string()),
                "NoDisplay" | "Hidden" if v.trim() == "true" => hidden = true,
                "Terminal" if v.trim() == "true" => terminal = true,
                _ => {}
            }
        }
        if hidden || kind.as_deref() != Some("Application") {
            return None;
        }
        Some(Entry {
            app_id: stem.to_string(),
            name: { let [de, en] = local; crate::language::desktop_name(name, de, en)? },
            exec: exec?,
            terminal,
            icon,
        })
    }

    /// WHERE AN APP COMES FROM (round 36): «Matrix», «Duo», «GNOME», «KDE» —
    /// lists that mix families say it (the app card showed GNOME's «Dateien»
    /// beside Matrix Dateien without a word, 02.11.2026). None: unknown.
    pub fn origin(app_id: &str) -> Option<&'static str> {
        let id = app_id.to_ascii_lowercase();
        if id.starts_with("matrix-") || id.starts_with("ch.matrixhardline.duo") {
            Some(if id.starts_with("ch.matrixhardline.duo") { "Duo" } else { "Matrix" })
        } else if id.starts_with("ch.matrixhardline.") {
            Some("Matrix")
        } else if id.starts_with("org.gnome.") {
            Some("GNOME")
        } else if id.starts_with("org.kde.") {
            Some("KDE")
        } else {
            None
        }
    }

    #[cfg(test)]
    mod origin_tests {
        #[test]
        fn families_are_named() {
            assert_eq!(super::origin("matrix-files"), Some("Matrix"));
            assert_eq!(super::origin("ch.matrixhardline.DuoLLM"), Some("Duo"));
            assert_eq!(super::origin("ch.matrixhardline.Vorlesen"), Some("Matrix"));
            assert_eq!(super::origin("org.gnome.Nautilus"), Some("GNOME"));
            assert_eq!(super::origin("firefox"), None);
        }
    }

    /// Every installed app, sorted by name.
    pub fn index() -> Vec<Entry> {
        let home = std::env::var("HOME").unwrap_or_default();
        let sources = data_dirs();
        let mut by_id: std::collections::HashMap<String, Entry> =
            std::collections::HashMap::new();
        for source in &sources {
            let Ok(dir) = std::fs::read_dir(source) else { continue };
            for e in dir.flatten() {
                let path = e.path();
                if path.extension().and_then(|x| x.to_str()) != Some("desktop") {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if let Some(entry) = desktop_read(&path, stem) {
                    by_id.insert(stem.to_string(), entry);
                } else {
                    by_id.remove(stem);
                }
            }
        }
        // ~/Programme: the folder is the register.
        let programs = format!("{home}/Programme");
        if let Ok(dir) = std::fs::read_dir(&programs) {
            for e in dir.flatten() {
                let path = e.path();
                let is_appimage = path
                    .extension()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| x.eq_ignore_ascii_case("appimage"));
                if !is_appimage {
                    continue;
                }
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                by_id.entry(stem.to_string()).or_insert(Entry {
                    app_id: stem.to_string(),
                    name: stem.to_string(),
                    exec: path.to_string_lossy().into_owned(),
                    terminal: false,
                    icon: None,
                });
            }
        }
        let mut all: Vec<Entry> = by_id.into_values().collect();
        all.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        all
    }

    fn exec_cleanup(exec: &str) -> String {
        exec.split_whitespace()
            .filter(|w| !w.starts_with('%'))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Launch an entry the way the reference launches from its home
    /// screen — fire and forget.
    pub fn launch(entry: &Entry) {
        let command = exec_cleanup(&entry.exec);
        // Round 51: reaped — the launcher (bar, home) runs all day.
        if entry.terminal {
            let _ = crate::process::spawn_reaped(std::process::Command::new("foot").args(["-e", "sh", "-c", &command]));
        } else {
            let _ = crate::process::spawn_reaped(std::process::Command::new("sh").args(["-c", &command]));
        }
    }
}

pub mod language {
    use std::collections::HashMap;
    use std::sync::OnceLock;

    /// The two languages Matrix speaks (Morpheus, 06.11.2026): German
    /// (Switzerland) and English (UK) — the leading entry of a chain.
    pub const GERMAN: &str = "de-CH";
    pub const ENGLISH: &str = "en-GB";
    /// The DEVICE's language and keyboard (written by Morpheus and the
    /// first-start setup); an account's own setting wins over them.
    pub const SYSTEM_LANGUAGE: &str = "/etc/matrix/languages";
    pub const SYSTEM_KEYBOARD: &str = "/etc/matrix/keyboard";
    /// The English catalogue: one line per text, «German<TAB>English».
    pub const CATALOGUE: &str = "/usr/share/matrix/sprache/en.tsv";

    fn system(path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    }

    /// The chain as a comma list → its entries.
    pub fn parse_chain(s: &str) -> Vec<String> {
        s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
    }

    /// Ordered language chain (AppleLanguages principle); the first one leads.
    /// Stored as a comma list under "languages" (e.g. "de-CH,en"); none for
    /// the account: the device's (/etc/matrix/languages); none: German.
    pub fn chain() -> &'static Vec<String> {
        static K: OnceLock<Vec<String>> = OnceLock::new();
        K.get_or_init(|| {
            // MATRIX_LANGUAGE («en-GB») first: probes and one-off starts.
            std::env::var("MATRIX_LANGUAGE")
                .ok()
                .or_else(|| super::settings::read("languages"))
                .or_else(|| system(SYSTEM_LANGUAGE))
                .map(|s| parse_chain(&s))
                .filter(|v: &Vec<String>| !v.is_empty())
                .unwrap_or_else(|| vec![String::from(GERMAN), String::from("en")])
        })
    }

    /// A choice for THIS process (the first-start setup switches its own
    /// pages at once, before anything is written): Some(true) = English.
    static CHOSEN: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

    static CHOSEN_LAYOUT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

    /// The keyboard for THIS process («ch» / «gb»), like `choose`.
    pub fn choose_layout(code: &str) {
        CHOSEN_LAYOUT.store(if code == "gb" { 2 } else { 1 }, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn choose(code: &str) {
        CHOSEN.store(if code.starts_with("en") { 2 } else { 1 }, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_english() -> bool {
        match CHOSEN.load(std::sync::atomic::Ordering::Relaxed) {
            1 => false,
            2 => true,
            _ => chain().first().is_some_and(|s| s.starts_with("en")),
        }
    }

    /// The keyboard layout: the account's («keyboard-layout»), else the
    /// device's, else Swiss. «ch» or «gb» (xkb names).
    pub fn layout() -> &'static str {
        // A layout chosen in this process (the setup's keyboard page) wins.
        match CHOSEN_LAYOUT.load(std::sync::atomic::Ordering::Relaxed) {
            1 => return "ch",
            2 => return "gb",
            _ => {}
        }
        static L: OnceLock<String> = OnceLock::new();
        L.get_or_init(|| super::settings::read("keyboard-layout").or_else(|| system(SYSTEM_KEYBOARD)).unwrap_or_else(|| String::from("ch")))
    }

    /// The catalogue: «German<TAB>English» lines (# comments). Pure.
    pub fn parse_catalogue(text: &str) -> HashMap<String, String> {
        text.lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .filter_map(|l| l.split_once('\t'))
            .filter(|(de, en)| !de.is_empty() && !en.is_empty())
            .map(|(de, en)| (de.replace("\\n", "\n"), en.replace("\\n", "\n")))
            .collect()
    }

    fn catalogue() -> &'static HashMap<String, String> {
        static C: OnceLock<HashMap<String, String>> = OnceLock::new();
        C.get_or_init(|| std::fs::read_to_string(CATALOGUE).map(|t| parse_catalogue(&t)).unwrap_or_default())
    }

    fn built_in(de: &str) -> Option<&'static str> {
        Some(match de {
            "Schließen" => "Close",
            "Maximieren" => "Maximize",
            "In die Ablage (Super+M)" => "Minimize (Super+M)",
            "Abbrechen" => "Cancel",
            "Fertig" => "Done",
            "Suchen" => "Search",
            "Hilfe" => "Help",
            "Einstellungen" => "Settings",
            "Zurück" => "Back",
            "Weiter" => "Continue",
            "Öffnen" => "Open",
            "Umbenennen" => "Rename",
            "Papierkorb" => "Trash",
            "Leinwand" => "Canvas",
            _ => return None,
        })
    }

    /// Translate (gettext principle: the German text IS the key — no key
    /// zoo, nothing forgotten): built in, then the catalogue, else German.
    pub fn t(de: &'static str) -> &'static str {
        if !is_english() {
            return de;
        }
        built_in(de).or_else(|| catalogue().get(de).map(|s| s.as_str())).unwrap_or(de)
    }

    /// The same for a text made at run time (formats, names): owned.
    pub fn tr(de: &str) -> String {
        if !is_english() {
            return de.to_string();
        }
        built_in(de).map(String::from).or_else(|| catalogue().get(de).cloned()).unwrap_or_else(|| de.to_string())
    }

    /// A catalogue PATTERN («{} Treffer» → «{} results»): the German
    /// pattern's literal pieces must stand in `text` in order; what lies
    /// between them fills the English «{}» in order. Pure.
    pub fn fill(pattern: &str, english: &str, text: &str) -> Option<String> {
        fill_with(pattern, english, text, &|v| v.to_string())
    }

    /// Like `fill`, each caught value passed through `value` first.
    fn fill_with(pattern: &str, english: &str, text: &str, value: &dyn Fn(&str) -> String) -> Option<String> {
        let pieces: Vec<&str> = pattern.split("{}").collect();
        if pieces.len() < 2 {
            return None;
        }
        let mut rest = text.strip_prefix(pieces[0])?;
        let mut caught = Vec::new();
        for (k, piece) in pieces.iter().enumerate().skip(1) {
            let last = k == pieces.len() - 1;
            let at = if last {
                if !rest.ends_with(piece) || rest.len() < piece.len() {
                    return None;
                }
                rest.len() - piece.len()
            } else if piece.is_empty() {
                return None;
            } else {
                rest.find(piece)?
            };
            let value = &rest[..at];
            if value.is_empty() || value.contains('\n') {
                return None;
            }
            caught.push(value);
            rest = &rest[at + piece.len()..];
        }
        let mut out = String::new();
        let mut values = caught.into_iter();
        let mut parts = english.split("{}").peekable();
        while let Some(part) = parts.next() {
            out.push_str(part);
            if parts.peek().is_some() {
                out.push_str(&values.next().map(value).unwrap_or_default());
            }
        }
        Some(out)
    }

    fn patterns() -> &'static Vec<(String, String)> {
        static P: OnceLock<Vec<(String, String)>> = OnceLock::new();
        P.get_or_init(|| {
            let mut p: Vec<(String, String)> = catalogue().iter().filter(|(de, _)| de.contains("{}")).map(|(de, en)| (de.clone(), en.clone())).collect();
            // The most specific pattern first (most literal text), so the
            // choice never hangs on a hash order.
            p.sort_by(|a, b| b.0.replace("{}", "").len().cmp(&a.0.replace("{}", "").len()).then(a.0.cmp(&b.0)));
            p
        })
    }

    /// WHAT THE SCREEN SHOWS (06.11.2026): every text the house measures,
    /// wraps and paints passes here — in English the catalogue's text (a
    /// whole text, or a pattern with values), else the text as it is. So
    /// every app speaks English without touching each of its strings.
    pub fn shown(text: &str) -> std::borrow::Cow<'_, str> {
        use std::borrow::Cow;
        if !is_english() || text.len() < 2 || !text.bytes().any(|b| b.is_ascii_alphabetic()) {
            return Cow::Borrowed(text);
        }
        if let Some(en) = built_in(text) {
            return Cow::Owned(en.to_string());
        }
        if let Some(en) = catalogue().get(text) {
            return Cow::Owned(en.clone());
        }
        // Values inside a pattern are texts too («{} Objekte · {}»).
        let inner = |v: &str| shown(v).into_owned();
        for (de, en) in patterns() {
            if let Some(out) = fill_with(de, en, text, &inner) {
                return Cow::Owned(out);
            }
        }
        // A status line of parts («Archiv · ZIP · 8 MB · vor 5 Tagen»):
        // part by part.
        if text.contains(" · ") {
            let parts: Vec<_> = text.split(" · ").map(shown).collect();
            if parts.iter().any(|p| matches!(p, Cow::Owned(_))) {
                return Cow::Owned(parts.iter().map(|p| p.as_ref()).collect::<Vec<_>>().join(" · "));
            }
        }
        // Section heads are set in capitals («GERÄT»): the key in capitals.
        if text.chars().any(|c| c.is_uppercase()) && !text.chars().any(|c| c.is_lowercase()) {
            if let Some(en) = capitals().get(text) {
                return Cow::Owned(en.clone());
            }
        }
        missed(text);
        Cow::Borrowed(text)
    }

    fn capitals() -> &'static HashMap<String, String> {
        static C: OnceLock<HashMap<String, String>> = OnceLock::new();
        C.get_or_init(|| catalogue().iter().map(|(de, en)| (de.to_uppercase(), en.to_uppercase())).collect())
    }

    /// The MISSING BOOK: with MATRIX_LANGUAGE_MISSES=<file> every shown
    /// text without a translation is written there once — the list of what
    /// the catalogue still lacks, gathered from real screens.
    fn missed(text: &str) {
        static BOOK: OnceLock<Option<std::sync::Mutex<(std::collections::HashSet<String>, std::fs::File)>>> = OnceLock::new();
        let book = BOOK.get_or_init(|| {
            let path = std::env::var_os("MATRIX_LANGUAGE_MISSES")?;
            let file = std::fs::OpenOptions::new().create(true).append(true).open(path).ok()?;
            Some(std::sync::Mutex::new((std::collections::HashSet::new(), file)))
        });
        let Some(book) = book else { return };
        if text.chars().filter(|c| c.is_alphabetic()).count() < 2 {
            return;
        }
        // Already English (a translation measured again, or a piece of
        // one): not missing.
        if catalogue().values().any(|en| en.contains(text.trim())) {
            return;
        }
        if let Ok(mut b) = book.lock() {
            if b.0.insert(text.to_string()) {
                use std::io::Write;
                let _ = writeln!(b.1, "{}", text.replace('\n', "\\n"));
            }
        }
    }

    /// AN APP'S NAME by language (desktop files): English takes
    /// Name[en_GB], then Name[en], then Name (the base name of foreign
    /// apps is English); German takes Name[de], then Name. Our own apps
    /// carry a German Name — the catalogue turns it at the screen.
    pub fn desktop_name(name: Option<String>, de: Option<String>, en: Option<String>) -> Option<String> {
        let pick = if is_english() { en.or(name) } else { de.or(name) };
        pick.filter(|n| !n.is_empty())
    }

    /// The localised keys a desktop reader keeps: Some(slot) for
    /// «Name[de]» (0) and «Name[en_GB]»/«Name[en]» (1).
    pub fn desktop_name_key(key: &str) -> Option<usize> {
        match key {
            "Name[de]" | "Name[de_CH]" => Some(0),
            "Name[en_GB]" | "Name[en]" => Some(1),
            _ => None,
        }
    }

    /// One of two, by language — for sentences with values in them.
    pub fn pick<'a>(de: &'a str, en: &'a str) -> &'a str {
        if is_english() {
            en
        } else {
            de
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_catalogue_reads_german_to_english() {
            let c = parse_catalogue("# Kommentar\nAnmelden\tLog in\nZwei\\nZeilen\tTwo\\nlines\nkaputt\n\tleer\n");
            assert_eq!(c.get("Anmelden").map(String::as_str), Some("Log in"));
            assert_eq!(c.get("Zwei\nZeilen").map(String::as_str), Some("Two\nlines"));
            assert_eq!(c.len(), 2);
            assert_eq!(parse_chain(" en-GB , de-CH"), vec!["en-GB", "de-CH"]);
        }

        #[test]
        fn patterns_carry_their_values() {
            assert_eq!(fill("{} Treffer", "{} results", "20 Treffer").as_deref(), Some("20 results"));
            assert_eq!(fill("Akku {} %", "Battery {} %", "Akku 64 %").as_deref(), Some("Battery 64 %"));
            assert_eq!(fill("{} von {}", "{} of {}", "3 von 12").as_deref(), Some("3 of 12"));
            assert_eq!(fill("Mit «{}» verbunden ✓", "Connected to «{}» ✓", "Mit «lzi» verbunden ✓").as_deref(), Some("Connected to «lzi» ✓"));
            assert_eq!(fill("{} Treffer", "{} results", "Treffer"), None, "a value is needed");
            assert_eq!(fill_with("{} Objekte · {}", "{} items · {}", "2 Objekte · 5 GB", &|v| v.replace("GB", "gb")).as_deref(), Some("2 items · 5 gb"));
            assert_eq!(fill("Akku {} %", "Battery {} %", "Lautstärke 5 %"), None);
        }
    }
}

/// Regions-Grammatik (R100, live vermessen — drei Locales befragt):
/// de_CH: 1'234'567.89 (Apostroph-Gruppe, PUNKT-Dezimal!), 21.07.2026,
/// 24 h · de_DE: 1.234.567,89 · en_US: 1,234,567.89, Jul 21, 4:13 AM.
pub mod region {
    use std::sync::OnceLock;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Region {
        Switzerland,
        Germany,
        International,
    }
    pub fn current() -> Region {
        static R: OnceLock<Region> = OnceLock::new();
        *R.get_or_init(|| match super::settings::read("region").as_deref() {
            Some("de") => Region::Germany,
            Some("int") | Some("us") => Region::International,
            _ => Region::Switzerland,
        })
    }

    /// Decimal separator of the region (CH and international: point).
    pub use self::decimal_sep as decimal;
    pub fn decimal_sep() -> char {
        match current() {
            Region::Germany => ',',
            _ => '.',
        }
    }

    /// Short date per regional grammar (measured: CH/DE day first
    /// with dots, international month first with slashes).
    pub use self::short_date as date_short;
    pub fn short_date(day: u32, month: u32, year: u32) -> String {
        match current() {
            Region::International => format!("{month}/{day}/{year}"),
            _ => format!("{day:02}.{month:02}.{year}"),
        }
    }
}

/// Speicher-Doktrin (R99-Vermessung, 20.7.2026, lebendes Referenz-macOS):
/// "Free RAM is wasted RAM." During measurement the reference ran
/// 61 MB (!) of free pages out of 8 GB — silky: 2.1 GB compressed,
/// swap 0.25 MB, hardly any swapins. That is why the reference never
/// reports "X % used" to humans but PRESSURE (memory pressure). The
/// Linux counterpart is PSI: /proc/pressure/memory, avg10 = percent of
/// time processes recently had to wait for memory. For disks the
/// "almost full" early warning applies — sharpened here by the 3 % wall
/// of rpm-ostree (updates fail BELOW 3 % free; experienced repeatedly).
pub mod memory {
    /// PSI-avg10-Schwellen (Anteil wartender Zeit in %):
    /// below "low", in between "elevated", above "high".
    pub const PRESSURE_ELEVATED: f32 = 5.0;
    pub const PRESSURE_UP: f32 = 25.0;
    /// Disk early warning: below 10 % free = warning,
    /// below 5 % = critical (the 3 % update wall approaches).
    pub const DISK_WARNING_RATIO: f64 = 0.10;
    pub const DISK_CRITICAL_RATIO: f64 = 0.05;

    /// Read memory pressure: avg10 of the "some" line from PSI.
    /// None = kernel without PSI (the display stays silent then).
    pub use self::read_pressure as pressure_read;
    pub fn read_pressure() -> Option<f32> {
        let text = std::fs::read_to_string("/proc/pressure/memory").ok()?;
        let line = text.lines().find(|z| z.starts_with("some"))?;
        let avg10 = line.split_whitespace().find(|f| f.starts_with("avg10="))?;
        avg10.trim_start_matches("avg10=").parse().ok()
    }

    /// The three pressure levels of the reference (green/yellow/red).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Pressure {
        Low,
        Elevated,
        High,
    }
    pub fn classify(avg10: f32) -> Pressure {
        if avg10 >= PRESSURE_UP {
            Pressure::High
        } else if avg10 >= PRESSURE_ELEVATED {
            Pressure::Elevated
        } else {
            Pressure::Low
        }
    }

    /// The purge contract (R157, dug out of XNU's memorystatus):
    /// kern.memorystatus_purge_on_warning = 2, _urgent = 5,
    /// _critical = 8 — under pressure the reference does NOT swap and
    /// does NOT kill first: it asks holders of volatile memory to give
    /// it back, tier by tier. Every app that keeps a cache answers here.
    ///
    /// Tier = how expendable a cache is: 1 = trivially rebuilt
    /// (thumbnails on disk), 5 = costly (a decoded image), 9 = the last
    /// thing you drop (a live index). An app drops everything whose
    /// tier is BELOW the level the pressure demands.
    pub fn purge_below(p: Pressure) -> u8 {
        match p {
            // Apple's own three numbers, verbatim.
            Pressure::Low => 0,
            Pressure::Elevated => 2,
            Pressure::High => 8,
        }
    }

    /// Round 98: memory still AVAILABLE (MemAvailable), in MB, and the total.
    pub fn available_mb() -> Option<(u64, u64)> {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        let field = |name: &str| -> Option<u64> {
            text.lines().find(|l| l.starts_with(name))?.split_whitespace().nth(1)?.parse::<u64>().ok().map(|kb| kb / 1024)
        };
        Some((field("MemAvailable:")?, field("MemTotal:")?))
    }

    /// Round 98: the pressure level — from PSI where the kernel has it, else
    /// from what is still available (the Duo's 4.14 kernel has no memory
    /// PSI: the purge contract never fired there; on 04.11.2026 six GPU apps
    /// filled the 5.5 GB and froze the Duo). Under 15 % elevated, under 8 % high.
    pub fn pressure() -> Option<Pressure> {
        if let Some(p) = read_pressure() {
            return Some(classify(p));
        }
        let (avail, total) = available_mb()?;
        Some(level_of(avail, total))
    }

    /// The level for `avail` of `total` MB (see `pressure`).
    pub fn level_of(avail: u64, total: u64) -> Pressure {
        let ratio = avail as f64 / total.max(1) as f64;
        if ratio < 0.08 {
            Pressure::High
        } else if ratio < 0.15 {
            Pressure::Elevated
        } else {
            Pressure::Low
        }
    }

    /// Round 98: what a Matrix process holds — RAM (RSS) and GPU memory (on
    /// Qualcomm the KGSL count; that is RAM too and shows in no process list).
    #[derive(Debug, Clone, PartialEq)]
    pub struct AppUse {
        pub name: String,
        pub pid: u32,
        pub ram_mb: u64,
        pub gpu_mb: u64,
    }

    impl AppUse {
        pub fn total_mb(&self) -> u64 {
            self.ram_mb + self.gpu_mb
        }
    }

    /// The GPU memory of `pid` in MB (KGSL), 0 where there is none.
    pub fn gpu_mb(pid: u32) -> u64 {
        std::fs::read_to_string(format!("/sys/class/kgsl/kgsl/proc/{pid}/gpumem_mapped"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map_or(0, |b| b / (1024 * 1024))
    }

    /// Every running Matrix part (matrix-*, the compositor), biggest first.
    pub fn app_usage() -> Vec<AppUse> {
        let mut out: Vec<AppUse> = std::fs::read_dir("/proc")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let pid: u32 = e.file_name().to_string_lossy().parse().ok()?;
                let name = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?.trim().to_string();
                if !(name.starts_with("matrix-") || name.starts_with("niri")) {
                    return None;
                }
                let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
                let rss_kb: u64 = status.lines().find(|l| l.starts_with("VmRSS:"))?.split_whitespace().nth(1)?.parse().ok()?;
                Some(AppUse { name, pid, ram_mb: rss_kb / 1024, gpu_mb: gpu_mb(pid) })
            })
            .collect();
        out.sort_by(|a, b| b.total_mb().cmp(&a.total_mb()));
        out
    }

    /// The process's name for people: «matrix-wallpape» → «Hintergrund».
    pub fn app_label(comm: &str) -> String {
        let known = [
            ("matrix-wallpape", "Hintergrund"), ("matrix-bar", "Topbar"), ("matrix-dock", "Dock"),
            ("matrix-tube", "Tube"), ("matrix-settings", "Einstellungen"), ("matrix-files", "Dateien"),
            ("matrix-shopping", "Einkauf"), ("matrix-terminal", "Terminal"), ("matrix-player", "Player"),
            ("matrix-camera", "Kamera"), ("matrix-context", "Kontextmenü"), ("matrix-notifica", "Mitteilungen"),
            ("matrix-keyboard", "Tastatur"), ("niri-canvas", "Compositor (niri)"), ("matrix-oracle", "reference app"),
        ];
        known.iter().find(|(c, _)| *c == comm).map(|(_, l)| l.to_string()).unwrap_or_else(|| comm.trim_start_matches("matrix-").to_string())
    }

    #[cfg(test)]
    mod level_tests {
        #[test]
        fn what_is_left_decides_without_psi() {
            use super::{level_of, Pressure};
            assert_eq!(level_of(2600, 5512), Pressure::Low);
            assert_eq!(level_of(700, 5512), Pressure::Elevated);
            assert_eq!(level_of(400, 5512), Pressure::High);
            assert_eq!(super::app_label("matrix-wallpape"), "Hintergrund");
            assert_eq!(super::app_label("matrix-codes"), "codes");
        }
    }

    /// Should a cache of this tier be dropped right now?
    pub fn purge(tier: u8, p: Pressure) -> bool {
        tier < purge_below(p)
    }

    /// Current pressure, for apps that only want the answer.
    pub fn pressure_now() -> Pressure {
        classify(read_pressure().unwrap_or(0.0))
    }
}

/// Power assertions (R158) — dug live out of the reference's
/// assertion table (`pmset -g assertions`, 28.07.2026). The anatomy,
/// verbatim from a running caffeinate:
///
///   pid 902(caffeinate): [0x00016d64...] PreventUserIdleDisplaySleep
///       named: "caffeinate command-line tool"
///       Localized=THE CAFFEINATE TOOL IS PREVENTING SLEEP.
///       Timeout will fire in 8973 secs Action=TimeoutActionRelease
///
/// Four properties, all load-bearing: a machine name AND a sentence
/// for humans; a DEADLINE with an action (Apple gives every assertion
/// one — a forgotten assertion must not keep a machine awake forever);
/// and it dies with its holder (the table showed 0 the moment
/// caffeinate exited).
///
/// Matrix has the mechanisms (systemd-inhibit, idle-inhibit) but had
/// no REGISTRY: nobody could say who keeps the screen awake and why.
/// This is that registry — one file per assertion in the runtime
/// directory, so it dies with the session by construction.
pub mod assertion {
    use std::path::PathBuf;

    /// What an assertion protects.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Kind {
        /// The screen must stay on (a video is playing).
        Display,
        /// The machine must not idle-sleep (a build is running).
        System,
    }

    impl Kind {
        pub fn as_str(self) -> &'static str {
            match self {
                Kind::Display => "display",
                Kind::System => "system",
            }
        }
        fn parse(s: &str) -> Option<Kind> {
            match s {
                "display" => Some(Kind::Display),
                "system" => Some(Kind::System),
                _ => None,
            }
        }
    }

    /// A live assertion held by some process.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Held {
        pub pid: u32,
        pub kind: Kind,
        /// The sentence a human reads ("Video läuft").
        pub reason: String,
        /// Unix seconds after which it is void (Apple's timeout).
        pub deadline: u64,
    }

    fn dir() -> PathBuf {
        let run = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
        PathBuf::from(run).join("matrix/assertions")
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// The line format — one assertion, one file, three fields.
    pub fn encode(kind: Kind, reason: &str, deadline: u64) -> String {
        // Reasons never carry tabs; a stray one would split the record.
        format!("{}\t{}\t{}\n", kind.as_str(), deadline, reason.replace('\t', " "))
    }

    pub fn decode(pid: u32, body: &str) -> Option<Held> {
        let mut it = body.trim_end_matches('\n').splitn(3, '\t');
        let kind = Kind::parse(it.next()?)?;
        let deadline = it.next()?.parse().ok()?;
        Some(Held { pid, kind, reason: it.next().unwrap_or("").to_string(), deadline })
    }

    /// Is this assertion still valid RIGHT NOW? Apple's two death
    /// conditions: the deadline passed, or the holder is gone.
    pub fn valid(h: &Held, now_s: u64, holder_alive: bool) -> bool {
        holder_alive && h.deadline > now_s
    }

    /// A held assertion. Dropping it releases — the RAII shape is what
    /// keeps a Matrix app from leaking what Apple would time out.
    pub struct Guard(PathBuf);

    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    /// Hold an assertion for at most `seconds` (Apple's deadline; a
    /// long-running holder simply renews).
    pub fn hold(kind: Kind, reason: &str, seconds: u64) -> Option<Guard> {
        let d = dir();
        std::fs::create_dir_all(&d).ok()?;
        let pid = std::process::id();
        let file = d.join(format!("{pid}.{}", kind.as_str()));
        let tmp = file.with_extension("new");
        std::fs::write(&tmp, encode(kind, reason, now() + seconds)).ok()?;
        std::fs::rename(&tmp, &file).ok()?;
        Some(Guard(file))
    }

    /// Every assertion that is valid right now — expired ones and those
    /// of dead holders are swept away as we read (self-healing table).
    pub fn active() -> Vec<Held> {
        let mut out = Vec::new();
        let now_s = now();
        let Ok(rd) = std::fs::read_dir(dir()) else { return out };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let Some(pid) = name.split('.').next().and_then(|p| p.parse::<u32>().ok()) else {
                continue;
            };
            let Ok(body) = std::fs::read_to_string(e.path()) else { continue };
            let Some(held) = decode(pid, &body) else { continue };
            let alive = std::path::Path::new(&format!("/proc/{pid}")).exists();
            if valid(&held, now_s, alive) {
                out.push(held);
            } else {
                let _ = std::fs::remove_file(e.path());
            }
        }
        out
    }

    /// Does any live assertion protect this process from napping?
    pub fn holds(pid: u32) -> bool {
        active().iter().any(|h| h.pid == pid)
    }
}

/// List metrics (R98 measurement, 2026-07-20 — the living reference
/// toolkit queried directly, not believed from docs):
/// table row 24 pt · tree indent PER LEVEL exactly 13 pt (not 16,
/// not 20!) · scroller 17 pt (regular) / 15 pt (small) · title bar
/// 32 pt · status bar 22 pt. Wherever a kit app builds lists, these
/// measurements apply instead of guessed round numbers.
pub mod list {
    /// Default row height of a table/list (rowHeight = 24).
    pub const ROW_HEIGHT: f32 = 24.0;
    /// Einzug pro Hierarchie-Ebene (indentationPerLevel = 13).
    pub const INDENT: f32 = 13.0;
    /// Width of the overlay scroller (regular / small).
    pub const SCROLLER: f32 = 17.0;
    pub const SCROLLER_SMALL: f32 = 15.0;
}

/// Reading width (R65b, readableContentGuide extract): body text never
/// runs wider than ~70 characters — the reference's quietest consistency rule.
pub const READING_WIDTH: f32 = 560.0;

/// Fill levels for shapes (R65, UIColor.systemFill extract from the
/// touch-reference-27 SDK): "systemFillColor is appropriate for filling thin and
/// small shapes … quaternary … for filling large areas containing
/// complex content" — the LARGER the surface, the SOFTER the fill.
/// Before, every app mixed its own alphas (0.08/0.12/0.14/0.5 …).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillLevel {
    /// Thin, small shapes (chips, keyboard keys, field grounds).
    Thin,
    /// Medium shapes (row hover, small cards).
    Medium,
    /// Large shapes (cards, panels).
    Large,
    /// Wide surfaces with their own content (consoles, list grounds).
    Wide,
}

impl Palette {
    /// Text-Betonungsstufen (R65, UIColor.label*-Extrakt): VIER Stufen
    /// instead of two — the reference's real consistency tool is the
    /// BENANNTE Hierarchie. 1 = Inhalt, 2 = Begleittext, 3 = Nebensatz,
    /// 4 = barely more than structure (watermarks, placeholder decor).
    ///
    /// SDK measurement R71 (2026-07-19, living AppKit of reference 27
    /// queried directly): the model is ONE ink with alpha steps —
    /// label 0,85 · secondaryLabel 0,50 hell / 0,55 dunkel (dunkel
    /// needs MORE coverage!) · tertiaryLabel 0.26 · separator 0.10.
    /// The R65 values were estimates via on_surface_variant; now
    /// measured instead of believed. Even level 1 is never fully opaque.
    pub fn text_level(&self, level: u8) -> Rgba {
        match level {
            1 => self.on_surface.with_alpha(0.85),
            2 => self.on_surface.with_alpha(if self.is_light { 0.50 } else { 0.55 }),
            3 => self.on_surface.with_alpha(0.26),
            _ => self.on_surface.with_alpha(0.10),
        }
    }

    /// The fill color of a shape by its size class (see `Fuellung`).
    pub fn fill(&self, f: FillLevel) -> Rgba {
        let portion = match f {
            FillLevel::Thin => 0.14,
            FillLevel::Medium => 0.10,
            FillLevel::Large => 0.07,
            FillLevel::Wide => 0.045,
        };
        self.on_surface.over(self.surface_container, portion)
    }
}

/// Lightweight change watcher: reports when palette OR mode
/// have changed (mtime poll — deliberately without a notify dependency).
/// GLASS LEGIBILITY (round 27, the owner 02.11.2026: «the glass more
/// legible»): ink on compositor glass follows what lies BEHIND the glass,
/// not the palette's mode — white ink on glass over a bright green
/// wallpaper was hard to read. `luma` = mean luminance (0..1) of the
/// backdrop under the element (matrixkit_image::backdrop); the glass scrim
/// darkens it by `GLASS_SCRIM` before the decision.
pub mod glass {
    use super::{Palette, Rgba};

    /// The scrim of the niri glass rules (glass.kdl: glass-scrim 0.35).
    pub const GLASS_SCRIM: f32 = 0.35;
    /// Above this effective luminance dark ink reads better than light.
    pub const DARK_INK_ABOVE: f32 = 0.42;

    /// (ink, quiet ink) for text on glass over a backdrop of `luma`.
    /// None (backdrop unknown): the palette's own pair.
    pub fn ink(p: &Palette, luma: Option<f32>) -> (Rgba, Rgba) {
        let Some(l) = luma else { return (p.on_surface, p.on_surface_variant) };
        if l * (1.0 - GLASS_SCRIM) > DARK_INK_ABOVE {
            let dark = Rgba { r: 0.06, g: 0.07, b: 0.08, a: 1.0 };
            (dark, Rgba { a: 0.72, ..dark })
        } else {
            let light = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            (light, Rgba { a: 0.78, ..light })
        }
    }

    /// The palette with the glass ink in place of on_surface(_variant) —
    /// hand this to families that paint text on glass.
    pub fn palette(p: &Palette, luma: Option<f32>) -> Palette {
        let (ink, quiet) = ink(p, luma);
        Palette { on_surface: ink, on_surface_variant: quiet, ..*p }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn bright_backdrop_gets_dark_ink() {
            let p = Palette::default();
            let (bright, _) = ink(&p, Some(0.9));
            let (dark, _) = ink(&p, Some(0.15));
            assert!(bright.r < 0.2 && dark.r > 0.9);
            assert_eq!(ink(&p, None).0, p.on_surface);
            assert_eq!(palette(&p, Some(0.9)).on_surface, bright);
        }
    }
}

pub struct PaletteWatcher {
    last_colors: Option<SystemTime>,
    last_session: Option<SystemTime>,
    /// Accessibility aids (contrast/transparency) act like a palette
    /// change — the watcher reports them too.
    last_a11y: (Option<SystemTime>, Option<SystemTime>),
    last_text: Option<SystemTime>,
    last_fade: bool,
}

fn a11y_path(name: &str) -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config")
        });
    base.join("matrix").join(name)
}

impl PaletteWatcher {
    pub fn new() -> Self {
        Self {
            last_colors: mtime(&colors_path()),
            last_session: mtime(&session_path()),
            last_a11y: (mtime(&a11y_path("contrast")), mtime(&a11y_path("transparency"))),
            last_text: mtime(&a11y_path("text-size")),
            last_fade: false,
        }
    }
    /// true if anything changed since the last call.
    pub fn changed(&mut self) -> bool {
        let c = mtime(&colors_path());
        let s = mtime(&session_path());
        let a = (mtime(&a11y_path("contrast")), mtime(&a11y_path("transparency")));
        let t = mtime(&a11y_path("text-size"));
        let fade = transition::active();
        let changed = c != self.last_colors
            || s != self.last_session
            || a != self.last_a11y
            || t != self.last_text
            || fade
            // Edge: the first look AFTER the fade — for the one
            // finale Nachladen (Icons frisch backen, Ziel exakt setzen).
            || self.last_fade;
        self.last_fade = fade;
        self.last_colors = c;
        self.last_session = s;
        self.last_a11y = a;
        self.last_text = t;
        changed
    }
}

impl Default for PaletteWatcher {
    fn default() -> Self { Self::new() }
}

fn mtime(p: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(p).ok().and_then(|m| m.modified().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_touch_numbers_are_the_measured_ones() {
        // UILongPressGestureRecognizer.h (CLT SDK 26.2): 0.5 s, 10 pt.
        assert_eq!(touch::LONG_PRESS_SECS, 0.5);
        assert_eq!(touch::SLOP, 10.0);
        // The lift charges before the menu and stays subtle.
        assert!(touch::LIFT_DELAY_SECS < touch::LONG_PRESS_SECS);
        assert!(touch::LIFT_SCALE > 1.0 && touch::LIFT_SCALE < 1.1);
    }

    #[test]
    fn hex_parsing() {
        let c = Rgba::from_hex("#ffb59b").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!(Rgba::from_hex("kaputt").is_none());
    }

    #[test]
    fn format_bytes_reference_style() {
        assert_eq!(format::bytes(0), "0 Byte");
        assert_eq!(format::bytes(512), "512 Byte");
        assert_eq!(format::bytes(980_000), "980 KB");
        assert_eq!(format::bytes(4_100_000_000), "4.1 GB");
        assert_eq!(format::bytes(234_000_000), "234 MB");
        assert_eq!(format::bytes_paar(2_400_000_000, 15_500_000_000), "2.4 / 15.5 GB");
        assert_eq!(format::bytes_paar(47_000_000_000, 230_000_000_000), "47 / 230 GB");
        assert_eq!(format::bytes_decimal(17_179_869_184), "16.0 GB");
        assert_eq!(format::bytes_decimal(52_428_800), "50 MB");
        // Relative time (round 14)
        assert_eq!(time::relative(1000, 1030), "gerade eben");
        assert_eq!(time::relative(1000, 1000 + 5 * 60), "vor 5 Min.");
        assert_eq!(time::relative(1000, 1000 + 2 * 3600), "vor 2 Std.");
        assert_eq!(time::relative(1000, 1000 + 30 * 3600), "gestern");
        assert_eq!(time::relative(1000, 1000 + 5 * 86400), "vor 5 Tagen");
        // Undo stack (round 13): LIFO, named, limit of 20
        let mut st = undo::Stack::new();
        assert!(st.empty());
        for i in 0..25 {
            st.remember("Wert", i);
        }
        assert_eq!(st.back(), Some((String::from("Wert"), 24)));
        let mut last = 0;
        while let Some((_, w)) = st.back() {
            last = w;
        }
        assert_eq!(last, 5); // the oldest 5 have yielded to the limit
        assert!(st.empty());
        assert_eq!(format::bytes_decimal_pair(2_576_980_378, 16_642_998_272), "2.4 / 15.5 GB");
        assert_eq!(format::rate(1_200_000.0), "1.2 MB/s");
        assert_eq!(format::rate(-5.0), "0 B/s");
        assert_eq!(format::duration_mmss(1728), "28:48");
        assert_eq!(format::duration_mmss(7), "0:07");
    }

    #[test]
    fn bezier_endpoints_and_monotonicity() {
        for curve in [motion::STANDARD, motion::EMPH_DECEL, motion::EXPRESSIVE_SPATIAL] {
            assert_eq!(curve.eval(0.0), 0.0);
            assert_eq!(curve.eval(1.0), 1.0);
        }
        // Overshoot: expressive may go > 1
        let mut max = 0.0f32;
        for i in 0..=100 {
            max = max.max(motion::EXPRESSIVE_SPATIAL.eval(i as f32 / 100.0));
        }
        assert!(max > 1.0, "expressive Kurve muss federn (max war {max})");
    }

    #[test]
    fn spring_arrives_and_is_interruptible() {
        let mut s = motion::Spring::new(0.0);
        s.retarget(100.0);
        for _ in 0..60 { s.tick(1.0 / 60.0); }
        s.retarget(50.0); // Interruption mid-motion
        let mut steps = 0;
        while s.tick(1.0 / 60.0) && steps < 600 { steps += 1; }
        assert!((s.value - 50.0).abs() < 0.01);
    }

    #[test]
    fn spring_presets_like_reference() {
        // smooth (bounce 0) = critically damped: never past the target
        let running = |mut s: motion::Spring| {
            s.retarget(1.0);
            let mut max = 0.0f32;
            for _ in 0..600 {
                s.tick(1.0 / 60.0);
                max = max.max(s.value);
            }
            assert!((s.value - 1.0).abs() < 0.01, "Feder muss ankommen");
            max
        };
        let smooth = running(motion::Spring::smooth(0.0));
        let snappy = running(motion::Spring::snappy(0.0));
        let springy = running(motion::Spring::springy(0.0));
        assert!(smooth <= 1.0001, "glatt darf nicht überschwingen (war {smooth})");
        assert!(springy > 1.02, "federnd muss sichtbar überschwingen (war {springy})");
        assert!(
            smooth < snappy && snappy < springy,
            "Nachschwung muss mit dem sprung-Wert wachsen ({smooth} / {snappy} / {springy})"
        );
    }

    #[test]
    fn radius_concentricity() {
        use radius::*;
        // The base radius is the anchor.
        assert_eq!(NORMAL, CORNER_RADIUS);
        // Inner radius = outer − inset (concentric corners).
        assert_eq!(inner(NORMAL, spacing::XS), NORMAL - spacing::XS);
        assert_eq!(inner(16.0, 4.0), 12.0);
        // Never below MINI, even at large insets (no angular collapse).
        assert_eq!(inner(12.0, 20.0), MINI);
        // A capsule rounds a surface fully.
        assert_eq!(capsule(12.0), 6.0);
        assert_eq!(capsule(24.0), 12.0);
        // The scale is ordered.
        assert!(MINI < SMALL && SMALL < NORMAL && NORMAL < LARGE);
    }

    #[test]
    fn typo_hierarchy_is_monoton() {
        use typo::*;
        // The roles form a descending size staircase (like the reference's scale).
        let staircase = [LARGE_TITLE, TITLE, SUBTITLE, HEADLINE, BODY, CALLOUT, CAPTION];
        for paar in staircase.windows(2) {
            assert!(
                paar[0].size >= paar[1].size,
                "Typo-Rollen müssen von groß nach klein geordnet sein"
            );
        }
        // Emphasized roles carry more weight than body text of the same size.
        assert_eq!(HEADLINE.size, BODY.size);
        assert_eq!(HEADLINE.weight, Weight::Semibold);
        assert_eq!(BODY.weight, Weight::Normal);
        // The label is tiny but slightly emphasized (stays readable as structure).
        assert_eq!(LABEL.weight, Weight::Medium);
    }
}

/// System-wide setting "reduce motion" (accessibility, like the
/// reference): ~/.config/matrix/bewegung containing "reduziert" switches
/// springs/transitions to instant states.
/// Relative time (Foundation RelativeDateTimeFormatter, round 14):
/// "just now", "5 min ago", "2 hrs ago", "yesterday", "5 days ago".
/// Pure function (testable); from 14 days on the app formats absolutely.
pub mod time {
    /// A PACE (R215 round 22): something may happen at most once per
    /// `every` — a respawn, a log line, a poll. The keyboard guard's own
    /// «at most one start a second» became this.
    #[derive(Debug, Clone, Copy)]
    pub struct Pace {
        every: std::time::Duration,
        last: Option<std::time::Instant>,
    }

    impl Pace {
        pub fn new(every: std::time::Duration) -> Pace {
            Pace { every, last: None }
        }

        /// True (and counts as done) when the last time is long enough ago.
        pub fn ready(&mut self) -> bool {
            self.ready_at(std::time::Instant::now())
        }

        pub fn ready_at(&mut self, now: std::time::Instant) -> bool {
            if self.last.is_some_and(|l| now.duration_since(l) < self.every) {
                return false;
            }
            self.last = Some(now);
            true
        }
    }

    #[cfg(test)]
    mod pace_tests {
        use super::*;
        use std::time::{Duration, Instant};

        #[test]
        fn once_per_interval() {
            let t0 = Instant::now();
            let mut p = Pace::new(Duration::from_secs(1));
            assert!(p.ready_at(t0));
            assert!(!p.ready_at(t0 + Duration::from_millis(500)));
            assert!(p.ready_at(t0 + Duration::from_millis(1000)));
            assert!(!p.ready_at(t0 + Duration::from_millis(1999)));
        }
    }

    /// THE BAR'S CLOCK (round 56): «Di. 19:19» (or «Tue 7:19 PM» with
    /// `am_pm`) without starting `date` each minute. The weekday follows
    /// the first language of the chain: English names for «en…», German
    /// otherwise (the Duo's de_CH printed «Di.»).
    pub fn clock_label(am_pm: bool) -> String {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as libc::time_t).unwrap_or(0);
        // SAFETY: localtime_r writes only into the tm we own.
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        if unsafe { libc::localtime_r(&now, &mut tm) }.is_null() {
            return String::new();
        }
        let english = crate::language::chain().first().is_some_and(|l| l.starts_with("en"));
        clock_label_of(tm.tm_wday, tm.tm_hour, tm.tm_min, am_pm, english)
    }

    /// The pure formatting of `clock_label` (tested).
    pub fn clock_label_of(wday: i32, hour: i32, min: i32, am_pm: bool, english: bool) -> String {
        const DE: [&str; 7] = ["So.", "Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa."];
        const EN: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let day = if english { EN } else { DE }[wday.rem_euclid(7) as usize];
        if am_pm {
            let h = if hour % 12 == 0 { 12 } else { hour % 12 };
            format!("{day} {h}:{min:02} {}", if hour < 12 { "AM" } else { "PM" })
        } else {
            format!("{day} {hour:02}:{min:02}")
        }
    }

    #[cfg(test)]
    mod clock_tests {
        #[test]
        fn german_and_english_labels() {
            assert_eq!(super::clock_label_of(2, 19, 19, false, false), "Di. 19:19");
            assert_eq!(super::clock_label_of(0, 7, 5, false, false), "So. 07:05");
            assert_eq!(super::clock_label_of(2, 19, 19, true, true), "Tue 7:19 PM");
            assert_eq!(super::clock_label_of(1, 0, 0, true, true), "Mon 12:00 AM");
        }
    }

    /// The local time of day «HH:MM:SS» (round 29, for trails and logs).
    pub fn clock_hms() -> String {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as libc::time_t).unwrap_or(0);
        // SAFETY: localtime_r writes only into the tm we own.
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        if unsafe { libc::localtime_r(&now, &mut tm) }.is_null() {
            return String::from("--:--:--");
        }
        format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
    }

    pub fn relative(unix: i64, now: i64) -> String {
        let d = now - unix;
        if d < 90 {
            return String::from("gerade eben");
        }
        let min = d / 60;
        if min < 60 {
            return format!("vor {min} Min.");
        }
        let std = d / 3600;
        if std < 24 {
            return format!("vor {std} Std.");
        }
        let days = d / 86400;
        match days {
            1 => String::from("gestern"),
            2 => String::from("vorgestern"),
            _ => format!("vor {days} Tagen"),
        }
    }

    pub fn relative_now(unix: i64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(unix);
        relative(unix, now)
    }
}

/// Undo culture (reference UndoManager, round 13): a small named
/// undo stack. The app records the old value BEFORE every change;
/// Ctrl+Z (mkw::Key::Undo) brings it back.
pub mod undo {
    pub struct Stack<T> {
        entries: Vec<(String, T)>,
    }

    impl<T> Default for Stack<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> Stack<T> {
        pub fn new() -> Self {
            Self { entries: Vec::new() }
        }

        /// Record the old value BEFORE the change (keeps the last 20).
        pub fn remember(&mut self, name: impl Into<String>, value: T) {
            self.entries.push((name.into(), value));
            if self.entries.len() > 20 {
                self.entries.remove(0);
            }
        }

        /// Take back the latest change: (name, old value).
        pub fn back(&mut self) -> Option<(String, T)> {
            self.entries.pop()
        }

        pub fn empty(&self) -> bool {
            self.entries.is_empty()
        }
    }
}

/// accessibilityReduceTransparency extract (reference round 18):
/// ~/.config/matrix/transparency = "reduced" makes all bar surfaces
/// and veils opaque — readability over glass looks. The German value
/// "reduziert" stays accepted for homes written before R105.
/// ELIDE (R209 family): `text` cut to at most `max` characters, the last
/// one an ellipsis «…» when anything had to go — a visible end instead of
/// a hard cut-off (R139). Counts characters, not bytes (umlauts are safe).
pub fn elide(text: &str, max: usize) -> String {
    // Shorten what the screen shows (the language), not the German key.
    let text = language::shown(text);
    let text = text.as_ref();
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut short: String = text.chars().take(max.saturating_sub(1)).collect();
    short.push('\u{2026}');
    short
}

/// WRAP BY WORDS (R210 family): `text` in at most `max_lines` lines of
/// about `chars` characters, broken between words; what does not fit ends
/// the last line in «…». A word longer than a line is cut. (Video titles,
/// subtitles — Tube spelled it as `title_lines`.)
pub fn wrap_words(text: &str, chars: usize, max_lines: usize) -> Vec<String> {
    // Wrap what the screen shows (the language), not the German key.
    let text = language::shown(text);
    let text = text.as_ref();
    let chars = chars.max(4);
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut words = text.split_whitespace().peekable();
    while let Some(word) = words.next() {
        let word = elide(word, chars);
        if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > chars {
            lines.push(std::mem::take(&mut cur));
            if lines.len() == max_lines {
                let last = lines.pop().unwrap_or_default();
                lines.push(elide(&format!("{last} {word}"), chars).trim_end_matches('\u{2026}').to_string() + "\u{2026}");
                return lines;
            }
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(&word);
        let _ = words.peek();
    }
    if !cur.is_empty() && lines.len() < max_lines.max(1) {
        lines.push(cur);
    }
    lines
}

#[cfg(test)]
mod wrap_tests {
    use super::wrap_words;

    #[test]
    fn wraps_between_words_and_ends_in_ellipsis() {
        assert_eq!(wrap_words("eins zwei drei", 9, 2), vec!["eins zwei", "drei"]);
        let w = wrap_words("eins zwei drei vier fünf sechs", 9, 2);
        assert_eq!(w.len(), 2);
        assert!(w[1].ends_with('\u{2026}'));
        assert_eq!(wrap_words("", 10, 2), Vec::<String>::new());
    }
}

#[cfg(test)]
mod elide_tests {
    #[test]
    fn elide_keeps_short_and_ends_long_in_ellipsis() {
        assert_eq!(super::elide("Dateien", 22), "Dateien");
        assert_eq!(super::elide("Matrix Einstellungen", 8), "Matrix \u{2026}");
        assert_eq!(super::elide("Grüsse aus Zürich", 6).chars().count(), 6);
        assert_eq!(super::elide("abc", 3), "abc");
    }
}

/// IS GLASS ON? (R209 family, iced-free): on unless switched off in the
/// settings, and never against the accessibility contract (reduced
/// transparency = opaque surfaces). Bar, dock, Start and the house ask here.
pub fn glass_wanted() -> bool {
    !transparency_reduced() && settings::read("glass").as_deref() != Some("off")
}

/// THE GRAPHICS PATH (R209 family, iced-free): what every wgpu user must
/// set BEFORE its instance exists. On a duo: Vulkan only (left to choose,
/// background daemons took OpenGL, which /etc/environment routes through
/// zink — deadlock on surface switches), and no direct-display extensions
/// (with VK_KHR_display Turnip on KGSL fails its enumeration and wgpu falls
/// back to llvmpipe on the CPU; our vendored wgpu-hal honours the switch).
/// iced apps reach it through `present_mailbox`, the house's GPU painter
/// (MATRIX_GPU) directly.
pub mod gpu_path {
    pub fn prepare() {
        if !crate::device::is_duo() {
            return;
        }
        if std::env::var("WGPU_BACKEND").is_err() {
            std::env::set_var("WGPU_BACKEND", "vulkan");
        }
        std::env::set_var("WGPU_SKIP_DISPLAY_EXTENSIONS", "1");
    }
}

/// Processes of the Kit (R209 family, iced-free).
pub mod process {
    /// FOLLOW A HELPER'S LINES (round 49): run `make()` with its stdout
    /// piped, hand every line to `on_line(Some(line))`, and when it ends or
    /// cannot start, say so with `on_line(None)` and start it again after
    /// `pause`. `on_line` returning false ends the follow — the helper is
    /// killed AND reaped (`end`). Three hand-written copies of this loop
    /// (sound generation, hearing guard, sound hooks) each had to remember
    /// the reconnect and the reaping; the canvas stream forgot the latter.
    pub fn follow(
        make: impl Fn() -> std::process::Command + Send + 'static,
        pause: std::time::Duration,
        mut on_line: impl FnMut(Option<&str>) -> bool + Send + 'static,
    ) -> std::thread::JoinHandle<()> {
        use std::io::BufRead;
        std::thread::spawn(move || loop {
            let spawned = make().stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null()).spawn();
            if let Ok(mut child) = spawned {
                if let Some(out) = child.stdout.take() {
                    for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                        if !on_line(Some(&line)) {
                            end(&mut child);
                            return;
                        }
                    }
                }
                let _ = child.wait();
            }
            if !on_line(None) {
                return;
            }
            std::thread::sleep(pause);
        })
    }

    #[cfg(test)]
    mod follow_tests {
        use std::process::Command;
        use std::time::Duration;

        #[test]
        fn lines_arrive_and_false_stops() {
            let (tx, rx) = std::sync::mpsc::channel();
            let h = super::follow(
                || {
                    let mut c = Command::new("sh");
                    c.args(["-c", "printf 'a\\nb\\nc\\n'; sleep 30"]);
                    c
                },
                Duration::from_millis(10),
                move |line| {
                    let line = line.map(str::to_string);
                    let go = line.as_deref() != Some("b");
                    let _ = tx.send(line);
                    go
                },
            );
            h.join().unwrap(); // returns although `sleep 30` still ran: the helper was ended
            let got: Vec<Option<String>> = rx.try_iter().collect();
            assert_eq!(got, vec![Some("a".into()), Some("b".into())]);
        }

        #[test]
        fn an_ended_helper_is_restarted() {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut ends = 0;
            let h = super::follow(
                || {
                    let mut c = Command::new("sh");
                    c.args(["-c", "echo x"]);
                    c
                },
                Duration::from_millis(5),
                move |line| {
                    if line.is_none() {
                        ends += 1;
                    }
                    let _ = tx.send(line.map(str::to_string));
                    ends < 3
                },
            );
            h.join().unwrap();
            let got: Vec<Option<String>> = rx.try_iter().collect();
            assert_eq!(got.iter().filter(|l| l.as_deref() == Some("x")).count(), 3);
            assert_eq!(got.iter().filter(|l| l.is_none()).count(), 3);
        }
    }

    /// END A CHILD (round 48): kill AND reap. A bare `kill()` leaves a
    /// zombie until the parent exits — `canvas_stream` left one `[niri]`
    /// per closed card in matrix-context (two after 7 h on the Duo).
    pub fn end(child: &mut std::process::Child) {
        let _ = child.kill();
        let _ = child.wait();
    }

    /// DIE WITH THE PARENT: the kernel ends this process (SIGTERM) when the
    /// process that started it dies. For helper surfaces a daemon spawns
    /// (keyboard pill, wallpaper painters): killed or replaced, the parent
    /// used to leave orphans behind — still running the old binary.
    pub fn die_with_parent() {
        // SAFETY: prctl with PR_SET_PDEATHSIG only sets a flag of this
        // process; async-signal-safe, so it may also run in `pre_exec`.
        unsafe {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
        }
    }

    /// OUTPUT CONDUCTOR (round 155, 05.11.2026): this program once per
    /// screen — a child `--output NAME` (plus this process's own arguments)
    /// for each of `fixed` (the duo's panels) AND each screen the canvas has
    /// switched on (a TV on USB-C). Checked every 3 s: a screen plugged in
    /// gets its child, an unplugged one's child ends, a child that died
    /// starts anew. Children die with the conductor. Never returns.
    pub fn per_output(fixed: &[&str]) -> ! {
        let me = own_exe();
        let rest: Vec<String> = std::env::args().skip(1).collect();
        let mut children: std::collections::HashMap<String, std::process::Child> = std::collections::HashMap::new();
        loop {
            let want = conducted(fixed, &crate::canvas::outputs());
            for name in &want {
                let alive = children.get_mut(name).is_some_and(|c| matches!(c.try_wait(), Ok(None)));
                if alive {
                    continue;
                }
                let mut command = std::process::Command::new(&me);
                command.args(["--output", name.as_str()]).args(&rest);
                // SAFETY: pre_exec runs only the async-signal-safe prctl.
                unsafe {
                    use std::os::unix::process::CommandExt;
                    command.pre_exec(|| {
                        die_with_parent();
                        Ok(())
                    });
                }
                match command.spawn() {
                    Ok(child) => {
                        eprintln!("[ausgang] {name}: Maler gestartet");
                        children.insert(name.clone(), child);
                    }
                    Err(e) => eprintln!("[ausgang] {name}: Start gescheitert: {e}"),
                }
            }
            children.retain(|name, child| {
                if want.contains(name) {
                    return true;
                }
                eprintln!("[ausgang] {name}: Bildschirm weg — Maler endet");
                let _ = child.kill();
                let _ = child.wait();
                false
            });
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }

    /// The screens a conductor serves: the fixed ones first, then every
    /// other switched-on screen (no doubles).
    pub fn conducted(fixed: &[&str], on: &[String]) -> Vec<String> {
        let mut want: Vec<String> = fixed.iter().map(|s| s.to_string()).collect();
        for o in on {
            if !want.contains(o) {
                want.push(o.clone());
            }
        }
        want
    }

    #[cfg(test)]
    mod conductor_tests {
        #[test]
        fn a_plugged_screen_joins_the_panels() {
            let on = vec![String::from("DSI-1"), String::from("DP-1"), String::from("DSI-2")];
            assert_eq!(super::conducted(&["DSI-1", "DSI-2"], &on), vec!["DSI-1", "DSI-2", "DP-1"]);
            assert_eq!(super::conducted(&["DSI-1", "DSI-2"], &[]), vec!["DSI-1", "DSI-2"], "panels even before niri answers");
        }
    }

    /// How a started child is doing (R215 family, 02.11.2026).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Watch {
        Running,
        /// Ended; the exit code when it had one (None: killed by a signal).
        Exited(Option<i32>),
        /// Gone without a status for us — reaped elsewhere (ECHILD) or the
        /// pid vanished. Before this family the keyboard guard read only
        /// `Ok(Some)` and believed such a child alive forever: no keyboard
        /// came again (Duo 02.11.2026 15:35).
        Vanished,
    }

    impl Watch {
        pub fn alive(self) -> bool {
            self == Watch::Running
        }
    }

    /// WATCH A CHILD: never "alive" for a process that is not there.
    pub fn watch(child: &mut std::process::Child) -> Watch {
        match child.try_wait() {
            Ok(Some(status)) => Watch::Exited(status.code()),
            Err(_) => Watch::Vanished,
            Ok(None) => {
                if std::path::Path::new(&format!("/proc/{}", child.id())).exists() {
                    Watch::Running
                } else {
                    Watch::Vanished
                }
            }
        }
    }

    #[cfg(test)]
    mod watch_tests {
        use super::*;
        use std::process::Command;

        #[test]
        fn end_leaves_no_zombie() {
            let mut child = std::process::Command::new("sleep").arg("5").spawn().unwrap();
            let pid = child.id();
            super::end(&mut child);
            assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists(), "zombie left behind");
        }

        #[test]
        fn running_exited_vanished() {
            let mut sleeper = Command::new("sleep").arg("5").spawn().unwrap();
            assert_eq!(watch(&mut sleeper), Watch::Running);
            super::end(&mut sleeper);
            let mut quick = Command::new("true").spawn().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(200));
            assert_eq!(watch(&mut quick), Watch::Exited(Some(0)));
            // Reaped behind the Child's back: the guard's case.
            let mut stolen = Command::new("true").spawn().unwrap();
            // SAFETY: waitpid on our own child's pid.
            unsafe { libc::waitpid(stolen.id() as i32, std::ptr::null_mut(), 0) };
            assert_eq!(watch(&mut stolen), Watch::Vanished);
            assert!(!watch(&mut stolen).alive());
        }
    }

    /// THIS PROGRAM'S FILE, for starting itself again (R215 round 21).
    /// After an update replaced the binary on disk, Linux names the running
    /// one «…/matrix-keyboard (deleted)» — exec of that path fails (ENOENT).
    /// The keyboard guard tried 8700 times (Duo 02.11.2026), Tube's stage
    /// and others would have failed the same way. Rule: the NEW file at the
    /// same place if it exists (the update wins), else /proc/self/exe (the
    /// running, deleted inode still executes).
    pub fn own_exe() -> std::path::PathBuf {
        let me = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("/proc/self/exe"));
        own_exe_from(&me, |p| p.is_file())
    }

    /// Was this program's file replaced since it started (an update)?
    /// Then a long-running daemon can start itself anew at a quiet moment
    /// (`reexec`) instead of running the old code until the next login.
    pub fn updated() -> bool {
        let me = std::env::current_exe().unwrap_or_default();
        updated_from(&me, |p| p.is_file())
    }

    pub fn updated_from(me: &std::path::Path, exists: impl Fn(&std::path::Path) -> bool) -> bool {
        me.to_string_lossy().strip_suffix(" (deleted)").is_some_and(|plain| exists(std::path::Path::new(plain)))
    }

    /// THIS PROCESS'S RESIDENT MEMORY in MB (round 32): for daemons that
    /// keep a ceiling — a long-lived iced daemon that grows per surface
    /// (the context menu, ~18 MB per menu) starts anew when idle.
    pub fn rss_mb() -> Option<u64> {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let kb: u64 = status.lines().find(|l| l.starts_with("VmRSS:"))?.split_whitespace().nth(1)?.parse().ok()?;
        Some(kb / 1024)
    }

    /// Open file descriptors of this process (round 52).
    pub fn fd_count() -> Option<usize> {
        Some(std::fs::read_dir("/proc/self/fd").ok()?.count())
    }

    /// THE LEAK PROBE (round 52): a daemon watches itself. `check` is fed
    /// its fd count and RSS now and then; it remembers the lowest values
    /// as the baseline and answers a warning when one has doubled AND grown
    /// by a real amount (64 fds / 100 MB) — once per doubling, the baseline
    /// moves up then. Both leaks of 03.11.2026 (a hung niri child with
    /// 700 MB, 944 pane buffers) were found by hand; this finds the next.
    #[derive(Debug, Default)]
    pub struct LeakProbe {
        fd: Option<usize>,
        rss: Option<u64>,
    }

    impl LeakProbe {
        pub fn check(&mut self, fd: usize, rss_mb: u64) -> Option<String> {
            let mut out = Vec::new();
            match self.fd {
                Some(base) if fd >= base * 2 && fd >= base + 64 => {
                    out.push(format!("offene Dateien {base} → {fd}"));
                    self.fd = Some(fd);
                }
                Some(base) if fd < base => self.fd = Some(fd),
                None => self.fd = Some(fd),
                _ => {}
            }
            match self.rss {
                Some(base) if rss_mb >= base * 2 && rss_mb >= base + 100 => {
                    out.push(format!("Speicher {base} → {rss_mb} MB"));
                    self.rss = Some(rss_mb);
                }
                Some(base) if rss_mb < base => self.rss = Some(rss_mb),
                None => self.rss = Some(rss_mb),
                _ => {}
            }
            (!out.is_empty()).then(|| format!("Leck-Verdacht: {}", out.join(", ")))
        }

        /// Measure this process and check (None when /proc is unreadable).
        pub fn check_self(&mut self) -> Option<String> {
            self.check(fd_count()?, rss_mb()?)
        }
    }

    #[cfg(test)]
    mod leak_tests {
        use super::LeakProbe;

        #[test]
        fn steady_is_quiet_doubling_warns_once() {
            let mut p = LeakProbe::default();
            assert_eq!(p.check(20, 40), None);
            assert_eq!(p.check(30, 60), None); // small growth
            assert_eq!(p.check(50, 60), None); // doubled but < +64
            assert!(p.check(90, 60).unwrap().contains("20 → 90"));
            assert_eq!(p.check(100, 60), None); // baseline moved up
            assert!(p.check(180, 60).is_some()); // next doubling
            assert!(p.check(180, 250).unwrap().contains("40 → 250 MB"));
        }

        #[test]
        fn the_baseline_follows_the_lowest_value() {
            let mut p = LeakProbe::default();
            p.check(200, 300);
            p.check(30, 50); // after start-up settled
            assert!(p.check(100, 50).is_some());
        }
    }

    /// Become the new version: exec the file on disk with the same
    /// arguments (same pid; fds close on exec). Returns only on failure.
    pub fn reexec() -> std::io::Error {
        use std::os::unix::process::CommandExt;
        let args: Vec<String> = std::env::args().skip(1).collect();
        std::process::Command::new(own_exe()).args(args).exec()
    }

    /// `own_exe` for a returned io::Result (drop-in for current_exe()).
    /// Round 83: start this program anew IN THE SAME PROCESS (exec keeps
    /// the pid — a watchdog sees no gap), its `--output NAME` replaced by
    /// `output` (None: dropped). A shell part that must move to another
    /// panel (folded duo). Returns only when exec failed.
    pub fn exec_on_output(output: Option<&str>) -> std::io::Error {
        use std::os::unix::process::CommandExt as _;
        let exe = match own_exe_result() {
            Ok(e) => e,
            Err(e) => return e,
        };
        let mut args: Vec<String> = Vec::new();
        let mut skip = false;
        for a in std::env::args().skip(1) {
            if skip {
                skip = false;
                continue;
            }
            if a == "--output" {
                skip = true;
                continue;
            }
            args.push(a);
        }
        if let Some(o) = output {
            args.push(String::from("--output"));
            args.push(o.to_string());
        }
        // Round 103: exec keeps our CHILDREN but drops the threads that reap
        // them — every fold left `niri msg` zombies behind the dock (14 in an
        // hour, 04.11.2026). The compositor helpers end here and are reaped;
        // any other child is handed to the new image, which reaps it later.
        let strays = end_children(&["niri"]);
        let mut cmd = std::process::Command::new(exe);
        cmd.args(&args);
        if !strays.is_empty() {
            cmd.env(STRAYS_ENV, strays.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(","));
        }
        cmd.exec()
    }

    const STRAYS_ENV: &str = "MATRIX_STRAY_CHILDREN";

    /// Our direct children (pid, comm).
    fn children() -> Vec<(i32, String)> {
        let me = std::process::id().to_string();
        std::fs::read_dir("/proc")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let pid: i32 = e.file_name().to_string_lossy().parse().ok()?;
                let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
                // «pid (comm) state ppid …» — comm may hold spaces: split at the last ')'
                let (head, tail) = stat.rsplit_once(')')?;
                let comm = head.split_once('(')?.1.to_string();
                let ppid = tail.split_whitespace().nth(1)?;
                (ppid == me).then_some((pid, comm))
            })
            .collect()
    }

    /// End the children named in `helpers` (SIGTERM, reaped within 300 ms);
    /// returns the pids of every other child.
    fn end_children(helpers: &[&str]) -> Vec<i32> {
        let mut others = Vec::new();
        let mut ending = Vec::new();
        for (pid, comm) in children() {
            if helpers.contains(&comm.as_str()) {
                // SAFETY: signalling our own child.
                unsafe { libc::kill(pid, libc::SIGTERM) };
                ending.push(pid);
            } else {
                others.push(pid);
            }
        }
        let until = std::time::Instant::now() + std::time::Duration::from_millis(300);
        while !ending.is_empty() && std::time::Instant::now() < until {
            // SAFETY: waitpid on our own children, never blocking.
            ending.retain(|&pid| unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) } == 0);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        others.extend(ending);
        others
    }

    #[cfg(test)]
    mod stray_tests {
        /// A child named «niri» ends and is reaped; another child is handed on.
        #[test]
        fn helpers_end_others_are_handed_on() {
            let dir = std::env::temp_dir().join(format!("mk-strays-{}", std::process::id()));
            let _ = std::fs::create_dir_all(&dir);
            let niri = dir.join("niri");
            std::fs::copy("/usr/bin/sleep", &niri).unwrap();
            let helper = std::process::Command::new(&niri).arg("30").spawn().unwrap();
            let mut other = std::process::Command::new("sleep").arg("30").spawn().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
            let left = super::end_children(&["niri"]);
            assert!(left.contains(&(other.id() as i32)));
            assert!(!left.contains(&(helper.id() as i32)));
            // reaped already: no such child any more
            let r = unsafe { libc::waitpid(helper.id() as i32, std::ptr::null_mut(), libc::WNOHANG) };
            assert_eq!(r, -1);
            let _ = other.kill();
            let _ = other.wait();
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    /// Round 103: reap the children an earlier image of this process left
    /// (see `exec_on_output`) — only those pids, so no other thread's
    /// `Child::wait` is robbed. Cheap: a sleeping thread, gone when done.
    pub fn adopt_strays() {
        let Some(list) = std::env::var_os(STRAYS_ENV) else { return };
        // SAFETY: removing our own variable before other threads read the env.
        unsafe { std::env::remove_var(STRAYS_ENV) };
        let mut pids: Vec<i32> = list.to_string_lossy().split(',').filter_map(|p| p.parse().ok()).collect();
        if pids.is_empty() {
            return;
        }
        std::thread::spawn(move || {
            while !pids.is_empty() {
                // SAFETY: waitpid on exactly the inherited children, never blocking.
                pids.retain(|&pid| unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) } == 0);
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        });
    }

    pub fn own_exe_result() -> std::io::Result<std::path::PathBuf> {
        Ok(own_exe())
    }

    /// Pure core of `own_exe` (tested): strip « (deleted)» when the plain
    /// path exists, else fall back to /proc/self/exe.
    pub fn own_exe_from(me: &std::path::Path, exists: impl Fn(&std::path::Path) -> bool) -> std::path::PathBuf {
        let text = me.to_string_lossy();
        match text.strip_suffix(" (deleted)") {
            Some(plain) if exists(std::path::Path::new(plain)) => std::path::PathBuf::from(plain),
            Some(_) => std::path::PathBuf::from("/proc/self/exe"),
            None => me.to_path_buf(),
        }
    }

    #[cfg(test)]
    mod exe_tests {
        use super::*;
        use std::path::Path;

        #[test]
        fn a_replaced_binary_starts_the_new_file() {
            let deleted = Path::new("/usr/local/bin/matrix-keyboard (deleted)");
            assert_eq!(own_exe_from(deleted, |_| true), Path::new("/usr/local/bin/matrix-keyboard"));
            assert_eq!(own_exe_from(deleted, |_| false), Path::new("/proc/self/exe"));
            let fine = Path::new("/usr/local/bin/matrix-tube");
            assert_eq!(own_exe_from(fine, |_| false), fine);
            assert!(own_exe().exists());
            assert!(updated_from(Path::new("/usr/bin/x (deleted)"), |_| true));
            assert!(!updated_from(Path::new("/usr/bin/x (deleted)"), |_| false));
            assert!(!updated_from(Path::new("/usr/bin/x"), |_| true));
        }
    }

    /// DETACHED (round 26): start a program that outlives us — also an
    /// `exec` of ourselves (a thread waiting for it would die with the old
    /// image, and the child would never run its follow-up). `sh` puts it in
    /// the background and returns at once; init adopts it, no zombie.
    pub fn spawn_detached(program: &str, args: &[&str]) -> bool {
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").arg("\"$0\" \"$@\" >/dev/null 2>&1 &").arg(program).args(args);
        cmd.status().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(test)]
    mod detached_tests {
        #[test]
        fn returns_at_once_and_runs() {
            let mark = std::env::temp_dir().join(format!("mk-detached-{}", std::process::id()));
            let t0 = std::time::Instant::now();
            assert!(super::spawn_detached("sh", &["-c", &format!("sleep 0.3; touch {}", mark.display())]));
            assert!(t0.elapsed() < std::time::Duration::from_millis(250), "did not wait");
            std::thread::sleep(std::time::Duration::from_millis(700));
            assert!(mark.exists(), "the detached job ran");
            let _ = std::fs::remove_file(mark);
        }
    }

    /// START AN APP by name (round 27, iced-free — the dock's and the
    /// widgets' copies become this): the user's ~/.local/bin first (dev
    /// slot), else PATH; reaped by its own thread.
    pub fn app_start(name: &str, args: &[&str]) -> bool {
        let local = std::env::var("HOME").map(|h| format!("{h}/.local/bin/{name}")).unwrap_or_default();
        let program = if std::path::Path::new(&local).is_file() { local } else { name.to_string() };
        spawn_reaped(std::process::Command::new(program).args(args))
    }

    /// A shell line in the background, reaped (round 27).
    pub fn sh_spawn(line: &str) -> bool {
        spawn_reaped(std::process::Command::new("sh").args(["-c", line]))
    }

    /// START AND FORGET, WITHOUT A ZOMBIE (R210 family): spawn `cmd` and
    /// wait for it on a thread of its own — a child that ends must not
    /// stay behind as `<defunct>` of a long-living app (Tube's stage,
    /// 01.11.2026). False if it did not start.
    ///
    /// Round 51: seven call sites still dropped the Child (`let _ =
    /// cmd.spawn()`) — every app launched from the bar, every copy from the
    /// context menu stayed a zombie of that daemon. They use this now. The
    /// waiting thread is named and small (64 KiB stack).
    pub fn spawn_reaped(cmd: &mut std::process::Command) -> bool {
        match cmd.spawn() {
            Ok(mut child) => std::thread::Builder::new()
                .name(String::from("mk-reaper"))
                .stack_size(64 * 1024)
                .spawn(move || {
                    let _ = child.wait();
                })
                .is_ok(),
            Err(_) => false,
        }
    }

    #[cfg(test)]
    mod reaped_tests {
        #[test]
        fn an_ended_child_leaves_no_zombie() {
            let mark = std::env::temp_dir().join(format!("mk-reaped-{}", std::process::id()));
            assert!(super::spawn_reaped(std::process::Command::new("sh").args(["-c", &format!("echo $$ > {}", mark.display())])));
            std::thread::sleep(std::time::Duration::from_millis(400));
            let pid = std::fs::read_to_string(&mark).unwrap();
            let _ = std::fs::remove_file(&mark);
            assert!(!std::path::Path::new(&format!("/proc/{}", pid.trim())).exists(), "zombie left behind");
        }
    }
}

/// THE BAR'S MAILBOX (round 35, 02.11.2026): a layer shell part has no
/// window mailbox (`mk::house` is for windows) — the top bar listens on a
/// datagram socket of its own. The dock's «Apps» tile asks it for the app
/// card («apps DSI-1»); `ask` is false when no bar listens (the caller
/// then falls back, e.g. to matrix-start).
pub mod bar {
    use std::os::unix::net::UnixDatagram;

    fn path() -> std::path::PathBuf {
        crate::runtime::path("matrix-bar.sock")
    }

    pub fn ask(word: &str) -> bool {
        UnixDatagram::unbound().and_then(|s| s.send_to(word.as_bytes(), path())).is_ok()
    }

    /// The bar: bind the mailbox (an old one of a dead bar is replaced).
    pub fn listen() -> Option<UnixDatagram> {
        let p = path();
        let _ = std::fs::remove_file(&p);
        UnixDatagram::bind(&p).ok()
    }
}

/// THE QUICK CONTROLS (round 34, 02.11.2026): the control centre's
/// readers and writers as a kit family — sound, microphone, WLAN,
/// Bluetooth, brightness and the access switches. Before they lived only
/// in matrix-central; the top bar's control card (house, no iced) uses
/// them now. Reading spawns tools (wpctl, nmcli, bluetoothctl,
/// brightnessctl/ddcutil): call `read()` OFF the drawing thread.
pub mod quick {
    /// Where the brightness comes from: the laptop backlight or monitor DDC.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Light {
        Backlight,
        Ddc,
    }

    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct State {
        /// (level 0..1, muted).
        pub volume: Option<(f32, bool)>,
        pub mic_muted: Option<bool>,
        pub wlan: Option<bool>,
        pub bluetooth: Option<bool>,
        /// (percent 0..100, source).
        pub brightness: Option<(f32, Light)>,
        pub motion_reduced: bool,
        pub transparency_reduced: bool,
        pub contrast_up: bool,
        /// Round 64: a screen recording runs (matrix-capture's marker).
        pub recording: bool,
        /// Round 71: Night Shift is on (always or by the sun).
        pub night_shift: bool,
        /// Round 72: automatic brightness (Duo: duo-haltung and the light sensor).
        pub auto_bright: bool,
        /// Round 73: the rotation is locked.
        pub rotation_lock: bool,
        /// Round 79: deep sleep allowed (duo-schlaf obeys the setting).
        pub deep_sleep: bool,
    }

    /// Is a screen recording running? (matrix-capture keeps the marker
    /// `matrix-capture-recording` in the runtime directory while it films.)
    pub fn recording() -> bool {
        crate::runtime::path("matrix-capture-recording").exists()
    }

    /// What a control is on screen.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Kind {
        Toggle(bool),
        /// 0..1
        Slider(f32),
        Action,
    }

    /// "Volume: 0.45 [MUTED]" → (0.45, true)
    pub fn sound(target: &str) -> Option<(f32, bool)> {
        let z = crate::command::first_line("wpctl", &["get-volume", target])?;
        let value: f32 = z.split_whitespace().nth(1)?.parse().ok()?;
        Some((value, z.contains("MUTED")))
    }

    pub fn wlan() -> Option<bool> {
        crate::command::first_line("nmcli", &["radio", "wifi"]).map(|z| z == "enabled")
    }

    pub fn bluetooth() -> Option<bool> {
        let out = std::process::Command::new("bluetoothctl").arg("show").output().ok()?;
        let s = String::from_utf8_lossy(&out.stdout);
        if !s.contains("Controller") {
            return None;
        }
        Some(s.lines().any(|l| l.trim() == "Powered: yes"))
    }

    /// First the laptop backlight (fast), then monitor DDC (sluggish).
    pub fn brightness() -> Option<(f32, Light)> {
        if let Some(z) = crate::command::first_line("brightnessctl", &crate::device::brightnessctl_args(&["-m"])) {
            if let Some(v) = z.split(',').nth(3).and_then(|p| p.trim_end_matches('%').parse::<f32>().ok()) {
                return Some((v, Light::Backlight));
            }
        }
        let z = crate::command::first_line("ddcutil", &["getvcp", "10", "--brief"])?;
        Some((z.split_whitespace().nth(3)?.parse().ok()?, Light::Ddc))
    }

    pub fn read() -> State {
        State {
            volume: sound("@DEFAULT_AUDIO_SINK@"),
            mic_muted: sound("@DEFAULT_AUDIO_SOURCE@").map(|(_, m)| m),
            wlan: wlan(),
            bluetooth: bluetooth(),
            brightness: brightness(),
            motion_reduced: crate::motion_reduced(),
            transparency_reduced: crate::transparency_reduced(),
            contrast_up: crate::contrast_up(),
            recording: recording(),
            night_shift: crate::night::mode() != crate::night::Mode::Off,
            auto_bright: crate::display::auto_brightness(),
            rotation_lock: !crate::display::auto_rotation(),
            deep_sleep: crate::power::deep_sleep(),
        }
    }

    impl State {
        /// The control `key` as a toggle, a slider or an action.
        pub fn kind(&self, key: &str) -> Kind {
            match key {
                "mute" => Kind::Toggle(self.volume.is_some_and(|(_, m)| m)),
                "mic" => Kind::Toggle(self.mic_muted.unwrap_or(false)),
                "wlan" => Kind::Toggle(self.wlan.unwrap_or(false)),
                "bluetooth" => Kind::Toggle(self.bluetooth.unwrap_or(false)),
                "motion" => Kind::Toggle(self.motion_reduced),
                "transparency" => Kind::Toggle(self.transparency_reduced),
                "contrast" => Kind::Toggle(self.contrast_up),
                "recording" => Kind::Toggle(self.recording),
                "nightshift" => Kind::Toggle(self.night_shift),
                "autobright" => Kind::Toggle(self.auto_bright),
                "rotlock" => Kind::Toggle(self.rotation_lock),
                "deepsleep" => Kind::Toggle(self.deep_sleep),
                "volume" => Kind::Slider(self.volume.map(|(v, _)| v.clamp(0.0, 1.0)).unwrap_or(0.0)),
                "brightness" => Kind::Slider(self.brightness.map(|(v, _)| (v / 100.0).clamp(0.0, 1.0)).unwrap_or(0.0)),
                _ => Kind::Action,
            }
        }

        /// Flip a toggle: writes it and updates this state at once (the
        /// next `read()` confirms).
        pub fn toggle(&mut self, key: &str) {
            match key {
                "mute" => {
                    crate::command::silent("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]);
                    if let Some((v, m)) = self.volume {
                        self.volume = Some((v, !m));
                    }
                }
                "mic" => {
                    crate::command::silent("wpctl", &["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"]);
                    self.mic_muted = self.mic_muted.map(|m| !m);
                }
                "wlan" => {
                    let on = !self.wlan.unwrap_or(false);
                    crate::command::silent("nmcli", &["radio", "wifi", if on { "on" } else { "off" }]);
                    self.wlan = Some(on);
                }
                "bluetooth" => {
                    let on = !self.bluetooth.unwrap_or(false);
                    crate::command::silent("bluetoothctl", &["power", if on { "on" } else { "off" }]);
                    self.bluetooth = Some(on);
                }
                "deepsleep" => {
                    self.deep_sleep = !self.deep_sleep;
                    crate::power::set_deep_sleep(self.deep_sleep);
                }
                "rotlock" => {
                    self.rotation_lock = !self.rotation_lock;
                    crate::display::set_auto_rotation(!self.rotation_lock);
                }
                "autobright" => {
                    self.auto_bright = !self.auto_bright;
                    crate::display::set_auto_brightness(self.auto_bright);
                }
                "nightshift" => {
                    // Off ⇄ by the sun (the evening default); «always» is
                    // a choice in Einstellungen › Anzeige › Night Shift.
                    self.night_shift = !self.night_shift;
                    crate::night::set_mode(if self.night_shift { crate::night::Mode::Auto } else { crate::night::Mode::Off });
                }
                "motion" => {
                    self.motion_reduced = !self.motion_reduced;
                    crate::settings::write("motion", if self.motion_reduced { "reduced" } else { "full" });
                }
                "transparency" => {
                    self.transparency_reduced = !self.transparency_reduced;
                    crate::settings::write("transparency", if self.transparency_reduced { "reduced" } else { "full" });
                }
                "contrast" => {
                    self.contrast_up = !self.contrast_up;
                    crate::settings::write("contrast", if self.contrast_up { "high" } else { "normal" });
                }
                _ => {}
            }
        }

        /// A slider moved to `t` (0..1). Volume is written at once; the
        /// brightness only remembered — `brightness_commit` writes it
        /// (DDC is slow: on release, off the drawing thread).
        pub fn slide(&mut self, key: &str, t: f32) {
            let t = t.clamp(0.0, 1.0);
            match key {
                "volume" => {
                    crate::command::silent("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{t:.2}")]);
                    if let Some((_, m)) = self.volume {
                        self.volume = Some((t, m));
                    }
                }
                "brightness" => {
                    if let Some((_, src)) = self.brightness {
                        self.brightness = Some((t * 100.0, src));
                    }
                }
                _ => {}
            }
        }
    }

    /// Write a brightness (percent) — slow on DDC: off the drawing thread.
    pub fn brightness_commit(value: f32, source: Light) {
        match source {
            Light::Backlight => {
                let v = format!("{value:.0}%");
                crate::command::silent("brightnessctl", &crate::device::brightnessctl_args(&["set", &v]));
            }
            Light::Ddc => {
                crate::command::silent("ddcutil", &["setvcp", "10", &format!("{value:.0}")]);
            }
        }
    }

    /// An action control (appearance, lock, capture).
    pub fn act(key: &str) {
        match key {
            "appearance" => crate::light_toggle(),
            "lock" => {
                crate::process::app_start("matrix-lock", &[]);
            }
            "capture" => {
                crate::process::app_start("matrix-capture", &[]);
            }
            // Round 64: a photo of the whole screen — a moment later, so
            // the control centre that asked has shrunk away first.
            "screenshot" => {
                crate::process::sh_spawn("sleep 0.9; exec matrix-capture photo");
            }
            _ => {}
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{Kind, Light, State};

        #[test]
        fn kinds_follow_the_state() {
            let s = State { volume: Some((0.4, true)), wlan: Some(true), brightness: Some((80.0, Light::Backlight)), ..State::default() };
            assert_eq!(s.kind("mute"), Kind::Toggle(true));
            assert_eq!(s.kind("wlan"), Kind::Toggle(true));
            assert_eq!(s.kind("bluetooth"), Kind::Toggle(false));
            assert_eq!(s.kind("volume"), Kind::Slider(0.4));
            assert_eq!(s.kind("brightness"), Kind::Slider(0.8));
            assert_eq!(s.kind("lock"), Kind::Action);
        }
    }
}

/// THE SUN (round 71): sunrise and sunset for the user's place (setting
/// `location` "lat,lon", Zurich by default) — NOAA short form, precise to
/// a few minutes. Was the wallpaper's own (`sun_top`); Night Shift needs
/// it too, so it moved into the kit. Pure maths plus a clock, no `date`.
pub mod sun {
    /// One day's sun at a place.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Day {
        /// Rise and set in minutes after midnight UTC.
        Times { rise: f64, set: f64 },
        PolarDay,
        PolarNight,
    }

    /// The configured place (lat, lon).
    pub fn location() -> (f64, f64) {
        let fallback = (47.38, 8.54);
        let Some(text) = crate::settings::read("location") else { return fallback };
        let mut parts = text.split(',');
        match (parts.next().and_then(|v| v.trim().parse().ok()), parts.next().and_then(|v| v.trim().parse().ok())) {
            (Some(lat), Some(lon)) => (lat, lon),
            _ => fallback,
        }
    }

    /// The sun on day `day` (1 = 1 January) at (lat, lon).
    pub fn day(lat: f64, lon: f64, day: f64) -> Day {
        let rad = std::f64::consts::PI / 180.0;
        let gamma = 2.0 * std::f64::consts::PI / 365.0 * (day - 1.0);
        let eqtime = 229.18
            * (0.000075 + 0.001868 * gamma.cos() - 0.032077 * gamma.sin() - 0.014615 * (2.0 * gamma).cos() - 0.040849 * (2.0 * gamma).sin());
        let decl = 0.006918 - 0.399912 * gamma.cos() + 0.070257 * gamma.sin() - 0.006758 * (2.0 * gamma).cos()
            + 0.000907 * (2.0 * gamma).sin()
            - 0.002697 * (3.0 * gamma).cos()
            + 0.00148 * (3.0 * gamma).sin();
        let cos_ha = ((90.833 * rad).cos() - (lat * rad).sin() * decl.sin()) / ((lat * rad).cos() * decl.cos());
        if cos_ha < -1.0 {
            return Day::PolarDay;
        }
        if cos_ha > 1.0 {
            return Day::PolarNight;
        }
        let ha = cos_ha.acos() / rad;
        Day::Times { rise: 720.0 - 4.0 * (lon + ha) - eqtime, set: 720.0 - 4.0 * (lon - ha) - eqtime }
    }

    /// (day of year, minutes after midnight) in UTC, now.
    pub fn now_utc() -> (f64, f64) {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let days = (secs / 86_400) as i64;
        let minute = (secs % 86_400) as f64 / 60.0;
        // Civil date from days since 1970 (Howard Hinnant), then day of year.
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy_mar = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy_mar + 2) / 153;
        let d = doy_mar - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        const BEFORE: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        let doy = BEFORE[(m - 1) as usize] + d + i64::from(leap && m > 2);
        (doy as f64, minute)
    }

    /// Is the sun up now?
    pub fn is_up() -> bool {
        night_level() < 0.5
    }

    /// How much night it is: 0 by day, 1 by night, a smooth hour-long
    /// passage centred on sunset and sunrise (Night Shift's dial).
    pub fn night_level() -> f32 {
        let (lat, lon) = location();
        let (doy, minute) = now_utc();
        night_level_at(day(lat, lon, doy), minute)
    }

    /// `night_level` for a given day and UTC minute (testable).
    pub fn night_level_at(sun: Day, minute: f64) -> f32 {
        let (rise, set) = match sun {
            Day::PolarDay => return 0.0,
            Day::PolarNight => return 1.0,
            Day::Times { rise, set } => (rise, set),
        };
        let ramp = |edge: f64| ((minute - edge) / 60.0 + 0.5).clamp(0.0, 1.0) as f32;
        // Day: after rise (+ramp), before set (−ramp).
        let day = ramp(rise).min(1.0 - ramp(set));
        1.0 - day
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn zurich_in_november() {
            // 3 November in Zurich: rise ~06:15 UTC, set ~16:05 UTC.
            let Day::Times { rise, set } = day(47.38, 8.54, 307.0) else { panic!("no times") };
            assert!((rise - 375.0).abs() < 15.0, "rise {rise}");
            assert!((set - 965.0).abs() < 15.0, "set {set}");
            assert_eq!(night_level_at(day(47.38, 8.54, 307.0), 720.0), 0.0);
            assert_eq!(night_level_at(day(47.38, 8.54, 307.0), 22.0 * 60.0), 1.0);
            let dusk = night_level_at(day(47.38, 8.54, 307.0), set);
            assert!((dusk - 0.5).abs() < 0.01);
        }

        #[test]
        fn polar_and_clock() {
            assert_eq!(day(78.0, 15.0, 172.0), Day::PolarDay);
            assert_eq!(day(78.0, 15.0, 355.0), Day::PolarNight);
            let (doy, minute) = now_utc();
            assert!((1.0..=366.0).contains(&doy) && (0.0..1440.0).contains(&minute));
        }
    }
}

/// ENERGY (round 75): the battery as numbers people understand — how long
/// it lasts at this pace, how long until full — and whether the chip ever
/// reaches its deep sleep (the deep-sleep dig, GRABUNG-DUO-TIEFSCHLAF.md).
/// The draw is smoothed over minutes in one shared runtime file, so the
/// bar and the settings say the same.
pub mod power {
    /// Round 178 (family audit): ONE rule for «the device's battery» — four readers had three rules (the settings
    /// overview only took BAT*, so the Duo's gauge «battery» was never shown; bar and dock took the first supply with
    /// a capacity, so a Bluetooth mouse's battery could win). The gauge: «battery» (Qualcomm), else the first
    /// supply of type Battery whose scope is not «Device» (a peripheral's own battery).
    pub fn gauge() -> Option<std::path::PathBuf> {
        let root = std::path::Path::new("/sys/class/power_supply");
        let duo = root.join("battery");
        if duo.join("capacity").exists() {
            return Some(duo);
        }
        let mut found: Vec<std::path::PathBuf> = std::fs::read_dir(root)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                let read = |f: &str| std::fs::read_to_string(p.join(f)).map(|s| s.trim().to_string()).unwrap_or_default();
                read("type") == "Battery" && read("scope") != "Device" && p.join("capacity").exists()
            })
            .collect();
        found.sort();
        found.into_iter().next()
    }

    /// Smoothing time constant (s): the estimate follows a new pace in
    /// about five minutes instead of jumping with every frame.
    const TAU_S: f64 = 300.0;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Battery {
        pub percent: u32,
        pub charging: bool,
        /// Positive = into the battery (charging), negative = out.
        pub current_ma: i32,
        pub remaining_mah: Option<u32>,
        pub full_mah: Option<u32>,
    }

    fn read_i64(base: &std::path::Path, f: &str) -> Option<i64> {
        std::fs::read_to_string(base.join(f)).ok()?.trim().parse().ok()
    }

    /// The battery now (None: no battery).
    pub fn battery() -> Option<Battery> {
        let base = gauge()?;
        let read_i64 = |f: &str| read_i64(&base, f);
        let percent = read_i64("capacity")? as u32;
        let status = std::fs::read_to_string(base.join("status")).unwrap_or_default();
        Some(Battery {
            percent: percent.min(100),
            charging: matches!(status.trim(), "Charging" | "Full"),
            current_ma: read_i64("current_now").map_or(0, current_ma),
            remaining_mah: read_i64("charge_counter").map(|v| (v / 1000) as u32),
            full_mah: read_i64("charge_full").map(|v| (v / 1000) as u32),
        })
    }

    /// µA from `current_now` to mA, positive = charging. The qpnp gauge
    /// (Duo) already reports discharge as negative — round 75 first turned
    /// it around once more and so never saw a discharge.
    pub fn current_ma(ua: i64) -> i32 {
        (ua / 1000) as i32
    }

    /// Fold a new draw into the running average: (average, its time).
    pub fn fold(prev: Option<(f64, f64)>, ma: f64, now_s: f64) -> f64 {
        match prev {
            Some((avg, at)) if now_s > at && now_s - at < 3600.0 => {
                let k = 1.0 - (-(now_s - at) / TAU_S).exp();
                avg + (ma - avg) * k
            }
            _ => ma,
        }
    }

    /// The smoothed draw in mA while discharging (shared runtime file).
    pub fn smoothed_draw() -> Option<f64> {
        let b = battery()?;
        if b.charging || b.current_ma >= 0 {
            return None;
        }
        let ma = f64::from(-b.current_ma);
        let now_s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);
        let file = crate::runtime::path("matrix-power-draw");
        let prev = std::fs::read_to_string(&file).ok().and_then(|t| {
            let mut it = t.split_whitespace();
            Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
        });
        let avg = fold(prev, ma, now_s);
        let _ = std::fs::write(&file, format!("{avg:.1} {now_s:.0}"));
        Some(avg)
    }

    /// Hours left at the smoothed pace (discharging), or until full (charging).
    pub fn hours_left() -> Option<(f32, bool)> {
        let b = battery()?;
        if b.charging {
            let (rem, full) = (b.remaining_mah?, b.full_mah?);
            if b.current_ma <= 50 || rem >= full {
                return None;
            }
            return Some(((full - rem) as f32 / b.current_ma as f32, true));
        }
        let draw = smoothed_draw()?;
        if draw < 20.0 {
            return None;
        }
        Some((b.remaining_mah? as f32 / draw as f32, false))
    }

    /// «noch 3 h 40 min» / «voll in 55 min» — rounded to five minutes.
    pub fn label(hours: f32, charging: bool) -> String {
        let min = ((hours * 60.0 / 5.0).round() * 5.0).max(5.0) as u32;
        let t = if min >= 60 { if min % 60 == 0 { format!("{} h", min / 60) } else { format!("{} h {} min", min / 60, min % 60) } } else { format!("{min} min") };
        if charging { format!("voll in {t}") } else { format!("noch {t}") }
    }

    /// How often the whole chip reached its deep sleeps (AOSD, CXSD) since
    /// start — 0 means it never rests (world-readable on the Duo).
    pub fn chip_sleeps() -> Option<(u64, u64)> {
        let text = std::fs::read_to_string("/sys/power/system_sleep/stats").ok()?;
        parse_sleeps(&text)
    }

    pub fn parse_sleeps(text: &str) -> Option<(u64, u64)> {
        let mut mode = "";
        let (mut aosd, mut cxsd) = (None, None);
        for line in text.lines() {
            let l = line.trim();
            if let Some(m) = l.strip_prefix("RPM Mode:") {
                mode = m.trim();
            } else if let Some(c) = l.strip_prefix("count:") {
                let c = c.trim().parse().ok();
                match mode {
                    "aosd" => aosd = c,
                    "cxsd" => cxsd = c,
                    _ => {}
                }
            }
        }
        Some((aosd?, cxsd?))
    }

    // Round 78: DEEP SLEEP (suspend-to-RAM) — the setting the sleep service
    // (duo-schlaf on the Duo) obeys, and what it last did. The service reads
    // the same two files: ~/.config/matrix/deep-sleep and deep-sleep-delay.

    /// Does this machine have a sleep service the settings can steer?
    pub fn has_deep_sleep() -> bool {
        std::path::Path::new("/etc/systemd/system/duo-schlaf.service").exists()
    }

    /// Deep sleep allowed (default on; the service waits for every reason
    /// not to sleep: sound, cable, folded, an SSH session).
    pub fn deep_sleep() -> bool {
        // the hot flag (500 ms): settings page and control centre ask while drawing
        !crate::settings::flag("deep-sleep", &["off", "aus", "0", "false"])
    }

    pub fn set_deep_sleep(on: bool) {
        crate::settings::write("deep-sleep", if on { "on" } else { "off" });
    }

    /// How long the panels stay dark before the machine sleeps.
    pub const SLEEP_DELAYS: [(u32, &str); 4] = [(30, "30 s"), (60, "1 min"), (120, "2 min"), (300, "5 min")];

    pub fn deep_sleep_delay() -> u32 {
        crate::settings::read("deep-sleep-delay").and_then(|v| v.parse().ok()).map_or(60, |v: u32| v.max(20))
    }

    pub fn set_deep_sleep_delay(secs: u32) {
        crate::settings::write("deep-sleep-delay", &secs.max(20).to_string());
    }

    /// What the service did today: (last sleep "HH:MM", its seconds,
    /// sleeps today, seconds asleep today). None = no log.
    /// Round 80: kept 5 s — the settings page asks every frame (the R209
    /// lesson: a file read while drawing costs hundreds of opens a second).
    pub fn sleep_today() -> Option<SleepLog> {
        use std::sync::Mutex;
        use std::time::{Duration, Instant};
        static CACHE: Mutex<Option<(Instant, Option<SleepLog>)>> = Mutex::new(None);
        let mut c = CACHE.lock().ok()?;
        if let Some((at, log)) = c.as_ref() {
            if at.elapsed() < Duration::from_secs(5) {
                return log.clone();
            }
        }
        let log = sleep_today_fresh();
        *c = Some((Instant::now(), log.clone()));
        log
    }

    fn sleep_today_fresh() -> Option<SleepLog> {
        let text = std::fs::read_to_string("/var/lib/duo/schlaf.log").ok()?;
        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        unsafe { libc::localtime_r(&now, &mut tm) };
        Some(parse_sleep_log(&text, &format!("{:02}.{:02}.", tm.tm_mday, tm.tm_mon + 1)))
    }

    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct SleepLog {
        pub last: Option<(String, u32)>,
        pub count: u32,
        pub secs: u32,
        /// Round 79: mean draw while asleep (mA), from the charge counter
        /// before and after — only over sleeps long enough to resolve
        /// (the gauge counts whole mAh: ≥ 20 min together).
        pub sleep_ma: Option<f32>,
    }

    /// Smallest sleep time (s) a draw is computed over.
    const DRAW_MIN_S: u32 = 1200;

    /// Lines: "04.11. 12:15:11 GESCHLAFEN 128 s | …" (aborted attempts
    /// say "abgebrochen" and do not count).
    pub fn parse_sleep_log(text: &str, today: &str) -> SleepLog {
        let mut out = SleepLog::default();
        let (mut used_uah, mut used_s) = (0i64, 0u32);
        for line in text.lines() {
            // "akku=1164000>1160000" (µAh before > after), written since round 79
            if let Some(a) = line.split_whitespace().find_map(|w| w.strip_prefix("akku=")) {
                if let Some((b, e)) = a.split_once('>') {
                    if let (Ok(b), Ok(e), Some(s)) = (b.parse::<i64>(), e.parse::<i64>(), line.split_whitespace().nth(3).and_then(|s| s.parse::<u32>().ok())) {
                        if b >= e && line.contains("GESCHLAFEN") {
                            used_uah += b - e;
                            used_s += s;
                        }
                    }
                }
            }
            let mut w = line.split_whitespace();
            let (Some(day), Some(time), Some(kind), Some(secs)) = (w.next(), w.next(), w.next(), w.next()) else { continue };
            if kind != "GESCHLAFEN" {
                continue;
            }
            let Ok(secs) = secs.parse::<u32>() else { continue };
            if day == today {
                out.count += 1;
                out.secs += secs;
            }
            out.last = Some((format!("{}{}", if day == today { "" } else { day }, if day == today { time.get(..5).unwrap_or(time).to_string() } else { format!(" {}", time.get(..5).unwrap_or(time)) }), secs));
        }
        if used_s >= DRAW_MIN_S {
            out.sleep_ma = Some(used_uah as f32 / 1000.0 / (used_s as f32 / 3600.0));
        }
        out
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn sleep_log() {
            let log = "04.11. 11:59:00 abgebrochen 0 s | x\n03.11. 23:00:00 GESCHLAFEN 600 s | \n04.11. 12:15:11 GESCHLAFEN 128 s | aosd=0\n04.11. 13:00:00 GESCHLAFEN 72 s | \n";
            let l = parse_sleep_log(log, "04.11.");
            assert_eq!(l, SleepLog { last: Some(("13:00".into(), 72)), count: 2, secs: 200, sleep_ma: None });
            let y = parse_sleep_log("03.11. 23:00:00 GESCHLAFEN 600 s |", "04.11.");
            assert_eq!(y.last, Some(("03.11. 23:00".into(), 600)));
            assert_eq!(y.count, 0);
            assert_eq!(y.sleep_ma, None);
            // 3600 s from 1164 to 1149 mAh = 15 mA; a short sleep alone says nothing
            let d = parse_sleep_log("04.11. 22:00:00 GESCHLAFEN 3600 s | akku=1164000>1149000 |\n04.11. 23:10:00 GESCHLAFEN 60 s | akku=1149000>1149000 |", "04.11.");
            let ma = d.sleep_ma.unwrap();
            assert!((ma - 14.75).abs() < 0.1, "{ma}");
            assert_eq!(parse_sleep_log("04.11. 22:00:00 GESCHLAFEN 600 s | akku=1164000>1163000 |", "04.11.").sleep_ma, None);
        }

        #[test]
        fn labels_and_smoothing() {
            assert_eq!(label(3.67, false), "noch 3 h 40 min");
            assert_eq!(label(0.9, true), "voll in 55 min");
            assert_eq!(label(2.0, false), "noch 2 h");
            // Duo, discharging: current_now = -473144 µA.
            assert_eq!(current_ma(-473_144), -473);
            assert!(current_ma(800_000) > 0);
            assert_eq!(fold(None, 500.0, 100.0), 500.0);
            let a = fold(Some((500.0, 0.0)), 200.0, 300.0);
            assert!(a < 400.0 && a > 250.0, "{a}");
            assert_eq!(fold(Some((500.0, 0.0)), 200.0, 99_999.0), 200.0);
        }

        #[test]
        fn sleep_stats_parse() {
            let t = "RPM Mode:aosd\n\t count:3\ntime in last mode(msec):0\n\nRPM Mode:cxsd\n\t count:12\n\nRPM Mode:ddr \n\t count:0\n";
            assert_eq!(parse_sleeps(t), Some((3, 12)));
        }
    }
}

/// THE DISPLAY (round 72): automatic brightness and the measured light —
/// on the Duo duo-haltung does the work (light sensor, curve, learning);
/// this is the one place apps ask and switch.
/// Round 78: WAKE — did the machine just come back from suspend?
/// CLOCK_MONOTONIC (`Instant`) stands still in suspend, the boot clock
/// (/proc/uptime) runs on: a jump in their difference is a sleep. Every
/// app that polls hardware can ask (sounds: devices re-announce
/// themselves; a battery estimate: the pace over the gap is not a draw).
pub mod wake {
    use std::time::{Duration, Instant};

    /// Boot clock minus monotonic clock, in seconds (grows only in suspend).
    pub fn gap(base: Instant) -> Option<f64> {
        let up = std::fs::read_to_string("/proc/uptime").ok()?;
        let up: f64 = up.split_whitespace().next()?.parse().ok()?;
        Some(up - base.elapsed().as_secs_f64())
    }

    /// Poll it now and then (every second is plenty).
    pub struct Watch {
        base: Instant,
        last: Option<f64>,
        woke: Option<Instant>,
    }

    impl Default for Watch {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Watch {
        pub fn new() -> Self {
            let base = Instant::now();
            Self { base, last: gap(base), woke: None }
        }

        /// Seconds slept since the last check (sleeps under 2 s are clock noise).
        pub fn check(&mut self) -> Option<f64> {
            let now = gap(self.base)?;
            let slept = self.last.map(|l| now - l);
            self.last = Some(now);
            let slept = slept.filter(|s| *s > 2.0)?;
            self.woke = Some(Instant::now());
            Some(slept)
        }

        /// Woke within the last `d` (checks first).
        pub fn woke_within(&mut self, d: Duration) -> bool {
            self.check();
            self.woke.is_some_and(|t| t.elapsed() < d)
        }
    }

    /// A shared watch on its own thread: the instant of the last wake.
    pub fn spawn() -> std::sync::Arc<std::sync::Mutex<Option<Instant>>> {
        let woke = std::sync::Arc::new(std::sync::Mutex::new(None));
        let out = woke.clone();
        std::thread::Builder::new()
            .name("mk-wake".into())
            .spawn(move || {
                let mut w = Watch::new();
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    if w.check().is_some() {
                        if let Ok(mut t) = woke.lock() {
                            *t = Some(Instant::now());
                        }
                    }
                }
            })
            .ok();
        out
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn no_sleep_no_wake() {
            let mut w = super::Watch::new();
            assert_eq!(w.check(), None);
            assert!(!w.woke_within(std::time::Duration::from_secs(5)));
        }
    }
}

/// Round 102: how screen films are made (matrixkit_video::film reads the
/// choice through its caller; the settings show it under Ton › Ausgabe).
pub mod film {
    /// Record what the device plays along with the picture (default on).
    pub fn sound() -> bool {
        !crate::settings::flag("film-sound", &["off"])
    }

    pub fn set_sound(on: bool) {
        crate::settings::write("film-sound", if on { "on" } else { "off" });
    }
}

pub mod display {
    /// The light sensor's last value (duo-haltung publishes it; the sensor
    /// only reports changes, so it may be older).
    pub fn ambient_lux() -> Option<f32> {
        std::fs::read_to_string("/run/duo/lux").ok()?.trim().parse().ok()
    }

    /// Does this machine have automatic brightness (a light sensor duo-haltung reads)?
    /// Round 85: folded back, the duo's back panel goes FULLY off (its CRTC
    /// inactive, niri-canvas 0017) instead of only dark — an OLED at
    /// brightness 0 still shows a faint picture (owner, 04.11.2026).
    /// duo-haltung reads it at the next fold.
    pub fn back_panel_off() -> bool {
        crate::settings::flag("back-panel-off", &["on"])
    }

    pub fn set_back_panel_off(on: bool) {
        crate::settings::write("back-panel-off", if on { "on" } else { "off" });
    }

    /// THE SCREEN ON USB-C (round 158): what the duo does with it. Read by
    /// duo-haltung (`desk-closed`), mk::device::desk_screen (`desk-folded`)
    /// and duo-bildton (resolution, scale, sound). All «on» unless «off».
    pub fn desk_closed() -> bool {
        crate::settings::read("desk-closed").map_or(true, |v| v.trim() != "off")
    }

    pub fn set_desk_closed(on: bool) {
        crate::settings::write("desk-closed", if on { "on" } else { "off" });
    }

    pub fn desk_folded() -> bool {
        crate::settings::read("desk-folded").map_or(true, |v| v.trim() != "off")
    }

    pub fn set_desk_folded(on: bool) {
        crate::settings::write("desk-folded", if on { "on" } else { "off" });
    }

    pub fn desk_sound() -> bool {
        crate::settings::read("desk-sound").map_or(true, |v| v.trim() != "off")
    }

    pub fn set_desk_sound(on: bool) {
        crate::settings::write("desk-sound", if on { "on" } else { "off" });
    }

    /// (value, words): 4K runs at 30 Hz only — two of the four USB-C lanes
    /// carry USB 3.
    pub const DESK_RESOLUTIONS: [(&str, &str); 2] = [("1080p", "1080p · 60 Hz"), ("4k", "4K · 30 Hz")];
    pub const DESK_SCALES: [(&str, &str); 3] = [("1", "Klein"), ("1.5", "Standard"), ("2", "Gross")];

    /// The chosen entry of `table` under `key` (index; `default` if unset).
    pub fn desk_choice(key: &str, table: &[(&str, &str)], default: usize) -> usize {
        crate::settings::read(key).and_then(|v| table.iter().position(|(k, _)| *k == v.trim())).unwrap_or(default)
    }

    pub fn set_desk_choice(key: &str, table: &[(&str, &str)], index: usize) {
        if let Some((value, _)) = table.get(index) {
            crate::settings::write(key, value);
        }
    }

    pub fn has_auto_brightness() -> bool {
        crate::device::panels().is_some()
    }

    pub fn auto_brightness() -> bool {
        crate::settings::read("auto-brightness").as_deref().map(str::trim) == Some("on")
    }

    pub fn set_auto_brightness(on: bool) {
        crate::settings::write("auto-brightness", if on { "on" } else { "off" });
    }

    /// Round 73: the screen orientations duo-haltung knows (two panels):
    /// (key, name). «normal» = the book upright, «left-up» = panel A on top.
    pub const ORIENTATIONS: [(&str, &str); 4] = [("normal", "Buch"), ("left-up", "A oben"), ("right-up", "B oben"), ("bottom-up", "Kopfüber")];

    /// The orientation now (duo-haltung keeps it in /var/lib/duo/ausrichtung).
    pub fn orientation() -> Option<String> {
        let o = std::fs::read_to_string("/var/lib/duo/ausrichtung").ok()?.trim().to_string();
        ORIENTATIONS.iter().any(|(k, _)| *k == o).then_some(o)
    }

    /// Does the screen turn with the device?
    pub fn auto_rotation() -> bool {
        crate::settings::read("rotation-lock").as_deref().map(str::trim) != Some("on")
    }

    /// Turning on frees the sensor; turning off holds the orientation now.
    pub fn set_auto_rotation(on: bool) {
        if !on {
            if let Some(o) = orientation() {
                crate::settings::write("rotation", &o);
            }
        }
        crate::settings::write("rotation-lock", if on { "off" } else { "on" });
    }

    /// A fixed orientation (locks the rotation; duo-haltung turns at once).
    pub fn set_orientation(key: &str) {
        crate::settings::write("rotation-lock", "on");
        crate::settings::write("rotation", key);
    }
}

/// NIGHT SHIFT (round 71, the owner: «I'm missing … Night Shift»): warmer
/// colours in the evening. Settings `night-shift` (off | auto | on) and
/// `night-shift-warmth` (0..100); `matrix-nightshift` holds the gamma
/// ramps with niri (wlr-gamma-control) and follows these values.
pub mod night {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Mode {
        Off,
        /// From sunset to sunrise (mk::sun), with an hour-long passage.
        Auto,
        On,
        /// Round 72: own times (`night-shift-from`/`-to`, minutes after
        /// local midnight; default 22:00–07:00), same soft passage.
        Schedule,
    }

    pub fn mode() -> Mode {
        match crate::settings::read("night-shift").as_deref().map(str::trim) {
            Some("on") => Mode::On,
            Some("auto") => Mode::Auto,
            Some("schedule") => Mode::Schedule,
            _ => Mode::Off,
        }
    }

    pub fn set_mode(m: Mode) {
        crate::settings::write("night-shift", match m {
            Mode::Off => "off",
            Mode::Auto => "auto",
            Mode::On => "on",
            Mode::Schedule => "schedule",
        });
    }

    /// 0..1 — how warm the night colour is (default 0.5).
    pub fn warmth() -> f32 {
        crate::settings::read("night-shift-warmth").and_then(|v| v.trim().parse::<f32>().ok()).map_or(0.5, |v| (v / 100.0).clamp(0.0, 1.0))
    }

    pub fn set_warmth(w: f32) {
        crate::settings::write("night-shift-warmth", &format!("{:.0}", w.clamp(0.0, 1.0) * 100.0));
    }

    /// Daylight and the warmest night (Kelvin).
    pub const DAY_K: f32 = 6500.0;
    pub const WARMEST_K: f32 = 2700.0;
    pub const MILDEST_K: f32 = 5500.0;

    /// The colour temperature for a mode, warmth and night level (0..1).
    pub fn kelvin(mode: Mode, warmth: f32, night: f32) -> f32 {
        // Warmth 0 = a mild 5500 K, warmth 1 = the warmest 2700 K.
        let target = MILDEST_K - warmth.clamp(0.0, 1.0) * (MILDEST_K - WARMEST_K);
        let amount = match mode {
            Mode::Off => 0.0,
            Mode::On => 1.0,
            Mode::Auto | Mode::Schedule => night.clamp(0.0, 1.0),
        };
        DAY_K - (DAY_K - target) * amount
    }

    /// The schedule (from, to) in minutes after local midnight.
    pub fn schedule() -> (u32, u32) {
        let read = |key: &str, fallback: u32| crate::settings::read(key).and_then(|v| v.trim().parse::<u32>().ok()).map_or(fallback, |m| m % 1440);
        (read("night-shift-from", 22 * 60), read("night-shift-to", 7 * 60))
    }

    pub fn set_schedule(from: u32, to: u32) {
        crate::settings::write("night-shift-from", &(from % 1440).to_string());
        crate::settings::write("night-shift-to", &(to % 1440).to_string());
    }

    /// Night by the schedule at `minute` (local): 0..1 with an hour-long
    /// passage centred on both edges (like the sun's).
    pub fn schedule_level(from: u32, to: u32, minute: f64) -> f32 {
        // Distance (signed, wrapped to ±12 h) of `minute` past an edge.
        let past = |edge: u32| {
            let d = (minute - f64::from(edge)).rem_euclid(1440.0);
            if d > 720.0 { d - 1440.0 } else { d }
        };
        let ramp = |edge: u32| ((past(edge) / 60.0) + 0.5).clamp(0.0, 1.0) as f32;
        // Inside the window [from, to) (wrapping midnight)?
        let span = (f64::from(to) - f64::from(from)).rem_euclid(1440.0);
        let inside = (minute - f64::from(from)).rem_euclid(1440.0) < span;
        // Inside 1, outside 0 — and within half an hour of an edge, its ramp.
        if past(from).abs() < 30.0 {
            ramp(from)
        } else if past(to).abs() < 30.0 {
            1.0 - ramp(to)
        } else if inside {
            1.0
        } else {
            0.0
        }
    }

    /// Minutes after LOCAL midnight now (the UTC clock plus the zone's
    /// offset from `localtime_r`).
    pub fn local_minute() -> f64 {
        let now = unsafe { libc::time(std::ptr::null_mut()) };
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        unsafe { libc::localtime_r(&now, &mut tm) };
        f64::from(tm.tm_hour * 60 + tm.tm_min) + f64::from(tm.tm_sec) / 60.0
    }

    /// How much night it is for a mode (Auto: the sun, Schedule: the times).
    pub fn level(m: Mode) -> f32 {
        match m {
            Mode::Off => 0.0,
            Mode::On => 1.0,
            Mode::Auto => crate::sun::night_level(),
            Mode::Schedule => {
                let (from, to) = schedule();
                schedule_level(from, to, local_minute())
            }
        }
    }

    /// The colour temperature right now.
    pub fn kelvin_now() -> f32 {
        let m = mode();
        kelvin(m, warmth(), level(m))
    }

    /// Red, green, blue gains (0..1) of a black body at `k` Kelvin,
    /// normalised to white at 6500 K (Tanner Helland's fit).
    pub fn gains(k: f32) -> [f32; 3] {
        if k >= DAY_K - 1.0 {
            return [1.0, 1.0, 1.0];
        }
        let t = k / 100.0;
        let r = if t <= 66.0 { 1.0 } else { (329.698_73 * (t - 60.0).powf(-0.133_204_76) / 255.0).clamp(0.0, 1.0) };
        let g = if t <= 66.0 { ((99.470_8 * t.ln() - 161.119_57) / 255.0).clamp(0.0, 1.0) } else { (288.122_16 * (t - 60.0).powf(-0.075_514_85) / 255.0).clamp(0.0, 1.0) };
        let b = if t >= 66.0 { 1.0 } else if t <= 19.0 { 0.0 } else { ((138.517_73 * (t - 10.0).ln() - 305.044_8) / 255.0).clamp(0.0, 1.0) };
        // White at 6500 K is not exactly (1,1,1) in the fit — normalise.
        let w = {
            let t = 65.0f32;
            let g = (99.470_8 * t.ln() - 161.119_57) / 255.0;
            let b = (138.517_73 * (t - 10.0).ln() - 305.044_8) / 255.0;
            [1.0f32, g.clamp(0.01, 1.0), b.clamp(0.01, 1.0)]
        };
        [(r / w[0]).min(1.0), (g / w[1]).min(1.0), (b / w[2]).min(1.0)]
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn warmer_means_less_blue() {
            assert_eq!(gains(6500.0), [1.0, 1.0, 1.0]);
            let warm = gains(3000.0);
            assert!(warm[0] > 0.99 && warm[2] < warm[1] && warm[2] < 0.6, "{warm:?}");
            assert_eq!(kelvin(Mode::Off, 1.0, 1.0), DAY_K);
            assert!(kelvin(Mode::On, 1.0, 0.0) < kelvin(Mode::On, 0.0, 0.0));
            assert_eq!(kelvin(Mode::Auto, 0.5, 0.0), DAY_K);
            assert!(kelvin(Mode::Auto, 0.5, 1.0) < 5000.0);
        }

        #[test]
        fn the_schedule_wraps_midnight_softly() {
            let (from, to) = (22 * 60, 7 * 60);
            assert_eq!(schedule_level(from, to, 12.0 * 60.0), 0.0);
            assert_eq!(schedule_level(from, to, 2.0 * 60.0), 1.0);
            assert!((schedule_level(from, to, 22.0 * 60.0) - 0.5).abs() < 0.01);
            assert!((schedule_level(from, to, 7.0 * 60.0) - 0.5).abs() < 0.01);
            assert!(schedule_level(from, to, 21.0 * 60.0 + 45.0) > 0.0 && schedule_level(from, to, 21.0 * 60.0 + 45.0) < 0.5);
            assert_eq!(schedule_level(from, to, 23.0 * 60.0), 1.0);
            // A day window (no midnight) works too.
            assert_eq!(schedule_level(9 * 60, 17 * 60, 12.0 * 60.0), 1.0);
            assert_eq!(schedule_level(9 * 60, 17 * 60, 20.0 * 60.0), 0.0);
        }
    }
}

/// A NOTIFICATION (round 64): the freedesktop way (notify-send → the
/// notification daemon), reaped. The watchdog had its own copy; capture
/// needed one to say why a recording cannot start.
pub mod notify {
    /// The hint that carries what a tap opens (round 109): the notification
    /// daemon opens it itself (`mk::open`) — no app has to listen for the
    /// ActionInvoked signal.
    pub const OPEN_HINT: &str = "x-matrix-open";

    pub fn send(urgent: bool, title: &str, text: &str) {
        let mut n = Note::new(title).text(text);
        if urgent {
            n = n.urgent();
        }
        n.send();
    }

    /// ROUND 109: a notification built up — who sends it (the island shows
    /// that app's icon), how urgent, and what a tap opens.
    ///
    /// ```ignore
    /// mk::notify::Note::new("Bildschirmaufnahme gespeichert")
    ///     .app("Matrix Player").text("2:03 · mit Ton").opens(&path).send();
    /// ```
    #[derive(Debug, Clone, PartialEq)]
    pub struct Note {
        pub app: String,
        pub title: String,
        pub text: String,
        pub urgent: bool,
        pub open: Option<String>,
    }

    impl Note {
        pub fn new(title: &str) -> Note {
            Note { app: String::from("Matrix"), title: title.to_string(), text: String::new(), urgent: false, open: None }
        }

        pub fn app(mut self, app: &str) -> Note {
            self.app = app.to_string();
            self
        }

        pub fn text(mut self, text: &str) -> Note {
            self.text = text.to_string();
            self
        }

        pub fn urgent(mut self) -> Note {
            self.urgent = true;
            self
        }

        /// A file or link a tap opens (see `mk::open`).
        pub fn opens(mut self, target: impl AsRef<std::ffi::OsStr>) -> Note {
            self.open = Some(target.as_ref().to_string_lossy().into_owned());
            self
        }

        /// The notify-send arguments.
        pub fn args(&self) -> Vec<String> {
            let mut a = vec![String::from("-a"), self.app.clone()];
            if self.urgent {
                a.extend([String::from("-u"), String::from("critical")]);
            }
            if let Some(o) = &self.open {
                a.push(format!("--hint=string:{OPEN_HINT}:{o}"));
            }
            a.push(self.title.clone());
            if !self.text.is_empty() {
                a.push(self.text.clone());
            }
            a
        }

        pub fn send(&self) -> bool {
            crate::process::spawn_reaped(std::process::Command::new("notify-send").args(self.args()))
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn a_note_carries_app_urgency_and_target() {
            let n = super::Note::new("Gespeichert").app("Matrix Player").text("2:03").urgent().opens("/home/a b.mp4");
            assert_eq!(
                n.args(),
                ["-a", "Matrix Player", "-u", "critical", "--hint=string:x-matrix-open:/home/a b.mp4", "Gespeichert", "2:03"]
            );
            assert_eq!(super::Note::new("Nur Titel").args(), ["-a", "Matrix", "Nur Titel"]);
        }
    }
}

/// ROUND 109: OPEN a file or a link the Matrix way — videos in the Player,
/// Markdown in Markdown, everything else through the system (gio, then
/// xdg-open). Before, apps called `xdg-open` by hand — the Duo has none:
/// Dateien could open no video there, and nothing said so.
pub mod open {
    /// The Matrix app for `target` (None = the system decides).
    pub fn handler(target: &str) -> Option<&'static str> {
        if target.starts_with("http://") || target.starts_with("https://") {
            return None;
        }
        let ext = std::path::Path::new(target).extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        match ext.as_str() {
            "mp4" | "webm" | "mkv" | "mov" | "m4v" | "avi" => Some("matrix-player"),
            "md" | "markdown" => Some("matrix-markdown"),
            _ => None,
        }
    }

    /// Is `program` there (~/.local/bin or PATH)?
    pub fn installed(program: &str) -> bool {
        let home = std::env::var("HOME").unwrap_or_default();
        std::iter::once(std::path::PathBuf::from(format!("{home}/.local/bin")))
            .chain(std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect::<Vec<_>>()).unwrap_or_default())
            .any(|d| d.join(program).is_file())
    }

    /// Open it; false when nothing could.
    pub fn open(target: &str) -> bool {
        if let Some(app) = handler(target).filter(|a| installed(a)) {
            return crate::process::app_start(app, &[target]);
        }
        if installed("gio") && crate::process::spawn_reaped(std::process::Command::new("gio").args(["open", target])) {
            return true;
        }
        installed("xdg-open") && crate::process::spawn_reaped(std::process::Command::new("xdg-open").arg(target))
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn videos_go_to_the_player() {
            assert_eq!(super::handler("/x/Bildschirmaufnahme.MP4"), Some("matrix-player"));
            assert_eq!(super::handler("notiz.md"), Some("matrix-markdown"));
            assert_eq!(super::handler("https://matrixhardline.ch/a.mp4"), None);
            assert_eq!(super::handler("bild.png"), None);
        }
    }
}

/// ROUND 120: THE FEEL — one vocabulary for vibration. Before, 59 calls
/// across 17 files each chose a number (8, 10, 12, 15, 20, 25, 45, 60 ms);
/// now a few patterns, each with its own meaning, the same everywhere.
/// The Duo's motor (duo-haptik) takes «ms level»; a pattern is a row of
/// pulses with pauses, played off the caller's thread.
pub mod feel {
    /// What a vibration says.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Feel {
        /// A key typed — barely there.
        Tick,
        /// A button pressed.
        Tap,
        /// Something chosen, toggled, pinned, moved.
        Select,
        /// The first stage of a long press (a menu, a second action).
        Hold,
        /// The second stage — the wiggle mode: a double, the second firmer.
        Stage,
        /// Done (copied, signed in, saved): two soft pulses.
        Confirm,
        /// Something failed (a wrong password): two firm pulses.
        Warn,
    }

    /// One pulse: on for `ms`, at `strength` (0..1), then `pause` ms quiet.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Pulse {
        pub ms: u32,
        pub strength: f32,
        pub pause: u32,
    }

    const fn p(ms: u32, strength: f32, pause: u32) -> Pulse {
        Pulse { ms, strength, pause }
    }

    const TICK: [Pulse; 1] = [p(8, 0.5, 0)];
    const TAP: [Pulse; 1] = [p(12, 0.62, 0)];
    const SELECT: [Pulse; 1] = [p(15, 0.75, 0)];
    const HOLD: [Pulse; 1] = [p(25, 0.85, 0)];
    const STAGE: [Pulse; 2] = [p(20, 0.8, 70), p(32, 1.0, 0)];
    const CONFIRM: [Pulse; 2] = [p(12, 0.6, 90), p(14, 0.7, 0)];
    const WARN: [Pulse; 2] = [p(40, 1.0, 70), p(40, 1.0, 0)];

    impl Feel {
        pub fn pattern(self) -> &'static [Pulse] {
            match self {
                Feel::Tick => &TICK,
                Feel::Tap => &TAP,
                Feel::Select => &SELECT,
                Feel::Hold => &HOLD,
                Feel::Stage => &STAGE,
                Feel::Confirm => &CONFIRM,
                Feel::Warn => &WARN,
            }
        }
    }

    /// The motor's level for a strength (duo-haptik takes 1..=32767).
    pub fn level(strength: f32) -> i32 {
        (strength.clamp(0.05, 1.0) * 32767.0).round() as i32
    }

    /// Vibration on (Einstellungen › Bedienung › Vibration, default on).
    pub fn enabled() -> bool {
        !crate::settings::flag("haptics", &["off"])
    }

    pub fn set_enabled(on: bool) {
        crate::settings::write("haptics", if on { "on" } else { "off" });
    }

    /// Play a pattern (a single pulse at once, a row on a thread of its own).
    pub fn play(f: Feel) {
        if !crate::device::is_duo() || !enabled() {
            return;
        }
        let send = |pulse: &Pulse| {
            if let Ok(socket) = std::os::unix::net::UnixDatagram::unbound() {
                let _ = socket.send_to(format!("{} {}", pulse.ms, level(pulse.strength)).as_bytes(), "/run/duo/haptik.sock");
            }
        };
        let pattern = f.pattern();
        if pattern.len() == 1 {
            send(&pattern[0]);
            return;
        }
        std::thread::spawn(move || {
            for pulse in pattern {
                send(pulse);
                std::thread::sleep(std::time::Duration::from_millis((pulse.ms + pulse.pause) as u64));
            }
        });
    }

    pub fn tick() {
        play(Feel::Tick)
    }
    pub fn tap() {
        play(Feel::Tap)
    }
    pub fn select() {
        play(Feel::Select)
    }
    pub fn hold() {
        play(Feel::Hold)
    }
    pub fn stage() {
        play(Feel::Stage)
    }
    pub fn confirm() {
        play(Feel::Confirm)
    }
    pub fn warn() {
        play(Feel::Warn)
    }

    #[cfg(test)]
    mod tests {
        use super::Feel;

        #[test]
        fn every_feel_has_a_sane_pattern() {
            for f in [Feel::Tick, Feel::Tap, Feel::Select, Feel::Hold, Feel::Stage, Feel::Confirm, Feel::Warn] {
                let pat = f.pattern();
                assert!(!pat.is_empty());
                for p in pat {
                    assert!((1..=100).contains(&p.ms));
                    assert!((1..=32767).contains(&super::level(p.strength)));
                }
                // the last pulse needs no pause after it
                assert_eq!(pat.last().map(|p| p.pause), Some(0));
            }
            // the stages feel different: one pulse vs. a double
            assert_eq!(Feel::Hold.pattern().len(), 1);
            assert_eq!(Feel::Stage.pattern().len(), 2);
        }
    }
}

/// ROUND 113: EDITING THE BARS — the owner: a long press with a SECOND
/// vibration puts the dock and the top bar into a wiggle mode: elements
/// wiggle, «–» removes, «+» adds, dragging reorders (each bar keeps its
/// own elements). One shared state on the runtime board, so both bars
/// enter and leave together; the helpers below are the mode's grammar.
pub mod edit {
    const FLAG: &str = "matrix-edit";

    /// How long a finger rests for the SECOND stage (the first, the
    /// context menu, comes at `touch::LONG_PRESS_SECS`).
    pub const PRESS_SECS: f32 = 1.2;

    /// Both bars go into the wiggle mode.
    pub fn enter() {
        crate::runtime::write(FLAG, "on");
    }

    /// Both bars leave it.
    pub fn leave() {
        crate::runtime::write(FLAG, "off");
    }

    pub fn active() -> bool {
        crate::runtime::read(FLAG).is_some_and(|t| t.trim() == "on")
    }

    /// The wiggle of element `i` at `t` seconds: (dx, dy) in logical points
    /// — two slightly different rhythms per element, so the row lives
    /// rather than marching in step.
    pub fn wiggle(t: f32, i: usize) -> (f32, f32) {
        let phase = i as f32 * 1.7;
        let tau = std::f32::consts::TAU;
        ((tau * 2.6 * t + phase).sin() * 1.4, (tau * 3.1 * t + phase * 1.3).sin() * 1.0)
    }

    /// Round 165: a WINDOW that can be moved wiggles in the same rhythm as
    /// the dock's and the bar's chips (one wiggle language), a little wider
    /// — a whole window needs more to be seen trembling.
    pub fn wiggle_window(t: f32) -> (f32, f32) {
        let (dx, dy) = wiggle(t, 0);
        (dx * 1.6, dy * 1.6)
    }

    #[cfg(test)]
    mod wiggle_tests {
        #[test]
        fn a_window_wiggles_like_the_chips_only_wider() {
            for k in 0..120 {
                let t = k as f32 * 0.011;
                let (cx, cy) = super::wiggle(t, 0);
                let (wx, wy) = super::wiggle_window(t);
                assert!((wx - cx * 1.6).abs() < 1e-4 && (wy - cy * 1.6).abs() < 1e-4);
                assert!(wx.abs() <= 2.3 && wy.abs() <= 1.7);
            }
        }
    }

    /// Where an element dragged to `x` lands among elements whose centres
    /// are `centres` (in order): the number of centres left of it.
    pub fn slot_at(centres: &[f32], x: f32) -> usize {
        centres.iter().filter(|c| **c < x).count()
    }

    /// Move `list[from]` to position `to` (counted after its removal is
    /// undone: «in front of the element now at `to`»).
    pub fn move_item<T>(list: &mut Vec<T>, from: usize, to: usize) {
        if from >= list.len() {
            return;
        }
        let item = list.remove(from);
        let to = if to > from { to - 1 } else { to };
        list.insert(to.min(list.len()), item);
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn dragging_reorders_like_a_finger_expects() {
            let mut l = vec!['a', 'b', 'c', 'd'];
            // 'a' dragged past 'c' (its centre right of b and c): lands after c
            let centres = [10.0, 30.0, 50.0, 70.0];
            let to = super::slot_at(&centres, 55.0);
            super::move_item(&mut l, 0, to);
            assert_eq!(l, ['b', 'c', 'a', 'd']);
            // 'd' dragged to the very front
            super::move_item(&mut l, 3, super::slot_at(&centres, 2.0));
            assert_eq!(l, ['d', 'b', 'c', 'a']);
            // dropped where it was: unchanged
            super::move_item(&mut l, 1, super::slot_at(&centres, 31.0));
            assert_eq!(l, ['d', 'b', 'c', 'a']);
        }

        #[test]
        fn the_wiggle_stays_small() {
            for i in 0..8 {
                for k in 0..50 {
                    let (dx, dy) = super::wiggle(k as f32 * 0.037, i);
                    assert!(dx.abs() <= 1.4 && dy.abs() <= 1.0);
                }
            }
        }
    }
}

/// LOGIN SHARE (round 166, 05.11.2026): what the login screen may know of a
/// person. greetd runs the greeter as its own account, and /home/<user> is
/// closed to it (700) — it showed a violet night instead of the owner's
/// wallpaper and a plain clock instead of his lock clock. The session (the
/// wallpaper service) publishes to `DIR`: the wallpaper (a copy), the lock
/// clock's look and the palette's seed — nothing else (no keys, no files of
/// the person). The greeter reads `read(user)`.
pub mod login_share {
    use std::path::{Path, PathBuf};

    pub const DIR: &str = "/var/lib/matrix/anmeldung";
    /// The settings that travel (the lock clock's look).
    pub const KEYS: [&str; 5] = ["lock-clock-stretch", "lock-clock-weight", "lock-clock-width", "lock-clock-body", "lock-clock-glass"];

    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Shared {
        /// The copied wallpaper.
        pub wall: Option<PathBuf>,
        /// The palette's seed colour (the person's tint).
        pub seed: Option<crate::Rgba>,
        pairs: Vec<(String, String)>,
    }

    impl Shared {
        /// A shared setting (one of `KEYS`).
        pub fn get(&self, key: &str) -> Option<String> {
            self.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
        }
    }

    /// «key=value» lines (pure — tested).
    pub fn parse(text: &str) -> Vec<(String, String)> {
        text.lines().filter_map(|l| l.split_once('=')).map(|(k, v)| (k.trim().to_string(), v.trim().to_string())).filter(|(k, _)| !k.is_empty()).collect()
    }

    fn file(user: &str, ext: &str) -> PathBuf {
        Path::new(DIR).join(format!("{user}.{ext}"))
    }

    /// The greeter's side: what `user` shared (empty when nothing).
    pub fn read(user: &str) -> Shared {
        let pairs = std::fs::read_to_string(file(user, "look")).map(|t| parse(&t)).unwrap_or_default();
        let wall = Some(file(user, "wand")).filter(|p| p.is_file());
        let seed = pairs.iter().find(|(k, _)| k == "seed").and_then(|(_, v)| crate::Rgba::from_hex(v));
        Shared { wall, seed, pairs }
    }

    /// Round 168: the share folder is world-writable (sticky) — the
    /// temporary file is made NEW (never an existing one, never through a
    /// link someone laid there), unique to this process and file.
    fn write_atomic(path: &Path, data: &[u8]) -> bool {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let tmp = path.with_file_name(format!(".{name}.{}.neu", std::process::id()));
        let _ = std::fs::remove_file(&tmp);
        let written = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o644)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&tmp)
            .and_then(|mut f| f.write_all(data));
        if written.is_err() {
            let _ = std::fs::remove_file(&tmp);
            return false;
        }
        std::fs::rename(&tmp, path).is_ok()
    }

    /// The session's side: publish what changed (cheap when nothing did).
    /// False: no share folder (a device without the Matrix greeter).
    pub fn publish() -> bool {
        if !Path::new(DIR).is_dir() {
            return false;
        }
        let user = std::env::var("USER").unwrap_or_default();
        if user.is_empty() || user.contains('/') {
            return false;
        }
        let wall = crate::wallpaper::current();
        let mut text = String::new();
        if let Some(w) = &wall {
            let stamp = std::fs::metadata(w).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            text.push_str(&format!("wall-from={w}\nwall-stamp={stamp}\n"));
        }
        if let Some(p) = crate::Palette::load() {
            text.push_str(&format!("seed={}\n", p.primary.to_hex()));
        }
        for k in KEYS {
            if let Some(v) = crate::settings::read(k) {
                text.push_str(&format!("{k}={}\n", v.trim()));
            }
        }
        let look = file(&user, "look");
        let before = std::fs::read_to_string(&look).unwrap_or_default();
        if before == text {
            return true;
        }
        // The wallpaper copy only when the picture itself changed.
        let old = parse(&before);
        let same_wall = |k: &str| old.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone()) == parse(&text).into_iter().find(|(a, _)| a == k).map(|(_, v)| v);
        if let Some(w) = &wall {
            if !(same_wall("wall-from") && same_wall("wall-stamp")) || !file(&user, "wand").is_file() {
                if let Ok(data) = std::fs::read(w) {
                    write_atomic(&file(&user, "wand"), &data);
                }
            }
        }
        write_atomic(&look, text.as_bytes())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_look_file_reads_back() {
            let pairs = parse("wall-from=/home/m/a.jpg\nseed=#5b4bdb\nlock-clock-weight=0.700\n\nbroken line\n");
            assert_eq!(pairs.len(), 3);
            let s = Shared { wall: None, seed: None, pairs };
            assert_eq!(s.get("lock-clock-weight").as_deref(), Some("0.700"));
            assert_eq!(s.get("lock-clock-body"), None);
            let c = crate::Rgba::from_hex("#5b4bdb").unwrap();
            assert_eq!(c.to_hex(), "#5b4bdb");
        }
    }
}

/// THE ISLAND (round 105): notifications shown IN the top bar — its glass
/// chips flow together into one glass island (app icon, scrolling text)
/// and apart again. The bar CLAIMS the island at start; the notification
/// daemon then shows no popup of its own (sound and history stay).
pub mod island {
    const CLAIM: &str = "matrix-island";

    /// The setting (Mitteilungen › In der Topbar), default on.
    pub fn enabled() -> bool {
        !crate::settings::flag("bar-island", &["off"])
    }

    pub fn set_enabled(on: bool) {
        crate::settings::write("bar-island", if on { "on" } else { "off" });
    }

    /// The bar: «I show notifications» (its pid).
    pub fn claim() {
        crate::runtime::write(CLAIM, &std::process::id().to_string());
    }

    /// Does a live bar show notifications right now?
    pub fn active() -> bool {
        enabled()
            && crate::runtime::read(CLAIM)
                .and_then(|t| t.trim().parse::<u32>().ok())
                .is_some_and(|pid| std::path::Path::new(&format!("/proc/{pid}")).exists())
    }

    /// The text the island scrolls: title and body on one line, at most
    /// `max` characters (then «…»).
    pub fn line(title: &str, text: &str, max: usize) -> String {
        let join = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let (t, x) = (join(title), join(text));
        let all = match (t.is_empty(), x.is_empty()) {
            (false, false) => format!("{t}: {x}"),
            (false, true) => t,
            (true, _) => x,
        };
        if all.chars().count() <= max {
            all
        } else {
            let cut: String = all.chars().take(max).collect();
            format!("{}…", cut.trim_end())
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn the_line_joins_and_cuts() {
            assert_eq!(super::line("Einkauf", "Brot\n und  Eier", 50), "Einkauf: Brot und Eier");
            assert_eq!(super::line("", "nur Text", 50), "nur Text");
            assert_eq!(super::line("Lang", "abcdefghij", 8), "Lang: ab…");
        }
    }
}

/// THE INBOX (round 31, 02.11.2026): the notification history as a
/// shared board. The notification daemon PUBLISHES it (runtime file,
/// whole-or-nothing) and reads COMMANDS from a mailbox; the bar's user card
/// shows it and sends «open / dismiss / clear». Before, the history lived
/// only in the daemon's memory and only its own bell panel could show it.
/// SESSION LOCK STATE (round 130): the lock screen says it covers the
/// session — apps behind it get no frames from the compositor and must
/// not read that as their own weakness (Matrix Tube stepped its quality
/// down and stopped a local file, 05.11.2026).
pub mod session_lock {
    const FLAG: &str = "matrix-locked";

    /// The lock: it covers the session now (its pid).
    pub fn set() {
        crate::runtime::write(FLAG, &std::process::id().to_string());
    }

    /// The lock: the session is open again.
    pub fn clear() {
        crate::runtime::remove(FLAG);
    }

    /// Anyone: is the session locked (the lock that said so still lives)?
    pub fn active() -> bool {
        crate::runtime::read(FLAG)
            .and_then(|t| t.trim().parse::<u32>().ok())
            .is_some_and(|pid| std::path::Path::new(&format!("/proc/{pid}")).exists())
    }
}

/// NOW PLAYING (round 129): what plays right now, for every surface that
/// shows it (the lock screen's player, the bar) — the same board-and-
/// mailbox shape as `inbox`. A playing app publishes about once a second
/// and takes the commands; a reader shows the board while its writer
/// lives (the pid) and the last word is fresh.
pub mod now_playing {
    use serde::{Deserialize, Serialize};

    const BOARD: &str = "matrix-now-playing.json";
    const MAIL: &str = "matrix-now-playing-cmd";

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Playing {
        /// The app's name for people («Matrix Tube»).
        pub app: String,
        pub title: String,
        pub artist: String,
        pub position_s: f64,
        pub duration_s: f64,
        pub playing: bool,
        /// A picture of it (cover, thumbnail) — a file path.
        pub art: Option<String>,
        /// The writer — the board counts while it lives.
        pub pid: u32,
        /// Unix milliseconds of this word (the position moves on from it).
        pub at_ms: u64,
        /// It can go to the next / previous piece.
        pub can_next: bool,
        pub can_previous: bool,
    }

    impl Playing {
        /// The position now (it moved on since the word while playing).
        pub fn position_now(&self) -> f64 {
            if !self.playing {
                return self.position_s;
            }
            let gone = now_ms().saturating_sub(self.at_ms) as f64 / 1000.0;
            (self.position_s + gone).min(if self.duration_s > 0.0 { self.duration_s } else { f64::MAX })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Command {
        Toggle,
        Next,
        Previous,
        Seek(f64),
    }

    impl Command {
        fn line(&self) -> String {
            match self {
                Command::Toggle => String::from("toggle"),
                Command::Next => String::from("next"),
                Command::Previous => String::from("previous"),
                Command::Seek(t) => format!("seek {t:.2}"),
            }
        }

        pub fn parse(line: &str) -> Option<Command> {
            let mut it = line.split_whitespace();
            match (it.next()?, it.next()) {
                ("toggle", None) => Some(Command::Toggle),
                ("next", None) => Some(Command::Next),
                ("previous", None) => Some(Command::Previous),
                ("seek", Some(t)) => t.parse().ok().map(Command::Seek),
                _ => None,
            }
        }
    }

    pub fn now_ms() -> u64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
    }

    /// The playing app: say what plays (pid and time are filled in).
    pub fn publish(mut p: Playing) {
        p.pid = std::process::id();
        p.at_ms = now_ms();
        if let Ok(text) = serde_json::to_string(&p) {
            crate::runtime::write(BOARD, &text);
        }
    }

    /// The playing app: it stopped — the board goes (only its own).
    pub fn clear() {
        if read_raw().is_some_and(|p| p.pid == std::process::id()) {
            crate::runtime::remove(BOARD);
        }
    }

    fn read_raw() -> Option<Playing> {
        crate::runtime::read(BOARD).and_then(|t| serde_json::from_str(&t).ok())
    }

    /// Anyone: what plays — None when nothing does, its writer is gone,
    /// or its last word is older than 15 s.
    pub fn read() -> Option<Playing> {
        let p = read_raw()?;
        let alive = std::path::Path::new(&format!("/proc/{}", p.pid)).exists();
        (alive && now_ms().saturating_sub(p.at_ms) < 15_000).then_some(p)
    }

    /// A remote: ask the playing app.
    pub fn send(cmd: Command) {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(crate::runtime::path(MAIL)) {
            let _ = writeln!(f, "{}", cmd.line());
        }
    }

    /// The playing app: take the waiting commands.
    pub fn take() -> Vec<Command> {
        let mail = crate::runtime::path(MAIL);
        let taken = crate::runtime::path(&format!(".{MAIL}.{}", std::process::id()));
        if std::fs::rename(&mail, &taken).is_err() {
            return Vec::new();
        }
        let text = std::fs::read_to_string(&taken).unwrap_or_default();
        let _ = std::fs::remove_file(&taken);
        text.lines().filter_map(Command::parse).collect()
    }

    /// «3:07» — minutes and seconds (hours when long).
    pub fn clock(s: f64) -> String {
        let s = s.max(0.0) as u64;
        if s >= 3600 {
            format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
        } else {
            format!("{}:{:02}", s / 60, s % 60)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn commands_round_trip() {
            for c in [Command::Toggle, Command::Next, Command::Previous, Command::Seek(12.5)] {
                assert_eq!(Command::parse(&c.line()), Some(c));
            }
            assert_eq!(Command::parse("seek x"), None);
        }

        #[test]
        fn the_clock_reads_like_a_player() {
            assert_eq!(clock(187.4), "3:07");
            assert_eq!(clock(3725.0), "1:02:05");
        }

        #[test]
        fn a_paused_position_stands() {
            let p = Playing { app: String::new(), title: String::new(), artist: String::new(), position_s: 10.0, duration_s: 100.0, playing: false, art: None, pid: 0, at_ms: 0, can_next: false, can_previous: false };
            assert_eq!(p.position_now(), 10.0);
            let q = Playing { playing: true, at_ms: now_ms() - 2000, ..p };
            assert!((q.position_now() - 12.0).abs() < 0.2);
        }
    }
}

pub mod inbox {
    use serde::{Deserialize, Serialize};

    const BOARD: &str = "matrix-inbox.json";
    const MAIL: &str = "matrix-inbox-cmd";

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Item {
        pub id: u32,
        pub app: String,
        pub title: String,
        pub text: String,
        /// Unix seconds when it arrived.
        pub at: u64,
        /// It has a default action (a tap opens something).
        pub opens: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Command {
        Open(u32),
        Dismiss(u32),
        Clear,
    }

    impl Command {
        fn line(&self) -> String {
            match self {
                Command::Open(id) => format!("open {id}"),
                Command::Dismiss(id) => format!("dismiss {id}"),
                Command::Clear => String::from("clear"),
            }
        }

        pub fn parse(line: &str) -> Option<Command> {
            let mut it = line.split_whitespace();
            match (it.next()?, it.next()) {
                ("open", Some(n)) => n.parse().ok().map(Command::Open),
                ("dismiss", Some(n)) => n.parse().ok().map(Command::Dismiss),
                ("clear", None) => Some(Command::Clear),
                _ => None,
            }
        }
    }

    /// The daemon: the history now (newest first).
    pub fn publish(items: &[Item]) {
        if let Ok(text) = serde_json::to_string(items) {
            crate::runtime::write(BOARD, &text);
        }
    }

    /// Anyone: the history as last published (empty when none).
    pub fn read() -> Vec<Item> {
        crate::runtime::read(BOARD).and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    /// The board's change time (cheap «has it changed?» for a poller).
    pub fn stamp() -> Option<std::time::SystemTime> {
        std::fs::metadata(crate::runtime::path(BOARD)).and_then(|m| m.modified()).ok()
    }

    /// A client: ask the daemon (appended; the daemon takes them in order).
    pub fn send(cmd: Command) {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(crate::runtime::path(MAIL)) {
            let _ = writeln!(f, "{}", cmd.line());
        }
    }

    /// The daemon: take the waiting commands (the mailbox is emptied by a
    /// rename, so a command written meanwhile waits for the next take).
    pub fn take() -> Vec<Command> {
        let mail = crate::runtime::path(MAIL);
        let taken = crate::runtime::path(&format!(".{MAIL}.{}", std::process::id()));
        if std::fs::rename(&mail, &taken).is_err() {
            return Vec::new();
        }
        let text = std::fs::read_to_string(&taken).unwrap_or_default();
        let _ = std::fs::remove_file(&taken);
        text.lines().filter_map(Command::parse).collect()
    }

    #[cfg(test)]
    mod tests {
        use super::Command;

        #[test]
        fn commands_round_trip() {
            for c in [Command::Open(7), Command::Dismiss(12), Command::Clear] {
                assert_eq!(Command::parse(&c.line()), Some(c));
            }
            assert_eq!(Command::parse("open x"), None);
            assert_eq!(Command::parse("clear 3"), None);
        }
    }
}

/// THE RUNTIME BOARD (R210 family): small shared states and mailboxes of
/// the session live in `$XDG_RUNTIME_DIR` (tmpfs, per user, gone at
/// logout) — cinema mode, the subtitle strip, the dock's reserve, the
/// house and companion sockets, the keyboard's wire. One place builds the
/// path (fallback /tmp) and reads, writes, removes; writes go through a
/// temporary file and a rename, so a reader never sees half a state.
pub mod runtime {
    use std::path::PathBuf;

    pub fn dir() -> PathBuf {
        PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp")))
    }

    pub fn path(name: &str) -> PathBuf {
        dir().join(name)
    }

    /// Write whole or not at all (temp file + rename).
    pub fn write(name: &str, text: &str) -> bool {
        let target = path(name);
        let temp = path(&format!(".{name}.{}", std::process::id()));
        std::fs::write(&temp, text).is_ok() && std::fs::rename(&temp, &target).is_ok()
    }

    pub fn read(name: &str) -> Option<String> {
        std::fs::read_to_string(path(name)).ok()
    }

    pub fn remove(name: &str) {
        let _ = std::fs::remove_file(path(name));
    }

    /// A TRAIL (round 29): the last `keep` lines of a running diagnosis in
    /// a runtime file, each with the time of day — the house bar's tap log
    /// that found two input bugs in one evening (02.11.2026) as one call
    /// instead of a hand-rolled copy each time. `cat $XDG_RUNTIME_DIR/name`.
    pub fn trail(name: &str, line: &str, keep: usize) {
        let old = read(name).unwrap_or_default();
        let lines = trail_push(&old, &format!("{} {line}", crate::time::clock_hms()), keep);
        write(name, &lines);
    }

    /// A STAMP (round 44): «it happened now» as an empty runtime file —
    /// across processes, survives nothing but the session.
    pub fn stamp(name: &str) {
        write(name, "");
    }

    /// Did `stamp(name)` happen within `ms`? (False when never.)
    pub fn fresh(name: &str, ms: u64) -> bool {
        std::fs::metadata(path(name))
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age.as_millis() < ms as u128)
    }

    #[cfg(test)]
    mod stamp_tests {
        #[test]
        fn a_stamp_is_fresh_then_old() {
            let name = format!("mk-stamp-probe-{}", std::process::id());
            assert!(!super::fresh(&name, 10_000));
            super::stamp(&name);
            assert!(super::fresh(&name, 10_000));
            std::thread::sleep(std::time::Duration::from_millis(30));
            assert!(!super::fresh(&name, 10));
            super::remove(&name);
        }
    }

    /// A LOG FILE WITH A CEILING (round 41): `name` in the runtime folder,
    /// opened for appending — emptied first when it is over `max_bytes`.
    /// For a child's stderr: the keyboard surface's log grew without end
    /// in a RAM folder (03.11.2026).
    pub fn log_file(name: &str, max_bytes: u64) -> Option<std::fs::File> {
        log_file_at(&path(name), max_bytes)
    }

    /// `log_file` at a given path (tests use it; the environment stays put).
    pub fn log_file_at(p: &std::path::Path, max_bytes: u64) -> Option<std::fs::File> {
        let too_big = std::fs::metadata(p).map(|m| m.len() > max_bytes).unwrap_or(false);
        std::fs::OpenOptions::new().create(true).append(!too_big).write(true).truncate(too_big).open(p).ok()
    }

    /// The pure part of `trail`: append, keep the last `keep` lines.
    pub fn trail_push(old: &str, line: &str, keep: usize) -> String {
        let mut lines: Vec<&str> = old.lines().collect();
        lines.push(line);
        let skip = lines.len().saturating_sub(keep.max(1));
        let mut out = lines[skip..].join("\n");
        out.push('\n');
        out
    }

    #[cfg(test)]
    mod log_file_tests {
        #[test]
        fn a_full_log_starts_empty() {
            use std::io::Write;
            let dir = std::env::temp_dir().join(format!("mk-log-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let p = dir.join("probe.log");
            let mut f = super::log_file_at(&p, 10).unwrap();
            f.write_all(b"0123456789abcdef").unwrap();
            drop(f);
            let mut f = super::log_file_at(&p, 10).unwrap();
            f.write_all(b"neu").unwrap();
            drop(f);
            assert_eq!(std::fs::read_to_string(dir.join("probe.log")).unwrap(), "neu");
            let mut f = super::log_file_at(&p, 10).unwrap();
            f.write_all(b"+").unwrap();
            assert_eq!(std::fs::read_to_string(dir.join("probe.log")).unwrap(), "neu+");
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    #[cfg(test)]
    mod trail_tests {
        #[test]
        fn keeps_the_last_lines() {
            let mut t = String::new();
            for i in 0..5 {
                t = super::trail_push(&t, &format!("l{i}"), 3);
            }
            assert_eq!(t, "l2\nl3\nl4\n");
        }
    }

    /// ONCE ACROSS PROCESSES (round 26): the first to claim `name` wins
    /// (atomic create); others get false. The claim lapses after `ttl`
    /// (an older claim counts as free again) — e.g. one panel painter makes
    /// the palette, the other skips (wallpaper, 02.11.2026).
    pub fn claim(name: &str, ttl: std::time::Duration) -> bool {
        let p = path(name);
        let stale = std::fs::metadata(&p).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age >= ttl);
        if stale {
            let _ = std::fs::remove_file(&p);
        }
        std::fs::OpenOptions::new().write(true).create_new(true).open(&p).is_ok()
    }

    #[cfg(test)]
    mod claim_tests {
        #[test]
        fn first_wins_until_it_lapses() {
            let name = format!("mk-claim-test-{}", std::process::id());
            let ttl = std::time::Duration::from_millis(300);
            assert!(super::claim(&name, ttl));
            assert!(!super::claim(&name, ttl), "second loses");
            std::thread::sleep(std::time::Duration::from_millis(350));
            assert!(super::claim(&name, ttl), "lapsed claim is free");
            super::remove(&name);
        }
    }

    /// Remove `<prefix><pid><suffix>` files of processes that no longer
    /// live — what a killed or crashed app left behind (mailboxes,
    /// Krabbler states; 20 of them had piled up on the duo, 02.11.2026).
    /// Returns how many went.
    pub fn sweep_orphans(prefix: &str, suffix: &str) -> usize {
        let Ok(entries) = std::fs::read_dir(dir()) else { return 0 };
        let mut gone = 0;
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(pid) = orphan_pid(&name, prefix, suffix) else { continue };
            if !std::path::Path::new(&format!("/proc/{pid}")).exists() && std::fs::remove_file(e.path()).is_ok() {
                gone += 1;
            }
        }
        gone
    }

    /// The pid in `<prefix><pid><suffix>` (pure — tested).
    pub fn orphan_pid(name: &str, prefix: &str, suffix: &str) -> Option<u32> {
        name.strip_prefix(prefix)?.strip_suffix(suffix)?.parse().ok()
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn writes_reads_and_removes_whole_states() {
            let name = format!("matrix-runtime-probe-{}", std::process::id());
            assert!(super::write(&name, "1\nzwei"));
            assert_eq!(super::read(&name).as_deref(), Some("1\nzwei"));
            super::remove(&name);
            assert_eq!(super::read(&name), None);
        }

        #[test]
        fn finds_the_orphans_pid() {
            assert_eq!(super::orphan_pid("matrix-house-4242.sock", "matrix-house-", ".sock"), Some(4242));
            assert_eq!(super::orphan_pid("matrix-krabbler-17", "matrix-krabbler-", ""), Some(17));
            assert_eq!(super::orphan_pid("matrix-house-x.sock", "matrix-house-", ".sock"), None);
            // A Ton test file is no orphan of anyone.
            assert_eq!(super::orphan_pid("matrix-test-tone-0.wav", "matrix-house-", ".sock"), None);
        }
    }
}

/// CINEMA MODE (R210 family, the owner 01.11.2026): while a video plays,
/// the backgrounds of all panels dim; paused or closed, they come back.
/// FLASH (R210 family, 02.11.2026): a short note that goes by itself —
/// «Übersprungen: Sponsor», «Gespeichert in Musik/Matrix Tube». Apps kept
/// `Option<(String, Instant)>` and compared the age by hand at every use.
#[derive(Debug, Clone, Default)]
pub struct Flash {
    note: Option<(String, std::time::Instant)>,
    life_s: f32,
}

impl Flash {
    /// A flash that shows each note for `life_s` seconds.
    pub fn new(life_s: f32) -> Flash {
        Flash { note: None, life_s }
    }

    pub fn set(&mut self, text: impl Into<String>) {
        self.note = Some((text.into(), std::time::Instant::now()));
    }

    pub fn clear(&mut self) {
        self.note = None;
    }

    /// The note while it lives.
    pub fn text(&self) -> Option<&str> {
        self.note.as_ref().filter(|(_, at)| at.elapsed().as_secs_f32() < self.life_s).map(|(s, _)| s.as_str())
    }

    /// Still showing (redraw until it is gone — one beat more).
    pub fn alive(&self) -> bool {
        self.note.as_ref().is_some_and(|(_, at)| at.elapsed().as_secs_f32() < self.life_s + 0.6)
    }
}

/// BLUETOOTH (R210 family, 02.11.2026): BlueZ through `bluetoothctl` —
/// what Matrix Einstellungen showed and what a player needs for headphones.
/// Blocking calls: run them off the surface (task, loom).
pub mod bluetooth {
    use crate::command::text_from;

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct Device {
        pub mac: String,
        pub name: String,
        pub connected: bool,
        /// BlueZ's kind («audio-headset», «audio-card», «input-keyboard» …).
        pub icon: String,
    }

    impl Device {
        /// Does it play sound (headphones, speakers)?
        /// What kind of device it is (round 160) — BlueZ's icon, else its name.
        pub fn kind(&self) -> Kind {
            kind_of(&self.icon, &self.name)
        }

        pub fn audio(&self) -> bool {
            self.icon.starts_with("audio")
        }
    }

    /// "Device AA:BB:CC:DD:EE:FF name with spaces" lines -> (mac, name).
    /// The kinds a Bluetooth device comes as (round 160, 05.11.2026: a Rapoo
    /// mouse and keyboard for the duo at the TV — all showed the same symbol).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Audio,
        Keyboard,
        Mouse,
        Gamepad,
        Phone,
        Computer,
        Other,
    }

    impl Kind {
        /// Its symbol (Material Symbols code points, in the Matrix font).
        pub fn glyph(self) -> char {
            match self {
                Kind::Audio => '\u{e310}',
                Kind::Keyboard => '\u{e312}',
                Kind::Mouse => '\u{e323}',
                Kind::Gamepad => '\u{ea28}',
                Kind::Phone => '\u{e32c}',
                Kind::Computer => '\u{e30a}',
                Kind::Other => '\u{e1a7}',
            }
        }

        /// What it is, in a word (the list's second line).
        pub fn word(self) -> &'static str {
            match self {
                Kind::Audio => "Ton",
                Kind::Keyboard => "Tastatur",
                Kind::Mouse => "Maus",
                Kind::Gamepad => "Spielsteuerung",
                Kind::Phone => "Telefon",
                Kind::Computer => "Computer",
                Kind::Other => "Gerät",
            }
        }
    }

    /// The kind from BlueZ's `Icon:` (input-keyboard, audio-headset …), or —
    /// a device not yet asked — from its name (pure — tested).
    pub fn kind_of(icon: &str, name: &str) -> Kind {
        let i = icon.trim();
        let n = name.to_lowercase();
        let has = |words: &[&str]| words.iter().any(|w| n.contains(w));
        if i.starts_with("audio") || has(&["buds", "headphone", "headset", "airpods", "speaker", "jbl", "flip", "sound"]) {
            Kind::Audio
        } else if i == "input-keyboard" || has(&["keyboard", "tastatur", "keys"]) {
            Kind::Keyboard
        } else if i == "input-mouse" || i == "input-tablet" || has(&["mouse", "maus", "trackpad"]) {
            Kind::Mouse
        } else if i == "input-gaming" || has(&["controller", "gamepad", "xbox", "dualsense", "joy-con"]) {
            Kind::Gamepad
        } else if i == "phone" || has(&["phone", "galaxy", "pixel"]) {
            Kind::Phone
        } else if i == "computer" || has(&["laptop", "macbook", "surface"]) {
            Kind::Computer
        } else {
            Kind::Other
        }
    }

    #[cfg(test)]
    mod kind_tests {
        use super::*;

        #[test]
        fn the_rapoo_pair_and_the_buds_know_their_kind() {
            assert_eq!(kind_of("input-mouse", "RAPOO BT MOUSE"), Kind::Mouse);
            assert_eq!(kind_of("input-keyboard", "RAPOO Keyboard-1"), Kind::Keyboard);
            assert_eq!(kind_of("", "RAPOO Keyboard-1"), Kind::Keyboard, "nearby: by name");
            assert_eq!(kind_of("audio-headset", "the owner Alexander's Buds3 Pro"), Kind::Audio);
            assert_eq!(kind_of("", "JBL Flip 5"), Kind::Audio);
            assert_eq!(kind_of("", "48-97-82-6D-30-5F"), Kind::Other);
            assert_ne!(Kind::Mouse.glyph(), Kind::Keyboard.glyph());
        }
    }

    pub fn device_lines(out: &str) -> Vec<(String, String)> {
        out.lines()
            .filter_map(|line| {
                let mut parts = line.splitn(3, ' ');
                if parts.next() != Some("Device") {
                    return None;
                }
                Some((parts.next()?.to_string(), parts.next()?.to_string()))
            })
            .collect()
    }

    /// A scanned device without a name shows its address — useless to pick.
    pub fn named(mac: &str, name: &str) -> bool {
        !name.is_empty() && name.replace('-', ":") != mac
    }

    pub fn powered() -> bool {
        text_from("bluetoothctl", &["show"]).is_some_and(|s| s.lines().any(|l| l.trim() == "Powered: yes"))
    }

    pub fn power(on: bool) -> bool {
        let ok = text_from("bluetoothctl", &["power", if on { "on" } else { "off" }]).is_some();
        if on {
            // Pairable from the start (it stood on «no» on the duo).
            let _ = text_from("bluetoothctl", &["pairable", "on"]);
        }
        ok
    }

    pub fn connected(mac: &str) -> bool {
        text_from("bluetoothctl", &["info", mac]).is_some_and(|i| i.lines().any(|l| l.trim() == "Connected: yes"))
    }

    /// Connected and its icon, from one `info`.
    fn info(mac: &str) -> (bool, String) {
        let info = text_from("bluetoothctl", &["info", mac]).unwrap_or_default();
        let connected = info.lines().any(|l| l.trim() == "Connected: yes");
        let icon = info.lines().find_map(|l| l.trim().strip_prefix("Icon:").map(|i| i.trim().to_string())).unwrap_or_default();
        (connected, icon)
    }

    /// The paired devices, connected first.
    pub fn paired() -> Vec<Device> {
        let mut list: Vec<Device> = text_from("bluetoothctl", &["devices", "Paired"])
            .map(|o| device_lines(&o))
            .unwrap_or_default()
            .into_iter()
            .map(|(mac, name)| {
                let (connected, icon) = info(&mac);
                Device { connected, mac, name, icon }
            })
            .collect();
        list.sort_by(|a, b| b.connected.cmp(&a.connected).then(a.name.cmp(&b.name)));
        list
    }

    /// Seen by a scan, named, not paired — at most `max`, by name.
    pub fn nearby(max: usize) -> Vec<Device> {
        let paired = text_from("bluetoothctl", &["devices", "Paired"]).map(|o| device_lines(&o)).unwrap_or_default();
        let mut list: Vec<Device> = text_from("bluetoothctl", &["devices"])
            .map(|o| device_lines(&o))
            .unwrap_or_default()
            .into_iter()
            .filter(|(mac, name)| named(mac, name) && !paired.iter().any(|(m, _)| m == mac))
            .map(|(mac, name)| Device { mac, name, connected: false, icon: String::new() })
            .collect();
        list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        list.truncate(max);
        list
    }

    /// Look around for `secs` seconds (blocks).
    pub fn scan(secs: u32) {
        let _ = std::process::Command::new("bluetoothctl").args(["--timeout", &secs.to_string(), "scan", "on"]).output();
    }

    fn run(args: &[&str]) -> bool {
        std::process::Command::new("bluetoothctl")
            .args(["--timeout", "20"])
            .args(args)
            .output()
            .map(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).contains("Failed"))
            .unwrap_or(false)
    }

    /// Pair, trust and connect (headphones need no PIN). True = connected.
    pub fn pair(mac: &str) -> bool {
        pair_session(mac).is_ok()
    }

    /// PAIRING IN ONE SESSION (02.11.2026, the owner's Buds3 Pro): BlueZ
    /// forgets a found device when its discovery ends (`[DEL] Device`) —
    /// a later `bluetoothctl pair` in a fresh process then says «not
    /// available» and the tap did nothing. So one bluetoothctl session
    /// does it all: an agent (NoInputNoOutput — headphones), pairable,
    /// discovery until the device shows, pair, trust, connect. Ok, or a
    /// reason in words. Blocks up to about 50 s.
    pub fn pair_session(mac: &str) -> Result<(), String> {
        use std::io::{Read, Write};
        use std::process::{Command, Stdio};
        use std::sync::{Arc, Mutex};
        use std::time::{Duration, Instant};
        let mut child = Command::new("bluetoothctl").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|_| String::from("bluetoothctl fehlt"))?;
        let out = Arc::new(Mutex::new(String::new()));
        if let Some(mut so) = child.stdout.take() {
            let out = Arc::clone(&out);
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                while let Ok(n) = so.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    if let Ok(mut o) = out.lock() {
                        o.push_str(&String::from_utf8_lossy(&buf[..n]));
                    }
                }
            });
        }
        let mut stdin = child.stdin.take().ok_or_else(|| String::from("bluetoothctl ohne Eingang"))?;
        let mut say = |line: &str| {
            let _ = writeln!(stdin, "{line}");
        };
        let seen = |needles: &[&str]| out.lock().map(|o| needles.iter().any(|n| o.contains(n))).unwrap_or(false);
        let wait = |needles: &[&str], secs: u64| {
            let t = Instant::now();
            while t.elapsed() < Duration::from_secs(secs) {
                if seen(needles) {
                    return true;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            false
        };
        say("agent NoInputNoOutput");
        say("default-agent");
        say("pairable on");
        say("scan on");
        let found = wait(&[&format!("Device {mac}")], 15);
        let result = if !found {
            Err(String::from("nicht gefunden — ist das Gerät im Kopplungsmodus?"))
        } else {
            say(&format!("pair {mac}"));
            let mut paired = wait(&["Paired: yes", "Pairing successful", "AlreadyExists"], 25);
            // Round 160: a device that breaks off (the Rapoo keyboard: it
            // connected and cancelled) pairs at the second try, once its
            // half entry is gone and it has been seen anew.
            if !paired && seen(&["AuthenticationCanceled", "AuthenticationFailed", "ConnectionAttemptFailed"]) {
                if let Ok(mut o) = out.lock() {
                    o.clear();
                }
                say(&format!("remove {mac}"));
                std::thread::sleep(Duration::from_secs(1));
                if wait(&[&format!("Device {mac}")], 12) {
                    say(&format!("pair {mac}"));
                    paired = wait(&["Paired: yes", "Pairing successful", "AlreadyExists"], 25);
                }
            }
            if !paired {
                Err(if seen(&["AuthenticationRejected", "AuthenticationFailed", "AuthenticationCanceled"]) { String::from("das Gerät hat die Kopplung abgebrochen — Kopplungsmodus neu starten und noch einmal") } else { String::from("keine Antwort beim Koppeln") })
            } else {
                say(&format!("trust {mac}"));
                say(&format!("connect {mac}"));
                if wait(&["Connection successful", "Connected: yes"], 15) {
                    Ok(())
                } else {
                    Err(String::from("gekoppelt, aber die Verbindung kam nicht zustande"))
                }
            }
        };
        say("scan off");
        say("quit");
        std::thread::sleep(Duration::from_millis(300));
        let _ = child.kill();
        let _ = child.wait();
        result
    }

    pub fn connect(mac: &str, on: bool) -> bool {
        run(&[if on { "connect" } else { "disconnect" }, mac])
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn device_lines_and_names() {
            let d = device_lines("Device AA:BB:CC:DD:EE:FF WH-1000XM4\nDevice 11:22:33:44:55:66 11-22-33-44-55-66\nController x");
            assert_eq!(d.len(), 2);
            assert!(named(&d[0].0, &d[0].1));
            assert!(!named(&d[1].0, &d[1].1));
        }
    }
}

#[cfg(test)]
mod flash_tests {
    #[test]
    fn flash_lives_and_goes() {
        let mut f = super::Flash::new(0.05);
        assert!(f.text().is_none());
        f.set("hallo");
        assert_eq!(f.text(), Some("hallo"));
        std::thread::sleep(std::time::Duration::from_millis(80));
        assert!(f.text().is_none());
    }
}

/// NIGHT GLASS (R211 family, 02.11.2026): the Matrix night palette —
/// a dark ground that stays dark whatever the light/dark setting says,
/// ink for text on it, and six neon accents that tell kinds apart (the
/// four Krabbler species, chart series, tags). Neon needs the dark; on a
/// light palette use the theme's own colours instead.
pub mod night_glass {
    use crate::Rgba;

    pub const GROUND: Rgba = Rgba { r: 0.035, g: 0.04, b: 0.06, a: 1.0 };
    pub const INK: Rgba = Rgba { r: 0.78, g: 0.80, b: 0.85, a: 0.82 };
    pub const INK_SOFT: Rgba = Rgba { r: 0.78, g: 0.80, b: 0.85, a: 0.45 };
    /// A faint light (gaps, chip grounds).
    pub const HAZE: Rgba = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 0.05 };
    pub const AMBER: Rgba = Rgba { r: 0.98, g: 0.84, b: 0.25, a: 1.0 };
    pub const ROSE: Rgba = Rgba { r: 1.0, g: 0.36, b: 0.48, a: 1.0 };
    pub const CYAN: Rgba = Rgba { r: 0.30, g: 0.90, b: 0.95, a: 1.0 };
    pub const LIME: Rgba = Rgba { r: 0.55, g: 1.0, b: 0.45, a: 1.0 };
    pub const VIOLET: Rgba = Rgba { r: 0.70, g: 0.55, b: 1.0, a: 1.0 };
    pub const ORANGE: Rgba = Rgba { r: 1.0, g: 0.60, b: 0.25, a: 1.0 };
    /// The accents in order — kind `i` takes `NEON[i % 6]`.
    pub const NEON: [Rgba; 6] = [AMBER, ROSE, CYAN, LIME, VIOLET, ORANGE];

    /// Fill a BGRA buffer with the ground (the house's `base`).
    pub fn fill_bgra(buf: &mut [u8]) {
        let (b, g, r) = ((GROUND.b * 255.0) as u8, (GROUND.g * 255.0) as u8, (GROUND.r * 255.0) as u8);
        for px in buf.chunks_exact_mut(4) {
            px[0] = b;
            px[1] = g;
            px[2] = r;
            px[3] = 0xff;
        }
    }
}

/// DOZE (R211 family, 02.11.2026): an animated app falls asleep when nobody
/// touches it for a while — no frames, no battery (Matrix Krabbler); a
/// touch wakes it. The clock is the app's own (seconds of `glide`).
#[derive(Debug, Clone)]
pub struct Doze {
    after_s: f32,
    quiet_s: f32,
    asleep: bool,
}

impl Doze {
    pub fn new(after_s: f32) -> Doze {
        Doze { after_s, quiet_s: 0.0, asleep: false }
    }

    /// Time passed; true at the moment it falls asleep.
    pub fn tick(&mut self, dt: f32) -> bool {
        if self.asleep {
            return false;
        }
        self.quiet_s += dt;
        if self.quiet_s >= self.after_s {
            self.asleep = true;
            return true;
        }
        false
    }

    /// A touch; true when it woke it.
    pub fn touch(&mut self) -> bool {
        self.quiet_s = 0.0;
        std::mem::replace(&mut self.asleep, false)
    }

    pub fn asleep(&self) -> bool {
        self.asleep
    }
}

#[cfg(test)]
mod doze_tests {
    #[test]
    fn dozes_off_and_wakes() {
        let mut d = super::Doze::new(1.0);
        assert!(!d.tick(0.6));
        assert!(d.tick(0.6), "asleep after a second of quiet");
        assert!(!d.tick(5.0), "falls asleep once");
        assert!(d.asleep());
        assert!(d.touch(), "a touch wakes it");
        assert!(!d.asleep() && !d.touch());
        assert_eq!(super::night_glass::NEON.len(), 6);
    }
}

/// Players say `set(true/false)`; the wallpaper watches `path()` and asks
/// `active()`. The file lists the pids that play — a crashed player no
/// longer counts (its pid is gone).
pub mod cinema {
    use std::path::PathBuf;

    pub fn path() -> PathBuf {
        crate::runtime::path("matrix-kino")
    }

    fn alive(pid: u32) -> bool {
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }

    fn players() -> Vec<u32> {
        crate::runtime::read("matrix-kino")
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .filter(|p| alive(*p))
            .collect()
    }

    /// This process plays (true) or not (false).
    pub fn set(playing: bool) {
        let me = std::process::id();
        let mut list = players();
        list.retain(|p| *p != me);
        if playing {
            list.push(me);
        }
        let text: String = list.iter().map(|p| format!("{p}\n")).collect();
        crate::runtime::write("matrix-kino", &text);
    }

    /// Does any player play now?
    pub fn active() -> bool {
        !players().is_empty()
    }

    /// THE SUBTITLE STRIP (R210 family, the owner 02.11.2026): in cinema mode
    /// the dock steps aside and the subtitle stands in its own glass strip
    /// there — 10 px from the edges, its size free to grow without
    /// covering the picture. The player writes the line (`write`), the dock
    /// shows it; the dock publishes its own reserve so the player knows by
    /// how much a taller strip shrinks the picture's area.
    pub mod subtitle {
        use std::path::PathBuf;

        /// The size menu: label and font size (logical px).
        pub const SIZES: [(&str, f32); 4] = [("Klein", 18.0), ("Mittel", 24.0), ("Gross", 32.0), ("Riesig", 42.0)];
        /// Air around the strip (as around every window).
        pub const MARGIN: i32 = 10;
        const PAD: f32 = 12.0;

        pub fn path() -> PathBuf {
            crate::runtime::path("matrix-kino-untertitel")
        }

        /// The line on screen now (empty = a pause between lines).
        pub fn write(size: usize, text: &str) {
            crate::runtime::write("matrix-kino-untertitel", &format!("{size}\n{text}"));
        }

        /// No subtitles any more (off, paused, closed).
        pub fn clear() {
            crate::runtime::remove("matrix-kino-untertitel");
        }

        /// (size, text) while subtitles are on.
        pub fn read() -> Option<(usize, String)> {
            let all = crate::runtime::read("matrix-kino-untertitel")?;
            let (size, text) = all.split_once('\n').unwrap_or((all.as_str(), ""));
            Some((size.trim().parse().unwrap_or(1).min(SIZES.len() - 1), text.to_string()))
        }

        pub fn px(size: usize) -> f32 {
            SIZES[size.min(SIZES.len() - 1)].1
        }

        /// The strip's height for a size: two lines and padding.
        pub fn strip_height(size: usize) -> u32 {
            (2.0 * px(size) * 1.3 + 2.0 * PAD).ceil() as u32
        }

        /// What the strip takes from the bottom edge (strip + its margin).
        pub fn strip_zone(size: usize) -> i32 {
            strip_height(size) as i32 + MARGIN
        }

        /// The chosen size (settings `untertitel-groesse`, Mittel by default).
        pub fn saved_size() -> usize {
            crate::settings::read("untertitel-groesse").and_then(|v| v.parse().ok()).unwrap_or(1).min(SIZES.len() - 1)
        }

        pub fn save_size(size: usize) {
            crate::settings::write("untertitel-groesse", &size.to_string());
        }

        /// The dock tells what it takes from the bottom edge in total.
        pub fn publish_dock_zone(zone: i32) {
            crate::runtime::write("matrix-dock-zone", &zone.to_string());
        }

        pub fn dock_zone() -> i32 {
            crate::runtime::read("matrix-dock-zone").and_then(|v| v.trim().parse().ok()).unwrap_or(66)
        }

        /// How much less height the picture has with the strip in place.
        pub fn picture_shrink(size: usize) -> f32 {
            (strip_zone(size) - dock_zone()).max(0) as f32
        }
    }

    /// THE ANNOUNCER (R210 family): a player holds one and tells it every
    /// frame whether it plays — it speaks only on a change, and when the
    /// player ends (dropped, closed, crashed out of its loop) it signs off.
    #[derive(Default)]
    pub struct Announcer {
        on: bool,
    }

    impl Announcer {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn follow(&mut self, playing: bool) {
            if playing != self.on {
                self.on = playing;
                set(playing);
            }
        }

        pub fn is_on(&self) -> bool {
            self.on
        }
    }

    impl Drop for Announcer {
        fn drop(&mut self) {
            if self.on {
                set(false);
            }
        }
    }
}

/// THE HOUSE'S MAILBOX (R210 family): every window of the light shell
/// (iced-free Matrix apps) listens on one socket per process. The top bar
/// speaks to the FOCUSED window through it — on a duo windows carry no
/// head, the app's name chip in the bar holds their menu: `root` opens the
/// app's settings (its root pane), `close` closes it the app's own way.
pub mod house {
    use std::path::PathBuf;

    pub fn socket(pid: u32) -> PathBuf {
        crate::runtime::path(&format!("matrix-house-{pid}.sock"))
    }

    /// Is the process `pid` a light-shell window (it has a mailbox)?
    pub fn listens(pid: u32) -> bool {
        socket(pid).exists() && std::path::Path::new(&format!("/proc/{pid}")).exists()
    }

    /// A word to the window of process `pid` — false if none listens.
    pub fn send(pid: u32, word: &str) -> bool {
        std::os::unix::net::UnixDatagram::unbound()
            .and_then(|s| s.send_to(word.as_bytes(), socket(pid)))
            .is_ok()
    }
}

/// HEARING GUARD (R213 family, the owner 02.11.2026: «hearing protection — so that
/// notifications and everything don't suddenly blast up in volume
/// when I have headphones on»). The settings and the pure rules; the
/// service `matrix-earguard` applies them through PipeWire's pactl:
/// on headphones the volume stays under a cap, they start at a safe level,
/// every new sound fades in, system sounds play quieter.
pub mod hearing {
    /// Settings keys (CONTRACTS.txt).
    pub const KEY_ON: &str = "hearing-guard";
    pub const KEY_MAX: &str = "hearing-max";
    pub const KEY_START: &str = "hearing-start";
    pub const KEY_EVENTS: &str = "hearing-events";

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Guard {
        pub on: bool,
        /// Highest headphone volume, percent.
        pub max: u32,
        /// Volume headphones start at when they arrive, percent.
        pub start: u32,
        /// System and notification sounds, percent of their volume.
        pub events: u32,
        /// How long a new sound takes to reach its level, seconds.
        pub fade_s: f32,
    }

    impl Default for Guard {
        fn default() -> Self {
            Guard { on: true, max: 60, start: 35, events: 40, fade_s: 1.2 }
        }
    }

    fn percent(key: &str, default: u32) -> u32 {
        crate::settings::read(key).and_then(|v| v.trim().parse().ok()).unwrap_or(default).clamp(5, 100)
    }

    impl Guard {
        pub fn load() -> Guard {
            let d = Guard::default();
            Guard {
                on: crate::settings::read(KEY_ON).map(|v| v.trim() != "off").unwrap_or(true),
                max: percent(KEY_MAX, d.max),
                start: percent(KEY_START, d.start),
                events: percent(KEY_EVENTS, d.events),
                fade_s: d.fade_s,
            }
        }

        pub fn save(&self) {
            crate::settings::write(KEY_ON, if self.on { "on" } else { "off" });
            crate::settings::write(KEY_MAX, &self.max.to_string());
            crate::settings::write(KEY_START, &self.start.to_string());
            crate::settings::write(KEY_EVENTS, &self.events.to_string());
        }
    }

    /// What the guard sees and last did — written by the service into the
    /// runtime file `matrix-hearing` (JSON), read by Einstellungen and the
    /// volume keys. Proof that it works, not a promise (owner 02.11.2026:
    /// «how am I supposed to be sure that this works reliably?»).
    #[derive(Debug, Clone, Default, PartialEq)]
    pub struct Status {
        /// The headphones now playing (none: a speaker).
        pub phones: Option<String>,
        /// The output now playing, whatever it is (its name for people).
        pub output: Option<String>,
        /// Their volume, percent.
        pub volume: u32,
        /// The last thing the guard did, in words, and when (unix seconds).
        pub last: Option<(String, u64)>,
        /// How often it stepped in since it started.
        pub count: u32,
    }

    const STATUS_FILE: &str = "matrix-hearing";

    impl Status {
        pub fn publish(&self) {
            let v = serde_json::json!({
                "phones": self.phones, "output": self.output, "volume": self.volume, "count": self.count,
                "last": self.last.as_ref().map(|(t, at)| serde_json::json!([t, at])),
            });
            crate::runtime::write(STATUS_FILE, &v.to_string());
        }

        pub fn read() -> Option<Status> {
            let v: serde_json::Value = serde_json::from_str(&crate::runtime::read(STATUS_FILE)?).ok()?;
            Some(Status {
                phones: v.get("phones").and_then(|p| p.as_str()).map(String::from),
                output: v.get("output").and_then(|p| p.as_str()).map(String::from),
                volume: v.get("volume").and_then(|p| p.as_u64()).unwrap_or(0) as u32,
                count: v.get("count").and_then(|p| p.as_u64()).unwrap_or(0) as u32,
                last: v.get("last").and_then(|l| Some((l.get(0)?.as_str()?.to_string(), l.get(1)?.as_u64()?))),
            })
        }
    }

    /// The highest volume the keys may set now (0..1): the guard's cap on
    /// headphones, else full. The volume key passes it to `wpctl -l`, so
    /// it never even goes past it.
    pub fn key_limit() -> f32 {
        let g = Guard::load();
        match Status::read() {
            Some(st) if g.on && st.phones.is_some() => g.max as f32 / 100.0,
            _ => 1.0,
        }
    }

    /// HEADPHONES OR SPEAKER, per device (owner 02.11.2026: «can I
    /// assign Bluetooth devices myself?»). The user's word wins; else the
    /// device's own kind decides; unknown counts as headphones (safe).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Headphones,
        Speaker,
    }

    pub const KEY_DEVICES: &str = "hearing-devices";

    /// The user's choices: «E4:92:…=headphones» lines.
    pub fn device_kinds() -> Vec<(String, Kind)> {
        crate::settings::read(KEY_DEVICES)
            .unwrap_or_default()
            .split([',', '\n'])
            .filter_map(|l| {
                let (mac, k) = l.trim().split_once('=')?;
                Some((mac.to_uppercase(), if k.trim() == "speaker" { Kind::Speaker } else { Kind::Headphones }))
            })
            .collect()
    }

    pub fn device_kind(mac: &str) -> Option<Kind> {
        device_kinds().into_iter().find(|(m, _)| m.eq_ignore_ascii_case(mac)).map(|(_, k)| k)
    }

    pub fn set_device_kind(mac: &str, kind: Kind) {
        let mut all: Vec<(String, Kind)> = device_kinds().into_iter().filter(|(m, _)| !m.eq_ignore_ascii_case(mac)).collect();
        all.push((mac.to_uppercase(), kind));
        let text = all.iter().map(|(m, k)| format!("{m}={}", if *k == Kind::Speaker { "speaker" } else { "headphones" })).collect::<Vec<_>>().join(",");
        crate::settings::write(KEY_DEVICES, &text);
    }

    /// What the device says it is: BlueZ's icon or PipeWire's form factor
    /// («audio-card»/«speaker» = speaker; headset, headphones, hands-free,
    /// unknown = headphones — the safe guess).
    pub fn suggest(icon: Option<&str>, form_factor: Option<&str>) -> Kind {
        let speaker = matches!(form_factor, Some("speaker") | Some("hifi") | Some("car") | Some("tv"))
            || icon.is_some_and(|i| i.starts_with("audio-card") || i.contains("speaker"));
        if speaker {
            Kind::Speaker
        } else {
            Kind::Headphones
        }
    }

    /// The Bluetooth address in a PipeWire output's name
    /// («bluez_output.E4_92_82_07_76_63.1» → «E4:92:82:07:76:63»).
    pub fn bt_address(sink: &str) -> Option<String> {
        let mid = sink.strip_prefix("bluez_output.")?.split('.').next()?;
        let mac = mid.replace('_', ":");
        (mac.len() == 17).then_some(mac)
    }

    /// Is this output headphones? A Bluetooth output: the user's choice,
    /// else its kind. Otherwise a headphone/headset form factor or port (a
    /// jack, USB). `choice` = `device_kind` of its address (looked up by the
    /// caller — this stays pure).
    pub fn is_headphones(name: &str, port: Option<&str>, form_factor: Option<&str>, bus: Option<&str>, choice: Option<Kind>) -> bool {
        if name.starts_with("bluez_output") || bus == Some("bluetooth") {
            return choice.unwrap_or_else(|| suggest(None, form_factor)) == Kind::Headphones;
        }
        matches!(form_factor, Some("headphone") | Some("headset") | Some("hands-free")) || port.is_some_and(|p| {
            let p = p.to_lowercase();
            p.contains("headphone") || p.contains("headset")
        })
    }

    /// Is this stream a system or notification sound? (its role, or the
    /// little players that system sounds use)
    pub fn is_event(role: Option<&str>, binary: Option<&str>, app: Option<&str>) -> bool {
        let role = role.unwrap_or("").to_lowercase();
        matches!(role.as_str(), "event" | "notification" | "alert" | "alarm")
            || matches!(binary, Some("pw-play") | Some("paplay") | Some("pw-cat") | Some("canberra-gtk-play"))
            || app.is_some_and(|a| a.contains("matrix-sounds") || a.contains("matrix-notifications") || matches!(a, "pw-play" | "paplay" | "pw-cat"))
    }

    /// The fade of a new sound: (percent, delay before it in ms) from a
    /// tenth of `target` up to `target`, in `steps` steps over `fade_s`.
    pub fn fade(target: u32, fade_s: f32, steps: u32) -> Vec<(u32, u64)> {
        let steps = steps.max(1);
        let floor = (target / 10).max(1);
        (0..=steps)
            .map(|i| {
                let t = i as f32 / steps as f32;
                // Ease in: quiet for longer, the ear notices the start most.
                let e = t * t;
                let v = floor as f32 + (target - floor.min(target)) as f32 * e;
                (v.round() as u32, if i == 0 { 0 } else { (fade_s * 1000.0 / steps as f32) as u64 })
            })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn knows_headphones_and_events() {
            assert!(is_headphones("bluez_output.E4_92_82_07_76_63.1", None, None, None, None));
            assert!(is_headphones("bluez_output.E4_92_82_07_76_63.1", None, Some("headset"), Some("bluetooth"), None));
            // A Bluetooth speaker is no headphones — by its kind, or by choice.
            assert!(!is_headphones("bluez_output.D8_37_3B_27_93_F2.1", None, Some("speaker"), Some("bluetooth"), None));
            assert!(!is_headphones("bluez_output.E4_92_82_07_76_63.1", None, Some("headset"), None, Some(Kind::Speaker)));
            assert!(is_headphones("bluez_output.D8_37_3B_27_93_F2.1", None, Some("speaker"), None, Some(Kind::Headphones)));
            assert!(is_headphones("alsa_output.usb", Some("analog-output-headphones"), None, None, None));
            assert!(!is_headphones("alsa_output.platform-soc", None, Some("internal"), None, None));
            assert_eq!(bt_address("bluez_output.E4_92_82_07_76_63.1").as_deref(), Some("E4:92:82:07:76:63"));
            assert_eq!(suggest(Some("audio-card"), None), Kind::Speaker);
            assert_eq!(suggest(Some("audio-headset"), None), Kind::Headphones);
            assert!(is_event(Some("event"), None, None));
            assert!(is_event(None, Some("pw-play"), None));
            // A short system sound often names only its app (measured 02.11.).
            assert!(is_event(None, None, Some("pw-play")));
            assert!(!is_event(Some("music"), Some("matrix-tube"), Some("Matrix Tube")));
        }

        #[test]
        fn status_in_and_out_of_json() {
            let st = Status { phones: Some("Buds".into()), output: Some("Buds".into()), volume: 60, last: Some(("auf 60 % zurückgeholt".into(), 7)), count: 3 };
            let v = serde_json::json!({"phones": st.phones, "volume": st.volume, "count": st.count, "last": [st.last.as_ref().unwrap().0, 7]});
            assert_eq!(v["last"][0], "auf 60 % zurückgeholt");
        }

        #[test]
        fn fades_up_to_its_level() {
            let f = fade(80, 1.2, 6);
            assert_eq!(f.first().unwrap().0, 8);
            assert_eq!(f.last().unwrap().0, 80);
            assert!(f.windows(2).all(|w| w[0].0 <= w[1].0), "only upwards");
            assert_eq!(f.iter().map(|(_, d)| d).sum::<u64>(), 1200);
        }
    }
}

/// KRABBLER IN ANY APP (R211 family, 02.11.2026): the top bar's app card
/// switches word creatures on in a house app (`fam::Crawlers`). The card
/// says `crawl langleser,funke` / `crawl off` into the app's mailbox; the
/// app writes what crawls in it to the runtime file `matrix-krabbler-<pid>`
/// so the card shows the truth.
pub mod crawlers {
    /// (key, name, appetite) — the order of the card's rows.
    pub const SPECIES: [(&str, &str, &str); 4] = [
        ("langleser", "Langleser", "frisst Nomen"),
        ("weberin", "Weberin", "frisst lange Wörter"),
        ("fueller", "Füller", "frisst kleine Wörter"),
        ("funke", "Funke", "frisst Tätigkeiten"),
    ];

    fn file(pid: u32) -> String {
        format!("matrix-krabbler-{pid}")
    }

    /// The mailbox word for these species (none = off).
    pub fn word(keys: &[String]) -> String {
        if keys.is_empty() {
            String::from("crawl off")
        } else {
            format!("crawl {}", keys.join(","))
        }
    }

    /// The species a word asks for (None = not a crawl word).
    pub fn parse(word: &str) -> Option<Vec<String>> {
        let rest = word.strip_prefix("crawl")?.trim();
        if rest == "off" || rest.is_empty() {
            return Some(Vec::new());
        }
        Some(rest.split(',').map(str::trim).filter(|k| SPECIES.iter().any(|(s, _, _)| s == k)).map(String::from).collect())
    }

    /// Ask the app of process `pid` (false: it has no mailbox).
    pub fn ask(pid: u32, keys: &[String]) -> bool {
        crate::house::send(pid, &word(keys))
    }

    /// What crawls in the app of `pid` now (empty = nothing).
    pub fn state(pid: u32) -> Vec<String> {
        crate::runtime::read(&file(pid)).map(|s| s.split(',').map(str::trim).filter(|k| !k.is_empty()).map(String::from).collect()).unwrap_or_default()
    }

    /// The app tells what crawls in it (empty = remove the file).
    pub fn publish(keys: &[String]) {
        let pid = std::process::id();
        if keys.is_empty() {
            crate::runtime::remove(&file(pid));
        } else {
            crate::runtime::write(&file(pid), &keys.join(","));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn words_go_both_ways() {
            let keys = vec![String::from("langleser"), String::from("funke")];
            assert_eq!(parse(&word(&keys)), Some(keys));
            assert_eq!(parse("crawl off"), Some(Vec::new()));
            assert_eq!(parse("crawl spinne,weberin"), Some(vec![String::from("weberin")]));
            assert_eq!(parse("root"), None);
        }
    }
}

/// COMPANION WINDOWS (R210 family): an app opens a second window of its
/// own — on a duo the other panel (Tube: list here, stage there) — and
/// talks to it in words over a datagram socket per app and role. The
/// companion `listen`s; `send` says whether one is listening.
pub mod companion {
    use std::os::unix::net::UnixDatagram;
    use std::path::PathBuf;

    pub fn socket(app: &str, role: &str) -> PathBuf {
        crate::runtime::path(&format!("{app}-{role}.sock"))
    }

    /// A word to the companion — false if none listens.
    pub fn send(app: &str, role: &str, word: &str) -> bool {
        UnixDatagram::unbound().and_then(|s| s.send_to(word.as_bytes(), socket(app, role))).is_ok()
    }

    /// Become the companion: every word reaches `on` (own thread).
    pub fn listen(app: &str, role: &str, on: impl Fn(String) + Send + 'static) {
        let path = socket(app, role);
        let _ = std::fs::remove_file(&path);
        let Ok(sock) = UnixDatagram::bind(&path) else { return };
        std::thread::spawn(move || {
            // Room for a status word with lists (Tube: subtitle tracks).
            let mut buf = vec![0u8; 64 << 10];
            while let Ok(n) = sock.recv(&mut buf) {
                on(String::from_utf8_lossy(&buf[..n]).trim().to_string());
            }
        });
    }

    /// Ask the companion, or start it with `start` when none listens.
    pub fn send_or_start(app: &str, role: &str, word: &str, start: &mut std::process::Command) -> bool {
        send(app, role, word) || crate::process::spawn_reaped(start)
    }
}

/// THE ON-SCREEN KEYBOARD'S WIRE (R209 family, iced-free): one datagram to
/// matrix-keyboard's guard — "open" (a field wants it), "open!" (touch:
/// the physical-keyboard veto does not apply), "close". Errors are ignored
/// (no guard = no need). `matrixkit_widgets::keyboard` and the shell's
/// typing clients both speak through here.
pub mod onscreen_keyboard {
    /// The guard's mailbox.
    pub fn socket() -> std::path::PathBuf {
        crate::runtime::path("matrix-keyboard.sock")
    }

    /// A word to the guard.
    pub fn send(word: &str) {
        if let Ok(s) = std::os::unix::net::UnixDatagram::unbound() {
            let _ = s.send_to(word.as_bytes(), socket());
        }
    }

    /// CLAIM (R209): a popup that types, on a finger device — summon the
    /// keyboard by touch word and, with several panels, make the popup's
    /// panel (`setting`, else `duo_default`) the focused one: an exclusive
    /// keyboard surface gets keys only there (niri 0010 no longer moves the
    /// focus on shell taps). The iced side adds its field-beat permission.
    pub fn claim(setting: &str, duo_default: &str) {
        if !crate::device::finger_first() {
            return;
        }
        send("open!");
        if crate::device::panels().is_some() {
            let panel = crate::device::duo_panel(setting, duo_default);
            crate::canvas::focus_output(&panel);
        }
    }
}

pub fn transparency_reduced() -> bool {
    settings::flag("transparency", &["reduced", "reduziert"])
}

/// ColorSchemeContrast-Extrakt: ~/.config/matrix/kontrast = "hoch"
/// strengthens the palette system-wide (secondary tones move toward
/// the primaries, outlines get stronger) — via Palette::load everywhere.
pub fn contrast_up() -> bool {
    settings::flag("contrast", &["high", "hoch"])
}

/// The HABITAT of a Matrix application (R207, the owner 21.09.2026):
/// «It matters whether a program runs for Matrix or for Linux.»
/// On the canvas the compositor takes care of rounding, moving,
/// glass and fonts; on a foreign desktop all of that is missing, and
/// the app brings it along itself — like an app that switches its view
/// on a phone. Whoever adapts asks HERE, not in
/// twenty places.
pub mod habitat {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Habitat {
        /// The canvas: niri-canvas, the Matrix image lives under
        /// /usr/share/matrix.
        Matrix,
        /// Irgendein anderer Linux-Desktop (GNOME, KDE, Sway …).
        Foreign,
    }

    static HABITAT: std::sync::OnceLock<Habitat> = std::sync::OnceLock::new();

    /// Detected once, then fixed. `MATRIX_HABITAT=matrix|fremd`
    /// overrides the detection (probes on the canvas).
    pub fn habitat() -> Habitat {
        *HABITAT.get_or_init(|| {
            match std::env::var("MATRIX_HABITAT").ok().as_deref() {
                Some("matrix") => return Habitat::Matrix,
                Some("fremd") | Some("foreign") => return Habitat::Foreign,
                _ => {}
            }
            let niri = std::env::var_os("NIRI_SOCKET").is_some();
            let image = std::path::Path::new("/usr/share/matrix").is_dir();
            if niri && image {
                Habitat::Matrix
            } else {
                Habitat::Foreign
            }
        })
    }

    pub fn is_matrix() -> bool {
        habitat() == Habitat::Matrix
    }

    /// The name of the foreign desktop, for the log and for special cases.
    pub fn desktop_name() -> String {
        std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
            .unwrap_or_else(|_| String::from("unbekannt"))
    }
}

/// The DEVICE (R209): some hardware needs its own flavour of the canvas
/// that must never touch laptop, Surface or tablet. Detected once, then
/// fixed. `MATRIX_DEVICE=duo|standard` overrides the detection.
pub mod device {
    /// THE CPU'S TEMPERATURE (round 32, 02.11.2026): the hottest PROCESSOR
    /// zone in °C. Before, Settings took the hottest of ALL thermal zones —
    /// on the Duo those include limits (lmh-dcvs 75 °C), battery current/
    /// voltage channels (ibat/vbat «lvl») and step sensors, and showed 101 °C.
    pub fn cpu_temperature() -> Option<f32> {
        let mut zones = Vec::new();
        for e in std::fs::read_dir("/sys/class/thermal").ok()?.flatten() {
            let p = e.path();
            if !p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("thermal_zone")) {
                continue;
            }
            let kind = std::fs::read_to_string(p.join("type")).unwrap_or_default().trim().to_string();
            // Round 56: read the temperature only of processor zones. Reading
            // every zone woke the Duo's charger sensors (SMB138X TADC), which
            // answer «disabled by a vote» — 242 kernel errors an hour while
            // Settings was open.
            if !is_cpu_zone(&kind) {
                continue;
            }
            if let Some(milli) = std::fs::read_to_string(p.join("temp")).ok().and_then(|t| t.trim().parse::<f32>().ok()) {
                zones.push((kind, milli));
            }
        }
        cpu_temperature_of(&zones)
    }

    /// Is a thermal zone of this type a processor's (round 56: decided
    /// before its temperature is read)?
    pub fn is_cpu_zone(kind: &str) -> bool {
        let not_heat = ["lmh", "bcl", "vbat", "ibat", "lvl", "limit", "step", "lowf", "test"];
        let k = kind.to_ascii_lowercase();
        (k.contains("cpu") || k.contains("x86_pkg") || k == "acpitz" || k.contains("soc")) && !not_heat.iter().any(|n| k.contains(n))
    }

    /// The first global IPv4 address of this machine (round 56: from the
    /// kernel via getifaddrs — Settings ran `ip -4 -o addr` every 3 s).
    /// Loopback and link-local (169.254/16) do not count.
    pub fn ipv4_global() -> Option<String> {
        let mut head: *mut libc::ifaddrs = std::ptr::null_mut();
        // SAFETY: getifaddrs allocates the list; freed below with freeifaddrs.
        if unsafe { libc::getifaddrs(&mut head) } != 0 {
            return None;
        }
        let mut found = None;
        let mut cur = head;
        while !cur.is_null() {
            // SAFETY: cur walks the list getifaddrs returned.
            let ifa = unsafe { &*cur };
            if !ifa.ifa_addr.is_null() && unsafe { (*ifa.ifa_addr).sa_family } as i32 == libc::AF_INET {
                // SAFETY: an AF_INET address is a sockaddr_in.
                let sin = unsafe { &*(ifa.ifa_addr as *const libc::sockaddr_in) };
                let ip = std::net::Ipv4Addr::from(u32::from_be(sin.sin_addr.s_addr));
                if !ip.is_loopback() && !ip.is_link_local() && !ip.is_unspecified() {
                    found = Some(ip.to_string());
                    break;
                }
            }
            cur = ifa.ifa_next;
        }
        // SAFETY: the list from getifaddrs above.
        unsafe { libc::freeifaddrs(head) };
        found
    }

    /// The pure choice (tested): (zone type, millidegrees) → °C.
    pub fn cpu_temperature_of(zones: &[(String, f32)]) -> Option<f32> {
        zones
            .iter()
            .filter(|(k, _)| is_cpu_zone(k))
            .map(|(_, m)| m / 1000.0)
            .filter(|c| (5.0..=110.0).contains(c))
            .fold(None, |best: Option<f32>, c| Some(best.map_or(c, |b| b.max(c))))
    }

    #[cfg(test)]
    mod temperature_tests {
        #[test]
        fn only_processor_zones_count() {
            let z = |k: &str, m: f32| (k.to_string(), m);
            let duo = [z("cpu-1-6-usr", 31900.0), z("cpuss-0-usr", 34200.0), z("lmh-dcvs-00", 75000.0), z("cpu-1-6-step", 36500.0), z("pm8150b-ibat-lvl1", 101000.0), z("gpuss-0-usr", 29600.0)];
            assert_eq!(super::cpu_temperature_of(&duo), Some(34.2));
            let pc = [z("acpitz", 27800.0), z("x86_pkg_temp", 48000.0), z("iwlwifi_1", 40000.0)];
            assert_eq!(super::cpu_temperature_of(&pc), Some(48.0));
            assert_eq!(super::cpu_temperature_of(&[z("battery", 26000.0)]), None);
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Device {
        /// Two panels on a hinge (Surface Duo): every panel is a screen of
        /// its own, both belong to the shell.
        Duo,
        /// Everything else: laptop, tablet, desktop.
        Standard,
    }

    /// The panels of a duo device (DRM connector names).
    pub const DUO_PANELS: [&str; 2] = ["DSI-1", "DSI-2"];

    static DEVICE: std::sync::OnceLock<Device> = std::sync::OnceLock::new();

    pub fn device() -> Device {
        *DEVICE.get_or_init(|| {
            match std::env::var("MATRIX_DEVICE").ok().as_deref() {
                Some("duo") => return Device::Duo,
                Some("standard") => return Device::Standard,
                _ => {}
            }
            let model = std::fs::read_to_string("/proc/device-tree/model").unwrap_or_default();
            if model.contains("Surface Duo") {
                Device::Duo
            } else {
                Device::Standard
            }
        })
    }

    pub fn is_duo() -> bool {
        device() == Device::Duo
    }

    /// The device's several panels (DRM connectors), if it has more than
    /// one screen of its own — surfaces that belong on EVERY screen start
    /// once per panel (`mkw::panel::per_panel`). None = one screen.
    pub fn panels() -> Option<&'static [&'static str]> {
        is_duo().then_some(&DUO_PANELS[..])
    }

    /// EQUIPMENT: does this device carry Hardline (node, dock widget,
    /// settings section)? A duo does not (owner, 01.11.2026) — one
    /// answer for every place that would otherwise ask `is_duo()` itself.
    pub fn carries_hardline() -> bool {
        !is_duo()
    }

    /// FINGER FIRST: is this device worked with the finger (no pointer to
    /// hover)? Then elements answer a TAP (`mkw::tap_answer`: haptic tick +
    /// fading ring) instead of growing under a pointer (magnifier).
    pub fn finger_first() -> bool {
        is_duo()
    }

    /// Round 83: the panel IN FRONT while a duo is folded to one panel
    /// (duo-haltung writes /run/duo/single, removes it when both are open).
    pub fn duo_front_panel() -> Option<String> {
        let s = std::fs::read_to_string("/run/duo/single").ok()?;
        let s = s.trim();
        matches!(s, "DSI-1" | "DSI-2").then(|| s.to_string())
    }

    /// Round 85: how a duo is held — both panels open, or folded back with
    /// one panel in front (duo-haltung's /run/duo/single).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Fold {
        Open,
        Folded { front: String },
    }

    pub fn fold() -> Fold {
        match duo_front_panel() {
            Some(front) => Fold::Folded { front },
            None => Fold::Open,
        }
    }

    /// The panel's name for people (the bar's words): DSI-1 is the upper
    /// panel, DSI-2 the lower one.
    pub fn panel_label(name: &str) -> &'static str {
        match name {
            "DSI-1" => "oberes Panel",
            "DSI-2" => "unteres Panel",
            _ => "Bildschirm",
        }
    }

    /// Round 83: the panel a shell part lives on — the front one while the
    /// duo is folded to one panel (owner: «never both docks on one
    /// display»: dock on DSI-1, bar on DSI-2, one of them always dark), else
    /// the panel named in `setting`, else `duo_default`.
    pub fn duo_panel(setting: &str, duo_default: &str) -> String {
        desk_screen()
            .or_else(duo_front_panel)
            .unwrap_or_else(|| crate::settings::read(setting).unwrap_or_else(|| String::from(duo_default)))
    }

    /// THE DESK (round 158, 05.11.2026): a duo with a screen on USB-C works
    /// like a computer. Shut (duo-haltung's /run/duo/schreibtisch): dark like
    /// a closed laptop, everything on the screen. Folded back: the bar and
    /// the dock go to the screen, the front panel stays a second monitor.
    /// Open: the panels keep them. Settings: `desk-closed`, `desk-folded`
    /// («off» = as before).
    pub const DESK_FILE: &str = "/run/duo/schreibtisch";

    /// A screen is plugged into the USB-C (DisplayPort link up) — a cheap
    /// file read, before anything asks the compositor.
    pub fn screen_plugged() -> bool {
        std::fs::read_dir("/sys/class/drm").into_iter().flatten().flatten().any(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.contains("-DP-") && std::fs::read_to_string(e.path().join("status")).is_ok_and(|s| s.trim() == "connected")
        })
    }

    /// The duo is shut on the desk (panels dark, the screen is everything).
    pub fn desk_shut() -> bool {
        std::path::Path::new(DESK_FILE).exists()
    }

    /// Where the shell belongs, by the desk rules (pure — tested).
    pub fn desk_rule(duo: bool, plugged: bool, shut: bool, folded_back: bool, folded_wish: Option<&str>) -> bool {
        duo && plugged && (shut || (folded_back && folded_wish.map(str::trim) != Some("off")))
    }

    /// HOW THE DUO IS USED (round 159, one word for apps): on its own
    /// panels, shut on the desk (the screen is everything), or folded back
    /// with the screen as the main one and the front panel beside it.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Desk {
        /// The panels carry the shell (no screen, open, or chosen off).
        Panels,
        /// Shut like a laptop: everything on `screen`.
        Shut { screen: String },
        /// Folded back: the shell on `screen`, `front` a second monitor.
        Second { screen: String, front: String },
    }

    impl Desk {
        /// The screen the shell lives on, if not the panels.
        pub fn screen(&self) -> Option<&str> {
            match self {
                Desk::Panels => None,
                Desk::Shut { screen } | Desk::Second { screen, .. } => Some(screen),
            }
        }

        /// In words for people (the settings' «Haltung»).
        pub fn words(&self) -> String {
            match self {
                Desk::Panels => String::from("Auf den Panels"),
                Desk::Shut { .. } => String::from("Zugeklappt am Bildschirm — wie ein Laptop"),
                Desk::Second { front, .. } => format!("Zurückgeklappt — Bildschirm vorne, das {} als zweiter Monitor", panel_label(front)),
            }
        }
    }

    /// The desk from its parts (pure — tested).
    pub fn desk_from(screen: Option<String>, shut: bool, front: Option<String>, folded_wish: Option<&str>) -> Desk {
        let Some(screen) = screen else { return Desk::Panels };
        if shut {
            return Desk::Shut { screen };
        }
        match front {
            Some(front) if folded_wish.map(str::trim) != Some("off") => Desk::Second { screen, front },
            _ => Desk::Panels,
        }
    }

    /// How the duo is used right now (cheap when no screen is plugged).
    /// Round 168: kept 2 s — the bar and the dock ask on every pulse, and
    /// with a screen plugged each asked the compositor for its outputs.
    pub fn desk() -> Desk {
        use std::sync::Mutex;
        static LAST: Mutex<Option<(std::time::Instant, Desk)>> = Mutex::new(None);
        if let Ok(guard) = LAST.lock() {
            if let Some((at, d)) = guard.as_ref() {
                if at.elapsed().as_secs_f32() < 2.0 {
                    return d.clone();
                }
            }
        }
        let d = desk_now();
        if let Ok(mut guard) = LAST.lock() {
            *guard = Some((std::time::Instant::now(), d.clone()));
        }
        d
    }

    fn desk_now() -> Desk {
        if !is_duo() || !screen_plugged() {
            return Desk::Panels;
        }
        let wish = crate::settings::read("desk-folded");
        let shut = desk_shut();
        let front = duo_front_panel();
        if !desk_rule(true, true, shut, front.is_some(), wish.as_deref()) {
            return Desk::Panels;
        }
        desk_from(crate::canvas::external_screen().map(|s| s.name), shut, front, wish.as_deref())
    }

    /// The screen the bar and the dock belong on now — None: the panels.
    pub fn desk_screen() -> Option<String> {
        desk().screen().map(String::from)
    }

    #[cfg(test)]
    mod desk_tests {
        use super::desk_rule;

        #[test]
        fn the_shell_goes_to_the_screen_shut_or_folded_back() {
            assert!(desk_rule(true, true, true, false, None), "shut: everything on the screen");
            assert!(desk_rule(true, true, false, true, None), "folded back: second monitor, shell on the screen");
            assert!(!desk_rule(true, true, false, true, Some("off")), "folded back, chosen off");
            assert!(!desk_rule(true, true, false, false, None), "open: the panels keep the shell");
            assert!(!desk_rule(true, false, true, true, None), "no screen: nothing moves");
            assert!(!desk_rule(false, true, true, true, None), "not a duo");
        }

        #[test]
        fn the_desk_names_itself() {
            use super::{desk_from, Desk};
            let tv = || Some(String::from("DP-1"));
            assert_eq!(desk_from(tv(), true, None, None), Desk::Shut { screen: String::from("DP-1") });
            let second = desk_from(tv(), false, Some(String::from("DSI-2")), None);
            assert_eq!(second.screen(), Some("DP-1"));
            assert!(second.words().contains("unteres Panel"));
            assert_eq!(desk_from(tv(), false, Some(String::from("DSI-2")), Some("off")), Desk::Panels);
            assert_eq!(desk_from(tv(), false, None, None), Desk::Panels, "open: the panels");
            assert_eq!(desk_from(None, true, None, None), Desk::Panels, "the screen not yet known");
        }
    }

    /// The panel the DOCK lives on (duo; elsewhere unused): popups that
    /// belong to the dock (Matrix Start) open there too.
    pub const DUO_DOCK_PANEL: &str = "DSI-1";

    /// The panel the on-screen KEYBOARD opens on (duo): the other panel
    /// than the dock and its popups, so a search on one panel types from
    /// the other (owner, 01.11.2026).
    pub const DUO_KEYBOARD_PANEL: &str = "DSI-2";

    /// The air around every window (niri `gaps`, skel local.kdl R51) —
    /// full-panel popups keep the same air so they line up with windows.
    pub const WINDOW_GAP: f32 = 10.0;

    /// How far a full-panel popup must stay above the bottom edge on a duo:
    /// the dock's lift (10) + its snug tiles + one window gap.
    /// WHERE A CARD LIES on a panel of `size` (round 36; the top bar's app,
    /// user, control and app-list cards each computed it): one window gap
    /// from the sides, below `top` (the bar's zone) and above `bottom` (the
    /// dock's clearance without its own gap), at most 720 wide, centred.
    pub fn card_frame(size: (f32, f32), top: i32, bottom: i32) -> (i32, i32, u32, u32) {
        let gap = WINDOW_GAP as i32;
        let (ow, oh) = (size.0 as i32, size.1 as i32);
        let y = top + gap;
        let w = (ow - 2 * gap).clamp(200, 720) as u32;
        let h = (oh - y - gap - bottom.max(0)).max(160) as u32;
        let x = ((ow - w as i32) / 2).max(0);
        (x, y, w, h)
    }

    #[cfg(test)]
    mod card_frame_tests {
        #[test]
        fn a_duo_panel_card() {
            // 652×489 landscape panel, bar zone 46 above, nothing below.
            assert_eq!(super::card_frame((652.0, 489.0), 46, 0), (10, 56, 632, 423));
            // Wide screen: capped and centred.
            assert_eq!(super::card_frame((1920.0, 1080.0), 0, 0).2, 720);
            assert_eq!(super::card_frame((1920.0, 1080.0), 0, 0).0, 600);
        }
    }

    /// ARE THE PANELS ASLEEP? (round 44): the power key switched them off
    /// (duo-powerkey v4 keeps `/run/duo/aus`). Daemons skip work nobody
    /// sees — the bar's readings, a clock — until they wake.
    pub fn display_asleep() -> bool {
        std::path::Path::new("/run/duo/aus").exists()
    }

    /// WHAT THE BATTERY GIVES NOW (round 45): (milliamperes, watts) while
    /// discharging, read from the kernel (`current_now` µA, `voltage_now`
    /// µV). None on mains or without a battery. Measured on the Duo:
    /// panels on ≈ 375 mA, asleep ≈ 270 mA.
    pub fn battery_draw() -> Option<(u32, f32)> {
        let base = std::path::Path::new("/sys/class/power_supply/battery");
        let read = |f: &str| std::fs::read_to_string(base.join(f)).ok()?.trim().parse::<i64>().ok();
        let status = std::fs::read_to_string(base.join("status")).ok()?;
        if status.trim() != "Discharging" {
            return None;
        }
        let ua = read("current_now")?.unsigned_abs();
        let uv = read("voltage_now")?.unsigned_abs();
        Some(((ua / 1000) as u32, (ua as f64 * uv as f64 / 1e12) as f32))
    }

    pub fn dock_clearance() -> f32 {
        10.0 + SNUG_TILE_HEIGHT as f32 + WINDOW_GAP
    }

    /// The longest image side (px) worth holding for a full-screen picture
    /// on this device — a duo panel shows 1800 × 1350, a desktop up to 4K.
    pub fn image_long_side() -> u32 {
        if is_duo() {
            1800
        } else {
            3840
        }
    }

    /// How many characters of the focused window's title the bar shows
    /// before «…» — on a duo panel a long title ran into the clock.
    pub fn bar_title_chars() -> usize {
        if is_duo() {
            22
        } else {
            64
        }
    }

    /// The backlight that really lights this device's screen (brightnessctl
    /// device name), None = brightnessctl's first. A duo lists three: the
    /// WLED `backlight` does nothing visible, panel0 is the one (duo-blmirror
    /// mirrors it to panel1).
    pub fn backlight() -> Option<&'static str> {
        if is_duo() {
            Some("panel0-backlight")
        } else {
            None
        }
    }

    /// The backlight of a duo panel (DSI-1 → panel0, DSI-2 → panel1).
    pub fn panel_backlight(output: &str) -> Option<&'static str> {
        if !is_duo() {
            return None;
        }
        match output {
            "DSI-1" => Some("panel0-backlight"),
            "DSI-2" => Some("panel1-backlight"),
            _ => None,
        }
    }

    /// The graphical login session's D-Bus path (logind) — the session
    /// may set its own backlights without root (`SetBrightness`).
    fn login_session_path() -> String {
        let id = std::env::var("XDG_SESSION_ID").ok().filter(|s| !s.is_empty()).or_else(|| {
            let out = std::process::Command::new("loginctl").args(["list-sessions", "--no-legend"]).output().ok()?;
            let user = std::env::var("USER").unwrap_or_default();
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|l| l.split_whitespace().collect::<Vec<_>>())
                .find(|c| c.len() >= 4 && c[2] == user && c[3] == "seat0")
                .map(|c| c[0].to_string())
        });
        match id {
            // Object path escaping: a leading digit becomes _3X.
            Some(id) if id.starts_with(|c: char| c.is_ascii_digit()) => format!("/org/freedesktop/login1/session/_3{id}"),
            Some(id) => format!("/org/freedesktop/login1/session/{id}"),
            None => String::from("/org/freedesktop/login1/session/auto"),
        }
    }

    /// Set one backlight through logind (brightnessctl has no right to it
    /// on the duo — measured 02.11.2026).
    pub fn set_backlight(device: &str, value: u32) -> bool {
        std::process::Command::new("busctl")
            .args(["--system", "call", "org.freedesktop.login1"])
            .arg(login_session_path())
            .args(["org.freedesktop.login1.Session", "SetBrightness", "ssu", "backlight", device])
            .arg(value.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    /// ONE PANEL DARK (R210 family, the owner 02.11.2026): a duo panel only
    /// watched over (Tube's list while the video plays) gives up its light;
    /// touch still works, `panel_light_on` brings it back. The posture
    /// service duo-haltung owns the panels (and copies panel0's brightness
    /// onto panel1 four times a second — setting it directly flickered back
    /// at once), so this only ASKS: the runtime file `matrix-panel-ruhe`
    /// names the output, duo-haltung switches bl_power. Only the second panel:
    /// duo-blmirror copies panel0's bl_power onto panel1. False = not asked.
    pub fn panel_light_off(output: &str) -> bool {
        // With our pid: a crashed asker no longer holds the panel dark.
        panel_backlight(output) == Some("panel1-backlight") && crate::runtime::write("matrix-panel-ruhe", &format!("{output} {}", std::process::id()))
    }

    /// The panel's light back (the request withdrawn).
    pub fn panel_light_on() {
        crate::runtime::remove("matrix-panel-ruhe");
    }

    /// FOLDED (R210, 02.11.2026): the duo lies closed (duo-haltung keeps
    /// /run/duo/zugeklappt while it sleeps). Nobody sees a picture — sound
    /// may play on (headphones), pictures need not be made.
    pub fn folded() -> bool {
        std::path::Path::new("/run/duo/zugeklappt").exists()
    }

    /// BRIGHTNESS FAMILY: brightnessctl arguments aimed at the backlight
    /// that really lights the screen (`backlight()`), else the first of class
    /// backlight; `rest` follows (`-m`, `set 40%`, `set +10%` …).
    pub fn brightnessctl_args<'a>(rest: &[&'a str]) -> Vec<&'a str> {
        let mut args = match backlight() {
            Some(device) => vec!["-d", device],
            None => vec!["--class=backlight"],
        };
        args.extend_from_slice(rest);
        args
    }

    /// BAR AS CHIPS (R209, the owner): on a duo the top bar is no slab across
    /// the panel but a row of glass chips like the dock tiles — same thin
    /// height, every widget its own surface. Elsewhere the bar stays whole.
    pub fn bar_chips() -> bool {
        is_duo()
    }

    /// The BAR's default assignment (three zones split by `|`) when the user
    /// has none in `bar-widgets`. A duo panel has no room for the pulse
    /// (CPU · memory · sound · net), the owner 01.11.2026.
    pub fn bar_default() -> &'static str {
        // Duo (owner, 01.11.): no Matrix chip at the far left — «Apps»
        // leads the dock on the other panel. The clock heads the right
        // group: centered it collided with it on the 654 pt panel.
        if is_duo() {
            "focus | | clock battery tray media bell central user"
        } else {
            "matrix focus | clock | battery pulse tray media bell central user"
        }
    }

    /// The WINDOW LIGHTS (close / shelf / maximize) of this device: their
    /// size in logical px and whether the glyphs (× − +) show without a
    /// pointer hovering. A finger needs a bigger target and has no hover
    /// (owner, Duo, 01.11.2026); elsewhere 14 pt (R168g), glyphs on hover.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Lights {
        pub size: f32,
        pub glyphs: bool,
        /// The middle light «put on the shelf». A duo has none: there the
        /// app's own dock tile shelves it on a second tap (the owner).
        pub shelf: bool,
    }

    pub fn lights() -> Lights {
        if is_duo() {
            Lights { size: 24.0, glyphs: true, shelf: false }
        } else {
            Lights { size: 14.0, glyphs: false, shelf: true }
        }
    }

    /// The SHELF form (dock) of this device — every duo rule of the tile
    /// shelf in one place, so the dock asks once instead of eight times.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Shelf {
        /// Tile height in logical px; None = level with two widget rows
        /// (R172k, laptop/Surface).
        pub tile_height: Option<u32>,
        /// One caption line per open window (the tile grows).
        pub window_lines: bool,
        /// Glass grows under the pointer.
        pub magnifier: bool,
        /// «Apps» leads the left fleet as a tile among tiles.
        pub apps_leads: bool,
        /// Tapping the tile of the app in front puts its window on the
        /// shelf (minimize) — the duo's window has no shelf light.
        pub tap_shelves: bool,
    }

    /// A tile that hugs its content: 2 × padding S, title (SUBTITLE
    /// 16 × 1.3) and maker (CAPTION 12 × 1.3).
    pub const SNUG_TILE_HEIGHT: u32 = 56;

    pub fn shelf() -> Shelf {
        if is_duo() {
            Shelf { tile_height: Some(SNUG_TILE_HEIGHT), window_lines: false, magnifier: false, apps_leads: true, tap_shelves: true }
        } else {
            Shelf { tile_height: None, window_lines: true, magnifier: true, apps_leads: false, tap_shelves: false }
        }
    }

    /// A short haptic tick (`ms` milliseconds) on duo devices — through the
    /// duo-haptik helper (`/run/duo/haptik.sock`, one datagram = the length).
    /// Elsewhere, or without the helper, nothing happens.
    pub fn haptic(ms: u32) {
        if !is_duo() {
            return;
        }
        if let Ok(socket) = std::os::unix::net::UnixDatagram::unbound() {
            let _ = socket.send_to(ms.to_string().as_bytes(), "/run/duo/haptik.sock");
        }
    }
}

/// The FORM of the window (R207): how wide is the surface, and how
/// does the app behave in it? A window on a foreign desktop
/// never gets smaller than a smartphone display (`PHONE_W` × `PHONE_H`,
/// logical) — and at this width the app behaves as on the
/// phone: one column, tools in two rows, no side menu.
/// The stages are widths, not devices: the same window changes
/// its form when the owner drags it.
pub mod form {
    /// Ein Smartphone-Display in logischen Pixeln (Android-Grundmass).
    pub const PHONE_W: f32 = 360.0;
    pub const PHONE_H: f32 = 640.0;
    /// Up to here it is a phone, up to `TABLET_MAX` a tablet.
    pub const PHONE_MAX: f32 = 600.0;
    pub const TABLET_MAX: f32 = 960.0;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Form {
        Phone,
        Tablet,
        Desktop,
    }

    /// Die Form zu einer Breite.
    pub fn of(width: f32) -> Form {
        if width < PHONE_MAX {
            Form::Phone
        } else if width < TABLET_MAX {
            Form::Tablet
        } else {
            Form::Desktop
        }
    }

    /// The smallest window size (logical) that the house reports to the
    /// compositor in the host land.
    pub fn least() -> (u32, u32) {
        (PHONE_W as u32, PHONE_H as u32)
    }

    /// The width of this app's window, as its frame last saw it (R209):
    /// families that cannot be handed a width (row_value with 27 callers)
    /// ask here. 0 = not known yet -> desktop.
    static WINDOW_WIDTH: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    /// The frame reports the window width (logical px).
    pub fn set_window_width(width: f32) {
        WINDOW_WIDTH.store(width.max(0.0).to_bits(), std::sync::atomic::Ordering::Relaxed);
    }

    /// The form of this app's window (desktop until the frame has measured).
    pub fn current() -> Form {
        let width = f32::from_bits(WINDOW_WIDTH.load(std::sync::atomic::Ordering::Relaxed));
        if width > 0.0 { of(width) } else { Form::Desktop }
    }

    /// How many columns of at least `min_col` width fit into `width` with
    /// `gap` spacing? Never fewer than one.
    pub fn columns(width: f32, min_col: f32, gap: f32) -> usize {
        let n = ((width + gap) / (min_col + gap)).floor();
        (n as usize).max(1)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_form_follows_the_width() {
            assert_eq!(of(PHONE_W), Form::Phone);
            assert_eq!(of(599.0), Form::Phone);
            assert_eq!(of(600.0), Form::Tablet);
            assert_eq!(of(1200.0), Form::Desktop);
        }

        #[test]
        fn columns_never_vanish() {
            assert_eq!(columns(300.0, 240.0, 16.0), 1);
            assert_eq!(columns(496.0, 240.0, 16.0), 2);
            assert_eq!(columns(100.0, 240.0, 16.0), 1);
        }
    }
}

pub fn motion_reduced() -> bool {
    settings::flag("motion", &["reduced", "reduziert"])
}

/// App PermissionSet — the BINDING permission management of MatrixKit.
///
/// Every MatrixKit app obtains system access exclusively through the
/// frame; the frame checks these permissions BEFORE every access. A
/// disabled switch means: the app CANNOT access — not "should not".
/// Storage: ~/.config/matrix/permissions/<app>.conf (legacy: berechtigungen/)
pub mod permissions {
    use std::path::PathBuf;
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Permission {
        Network,
        Clipboard,
        Camera,
        Microphone,
        FilesRead,
        FilesWrite,
        /// R177 (the owner): may foreign content use the device's
        /// CONTROLS? Concerns Hardtext modules with a granted
        /// drawing right: with this right they look like the system,
        /// without it they stay in their own frame. Default OFF —
        /// unlike the other rights, because here foreign content, and
        /// not the app itself, asks for access.
        SystemWidgets,
    }

    impl Permission {
        pub fn key(self) -> &'static str {
            match self {
                Permission::Network => "network",
                Permission::Clipboard => "clipboard",
                Permission::Camera => "camera",
                Permission::Microphone => "microphone",
                Permission::FilesRead => "files-read",
                Permission::FilesWrite => "files-write",
                Permission::SystemWidgets => "system-widgets",
            }
        }
        pub fn display_name(self) -> &'static str {
            match self {
                Permission::Network => "Netzwerk",
                Permission::Clipboard => "Zwischenablage",
                Permission::Camera => "Kamera",
                Permission::Microphone => "Mikrofon",
                Permission::FilesRead => "Dateien lesen",
                Permission::FilesWrite => "Dateien schreiben",
                Permission::SystemWidgets => "Systemelemente für fremde Inhalte",
            }
        }
    }

    /// Rechtestand einer App. Fehlender Eintrag = erlaubt (Opt-out-Modell;
    /// the app only queries permissions it actually uses anyway).
    #[derive(Clone, Debug)]
    pub struct PermissionSet {
        app_id: String,
        denied: Vec<Permission>,
    }

    fn file(app_id: &str) -> PathBuf {
        // R190: an explicit seam instead of a global environment.
        // Tests that bend HOME pull the floor out from under all other tests in
        // the same process — exactly that happened to the canon test on
        // 04.08.2026. This variable belongs to the test,
        // not to the system.
        if let Ok(dir) = std::env::var("MATRIX_PERMISSIONS_DIR") {
            return PathBuf::from(dir).join(format!("{app_id}.conf"));
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        PathBuf::from(home).join(format!(".config/matrix/permissions/{app_id}.conf"))
    }

    /// Legacy German permission keys → current English keys (R101e).
    const OLD_KEYS: &[(&str, Permission)] = &[
        ("netzwerk", Permission::Network),
        ("zwischenablage", Permission::Clipboard),
        ("kamera", Permission::Camera),
        ("mikrofon", Permission::Microphone),
        ("dateien-lesen", Permission::FilesRead),
        ("dateien-schreiben", Permission::FilesWrite),
        ("systemelemente", Permission::SystemWidgets),
    ];

    /// What a right is worth before anyone was asked. Everything is
    /// granted by default — except handing the device's own controls
    /// to foreign content, which nobody gets without being asked.
    fn default_word(r: Permission) -> &'static str {
        match r {
            Permission::SystemWidgets => "off",
            _ => "on",
        }
    }

    fn write_atomically(path: &std::path::Path, content: &str) {
        let tmp = path.with_extension("conf.new");
        if std::fs::write(&tmp, content).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }

    impl PermissionSet {
        pub fn load(app_id: &str) -> Self {
            let mut denied = Vec::new();
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            let legacy = PathBuf::from(&home)
                // R101 legacy path — pre-migration permission files.
                .join(format!(".config/matrix/berechtigungen/{app_id}.conf"));
            let content = std::fs::read_to_string(file(app_id))
                .or_else(|_| std::fs::read_to_string(legacy))
                .unwrap_or_default();
            for line in content.lines() {
                if let Some((k, w)) = line.split_once('=') {
                    if matches!(w.trim(), "off" | "aus") {
                        let k = k.trim();
                        for r in ALL {
                            if r.key() == k {
                                denied.push(*r);
                            }
                        }
                        for (ok, r) in OLD_KEYS {
                            if *ok == k {
                                denied.push(*r);
                            }
                        }
                    }
                }
            }
            denied.dedup();
            Self { app_id: app_id.into(), denied }
        }

        /// THE checkpoint: the frame calls it before every access.
        pub fn allowed(&self, r: Permission) -> bool {
            !self.denied.contains(&r)
        }

        /// R177: every harness app registers itself in the rights list
        /// at start — otherwise it does not show up in the settings
        /// under „Apps und Rechte" at all, as long as nobody
        /// has ever touched a switch (exactly the gap that
        /// The owner found on 03.08.2026).
        ///
        /// If the file is missing, it is created. `SystemWidgets` starts
        /// as DENIED: foreign content only gets the device's
        /// controls when the user says so.
        ///
        /// R179: the file records EVERY permission explicitly,
        /// `on` as well as `off`. Otherwise "not listed" is ambiguous —
        /// it means either "granted" or "never asked", and the
        /// default of a permission added later never
        /// reaches an existing file. That is exactly how
        /// `system-widgets` was ON for Rabbit without the owner ever
        /// having been asked (found on 04.08.2026).
        pub fn register(app_id: &str, relevant: &[Permission]) {
            let path = file(app_id);
            let existing = std::fs::read_to_string(&path).unwrap_or_default();
            let known: Vec<&str> = existing
                .lines()
                .filter_map(|l| l.split_once('='))
                .map(|(k, _)| k.trim())
                .collect();
            let missing: Vec<&Permission> = relevant
                .iter()
                .filter(|r| !known.contains(&r.key()))
                .collect();
            if path.exists() && missing.is_empty() {
                return;
            }
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let mut content = if existing.is_empty() {
                String::from(
                    "# MatrixKit permission set — managed via the app name in the title bar\n",
                )
            } else {
                existing
            };
            for r in missing {
                content.push_str(&format!("{}={}\n", r.key(), default_word(*r)));
            }
            write_atomically(&path, &content);
        }

        /// Set a permission and persist atomically right away. Every
        /// other key in the file survives untouched — a decision the
        /// user once made must not fall out because some other code
        /// path rewrote the file from a partial view.
        pub fn set(&mut self, r: Permission, allowed: bool) {
            self.denied.retain(|v| *v != r);
            if !allowed {
                self.denied.push(r);
            }
            let path = file(&self.app_id);
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let existing = std::fs::read_to_string(&path).unwrap_or_default();
            let mut content = String::from(
                "# MatrixKit permission set — managed via the app name in the title bar\n",
            );
            for line in existing.lines() {
                let Some((k, _)) = line.split_once('=') else {
                    continue;
                };
                // The legacy spelling means the same right — it must
                // go too, or it keeps outvoting the new decision.
                let same = k.trim() == r.key()
                    || OLD_KEYS.iter().any(|(ok, v)| *ok == k.trim() && *v == r);
                if !same {
                    content.push_str(line);
                    content.push('\n');
                }
            }
            content.push_str(&format!(
                "{}={}\n",
                r.key(),
                if allowed { "on" } else { "off" }
            ));
            write_atomically(&path, &content);
        }
    }

    #[cfg(test)]
    mod register_tests {
        use super::*;

        /// Uses the explicit seam, NOT HOME: bending HOME would break
        /// every other test in this process (it did, on 04.08.2026).
        #[test]
        fn an_old_file_still_learns_a_new_right() {
            let dir = std::env::temp_dir().join("mk-permissions-test");
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            std::env::set_var("MATRIX_PERMISSIONS_DIR", &dir);

            // The state Rabbit was actually in: a file written before
            // SystemWidgets existed, so the key is simply absent.
            std::fs::write(dir.join("probe.conf"), "# header\nnetwork=on\n").unwrap();

            PermissionSet::register("probe", &[Permission::Network, Permission::SystemWidgets]);
            let set = PermissionSet::load("probe");
            assert!(set.allowed(Permission::Network), "an old decision survives");
            assert!(
                !set.allowed(Permission::SystemWidgets),
                "a right added later must arrive at its default, not at 'granted'"
            );

            // And once the user says yes, saying yes must stick —
            // register() may not talk them out of it on the next start.
            let mut set = PermissionSet::load("probe");
            set.set(Permission::SystemWidgets, true);
            PermissionSet::register("probe", &[Permission::Network, Permission::SystemWidgets]);
            assert!(
                PermissionSet::load("probe").allowed(Permission::SystemWidgets),
                "a granted right must not be reset by the next registration"
            );

            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    /// Confirmation with the account password (PAM via `sudo -Skv`).
    /// Blocks until the answer (a wrong password is delayed ~2 s by PAM) —
    /// apps call this via Task::perform in the background. The sudo
    /// timestamp is discarded immediately afterwards (-K): the check
    /// leaves NO elevated privileges behind.
    pub fn password_verify(password: &str) -> bool {
        use std::io::Write;
        let Ok(mut child) = std::process::Command::new("sudo")
            .args(["-S", "-k", "-v"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        else {
            return false;
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = writeln!(stdin, "{password}");
        }
        let ok = child.wait().map(|s| s.success()).unwrap_or(false);
        let _ = std::process::Command::new("sudo")
            .arg("-K")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        ok
    }

    pub const ALL: &[Permission] = &[
        Permission::Network,
        Permission::Clipboard,
        Permission::Camera,
        Permission::Microphone,
        Permission::FilesRead,
        Permission::FilesWrite,
        Permission::SystemWidgets,
    ];
}

/// The canvas facade (R45): ONE place in the whole kit knows the
/// compositor. Today that substrate is our niri fork "Leinwand" —
/// apps and services speak only this Matrix vocabulary. This is
/// decoupling step 1: if the substrate changes (hard fork,
/// homegrown Smithay), exactly this file changes.
/// Transitional alias — existing call sites migrate crate by crate
/// (code-language doctrine, 20.7.2026: code is English; the canvas is
/// only called "Leinwand" where humans read it).

pub mod canvas {
    use std::process::{Child, Command, Output, Stdio};

    /// The compositor binary name — ONLY here and in the greeter command.
    pub const BINARY: &str = "niri";

    fn runtime_dir() -> String {
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into())
    }

    /// Is this process inside a canvas session? (Anchor for services
    /// like the watch service.)
    pub fn anchor() -> Option<String> {
        std::env::var("NIRI_SOCKET").ok()
    }

    /// Socket of the running canvas: environment first, otherwise a
    /// search in the runtime directory (deploy paths without session env).
    pub fn socket() -> Option<String> {
        if let Some(s) = anchor() {
            return Some(s);
        }
        std::fs::read_dir(runtime_dir()).ok()?.filter_map(Result::ok).find_map(|e| {
            let n = e.file_name().to_str()?.to_string();
            (n.starts_with("niri.") && n.ends_with(".sock"))
                .then(|| e.path().to_string_lossy().into_owned())
        })
    }

    fn command(args: &[&str]) -> Command {
        let mut c = Command::new(BINARY);
        if let Some(s) = socket() {
            c.env("NIRI_SOCKET", s);
        }
        c.args(args);
        c
    }

    /// Raw compositor call with output — for the kit itself and the
    /// few special cases (guardian choreographies).
    pub fn raw(args: &[&str]) -> Option<Output> {
        command(args).output().ok()
    }

    /// One layer surface the compositor holds (R215 round 23).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Layer {
        pub namespace: String,
        pub output: String,
        /// Background, Bottom, Top or Overlay.
        pub layer: String,
    }

    /// All layer surfaces now (None: the compositor did not answer).
    pub fn layers() -> Option<Vec<Layer>> {
        let out = raw(&["msg", "-j", "layers"])?;
        if !out.status.success() {
            return None;
        }
        layers_from(&serde_json::from_slice(&out.stdout).ok()?)
    }

    /// Pure part of `layers` (tested).
    pub fn layers_from(v: &serde_json::Value) -> Option<Vec<Layer>> {
        Some(
            v.as_array()?
                .iter()
                .map(|l| {
                    let s = |k: &str| l.get(k).and_then(|x| x.as_str()).unwrap_or_default().to_string();
                    Layer { namespace: s("namespace"), output: s("output"), layer: s("layer") }
                })
                .collect(),
        )
    }

    /// A screen's logical size by name (round 27; None: unknown).
    pub fn output_size(name: &str) -> Option<(f32, f32)> {
        // Round 61: remembered for 30 s (one `niri msg` cost ~80 ms and ran
        // on every card opening); a turn of the Duo rewrites its outputs
        // file, and a changed stamp asks anew.
        use std::sync::Mutex;
        static CACHE: Mutex<Option<(std::time::Instant, Option<std::time::SystemTime>, serde_json::Value)>> = Mutex::new(None);
        let stamp = std::fs::metadata("/run/duo/niri-ausgaenge.kdl").and_then(|m| m.modified()).ok();
        let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
        let fresh = cache.as_ref().is_some_and(|(at, st, _)| at.elapsed().as_secs() < 30 && *st == stamp);
        if !fresh {
            let out = raw(&["msg", "-j", "outputs"])?;
            let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
            *cache = Some((std::time::Instant::now(), stamp, v));
        }
        let v = &cache.as_ref()?.2;
        let l = v.get(name)?.get("logical")?;
        Some((l.get("width")?.as_f64()? as f32, l.get("height")?.as_f64()? as f32))
    }

    /// IS IT ON SCREEN (round 23): a surface process may live and still
    /// show nothing — the context menu took requests and opened no card
    /// for hours (02.11.2026). Some(false) = the compositor has no such
    /// layer; None = it could not say (then do not act on doubt).
    pub fn layer_present(namespace: &str) -> Option<bool> {
        layers().map(|all| all.iter().any(|l| l.namespace == namespace))
    }

    #[cfg(test)]
    mod layer_tests {
        #[test]
        fn layers_parse_and_find() {
            let v: serde_json::Value = serde_json::from_str(
                r#"[{"namespace":"matrix-card","output":"DSI-2","layer":"Overlay","keyboard_interactivity":"None"},
                    {"namespace":"matrix-keyboard","output":"DSI-2","layer":"Top"}]"#,
            )
            .unwrap();
            let all = super::layers_from(&v).unwrap();
            assert_eq!(all.len(), 2);
            assert_eq!((all[0].namespace.as_str(), all[0].output.as_str(), all[0].layer.as_str()), ("matrix-card", "DSI-2", "Overlay"));
            assert!(all.iter().any(|l| l.namespace == "matrix-keyboard"));
            assert!(super::layers_from(&serde_json::json!({"x": 1})).is_none());
        }
    }

    /// Fire-and-forget action.
    /// SCREEN TRANSITION (round 135): niri keeps the picture on the screen
    /// now and blends it into whatever comes next (after `delay_ms`) — a
    /// lock opening, a window changing its form. One soft change instead
    /// of a cut.
    pub fn screen_transition(delay_ms: u32) -> bool {
        action(&["msg", "action", "do-screen-transition", "--delay-ms", &delay_ms.to_string()])
    }

    pub fn action(args: &[&str]) -> bool {
        command(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    // ------------------------------------------------- the vocabulary

    /// End the session → back to the greeter. `confirm_prompt` leaves the
    /// compositor its own confirmation.
    pub fn logout(confirm_prompt: bool) {
        if confirm_prompt {
            action(&["msg", "action", "quit"]);
        } else {
            action(&["msg", "action", "quit", "--skip-confirmation"]);
        }
    }

    /// Focus a window — the canvas drives it fully into view.
    pub fn window_focus(id: u64) {
        action(&["msg", "action", "focus-window", "--id", &id.to_string()]);
    }

    /// R190: fullscreen on/off — the way into the room. Deliberately NOT
    /// "close all other windows": a fullscreen window
    /// covers them, and covering is reversible, closing is not.
    /// The bars stay visible (the layer shell lies on top) — exactly
    /// Apple's `ornament`: controls outside the content that
    /// stay while the content takes the room.
    pub fn window_fullscreen(id: u64) {
        action(&["msg", "action", "fullscreen-window", "--id", &id.to_string()]);
    }

    pub fn window_close(id: u64) {
        action(&["msg", "action", "close-window", "--id", &id.to_string()]);
    }

    /// Move a floating window relatively (web docking coupling).
    pub fn window_move(id: u64, dx: i64, dy: i64) {
        action(&[
            "msg", "action", "move-floating-window", "--id", &id.to_string(),
            "-x", &format!("{dx:+}"), "-y", &format!("{dy:+}"),
        ]);
    }

    pub fn window_width(id: u64, w: i64) {
        action(&["msg", "action", "set-window-width", "--id", &id.to_string(), &w.to_string()]);
    }

    pub fn window_height(id: u64, h: i64) {
        action(&["msg", "action", "set-window-height", "--id", &id.to_string(), &h.to_string()]);
    }

    /// BRING A WINDOW (R209 family): focus window `id` — and if it lies on
    /// the shelf (minimized), first fetch it to the ACTIVE workspace, so the
    /// shelf itself never comes into view. Before, the dock focused shelved
    /// windows plainly, which turned the shelf workspace visible (on a duo:
    /// the whole upper panel became the shelf, and every later «minimize»
    /// landed in plain sight). With several screens the shelf lives on one of
    /// them: then the window moves to the active screen (its active
    /// workspace); on one screen it moves by workspace index as before.
    pub fn window_bring(id: u64) {
        let parse = |args: &[&str]| -> Vec<serde_json::Value> {
            raw(args)
                .and_then(|out| serde_json::from_slice(&out.stdout).ok())
                .unwrap_or_default()
        };
        let windows = parse(&["msg", "-j", "windows"]);
        let workspaces = parse(&["msg", "-j", "workspaces"]);
        let window = windows.iter().find(|w| w.get("id").and_then(|i| i.as_u64()) == Some(id));
        let shelf = workspaces.iter().find(|w| w.get("name").and_then(|n| n.as_str()) == Some("shelf"));
        let active = workspaces.iter().find(|w| w.get("is_focused").and_then(|b| b.as_bool()) == Some(true));
        if let (Some(window), Some(shelf), Some(active)) = (window, shelf, active) {
            let shelved = window.get("workspace_id").is_some() && window.get("workspace_id") == shelf.get("id");
            if shelved && shelf.get("id") != active.get("id") {
                let output = |w: &serde_json::Value| w.get("output").and_then(|o| o.as_str()).map(String::from);
                match (output(shelf), output(active)) {
                    (Some(from), Some(to)) if from != to => {
                        action(&["msg", "action", "move-window-to-monitor", "--id", &id.to_string(), &to]);
                    }
                    _ => {
                        if let Some(idx) = active.get("idx").and_then(|v| v.as_u64()) {
                            action(&[
                                "msg", "action", "move-window-to-workspace",
                                "--window-id", &id.to_string(), "--focus", "false", &idx.to_string(),
                            ]);
                        }
                    }
                }
            }
        }
        window_focus(id);
    }

    /// Make `output` the focused screen (R209 family). A surface that takes
    /// the keyboard exclusively gets it only on the focused screen — and a
    /// tap on a shell surface no longer moves that focus (niri 0010), so a
    /// popup that types (Matrix Start on a duo) claims its screen itself.
    pub fn focus_output(output: &str) {
        action(&["msg", "action", "focus-monitor", output]);
    }

    /// A SCREEN (round 156): what the canvas says about one output.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Screen {
        pub name: String,
        /// The make and model (a TV says «Panasonic …»).
        pub label: String,
        /// The native mode size (even, as encoders want) and its turn.
        pub width: u32,
        pub height: u32,
        pub transform: String,
        /// Logical size and scale; None = switched off.
        pub logical: Option<(u32, u32)>,
        pub scale: f32,
        /// Round 171: where it lies in the canvas (logical px, top left).
        pub pos: (i32, i32),
    }

    impl Screen {
        /// Plugged in from outside (a TV or monitor on USB-C/HDMI) — not a
        /// built-in panel (DSI, eDP, LVDS) nor the duo's write-back.
        pub fn external(&self) -> bool {
            !["DSI", "eDP", "LVDS", "Virtual"].iter().any(|p| self.name.starts_with(p))
        }

        pub fn on(&self) -> bool {
            self.logical.is_some()
        }
    }

    /// The screens in `niri msg -j outputs` (sorted by name).
    pub fn parse_screens(json: &[u8]) -> Vec<Screen> {
        let Ok(all) = serde_json::from_slice::<serde_json::Value>(json) else { return Vec::new() };
        let Some(map) = all.as_object() else { return Vec::new() };
        let mut out: Vec<Screen> = map
            .iter()
            .map(|(name, o)| {
                let mode = o.get("current_mode").and_then(|m| m.as_u64()).and_then(|m| o.get("modes")?.get(m as usize));
                let dim = |k: &str| mode.and_then(|m| m.get(k)).and_then(|v| v.as_u64()).unwrap_or(0) as u32 & !1;
                let l = o.get("logical").filter(|l| !l.is_null());
                let text = |k: &str| o.get(k).and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                Screen {
                    name: name.clone(),
                    label: format!("{} {}", text("make"), text("model")).trim().to_string(),
                    width: dim("width"),
                    height: dim("height"),
                    transform: l.and_then(|l| l.get("transform")).and_then(|t| t.as_str()).unwrap_or("normal").to_string(),
                    logical: l.map(|l| (l.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32, l.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32)),
                    scale: l.and_then(|l| l.get("scale")).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                    pos: (
                        l.and_then(|l| l.get("x")).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        l.and_then(|l| l.get("y")).and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    ),
                }
            })
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    /// All screens the canvas knows, on or off.
    pub fn screens() -> Vec<Screen> {
        raw(&["msg", "-j", "outputs"]).map(|o| parse_screens(&o.stdout)).unwrap_or_default()
    }

    /// The screens switched on right now (names).
    pub fn outputs() -> Vec<String> {
        screens().into_iter().filter(Screen::on).map(|s| s.name).collect()
    }

    /// A screen plugged in from outside and switched on (a TV), if any.
    pub fn external_screen() -> Option<Screen> {
        screens().into_iter().find(|s| s.on() && s.external())
    }

    /// The duo's two panels are open, switched on and alike (size and
    /// turn): then a photo or a film of «the screen» takes both (round 151).
    pub fn panels_alike() -> bool {
        if crate::device::panels().is_none() || !matches!(crate::device::fold(), crate::device::Fold::Open) {
            return false;
        }
        let all = screens();
        let get = |n: &str| all.iter().find(|s| s.name == n && s.on()).map(|s| (s.width, s.height, s.transform.clone()));
        matches!((get("DSI-1"), get("DSI-2")), (Some(a), Some(b)) if a == b)
    }

    /// PHOTO OF SCREENS (round 156, from Matrix Aufnahme): `names` taken as
    /// they show (grim) and stacked ONE ABOVE THE OTHER (ffmpeg vstack) into
    /// the PNG `target` — the duo's photo of both panels, like its film. One
    /// name: a plain photo of that screen.
    pub fn photo_stacked(names: &[&str], target: &std::path::Path) -> bool {
        if names.is_empty() {
            return false;
        }
        if names.len() == 1 {
            return std::process::Command::new("grim").args(["-o", names[0]]).arg(target).status().is_ok_and(|s| s.success());
        }
        let dir = std::env::var("XDG_RUNTIME_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| std::env::temp_dir());
        let parts: Vec<std::path::PathBuf> = names.iter().map(|o| dir.join(format!("matrix-foto-{o}.ppm"))).collect();
        let took = names.iter().zip(&parts).all(|(o, p)| std::process::Command::new("grim").args(["-t", "ppm", "-o", o]).arg(p).status().is_ok_and(|s| s.success()));
        let mut ffmpeg = std::process::Command::new("ffmpeg");
        ffmpeg.args(["-v", "error", "-y"]);
        for p in &parts {
            ffmpeg.arg("-i").arg(p);
        }
        let ok = took
            && ffmpeg
                .args(["-filter_complex", &format!("vstack=inputs={}", parts.len()), "-frames:v", "1"])
                .arg(target)
                .status()
                .is_ok_and(|s| s.success());
        for p in &parts {
            let _ = std::fs::remove_file(p);
        }
        ok
    }

    #[cfg(test)]
    mod screen_tests {
        use super::*;

        const NIRI: &str = r#"{
          "DSI-1": {"make":"PNP(QCM)","model":"0x0001","modes":[{"width":1350,"height":1800,"refresh_rate":21932}],"current_mode":0,
                    "logical":{"x":0,"y":0,"width":654,"height":490,"scale":2.75,"transform":"90"}},
          "DP-1":  {"make":"Panasonic Industry Company","model":"Panasonic-TV","modes":[{"width":3840,"height":2160,"refresh_rate":30000},{"width":1920,"height":1080,"refresh_rate":60000}],"current_mode":1,
                    "logical":{"x":1310,"y":0,"width":1280,"height":720,"scale":1.5,"transform":"Normal"}},
          "Virtual-1": {"make":"Unknown","model":"Unknown","modes":[{"width":4096,"height":2160,"refresh_rate":60000}],"current_mode":null,"logical":null}
        }"#;

        #[test]
        fn a_tv_counts_as_external_screen() {
            let all = parse_screens(NIRI.as_bytes());
            assert_eq!(all.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), vec!["DP-1", "DSI-1", "Virtual-1"]);
            let tv = &all[0];
            assert!(tv.external() && tv.on());
            assert_eq!((tv.width, tv.height, tv.scale, tv.logical), (1920, 1080, 1.5, Some((1280, 720))));
            assert_eq!(tv.label, "Panasonic Industry Company Panasonic-TV");
            let panel = &all[1];
            assert!(!panel.external() && panel.on());
            assert_eq!((panel.width, panel.height, panel.transform.as_str()), (1350, 1800, "90"));
            let wb = &all[2];
            assert!(!wb.on() && !wb.external(), "the write-back is neither on nor a screen to show");
            assert!(parse_screens(b"not json").is_empty());
        }
    }

    /// The screen that holds the focus (niri output name).
    pub fn focused_output() -> Option<String> {
        let out = raw(&["msg", "-j", "focused-output"])?;
        let value: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
        value.get("name").and_then(|n| n.as_str()).map(String::from)
    }

    /// One window of an app, with the panel it lies on.
    #[derive(Clone, Debug, PartialEq)]
    pub struct AppWindow {
        pub id: u64,
        pub pid: u32,
        pub output: String,
    }

    /// THE APP'S WINDOWS (R210 family, duo grammar): every window of
    /// `app_id` and its output (window → workspace → output).
    pub fn app_windows(app_id: &str) -> Vec<AppWindow> {
        let parse = |args: &[&str]| -> Option<serde_json::Value> { serde_json::from_slice(&raw(args)?.stdout).ok() };
        let (Some(windows), Some(spaces)) = (parse(&["msg", "-j", "windows"]), parse(&["msg", "-j", "workspaces"])) else {
            return Vec::new();
        };
        let output_of = |ws: u64| -> String {
            spaces
                .as_array()
                .and_then(|a| a.iter().find(|w| w.get("id").and_then(|i| i.as_u64()) == Some(ws)))
                .and_then(|w| w.get("output").and_then(|o| o.as_str()))
                .unwrap_or_default()
                .to_string()
        };
        windows
            .as_array()
            .map(|a| {
                a.iter()
                    .filter(|w| w.get("app_id").and_then(|i| i.as_str()) == Some(app_id))
                    .filter_map(|w| {
                        Some(AppWindow {
                            id: w.get("id")?.as_u64()?,
                            pid: w.get("pid").and_then(|p| p.as_u64()).unwrap_or(0) as u32,
                            output: output_of(w.get("workspace_id").and_then(|i| i.as_u64()).unwrap_or(0)),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The focused window with its app: (window, app_id) — the top bar's
    /// app card acts on the app it belongs to.
    pub fn focused_app_window() -> Option<(AppWindow, String)> {
        let v: serde_json::Value = serde_json::from_slice(&raw(&["msg", "-j", "focused-window"])?.stdout).ok()?;
        let app_id = v.get("app_id")?.as_str()?.to_string();
        let id = v.get("id")?.as_u64()?;
        let window = app_windows(&app_id).into_iter().find(|w| w.id == id)?;
        Some((window, app_id))
    }

    /// SCREENS OFF (R210 family): niri switches every output off (DPMS);
    /// any touch or key brings them back. The sleep timer's last step —
    /// a lock is not taken here (the session lock is untested on a duo).
    pub fn screens_off() -> bool {
        action(&["msg", "action", "power-off-monitors"])
    }

    /// FIT THE OWN WINDOW (R210 family): this process's window of `app_id`
    /// becomes floating, exactly `w` × `h` logical px, centred — a video
    /// stage the size of its picture, no black bars (owner 01.11.2026).
    pub fn fit_own_window(app_id: &str, w: u32, h: u32) -> bool {
        let Some(window) = own_window(app_id) else { return false };
        let id = window.id.to_string();
        action(&["msg", "action", "move-window-to-floating", "--id", &id]);
        action(&["msg", "action", "set-window-width", "--id", &id, &w.to_string()]);
        action(&["msg", "action", "set-window-height", "--id", &id, &h.to_string()]);
        // Centre only once the client has taken the new size: centred at
        // once, niri used the old height and the window reached into the
        // dock's gap (01.11.2026). Round 174: wait up to 4 s (was 1.5 s) —
        // a stage busy starting its video took longer, was centred at its
        // OLD width (873) and then shrank to 634 at x = −109, cut off on
        // the left (05.11.2026). Centred again once the size has settled.
        let settled = |w: u32, h: u32| window_size(window.id).is_some_and(|(cw, ch)| cw.abs_diff(w) <= 1 && ch.abs_diff(h) <= 1);
        for _ in 0..80 {
            if settled(w, h) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let done = action(&["msg", "action", "center-window", "--id", &id]);
        std::thread::sleep(std::time::Duration::from_millis(300));
        // Round 174: niri's floating coordinates on the duo panel lay ~120 px
        // off the view — «centred» was x = −109, the video cut off on the
        // left. Measured where it really lies; a RELATIVE move (independent
        // of that offset) puts it in the middle of its screen.
        if let Some(dx) = window_view_pos(window.id).and_then(|(x, _)| {
            let ow = screens().into_iter().find(|s| s.name == window.output)?.logical?.0 as f32;
            Some(centre_shift(x, w as f32, ow))
        }) {
            if dx.abs() > 2.0 {
                eprintln!("[{app_id}] Mitte nachgerückt: {dx:+.0} px");
                action(&["msg", "action", "move-floating-window", "--id", &id, "-x", &format!("{dx:+.0}"), "-y", "+0"]);
            }
        }
        done
    }

    /// Where window `id` really lies in its screen's view (logical px, its
    /// tile's top left) — `layout.tile_pos_in_workspace_view` (round 174).
    pub fn window_view_pos(id: u64) -> Option<(f32, f32)> {
        let out = raw(&["msg", "-j", "windows"])?;
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
        let w = v.as_array()?.iter().find(|w| w["id"].as_u64() == Some(id))?;
        let p = w["layout"]["tile_pos_in_workspace_view"].as_array()?;
        Some((p.first()?.as_f64()? as f32, p.get(1)?.as_f64()? as f32))
    }

    /// How far a window at `x`, `w` wide, must move to stand in the middle
    /// of a screen `screen_w` wide (round 174). Pure.
    pub fn centre_shift(x: f32, w: f32, screen_w: f32) -> f32 {
        (screen_w - w) / 2.0 - x
    }

    #[cfg(test)]
    mod centre_tests {
        #[test]
        fn the_cut_off_stage_moves_back_to_the_middle() {
            // 05.11.2026: 634 px wide at x = −109.45 on the 654 px panel.
            let dx = super::centre_shift(-109.45, 634.0, 654.0);
            assert!((dx - 119.45).abs() < 0.01, "{dx}");
            assert_eq!(super::centre_shift(10.0, 634.0, 654.0), 0.0);
        }
    }

    /// THIS PROCESS'S WINDOW of `app_id` (round 39 — fit, step back and
    /// «is another in front» each looked it up the same way).
    pub fn own_window(app_id: &str) -> Option<AppWindow> {
        let me = std::process::id();
        app_windows(app_id).into_iter().find(|win| win.pid == me)
    }

    /// STEP BACK (03.11.2026, the owner: «the stage is always in the foreground»):
    /// this process's window of `app_id` back into the tiled layout — a
    /// floating window stays above every tiled one in niri, so a fitted
    /// video stage covered each app opened after it.
    pub fn tile_own_window(app_id: &str) -> bool {
        let Some(window) = own_window(app_id) else { return false };
        action(&["msg", "action", "move-window-to-tiling", "--id", &window.id.to_string()])
    }

    /// Is ANOTHER window in front on the panel of this process's window
    /// of `app_id`? (The focused window is not ours and lies on our output.)
    pub fn other_window_in_front(app_id: &str) -> bool {
        let Some(mine) = own_window(app_id) else { return false };
        match focused_app_window() {
            Some((w, _)) => w.id != mine.id && w.output == mine.output,
            None => false,
        }
    }

    /// A window's size (logical px) as niri lays it out.
    pub fn window_size(id: u64) -> Option<(u32, u32)> {
        let v: serde_json::Value = serde_json::from_slice(&raw(&["msg", "-j", "windows"])?.stdout).ok()?;
        let w = v.as_array()?.iter().find(|w| w.get("id").and_then(|i| i.as_u64()) == Some(id))?;
        let size = w.get("layout")?.get("window_size")?.as_array()?;
        Some((size.first()?.as_u64()? as u32, size.get(1)?.as_u64()? as u32))
    }

    /// Where a window lies on its output: (output, x, y, width, height) in
    /// logical pixels (R214, the owner 02.11.2026: «the card should be the same
    /// size as the window»). A floating window: niri says it. A
    /// tiled one: niri gives only its size — it fills the working area
    /// between the gaps, so the rest of the height is what the bar (top)
    /// and the dock (bottom) reserve on that output (`frame_y`).
    /// `top_reserve`: what the bar keeps at the top of its output (margin +
    /// zone) when the caller knows it (the bar does) — then a tiled window
    /// hangs right below it, whatever the bottom holds (the on-screen
    /// keyboard shrank the window from below and the card landed low —
    /// measured: 171 px tall, placed at 308, 02.11.2026).
    pub fn window_frame(id: u64, top_reserve: Option<i32>) -> Option<(String, i32, i32, u32, u32)> {
        let parse = |args: &[&str]| -> Option<serde_json::Value> { serde_json::from_slice(&raw(args)?.stdout).ok() };
        let windows = parse(&["msg", "-j", "windows"])?;
        let w = windows.as_array()?.iter().find(|w| w.get("id").and_then(|i| i.as_u64()) == Some(id))?;
        let layout = w.get("layout")?;
        let size = layout.get("window_size")?.as_array()?;
        let (ww, wh) = (size.first()?.as_u64()? as u32, size.get(1)?.as_u64()? as u32);
        let ws = w.get("workspace_id")?.as_u64()?;
        let spaces = parse(&["msg", "-j", "workspaces"])?;
        let output = spaces.as_array()?.iter().find(|s| s.get("id").and_then(|i| i.as_u64()) == Some(ws))?.get("output")?.as_str()?.to_string();
        if let Some(pos) = layout.get("tile_pos_in_workspace_view").and_then(|p| p.as_array()) {
            let off = layout.get("window_offset_in_tile").and_then(|o| o.as_array()).cloned().unwrap_or_default();
            let at = |i: usize| pos.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0) + off.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);
            return Some((output, at(0).round() as i32, at(1).round() as i32, ww, wh));
        }
        let outputs = parse(&["msg", "-j", "outputs"])?;
        let logical = outputs.get(&output)?.get("logical")?;
        let (ow, oh) = (logical.get("width")?.as_u64()? as u32, logical.get("height")?.as_u64()? as u32);
        let layers = parse(&["msg", "-j", "layers"]).and_then(|l| l.as_array().cloned()).unwrap_or_default();
        let on = |prefix: &str| layers.iter().any(|l| l.get("output").and_then(|o| o.as_str()) == Some(output.as_str()) && l.get("namespace").and_then(|n| n.as_str()).is_some_and(|n| n.starts_with(prefix)));
        let (bar, dock) = (on("matrix-bar"), on("matrix-shelf") || on("matrix-dock"));
        let x = (ow.saturating_sub(ww) / 2) as i32;
        let y = match top_reserve {
            Some(top) if bar => top + WINDOW_GAP as i32,
            _ => frame_y(oh, wh, bar, dock, crate::cinema::subtitle::dock_zone()),
        };
        Some((output, x, y, ww, wh))
    }

    /// The gap between windows and around them (niri `gaps`, local.kdl).
    pub const WINDOW_GAP: u32 = 10;

    /// The top of a tiled, full-height window (pure — tested): the height
    /// left over beside the window and its two gaps belongs to the bar
    /// (top) and the dock (bottom, `dock_zone` of it when both are there).
    pub fn frame_y(output_h: u32, window_h: u32, bar: bool, dock: bool, dock_zone: i32) -> i32 {
        let free = output_h as i32 - window_h as i32 - 2 * WINDOW_GAP as i32;
        let top = match (bar, dock) {
            (true, true) => free - dock_zone.max(0),
            (true, false) => free,
            (false, true) => 0,
            (false, false) => free / 2,
        };
        top.max(0) + WINDOW_GAP as i32
    }

    #[cfg(test)]
    mod frame_tests {
        #[test]
        fn tiled_window_sits_below_the_bar() {
            // The duo's lower panel: bar on top, a 403 px window (measured).
            assert_eq!(super::frame_y(489, 403, true, false, 66), 76);
            // The upper panel: only the dock below — the window at the top gap.
            assert_eq!(super::frame_y(489, 403, false, true, 66), 10);
            assert_eq!(super::frame_y(500, 400, false, false, 0), 50);
        }
    }

    /// One window to the other panel of the duo.
    pub fn window_to_other_panel(window: &AppWindow) {
        if let Some(other) = crate::device::panels().and_then(|p| p.iter().find(|o| **o != window.output)) {
            action(&["msg", "action", "move-window-to-monitor", "--id", &window.id.to_string(), other]);
        }
    }

    /// Do the app's windows lie on more than one panel (a duo pair)?
    pub fn spans_panels(windows: &[AppWindow]) -> bool {
        let mut outputs: Vec<&str> = windows.iter().map(|w| w.output.as_str()).filter(|o| !o.is_empty()).collect();
        outputs.sort();
        outputs.dedup();
        outputs.len() >= 2
    }

    /// SWAP THE PANELS (R210 family): every window of `app_id` to the other
    /// panel of the duo — one tap when a pair came up the wrong way round.
    pub fn swap_panels(app_id: &str) {
        let Some(panels) = crate::device::panels() else { return };
        for w in app_windows(app_id) {
            if let Some(other) = panels.iter().find(|p| **p != w.output) {
                action(&["msg", "action", "move-window-to-monitor", "--id", &w.id.to_string(), other]);
            }
        }
    }

    /// CLOSE THE WHOLE APP (R210 family): every window of `app_id` but the
    /// caller's own process — light-shell windows their own way (mailbox,
    /// so Tube hands its sound over), others through niri.
    pub fn close_app(app_id: &str, own_pid: u32) {
        for w in app_windows(app_id) {
            if w.pid == own_pid {
                continue;
            }
            if w.pid > 0 && crate::house::listens(w.pid) {
                crate::house::send(w.pid, "close");
            } else {
                action(&["msg", "action", "close-window", "--id", &w.id.to_string()]);
            }
        }
    }

    /// THE FOCUSED WINDOW (R210 family): niri's window id and the pid of
    /// its process — for menus that act on «this window» (top bar).
    pub fn focused_window() -> Option<(u64, u32)> {
        let out = raw(&["msg", "-j", "focused-window"])?;
        let value: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
        let id = value.get("id")?.as_u64()?;
        let pid = value.get("pid").and_then(|p| p.as_u64()).unwrap_or(0) as u32;
        Some((id, pid))
    }

    /// PANEL CLOCK (R210 family): the slowest refresh rate (Hz) of the
    /// screens the house draws on. A video faster than this drops frames
    /// unevenly — that is stutter; decoded at this rate, each refresh shows
    /// exactly one frame. The duo's DSI panels run at 21.932 Hz (01.11.2026).
    ///
    /// A panel whose mode computes to a wrong rate gets the real one from
    /// the session (`NIRI_CANVAS_REFRESH_HZ`, niri patch 0011; the duo's
    /// vblank runs at 60 Hz) — that variable wins over niri's mode list.
    pub fn refresh_hz() -> Option<f64> {
        if let Some(hz) = std::env::var("NIRI_CANVAS_REFRESH_HZ").ok().and_then(|v| v.trim().parse::<f64>().ok()) {
            if (10.0..=500.0).contains(&hz) {
                return Some(hz);
            }
        }
        let out = raw(&["msg", "-j", "outputs"])?;
        let value: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
        let outputs = value.as_object()?;
        outputs
            .values()
            .filter_map(|o| {
                let i = o.get("current_mode")?.as_u64()? as usize;
                let mode = o.get("modes")?.as_array()?.get(i)?;
                Some(mode.get("refresh_rate")?.as_f64()? / 1000.0)
            })
            .filter(|hz| *hz > 1.0)
            .reduce(f64::min)
    }

    /// THE FLING TARGET (round 171): the screen that lies in direction `dir`
    /// (a finger's flick, logical px/s) from screen `here` — the one whose
    /// centre lies closest to that direction (within ±70°), the nearer one
    /// on a tie. Nothing in that direction but exactly one other screen:
    /// that one (two panels, no TV — any flick means «the other»). Pure.
    pub fn fling_target(screens: &[Screen], here: &str, dir: (f32, f32)) -> Option<String> {
        let centre = |s: &Screen| -> Option<(f32, f32)> {
            let (w, h) = s.logical?;
            Some((s.pos.0 as f32 + w as f32 / 2.0, s.pos.1 as f32 + h as f32 / 2.0))
        };
        let from = centre(screens.iter().find(|s| s.name == here)?)?;
        let len = dir.0.hypot(dir.1);
        if len < 1.0 {
            return None;
        }
        let d = (dir.0 / len, dir.1 / len);
        let others: Vec<(&Screen, (f32, f32))> =
            screens.iter().filter(|s| s.name != here).filter_map(|s| Some((s, centre(s)?))).collect();
        let best = others
            .iter()
            .filter_map(|(s, c)| {
                let v = (c.0 - from.0, c.1 - from.1);
                let dist = v.0.hypot(v.1).max(1.0);
                let cos = (v.0 * d.0 + v.1 * d.1) / dist;
                (cos >= 0.34).then_some((s, cos, dist))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal).then(b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)));
        match best {
            Some((s, _, _)) => Some(s.name.clone()),
            None if others.len() == 1 => Some(others[0].0.name.clone()),
            None => None,
        }
    }

    /// FLING THIS APP'S WINDOW (round 171, the owner: «long-press the app with
    /// a finger and flick it towards the other panel or
    /// display»): a finger ends at the hinge and never reaches a TV, so the
    /// window jumps to the screen in the flick's direction. Our window is
    /// found by our pid; its screen is where it lies. False: no target.
    pub fn window_fling(dir: (f32, f32)) -> bool {
        let me = Some(std::process::id() as u64);
        let json = |args: &[&str]| raw(args).and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok()).unwrap_or_default();
        let windows = json(&["msg", "-j", "windows"]);
        let list = windows.as_array().cloned().unwrap_or_default();
        // Our window: the focused one of ours, else any of ours.
        let window = list
            .iter()
            .find(|w| w["pid"].as_u64() == me && w["is_focused"].as_bool() == Some(true))
            .or_else(|| list.iter().find(|w| w["pid"].as_u64() == me));
        let mine = window.and_then(|w| w["id"].as_u64());
        // Its screen: through its workspace — the focused screen may be the other panel.
        let workspaces = json(&["msg", "-j", "workspaces"]);
        let own_output = window.and_then(|w| {
            let ws = w["workspace_id"].as_u64()?;
            workspaces.as_array()?.iter().find(|x| x["id"].as_u64() == Some(ws))?["output"].as_str().map(String::from)
        });
        let Some(here) = own_output.or_else(focused_output) else { return false };
        let Some(target) = fling_target(&screens(), &here, dir) else { return false };
        match mine {
            Some(id) => {
                action(&["msg", "action", "move-window-to-monitor", "--id", &id.to_string(), &target]);
            }
            None => {
                action(&["msg", "action", "move-window-to-monitor", &target]);
            }
        }
        action(&["msg", "action", "focus-monitor", &target]);
        true
    }

    #[cfg(test)]
    mod fling_target_tests {
        use super::*;

        fn s(name: &str, x: i32, y: i32, w: u32, h: u32) -> Screen {
            Screen { name: name.into(), label: String::new(), width: w, height: h, transform: "normal".into(), logical: Some((w, h)), scale: 1.0, pos: (x, y) }
        }

        #[test]
        fn a_flick_finds_the_screen_in_its_direction() {
            // The duo in landscape: DSI-1 above DSI-2; a TV to the right.
            let all = [s("DSI-1", 0, 0, 654, 490), s("DSI-2", 0, 491, 654, 490), s("DP-1", 654, 0, 1280, 720)];
            assert_eq!(fling_target(&all, "DSI-1", (0.0, 900.0)).as_deref(), Some("DSI-2"), "down over the hinge");
            assert_eq!(fling_target(&all, "DSI-2", (0.0, -900.0)).as_deref(), Some("DSI-1"));
            assert_eq!(fling_target(&all, "DSI-1", (900.0, 0.0)).as_deref(), Some("DP-1"), "right to the TV");
            assert_eq!(fling_target(&all, "DP-1", (-900.0, 100.0)).as_deref(), Some("DSI-1"), "back, the nearer panel");
            assert_eq!(fling_target(&all, "DSI-1", (-900.0, 0.0)), None, "nothing to the left");
        }

        #[test]
        fn two_panels_mean_the_other_one() {
            let two = [s("DSI-1", 0, 0, 654, 490), s("DSI-2", 0, 491, 654, 490)];
            assert_eq!(fling_target(&two, "DSI-1", (900.0, 0.0)).as_deref(), Some("DSI-2"), "sideways still the other panel");
            assert_eq!(fling_target(&two, "DSI-9", (0.0, 900.0)), None, "unknown screen");
            assert_eq!(fling_target(&two, "DSI-1", (0.0, 0.0)), None, "no flick");
        }
    }

    /// SEND TO THE NEIGHBOUR PANEL (R209 family): move the focused window to
    /// the screen beside it. A device with its own panels (duo) knows its
    /// two by name — the OTHER one is meant, whatever the direction: the
    /// panels' logical layout follows the posture (side by side or stacked)
    /// and need not match the window's own up/down, so `-up` on a duo found
    /// no screen at all (01.11.2026). Elsewhere the direction decides.
    pub fn window_to_neighbour(side: &str) {
        if let (Some(panels), Some(here)) = (crate::device::panels(), focused_output()) {
            if let Some(other) = panels.iter().find(|p| **p != here) {
                action(&["msg", "action", "move-window-to-monitor", other]);
                return;
            }
        }
        action(&["msg", "action", &format!("move-window-to-monitor-{side}")]);
    }

    /// Put the focused window on the shelf (minimize replacement).
    pub fn window_to_shelf() {
        action(&["msg", "action", "move-window-to-workspace", "shelf"]);
    }

    /// R197: shelf the focused window WITHOUT the focus riding along —
    /// the view stays where it is (the plain variant pulls the view to
    /// the shelf workspace, which put the host behind the Raum scrim).
    pub fn window_to_shelf_stay() {
        action(&["msg", "action", "move-window-to-workspace", "--focus", "false", "shelf"]);
    }

    /// Focus a workspace by name or index.
    pub fn workspace_focus(reference: &str) {
        action(&["msg", "action", "focus-workspace", reference]);
    }

    /// The currently focused workspace — its name if it has one, else
    /// its index. Feed it back to `workspace_focus` to return home.
    pub fn workspace_current() -> Option<String> {
        let out = raw(&["msg", "-j", "workspaces"])?;
        let ws: Vec<serde_json::Value> = serde_json::from_slice(&out.stdout).ok()?;
        let f = ws.iter().find(|w| w.get("is_focused").and_then(|b| b.as_bool()) == Some(true))?;
        if let Some(name) = f.get("name").and_then(|n| n.as_str()) {
            return Some(name.to_string());
        }
        f.get("idx").and_then(|i| i.as_u64()).map(|i| i.to_string())
    }

    /// Event stream of the compositor as a child process (stdout line by
    /// line, `json` = machine-readable). The caller reads and reconnects.
    pub fn event_stream(json: bool) -> std::io::Result<Child> {
        event_command(json).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()
    }

    /// The event-stream command itself (round 50), for `follow_events`.
    /// Round 168: the stream DIES WITH ITS APP — a killed or crashed bar left
    /// `niri msg event-stream` behind as an orphan (12 MB each; two after
    /// one restart of bar and dock, 05.11.2026).
    pub fn event_command(json: bool) -> Command {
        let mut cmd = command(if json { &["msg", "-j", "event-stream"] } else { &["msg", "event-stream"] });
        // SAFETY: pre_exec runs only the async-signal-safe prctl.
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                crate::process::die_with_parent();
                Ok(())
            });
        }
        cmd
    }

    /// FOLLOW THE COMPOSITOR'S EVENTS (round 50): every line of
    /// `niri msg [-j] event-stream` to `on_line(Some(line))`; a torn stream
    /// (compositor restart) is `on_line(None)` and reconnects after 2 s;
    /// `false` ends it — the stream process killed AND reaped. The bar, the
    /// dock, the sound hooks and the iced canvas stream each ran this loop
    /// by hand; one of them left a zombie per closed card (round 48).
    pub fn follow_events(
        json: bool,
        on_line: impl FnMut(Option<&str>) -> bool + Send + 'static,
    ) -> std::thread::JoinHandle<()> {
        crate::process::follow(move || event_command(json), std::time::Duration::from_secs(2), on_line)
    }
}

/// ACCOUNT (Morpheus, round 175, 06.11.2026): the first account of a new
/// Matrix device — its name from the person's name, the password rules, and
/// the request to the first-start helper (`matrix-erststart-helfer`, root,
/// socket /run/matrix-erststart.sock), which creates exactly ONE account
/// and only while none exists. The login screen runs the setup with it.
pub mod account {
    /// Where the first-start helper listens.
    pub const SOCKET: &str = "/run/matrix-erststart.sock";
    /// A password at least this long.
    pub const MIN_PASSWORD: usize = 4;

    /// The account name from a person's name: lower case, umlauts written
    /// out (ä→ae, ß→ss), words joined by a dot, only a–z, 0–9, «.» — and a
    /// letter first; nothing usable: «matrix». At most 32 characters.
    pub fn username_from(name: &str) -> String {
        let mut out = String::new();
        let mut gap = false;
        for c in name.trim().chars().flat_map(|c| c.to_lowercase()) {
            let piece: &str = match c {
                'ä' => "ae",
                'ö' => "oe",
                'ü' => "ue",
                'ß' => "ss",
                'à' | 'á' | 'â' | 'ã' | 'å' => "a",
                'è' | 'é' | 'ê' | 'ë' => "e",
                'ì' | 'í' | 'î' | 'ï' => "i",
                'ò' | 'ó' | 'ô' | 'õ' => "o",
                'ù' | 'ú' | 'û' => "u",
                'ç' => "c",
                'ñ' => "n",
                _ => "",
            };
            if !piece.is_empty() {
                if gap && !out.is_empty() {
                    out.push('.');
                }
                out.push_str(piece);
                gap = false;
            } else if c.is_ascii_alphanumeric() {
                if gap && !out.is_empty() {
                    out.push('.');
                }
                out.push(c);
                gap = false;
            } else {
                gap = true;
            }
        }
        let out: String = out.trim_start_matches(|c: char| !c.is_ascii_lowercase()).chars().take(32).collect();
        let out = out.trim_end_matches('.').to_string();
        if out.is_empty() {
            String::from("matrix")
        } else {
            out
        }
    }

    /// Is this a valid account name (what the helper accepts)?
    pub fn valid_username(u: &str) -> bool {
        let mut c = u.chars();
        matches!(c.next(), Some(f) if f.is_ascii_lowercase())
            && u.len() <= 32
            && u.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_')
            && !["root", "greetd", "matrix-erststart"].contains(&u)
    }

    /// The password rules: long enough, both entries the same.
    pub fn password_check(first: &str, again: &str) -> Result<(), &'static str> {
        if first.chars().count() < MIN_PASSWORD {
            Err("Mindestens 4 Zeichen")
        } else if first != again {
            Err("Die beiden Passwörter sind nicht gleich")
        } else {
            Ok(())
        }
    }

    /// The first person's account in a passwd text (uid 1000…, a shell).
    pub fn first_person(passwd: &str) -> Option<(String, String)> {
        passwd.lines().find_map(|l| {
            let f: Vec<&str> = l.split(':').collect();
            let uid: u32 = f.get(2)?.parse().ok()?;
            ((1000..60000).contains(&uid) && f.len() >= 7 && !f[6].contains("nologin") && !f[6].ends_with("false"))
                .then(|| (f[0].to_string(), f[4].split(',').next().unwrap_or("").trim().to_string()))
        })
    }

    /// Does this device have its person yet?
    pub fn has_person() -> bool {
        std::fs::read_to_string("/etc/passwd").ok().and_then(|p| first_person(&p)).is_some()
    }

    /// Ask the helper to create the account (blocking, a few seconds), in
    /// language `lang` («de-CH» / «en-GB») with keyboard `keyboard` («ch» / «gb»).
    pub fn create(name: &str, user: &str, password: &str, lang: &str, keyboard: &str) -> Result<(), String> {
        use std::io::{BufRead, Write};
        let mut s = std::os::unix::net::UnixStream::connect(SOCKET).map_err(|e| format!("Einrichtungsdienst nicht erreichbar: {e}"))?;
        s.set_read_timeout(Some(std::time::Duration::from_secs(60))).ok();
        let line = serde_json::json!({ "name": name, "user": user, "password": password, "sprache": lang, "tastatur": keyboard }).to_string();
        s.write_all(line.as_bytes()).and_then(|_| s.write_all(b"\n")).map_err(|e| format!("Senden: {e}"))?;
        let mut answer = String::new();
        std::io::BufReader::new(s).read_line(&mut answer).map_err(|e| format!("Antwort: {e}"))?;
        let answer = answer.trim();
        match answer.strip_prefix("ok") {
            Some(_) => Ok(()),
            None => Err(answer.trim_start_matches("fehler:").trim().to_string()),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn names_become_accounts() {
            assert_eq!(username_from("Anna"), "anna");
            assert_eq!(username_from("Anna Müller"), "anna.mueller");
            assert_eq!(username_from("  Anna-Lena Strauß "), "anna.lena.strauss");
            assert_eq!(username_from("José Peña"), "jose.pena");
            assert_eq!(username_from("42 Neo"), "neo");
            assert_eq!(username_from("✨"), "matrix");
            assert!(username_from(&"x".repeat(50)).len() <= 32);
            for n in ["the owner Müller", "42 Neo", "✨"] {
                assert!(valid_username(&username_from(n)), "{n}");
            }
            assert!(!valid_username("root") && !valid_username("Gross") && !valid_username("1a"));
        }

        #[test]
        fn passwords_are_checked() {
            assert!(password_check("abc", "abc").is_err());
            assert!(password_check("abcd", "abce").is_err());
            assert!(password_check("abcd", "abcd").is_ok());
        }

        #[test]
        fn the_first_person_is_found() {
            let p = "root:x:0:0:root:/root:/bin/bash\ngreetd:x:977:977::/var/lib/greetd:/sbin/nologin\nanna:x:1000:1000:Anna Müller:/home/anna:/bin/bash\n";
            assert_eq!(first_person(p), Some((String::from("anna"), String::from("Anna Müller"))));
            assert_eq!(first_person("root:x:0:0:root:/root:/bin/bash\n"), None, "a new device");
        }
    }
}

/// Window civilisation: single instance, shelf (minimise replacement),
/// Size memory — the reference matter-of-course behaviors.
pub mod window {
    use std::path::PathBuf;

    fn home() -> String {
        std::env::var("HOME").unwrap_or_else(|_| "/root".into())
    }

    fn runtime_dir() -> String {
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into())
    }

    // Compositor access runs through the canvas facade since R45.
    use super::canvas;

    /// Single instance like the reference: if the app already runs, its
    /// window is focused and false returned — the new process exits.
    /// The lock socket lives as long as the process; crash corpses are
    /// detected (nobody listens) and cleaned up.
    pub fn single_instance(app_id: &str) -> bool {
        use std::os::unix::net::{UnixListener, UnixStream};
        let path = format!("{}/matrixkit-{app_id}.sock", runtime_dir());
        match UnixListener::bind(&path) {
            Ok(l) => {
                std::mem::forget(l);
                true
            }
            Err(_) => {
                if UnixStream::connect(&path).is_err() {
                    // corpse of a crash — clean up and take its place
                    let _ = std::fs::remove_file(&path);
                    if let Ok(l) = UnixListener::bind(&path) {
                        std::mem::forget(l);
                        return true;
                    }
                }
                focus_window(app_id);
                false
            }
        }
    }

    /// Bring the (first) window of an app id into focus. If it lies on
    /// the shelf (minimized), it is first brought to the ACTIVE workspace
    /// — the reference restore via launcher/app relaunch.
    pub fn focus_window(app_id: &str) {
        let Some(out) = canvas::raw(&["msg", "-j", "windows"]) else { return };
        let Ok(window) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) else {
            return;
        };
        let Some(f) = window
            .iter()
            .find(|f| f.get("app_id").and_then(|a| a.as_str()) == Some(app_id))
        else {
            return;
        };
        let Some(id) = f.get("id").and_then(|i| i.as_u64()) else { return };
        canvas::window_bring(id);
    }

    /// One window of an app as a dock tile sees it (round 169).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TileWin {
        pub id: u64,
        pub focus: bool,
        /// Lies on the shelf (minimized).
        pub shelved: bool,
    }

    /// What a tap on an app's dock tile does (round 169).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TileTap {
        /// Fetch this window (from the shelf, if it lies there) and focus it.
        Bring(u64),
        /// Put the window in front on the shelf (duo: no shelf light).
        Shelve,
        /// No window: start the app.
        Start,
    }

    /// THE TILE TAP (round 169): a window on the shelf comes back first —
    /// the newest of them. Prism put its progress dialog on the shelf with
    /// its own minimize button; the tile then brought the OLDEST window (the
    /// main one, already in front) or shelved it, and the dialog could never
    /// be reached (owner, Duo, 05.11.2026, twice). Then, on a duo, the app
    /// in front goes on the shelf; elsewhere the app in front cycles its
    /// windows; else its oldest window comes.
    pub fn tile_tap(windows: &[TileWin], tap_shelves: bool) -> TileTap {
        if let Some(w) = windows.iter().filter(|w| w.shelved).max_by_key(|w| w.id) {
            return TileTap::Bring(w.id);
        }
        let Some(first) = windows.iter().min_by_key(|w| w.id) else { return TileTap::Start };
        let front = windows.iter().any(|w| w.focus);
        if front && tap_shelves {
            return TileTap::Shelve;
        }
        if front && windows.len() > 1 {
            let mut ids: Vec<u64> = windows.iter().map(|w| w.id).collect();
            ids.sort_unstable();
            if let Some(i) = windows.iter().find(|w| w.focus).and_then(|f| ids.iter().position(|&id| id == f.id)) {
                return TileTap::Bring(ids[(i + 1) % ids.len()]);
            }
        }
        TileTap::Bring(first.id)
    }

    /// The id of the shelf workspace, from `niri msg -j workspaces` (round 169).
    pub fn shelf_workspace(workspaces: &serde_json::Value) -> Option<u64> {
        workspaces.as_array()?.iter().find(|w| w["name"].as_str() == Some("shelf"))?["id"].as_u64()
    }

    #[cfg(test)]
    mod tile_tap_tests {
        use super::*;

        fn w(id: u64, focus: bool, shelved: bool) -> TileWin {
            TileWin { id, focus, shelved }
        }

        #[test]
        fn the_shelved_dialog_comes_back() {
            // Prism: main window in front, progress dialog on the shelf.
            let prism = [w(14, true, false), w(16, false, true)];
            assert_eq!(tile_tap(&prism, true), TileTap::Bring(16), "duo: not «shelve the main window»");
            assert_eq!(tile_tap(&prism, false), TileTap::Bring(16));
            // Two on the shelf: the newest first.
            assert_eq!(tile_tap(&[w(3, false, true), w(9, false, true)], true), TileTap::Bring(9));
        }

        #[test]
        fn front_shelves_on_a_duo_and_cycles_elsewhere() {
            let two = [w(5, true, false), w(8, false, false)];
            assert_eq!(tile_tap(&two, true), TileTap::Shelve);
            assert_eq!(tile_tap(&two, false), TileTap::Bring(8));
            assert_eq!(tile_tap(&[w(5, false, false), w(8, true, false)], false), TileTap::Bring(5));
            assert_eq!(tile_tap(&[w(8, false, false), w(5, false, false)], true), TileTap::Bring(5), "the oldest");
            assert_eq!(tile_tap(&[], true), TileTap::Start);
        }

        #[test]
        fn finds_the_shelf() {
            let ws: serde_json::Value = serde_json::json!([{"id": 1, "name": "arbeit"}, {"id": 2, "name": "shelf"}]);
            assert_eq!(shelf_workspace(&ws), Some(2));
            assert_eq!(shelf_workspace(&serde_json::json!([])), None);
        }
    }

    /// "Minimize" on Matrix: put the window onto the shelf workspace.
    pub fn shelf() {
        let _ = canvas::raw(&[
            "msg",
            "action",
            "move-window-to-workspace",
            "--focus",
            "false",
            "shelf",
        ]);
    }

    fn sizes_path(app_id: &str) -> PathBuf {
        // R107: windows/ is the contract; fenster/ read as legacy.
        let dir = PathBuf::from(home()).join(".config/matrix");
        let modern = dir.join(format!("windows/{app_id}"));
        if modern.exists() {
            return modern;
        }
        let legacy = dir.join(format!("fenster/{app_id}"));
        if legacy.exists() {
            return legacy;
        }
        modern
    }

    /// Remembered window size (width, height) — None on first start.
    pub fn size_read(app_id: &str) -> Option<(f32, f32)> {
        let content = std::fs::read_to_string(sizes_path(app_id)).ok()?;
        let (w, h) = content.trim().split_once('x')?;
        Some((w.parse().ok()?, h.parse().ok()?))
    }

    /// Remember the size for the next open (reference window memory).
    pub fn size_remember(app_id: &str, width: f32, height: f32) {
        let path = sizes_path(app_id);
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, format!("{width:.0}x{height:.0}"));
    }
}

/// The OSD channel of the bar family: `matrix-osd` (keys binary)
/// writes the state here, the DOCK reads it and briefly morphs into
/// the OSD ("Dynamic Dock", user design 2026-07-07). A simple runtime
/// file, no DBus — the mtime is the heartbeat clock.
pub mod osd {
    /// What was last adjusted on the system.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct State {
        pub kind: Kind,
        /// 0–100.
        pub percent: f32,
        pub muted: bool,
        /// Did the change come from a level step (louder/quieter/
        /// brighter/darker) or from a toggle (mute/mic)?
        pub step: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Kind {
        Sound,
        Mic,
        Light,
    }

    impl State {
        /// Row 2 of the Dynamic Dock: WHAT changed, in words.
        pub fn text(&self) -> String {
            match (self.kind, self.step, self.muted) {
                (Kind::Sound, true, _) => format!("Lautstärke {:.0} %", self.percent),
                (Kind::Sound, false, true) => String::from("Ton aus"),
                (Kind::Sound, false, false) => format!("Ton an — {:.0} %", self.percent),
                (Kind::Mic, _, true) => String::from("Mikrofon aus"),
                (Kind::Mic, _, false) => String::from("Mikrofon an"),
                (Kind::Light, _, _) => format!("Helligkeit {:.0} %", self.percent),
            }
        }
    }

    /// Where the OSD state lives. Public because a listener has to
    /// know what to watch (R135) instead of asking every 150 ms.
    pub fn path() -> String {
        let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
        format!("{dir}/matrix-osd-state")
    }

    /// One line: `sound 75 1 step|toggle`.
    pub fn write(state: State) {
        let kind = match state.kind {
            Kind::Sound => "sound",
            Kind::Mic => "mic",
            Kind::Light => "light",
        };
        let mode = if state.step { "step" } else { "toggle" };
        let _ = std::fs::write(
            path(),
            format!("{kind} {:.0} {} {mode}", state.percent, state.muted as u8),
        );
    }

    /// State + time of the last key press (file mtime).
    pub fn read() -> Option<(State, std::time::SystemTime)> {
        let p = path();
        let mtime = std::fs::metadata(&p).ok()?.modified().ok()?;
        let content = std::fs::read_to_string(&p).ok()?;
        let mut parts = content.split_whitespace();
        let kind = match parts.next()? {
            "sound" => Kind::Sound,
            "mic" => Kind::Mic,
            "light" => Kind::Light,
            _ => return None,
        };
        let percent: f32 = parts.next()?.parse().ok()?;
        let muted = parts.next() == Some("1");
        let step = parts.next() == Some("step");
        Some((State { kind, percent, muted, step }, mtime))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn roundtrip_and_texts() {
            std::env::set_var("XDG_RUNTIME_DIR", std::env::temp_dir());
            let s = State { kind: Kind::Sound, percent: 75.0, muted: false, step: true };
            write(s);
            let (read_in, _) = read().unwrap();
            assert_eq!(read_in, s);
            assert_eq!(read_in.text(), "Lautstärke 75 %");
            let muted = State { kind: Kind::Sound, percent: 75.0, muted: true, step: false };
            assert_eq!(muted.text(), "Ton aus");
            let light = State { kind: Kind::Light, percent: 40.0, muted: false, step: true };
            assert_eq!(light.text(), "Helligkeit 40 %");
        }
    }
}

/// Command bridges of the bar family: EVERY app speaks to system
/// tools (wpctl, nmcli, brightnessctl, niri …) via these two paths —
/// never via its own command snippets. Lives in the theme so that
/// iced-free tools (matrix-osd) can use them too.
/// Round 178 (family audit): copying text to the clipboard — seven hand-rolled copies, two of them passing the
/// text as an argument without `--` (a calculator result «-5» was read as an option and nothing was copied).
/// The text goes through stdin; `wl-copy` forks its own server and the call returns at once.
pub mod clipboard {
    pub fn copy(text: &str) -> bool {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() else {
            return false;
        };
        let wrote = child.stdin.take().is_some_and(|mut i| i.write_all(text.as_bytes()).is_ok());
        // The parent leaves once it has read stdin (its server child keeps the text); reap it with a deadline.
        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(st)) => return wrote && st.success(),
                Ok(None) if start.elapsed() < std::time::Duration::from_secs(5) => std::thread::sleep(std::time::Duration::from_millis(5)),
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return false;
                }
            }
        }
    }
}

pub mod command {
    //! Round 178 (07.11.2026, stability audit): EVERY helper call has a deadline. Before, `bluetoothctl show` with a
    //! dead bluetoothd blocked forever — the control centre's pulse froze, and the dock gathered one hung thread and
    //! one live child every 15 pulses. A call that runs out is killed AND reaped (no zombie, no stuck thread).
    use std::time::Duration;

    /// Reads (`first_line`, `text_from`): answers come within milliseconds; 8 s covers a slow `ddcutil`.
    pub const READ_LIMIT: Duration = Duration::from_secs(8);
    /// Deeds (`silent`): `nmcli`/`bluetoothctl` connects take seconds, never this long.
    pub const DEED_LIMIT: Duration = Duration::from_secs(30);

    /// Run with a deadline: (success, stdout) — None when it could not start or ran out (then killed + reaped).
    /// `want_output = false` gives the child no stdout pipe at all (a daemonising child like `wl-copy` keeps a
    /// pipe open forever).
    pub fn run_within(cmd: &str, args: &[&str], limit: Duration, want_output: bool) -> Option<(bool, Vec<u8>)> {
        use std::io::Read;
        use std::process::{Command, Stdio};
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::null())
            .stdout(if want_output { Stdio::piped() } else { Stdio::null() })
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        // Read stdout on its own thread: a full pipe must never block the child (and so the deadline).
        let rx = child.stdout.take().map(|mut out| {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let _ = out.read_to_end(&mut buf);
                let _ = tx.send(buf);
            });
            rx
        });
        let start = std::time::Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(st)) => break Some(st),
                Ok(None) if start.elapsed() < limit => std::thread::sleep(Duration::from_millis(5)),
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
            }
        }?;
        // A grandchild may still hold the pipe: take what came, don't wait for its end.
        let out = rx.and_then(|rx| rx.recv_timeout(Duration::from_millis(500)).ok()).unwrap_or_default();
        Some((status.success(), out))
    }

    /// Execute, report success — output does not matter.
    pub fn silent(cmd: &str, args: &[&str]) -> bool {
        silent_within(cmd, args, DEED_LIMIT)
    }

    pub fn silent_within(cmd: &str, args: &[&str], limit: Duration) -> bool {
        run_within(cmd, args, limit, false).is_some_and(|(ok, _)| ok)
    }

    /// First output line (trimmed) or None on errors/emptiness/time-out.
    pub fn first_line(cmd: &str, args: &[&str]) -> Option<String> {
        first_line_within(cmd, args, READ_LIMIT)
    }

    pub fn first_line_within(cmd: &str, args: &[&str], limit: Duration) -> Option<String> {
        let (ok, out) = run_within(cmd, args, limit, true)?;
        if !ok {
            return None;
        }
        let s = String::from_utf8_lossy(&out);
        let first = s.lines().next()?.trim();
        (!first.is_empty()).then(|| first.to_string())
    }

    /// The full stdout of a command (None on failure status or time-out).
    pub fn text_from(cmd: &str, args: &[&str]) -> Option<String> {
        text_from_within(cmd, args, READ_LIMIT)
    }

    pub fn text_from_within(cmd: &str, args: &[&str], limit: Duration) -> Option<String> {
        let (ok, out) = run_within(cmd, args, limit, true)?;
        ok.then(|| String::from_utf8_lossy(&out).to_string())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn bridge_behavior() {
            assert!(silent("true", &[]));
            assert!(!silent("false", &[]));
            assert_eq!(first_line("echo", &["hallo", "welt"]).as_deref(), Some("hallo welt"));
            assert_eq!(first_line("false", &[]), None);
        }

        #[test]
        fn the_gauge_rule_skips_device_batteries() {
            // On any machine: if there is a gauge, it is not a peripheral's (scope Device).
            if let Some(g) = crate::power::gauge() {
                let scope = std::fs::read_to_string(g.join("scope")).unwrap_or_default();
                assert_ne!(scope.trim(), "Device");
            }
        }

        #[test]
        fn a_hanging_call_is_cut_and_reaped() {
            let t = std::time::Instant::now();
            assert!(!silent_within("sleep", &["10"], Duration::from_millis(200)));
            assert_eq!(first_line_within("sleep", &["10"], Duration::from_millis(200)), None);
            assert!(t.elapsed() < Duration::from_secs(3), "deadline held");
        }

        #[test]
        fn big_output_does_not_block_the_deadline() {
            // 1 MB through the pipe — more than a pipe buffer holds.
            let text = text_from_within("head", &["-c", "1048576", "/dev/zero"], Duration::from_secs(5)).unwrap();
            assert_eq!(text.len(), 1_048_576);
        }
    }
}

/// Dock badge — the NSDockTile.badgeLabel extract (reference round 17):
/// an app attaches a small number/note to its dock icon ("1 update").
/// Channel: one file per app_id in the runtime directory — easy to
/// set, easy to delete, nothing survives a restart.
pub mod badge {
    fn dir() -> std::path::PathBuf {
        let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
        std::path::PathBuf::from(base).join("matrix-abzeichen")
    }

    pub fn set(app_id: &str, text: &str) {
        let d = dir();
        let _ = std::fs::create_dir_all(&d);
        let _ = std::fs::write(d.join(app_id), text);
    }

    pub fn delete(app_id: &str) {
        let _ = std::fs::remove_file(dir().join(app_id));
    }

    pub fn read(app_id: &str) -> Option<String> {
        let t = std::fs::read_to_string(dir().join(app_id)).ok()?;
        let t = t.trim().to_string();
        (!t.is_empty()).then_some(t)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn roundtrip() {
            std::env::set_var("XDG_RUNTIME_DIR", std::env::temp_dir());
            set("test-app", "3");
            assert_eq!(read("test-app").as_deref(), Some("3"));
            delete("test-app");
            assert_eq!(read("test-app"), None);
        }
    }
}

/// The sensoryFeedback extract (reference round 20): SEMANTIC
/// feedback roles instead of file paths — apps say WHAT happened
/// (success, error, notice), the system picks the Matrix sound and
/// respects the sound culture (~/.config/matrix/sounds.conf:
/// master "all" and the notification toggle; klaenge.conf is read as legacy).
pub mod feedback {
    /// Legacy German conf key for an English one (pre-R105 files).
    fn legacy_key(k: &str) -> Option<&'static str> {
        Some(match k {
            "all" => "alle",
            "notifications" => "benachrichtigungen",
            "01-login" => "01-anmeldung",
            "02-logout" => "02-abmeldung",
            "03-notification" => "03-benachrichtigung",
            "04-notification-quiet" => "04-benachrichtigung-leise",
            "05-volume" => "05-lautstaerke",
            "06-device-connected" => "06-geraet-verbunden",
            "07-device-disconnected" => "07-geraet-getrennt",
            "08-workspace" => "08-arbeitsflaeche",
            "10-trash" => "10-papierkorb",
            "11-done" => "11-fertig",
            "12-error" => "12-fehler",
            "13-key-detected" => "13-schluessel-erkannt",
            "14-guardian-call" => "14-waechter-ruf",
            _ => return None,
        })
    }

    fn is_off(k: &str) -> bool {
        let home = std::env::var("HOME").unwrap_or_default();
        // R105 migration: sounds.conf is the truth, klaenge.conf the
        // legacy fallback; values accept off AND the old "aus".
        let conf = std::fs::read_to_string(format!("{home}/.config/matrix/sounds.conf"))
            .or_else(|_| std::fs::read_to_string(format!("{home}/.config/matrix/klaenge.conf")))
            .unwrap_or_default();
        conf.lines().any(|z| {
            let Some((a, b)) = z.split_once('=') else { return false };
            let (a, b) = (a.trim(), b.trim());
            (b == "off" || b == "aus") && (a == k || Some(a) == legacy_key(k))
        })
    }

    /// The sound culture: master "all" plus one key per sound family
    /// (missing key = on).
    fn allowed_for(key: &str) -> bool {
        !is_off("all") && !is_off(key)
    }

    fn allowed() -> bool {
        allowed_for("notifications")
    }

    /// Resolve a sound file: the dev state in the home WINS (12-error
    /// as anchor), otherwise the image — same rule as the sound folder
    /// of the Sounds app. Before, feedback stubbornly played from
    /// /usr/share and left freshly rendered sounds unheard
    /// (Nutzer-Fund 15.7.).
    fn sound_path(file: &str) -> String {
        // Every sound played through the kit leaves its time (round 41): a
        // second player of the same event stays quiet (`played_recently`).
        crate::runtime::stamp(&format!("matrix-klang-{file}"));
        sound_dir().join(file).to_string_lossy().into_owned()
    }

    /// Was `file` played through the kit within `ms`? (03.11.2026: the
    /// volume key sounded twice — the OSD played it, and the sound hooks
    /// heard the volume change and played it again.)
    pub fn played_recently(file: &str, ms: u64) -> bool {
        crate::runtime::fresh(&format!("matrix-klang-{file}"), ms)
    }

    /// THE SOUND FOLDER (03.11.2026, one rule for every player): the dev
    /// state in the home wins (12-error as anchor); on a Surface Duo its
    /// own voice (`/usr/share/matrix/sounds-duo`, glass in D major,
    /// stereo — «system sounds that only the Duo has»); else the Matrix gong set.
    pub fn sound_dir() -> std::path::PathBuf {
        let anchor = |d: &std::path::Path| d.join("12-error.wav").exists();
        if let Ok(home) = std::env::var("HOME") {
            let dev = std::path::PathBuf::from(home).join(".local/share/matrix/sounds");
            if anchor(&dev) {
                return dev;
            }
        }
        let duo = std::path::PathBuf::from("/usr/share/matrix/sounds-duo");
        if super::device::is_duo() && anchor(&duo) {
            return duo;
        }
        std::path::PathBuf::from("/usr/share/matrix/sounds")
    }

    fn play(file: &'static str) {
        if !allowed() {
            return;
        }
        std::thread::spawn(move || {
            let path = sound_path(file);
            for spieler in ["pw-play", "paplay"] {
                if super::command::silent(spieler, &[&path]) {
                    return;
                }
            }
        });
    }

    /// Play SYNCHRONOUSLY — for short-lived tools (matrix-osd) and for
    /// moments when the sound must be DONE before things continue
    /// (logout sound before poweroff). Blocks for the duration of the file.
    pub fn now(key: &str, file: &str) -> bool {
        if !allowed_for(key) {
            return false;
        }
        let path = sound_path(file);
        for spieler in ["pw-play", "paplay"] {
            if super::command::silent(spieler, &[&path]) {
                return true;
            }
        }
        false
    }

    /// The keyboard click trio (R168b): the reference gives typing
    /// three DISTINCT voices — character, delete, modifier (the
    /// device's keyboard_press_normal/delete/clear; macOS ships
    /// none of them, so Matrix composes its own). ONE switch
    /// ("16-key") rules all three, like the reference's single
    /// "keyboard clicks" toggle. Non-blocking — typing never waits.
    pub fn key_press() {
        if allowed_for("16-key") {
            play("16-key.wav");
        }
    }
    pub fn key_delete() {
        if allowed_for("16-key") {
            play("17-key-delete.wav");
        }
    }
    pub fn key_modifier() {
        if allowed_for("16-key") {
            play("18-key-modifier.wav");
        }
    }

    /// "fertig" — something succeeded (11).
    pub fn success() {
        play("11-done.wav");
    }
    /// "fehler" — something went wrong (12).
    pub fn error() {
        play("12-error.wav");
    }
    /// A notice wants attention (03).
    pub fn hint() {
        play("03-notification.wav");
    }
}

/// Behavior canon (R140) — the "linked on or after" extract.
///
/// Apple freezes old behavior per app via `dyld_sdk_at_least`; a fix
/// never breaks an existing binary. MatrixKit is statically linked, so
/// what needs versioning are the CONTRACT surfaces (files, sockets,
/// helper protocols) where an old app build meets a newer system. The
/// canon ledger records, per app id, which behavior generation the app
/// was built against — the Matrix analog of Apple's per-app exception
/// database, central and healable.
pub mod canon {
    use std::path::PathBuf;

    /// The current behavior generation of the kit. Raise it when a
    /// FAMILY changes observable behavior; gate the change with
    /// `at_least(of(app_id), N)` so older builds keep the old world.
    pub const GENERATION: u32 = 1;

    fn dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/root"));
        PathBuf::from(home).join(".config/matrix/canon")
    }

    /// Record this build's generation in the ledger. Called by the
    /// frame on startup; the entry only ever RISES (an old binary
    /// started later must not lower the recorded canon).
    pub fn record(app_id: &str) {
        let d = dir();
        let _ = std::fs::create_dir_all(&d);
        let path = d.join(app_id);
        if of(app_id) < GENERATION {
            let _ = std::fs::write(path, GENERATION.to_string());
        }
    }

    /// The generation an app last ran with. 0 = pre-canon build
    /// (anything from before R140 — gets the oldest behavior).
    pub fn of(app_id: &str) -> u32 {
        std::fs::read_to_string(dir().join(app_id))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    /// The gate for behavior switches.
    pub fn at_least(recorded: u32, wanted: u32) -> bool {
        recorded >= wanted
    }

    /// Quirk entries (the ArchivingExceptions extract): the SYSTEM may
    /// carry per-app exceptions under /usr/share/matrix/quirks/<app>,
    /// one key per line — centrally healable without touching the app.
    pub fn quirk(app_id: &str, key: &str) -> bool {
        std::fs::read_to_string(format!("/usr/share/matrix/quirks/{app_id}"))
            .is_ok_and(|q| q.lines().any(|l| l.trim() == key))
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn ledger_records_and_never_lowers() {
            // Isolated HOME so the test never touches a real ledger.
            let tmp = std::env::temp_dir().join(format!("canon-test-{}", std::process::id()));
            std::fs::create_dir_all(&tmp).unwrap();
            let old_home = std::env::var("HOME").ok();
            std::env::set_var("HOME", &tmp);

            assert_eq!(super::of("probe-app"), 0);
            super::record("probe-app");
            assert_eq!(super::of("probe-app"), super::GENERATION);
            // A "newer" ledger entry survives an older binary's record().
            std::fs::write(tmp.join(".config/matrix/canon/probe-app"), "999").unwrap();
            super::record("probe-app");
            assert_eq!(super::of("probe-app"), 999);
            assert!(super::at_least(999, super::GENERATION));

            if let Some(h) = old_home {
                std::env::set_var("HOME", h);
            }
            let _ = std::fs::remove_dir_all(&tmp);
        }
    }
}

#[cfg(test)]
mod pressure_tests {
    use super::memory::*;

    #[test]
    fn the_purge_tiers_are_apples_numbers() {
        // kern.memorystatus_purge_on_warning=2, _urgent=5, _critical=8
        assert_eq!(purge_below(Pressure::Low), 0);
        assert_eq!(purge_below(Pressure::Elevated), 2);
        assert_eq!(purge_below(Pressure::High), 8);
    }

    #[test]
    fn cheap_caches_go_first_expensive_ones_last() {
        // A thumbnail (tier 1) already goes at elevated pressure.
        assert!(purge(1, Pressure::Elevated));
        // A decoded image (tier 5) survives elevated, falls at high.
        assert!(!purge(5, Pressure::Elevated));
        assert!(purge(5, Pressure::High));
        // A live index (tier 9) is never dropped by pressure alone.
        assert!(!purge(9, Pressure::High));
        // Calm system: nothing is dropped.
        assert!(!purge(1, Pressure::Low));
    }
}

#[cfg(test)]
mod assertion_tests {
    use super::assertion::*;

    #[test]
    fn a_record_survives_the_round_trip() {
        let line = encode(Kind::Display, "Video läuft", 1_700_000_000);
        let held = decode(42, &line).expect("decodes");
        assert_eq!(held.pid, 42);
        assert_eq!(held.kind, Kind::Display);
        assert_eq!(held.reason, "Video läuft");
        assert_eq!(held.deadline, 1_700_000_000);
    }

    #[test]
    fn a_tab_in_the_reason_cannot_split_the_record() {
        let held = decode(1, &encode(Kind::System, "a\tb", 10)).expect("decodes");
        assert_eq!(held.reason, "a b");
    }

    #[test]
    fn the_two_apple_death_conditions() {
        let h = Held { pid: 1, kind: Kind::Display, reason: String::new(), deadline: 100 };
        // Alive holder, deadline ahead: valid.
        assert!(valid(&h, 99, true));
        // The deadline passed — Apple's TimeoutActionRelease.
        assert!(!valid(&h, 100, true));
        // The holder died — the table emptied the moment caffeinate exited.
        assert!(!valid(&h, 99, false));
    }
}

/// R190 "The room" — extract from the living visionOS SDK
/// (XROS 27.0, SwiftUI module interface, 36 129 lines, dug up
/// on 04.08.2026).
///
/// There are four constructs there that exist nowhere else, and
/// three of them are just as true on a flat surface as in space:
///
/// * `ImmersionStyle` (mixed | progressive | full) — how far the
///   surroundings recede. Not on/off, but a ladder of steps.
/// * `ornament` — controls that stay anchored OUTSIDE the content
///   while the content takes up the room.
/// * `scrollTransition` with `ScrollTransitionPhase`: every element
///   gets ONE number, −1 (out at the bottom) via 0 (centred, fully there)
///   to +1 (out at the top). Opacity, size and offset derive
///   from it. That is the whole magic of "little by little".
/// * `glassBackgroundEffect`, `hoverEffect` — Matrix has had these for a long time
///   (compositor glass, magnifier).
///
/// The guiding principle of the Hardline specification demands exactly this: a
/// document names meaning, the DEVICE chooses the form — "all the way
/// to the room". This family is the choice of form.
/// Stage 3 of the iced replacement: the FOUNDATION STONE of our own
/// layout machine (GRABUNG-ICED-ABLOESUNG.md, "the mountains").
///
/// Deliberately started small: nodes are rows, columns and rectangles;
/// the layout is shrink-wrap with padding and spacing — exactly the
/// grammar that the dock, bars and tiles are made of. No iced
/// type touches this module; it computes pure geometry and is
/// therefore testable like the capability core. Text (cosmic-text) and
/// the state tree come as the next stages — the contract here grows,
/// it does not change.
pub mod scene {
    use super::Rgba;

    /// Stage 64: the SHADOW. It says LAYER, never drama
    /// (DESIGNSYSTEM: "shadows communicate layer, never drama").
    /// The iced version stacks THREE layers because its shadow falls off
    /// almost linearly and looks "far too abrupt" (owner, 8.7.) —
    /// our own rasteriser does not need that: it knows the DISTANCE
    /// to the rounded edge and can lay a soft curve from it in ONE
    /// pass.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Shadow {
        pub color: Rgba,
        pub dx: f32,
        pub dy: f32,
        /// The softness: over this distance the shadow runs
        /// from full to nothing.
        pub blur: f32,
    }

    /// One placed, drawable thing — the OUTPUT of layout.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Placed {
        pub x: f32,
        pub y: f32,
        pub w: f32,
        pub h: f32,
        pub color: Rgba,
        pub radius: f32,
        /// The VIEW BOX (stage 29): if it is set, the rectangle
        /// exists only inside it — for rasteriser, hits and
        /// diff alike. Scrolling lists live on it. The
        /// layout sets None; whoever scrolls clips afterwards.
        pub clip: Option<Rect>,
        /// The shadow under the rectangle (stage 64). The layout
        /// sets None; whoever wants to float sets it AFTER laying out —
        /// the same seam as with the view box.
        pub shadow: Option<Shadow>,
    }

    /// The INPUT: a tree of meaning, sizes come from content.
    pub enum Node {
        Row { gap: f32, padding: f32, background: Option<(Rgba, f32)>, children: Vec<Node> },
        Column { gap: f32, padding: f32, background: Option<(Rgba, f32)>, children: Vec<Node> },
        Quad { w: f32, h: f32, color: Rgba, radius: f32 },
        /// One child, asymmetric padding — the shape of buttons and
        /// pills (fixed height via pad_y, breathing room via pad_x).
        /// Grown for the family move (Lager 6); the contract grows,
        /// it does not change.
        Wrap { pad_x: f32, pad_y: f32, background: Option<(Rgba, f32)>, child: Box<Node> },
        /// The BOX (stage 50): an imposed size, and the child
        /// sits aligned inside it. That is the iced-free answer
        /// to Length::Fill (the app KNOWS its surface and passes the
        /// size in) and to Alignment (the centre is laid out, not
        /// computed). None on an axis means: there the child's size
        /// applies, the box is a pass-through on that axis.
        /// A child that is larger than its box hangs out —
        /// clipping happens via Placed::clip, not here.
        Slot {
            w: Option<f32>,
            h: Option<f32>,
            ax: Align,
            ay: Align,
            background: Option<(Rgba, f32)>,
            child: Box<Node>,
        },
        /// The SCALING (stage 59): the child is laid out and then
        /// stretched by the factor — position, size AND radius. That is the
        /// third missing property after alignment and stretching:
        /// the magnifier (mkw::magnifier, 1.07 on hover), the
        /// entrance growth and every zoom live on it.
        /// The box GROWS along; whoever wants to grow in place
        /// puts the scaling into a slot of fixed size — then
        /// the centre stays the centre.
        /// TEXT is scaled by the family (px × factor); the scene
        /// only knows geometry.
        Scale { factor: f32, child: Box<Node> },
        /// The STACK (stage 50): all children on the SAME origin,
        /// the order is the z-order (later = on top, and the
        /// topmost hit wins — hit() reads the same precedence).
        /// The size is the largest child. Badges over tiles,
        /// veils over content, caret over field live here.
        Stack { children: Vec<Node> },
        /// The OFFSET (stage 109): the child is laid out shifted by (dx, dy),
        /// the size stays that of the child. Scrolling without a box,
        /// panning a view, swiping a row — everywhere
        /// a piece wanders from its origin without the tree
        /// around it knowing. Negative is allowed; clipping happens
        /// as with scrolling via Placed::clip.
        Offset { dx: f32, dy: f32, child: Box<Node> },
    }

    /// Where a child sits in its box (stage 50) — per axis.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Align {
        Start,
        Center,
        End,
    }

    /// The offset of a child in its box. If the child is larger,
    /// the offset becomes negative — centred stays centred, even when it
    /// hangs out on both sides.
    fn offset(align: Align, box_size: f32, child_size: f32) -> f32 {
        match align {
            Align::Start => 0.0,
            Align::Center => (box_size - child_size) / 2.0,
            Align::End => box_size - child_size,
        }
    }

    /// Shrink-wrap layout: returns (width, height) and appends the
    /// placed quads back-to-front (backgrounds before children).
    pub fn lay_out(node: &Node, x: f32, y: f32, out: &mut Vec<Placed>) -> (f32, f32) {
        match node {
            Node::Quad { w, h, color, radius } => {
                out.push(Placed { x, y, w: *w, h: *h, color: *color, radius: *radius, clip: None, shadow: None });
                (*w, *h)
            }
            Node::Row { gap, padding, background, children } => {
                let slot = out.len();
                let (mut cx, mut max_h) = (x + padding, 0.0f32);
                for child in children {
                    let (w, h) = lay_out(child, cx, y + padding, out);
                    cx += w + gap;
                    max_h = max_h.max(h);
                }
                let w = (cx - gap - x + padding).max(2.0 * padding);
                let h = max_h + 2.0 * padding;
                if let Some((color, radius)) = background {
                    out.insert(slot, Placed { x, y, w, h, color: *color, radius: *radius, clip: None, shadow: None });
                }
                (w, h)
            }
            Node::Column { gap, padding, background, children } => {
                let slot = out.len();
                let (mut cy, mut max_w) = (y + padding, 0.0f32);
                for child in children {
                    let (w, h) = lay_out(child, x + padding, cy, out);
                    cy += h + gap;
                    max_w = max_w.max(w);
                }
                let w = max_w + 2.0 * padding;
                let h = (cy - gap - y + padding).max(2.0 * padding);
                if let Some((color, radius)) = background {
                    out.insert(slot, Placed { x, y, w, h, color: *color, radius: *radius, clip: None, shadow: None });
                }
                (w, h)
            }
            Node::Wrap { pad_x, pad_y, background, child } => {
                let slot = out.len();
                let (cw, ch) = lay_out(child, x + pad_x, y + pad_y, out);
                let w = cw + 2.0 * pad_x;
                let h = ch + 2.0 * pad_y;
                if let Some((color, radius)) = background {
                    out.insert(slot, Placed { x, y, w, h, color: *color, radius: *radius, clip: None, shadow: None });
                }
                (w, h)
            }
            Node::Slot { w, h, ax, ay, background, child } => {
                // Aligning means: knowing the child's size BEFORE its
                // position. The single pass remains — the child is
                // laid out at the origin and then moved to its place
                // (Placed carries its position, the moving
                // is exact).
                let mut inner = Vec::new();
                let (cw, ch) = lay_out(child, 0.0, 0.0, &mut inner);
                let bw = w.unwrap_or(cw);
                let bh = h.unwrap_or(ch);
                if let Some((color, radius)) = background {
                    out.push(Placed { x, y, w: bw, h: bh, color: *color, radius: *radius, clip: None, shadow: None });
                }
                let dx = x + offset(*ax, bw, cw);
                let dy = y + offset(*ay, bh, ch);
                for mut placed in inner {
                    placed.x += dx;
                    placed.y += dy;
                    out.push(placed);
                }
                (bw, bh)
            }
            Node::Stack { children } => {
                let (mut max_w, mut max_h) = (0.0f32, 0.0f32);
                for child in children {
                    let (w, h) = lay_out(child, x, y, out);
                    max_w = max_w.max(w);
                    max_h = max_h.max(h);
                }
                (max_w, max_h)
            }
            Node::Scale { factor, child } => {
                // The same move as with the box: first lay out at the
                // origin, then transform the finished Placed — here
                // stretched instead of moved. A negative or
                // tiny factor folds nothing: it gets clamped.
                let f = factor.max(0.0);
                let mut inner = Vec::new();
                let (cw, ch) = lay_out(child, 0.0, 0.0, &mut inner);
                for mut placed in inner {
                    placed.x = x + placed.x * f;
                    placed.y = y + placed.y * f;
                    placed.w *= f;
                    placed.h *= f;
                    placed.radius *= f;
                    if let Some(c) = placed.clip.as_mut() {
                        c.x = x + c.x * f;
                        c.y = y + c.y * f;
                        c.w *= f;
                        c.h *= f;
                    }
                    out.push(placed);
                }
                (cw * f, ch * f)
            }
            Node::Offset { dx, dy, child } => lay_out(child, x + dx, y + dy, out),
        }
    }

    /// How many Placed a tree will lay — the bookkeeping seam for
    /// composed families (Lager 8): whoever assembles a scene from
    /// building blocks can address their quads without laying out
    /// first. Contract: lay_out() appends EXACTLY this many.
    pub fn placed_count(node: &Node) -> usize {
        match node {
            Node::Quad { .. } => 1,
            Node::Row { background, children, .. }
            | Node::Column { background, children, .. } => {
                usize::from(background.is_some())
                    + children.iter().map(placed_count).sum::<usize>()
            }
            Node::Wrap { background, child, .. } => {
                usize::from(background.is_some()) + placed_count(child)
            }
            Node::Slot { background, child, .. } => {
                usize::from(background.is_some()) + placed_count(child)
            }
            Node::Stack { children } => children.iter().map(placed_count).sum::<usize>(),
            Node::Scale { child, .. } => placed_count(child),
            Node::Offset { child, .. } => placed_count(child),
        }
    }

    /// The per-widget state shelf (Lager 14): one shelf per KIND of
    /// state (scroll, focus, …), keyed by stable widget identities —
    /// the same idea as the fleet's structure signatures. visit()
    /// inside view() names a widget and yields its state (Default on
    /// first use); sweep() after view() drops everything the tree no
    /// longer names. State lives EXACTLY as long as its widget —
    /// surviving every rebuild in between, like iced's widget tree.
    pub struct Keep<S> {
        store: std::collections::HashMap<u64, (S, bool)>,
    }

    impl<S: Default> Keep<S> {
        pub fn new() -> Self {
            Keep { store: std::collections::HashMap::new() }
        }

        /// Name the widget and get its state — marks it alive for
        /// this frame. First use creates the Default.
        pub fn visit(&mut self, key: u64) -> &mut S {
            let slot = self.store.entry(key).or_insert_with(|| (S::default(), false));
            slot.1 = true;
            &mut slot.0
        }

        /// Mutate WITHOUT naming (input handlers between frames) —
        /// a widget the tree dropped stays dropped.
        pub fn get_mut(&mut self, key: u64) -> Option<&mut S> {
            self.store.get_mut(&key).map(|(s, _)| s)
        }

        /// After view(): drop what the tree no longer names, clear
        /// the marks for the next frame.
        pub fn sweep(&mut self) {
            self.store.retain(|_, (_, alive)| *alive);
            for slot in self.store.values_mut() {
                slot.1 = false;
            }
        }
    }

    impl<S: Default> Default for Keep<S> {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Hit test: does (x, y) land inside a placed quad? Purely
    /// geometric, using the SAME rounded-corner distance the
    /// rasterizer paints with — what looks hit IS hit. Layout paints
    /// back-to-front, so the LAST match wins: the topmost element.
    /// A quad's Sichtkasten binds here too (Lager 29): what the
    /// rasterizer cuts away cannot be hit.
    pub fn hit(placed: &[Placed], x: f32, y: f32) -> Option<usize> {
        placed.iter().rposition(|p| {
            contains(p, x, y)
                && p.clip.map_or(true, |c| {
                    x >= c.x && x < c.x + c.w && y >= c.y && y < c.y + c.h
                })
        })
    }

    /// An axis-aligned rectangle — the currency of damage.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Rect {
        pub x: f32,
        pub y: f32,
        pub w: f32,
        pub h: f32,
    }

    impl Rect {
        /// The hull of a placed quad, one pixel wider on every side
        /// — the rasterizer smooths edges into that pixel.
        pub fn of(p: &Placed) -> Rect {
            let base = Rect { x: p.x - 1.0, y: p.y - 1.0, w: p.w + 2.0, h: p.h + 2.0 };
            // The shadow belongs to the damage: whoever does not include it in the
            // hull leaves it standing in the diff — it smeared
            // across the surface (stage 64).
            match p.shadow {
                None => base,
                Some(sh) => base.union(&Rect {
                    x: p.x + sh.dx - sh.blur - 1.0,
                    y: p.y + sh.dy - sh.blur - 1.0,
                    w: p.w + 2.0 * sh.blur + 2.0,
                    h: p.h + 2.0 * sh.blur + 2.0,
                }),
            }
        }

        pub fn intersects(&self, o: &Rect) -> bool {
            self.x < o.x + o.w && o.x < self.x + self.w
                && self.y < o.y + o.h && o.y < self.y + self.h
        }

        pub fn union(&self, o: &Rect) -> Rect {
            let x = self.x.min(o.x);
            let y = self.y.min(o.y);
            Rect {
                x,
                y,
                w: (self.x + self.w).max(o.x + o.w) - x,
                h: (self.y + self.h).max(o.y + o.h) - y,
            }
        }

        pub fn area(&self) -> f32 {
            self.w * self.h
        }

        /// The intersection of two rectangles — None if nothing remains
        /// (stage 29: the currency of the view box).
        pub fn cut(&self, o: &Rect) -> Option<Rect> {
            let x = self.x.max(o.x);
            let y = self.y.max(o.y);
            let x1 = (self.x + self.w).min(o.x + o.w);
            let y1 = (self.y + self.h).min(o.y + o.h);
            if x1 > x && y1 > y {
                Some(Rect { x, y, w: x1 - x, h: y1 - y })
            } else {
                None
            }
        }
    }

    /// The VISIBLE hull of a rectangle (stage 29): its hull,
    /// clipped at the view box — None if it scrolled out completely.
    /// The box is widened like the hull by the anti-aliasing pixel.
    pub fn hull(p: &Placed) -> Option<Rect> {
        let r = Rect::of(p);
        match p.clip {
            None => Some(r),
            Some(c) => r.cut(&Rect { x: c.x - 1.0, y: c.y - 1.0, w: c.w + 2.0, h: c.h + 2.0 }),
        }
    }

    /// The diff: damage between two laid-out scenes. Returns the
    /// hulls of every placed that vanished, appeared or changed —
    /// and of unchanged pairs whose PAINT ORDER flipped while they
    /// overlap (alpha blending is not commutative). Identical
    /// scenes return NOTHING: stillness costs nothing. That is the
    /// whole point of a retained scene — the old frame is the state
    /// tree, the new frame is compared against it, and only the
    /// difference is ever painted or sent to the compositor.
    pub fn diff(old: &[Placed], new: &[Placed]) -> Vec<Rect> {
        let mut damage = Vec::new();
        // Match value-equal quads pairwise (multiset alignment).
        let mut pair: Vec<Option<usize>> = vec![None; old.len()];
        let mut taken = vec![false; new.len()];
        // Damage counts only what is VISIBLE (Lager 29): the hull
        // cut by the Sichtkasten — a quad that rolled out entirely
        // has nothing to repaint.
        for (i, p) in old.iter().enumerate() {
            match (0..new.len()).find(|&j| !taken[j] && new[j] == *p) {
                Some(j) => {
                    pair[i] = Some(j);
                    taken[j] = true;
                }
                None => damage.extend(hull(p)),
            }
        }
        for (j, q) in new.iter().enumerate() {
            if !taken[j] {
                damage.extend(hull(q));
            }
        }
        // Order flips among surviving, overlapping quads.
        let kept: Vec<(usize, usize)> =
            pair.iter().enumerate().filter_map(|(i, j)| j.map(|j| (i, j))).collect();
        for a in 0..kept.len() {
            for b in a + 1..kept.len() {
                let (ia, ja) = kept[a];
                let (ib, jb) = kept[b];
                if (ia < ib) != (ja < jb) {
                    if let (Some(ha), Some(hb)) = (hull(&old[ia]), hull(&old[ib])) {
                        if ha.intersects(&hb) {
                            damage.push(ha);
                            damage.push(hb);
                        }
                    }
                }
            }
        }
        coalesce(damage)
    }

    /// Merge overlapping rectangles until no two intersect — the
    /// painter wants disjoint patches. Conservative by design: a
    /// union may cover unchanged pixels; repainting them is correct,
    /// merely unnecessary.
    pub fn coalesce(mut rects: Vec<Rect>) -> Vec<Rect> {
        let mut merged = true;
        while merged {
            merged = false;
            'outer: for a in 0..rects.len() {
                for b in a + 1..rects.len() {
                    if rects[a].intersects(&rects[b]) {
                        rects[a] = rects[a].union(&rects[b]);
                        rects.swap_remove(b);
                        merged = true;
                        break 'outer;
                    }
                }
            }
        }
        rects
    }

    fn contains(p: &Placed, x: f32, y: f32) -> bool {
        if x < p.x || y < p.y || x >= p.x + p.w || y >= p.y + p.h {
            return false;
        }
        let r = p.radius.min(p.w / 2.0).min(p.h / 2.0);
        if r <= 0.0 {
            return true;
        }
        // Same signed-distance idea as the rasterizer: distance to
        // the inner box, compared against the corner radius.
        let dx = ((x - (p.x + p.w / 2.0)).abs() - (p.w / 2.0 - r)).max(0.0);
        let dy = ((y - (p.y + p.h / 2.0)).abs() - (p.h / 2.0 - r)).max(0.0);
        dx * dx + dy * dy <= r * r
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn q(w: f32, h: f32) -> Node {
            Node::Quad { w, h, color: Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, radius: 0.0 }
        }

        #[test]
        fn row_wraps_children_with_gap_and_padding() {
            let row = Node::Row { gap: 10.0, padding: 8.0, background: None,
                children: vec![q(40.0, 20.0), q(40.0, 30.0), q(40.0, 20.0)] };
            let mut out = Vec::new();
            let (w, h) = lay_out(&row, 0.0, 0.0, &mut out);
            assert_eq!(w, 3.0 * 40.0 + 2.0 * 10.0 + 2.0 * 8.0);
            assert_eq!(h, 30.0 + 2.0 * 8.0);
            assert_eq!(out.len(), 3);
            assert_eq!(out[0].x, 8.0);
            assert_eq!(out[1].x, 8.0 + 40.0 + 10.0);
        }

        #[test]
        fn background_is_painted_first_and_spans_the_wrap() {
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let row = Node::Row { gap: 0.0, padding: 4.0, background: Some((white, 9.0)),
                children: vec![q(10.0, 10.0)] };
            let mut out = Vec::new();
            let (w, h) = lay_out(&row, 5.0, 5.0, &mut out);
            assert_eq!(out.len(), 2);
            assert_eq!((out[0].w, out[0].h), (w, h));
            assert_eq!((out[0].x, out[0].y), (5.0, 5.0));
            assert_eq!(out[0].radius, 9.0);
        }

        #[test]
        fn nesting_stacks_columns_inside_rows() {
            let col = Node::Column { gap: 2.0, padding: 0.0, background: None,
                children: vec![q(10.0, 10.0), q(10.0, 10.0)] };
            let row = Node::Row { gap: 0.0, padding: 0.0, background: None,
                children: vec![col, q(6.0, 4.0)] };
            let mut out = Vec::new();
            let (w, h) = lay_out(&row, 0.0, 0.0, &mut out);
            assert_eq!(w, 16.0);
            assert_eq!(h, 22.0);
            assert_eq!(out[2].x, 10.0);
        }

        #[test]
        fn slot_lays_the_middle_instead_of_computing_it() {
            // The box is 100×40, the child 20×10: centred means
            // (40, 15) — exactly the calculation the families do
            // by hand today.
            let slot = Node::Slot { w: Some(100.0), h: Some(40.0),
                ax: Align::Center, ay: Align::Center, background: None,
                child: Box::new(q(20.0, 10.0)) };
            let mut out = Vec::new();
            let (w, h) = lay_out(&slot, 0.0, 0.0, &mut out);
            assert_eq!((w, h), (100.0, 40.0), "das auferlegte Maß gilt");
            assert_eq!(out.len(), 1);
            assert_eq!((out[0].x, out[0].y), (40.0, 15.0));
        }

        #[test]
        fn slot_without_a_measure_passes_the_child_through() {
            // Without a size the box is a pass-through — that way
            // families can adopt it without changing their face.
            let slot = Node::Slot { w: None, h: None,
                ax: Align::Center, ay: Align::End, background: None,
                child: Box::new(q(24.0, 12.0)) };
            let mut bare = Vec::new();
            let mut boxed = Vec::new();
            let a = lay_out(&q(24.0, 12.0), 7.0, 3.0, &mut bare);
            let b = lay_out(&slot, 7.0, 3.0, &mut boxed);
            assert_eq!(a, b);
            assert_eq!((bare[0].x, bare[0].y), (boxed[0].x, boxed[0].y));
        }

        #[test]
        fn slot_ends_flush_with_the_far_edge() {
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let slot = Node::Slot { w: Some(80.0), h: Some(20.0),
                ax: Align::End, ay: Align::Start, background: Some((white, 4.0)),
                child: Box::new(q(30.0, 20.0)) };
            let mut out = Vec::new();
            lay_out(&slot, 0.0, 0.0, &mut out);
            // Background first (it carries the box size), then the child.
            assert_eq!(out.len(), 2);
            assert_eq!((out[0].w, out[0].h), (80.0, 20.0));
            assert_eq!(out[1].x, 50.0, "rechtsbündig: 80 − 30");
            assert_eq!(out[1].y, 0.0);
        }

        #[test]
        fn scale_stretches_place_measure_and_radius() {
            // A child at (10,10), 20×10, radius 4 — at 1.5
            // EVERYTHING is stretched, also the position in the tree and the radius.
            let inner = Node::Row { gap: 0.0, padding: 10.0, background: None,
                children: vec![Node::Quad { w: 20.0, h: 10.0,
                    color: Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, radius: 4.0 }] };
            let mut plain = Vec::new();
            let (pw, ph) = lay_out(&inner, 0.0, 0.0, &mut plain);
            let mut big = Vec::new();
            let (bw, bh) = lay_out(&Node::Scale { factor: 1.5, child: Box::new(inner) }, 0.0, 0.0, &mut big);
            assert_eq!((bw, bh), (pw * 1.5, ph * 1.5), "der Kasten wächst mit");
            assert_eq!(big.len(), plain.len(), "die Zählung bleibt");
            assert_eq!(big[0].x, plain[0].x * 1.5);
            assert_eq!((big[0].w, big[0].h), (plain[0].w * 1.5, plain[0].h * 1.5));
            assert_eq!(big[0].radius, plain[0].radius * 1.5, "auch die Rundung");
            // Factor 1 changes nothing — the magnifier must not
            // secretly shift anything when at rest.
            let mut same = Vec::new();
            let node = Node::Quad { w: 8.0, h: 8.0, color: Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, radius: 2.0 };
            lay_out(&Node::Scale { factor: 1.0, child: Box::new(node) }, 3.0, 4.0, &mut same);
            assert_eq!((same[0].x, same[0].y, same[0].w), (3.0, 4.0, 8.0));
        }

        #[test]
        fn stack_shares_one_origin_and_the_top_wins() {
            // Two rectangles on the same spot: the size is the
            // largest, the order is the z-order, and the
            // hit goes to the upper one — the same precedence rule
            // the painter uses when blending.
            let stack = Node::Stack { children: vec![q(60.0, 30.0), q(20.0, 20.0)] };
            let mut out = Vec::new();
            let (w, h) = lay_out(&stack, 5.0, 5.0, &mut out);
            assert_eq!((w, h), (60.0, 30.0));
            assert_eq!(out.len(), 2);
            assert_eq!((out[0].x, out[0].y), (5.0, 5.0));
            assert_eq!((out[1].x, out[1].y), (5.0, 5.0));
            assert_eq!(hit(&out, 10.0, 10.0), Some(1), "das obere gewinnt");
            assert_eq!(hit(&out, 50.0, 20.0), Some(0), "daneben trägt das untere");
        }

        #[test]
        fn hit_respects_rounded_corners() {
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let p = [Placed { x: 0.0, y: 0.0, w: 100.0, h: 100.0, color: white, radius: 20.0, clip: None, shadow: None }];
            // Centre and edge centre: inside.
            assert_eq!(hit(&p, 50.0, 50.0), Some(0));
            assert_eq!(hit(&p, 50.0, 1.0), Some(0));
            // The square bounding box would hit (2,2) — the round corner
            // cuts it away. On the rounding (10,10) it hits.
            assert_eq!(hit(&p, 2.0, 2.0), None);
            assert_eq!(hit(&p, 10.0, 10.0), Some(0));
            // Outside the box: never.
            assert_eq!(hit(&p, 150.0, 50.0), None);
            assert_eq!(hit(&p, 100.0, 50.0), None);
        }

        #[test]
        fn topmost_placed_wins_the_hit() {
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let back = Placed { x: 0.0, y: 0.0, w: 200.0, h: 100.0, color: white, radius: 0.0, clip: None, shadow: None };
            let tile = Placed { x: 20.0, y: 20.0, w: 50.0, h: 50.0, color: white, radius: 8.0, clip: None, shadow: None };
            // Painting goes from back to front: background first.
            let p = [back, tile];
            // On the tile the tile wins, beside it the field.
            assert_eq!(hit(&p, 40.0, 40.0), Some(1));
            assert_eq!(hit(&p, 150.0, 40.0), Some(0));
            assert_eq!(hit(&p, -1.0, 40.0), None);
        }

        #[test]
        fn state_lives_exactly_as_long_as_its_widget() {
            let mut keep: Keep<f32> = Keep::new();
            // First use: Default; then the state is preserved across
            // any number of view() rebuilds …
            assert_eq!(*keep.visit(7), 0.0);
            *keep.visit(7) = 42.0;
            keep.sweep();
            assert_eq!(*keep.visit(7), 42.0, "der Neubau vergisst nichts");
            keep.sweep();
            // … input handlers change WITHOUT naming …
            *keep.get_mut(7).expect("lebt") = 43.0;
            assert_eq!(*keep.visit(7), 43.0);
            keep.sweep();
            // … and whoever the tree no longer names gets swept by sweep.
            keep.sweep();
            assert!(keep.get_mut(7).is_none(), "Zustand stirbt mit dem Widget");
            assert_eq!(*keep.visit(7), 0.0, "Wiedergeburt beginnt frisch");
        }

        #[test]
        fn placed_count_matches_lay_out() {
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let tree = Node::Column {
                gap: 4.0,
                padding: 8.0,
                background: Some((white, 12.0)),
                children: vec![
                    Node::Row { gap: 2.0, padding: 0.0, background: None,
                        children: vec![q(10.0, 10.0), q(10.0, 10.0)] },
                    Node::Wrap { pad_x: 3.0, pad_y: 1.0, background: Some((white, 15.0)),
                        child: Box::new(q(24.0, 24.0)) },
                    // Stage 50: box and stack are under
                    // the same contract — the count IS the layout.
                    Node::Slot { w: Some(60.0), h: Some(20.0),
                        ax: Align::Center, ay: Align::Center, background: Some((white, 6.0)),
                        child: Box::new(q(10.0, 10.0)) },
                    Node::Scale { factor: 1.5, child: Box::new(q(10.0, 10.0)) },
                    Node::Stack { children: vec![
                        q(30.0, 30.0),
                        Node::Slot { w: None, h: None, ax: Align::Start, ay: Align::Start,
                            background: None, child: Box::new(q(8.0, 8.0)) },
                    ] },
                ],
            };
            let mut out = Vec::new();
            lay_out(&tree, 0.0, 0.0, &mut out);
            assert_eq!(out.len(), placed_count(&tree), "die Zählung IST der Vertrag");
        }

        #[test]
        fn wrap_pads_asymmetrically_and_grounds_first() {
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let node = Node::Wrap {
                pad_x: 16.0,
                pad_y: 13.0,
                background: Some((white, 12.0)),
                child: Box::new(q(50.0, 14.0)),
            };
            let mut out = Vec::new();
            let (w, h) = lay_out(&node, 5.0, 5.0, &mut out);
            // Button shape: width breathes (pad_x), height is the beat (pad_y).
            assert_eq!(w, 50.0 + 32.0);
            assert_eq!(h, 14.0 + 26.0);
            assert_eq!(out.len(), 2);
            assert_eq!((out[0].x, out[0].y, out[0].w, out[0].h), (5.0, 5.0, w, h));
            assert_eq!((out[1].x, out[1].y), (21.0, 18.0));
        }

        #[test]
        fn stillness_costs_nothing() {
            // Identical scenes: the diff is EMPTY — no
            // painting, no damage_buffer, no work.
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let scene = [
                Placed { x: 0.0, y: 0.0, w: 200.0, h: 80.0, color: white, radius: 20.0, clip: None, shadow: None },
                Placed { x: 10.0, y: 10.0, w: 50.0, h: 50.0, color: white, radius: 8.0, clip: None, shadow: None },
            ];
            assert!(diff(&scene, &scene).is_empty());
        }

        #[test]
        fn damage_covers_exactly_the_change() {
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let red = Rgba { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
            let bg = Placed { x: 0.0, y: 0.0, w: 400.0, h: 100.0, color: white, radius: 0.0, clip: None, shadow: None };
            let t1 = Placed { x: 20.0, y: 20.0, w: 50.0, h: 50.0, color: white, radius: 8.0, clip: None, shadow: None };
            let t2 = Placed { x: 200.0, y: 20.0, w: 50.0, h: 50.0, color: white, radius: 8.0, clip: None, shadow: None };
            let t2_red = Placed { color: red, ..t2 };
            let d = diff(&[bg, t1, t2], &[bg, t1, t2_red]);
            // One patch, and it encloses ONLY the recoloured
            // tile (plus the 1 px anti-aliasing border).
            assert_eq!(d.len(), 1);
            assert_eq!(d[0], Rect::of(&t2));
            assert!(d[0].area() < 0.1 * (400.0 * 100.0));
        }

        #[test]
        fn a_move_damages_both_shores() {
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let old = Placed { x: 0.0, y: 0.0, w: 40.0, h: 40.0, color: white, radius: 0.0, clip: None, shadow: None };
            let new = Placed { x: 300.0, y: 0.0, ..old };
            let d = diff(&[old], &[new]);
            // Disjunkter Umzug: altes Ufer freilegen, neues malen.
            assert_eq!(d.len(), 2);
            assert!(d.contains(&Rect::of(&old)));
            assert!(d.contains(&Rect::of(&new)));
        }

        #[test]
        fn paint_order_is_part_of_the_picture() {
            // Alpha blending is not commutative: if two
            // overlapping rectangles swap their paint order, that is
            // damage — even if both stayed "the same".
            let a = Placed { x: 0.0, y: 0.0, w: 60.0, h: 60.0,
                color: Rgba { r: 1.0, g: 0.0, b: 0.0, a: 0.5 }, radius: 0.0, clip: None, shadow: None };
            let b = Placed { x: 30.0, y: 30.0, w: 60.0, h: 60.0,
                color: Rgba { r: 0.0, g: 0.0, b: 1.0, a: 0.5 }, radius: 0.0, clip: None, shadow: None };
            let d = diff(&[a, b], &[b, a]);
            assert!(!d.is_empty());
            // Getrennt liegende duerfen tauschen: niemand sieht es.
            let far = Placed { x: 500.0, y: 0.0, ..b };
            assert!(diff(&[a, far], &[far, a]).is_empty());
        }

        #[test]
        fn the_clip_binds_hit_diff_and_hull() {
            // Stage 29: the view box. What the rasteriser
            // cuts away does not hit and does not damage.
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let pane = Rect { x: 0.0, y: 50.0, w: 200.0, h: 100.0 };
            let card = Placed {
                x: 10.0, y: 20.0, w: 100.0, h: 20.0,
                color: white, radius: 8.0, clip: Some(pane), shadow: None,
            };
            // The card lies entirely above the edge: invisible.
            assert_eq!(hull(&card), None, "hinausgerollt = keine Hülle");
            assert_eq!(hit(&[card], 30.0, 30.0), None, "unsichtbar trifft nicht");
            // Half in the box: hull = the intersection, hits only at the bottom.
            let half = Placed { y: 40.0, h: 40.0, ..card };
            let h = hull(&half).expect("halb sichtbar hat eine Hülle");
            assert!(h.y >= 49.0 && h.h < 40.0, "die Hülle beginnt am Kasten");
            assert_eq!(hit(&[half], 30.0, 45.0), None, "der Kopf über der Kante ist taub");
            assert_eq!(hit(&[half], 30.0, 60.0), Some(0), "der Leib im Kasten trifft");
            // Diff: the one that vanished completely damages nothing.
            let d = diff(&[card], &[]);
            assert!(d.is_empty(), "ohne Hülle kein Schaden");
            let d = diff(&[half], &[]);
            assert_eq!(d.len(), 1, "die halbe beschädigt ihren Schnitt");
            assert!(d[0].y >= 49.0, "der Schaden bleibt im Kasten");
        }

        #[test]
        fn coalesce_leaves_disjoint_patches() {
            let r = |x: f32| Rect { x, y: 0.0, w: 30.0, h: 30.0 };
            // Eine Kette Ueberlapper verschmilzt zu einem Fleck …
            let merged = coalesce(vec![r(0.0), r(20.0), r(40.0)]);
            assert_eq!(merged.len(), 1);
            assert_eq!((merged[0].x, merged[0].w), (0.0, 70.0));
            // … what is separate stays separate.
            let apart = coalesce(vec![r(0.0), r(100.0)]);
            assert_eq!(apart.len(), 2);
            for a in 0..apart.len() {
                for b in a + 1..apart.len() {
                    assert!(!apart[a].intersects(&apart[b]));
                }
            }
        }

        #[test]
        fn a_layouted_scene_is_hittable() {
            // The seam's contract: what lay_out lays out, hit finds —
            // the order of the Placed list is the z-order.
            let white = Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
            let row = Node::Row { gap: 10.0, padding: 8.0, background: Some((white, 0.0)),
                children: vec![q(40.0, 20.0), q(40.0, 20.0)] };
            let mut out = Vec::new();
            lay_out(&row, 0.0, 0.0, &mut out);
            // out[0] = Hintergrund, out[1] = erste, out[2] = zweite Kachel.
            assert_eq!(hit(&out, 10.0, 10.0), Some(1));
            assert_eq!(hit(&out, 60.0, 10.0), Some(2));
            // In the gap between the tiles: the field.
            assert_eq!(hit(&out, 50.0, 10.0), Some(0));
        }
    }
}

pub mod spatial {
    /// How far the surroundings recede (visionOS: ImmersionStyle).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Immersion {
        /// Windows among windows — everyday life.
        Window,
        /// Reading version: the content gets air, the surroundings dim.
        Reading,
        /// The room: only content left, surroundings almost gone. The
        /// ornaments (bars) stay — that is exactly the point of the
        /// term at Apple.
        Space,
    }

    impl Immersion {
        /// How strongly the background is darkened (0…1).
        /// Measured Apple feel: "slight" means slight — at
        /// full immersion the surroundings remain perceptible, otherwise
        /// the human loses orientation in the room.
        pub fn dim(self) -> f32 {
            match self {
                Immersion::Window => 0.0,
                Immersion::Reading => 0.35,
                Immersion::Space => 0.72,
            }
        }

        /// Factor on the title's font size. In the room the
        /// title becomes a stage (presentation effect).
        pub fn title_scale(self) -> f32 {
            match self {
                Immersion::Window => 1.0,
                Immersion::Reading => 1.4,
                Immersion::Space => 2.6,
            }
        }

        /// Air between the elements — in the room everything breathes wider.
        pub fn breath(self) -> f32 {
            match self {
                Immersion::Window => 1.0,
                Immersion::Reading => 1.6,
                Immersion::Space => 2.4,
            }
        }

        pub fn label(self) -> &'static str {
            match self {
                Immersion::Window => "Fenster",
                Immersion::Reading => "Lesen",
                Immersion::Space => "Raum",
            }
        }
    }

    /// visionOS' `ScrollTransitionPhase.value`, rebuilt: where does
    /// an element stand in the viewport? −1 = still out at the bottom,
    /// 0 = centred and fully there, +1 = out at the top.
    ///
    /// `center` and `height` in the same unit (pixels in the
    /// viewport). Outside it is clamped, never extrapolated.
    pub fn phase(center: f32, height: f32) -> f32 {
        if height <= 0.0 {
            return 0.0;
        }
        // The zone in which an element "arrives": the middle
        // third is identity, around it the transition.
        let mid = height * 0.5;
        let reach = height * 0.5;
        ((mid - center) / reach).clamp(-1.0, 1.0)
    }

    /// Opacity from the phase. Apple does NOT let elements vanish
    /// completely — whoever scrolls should see that there is still something
    /// there. 0.15 as the floor is the feel of their templates.
    pub fn fade(phase: f32) -> f32 {
        let d = phase.abs();
        (1.0 - d * d).max(0.0) * 0.85 + 0.15
    }

    /// Size from the phase: what arrives grows slightly to its
    /// true size — that creates spatial depth without needing
    /// depth.
    pub fn rise(phase: f32) -> f32 {
        1.0 - phase.abs() * 0.12
    }

    /// Offset from the phase, in pixels: elements come in from below
    /// and leave at the top.
    pub fn drift(phase: f32, amount: f32) -> f32 {
        phase * amount
    }

    /// R191: the proof that the effect machine holds. Apple's
    /// `scrollTransition` is NOT a system of its own — it is a
    /// `visualEffect`, fed from the phase. Exactly so here: the
    /// entrance rule is a rule like any other, and it falls
    /// under the same contract (does not touch the layout).
    pub fn appearance(g: crate::effect::Geometry, drift_px: f32) -> crate::effect::Effect {
        use crate::effect::{Effect, Wirkung};
        let p = phase(g.center_y, g.space_height);
        Effect::none()
            .with(Wirkung::Opacity(fade(p)))
            .with(Wirkung::Scale(rise(p)))
            .with(Wirkung::Offset(0.0, drift(p, drift_px)))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn phase_is_the_apple_range() {
            // Centred = identity, top/bottom = the edges.
            assert!((phase(500.0, 1000.0) - 0.0).abs() < 0.001);
            assert!((phase(0.0, 1000.0) - 1.0).abs() < 0.001);
            assert!((phase(1000.0, 1000.0) + 1.0).abs() < 0.001);
            // Far outside it is clamped, never extrapolated.
            assert_eq!(phase(-9000.0, 1000.0), 1.0);
            assert_eq!(phase(9000.0, 1000.0), -1.0);
            assert_eq!(phase(10.0, 0.0), 0.0);
        }

        #[test]
        fn nothing_ever_vanishes_completely() {
            // Apple leaves residual opacity at the edge — otherwise
            // the list looks torn instead of deep.
            for p in [-1.0, -0.5, 0.0, 0.5, 1.0] {
                assert!(fade(p) >= 0.15, "phase {p} verschwand ganz");
                assert!(fade(p) <= 1.0);
            }
            assert!(fade(0.0) > fade(1.0), "mittig muss am hellsten sein");
        }

        #[test]
        fn the_scroll_appearance_is_just_a_visual_effect() {
            use crate::effect::Geometry;
            let space = 1000.0;
            let mid = Geometry {
                width: 400.0, height: 80.0, center_x: 200.0,
                center_y: 500.0, space_height: space,
            };
            let edge = Geometry { center_y: 0.0, ..mid };
            let a = appearance(mid, 40.0);
            let b = appearance(edge, 40.0);
            // Centred is brighter, larger and without offset.
            assert!(a.opacity() > b.opacity());
            assert!(a.scale() > b.scale());
            assert!(a.offset().1.abs() < b.offset().1.abs());
            // And it falls under the family's contract.
            assert!(!a.touches_layout());
            assert!(!b.touches_layout());
        }

        #[test]
        fn immersion_is_a_ladder_not_a_switch() {
            let steps = [Immersion::Window, Immersion::Reading, Immersion::Space];
            for pair in steps.windows(2) {
                assert!(pair[1].dim() > pair[0].dim());
                assert!(pair[1].title_scale() > pair[0].title_scale());
                assert!(pair[1].breath() > pair[0].breath());
            }
            // Even in the room the surroundings remain perceptible (orientation).
            assert!(Immersion::Space.dim() < 0.85);
        }
    }
}

/// R191 "The effect machine" — extract from visionOS/SwiftUICore
/// (XROS 27.0, dug up on 04.08.2026).
///
/// Apple's find in one sentence:
///
/// ```text
/// visualEffect { effect, geometry in ... }   geometry in, effect out
/// ```
///
/// An element gets ITS OWN geometry and returns a
/// visual effect from it — from a **closed list**. And
/// the decisive contract: **these effects do not change the layout.**
/// The element stays where it lies and as large as it is;
/// only its appearance is transformed.
///
/// That is why Apple could build `scrollTransition` at all — it IS
/// merely a `visualEffect`, fed from the scroll phase. The same
/// machine produces parallax, focus lift, fade-in at the edge.
///
/// Why the kit gets it: magnifier, springs and scroll fade are
/// one-offs today, each with its own mechanics. Here is ONE
/// machine whose freedom from layout changes can be checked.
pub mod effect {
    /// What an element knows about itself (SwiftUI:
    /// `GeometryProxy`). Deliberately small: size and position in the
    /// surrounding space — none of the 16 effects needs more.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Geometry {
        pub width: f32,
        pub height: f32,
        /// Position of the centre in the reference space (e.g. in the viewport).
        pub center_x: f32,
        pub center_y: f32,
        /// Height of the reference space — denominator for all fractions.
        pub space_height: f32,
    }

    impl Geometry {
        /// Fraction 0…1: how far down in the reference space is the centre?
        pub fn fraction(&self) -> f32 {
            if self.space_height <= 0.0 {
                return 0.5;
            }
            (self.center_y / self.space_height).clamp(0.0, 1.0)
        }
    }

    /// The closed list, literally from SwiftUICore::VisualEffect.
    /// Closed is intentional — whoever needs a 17th effect must
    /// add it to the family, not invent it on the side.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Wirkung {
        Opacity(f32),
        Scale(f32),
        Offset(f32, f32),
        Blur(f32),
        Brightness(f32),
        Contrast(f32),
        Saturation(f32),
        Grayscale(f32),
        HueRotation(f32),
        Rotation(f32),
    }

    /// A stack of effects — `combining` at Apple. The order
    /// is preserved because it matters for blur and scaling.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Effect {
        pub steps: Vec<Wirkung>,
    }

    impl Effect {
        pub fn none() -> Self {
            Self::default()
        }

        pub fn with(mut self, w: Wirkung) -> Self {
            self.steps.push(w);
            self
        }

        /// The whole contract in one function: what an effect
        /// adds up to in opacity/size/offset.
        /// If a kind is missing, the neutral value applies.
        pub fn opacity(&self) -> f32 {
            self.steps
                .iter()
                .filter_map(|w| match w {
                    Wirkung::Opacity(v) => Some(*v),
                    _ => None,
                })
                .product::<f32>()
                .clamp(0.0, 1.0)
        }

        pub fn scale(&self) -> f32 {
            self.steps
                .iter()
                .filter_map(|w| match w {
                    Wirkung::Scale(v) => Some(*v),
                    _ => None,
                })
                .product()
        }

        pub fn offset(&self) -> (f32, f32) {
            self.steps.iter().fold((0.0, 0.0), |(x, y), w| match w {
                Wirkung::Offset(dx, dy) => (x + dx, y + dy),
                _ => (x, y),
            })
        }

        /// THE contract: an effect does not touch the layout. There
        /// is no kind that changes width or height — this
        /// check is the executable version of the promise.
        pub fn touches_layout(&self) -> bool {
            false
        }
    }

    /// Rebuild of `visualEffect`: geometry in, effect out.
    /// The function is the whole seam — callers pass their rule
    /// as a closure, the kit provides the geometry.
    pub fn visual<F: Fn(Geometry) -> Effect>(geometry: Geometry, rule: F) -> Effect {
        rule(geometry)
    }

    /// Pointing carries a POSITION, not merely a yes/no
    /// (SwiftUI: `HoverPhase::active(CGPoint)` | `ended`). That way
    /// the compositor glass can put its light where the human
    /// points, instead of building it in fixed (R174).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Hover {
        Active { x: f32, y: f32 },
        Ended,
    }

    impl Hover {
        pub fn is_active(self) -> bool {
            matches!(self, Hover::Active { .. })
        }

        /// Where in the element is being pointed at, as a fraction −1…+1 from
        /// the centre? That is the form a light direction
        /// needs: a direction vector, not a pixel value.
        pub fn direction(self, g: Geometry) -> (f32, f32) {
            match self {
                Hover::Ended => (0.0, 0.0),
                Hover::Active { x, y } => {
                    if g.width <= 0.0 || g.height <= 0.0 {
                        return (0.0, 0.0);
                    }
                    (
                        ((x / g.width) * 2.0 - 1.0).clamp(-1.0, 1.0),
                        ((y / g.height) * 2.0 - 1.0).clamp(-1.0, 1.0),
                    )
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn g() -> Geometry {
            Geometry { width: 200.0, height: 100.0, center_x: 100.0, center_y: 500.0, space_height: 1000.0 }
        }

        #[test]
        fn a_wirkung_never_touches_the_layout() {
            // The contract of the whole family. No kind of the
            // closed list can change width or height —
            // whoever adds one that could breaks this test.
            let e = Effect::none()
                .with(Wirkung::Opacity(0.5))
                .with(Wirkung::Scale(2.0))
                .with(Wirkung::Offset(40.0, -20.0))
                .with(Wirkung::Blur(8.0));
            assert!(!e.touches_layout());
            // Scaling is appearance, not size: the geometry
            // stays what it was.
            let before = g();
            let after = g();
            assert_eq!(before.width, after.width);
            assert_eq!(before.height, after.height);
        }

        #[test]
        fn effects_stack_like_apple_combines_them() {
            let e = Effect::none()
                .with(Wirkung::Opacity(0.5))
                .with(Wirkung::Opacity(0.5))
                .with(Wirkung::Offset(10.0, 5.0))
                .with(Wirkung::Offset(-4.0, 1.0))
                .with(Wirkung::Scale(2.0));
            // Opacity multiplies, offset adds — that is how
            // combining() stacks at Apple.
            assert!((e.opacity() - 0.25).abs() < 0.001);
            assert_eq!(e.offset(), (6.0, 6.0));
            assert!((e.scale() - 2.0).abs() < 0.001);
            // Neutral when the kind is missing.
            assert_eq!(Effect::none().opacity(), 1.0);
            assert_eq!(Effect::none().offset(), (0.0, 0.0));
        }

        #[test]
        fn the_rule_gets_the_geometry_and_nothing_else() {
            // visual() is the whole seam: the same rule, different
            // geometry, different result — without global state.
            let rule = |geo: Geometry| Effect::none().with(Wirkung::Opacity(geo.fraction()));
            let oben = Geometry { center_y: 0.0, ..g() };
            let unten = Geometry { center_y: 1000.0, ..g() };
            assert!(visual(oben, rule).opacity() < visual(unten, rule).opacity());
            // No reference space = no guess, but the centre.
            let leer = Geometry { space_height: 0.0, ..g() };
            assert_eq!(leer.fraction(), 0.5);
        }

        #[test]
        fn hover_carries_a_direction_not_a_boolean() {
            let geo = g();
            // Centre = no direction.
            assert_eq!(Hover::Active { x: 100.0, y: 50.0 }.direction(geo), (0.0, 0.0));
            // Top left = towards the top left.
            let (dx, dy) = Hover::Active { x: 0.0, y: 0.0 }.direction(geo);
            assert!(dx < 0.0 && dy < 0.0);
            // Rechts unten = umgekehrt.
            let (dx, dy) = Hover::Active { x: 200.0, y: 100.0 }.direction(geo);
            assert!(dx > 0.0 && dy > 0.0);
            // Ended = neutral, never an old value.
            assert_eq!(Hover::Ended.direction(geo), (0.0, 0.0));
            assert!(!Hover::Ended.is_active());
        }
    }
}

#[cfg(test)]
mod tween_tests {
    use super::motion::{Tween, STANDARD};

    #[test]
    fn a_tween_arrives_exactly_after_its_duration() {
        let mut t = Tween::fly(0.0, 100.0, 0.5, STANDARD);
        let mut steps = 0;
        while t.tick(0.1) {
            steps += 1;
            assert!(steps < 100);
        }
        assert_eq!(steps, 5);
        assert_eq!(t.value(), 100.0);
        assert!(t.is_settled());
    }

    #[test]
    fn retarget_continues_from_the_current_value() {
        let mut t = Tween::fly(0.0, 100.0, 1.0, STANDARD);
        t.tick(0.5);
        let mid = t.value();
        assert!(mid > 0.0 && mid < 100.0);
        t.retarget(-50.0, 1.0);
        assert!((t.value() - mid).abs() < 1e-4, "kein Sprung beim Umlenken");
        while t.tick(0.25) {}
        assert_eq!(t.value(), -50.0);
    }

    #[test]
    fn staggered_tweens_hold_then_fly() {
        let mut t = Tween::fly(10.0, 20.0, 0.2, STANDARD).staggered(3, 0.1);
        assert!(t.tick(0.2));
        assert_eq!(t.value(), 10.0, "während der Verzögerung ruht der Wert");
        assert!(t.tick(0.15));
        assert!(t.value() > 10.0, "der Rest des Schritts fliegt schon");
        assert!(!Tween::at(4.0).tick(1.0));
    }
}

#[cfg(test)]
mod offset_tests {
    use super::scene::{lay_out, placed_count, Node};
    use super::Rgba;

    #[test]
    fn the_offset_moves_its_child_and_keeps_the_measure() {
        let q = Node::Quad { w: 10.0, h: 4.0, color: Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, radius: 0.0 };
        let n = Node::Offset { dx: -3.0, dy: 7.5, child: Box::new(q) };
        let mut out = Vec::new();
        let (w, h) = lay_out(&n, 100.0, 100.0, &mut out);
        assert_eq!((w, h), (10.0, 4.0));
        assert_eq!((out[0].x, out[0].y), (97.0, 107.5));
        assert_eq!(placed_count(&n), 1);
    }
}


#[cfg(test)]
mod seed_tests {
    use super::*;

    #[test]
    fn a_seed_grows_a_readable_palette_in_both_modes() {
        let seed = Rgba::from_hex("#3584e4").unwrap();
        let dark = Palette::from_seed(seed, false);
        let light = Palette::from_seed(seed, true);
        let l = |c: Rgba| c.to_hsl().2;
        // Dunkel: dunkler Grund, helle Schrift; hell umgekehrt.
        assert!(l(dark.surface) < 0.15 && l(dark.on_surface) > 0.8);
        assert!(l(light.surface) > 0.95 && l(light.on_surface) < 0.2);
        // The seed's hue is preserved in the primary (±15°).
        let (h0, _, _) = seed.to_hsl();
        let (h1, _, _) = dark.primary.to_hsl();
        assert!((h0 - h1).abs() < 15.0, "Farbton {h0} → {h1}");
        // HSL there and back is lossless enough.
        let (h, s_, l_) = seed.to_hsl();
        let back = Rgba::from_hsl(h, s_, l_);
        assert!((back.r - seed.r).abs() < 0.01 && (back.g - seed.g).abs() < 0.01 && (back.b - seed.b).abs() < 0.01);
    }
}

/// THE CAMERA (R215 family, 02.11.2026): the duo's IMX351 without CamX.
/// `duo-camgrab` (werkzeuge/kamera in the Duo dossier) drives the Spectra
/// kernel driver — sensor, CSIPHY, IFE in raw dump — streams at 30 fps
/// with its own auto exposure and serves the newest frame on a unix
/// socket. This family is the client side: the wire header, the RAW10
/// BGGR unpacking, statistics and a cheap development (black level,
/// grey-world white balance, gamma) to RGBA — pure, tested, no GPU.
pub mod camera {
    use std::io::{self, Read, Write};
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    /// The stream server's socket in the runtime dir.
    pub fn socket_path() -> PathBuf {
        super::runtime::path("matrix-camera.sock")
    }

    /// The capture tool (installed beside the house apps).
    pub const TOOL: &str = "duo-camgrab";
    /// RAW10 black level of the IMX351 (measured: dark rows sit at 64).
    pub const BLACK_LEVEL: u16 = 64;
    /// Wire format 1: packed MIPI RAW10, BGGR (the sensor mirrors + flips).
    pub const FORMAT_RAW10_BGGR: u32 = 1;
    /// Size of the socket reply header.
    pub const HEADER_LEN: usize = 56;

    /// The 56-byte header in front of every frame ('F') — little endian,
    /// magic "DCAM", version 1.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Header {
        pub width: u32,
        pub height: u32,
        pub stride: u32,
        pub format: u32,
        pub frame: u64,
        pub req: u64,
        pub sof_boot_ns: u64,
        pub len: u32,
        pub buf: u32,
    }

    impl Header {
        pub fn parse(b: &[u8]) -> Option<Header> {
            if b.len() < HEADER_LEN || &b[0..4] != b"DCAM" {
                return None;
            }
            let u32_at = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
            let u64_at = |o: usize| u64::from_le_bytes(b[o..o + 8].try_into().unwrap());
            if u32_at(4) != 1 {
                return None;
            }
            Some(Header {
                width: u32_at(8),
                height: u32_at(12),
                stride: u32_at(16),
                format: u32_at(20),
                frame: u64_at(24),
                req: u64_at(32),
                sof_boot_ns: u64_at(40),
                len: u32_at(48),
                buf: u32_at(52),
            })
        }
    }

    /// One raw frame as it left the sensor.
    #[derive(Debug, Clone)]
    pub struct Frame {
        pub header: Header,
        pub data: Vec<u8>,
    }

    impl Frame {
        /// A frame read from a file `duo-camgrab` wrote (size tells the mode).
        pub fn from_raw_file(path: &Path) -> io::Result<Frame> {
            let data = std::fs::read(path)?;
            let (width, height) = if data.len() >= 5040 * 2784 { (4032, 2784) } else { (1920, 1080) };
            let header = Header { width, height, stride: width * 10 / 8, format: FORMAT_RAW10_BGGR, len: data.len() as u32, ..Header::default() };
            let frame = Frame { header, data };
            // Round 178 (crash audit): a short file used to panic in the develop step.
            if !frame.view().sound() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Rohbild kürzer als sein Kopf"));
            }
            Ok(frame)
        }

        /// The 10-bit value at (x, y).
        pub fn at(&self, x: usize, y: usize) -> u16 {
            raw10(&self.data[y * self.header.stride as usize..], x)
        }
    }

    /// A frame borrowed where it lies (a mapped ring buffer of the stream).
    #[derive(Debug, Clone, Copy)]
    pub struct View<'a> {
        pub header: Header,
        pub data: &'a [u8],
    }

    impl View<'_> {
        /// Round 178 (crash audit): does the buffer hold what the header claims? (stride covers a RAW10 row, and
        /// stride × height bytes are there) — a short or inconsistent frame from the grabber panicked the develop
        /// thread and froze the preview.
        pub fn sound(&self) -> bool {
            let (w, h, stride) = (self.header.width as usize, self.header.height as usize, self.header.stride as usize);
            w > 0 && h > 0 && stride >= (w * 10).div_ceil(8) && self.data.len() >= stride * h
        }

        pub fn at(&self, x: usize, y: usize) -> u16 {
            raw10(&self.data[y * self.header.stride as usize..], x)
        }
    }

    impl Frame {
        pub fn view(&self) -> View<'_> {
            View { header: self.header, data: &self.data }
        }
    }

    /// One pixel of a packed MIPI RAW10 row: four 8-bit high parts, then a
    /// byte with the four 2-bit low parts.
    pub fn raw10(row: &[u8], x: usize) -> u16 {
        let base = x / 4 * 5;
        let k = x % 4;
        ((row[base + k] as u16) << 2) | ((row[base + 4] >> (2 * k)) & 3) as u16
    }

    /// A connection to the stream server.
    pub struct Client {
        stream: UnixStream,
    }

    impl Client {
        pub fn connect(path: &Path) -> io::Result<Client> {
            let stream = UnixStream::connect(path)?;
            stream.set_read_timeout(Some(Duration::from_secs(1)))?;
            stream.set_write_timeout(Some(Duration::from_secs(1)))?;
            Ok(Client { stream })
        }

        /// The newest finished frame, or None while none is published yet.
        pub fn latest(&mut self) -> io::Result<Option<Frame>> {
            self.stream.write_all(b"F")?;
            let mut head = [0u8; HEADER_LEN];
            self.stream.read_exact(&mut head)?;
            let header = Header::parse(&head).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "kein DCAM-Kopf"))?;
            if header.len == 0 {
                return Ok(None);
            }
            let mut data = vec![0u8; header.len as usize];
            self.stream.read_exact(&mut data)?;
            Ok(Some(Frame { header, data }))
        }
    }

    /// The stream's ring, mapped (command 'D'): the shared header (memfd,
    /// seqlock) and the K dma-bufs — no copy per frame. A slot is written
    /// again only K-2 frames after it was published, so a reader that
    /// re-checks the frame counter afterwards knows whether it raced.
    pub struct Shared {
        _stream: UnixStream,
        head: *const u8,
        bufs: Vec<(*const u8, usize)>,
    }

    // The mappings are read-only and owned by this value.
    unsafe impl Send for Shared {}

    impl Shared {
        pub fn connect(path: &Path) -> io::Result<Shared> {
            use std::os::fd::AsRawFd;
            let mut stream = UnixStream::connect(path)?;
            stream.set_read_timeout(Some(Duration::from_secs(1)))?;
            stream.write_all(b"D")?;
            let mut head = [0u8; HEADER_LEN];
            let mut fds = [-1i32; 9];
            // SAFETY: recvmsg into our own buffers; control space for 9 fds.
            let got = unsafe {
                let mut iov = libc::iovec { iov_base: head.as_mut_ptr().cast(), iov_len: HEADER_LEN };
                let space = libc::CMSG_SPACE((fds.len() * 4) as u32) as usize;
                let mut cbuf = vec![0u8; space];
                let mut msg: libc::msghdr = std::mem::zeroed();
                msg.msg_iov = &mut iov;
                msg.msg_iovlen = 1;
                msg.msg_control = cbuf.as_mut_ptr().cast();
                msg.msg_controllen = space as _;
                let n = libc::recvmsg(stream.as_raw_fd(), &mut msg, libc::MSG_CMSG_CLOEXEC);
                if n != HEADER_LEN as isize {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "keine D-Antwort"));
                }
                let cm = libc::CMSG_FIRSTHDR(&msg);
                if cm.is_null() || (*cm).cmsg_type != libc::SCM_RIGHTS {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "keine Puffer mitgeschickt"));
                }
                let count = ((*cm).cmsg_len as usize - libc::CMSG_LEN(0) as usize) / 4;
                let data = libc::CMSG_DATA(cm) as *const i32;
                for (i, fd) in fds.iter_mut().enumerate().take(count.min(9)) {
                    *fd = *data.add(i);
                }
                count.min(9)
            };
            let close_all = |fds: &[i32]| fds.iter().filter(|f| **f >= 0).for_each(|f| unsafe { libc::close(*f); });
            if got < 2 || Header::parse(&head).is_none() {
                close_all(&fds);
                return Err(io::Error::new(io::ErrorKind::InvalidData, "D-Antwort unvollständig"));
            }
            let map = |fd: i32, len: usize| -> *const u8 {
                // SAFETY: a read-only shared mapping of a received fd.
                let p = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ, libc::MAP_SHARED, fd, 0) };
                if p == libc::MAP_FAILED { std::ptr::null() } else { p as *const u8 }
            };
            let head_map = map(fds[0], 4096);
            if head_map.is_null() {
                close_all(&fds[..got]);
                return Err(io::Error::other("Kopf nicht abbildbar"));
            }
            // SAFETY: the mapped shared header is 4K, fields at fixed offsets.
            let buf_size = unsafe { std::ptr::read_volatile(head_map.add(24) as *const u32) } as usize;
            let mut bufs = Vec::new();
            for &fd in &fds[1..got] {
                let p = map(fd, buf_size);
                if p.is_null() {
                    close_all(&fds[..got]);
                    return Err(io::Error::other("Puffer nicht abbildbar"));
                }
                bufs.push((p, buf_size));
            }
            close_all(&fds[..got]); // the mappings keep the buffers alive
            Ok(Shared { _stream: stream, head: head_map, bufs })
        }

        fn u32_at(&self, o: usize) -> u32 {
            // SAFETY: inside the 4K header mapping.
            unsafe { std::ptr::read_volatile(self.head.add(o) as *const u32) }
        }

        fn u64_at(&self, o: usize) -> u64 {
            // SAFETY: inside the 4K header mapping, 8-aligned offsets.
            unsafe { std::ptr::read_volatile(self.head.add(o) as *const u64) }
        }

        /// The newest frame where it lies, with its frame counter; None
        /// while the writer is mid-update or nothing is published.
        pub fn latest(&self) -> Option<View<'_>> {
            let seq = self.u64_at(32);
            if seq % 2 == 1 {
                return None;
            }
            std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
            let (latest, frame) = (self.u32_at(28), self.u64_at(40));
            std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
            if self.u64_at(32) != seq || latest as usize >= self.bufs.len() {
                return None;
            }
            let (w, h, stride) = (self.u32_at(8), self.u32_at(12), self.u32_at(16));
            let (p, len) = self.bufs[latest as usize];
            let need = (stride as usize) * (h as usize);
            if need > len {
                return None;
            }
            // SAFETY: the mapping holds `len` bytes; it lives as long as self.
            let data = unsafe { std::slice::from_raw_parts(p, need) };
            Some(View { header: Header { width: w, height: h, stride, format: FORMAT_RAW10_BGGR, frame, len: need as u32, buf: latest, ..Header::default() }, data })
        }

        /// Was a frame read at `frame` possibly overwritten meanwhile?
        pub fn raced(&self, frame: u64) -> bool {
            self.u64_at(40).saturating_sub(frame) + 2 >= self.bufs.len() as u64
        }
    }

    impl Drop for Shared {
        fn drop(&mut self) {
            // SAFETY: unmapping our own mappings once.
            unsafe {
                libc::munmap(self.head as *mut _, 4096);
                for &(p, len) in &self.bufs {
                    libc::munmap(p as *mut _, len);
                }
            }
        }
    }

    /// Mean R, G, B above black of a BGGR frame, every `step`-th 2x2 cell;
    /// and the share of cells with a clipped channel.
    pub fn stats(frame: &Frame, step: usize) -> ([f32; 3], f32) {
        stats_view(frame.view(), step)
    }

    pub fn stats_view(frame: View<'_>, step: usize) -> ([f32; 3], f32) {
        let (w, h) = (frame.header.width as usize, frame.header.height as usize);
        let step = step.max(1) * 2;
        let (mut sum, mut n, mut clip) = ([0u64; 3], 0u64, 0u64);
        let mut y = 0;
        while y + 1 < h {
            let mut x = 0;
            while x + 1 < w {
                let (b, g1, g2, r) = (frame.at(x, y), frame.at(x + 1, y), frame.at(x, y + 1), frame.at(x + 1, y + 1));
                sum[0] += r.saturating_sub(BLACK_LEVEL) as u64;
                sum[1] += ((g1 + g2) / 2).saturating_sub(BLACK_LEVEL) as u64;
                sum[2] += b.saturating_sub(BLACK_LEVEL) as u64;
                clip += (r.max(g1).max(g2).max(b) >= 1000) as u64;
                n += 1;
                x += step;
            }
            y += step;
        }
        let n = n.max(1) as f32;
        ([sum[0] as f32 / n, sum[1] as f32 / n, sum[2] as f32 / n], clip as f32 / n)
    }

    /// Grey-world white balance: gains that bring R and B to G's mean.
    pub fn grey_world(mean: [f32; 3]) -> [f32; 3] {
        let g = mean[1].max(1.0);
        [(g / mean[0].max(1.0)).clamp(0.5, 4.0), 1.0, (g / mean[2].max(1.0)).clamp(0.5, 4.0)]
    }

    /// How a raw frame becomes a picture.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Develop {
        /// White-balance gains R, G, B.
        pub gains: [f32; 3],
        /// Overall brightness: the level (above black) that becomes white.
        pub white: f32,
    }

    impl Develop {
        /// Balanced from the frame itself, its mean green to mid grey.
        pub fn auto(frame: &Frame) -> Develop {
            Develop::auto_view(frame.view())
        }

        pub fn auto_view(frame: View<'_>) -> Develop {
            let (mean, _) = stats_view(frame, 8);
            Develop { gains: grey_world(mean), white: (mean[1] * 4.5).clamp(64.0, 1023.0 - BLACK_LEVEL as f32) }
        }
    }

    /// Half resolution, one RGBA pixel per 2x2 BGGR cell (no demosaic
    /// artefacts): one 1024-entry table per channel (black, balance,
    /// brightness and gamma in one lookup), rows split over four threads —
    /// 960x540 at 30 fps on the duo.
    pub fn develop_half(frame: &Frame, dev: &Develop) -> (u32, u32, Vec<u8>) {
        develop_half_view(frame.view(), dev)
    }

    pub fn develop_half_view(frame: View<'_>, dev: &Develop) -> (u32, u32, Vec<u8>) {
        if !frame.sound() {
            return (0, 0, Vec::new());
        }
        let (w, h) = (frame.header.width as usize / 2, frame.header.height as usize / 2);
        let scale = 1023.0 / dev.white.max(1.0);
        let lut = |gain: f32| -> [u8; 1024] {
            let mut t = [0u8; 1024];
            for (i, v) in t.iter_mut().enumerate() {
                let lin = ((i as f32 - BLACK_LEVEL as f32).max(0.0) * gain * scale / 1023.0).clamp(0.0, 1.0);
                *v = (lin.powf(1.0 / 2.2) * 255.0 + 0.5) as u8;
            }
            t
        };
        let (lr, lg, lb) = (lut(dev.gains[0]), lut(dev.gains[1]), lut(dev.gains[2]));
        let mut out = vec![0u8; w * h * 4];
        let stride = frame.header.stride as usize;
        let data = frame.data;
        let rows_per = h.div_ceil(4).max(1);
        std::thread::scope(|scope| {
            for (part, chunk) in out.chunks_mut(rows_per * w * 4).enumerate() {
                let (lr, lg, lb) = (&lr, &lg, &lb);
                scope.spawn(move || {
                    for (i, line) in chunk.chunks_exact_mut(w * 4).enumerate() {
                        let y = part * rows_per + i;
                        let top = &data[2 * y * stride..];
                        let bottom = &data[(2 * y + 1) * stride..];
                        // Four pixels share five bytes: two BGGR cells per group.
                        for (gx, px) in line.chunks_exact_mut(8).enumerate() {
                            let o = gx * 5;
                            let (t, u) = (&top[o..o + 5], &bottom[o..o + 5]);
                            let v = |row: &[u8], k: usize| ((row[k] as u16) << 2) | ((row[4] >> (2 * k)) & 3) as u16;
                            for c in 0..2 {
                                let (b, g1, g2, r) = (v(t, 2 * c), v(t, 2 * c + 1), v(u, 2 * c), v(u, 2 * c + 1));
                                let q = &mut px[c * 4..c * 4 + 4];
                                q[0] = lr[r as usize];
                                q[1] = lg[((g1 + g2) / 2) as usize];
                                q[2] = lb[b as usize];
                                q[3] = 255;
                            }
                        }
                        // An odd cell count leaves one cell outside the groups.
                        if w % 2 == 1 {
                            let x = w - 1;
                            let (b, g1, g2, r) = (raw10(top, 2 * x), raw10(top, 2 * x + 1), raw10(bottom, 2 * x), raw10(bottom, 2 * x + 1));
                            let q = &mut line[x * 4..x * 4 + 4];
                            q.copy_from_slice(&[lr[r as usize], lg[((g1 + g2) / 2) as usize], lb[b as usize], 255]);
                        }
                    }
                });
            }
        });
        (w as u32, h as u32, out)
    }

    /// Full resolution, bilinear demosaic of BGGR (R215, for stills):
    /// every pixel keeps its own colour and takes the missing two from the
    /// neighbours (edges mirrored), then the same per-channel tables as
    /// `develop_half`. Rows over four threads.
    pub fn develop_full_view(frame: View<'_>, dev: &Develop) -> (u32, u32, Vec<u8>) {
        if !frame.sound() {
            return (0, 0, Vec::new());
        }
        let (w, h) = (frame.header.width as usize, frame.header.height as usize);
        let scale = 1023.0 / dev.white.max(1.0);
        let lut = |gain: f32| -> Vec<u8> {
            (0..1024)
                .map(|i| {
                    let lin = ((i as f32 - BLACK_LEVEL as f32).max(0.0) * gain * scale / 1023.0).clamp(0.0, 1.0);
                    (lin.powf(1.0 / 2.2) * 255.0 + 0.5) as u8
                })
                .collect()
        };
        let luts = [lut(dev.gains[0]), lut(dev.gains[1]), lut(dev.gains[2])];
        let stride = frame.header.stride as usize;
        let data = frame.data;
        let px = |x: isize, y: isize| -> u32 {
            // Mirror at the edges keeps the Bayer phase (step by two).
            let x = if x < 0 { x + 2 } else if x >= w as isize { x - 2 } else { x } as usize;
            let y = if y < 0 { y + 2 } else if y >= h as isize { y - 2 } else { y } as usize;
            raw10(&data[y * stride..], x) as u32
        };
        let mut out = vec![0u8; w * h * 4];
        let rows_per = h.div_ceil(4).max(1);
        std::thread::scope(|scope| {
            for (part, chunk) in out.chunks_mut(rows_per * w * 4).enumerate() {
                let (luts, px) = (&luts, &px);
                scope.spawn(move || {
                    for (i, line) in chunk.chunks_exact_mut(w * 4).enumerate() {
                        let y = (part * rows_per + i) as isize;
                        for x in 0..w as isize {
                            let c = px(x, y);
                            let cross = (px(x - 1, y) + px(x + 1, y) + px(x, y - 1) + px(x, y + 1)) / 4;
                            let diag = (px(x - 1, y - 1) + px(x + 1, y - 1) + px(x - 1, y + 1) + px(x + 1, y + 1)) / 4;
                            let horiz = (px(x - 1, y) + px(x + 1, y)) / 2;
                            let vert = (px(x, y - 1) + px(x, y + 1)) / 2;
                            // BGGR: even row = B G B G, odd row = G R G R.
                            let (r, g, b) = match (y & 1, x & 1) {
                                (0, 0) => (diag, cross, c),
                                (0, _) => (vert, c, horiz),
                                (_, 0) => (horiz, c, vert),
                                _ => (c, cross, diag),
                            };
                            let q = &mut line[x as usize * 4..x as usize * 4 + 4];
                            q[0] = luts[0][r.min(1023) as usize];
                            q[1] = luts[1][g.min(1023) as usize];
                            q[2] = luts[2][b.min(1023) as usize];
                            q[3] = 255;
                        }
                    }
                });
            }
        });
        (w as u32, h as u32, out)
    }

    /// Turn an RGBA picture by `quarters` x 90° clockwise (R215: the
    /// sensor's picture follows how the duo is held).
    pub fn rotate_rgba(w: u32, h: u32, rgba: &[u8], quarters: u32) -> (u32, u32, Vec<u8>) {
        let (w, h) = (w as usize, h as usize);
        let q = quarters % 4;
        if q == 0 {
            return (w as u32, h as u32, rgba.to_vec());
        }
        let (nw, nh) = if q % 2 == 1 { (h, w) } else { (w, h) };
        let mut out = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let (nx, ny) = match q {
                    1 => (h - 1 - y, x),
                    2 => (w - 1 - x, h - 1 - y),
                    _ => (y, w - 1 - x),
                };
                let (src, dst) = ((y * w + x) * 4, (ny * nw + nx) * 4);
                out[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
            }
        }
        (nw as u32, nh as u32, out)
    }

    /// The stream's exposure in lines (hd, 30 fps frame of 8368 lines) and
    /// gain code, carried over to a full-resolution still (mode 0, 30 fps
    /// frame of 4640 lines): the same time in its longer lines; what does
    /// not fit below the frame goes into gain.
    pub fn still_exposure(hd_lines: u32, gain_code: u32) -> (u32, u32) {
        const HD_FRAME: f64 = 8368.0;
        const FULL_FRAME: f64 = 4640.0;
        let lines = hd_lines as f64 * FULL_FRAME / HD_FRAME;
        let max = FULL_FRAME - 21.0;
        let gain = 1024.0 / (1024.0 - gain_code.min(960) as f64);
        let (lines, gain) = if lines > max { (max, gain * lines / max) } else { (lines, gain) };
        let code = (1024.0 - 1024.0 / gain).clamp(0.0, 960.0);
        (lines.round().max(1.0) as u32, code.round() as u32)
    }

    /// What the running stream reports per frame (parsed from its log
    /// line: «… e 8347 g 673»).
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct Exposure {
        pub lines: u32,
        pub gain_code: u32,
    }

    pub fn parse_exposure(line: &str) -> Option<Exposure> {
        let mut it = line.split_whitespace();
        let (mut e, mut g) = (None, None);
        while let Some(t) = it.next() {
            match t {
                "e" => e = it.next().and_then(|v| v.parse::<i64>().ok()),
                "g" => g = it.next().and_then(|v| v.parse::<i64>().ok()),
                _ => {}
            }
        }
        match (e, g) {
            (Some(e), Some(g)) if e > 0 && g >= 0 => Some(Exposure { lines: e as u32, gain_code: g as u32 }),
            _ => None,
        }
    }

    /// The running stream server: `duo-camgrab -N 0 -A -u <socket>`.
    /// Dropping it stops the stream with SIGINT (clean teardown in the
    /// tool) and waits for it.
    pub struct Stream {
        child: Child,
        exposure: Arc<Mutex<Exposure>>,
    }

    impl Stream {
        pub fn start() -> io::Result<Stream> {
            let sock = socket_path();
            let mut child = Command::new(TOOL)
                .args(["-N", "0", "-T", "86400", "-t", "0", "-A", "-u"])
                .arg(&sock)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?;
            let exposure = Arc::new(Mutex::new(Exposure::default()));
            if let Some(out) = child.stdout.take() {
                let shared = Arc::clone(&exposure);
                std::thread::spawn(move || {
                    use std::io::BufRead;
                    for line in io::BufReader::new(out).lines().map_while(Result::ok) {
                        if let Some(e) = parse_exposure(&line) {
                            if let Ok(mut s) = shared.lock() {
                                *s = e;
                            }
                        }
                    }
                });
            }
            Ok(Stream { child, exposure })
        }

        /// The exposure the auto exposure settled on (lines 0 = not yet).
        pub fn exposure(&self) -> Exposure {
            self.exposure.lock().map(|e| *e).unwrap_or_default()
        }

        /// Still running?
        pub fn alive(&mut self) -> bool {
            super::process::watch(&mut self.child).alive()
        }

        /// Stop cleanly and wait (the tool tears the camera down).
        pub fn stop(&mut self) {
            let pid = self.child.id() as i32;
            // SIGINT: the tool's own handler tears the camera down.
            let _ = Command::new("kill").args(["-INT", &pid.to_string()]).status();
            for _ in 0..50 {
                if !self.alive() {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    impl Drop for Stream {
        fn drop(&mut self) {
            if self.alive() {
                self.stop();
            }
        }
    }

    /// One full-resolution still (11 MP) with the given exposure; the
    /// stream must be stopped (the sensor serves one client).
    pub fn take_still(out: &Path, exposure: Exposure) -> io::Result<Frame> {
        let (lines, code) = still_exposure(exposure.lines.max(1), exposure.gain_code);
        let status = Command::new(TOOL)
            .args(["-m", "full", "-t", "0", "-e", &lines.to_string(), "-g", &code.to_string()])
            .arg(out)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            return Err(io::Error::other(format!("{TOOL} endete mit {status}")));
        }
        let frame = Frame::from_raw_file(out);
        let _ = std::fs::remove_file(format!("{}.pgm", out.display()));
        frame
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn pack(px: &[u16]) -> Vec<u8> {
            let mut out = Vec::new();
            for q in px.chunks(4) {
                let mut lo = 0u8;
                for (k, v) in q.iter().enumerate() {
                    out.push((v >> 2) as u8);
                    lo |= ((v & 3) as u8) << (2 * k);
                }
                out.push(lo);
            }
            out
        }

        #[test]
        fn raw10_unpacks_mipi() {
            let row = pack(&[1023, 64, 513, 2, 7, 8, 9, 10]);
            assert_eq!((0..8).map(|x| raw10(&row, x)).collect::<Vec<_>>(), vec![1023, 64, 513, 2, 7, 8, 9, 10]);
        }

        #[test]
        fn header_round_trip() {
            let mut b = vec![0u8; HEADER_LEN];
            b[0..4].copy_from_slice(b"DCAM");
            b[4..8].copy_from_slice(&1u32.to_le_bytes());
            b[8..12].copy_from_slice(&1920u32.to_le_bytes());
            b[12..16].copy_from_slice(&1080u32.to_le_bytes());
            b[16..20].copy_from_slice(&2400u32.to_le_bytes());
            b[24..32].copy_from_slice(&42u64.to_le_bytes());
            b[48..52].copy_from_slice(&2_592_000u32.to_le_bytes());
            let h = Header::parse(&b).unwrap();
            assert_eq!((h.width, h.height, h.stride, h.frame, h.len), (1920, 1080, 2400, 42, 2_592_000));
            b[0] = b'X';
            assert!(Header::parse(&b).is_none());
        }

        #[test]
        fn develop_balances_a_grey_cell() {
            // One BGGR cell: blue 164, green 264, red 214 → grey after balance.
            let row0 = pack(&[164, 264, 164, 264]);
            let row1 = pack(&[264, 214, 264, 214]);
            let header = Header { width: 4, height: 2, stride: 5, format: 1, ..Header::default() };
            let frame = Frame { header, data: [row0, row1].concat() };
            let dev = Develop::auto(&frame);
            let (w, h, rgba) = develop_half(&frame, &dev);
            assert_eq!((w, h), (2, 1));
            assert!((rgba[0] as i32 - rgba[1] as i32).abs() <= 1 && (rgba[2] as i32 - rgba[1] as i32).abs() <= 1, "{rgba:?}");
        }

        #[test]
        fn full_demosaic_keeps_a_grey_field_grey() {
            // A flat grey 4x4 BGGR field: every pixel comes out neutral.
            let row_b = pack(&[164, 264, 164, 264]);
            let row_r = pack(&[264, 214, 264, 214]);
            let header = Header { width: 4, height: 4, stride: 5, format: 1, ..Header::default() };
            let frame = Frame { header, data: [row_b.clone(), row_r.clone(), row_b, row_r].concat() };
            let dev = Develop::auto(&frame);
            let (w, h, rgba) = develop_full_view(frame.view(), &dev);
            assert_eq!((w, h), (4, 4));
            for px in rgba.chunks(4) {
                assert!((px[0] as i32 - px[1] as i32).abs() <= 2 && (px[2] as i32 - px[1] as i32).abs() <= 2, "{px:?}");
            }
        }

        #[test]
        fn rotation_turns_and_returns() {
            // 2x1: red, blue.
            let rgba = [255, 0, 0, 255, 0, 0, 255, 255];
            let (w, h, q1) = rotate_rgba(2, 1, &rgba, 1);
            assert_eq!((w, h), (1, 2));
            assert_eq!(&q1[0..4], &[255, 0, 0, 255]); // red on top after a right turn
            let (_, _, q2) = rotate_rgba(2, 1, &rgba, 2);
            assert_eq!(&q2[0..4], &[0, 0, 255, 255]);
            let (w4, h4, back) = rotate_rgba(w, h, &q1, 3);
            assert_eq!((w4, h4, back), (2, 1, rgba.to_vec()));
        }

        #[test]
        fn still_keeps_the_light() {
            // 33 ms at 30 fps hd (8347 lines) → nearly a whole full frame.
            let (lines, code) = still_exposure(8347, 673);
            assert!(lines >= 4600 && lines <= 4619, "{lines}");
            assert!((code as i32 - 673).abs() <= 2, "{code}");
            // A short exposure stays in lines, gain untouched.
            assert_eq!(still_exposure(836, 0), (464, 0));
            assert_eq!(parse_exposure("  #9 req 9 G 200.6 sat 0.0%  e 8347 g 673"), Some(Exposure { lines: 8347, gain_code: 673 }));
        }
    }
}

/// Removable drives: list, mount, eject, free space (R217 family).
/// TWO TAPS FOR WHAT CANNOT BE UNDONE (round 177, 06.11.2026): the first
/// tap arms («Wirklich entziehen?»), a second tap on the SAME thing within
/// `WINDOW` does it; anything else, or waiting, disarms. Settings (paired
/// laptops), the installer, the bar's power menu, the greeter and Shopping
/// each kept their own flag — this is the one.
pub mod confirm {
    use std::time::{Duration, Instant};

    /// How long an armed tap waits for its second.
    pub const WINDOW: Duration = Duration::from_secs(5);

    #[derive(Debug, Clone)]
    pub struct Armed<K: PartialEq + Clone> {
        at: Option<(K, Instant)>,
    }

    impl<K: PartialEq + Clone> Default for Armed<K> {
        fn default() -> Self {
            Self { at: None }
        }
    }

    impl<K: PartialEq + Clone> Armed<K> {
        pub fn new() -> Self {
            Self::default()
        }

        /// A tap on `k`: true = the second tap — do it now (disarmed);
        /// false = armed now (show the question).
        pub fn tap(&mut self, k: K) -> bool {
            self.tap_at(k, Instant::now())
        }

        pub fn tap_at(&mut self, k: K, now: Instant) -> bool {
            if self.is_at(&k, now) {
                self.at = None;
                return true;
            }
            self.at = Some((k, now));
            false
        }

        /// Is `k` armed (and not yet expired)?
        pub fn is(&self, k: &K) -> bool {
            self.is_at(k, Instant::now())
        }

        pub fn is_at(&self, k: &K, now: Instant) -> bool {
            matches!(&self.at, Some((a, t)) if a == k && now.duration_since(*t) < WINDOW)
        }

        pub fn clear(&mut self) {
            self.at = None;
        }

        /// On a beat: true once when an armed tap ran out (redraw the question away).
        pub fn lapsed(&mut self) -> bool {
            self.lapsed_at(Instant::now())
        }

        pub fn lapsed_at(&mut self, now: Instant) -> bool {
            if matches!(&self.at, Some((_, t)) if now.duration_since(*t) >= WINDOW) {
                self.at = None;
                return true;
            }
            false
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_second_tap_on_the_same_thing_does_it() {
            let t0 = Instant::now();
            let mut a = Armed::new();
            assert!(!a.tap_at(3, t0), "first tap arms");
            assert!(a.is_at(&3, t0) && !a.is_at(&4, t0));
            assert!(a.tap_at(3, t0 + Duration::from_secs(2)), "second tap does it");
            assert!(!a.is_at(&3, t0 + Duration::from_secs(2)), "and disarms");
        }

        #[test]
        fn another_thing_or_waiting_disarms() {
            let t0 = Instant::now();
            let mut a = Armed::new();
            a.tap_at("a", t0);
            assert!(!a.tap_at("b", t0), "a different thing only arms itself");
            assert!(!a.is_at(&"a", t0));
            assert!(!a.tap_at("b", t0 + WINDOW + Duration::from_millis(1)), "too late: arms again");
            assert!(!a.lapsed_at(t0 + WINDOW));
            assert!(a.lapsed_at(t0 + WINDOW * 3), "runs out once");
            assert!(!a.lapsed_at(t0 + WINDOW * 4));
        }
    }
}

pub mod volumes;

/// Copy and move with progress, name clashes, safe moves (R217 family).
pub mod fileops;
