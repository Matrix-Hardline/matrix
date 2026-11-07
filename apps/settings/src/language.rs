//! Section "Sprache & Region" (R100) — the reference extract of 2026-07-20:
//! language is a CHAIN (not one choice), region is SEPARATE from the
//! language (de_CH: English UI with Swiss formats is legitimate),
//! and the keyboard is a third, own axis. All measured live on the
//! reference system: de_CH writes 1'234'567.89 with POINT decimal,
//! de_DE 1.234.567,89 — the formats follow the region.
//!
//! Honesty grammar as everywhere: the language switch applies to newly
//! started apps (exactly like the reference, which demands app restarts);
//! the keyboard layout is stored and written as dms/eingabe.kdl —
//! new installations load it from the factory, existing devices
//! with the next system round.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::column;
use matrixkit_widgets::iced::{Element, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;

// Morpheus (06.11.2026): the two languages Matrix speaks, as the first-start setup offers them.
pub(crate) const LANGUAGES: [(&str, &str); 2] = [("de-CH,en-GB", "Deutsch (Schweiz)"), ("en-GB,de-CH", "English (UK)")];
pub(crate) const REGIONS: [(&str, &str); 3] =
    [("ch", "Schweiz"), ("de", "Deutschland"), ("int", "International")];
pub(crate) const LAYOUTS: [(&str, &str); 4] = [("ch", "Schweiz"), ("gb", "Vereinigtes Königreich"), ("de", "Deutschland"), ("us", "USA")];

pub struct Panel {
    palette: mk::Palette,
    language_idx: usize,
    region_idx: usize,
    keyboard_idx: usize,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Language(usize),
    Region(usize),
    Keyboard(usize),
}

/// Write the keyboard layout as a niri include. R112: the canonical
/// contract is input-layout.kdl; the pre-R111 dms/eingabe.kdl is
/// still written for homes whose chain only includes the legacy path.
/// Only the files — the include reference travels via the skeleton
/// (new installations) or the next system round; NEVER write blindly
/// into a living config (the binds{} lesson of 2026-07-18).
pub(crate) fn input_kdl_write(layout: &str) {
    // The kit's keyboard (mk::language::layout — the on-screen keys) reads it too.
    mk::settings::write("keyboard-layout", layout);
    let home = std::env::var("HOME").unwrap_or_default();
    let body = format!("// Matrix Einstellungen → Sprache & Region (Tastatur-Layout).\ninput {{\n    keyboard {{\n        xkb {{\n            layout \"{layout}\"\n        }}\n    }}\n}}\n");
    // R148: niri watches its config live — a half-written include is a
    // failed reload. Atomic like mk::settings::write: tmp + rename.
    let atomic = |path: String, body: &str| {
        let tmp = format!("{path}.new");
        if std::fs::write(&tmp, body).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    };
    atomic(format!("{home}/.config/niri/input-layout.kdl"), &body);
    let legacy_dir = format!("{home}/.config/niri/dms");
    let _ = std::fs::create_dir_all(&legacy_dir);
    atomic(format!("{legacy_dir}/eingabe.kdl"), &body);
}

impl Panel {
    pub fn new() -> Self {
        let languages = mk::settings::read("languages").unwrap_or_default();
        let region = mk::settings::read("region").unwrap_or_default();
        let layout = mk::settings::read("keyboard-layout").unwrap_or_default();
        Self {
            palette: mk::Palette::load().unwrap_or_default(),
            language_idx: LANGUAGES.iter().position(|(w, _)| *w == languages).unwrap_or(0),
            region_idx: REGIONS.iter().position(|(w, _)| *w == region).unwrap_or(0),
            keyboard_idx: LAYOUTS.iter().position(|(w, _)| *w == layout).unwrap_or(0),
        }
    }

    pub fn tick(&mut self, p: mk::Palette) {
        self.palette = p;
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Language(i) => {
                self.language_idx = i.min(LANGUAGES.len() - 1);
                mk::settings::write("languages", LANGUAGES[self.language_idx].0);
            }
            Msg::Region(i) => {
                self.region_idx = i.min(REGIONS.len() - 1);
                mk::settings::write("region", REGIONS[self.region_idx].0);
            }
            Msg::Keyboard(i) => {
                self.keyboard_idx = i.min(LAYOUTS.len() - 1);
                mk::settings::write("keyboard-layout", LAYOUTS[self.keyboard_idx].0);
                input_kdl_write(LAYOUTS[self.keyboard_idx].0);
            }
        }
        Task::none()
    }

    pub fn footer_text(&self) -> String {
        String::from(
            "Sprache und Formate gelten für neu gestartete Apps — das Tastatur-Layout ab der nächsten Anmeldung",
        )
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let p = self.palette;

        // Example row: the region grammar, demonstrated live (the values
        // from the R100 measurement of the three locales).
        let example = match REGIONS[self.region_idx].0 {
            "de" => "1.234.567,89 · 21.07.2026 · 04:13",
            "int" => "1,234,567.89 · 7/21/2026 · 4:13 AM",
            _ => "1'234'567.89 · 21.07.2026 · 04:13",
        };

        let language = mkw::section(
            "SPRACHE",
            vec![
                mkw::row_item(
                    "Systemsprache",
                    Some("Die erste Sprache der Kette führt; Fehlendes fällt auf Deutsch zurück. Gilt für neu gestartete Apps."),
                    None,
                    Some(
                        iced::widget::container(mkw::segments(
                            &["Deutsch", "English"],
                            self.language_idx,
                            Msg::Language,
                            p,
                        ))
                        .width(iced::Length::Fixed(240.0))
                        .into(),
                    ),
                    p,
                ),
            ],
            p,
        );

        let region = mkw::section(
            "REGION",
            vec![
                mkw::row_item(
                    "Formate",
                    Some("Zahlen, Datum und Uhrzeit — unabhängig von der Sprache (das Leitbild trennt beides)."),
                    None,
                    Some(
                        iced::widget::container(mkw::segments(
                            &["Schweiz", "Deutschland", "International"],
                            self.region_idx,
                            Msg::Region,
                            p,
                        ))
                        .width(iced::Length::Fixed(330.0))
                        .into(),
                    ),
                    p,
                ),
                mkw::row_value("Beispiel", None, example, p),
            ],
            p,
        );

        let keyboard = mkw::section(
            "TASTATUR",
            vec![
                mkw::row_item(
                    "Layout",
                    Some("Wird als Eingabe-Baustein gespeichert; neue Installationen laden ihn ab Werk."),
                    None,
                    Some(
                        iced::widget::container(mkw::segments(
                            &["Schweiz", "Deutschland", "USA"],
                            self.keyboard_idx,
                            Msg::Keyboard,
                            p,
                        ))
                        .width(iced::Length::Fixed(300.0))
                        .into(),
                    ),
                    p,
                ),
            ],
            p,
        );

        column![language, region, keyboard].spacing(mk::spacing::L).into()
    }
}
