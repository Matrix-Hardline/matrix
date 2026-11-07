//! Sound — as a panel of Matrix Einstellungen (fusion R41b, extended R72).
//! The reference "Sound" grammar in one section: sound effects (the
//! sounds, was app #4), output (device choice + volume + speaker test)
//! and input (device choice + level meter) — all via wpctl/PipeWire.
//! Switches are binding for the hooks (klaenge.conf).

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, container, mouse_area, row, Space};
use matrixkit_widgets::iced::{Element, Length, Subscription, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use mkw::color;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Event catalogue — order = display order.
pub(crate) const EVENTS: &[(&str, &str, &str)] = &[
    ("01-login", "Anmelden", "Drei tiefe Pulse steigen ruhig auf"),
    ("02-logout", "Abmelden", "Dieselben Stufen abwärts"),
    ("03-notification", "Benachrichtigung", "Zwei Pulse aufwärts — gute Nachricht"),
    ("04-notification-quiet", "Leiser Hinweis", "Ein einzelner ruhiger Puls"),
    ("05-volume", "Lautstärke", "Kurzer gedämpfter Tupfer"),
    ("06-device-connected", "Gerät verbunden", "Tiefe Quinte aufwärts"),
    ("07-device-disconnected", "Gerät getrennt", "Dieselbe Quinte, rückwärts"),
    ("08-workspace", "Arbeitsfläche", "Dunkler Hauch mit Grundpuls"),
    ("09-screenshot", "Bildschirmfoto", "Gedämpfter Auslöser"),
    ("10-trash", "Papierkorb", "Dunkles Rascheln, tiefer Punkt"),
    ("11-done", "Fertig", "Das positive Spiegelbild des Fehlers"),
    ("12-error", "Fehler", "Zwei gedämpfte tiefe Pulse — die Referenz"),
    ("13-key-detected", "Schlüssel erkannt", "Der Wächter — bewusst auffällig; am Login-Screen immer aktiv"),
    ("14-guardian-call", "Wächter-Ruf", "Der Ruf der Wiederherstellung — am Login-Screen immer aktiv"),
    ("16-key", "Tastatur", "Drei kurze Klicks — Zeichen, Löschen, Umschalter"),
];

/// Sound folder: dev state in the home wins, else the image version.
/// If both are missing, the image version counts as given (the
/// matrix-sounds binary renders itself if needed).
pub(crate) fn sound_folder() -> PathBuf {
    // One rule for every player (kit): dev state, the Duo's own voice, the image.
    matrixkit_theme::feedback::sound_dir()
}

/// The sound settings: missing entry = on (opt-out like permissions).
pub(crate) struct Settings {
    off_list: Vec<String>,
}

impl Settings {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        PathBuf::from(home).join(".config/matrix/sounds.conf")
    }

    /// Pre-R105 file name — read-only fallback, keys still German.
    fn legacy_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        PathBuf::from(home).join(".config/matrix/klaenge.conf")
    }

    /// English key for a legacy German conf key (R105 migration).
    fn modern_key(k: &str) -> &str {
        match k {
            "alle" => "all",
            "benachrichtigungen" => "notifications",
            "01-anmeldung" => "01-login",
            "02-abmeldung" => "02-logout",
            "03-benachrichtigung" => "03-notification",
            "04-benachrichtigung-leise" => "04-notification-quiet",
            "05-lautstaerke" => "05-volume",
            "06-geraet-verbunden" => "06-device-connected",
            "07-geraet-getrennt" => "07-device-disconnected",
            "08-arbeitsflaeche" => "08-workspace",
            "10-papierkorb" => "10-trash",
            "11-fertig" => "11-done",
            "12-fehler" => "12-error",
            "13-schluessel-erkannt" => "13-key-detected",
            "14-waechter-ruf" => "14-guardian-call",
            other => other,
        }
    }

    pub(crate) fn load() -> Self {
        let mut off_list = Vec::new();
        let content = std::fs::read_to_string(Self::path())
            .or_else(|_| std::fs::read_to_string(Self::legacy_path()))
            .unwrap_or_default();
        for line in content.lines() {
            if let Some((k, w)) = line.split_once('=') {
                let w = w.trim();
                if w == "off" || w == "aus" {
                    off_list.push(Self::modern_key(k.trim()).to_string());
                }
            }
        }
        Self { off_list }
    }

    pub(crate) fn on(&self, key: &str) -> bool {
        !self.off_list.iter().any(|a| a == key)
    }

    pub(crate) fn set(&mut self, key: &str, on: bool) {
        self.off_list.retain(|a| a != key);
        if !on {
            self.off_list.push(key.to_string());
        }
        let path = Self::path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let mut content =
            String::from("# Matrix-Systemklänge — verwaltet von der App „Matrix Klänge\u{201c}\n");
        for a in &self.off_list {
            content.push_str(&format!("{a}=off\n"));
        }
        let tmp = path.with_extension("conf.new");
        if std::fs::write(&tmp, content).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

// ------------------------------------------------- devices (R72, wpctl)
// Since R214 the Kit's audio family (`mk::audio`) carries endpoints,
// volumes and the test tone — this panel and the house face share it.

pub(crate) use mk::audio::Endpoint as Device;

pub(crate) fn wpctl(args: &[&str]) -> Option<String> {
    mk::audio::wpctl(args)
}

pub(crate) fn devices_read() -> (Vec<Device>, Vec<Device>) {
    mk::audio::endpoints()
}

pub(crate) fn volume_amt_read(target: &str) -> f32 {
    mk::audio::volume(target)
}

pub(crate) fn test_tone(channel: usize) -> Option<PathBuf> {
    mk::audio::test_tone(channel)
}

pub struct Panel {
    pub palette: mk::Palette,
    settings: Settings,
    folder: PathBuf,
    /// Last played sound — feedback in the footer.
    plays: Option<&'static str>,
    /// Index of the running preview + bounce spring of the ▶ (reference bounce).
    plays_index: Option<usize>,
    seek: mk::motion::Spring,
    // --- R72: output & input ---
    sinks: Vec<Device>,
    sources: Vec<Device>,
    out_vol: f32,
    in_vol: f32,
    /// Do not re-query the device list on every host tick.
    verified: Option<Instant>,
    /// Microphone level (0–100), fed by the pw-record thread.
    level: Arc<AtomicU32>,
    level_active: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    /// Own 60 fps tick for the ▶ bounce (independent of the root spring).
    SeekTick,
    Switch(usize, bool),
    MasterSwitch(bool),
    Probe(usize),
    // --- R72: output & input ---
    OutputChoice(u32),
    InputChoice(u32),
    OutputVol(f32),
    InputVol(f32),
    /// Speaker test: 0 = left, 1 = right.
    Test(usize),
    MicTest(bool),
    /// Redraw only — the level lives in the atomic.
    LevelTick,
}

impl Panel {
    pub fn new() -> Self {
        let (sinks, sources) = devices_read();
        Self {
            palette: mk::Palette::load().unwrap_or_default(),
            settings: Settings::load(),
                folder: sound_folder(),
                plays: None,
                plays_index: None,
            seek: mk::motion::Spring::new(1.0),
            sinks,
            sources,
            out_vol: volume_amt_read("@DEFAULT_AUDIO_SINK@"),
            in_vol: volume_amt_read("@DEFAULT_AUDIO_SOURCE@"),
            verified: Some(Instant::now()),
            level: Arc::new(AtomicU32::new(0)),
            level_active: Arc::new(AtomicBool::new(false)),
        }
    }

    /// From the host per tick: palette follows, footer text fades; the
    /// device list is re-queried at most every 3 s (hotplug).
    pub fn tick(&mut self, p: mk::Palette) {
        self.palette = p;
        self.plays = None;
        let fresh = self
            .verified
            .map(|t| t.elapsed() < Duration::from_secs(3))
            .unwrap_or(false);
        if !fresh {
            self.verified = Some(Instant::now());
            let (sinks, sources) = devices_read();
            self.sinks = sinks;
            self.sources = sources;
            self.out_vol = volume_amt_read("@DEFAULT_AUDIO_SINK@");
            self.in_vol = volume_amt_read("@DEFAULT_AUDIO_SOURCE@");
        }
    }

    pub fn footer_text(&self) -> String {
        match self.plays {
            Some(name) => format!("\u{25b6} {name} \u{2026}"),
            None => String::from("\u{25b6} spielt eine Hörprobe"),
        }
    }

    fn play(&mut self, i: usize) {
        let (file, name, _) = EVENTS[i];
        let path = self.folder.join(format!("{file}.wav"));
        if let Ok(mut child) = std::process::Command::new("pw-play").arg(&path).spawn() {
            // Reap the child in the background — else a zombie remains
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            self.plays = Some(name);
            self.plays_index = Some(i);
            // The ▶ springs: arrives large, settles to normal size
            self.seek = mk::motion::Spring::new(1.45);
            self.seek.retarget(1.0);
        }
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::SeekTick => {
                self.seek.tick(1.0 / 60.0);
                Task::none()
            }
            Msg::Switch(i, on) => {
                self.settings.set(EVENTS[i].0, on);
                Task::none()
            }
            Msg::MasterSwitch(on) => {
                self.settings.set("all", on);
                Task::none()
            }
            Msg::Probe(i) => {
                self.play(i);
                Task::none()
            }
            Msg::OutputChoice(id) => {
                wpctl(&["set-default", &id.to_string()]);
                for s in &mut self.sinks {
                    s.standard = s.id == id;
                }
                self.out_vol = volume_amt_read("@DEFAULT_AUDIO_SINK@");
                Task::none()
            }
            Msg::InputChoice(id) => {
                wpctl(&["set-default", &id.to_string()]);
                for q in &mut self.sources {
                    q.standard = q.id == id;
                }
                self.in_vol = volume_amt_read("@DEFAULT_AUDIO_SOURCE@");
                Task::none()
            }
            Msg::OutputVol(v) => {
                self.out_vol = v;
                wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{v:.2}")]);
                Task::none()
            }
            Msg::InputVol(v) => {
                self.in_vol = v;
                wpctl(&["set-volume", "@DEFAULT_AUDIO_SOURCE@", &format!("{v:.2}")]);
                Task::none()
            }
            Msg::Test(channel) => {
                if let Some(path) = test_tone(channel) {
                    if let Ok(mut child) = std::process::Command::new("pw-play").arg(&path).spawn() {
                        std::thread::spawn(move || {
                            let _ = child.wait();
                        });
                    }
                }
                Task::none()
            }
            Msg::MicTest(on) => {
                self.level_active.store(on, Ordering::SeqCst);
                if on {
                    // The level thread: pw-record delivers raw s16 samples from the
                    // default source; the peak per 100 ms becomes 0–100.
                    let level = Arc::clone(&self.level);
                    let active = Arc::clone(&self.level_active);
                    std::thread::spawn(move || {
                        use std::io::Read;
                        let Ok(mut child) = std::process::Command::new("pw-record")
                            .args(["--rate", "8000", "--channels", "1", "--format", "s16", "-"])
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::null())
                            .spawn()
                        else {
                            active.store(false, Ordering::SeqCst);
                            return;
                        };
                        let mut pipe = child.stdout.take().unwrap();
                        let mut puffer = [0u8; 1600];
                        while active.load(Ordering::SeqCst) {
                            if pipe.read_exact(&mut puffer).is_err() {
                                break;
                            }
                            let mut peak = 0i32;
                            for paar in puffer.chunks_exact(2) {
                                let value = i16::from_le_bytes([paar[0], paar[1]]) as i32;
                                peak = peak.max(value.abs());
                            }
                            level.store((peak * 100 / i16::MAX as i32) as u32, Ordering::SeqCst);
                        }
                        let _ = child.kill();
                        let _ = child.wait();
                        level.store(0, Ordering::SeqCst);
                    });
                }
                Task::none()
            }
            Msg::LevelTick => Task::none(),
        }
    }

    /// The ▶ bounce spring and — during the microphone test — the
    /// level pulse; everything else is subscribed by the host.
    pub fn subscription(&self) -> Subscription<Msg> {
        let mut subs = Vec::new();
        if !self.seek.is_settled() {
            subs.push(mkw::tick("sounds-seek", Duration::from_millis(16)).map(|_| Msg::SeekTick));
        }
        if self.level_active.load(Ordering::SeqCst) {
            subs.push(mkw::tick("sound-level", Duration::from_millis(50)).map(|_| Msg::LevelTick));
        }
        Subscription::batch(subs)
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let master_on = self.settings.on("all");

        // Master section — the reference form grammar: card + row with a
        // description under the title, switch on the right.
        let master = mkw::section(
            "",
            vec![mkw::row_toggle(
                "Systemklänge",
                Some(if master_on {
                    "Aktiv — Ereignisse klingen nach den Schaltern unten"
                } else {
                    "Stummgeschaltet — kein Ereignis klingt"
                }),
                Some(mkw::symbol(
                    if master_on { mkw::symbol::VOLUME_UP } else { mkw::symbol::VOLUME_OFF },
                    mk::font_size::XLARGE,
                    if master_on { p.primary } else { p.on_surface_variant },
                )),
                master_on,
                p,
                Some(Msg::MasterSwitch(!master_on)),
            )],
            p,
        );

        // Event section: ▶ preview leading, description under the title
        let mut lines: Vec<Element<'_, Msg>> = Vec::new();
        for (i, (key, name, description)) in EVENTS.iter().enumerate() {
            let on = self.settings.on(key);
            let seek_size = if self.plays_index == Some(i) {
                mk::font_size::LARGE * self.seek.value.clamp(0.6, 1.6)
            } else {
                mk::font_size::LARGE
            };
            let probe = iced::widget::button(mkw::symbol::<Msg>(
                mkw::symbol::PLAY_ARROW,
                seek_size,
                if master_on && on { p.primary } else { p.on_surface_variant },
            ))
            .padding(6)
            .on_press(Msg::Probe(i))
            .style(move |_, status| {
                let base = p.surface_container_high;
                let bg = match status {
                    iced::widget::button::Status::Hovered => {
                        Some(color(p.on_surface.over(base, mk::state_layer::HOVER)).into())
                    }
                    iced::widget::button::Status::Pressed => {
                        Some(color(p.on_surface.over(base, mk::state_layer::PRESSED)).into())
                    }
                    _ => None,
                };
                // family-exception: event row with ▶ spring — own row body
                iced::widget::button::Style {
                    background: bg,
                    border: iced::Border { radius: mk::radius::LARGE.into(), ..Default::default() },
                    ..Default::default()
                }
            });
            let z = mkw::row_toggle(
                name,
                Some(description),
                Some(probe.into()),
                on,
                p,
                Some(Msg::Switch(i, !on)),
            );
            lines.push(
                container(z)
                    .style(move |_| container::Style {
                        border: iced::Border::default(),
                        ..Default::default()
                    })
                    .into(),
            );
        }
        let events = mkw::section("EREIGNISSE", lines, p);

        // --- R72: OUTPUT — reference "Sound" tabs as sections ---
        let mut out_lines: Vec<Element<'_, Msg>> = Vec::new();
        for g in &self.sinks {
            out_lines.push(device_row(g, p, Msg::OutputChoice(g.id)));
        }
        if self.sinks.is_empty() {
            out_lines.push(mkw::row_value("Keine Ausgabegeräte", None, "\u{2014}", p));
        }
        out_lines.push(mkw::row_item(
            "Lautstärke",
            None,
            Some(mkw::symbol(mkw::symbol::VOLUME_UP, mk::font_size::LARGE, p.on_surface_variant)),
            Some(
                mkw::bar_slider(0.0..=1.0, self.out_vol, 0.01, p, Msg::OutputVol)
                    .width(Length::Fixed(220.0))
                    .into(),
            ),
            p,
        ));
        out_lines.push(mkw::row_item(
            "Lautsprecher testen",
            Some("Spielt einen Ton auf dem gewählten Gerät"),
            None,
            Some(
                row![
                    mkw::button("Links", mkw::buttons::Style::Tinted, mkw::buttons::Role::Normal, mkw::buttons::Size::Small, p, Some(Msg::Test(0))),
                    Space::new().width(mk::spacing::S),
                    mkw::button("Rechts", mkw::buttons::Style::Tinted, mkw::buttons::Role::Normal, mkw::buttons::Size::Small, p, Some(Msg::Test(1))),
                ]
                .into(),
            ),
            p,
        ));
        let output = mkw::section("AUSGABE", out_lines, p);

        // --- R72: INPUT ---
        let mut in_lines: Vec<Element<'_, Msg>> = Vec::new();
        for g in &self.sources {
            in_lines.push(device_row(g, p, Msg::InputChoice(g.id)));
        }
        if self.sources.is_empty() {
            in_lines.push(mkw::row_value("Keine Eingabegeräte", None, "\u{2014}", p));
        }
        in_lines.push(mkw::row_item(
            "Eingangslautstärke",
            None,
            Some(mkw::symbol(mkw::symbol::GRAPHIC_EQ, mk::font_size::LARGE, p.on_surface_variant)),
            Some(
                mkw::bar_slider(0.0..=1.0, self.in_vol, 0.01, p, Msg::InputVol)
                    .width(Length::Fixed(220.0))
                    .into(),
            ),
            p,
        ));
        let mic_on = self.level_active.load(Ordering::SeqCst);
        let level_meter: Element<'_, Msg> = if mic_on {
            let value = self.level.load(Ordering::SeqCst).min(100);
            let width = 220.0;
            container(
                row![
                    container(Space::new().width(Length::Fixed(width * value as f32 / 100.0)).height(Length::Fixed(8.0)))
                        .style(move |_| container::Style {
                            background: Some(color(p.primary).into()),
                            border: iced::Border { radius: mk::radius::SMALL.into(), ..Default::default() },
                            ..Default::default()
                        }),
                ],
            )
            .width(Length::Fixed(width))
            .style(move |_| container::Style {
                background: Some(color(p.on_surface.over(p.surface_container_high, 0.10)).into()),
                border: iced::Border { radius: mk::radius::SMALL.into(), ..Default::default() },
                ..Default::default()
            })
            .into()
        } else {
            Space::new().width(Length::Fixed(220.0)).into()
        };
        in_lines.push(mkw::row_toggle(
            "Mikrofon testen",
            Some(if mic_on { "Sprich — der Pegel folgt deiner Stimme" } else { "Zeigt den Eingangspegel live" }),
            Some(level_meter),
            mic_on,
            p,
            Some(Msg::MicTest(!mic_on)),
        ));
        let input = mkw::section("EINGABE", in_lines, p);

        column![
            master,
            Space::new().height(mk::spacing::L),
            output,
            Space::new().height(mk::spacing::L),
            input,
            Space::new().height(mk::spacing::L),
            events,
        ]
        .spacing(0)
        .into()
    }
}

/// A device row: ✓ leads at the default device, the whole row is
/// clickable (reference output list).
fn device_row<'a>(g: &'a Device, p: mk::Palette, on: Msg) -> Element<'a, Msg> {
    let leading = if g.standard {
        mkw::symbol(mkw::symbol::CHECK, mk::font_size::LARGE, p.primary)
    } else {
        mkw::symbol(' ', mk::font_size::LARGE, p.on_surface_variant)
    };
    mouse_area(mkw::row_item(
        g.name.as_str(),
        None,
        Some(leading),
        None,
        p,
    ))
    .on_press(on)
    .interaction(iced::mouse::Interaction::Pointer)
    .into()
}
