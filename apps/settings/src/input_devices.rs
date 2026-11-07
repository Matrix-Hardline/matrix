//! Section "Bedienung" (input, R92): controllers, keyboard, mouse & trackpad —
//! the input paths of the system in one place (reference grammar).
//!
//! CONTROLLER is real from the factory: matrix-pad (app #32) translates
//! Xbox, PlayStation and other gamepads system-wide into pointer,
//! scrolling and the focus grammar; status, mapping and speed sit here.
//! KEYBOARD shows the values of the input culture. MOUSE/TRACKPAD
//! are reservations like the display values: stored under
//! ~/.config/matrix/, the shell buildout will read them — the footer
//! says so honestly.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, container, row};
use matrixkit_widgets::iced::{Element, Length, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;

pub struct Panel {
    palette: mk::Palette,
    /// Lines from $XDG_RUNTIME_DIR/matrix-pad-geraete (empty = none).
    controller: Vec<String>,
    /// Pointer speed of the pad (1..=10) — matrix-pad re-reads every 2 s.
    pad_speed: f32,
    /// Preformatted keyboard value (lives with the panel — zeile_wert
    /// borrows &str).
    keyboard_value: String,
    /// Reservations for mouse/trackpad.
    mouse_speed: f32,
    natural_scroll: bool,
    tap_clicks: bool,
}

#[derive(Debug, Clone)]
pub enum Msg {
    PadSpeed(f32),
    MouseSpeed(f32),
    NaturalScrollen(bool),
    TapToClick(bool),
}

pub(crate) fn devices_read() -> Vec<String> {
    let base = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
    std::fs::read_to_string(format!("{base}/matrix-pad-geraete"))
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).map(String::from).collect())
        .unwrap_or_default()
}

impl Panel {
    pub fn new() -> Self {
        Self {
            palette: mk::Palette::load().unwrap_or_default(),
            controller: devices_read(),
            keyboard_value: format!(
                "{} ms · {}/s",
                mk::input::KEY_REPEAT_DELAY_MS,
                mk::input::KEY_REPEAT_RATE
            ),
            pad_speed: mk::settings::read("pad-speed")
                .and_then(|s| s.parse().ok())
                .unwrap_or(5.0),
            mouse_speed: mk::settings::read("mouse-speed")
                .and_then(|s| s.parse().ok())
                .unwrap_or(5.0),
            natural_scroll: mk::settings::read("natural-scrolling")
                .is_some_and(|s| s == "an"),
            tap_clicks: mk::settings::read("typing-clicks")
                .map(|s| s == "an")
                .unwrap_or(true),
        }
    }

    pub fn tick(&mut self, p: mk::Palette) {
        self.palette = p;
        self.controller = devices_read();
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::PadSpeed(v) => {
                self.pad_speed = v;
                mk::settings::write("pad-speed", &format!("{v:.0}"));
            }
            Msg::MouseSpeed(v) => {
                self.mouse_speed = v;
                mk::settings::write("mouse-speed", &format!("{v:.0}"));
            }
            Msg::NaturalScrollen(on) => {
                self.natural_scroll = on;
                mk::settings::write("natural-scrolling", if on { "on" } else { "off" });
            }
            Msg::TapToClick(on) => {
                self.tap_clicks = on;
                mk::settings::write("typing-clicks", if on { "on" } else { "off" });
            }
        }
        Task::none()
    }

    pub fn footer_text(&self) -> String {
        if self.controller.is_empty() {
            String::from("Controller einfach verbinden (USB oder Bluetooth) — Matrix Pad übernimmt sofort")
        } else {
            String::from("Maus- und Trackpad-Werte sind Vormerkungen für den Shell-Ausbau — Controller wirken sofort")
        }
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let p = self.palette;

        // ── Controller: status, speed, mapping ───────────────────────
        let mut controller_lines: Vec<Element<'_, Msg>> = Vec::new();
        if self.controller.is_empty() {
            controller_lines.push(mkw::row_item(
                "Kein Controller verbunden",
                Some("Xbox, PlayStation und Co. — per USB oder Bluetooth, ohne Einrichtung."),
                Some(mkw::symbol::<Msg>(mkw::symbol::TOUCH_APP, mk::icon_size::SMALL, p.on_surface_variant)),
                None,
                p,
            ));
        } else {
            for name in &self.controller {
                controller_lines.push(mkw::row_item(
                    name.as_str(),
                    Some("Verbunden — steuert Zeiger, Scrollen und Fokus systemweit."),
                    Some(mkw::symbol::<Msg>(mkw::symbol::CHECK, mk::icon_size::SMALL, p.primary)),
                    None,
                    p,
                ));
            }
        }
        controller_lines.push(slider_line(
            "Zeiger-Tempo",
            "Wie schnell der linke Stick den Mauszeiger bewegt.",
            self.pad_speed,
            Msg::PadSpeed,
            p,
        ));

        // ── The mapping — console grammar meets focus model ───
        let assignment = vec![
            mkw::row_value("Steuerkreuz", None, "Fokus bewegen (Pfeile)", p),
            mkw::row_value("A / Kreuz", None, "Auswählen (Enter)", p),
            mkw::row_value("B / Kreis", None, "Zurück (Esc)", p),
            mkw::row_value("Linker Stick", None, "Mauszeiger", p),
            mkw::row_value("Rechter Stick", None, "Scrollen", p),
            mkw::row_value("L1 · R1", None, "Links- · Rechtsklick", p),
            mkw::row_value("Start / Options", None, "Apps öffnen (Matrix Start)", p),
        ];

        // ── Keyboard: the applied input culture, black on white ──
        let keyboard = vec![
            mkw::row_value(
                "Tastenwiederholung",
                Some("Anlauf, dann Wiederholungen je Sekunde."),
                &self.keyboard_value,
                p,
            ),
            mkw::row_value("Doppelklick-Fenster", None, "500 ms", p),
        ];

        // ── Mouse & trackpad: reservations (honest, like display) ────
        let mouse = vec![
            slider_line(
                "Zeigertempo",
                "Vormerkung — der Shell-Ausbau liest den Wert künftig.",
                self.mouse_speed,
                Msg::MouseSpeed,
                p,
            ),
            mkw::row_toggle(
                "Natürliches Scrollen",
                Some("Inhalt folgt den Fingern — wie auf dem Leitbild."),
                None,
                self.natural_scroll,
                p,
                Some(Msg::NaturalScrollen(!self.natural_scroll)),
            ),
            mkw::row_toggle(
                "Tippen zum Klicken",
                Some("Trackpad: kurzes Tippen ist ein Klick."),
                None,
                self.tap_clicks,
                p,
                Some(Msg::TapToClick(!self.tap_clicks)),
            ),
        ];

        column![
            mkw::section("CONTROLLER", controller_lines, p),
            mkw::section("BELEGUNG", assignment, p),
            mkw::section("TASTATUR", keyboard, p),
            mkw::section("MAUS & TRACKPAD", mouse, p),
        ]
        .spacing(mk::spacing::L)
        .into()
    }
}

/// A slider row in the form grammar: title + description
/// on the left, slider + level on the right (display section style).
fn slider_line<'a>(
    title: &'a str,
    description: &'a str,
    value: f32,
    emit: impl Fn(f32) -> Msg + 'a,
    p: mk::Palette,
) -> Element<'a, Msg> {
    mkw::row_item(
        title,
        Some(description),
        None,
        Some(
            row![
                container(mkw::slider(1.0..=10.0, value, 1.0, emit, p))
                    .width(Length::Fixed(150.0)),
                mkw::txt(format!("{value:.0}"), mk::typo::CAPTION, p.on_surface_variant),
            ]
            .spacing(mk::spacing::S)
            .align_y(iced::Alignment::Center)
            .into(),
        ),
        p,
    )
}
