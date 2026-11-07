//! Leinwand & Hintergrund — as a panel of Matrix Einstellungen (fusion R41).
//! Was app #10 (matrix-leinwand); now two sections of the ONE
//! settings app. All sliders keep working live.

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, container, Space};
use matrixkit_widgets::iced::{Element, Length, Task};
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use mkw::color;

/// Ghost modes in display order (index 0 = default "Beim Scrollen").
const GHOST_MODES: [&str; 3] = ["Beim Scrollen", "Immer auf Leerraum", "Aus"];

pub struct Panel {
    pub palette: mk::Palette,
    ghost_mode: usize,
    paging: bool,
    click_fetches: bool,
    homeward: bool,
    saved: Option<String>,
    /// Wallpaper: target mode for setting (0 = light, 1 = dark).
    target_mode: usize,
    /// Gallery: (path, 224px thumbnail, from the Matrix system folder?).
    gallery: Vec<(std::path::PathBuf, iced::widget::image::Handle, bool)>,
    gallery_loading: bool,
    light_path: Option<String>,
    dark_path: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    GhostMode(usize),
    Paging(bool),
    ClickHolt(bool),
    Homeward(bool),
    TargetMode(usize),
    SetBackground(usize),
    GalleryLoaded(Vec<(std::path::PathBuf, u32, u32, Vec<u8>, bool)>),
}

const TARGETS: [&str; 2] = ["Hell", "Dunkel"];

fn session_json_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::PathBuf::from(home).join(".local/state/DankMaterialShell/session.json")
}

fn wallpaper_assignment() -> (Option<String>, Option<String>) {
    let Ok(raw) = std::fs::read_to_string(session_json_path()) else {
        return (None, None);
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return (None, None);
    };
    let hol = |k: &str| v.get(k).and_then(|x| x.as_str()).map(String::from).filter(|s| !s.is_empty());
    (hol("wallpaperPathLight"), hol("wallpaperPathDark"))
}

fn wallpaper_assign(light: bool, path: &str) {
    let p = session_json_path();
    let mut v: serde_json::Value = std::fs::read_to_string(&p)
        .ok()
        .and_then(|r| serde_json::from_str(&r).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    v[if light { "wallpaperPathLight" } else { "wallpaperPathDark" }] =
        serde_json::Value::String(path.to_string());
    if let Ok(new) = serde_json::to_string_pretty(&v) {
        let _ = std::fs::write(&p, new);
    }
}

/// Set live: the own wallpaper culture — matrix-wallpaper (app #21)
/// reads ~/.config/matrix/wallpaper-<mode>, repaints and creates
/// the palette (matugen). No DMS in the chain anymore.
fn wallpaper_live_set(path: &str) {
    let light = mk::Palette::load().map(|p| p.is_light).unwrap_or(false);
    mk::wallpaper::write(light, path);
}

/// Generate thumbnails in the background (224px, reference tile size) —
/// the originals (up to 24 MP) are spared from the renderer.
fn thumbs_generate() -> Vec<(std::path::PathBuf, u32, u32, Vec<u8>, bool)> {
    gallery_collect()
        .into_iter()
        .filter_map(|(path, system)| {
            let image = image::open(&path).ok()?;
            // Image-tile family (R44): ONE thumb recipe for wallpaper
            // and gallery — 16:9, 448x252, baked corners.
            let (b, h, raw) = mkw::image::tile_thumb(image);
            Some((path, b, h, raw, system))
        })
        .collect()
}


/// Image candidates. R86: the curated gallery album first — if the
/// user created an album "Hintergrundbilder" in Matrix Dateien, that
/// IS the choice (plus the system images). Otherwise the wide search:
/// ~/Bilder(+subfolders 1 level) and ~/Downloads — but without the
/// screenshot folders that used to flood everything (cap at 24 +
/// alphabetical = all "Bildschirmfoto …").
pub(crate) fn gallery_collect() -> Vec<(std::path::PathBuf, bool)> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut candidates: Vec<(std::path::PathBuf, bool)> = Vec::new();
    let album = std::path::PathBuf::from(format!("{home}/Bilder/Hintergrundbilder"));
    let album_full = std::fs::read_dir(&album)
        .map(|d| {
            d.flatten().any(|e| {
                let extension = e
                    .path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x.to_lowercase())
                    .unwrap_or_default();
                ["jpg", "jpeg", "png"].contains(&extension.as_str())
            })
        })
        .unwrap_or(false);
    let mut folder = vec![String::from("/usr/share/backgrounds/matrix")];
    if album_full {
        folder.push(album.display().to_string());
    } else {
        folder.push(format!("{home}/Bilder"));
        folder.push(format!("{home}/Pictures"));
        folder.push(format!("{home}/Downloads"));
        for base in [format!("{home}/Bilder"), format!("{home}/Pictures")] {
            if let Ok(sub) = std::fs::read_dir(&base) {
                for u in sub.flatten() {
                    let name = u.file_name();
                    let discarded = ["Screenshots", "Bildschirmfotos", "Bildschirmaufnahmen"]
                        .iter()
                        .any(|s| name == *s);
                    if u.path().is_dir() && !discarded {
                        folder.push(u.path().display().to_string());
                    }
                }
            }
        }
    }
    for o in folder {
        if let Ok(entries) = std::fs::read_dir(&o) {
            for e in entries.flatten() {
                let pf = e.path();
                let extension = pf
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|x| x.to_lowercase())
                    .unwrap_or_default();
                if ["jpg", "jpeg", "png"].contains(&extension.as_str()) {
                    candidates.push((pf, o.starts_with("/usr/share")));
                }
            }
        }
    }
    candidates.sort();
    candidates.dedup();
    candidates.truncate(24);
    candidates
}

fn switch_read(name: &str) -> bool {
    mk::settings::read(name).as_deref().is_none_or(|v| v != "off" && v != "aus")
}

fn switch_write(name: &str, on: bool) {
    mk::settings::write(name, if on { "on" } else { "off" });
}

impl Panel {
    pub fn new() -> (Self, Task<Msg>) {
        let ghost_mode = match mk::settings::read("canvas-ghost").as_deref() {
            Some("always") | Some("immer") => 1,
            Some("off") | Some("aus") => 2,
            _ => 0,
        };
        let (light_path, dark_path) = wallpaper_assignment();
        let is_light = mk::Palette::load().map(|p| p.is_light).unwrap_or(false);
        (
            Self {
                palette: mk::Palette::load().unwrap_or_default(),
                ghost_mode,
                paging: switch_read("canvas-paging"),
                click_fetches: switch_read("canvas-click-fetch"),
                homeward: switch_read("canvas-homeward"),
                saved: None,
                target_mode: if is_light { 0 } else { 1 },
                gallery: Vec::new(),
                gallery_loading: true,
                light_path,
                dark_path,
            },
            Task::perform(async { thumbs_generate() }, Msg::GalleryLoaded),
        )
    }

    fn receipt(&mut self, text: &str) {
        self.saved = Some(format!("{text} \u{2713} — wirkt sofort"));
    }

    /// Called by the host per tick: receipt fades, palette follows.
    pub fn tick(&mut self, p: mk::Palette) {
        self.palette = p;
        self.saved = None;
    }

    pub fn footer_text(&self) -> Option<String> {
        self.saved.clone()
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::GhostMode(i) => {
                self.ghost_mode = i.min(2);
                mk::settings::write(
                    "canvas-ghost",
                    ["on-scroll", "always", "off"][self.ghost_mode],
                );
                self.receipt("Fenster-Geist");
            }
            Msg::Paging(on) => {
                self.paging = on;
                switch_write("canvas-paging", on);
                self.receipt("Rad-Blättern");
            }
            Msg::ClickHolt(on) => {
                self.click_fetches = on;
                switch_write("canvas-click-fetch", on);
                self.receipt("Klick holt Fenster");
            }
            Msg::Homeward(on) => {
                self.homeward = on;
                switch_write("canvas-homeward", on);
                self.receipt("Heimweg");
            }
            Msg::TargetMode(i) => self.target_mode = i.min(1),
            Msg::GalleryLoaded(thumbs) => {
                self.gallery = thumbs
                    .into_iter()
                    .map(|(path, b, h, rgba, system)| {
                        (path, iced::widget::image::Handle::from_rgba(b, h, rgba), system)
                    })
                    .collect();
                self.gallery_loading = false;
            }
            Msg::SetBackground(i) => {
                if let Some((path, _, _)) = self.gallery.get(i) {
                    let path_s = path.display().to_string();
                    let light = self.target_mode == 0;
                    // FIRST set live (DMS rewrites the session afterwards from its
                    // memory), THEN our assignment on top — otherwise the patch
                    // loses the race against DMS.
                    if light == self.palette.is_light {
                        wallpaper_live_set(&path_s);
                    }
                    wallpaper_assign(light, &path_s);
                    if light {
                        self.light_path = Some(path_s);
                    } else {
                        self.dark_path = Some(path_s);
                    }
                    self.receipt(if light { "Hintergrund (Hell)" } else { "Hintergrund (Dunkel)" });
                }
            }
        }
        Task::none()
    }

    /// Section "Leinwand": window ghost + navigation.
    pub fn canvas_view(&self) -> Element<'_, Msg> {
        let p = self.palette;
        let in_canvas = mkw::session_is_canvas();

        let mut content = column![].spacing(0);

        // Honest context hint when the canvas is not running at all.
        if !in_canvas {
            content = content
                .push(mkw::section(
                    "",
                    vec![mkw::row_item(
                        "Du bist im klassischen Modus",
                        Some("Diese Einstellungen wirken in der Sitzung „Matrix Leinwand\u{201c} — beim Anmelden wählbar."),
                        Some(mkw::symbol::<Msg>(mkw::symbol::INFO, mk::icon_size::LARGE, p.on_surface_variant)),
                        None,
                        p,
                    )],
                    p,
                ))
                .push(Space::new().height(mk::spacing::L));
        }

        // The window ghost
        content = content.push(mkw::section(
            "DER FENSTER-GEIST",
            vec![
                mkw::row_item(
                    "Anzeigen",
                    Some("Die Icon-Karte der offenen Fenster neben der Maus."),
                    None,
                    Some(mkw::segments(&GHOST_MODES[..], self.ghost_mode, Msg::GhostMode, p)),
                    p,
                ),
            ],
            p,
        ));
        content = content.push(Space::new().height(mk::spacing::L));

        // Navigation
        content = content.push(mkw::section(
            "NAVIGATION",
            vec![
                mkw::row_toggle(
                    "Rad-Blättern",
                    Some("Mausrad auf dem Leerraum wandert durch die Fenster."),
                    None,
                    self.paging,
                    p,
                    Some(Msg::Paging(!self.paging)),
                ),
                mkw::row_toggle(
                    "Klick holt Fenster",
                    Some("Angeschnittene Fenster fahren beim Anklicken ganz herein."),
                    None,
                    self.click_fetches,
                    p,
                    Some(Msg::ClickHolt(!self.click_fetches)),
                ),
                mkw::row_toggle(
                    "Heimweg",
                    Some("Kurzer Klick auf den Leerraum kehrt zu den Fenstern zurück."),
                    None,
                    self.homeward,
                    p,
                    Some(Msg::Homeward(!self.homeward)),
                ),
            ],
            p,
        ));

        content.into()
    }

    /// Section "Hintergrund": preview, target mode, gallery tiles.
    pub fn wallpaper_view(&self, window_b: f32) -> Element<'_, Msg> {
        let p = self.palette;
        // Hintergrund — reference grammar (NSWorkspace desktopImage):
        // large desk preview on top, categorised tiles below.
        let name_from = |p: &Option<String>| -> String {
            p.as_deref()
                .and_then(|s| std::path::Path::new(s).file_stem().and_then(|f| f.to_str()))
                .unwrap_or("\u{2014}")
                .chars()
                .take(28)
                .collect()
        };
        let active_light = self.palette.is_light;
        let active_path = if active_light { &self.light_path } else { &self.dark_path };
        let mut wallpaper_lines: Vec<Element<'_, Msg>> = Vec::new();

        // Desk preview: the active image as a mini monitor.
        if let Some(pf) = active_path.as_deref() {
            let preview = self
                .gallery
                .iter()
                .find(|(q, _, _)| q.display().to_string() == pf)
                .map(|(_, h, _)| h.clone())
                .unwrap_or_else(|| iced::widget::image::Handle::from_path(pf));
            wallpaper_lines.push(
                container(
                    column![
                        container(
                            iced::widget::image(preview)
                                .width(Length::Fixed(300.0))
                                .height(Length::Fixed(169.0))
                                .content_fit(iced::ContentFit::Fill),
                        )
                        .style(move |_| container::Style {
                            border: iced::Border {
                                color: color(p.outline.over(p.surface_container_high, 0.5)),
                                width: 1.0,
                                radius: mk::radius::LARGE.into(),
                            },
                            ..Default::default()
                        }),
                        mkw::txt(
                            format!(
                                "{} \u{b7} {}",
                                name_from(active_path),
                                if active_light { "Hell aktiv" } else { "Dunkel aktiv" }
                            ),
                            mk::typo::CAPTION,
                            p.on_surface_variant,
                        ),
                    ]
                    .spacing(mk::spacing::XS)
                    .align_x(iced::Alignment::Center),
                )
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .padding(iced::Padding { top: mk::spacing::M, bottom: mk::spacing::S, ..iced::Padding::ZERO })
                .into(),
            );
        }
        wallpaper_lines.push(mkw::row_item(
            "Bild setzen für",
            Some("Der Sonnenstand wechselt automatisch zwischen beiden."),
            None,
            Some(mkw::segments(&TARGETS[..], self.target_mode, Msg::TargetMode, p)),
            p,
        ));
        if self.gallery_loading {
            wallpaper_lines.push(mkw::row_item(
                "Bilder werden geladen \u{2026}",
                None,
                None,
                None,
                p,
            ));
        }

        // Image-tile family (mkw::ui) — selection badge like the reference.
        let tile = |i: usize,
                      path: &std::path::PathBuf,
                      handle: &iced::widget::image::Handle|
         -> Element<'_, Msg> {
            let path_s = path.display().to_string();
            let is_light = self.light_path.as_deref() == Some(path_s.as_str());
            let is_dark = self.dark_path.as_deref() == Some(path_s.as_str());
            let marked = is_light || is_dark;
            let badge = if is_light && is_dark {
                Some(String::from("\u{2713} H+D"))
            } else if is_light {
                Some(String::from("\u{2713} Hell"))
            } else if is_dark {
                Some(String::from("\u{2713} Dunkel"))
            } else {
                None
            };
            let name: String = path
                .file_stem()
                .and_then(|f| f.to_str())
                .unwrap_or("?")
                .to_string();
            iced::widget::button(mkw::ui::image_tile(
                Some(handle.clone()),
                mkw::symbol::IMAGE,
                name,
                marked,
                badge,
                p,
            ))
            .padding(0)
            .style(|_, _| iced::widget::button::Style::default())
            .on_press(Msg::SetBackground(i))
            .into()
        };

        // R87/R88: the grid breathes with the real window width.
        // Deduction: sidebar, detail card (2xL), row padding (2xM).
        let window_b = if window_b > 0.0 { window_b } else { 940.0 };
        let area = window_b
            - mkw::ui::SIDEBAR_WIDTH
            - 2.0 * mk::spacing::L
            - 2.0 * mk::spacing::M;

        // Two sections like the reference: system images and own ones.
        for (title_group, system) in [("Matrix", true), ("Deine Bilder", false)] {
            let indices: Vec<usize> = self
                .gallery
                .iter()
                .enumerate()
                .filter(|(_, (_, _, s))| *s == system)
                .map(|(i, _)| i)
                .collect();
            if indices.is_empty() {
                continue;
            }
            wallpaper_lines.push(
                container(mkw::txt(title_group, mk::typo::LABEL, p.on_surface_variant))
                    .padding(iced::Padding { left: mk::spacing::M, top: mk::spacing::S, ..iced::Padding::ZERO })
                    .into(),
            );
            wallpaper_lines.push(
                container(mkw::ui::raster(
                    indices
                        .iter()
                        .map(|&i| {
                            let (path, handle, _) = &self.gallery[i];
                            tile(i, path, handle)
                        })
                        .collect(),
                    area,
                    mkw::ui::IMAGE_TILE_W + mk::spacing::S,
                ))
                .padding(iced::Padding { left: mk::spacing::M, right: mk::spacing::M, bottom: mk::spacing::S, ..iced::Padding::ZERO })
                .into(),
            );
        }
        if self.gallery.is_empty() && !self.gallery_loading {
            wallpaper_lines.push(mkw::row_item(
                "Keine Bilder gefunden",
                Some("Lege Bilder in ~/Bilder oder ~/Downloads (JPG/PNG)."),
                None,
                None,
                p,
            ));
        }
        // Credit duty of the default wallpaper (Pixabay culture).
        wallpaper_lines.push(mkw::row_item(
            "Standard-Hintergrund",
            Some("„Falcon\u{201c} von wfranz auf Pixabay (5350832) — danke!"),
            Some(mkw::symbol::<Msg>(mkw::symbol::PALETTE, mk::font_size::SMALL, p.on_surface_variant)),
            None,
            p,
        ));
        let wallpaper_section = mkw::section("HINTERGRUND", wallpaper_lines, p);
        column![wallpaper_section].into()
    }
}
