//! MatrixKit widgets — the shared window building blocks of all MatrixKit apps.
//!
//! Rule from the design system: a widget makes no design decisions of
//! its own, it references tokens. What lives here looks identical in
//! every app — coherence by construction instead of by discipline.

use iced::widget::{mouse_area, text};
use matrixkit_theme as mk;

/// Stage 0 of the iced replacement: mkw is the ONE gate to iced. Apps
/// no longer declare iced themselves — they speak the names
/// re-exported here (geometry and basic shapes) or
/// `mkw::iced::…` / `mkw::iced_layershell::…` for the rest. When
/// iced falls (stage 4), only these lines fall.
pub use iced;
pub use iced_layershell;
pub use iced_sessionlock;
pub use iced::{
    Alignment, Border, Color, Element, Font, Length, Padding, Pixels, Point, Rectangle, Shadow,
    Size, Subscription, Task, Theme, Vector,
};
pub use iced::widget::{column, container, row, Space};
pub use iced::Result as AppResult;

/// mk::Rgba → iced::Color (the one permitted conversion point).
pub fn color(c: mk::Rgba) -> Color {
    Color::from_rgba(c.r, c.g, c.b, c.a)
}

/// A plain colored plate (rounded rectangle). Charts, swatches and
/// legends build from this — apps never style containers themselves.
pub fn swatch<'a, M: 'a>(tint: mk::Rgba, width: Length, height: f32, radius: f32) -> Element<'a, M> {
    container(Space::new().width(Length::Fill))
        .width(width)
        .height(height)
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(color(tint))),
            border: iced::Border::default().rounded(radius),
            ..Default::default()
        })
        .into()
}

/// A raster image built once and cached by key — for generated
/// pixels like a scannable QR code, where re-encoding (and re-
/// uploading the texture) every frame would be waste. The closure
/// runs only on the first sighting of `key` and returns
/// (width, height, RGBA bytes). Nearest-neighbour scaling keeps
/// module edges razor-sharp.
pub fn pixelbild<'a, M: 'a>(
    key: &str,
    page: f32,
    erzeuge: impl FnOnce() -> (u32, u32, Vec<u8>),
) -> Element<'a, M> {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static CACHE: Mutex<Option<HashMap<String, iced::widget::image::Handle>>> =
        Mutex::new(None);
    let handle = {
        let mut g = CACHE.lock().unwrap();
        let map = g.get_or_insert_with(HashMap::new);
        map.entry(key.to_string())
            .or_insert_with(|| {
                let (w, h, px) = erzeuge();
                iced::widget::image::Handle::from_rgba(w, h, px)
            })
            .clone()
    };
    iced::widget::image(handle)
        .width(Length::Fixed(page))
        .height(Length::Fixed(page))
        .filter_method(iced::widget::image::FilterMethod::Nearest)
        .into()
}

/// A fixed page surface with absolutely placed blocks — the virtual
/// sheet of a document. Every block carries (x, y, width) as a
/// fraction of the page (0..1); the surface itself has fixed logical
/// size, like the sheet of paper it mirrors. Geometry comes from
/// measurement (OCR bounding boxes), not from guessing — the
/// pdf2htmlEX principle.
pub fn page_absolute<'a, M: 'a>(
    breite: f32,
    height: f32,
    bloecke: Vec<(f32, f32, f32, Element<'a, M>)>,
) -> Element<'a, M> {
    let mut layers = iced::widget::Stack::new()
        .width(Length::Fixed(breite))
        .height(Length::Fixed(height));
    layers = layers.push(
        Space::new()
            .width(Length::Fixed(breite))
            .height(Length::Fixed(height)),
    );
    for (x, y, b, el) in bloecke {
        let x = x.clamp(0.0, 0.97);
        let y = y.clamp(0.0, 0.99);
        let b = b.clamp(0.02, 1.0 - x);
        layers = layers.push(
            container(container(el).width(Length::Fixed(b * breite)))
                .padding(iced::Padding {
                    top: y * height,
                    right: 0.0,
                    bottom: 0.0,
                    left: x * breite,
                })
                .width(Length::Fixed(breite))
                .height(Length::Fixed(height)),
        );
    }
    layers.into()
}

/// Put text on the system clipboard. Apps hand this straight back from
/// `update`: `return mkw::kopieren(text);` — one kit call instead of
/// reaching into iced, so the clipboard stays a kit capability.
pub fn kopieren<M: Send + 'static>(text: String) -> Task<M> {
    iced::clipboard::write(text)
}

/// Files dropped onto any window of the app. Map with the app's own
/// message constructor: `mkw::file_drops().map(Msg::FileDropped)`.
pub fn file_drops() -> Subscription<std::path::PathBuf> {
    iced::event::listen_with(|event, _status, _id| {
        if let iced::Event::Window(iced::window::Event::FileDropped(path)) = event {
            Some(path)
        } else {
            None
        }
    })
}

/// Rich text — styled fragments that wrap as one paragraph. The kit's
/// own vocabulary over the span machinery: size, weight, italic and
/// ink arrive as mk types; apps never touch fonts or span internals.
pub mod rich {
    use super::*;

    pub type Span = iced::widget::text::Span<'static, (), iced::Font>;

    /// One styled fragment.
    pub fn span(
        text: impl Into<String>,
        size: f32,
        weight: mk::typo::Weight,
        italic: bool,
        ink: mk::Rgba,
    ) -> Span {
        iced::widget::span(text.into())
            .size(size * mk::typo::factor())
            .font(iced::Font {
                weight: font_weight(weight),
                style: if italic {
                    iced::font::Style::Italic
                } else {
                    iced::font::Style::Normal
                },
                ..iced::Font::with_name("Inter Variable")
            })
            .color(color(ink))
    }

    /// A mono chip on a soft plate (inline code).
    pub fn chip(text: impl Into<String>, size: f32, ink: mk::Rgba, plate: mk::Rgba) -> Span {
        iced::widget::span(text.into())
            .size(size * mk::typo::factor())
            .font(super::mono())
            .color(color(ink))
            .background(color(plate))
            .border(iced::Border::default().rounded(4.0))
            .padding(iced::Padding::from([0.0, 3.0]))
    }

    /// The wrapped paragraph.
    pub fn text<'a, M: 'a>(spans: Vec<Span>) -> Element<'a, M> {
        let w: iced::widget::text::Rich<'a, (), M> = iced::widget::rich_text(spans);
        w.into()
    }
}

/// A typo weight → iced font weight (Inter Variable carries all three).
pub fn font_weight(g: mk::typo::Weight) -> iced::font::Weight {
    match g {
        mk::typo::Weight::Normal => iced::font::Weight::Normal,
        mk::typo::Weight::Medium => iced::font::Weight::Medium,
        mk::typo::Weight::Semibold => iced::font::Weight::Semibold,
    }
}

/// Text in MatrixKit style: a semantic ROLE (mk::typo::*) + color role,
/// instead of raw size/weight. Reference principle: hierarchy is named, not
/// gezahlt. `mkw::txt("Titel", mk::typo::TITLE, p.on_surface)`.
pub fn txt<'a>(
    content: impl iced::widget::text::IntoFragment<'a>,
    style: mk::typo::TextStyle,
    tint: mk::Rgba,
) -> iced::widget::Text<'a> {
    text(content)
        .size(style.size * mk::typo::factor())
        .font(iced::Font {
            weight: font_weight(style.weight),
            ..iced::Font::with_name("Inter Variable")
        })
        .color(color(tint))
}

/// On-screen keyboard radio (R58): input fields report their focus to the
/// matrix-keyboard daemon so the keyboard APPEARS when no physical one
/// is attached (tablet reference grammar). The channel is a Unix
/// datagram — connectionless, fire-and-forget, costs nothing when
/// nobody listens. The heartbeat comes for free: iced's cursor blink
/// redraws focused fields every 500 ms, and the style closure of the
/// input field calls `funken()` — the daemon reads silence as blur.
pub mod keyboard {
    use matrixkit_theme as mk;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    /// The frame maintains it: only the ACTIVE window may transmit — a
    /// focused field in a background window summons no keyboard.
    static WINDOW_ACTIVE: AtomicBool = AtomicBool::new(true);
    /// Throttle: at most one datagram every 700 ms (Unix millis).
    static LASTLY: AtomicU64 = AtomicU64::new(0);
    /// Last touch interaction (ms since epoch) — the frame feeds it.
    /// Whoever touched the screen wants the on-screen keyboard even
    /// with a physical one attached (the iPad-with-keyboard doctrine).
    static TOUCH_LAST: AtomicU64 = AtomicU64::new(0);

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Called by the frame on every touch event.
    pub fn note_touch() {
        TOUCH_LAST.store(now_ms(), Ordering::Relaxed);
    }

    fn touch_fresh() -> bool {
        now_ms().saturating_sub(TOUCH_LAST.load(Ordering::Relaxed)) < 8_000
    }

    pub fn window_active(active: bool) {
        WINDOW_ACTIVE.store(active, Ordering::Relaxed);
    }

    /// The daemon's mailbox (one wire: `mk::onscreen_keyboard`).
    pub fn socket() -> PathBuf {
        mk::onscreen_keyboard::socket()
    }

    /// A word to the daemon — errors are ignored (no daemon = no need).
    pub fn send(wort: &str) {
        mk::onscreen_keyboard::send(wort);
    }

    /// KEEP THE KEYBOARD (R209 family): a steady once-a-second beat for a
    /// surface whose search field must hold the on-screen keyboard while it
    /// is open (Matrix Start). The style-closure beat comes only with
    /// redraws — a resting overlay fell silent and the keyboard left after
    /// 4 s, before the first key (Duo, 01.11.2026). Map it to a message and
    /// call `transmit()` there.
    pub fn heartbeat() -> iced::Subscription<()> {
        crate::tick("keyboard-heartbeat", std::time::Duration::from_secs(1)).map(|_| ())
    }

    /// From the style closure of focused input fields: throttled "open".
    pub fn transmit() {
        if !WINDOW_ACTIVE.load(Ordering::Relaxed) {
            return;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let before = LASTLY.load(Ordering::Relaxed);
        if now.saturating_sub(before) < 700 {
            return;
        }
        if LASTLY
            .compare_exchange(before, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return;
        }
        if touch_fresh() {
            // The touch word overrides the physical-keyboard veto.
            send("open!");
        } else {
            send("open");
        }
    }
}

/// Button grammar — reference principle (ButtonStyle × ButtonRole × ControlSize):
/// A button is described via three named axes, never via its own
/// colors. The STYLE says how present it is; the ROLE what it means
/// (destructive = error color world); the SIZE its density level.
pub use matrixkit_scene::buttons;

/// THE MatrixKit button: label + three axes + optional action
/// (None = locked/inert). Returns the button so callers can still
/// set `.width(...)`.
pub fn button<'a, M: Clone + 'a>(
    label: &'a str,
    style: buttons::Style,
    role: buttons::Role,
    size: buttons::Size,
    p: mk::Palette,
    on: Option<M>,
) -> iced::widget::Button<'a, M> {
    use buttons::*;
    let active = on.is_some();
    let (height, pad_h, typo_style) = match size {
        Size::Small => (26.0, mk::spacing::M, mk::typo::CAPTION),
        Size::Normal => (mk::control::BUTTON_HEIGHT, mk::spacing::L, mk::typo::BODY),
        Size::Large => (44.0, mk::spacing::XL, mk::typo::BODY),
    };
    // Color worlds per style × role — all from the palette, never own values.
    let (ground, font) = match (style, role) {
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
    let font_final = if active { font } else { p.on_surface_variant };
    iced::widget::button(
        txt(label, typo_style, font_final).center(),
    )
    .height(Length::Fixed(height))
    .padding([0, pad_h as u16])
    .on_press_maybe(on)
    .style(move |_, status| {
        let bg = ground.map(|g| {
            let g = if active { g } else { g.over(p.surface_container_high, 0.5) };
            match status {
                iced::widget::button::Status::Hovered => font.over(g, mk::state_layer::HOVER),
                iced::widget::button::Status::Pressed => font.over(g, mk::state_layer::PRESSED),
                _ => g,
            }
        });
        // Borderless: the state layer only appears on hover
        let bg = bg.or(match status {
            iced::widget::button::Status::Hovered if active => {
                Some(font.over(p.surface_container_high, mk::state_layer::HOVER))
            }
            iced::widget::button::Status::Pressed if active => {
                Some(font.over(p.surface_container_high, mk::state_layer::PRESSED))
            }
            _ => None,
        });
        iced::widget::button::Style {
            background: bg.map(|b| color(b).into()),
            border: iced::Border { radius: mk::CORNER_RADIUS.into(), ..Default::default() },
            ..Default::default()
        }
    })
}

/// Linear progress (reference LinearProgressViewStyle): quiet track,
/// Accent fill, token radius. `anteil` 0..1.
pub fn progress<'a, M: 'a>(portion: f32, p: mk::Palette) -> Element<'a, M> {
    let portion = portion.clamp(0.0, 1.0);
    let fill = container(Space::new().height(Length::Fixed(6.0)))
        .width(Length::FillPortion(((portion * 1000.0) as u16).max(1)))
        .style(move |_| container::Style {
            background: Some(color(p.primary).into()),
            border: iced::Border { radius: mk::radius::MINI.into(), ..Default::default() },
            ..Default::default()
        });
    let rest = container(Space::new().height(Length::Fixed(6.0)))
        .width(Length::FillPortion((((1.0 - portion) * 1000.0) as u16).max(1)));
    container(row![fill, rest])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(color(p.on_surface.over(p.surface_container, 0.10)).into()),
            border: iced::Border { radius: mk::radius::MINI.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

/// Elevation — shadows communicate LAYER (does it float?), never drama
/// (design system ch. 3). Two levels: resting card vs floating surface.
pub mod elevation {
    use iced::{Color, Shadow, Vector};

    /// Resting card (section, row): barely lifted, only gives an edge.
    pub fn card() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.10),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 5.0,
        }
    }

    /// Floating surface (root layer, dialog): clearly lifted, like a
    /// reference sheet above the dimmed ground.
    pub fn floating() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.32),
            offset: Vector::new(0.0, 10.0),
            blur_radius: 30.0,
        }
    }
}

/// Bundled window state of the frame — stops the parameter flood:
/// activity (appearsActive), traffic-light hover (glyphs ×/−/+ like the
/// reference) and the entrance spring at app start. An app holds exactly ONE.
pub struct WindowState {
    /// Is the window the active one ("key window")? → mkw::focus_subscription
    pub active: bool,
    /// Mouse over the traffic-light group? → glyphs appear in all three.
    pub lights_hover: bool,
    /// Entrance spring: the content settles softly at start (0→1).
    entrance: mk::motion::Spring,
}

impl WindowState {
    pub fn new() -> Self {
        let mut entrance = mk::motion::Spring::snappy(0.0);
        if mk::motion_reduced() {
            entrance = mk::motion::Spring::snappy(1.0);
        } else {
            entrance.retarget(1.0);
        }
        Self {
            active: true,
            // Dev hook for screenshots: show glyphs without a real mouse
            lights_hover: std::env::var("MATRIXKIT_AMPELN_HOVER").is_ok(),
            entrance,
        }
    }
    pub fn tick(&mut self) {
        self.entrance.tick(1.0 / 60.0);
    }
    pub fn is_animating(&self) -> bool {
        !self.entrance.is_settled()
    }
    /// Progress of the entrance (0..~1, with a slight overshoot).
    pub fn entrance(&self) -> f32 {
        self.entrance.value
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self::new()
    }
}

/// The MatrixKit window header: title left, close right, surface
/// draggable (native compositor gesture). Identical in all apps.
/// Window height in the rolled-up state (classic extract): exactly the
/// Title row — header padding (2×M) plus the row core.
pub const SHADE_HEIGHT: f32 = 50.0;

/// The air a scroll area puts above AND below its content
/// (R112 pocket). It was a bare 18 in the scroll area —
/// the first-light path did not know it and therefore set its lists
/// 18 logical pixels too high (finger probe Matrix Farben,
/// 16.08.2026). Whoever REBUILDS a scroll area takes this name.
pub const SCROLL_PAD: f32 = 18.0;

pub fn theme_header<'a, M: Clone + 'a>(
    title: &'a str,
    p: mk::Palette,
    on_drag: M,
    on_close: M,
    on_title: Option<M>,
    on_shelf: M,
    on_max: M,
    window: &WindowState,
    on_lights_hover: impl Fn(bool) -> M + 'a,
    shade: Option<(M, bool)>,
) -> Element<'a, M> {
    // Reference arrangement: traffic-light buttons left, title centered
    // (clickable → root layer), an optical counterweight right. Colors from
    // the palette instead of fixed traffic-light hex values (hex ban): error/secondary/primary.
    // appearsActive (reference SDK): if the window is not the active one,
    // the lights lose their color (uniform gray) and the title dims —
    // the eye knows immediately which window receives input.
    // And like the reference: hovering over the button group makes ALL
    // three lights show their glyph (× / − / +).
    let active = window.active;
    // R168g: 14 pt by user wish (31.07.) — deliberately ABOVE the
    // measured 12 pt reference (R67); the glyph scales along. R209: the
    // device decides (duo: 24 pt, glyphs always — a finger has no hover).
    let lights_form = mk::device::lights();
    let hover = window.lights_hover || lights_form.glyphs;
    let light_size = lights_form.size;
    // The light group's width (3 lights, 2 gaps) — the title stays centred
    // because the right side mirrors it.
    let light_count = if lights_form.shelf { 3.0 } else { 2.0 };
    let lights_width = light_count * light_size + (light_count - 1.0) * mk::spacing::S;
    let light = |tint: mk::Rgba, msg: M, hint: &'a str, glyph_fn: char| {
        let tint = if active { tint } else { p.outline.over(p.surface_container_high, 0.6) };
        let content: Element<'a, M> = if hover {
            // Dark glyph on the light — reference grammar
            container(symbol::<M>(glyph_fn, light_size * 0.75, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.55 }))
                .width(Length::Fixed(light_size))
                .height(Length::Fixed(light_size))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .into()
        } else {
            Space::new().width(Length::Fixed(light_size)).height(Length::Fixed(light_size)).into()
        };
        tooltip(
            // MatrixUI HarnessFamily: the lights, too, spring under the pointer.
            tap_or_magnify(
                iced::widget::button(content)
                    .padding(0)
                    .on_press(msg)
                    .style(move |_, status| {
                        let bg = match status {
                            iced::widget::button::Status::Hovered => mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }.over(tint, 0.18),
                            iced::widget::button::Status::Pressed => mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }.over(tint, 0.32),
                            _ => tint,
                        };
                        iced::widget::button::Style {
                            background: Some(color(bg).into()),
                            border: iced::Border { radius: mk::radius::capsule(light_size).into(), ..Default::default() },
                            ..Default::default()
                        }
                    }),
                light_size,
                p,
            ),
            hint,
            p,
        )
    };
    // R67 measurement (16.7.2026, living reference file-manager window,
    // pixel zoom): the reference runs 12 pt lights with 8 pt gap.
    // R168g: Matrix deliberately deviates to 14 pt (user wish 31.07.) —
    // a touch system wants larger targets than the pointer reference.
    // Reference: double-click on the title bar maximizes/restores.
    let on_max_dc = on_max.clone();
    let mut light_row = row![light(p.error, on_close, mk::language::t("Schließen"), symbol::CLOSE)].spacing(mk::spacing::S);
    if lights_form.shelf {
        light_row = light_row.push(light(p.secondary, on_shelf, mk::language::t("In die Ablage (Super+M)"), symbol::REMOVE));
    }
    let light_row = light_row.push(light(p.primary, on_max, mk::language::t("Maximieren"), symbol::ADD));
    let lights: Element<'a, M> = mouse_area(light_row)
    .on_enter(on_lights_hover(true))
    .on_exit(on_lights_hover(false))
    .into();

    // Rule: ALL interactive elements in title/navigation carry the
    // Material-State-Layer (Hover 0.12 / Pressed 0.20).
    let title_color = if active { p.on_surface } else { p.on_surface_variant };
    let title: Element<'a, M> = match on_title {
        Some(msg) => magnifier(iced::widget::button(
            txt(title, mk::typo::SUBTITLE, title_color),
        )
        .padding([2, mk::spacing::S as u16])
        .on_press(msg)
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
            iced::widget::button::Style {
                background: bg,
                border: iced::Border { radius: mk::radius::NORMAL.into(), ..Default::default() },
                ..Default::default()
            }
        })),
        None => txt(title, mk::typo::SUBTITLE, title_color).into(),
    };

    // Classic extract (Mac OS 8, "WindowShade"): at the right of the title
    // bar sits the roll box — one click rolls the window up onto its title
    // row, the next rolls it back out. macOS sacrificed this to the Dock
    // in 2001; on the canvas (nothing gets put away) it is the perfect
    // third window gesture. The button fills the 64 pt counterweight.
    let right: Element<'a, M> = match shade {
        Some((msg, rolled_up)) => container(
            tooltip(
                magnifier(
                    iced::widget::button(symbol::<M>(
                        if rolled_up { symbol::UNFOLD_MORE } else { symbol::UNFOLD_LESS },
                        14.0,
                        if active { p.on_surface_variant } else { p.outline },
                    ))
                    .padding([2, 6])
                    .on_press(msg)
                    .style(move |_, status| {
                        let bg = match status {
                            iced::widget::button::Status::Hovered => {
                                Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER)).into())
                            }
                            iced::widget::button::Status::Pressed => {
                                Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::PRESSED)).into())
                            }
                            _ => None,
                        };
                        iced::widget::button::Style {
                            background: bg,
                            border: iced::Border { radius: mk::radius::NORMAL.into(), ..Default::default() },
                            ..Default::default()
                        }
                    }),
                ),
                if rolled_up { "Ausrollen" } else { "Einrollen" },
                p,
            ),
        )
        .width(Length::Fixed(lights_width))
        .align_x(iced::alignment::Horizontal::Right)
        .into(),
        None => Space::new().width(Length::Fixed(lights_width)).into(), // optical balance
    };
    let header = mouse_area(
        container(
            row![
                container(lights).width(Length::Fixed(lights_width)),
                Space::new().width(Length::Fill),
                title,
                Space::new().width(Length::Fill),
                right,
            ]
            .align_y(iced::Alignment::Center),
        )
        .padding([mk::spacing::M as u16, mk::spacing::L as u16])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(color(p.surface_container_high).into()),
            border: iced::Border {
                radius: iced::border::Radius::default()
                    .top_left(mk::CORNER_RADIUS)
                    .top_right(mk::CORNER_RADIUS),
                ..Default::default()
            },
            ..Default::default()
        }),
    )
    .on_press(on_drag)
    .on_double_click(on_max_dc);
    // R209: on a duo device the header flicks the window to the other panel.
    if mk::device::is_duo() {
        flick_area(header)
    } else {
        header.into()
    }
}

/// The HarnessFamily element feedback (R209): a finger device gets the tap
/// answer (tick + ring), a pointer device the strong magnifier (R67).
fn tap_or_magnify<'a, M: 'a>(content: impl Into<Element<'a, M>>, size: f32, p: mk::Palette) -> Element<'a, M> {
    if mk::device::finger_first() {
        tap_answer(content, mk::radius::capsule(size), p)
    } else {
        magnifier_strong(content, 1.18)
    }
}

/// Window body in the theme: header + content + invisible resize grips
/// (right, bottom, corner — native xdg_toplevel gesture). THE standard
/// structure of every MatrixKit app without foreign decoration.
pub fn app_window<'a, M: Clone + 'a>(
    title: &'a str,
    p: mk::Palette,
    content: Element<'a, M>,
    on_drag: M,
    on_close: M,
    resize: impl Fn(iced::window::Direction) -> M + 'a,
    on_title: M,
    on_shelf: M,
    on_max: M,
    root: Option<Element<'a, M>>,
    window: &WindowState,
    on_lights_hover: impl Fn(bool) -> M + 'a,
    shade: Option<(M, bool)>,
) -> Element<'a, M> {
    use iced::widget::stack;
    let header = theme_header(
        title,
        p,
        on_drag,
        on_close,
        Some(on_title),
        on_shelf,
        on_max,
        window,
        on_lights_hover,
        shade,
    );
    // Entrance: the content settles softly from below at start (reference
    // feel — windows never appear abruptly, they arrive).
    let f = window.entrance().clamp(0.0, 1.2);
    let offset = (10.0 * (1.0 - f)).max(0.0);
    let content = container(content).padding(iced::Padding { top: offset, ..iced::Padding::ZERO });
    let content = container(column![header, content])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(color(p.surface).into()),
            ..Default::default()
        });

    let grip = 8.0_f32;
    let right = container(
        mouse_area(Space::new().width(Length::Fixed(grip)).height(Length::Fill))
            .on_press(resize(iced::window::Direction::East))
            .interaction(iced::mouse::Interaction::ResizingHorizontally),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Right);
    let bottom = container(
        mouse_area(Space::new().width(Length::Fill).height(Length::Fixed(grip)))
            .on_press(resize(iced::window::Direction::South))
            .interaction(iced::mouse::Interaction::ResizingVertically),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(iced::alignment::Vertical::Bottom);
    let corner = container(
        mouse_area(
            Space::new()
                .width(Length::Fixed(grip * 2.0))
                .height(Length::Fixed(grip * 2.0)),
        )
        .on_press(resize(iced::window::Direction::SouthEast))
        .interaction(iced::mouse::Interaction::Grab),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Right)
    .align_y(iced::alignment::Vertical::Bottom);

    match root {
        // Root layer open: the app dims out (like a paused VM),
        // the management surface lies above it.
        Some(overlay) => stack![content, overlay].into(),
        None => stack![content, right, bottom, corner].into(),
    }
}

// ============================================================================
// Frame — the shared app harness. Absorbs everything EVERY MatrixKit
// app repeated identically: window gestures, light activity/hover, the
// root layer with password lock, the palette poll incl. living icon and
// the subscription wiring. An app holds exactly ONE `Frame` and a
// single `Msg::Frame(FrameMsg)` variant — new frame features are then
// ONE change here instead of seven in the apps.
// ============================================================================

/// The frame messages (window + root layer + activity). The app wraps
/// them in its own variant: `Msg::Frame(FrameMsg)`.
#[derive(Debug, Clone)]
pub enum FrameMsg {
    Active(bool),
    TouchSeen,
    LightsHover(bool),
    Drag,
    Close,
    Resize(iced::window::Direction),
    Shelf,
    Maximize,
    Size(iced::Size),
    RootToggle,
    RootPassword(String),
    UnlockRoot,
    RootUnlocked(bool),
    Permission(mk::permissions::Permission, bool),
    AnimTick,
    /// Scroll ghost: mouse position (the frame tracks it for the whole window).
    Mouse(iced::Point),
    GhostScroll(iced::widget::scrollable::Viewport),
    GhostIn,
    GhostOut,
    /// Scroll physics: intercepted wheel/trackpad delta.
    Wheel(iced::mouse::ScrollDelta),
    /// Touch round (R59): the finger drags the content (dy per move).
    Pull(f32),
    /// Finger released — velocity (px/s) for the coast-out.
    PullEnd(f32),
    /// HelpLink (reference grammar, round 11): opens Matrix Hilfe with the
    /// app name as search term — the ? button in the About panel.
    Help(String),
    /// Classic extract (Mac OS 8 "WindowShade"): roll the window onto its
    /// roll the window up onto its title bar and back out again.
    Shade,
}

/// The shared state of every MatrixKit window app.
pub struct Frame {
    app_id: &'static str,
    pub palette: mk::Palette,
    watcher: mk::PaletteWatcher,
    /// Living app icon (for the About panel), follows the palette.
    pub icon: Option<iced::widget::image::Handle>,
    pub window: WindowState,
    pub root: RootState,
    pub permissions: mk::permissions::PermissionSet,
    relevant: Vec<mk::permissions::Permission>,
    /// The scroll ghost (Matrix original): scrollbar next to the mouse.
    pub ghost: ScrollGhost,
    /// Scroll physics (reference feel): soft wheel glide + momentum.
    pub roll: RollPhysics,
    roll_id: iced::advanced::widget::Id,
    /// Canvas mode (desktop reinvention): windows are never minimized —
    /// the − light lays the PRIVACY VEIL over the content, the window
    /// stays in place. Runtime truth from /proc (compositor name).
    pub canvas: bool,
    /// Privacy veil active: content covered, a click reopens.
    pub private: bool,
    /// Live window size (R87): the frame hears the resize events anyway
    /// for the window memory — here it also keeps them in mind so views
    /// can breathe along (responsive grids).
    /// At start: the remembered size, else ZERO (= "still unknown").
    pub size: iced::Size,
    /// Classic extract (Mac OS 8): rolled up = only the title row stands.
    pub shade: bool,
    /// The window height before rolling up — remembered for rolling out.
    shade_marker: f32,
}

impl Frame {
    /// `relevante` = the PermissionSet this app actually uses.
    pub fn new(app_id: &'static str, relevant: &[mk::permissions::Permission]) -> Self {
        // R140: every harness app registers its behavior generation in
        // the canon ledger — family behavior changes gate on it so a
        // fix never breaks an existing build (the dyld_sdk_at_least
        // extract).
        mk::canon::record(app_id);
        // R177: enter itself into the permissions list, so that the app
        // appears in the settings under "Apps und Rechte" —
        // even if no switch was ever touched.
        mk::permissions::PermissionSet::register(app_id, relevant);
        let palette = mk::Palette::load().unwrap_or_default();
        Self {
            icon: matrixkit_icons::render_png(app_id, &palette)
                .map(iced::widget::image::Handle::from_bytes),
            palette,
            watcher: mk::PaletteWatcher::new(),
            window: WindowState::new(),
            root: RootState::new(),
            permissions: mk::permissions::PermissionSet::load(app_id),
            relevant: relevant.to_vec(),
            ghost: ScrollGhost::new(),
            roll: RollPhysics::new(),
            roll_id: iced::advanced::widget::Id::new(app_id),
            // Runtime truth: the compositor decides, not the file
            // (the session can never change while the process lives).
            canvas: session_is_canvas(),
            private: std::env::var("MATRIXKIT_PRIVAT").is_ok(),
            size: mk::window::size_read(app_id)
                .map(|(w, h)| iced::Size::new(w, h))
                .unwrap_or(iced::Size::ZERO),
            // Dev hook for screenshots: start rolled up.
            shade: std::env::var("MATRIXKIT_ROLLO").is_ok(),
            shade_marker: 0.0,
            app_id,
        }
    }

    /// Palette poll: did the wallpaper change? Reloads + renders the
    /// icon anew, returns `true` (the app can attach its own follow-ups).
    /// Call from the app's data tick (the 2 s pulse exists anyway).
    pub fn palette_changed(&mut self) -> bool {
        if self.watcher.changed() {
            if let Some(p) = mk::Palette::load() {
                self.palette = p;
                // Icon rendering is expensive (SVG raster) — during the
                // palette fade NOT per frame, only at the end (the watcher
                // edge guarantees exactly one final call).
                if !mk::transition::active() {
                    self.icon = matrixkit_icons::render_png(self.app_id, &p)
                        .map(iced::widget::image::Handle::from_bytes);
                }
                return true;
            }
        }
        false
    }

    /// Is a frame animation running (root spring or entrance)?
    pub fn is_animating(&self) -> bool {
        self.root.is_animating() || self.window.is_animating() || self.ghost.is_animating() || self.roll.running()
    }

    /// scroll_to task for the frame scroll surface (physics drive).
    fn roll_close(&self) -> Task<FrameMsg> {
        iced::advanced::widget::operate(iced::advanced::widget::operation::scrollable::scroll_to(
            self.roll_id.clone(),
            iced::advanced::widget::operation::scrollable::AbsoluteOffset {
                x: None,
                y: Some(self.roll.is),
            },
        ))
    }

    /// Keyboard in the root layer: Esc closes, Tab wanders, Enter toggles
    /// the focused switch (only when unlocked) or "Done". Returns `true`
    /// if the key was consumed (root was open) — then the app must NOT
    /// apply its own focus logic.
    pub fn handle_key(&mut self, t: Key) -> bool {
        // Ctrl+, (reference settings scene): from ANY app into Settings —
        // the frame consumes the key globally, before the root layer.
        if matches!(t, Key::Settings) {
            settings_open();
            return true;
        }
        if matches!(t, Key::Escape) {
            if self.root.is_open() {
                self.root.close();
            }
            return self.root.is_open(); // war offen → verbraucht
        }
        if !self.root.is_open() {
            return false;
        }
        match t {
            Key::Next => self.root.focus.onward(),
            Key::Prev => self.root.focus.back(),
            Key::Activate => {
                let done = self.relevant.len();
                match self.root.focus.current() {
                    Some(i) if i < self.relevant.len() && self.root.unlocked => {
                        let r = self.relevant[i];
                        let new = !self.permissions.allowed(r);
                        self.permissions.set(r, new);
                    }
                    Some(i) if i == done => self.root.close(),
                    _ => {}
                }
            }
            Key::Escape => {}
            // With the root layer open there is nothing to search — consumed,
            // so the app does not focus its search behind the overlay.
            Key::Search => {}
            // Already consumed globally above — here only for completeness.
            Key::Settings => {}
            // The root layer knows no undo — consumed so the app does not
            // roll back values behind the overlay.
            Key::Undo => {}
            Key::Refresh => {}
            // No peek in the root layer — consumed like the others.
            Key::Peek => {}
        }
        true
    }

    pub fn update(&mut self, msg: FrameMsg) -> Task<FrameMsg> {
        match msg {
            FrameMsg::TouchSeen => {
                keyboard::note_touch();
                return Task::none();
            }
            FrameMsg::Active(a) => {
                self.window.active = a;
                // R58: only the active window may summon the on-screen
                // keyboard — else a field transmits behind other windows.
                keyboard::window_active(a);
                Task::none()
            }
            FrameMsg::LightsHover(h) => {
                self.window.lights_hover = h;
                Task::none()
            }
            FrameMsg::AnimTick => {
                if mk::transition::active() {
                    let _ = self.palette_changed();
                }
                self.root.tick();
                self.window.tick();
                self.ghost.tick();
                let drive = self.roll.running();
                self.roll.tick(1.0 / 60.0);
                // The ghost display follows the physics live
                if self.roll.max > 0.0 {
                    self.ghost.offset = (self.roll.is / self.roll.max).clamp(0.0, 1.0);
                }
                if drive || self.roll.running() {
                    self.roll_close()
                } else {
                    Task::none()
                }
            }
            FrameMsg::Drag => iced::window::latest().and_then(iced::window::drag),
            FrameMsg::Close => iced::window::latest().and_then(iced::window::close),
            FrameMsg::Resize(dir) => {
                iced::window::latest().and_then(move |id| iced::window::drag_resize(id, dir))
            }
            FrameMsg::Shelf => {
                if self.canvas {
                    // Canvas mode: never minimise — flip the privacy veil
                    self.private = !self.private;
                } else {
                    mk::window::shelf();
                }
                Task::none()
            }
            FrameMsg::Maximize => iced::window::latest().and_then(iced::window::toggle_maximize),
            FrameMsg::Help(term) => {
                help_open(&term);
                Task::none()
            }
            FrameMsg::Size(g) => {
                self.size = g;
                mk::form::set_window_width(g.width);
                if self.shade {
                    // Rolled up does not count for the window memory — and
                    // if the compositor discards the shade height (or the dev
                    // hook starts rolled up), it is reasserted.
                    if g.height > SHADE_HEIGHT * 1.5 {
                        if self.shade_marker <= 0.0 {
                            self.shade_marker = g.height;
                        }
                        let target = iced::Size::new(g.width, SHADE_HEIGHT);
                        return iced::window::latest()
                            .and_then(move |id| iced::window::resize(id, target));
                    }
                } else {
                    mk::window::size_remember(self.app_id, g.width, g.height);
                }
                Task::none()
            }
            FrameMsg::Shade => {
                self.shade = !self.shade;
                let target = if self.shade {
                    self.shade_marker = self.size.height.max(200.0);
                    iced::Size::new(self.size.width.max(320.0), SHADE_HEIGHT)
                } else {
                    iced::Size::new(self.size.width.max(320.0), self.shade_marker.max(200.0))
                };
                iced::window::latest().and_then(move |id| iced::window::resize(id, target))
            }
            FrameMsg::RootToggle => {
                self.root.toggle();
                self.root.focus.set_count(self.relevant.len() + 1);
                Task::none()
            }
            FrameMsg::RootPassword(pw) => {
                self.root.password = pw;
                self.root.failed_attempt = false;
                Task::none()
            }
            FrameMsg::UnlockRoot => {
                if self.root.password.is_empty() || self.root.check_running {
                    return Task::none();
                }
                self.root.check_running = true;
                let pw = self.root.password.clone();
                Task::perform(async move { mk::permissions::password_verify(&pw) }, FrameMsg::RootUnlocked)
            }
            FrameMsg::RootUnlocked(ok) => {
                self.root.unlock_result(ok);
                Task::none()
            }
            FrameMsg::Mouse(item) => {
                self.ghost.mouse = item;
                Task::none()
            }
            FrameMsg::GhostScroll(vp) => {
                self.ghost.scroll(vp);
                self.roll.sync(vp);
                Task::none()
            }
            FrameMsg::Wheel(delta) => {
                self.roll.input(delta);
                // Direct portions (trackpad / reduced motion) apply
                // immediately; the soft trailing is handled by the AnimTick.
                self.roll_close()
            }
            FrameMsg::Pull(dy) => {
                // R59: content sticks to the finger — apply immediately, no tick.
                self.roll.touch_drag(dy);
                if self.roll.max > 0.0 {
                    self.ghost.offset = (self.roll.is / self.roll.max).clamp(0.0, 1.0);
                }
                self.roll_close()
            }
            FrameMsg::PullEnd(v) => {
                self.roll.touch_end(v);
                Task::none()
            }
            FrameMsg::GhostIn => {
                self.ghost.enter();
                Task::none()
            }
            FrameMsg::GhostOut => {
                self.ghost.leave();
                Task::none()
            }
            FrameMsg::Permission(r, allowed) => {
                if self.root.unlocked {
                    self.permissions.set(r, allowed);
                }
                Task::none()
            }
        }
    }

    /// The frame subscription: activity, size, and — only while something
    /// springs — the 60 fps anim tick. The app batches this with its own
    /// subscriptions (data tick, keyboard).
    pub fn subscription(&self) -> iced::Subscription<FrameMsg> {
        self.subscription_with(true)
    }

    /// Like `subscription`, but the window keys (Ctrl+W/M) are opt-out —
    /// the terminal (R61) gives Ctrl+W to the shell (kill-word), not to
    /// the window. All other frame streams stay identical.
    pub fn subscription_with(&self, window_keys_active: bool) -> iced::Subscription<FrameMsg> {
        let active = focus_subscription(FrameMsg::Active);
        // Touch memory for the on-screen keyboard: every touch event
        // marks the input source; the fields then call with "open!".
        let touch = iced::event::listen_with(|event, _status, _id| {
            matches!(event, iced::Event::Touch(_)).then_some(FrameMsg::TouchSeen)
        });
        // Palette fade: while the transition runs, a 30 ms tick
        // drives the frame — palette_changed() in the AnimTick
        // delivers the intermediate palettes to EVERY harness app.
        let fade = palette_fade_subscription().map(|_| FrameMsg::AnimTick);
        let size = iced::window::resize_events().map(|(_, g)| FrameMsg::Size(g));
        // The window keys (reference NSWindow, round 15): Mod+W/Mod+M culture
        // as Ctrl+W (performClose) and Ctrl+M (performMiniaturize — on the
        // canvas this becomes the privacy veil). They live in the FRAME:
        // every harness app gets them automatically.
        let window_keys = iced::keyboard::listen().filter_map(|event| {
            use iced::keyboard::{Event, Key};
            let Event::KeyPressed { key, modifiers, .. } = event else {
                return None;
            };
            if !modifiers.control() {
                return None;
            }
            match key {
                Key::Character(ref c) if c.as_str() == "w" => Some(FrameMsg::Close),
                Key::Character(ref c) if c.as_str() == "m" => Some(FrameMsg::Shelf),
                _ => None,
            }
        });
        let window_keys = if window_keys_active {
            window_keys
        } else {
            iced::Subscription::none()
        };
        if self.is_animating() {
            let anim = tick("rahmen-anim", std::time::Duration::from_millis(16))
                .map(|_| FrameMsg::AnimTick);
            iced::Subscription::batch([fade, active, touch, size, window_keys, anim])
        } else {
            iced::Subscription::batch([fade, active, touch, size, window_keys])
        }
    }

    /// Build the complete window: header (lights/title), content, resize
    /// grips and — if needed — the STANDARD root layer (About + permissions).
    /// The app only supplies its content + About text.
    pub fn window<'a, M: Clone + 'a>(
        &'a self,
        title: &'a str,
        version: &'a str,
        description: &'a str,
        content: Element<'a, M>,
        emit: impl Fn(FrameMsg) -> M + Clone + 'a,
    ) -> Element<'a, M> {
        let root = self.root.visible().then(|| {
            let emit_r = emit.clone();
            let emit_p = emit.clone();
            root_view(
                RootInfo { name: title, version, description, icon: self.icon.clone() },
                self.palette,
                &self.permissions,
                &self.relevant,
                move |r, b| emit_r(FrameMsg::Permission(r, b)),
                emit(FrameMsg::RootToggle),
                &self.root,
                move |s| emit_p(FrameMsg::RootPassword(s)),
                emit(FrameMsg::UnlockRoot),
                emit(FrameMsg::Help(title.to_string())),
            )
        });
        self.shell(title, content, root, emit)
    }

    /// Like `fenster`, but the app supplies its OWN root overlay (e.g.
    /// Matrix Codes with account management). `root` = None → no overlay.
    pub fn shell<'a, M: Clone + 'a>(
        &'a self,
        title: &'a str,
        content: Element<'a, M>,
        root: Option<Element<'a, M>>,
        emit: impl Fn(FrameMsg) -> M + Clone + 'a,
    ) -> Element<'a, M> {
        let emit_r = emit.clone();
        let emit_h = emit.clone();
        let emit_m = emit.clone();
        // Privacy veil (canvas mode): the content gets covered, the window
        // stays in place — clicking the surface reopens.
        let content: Element<'a, M> = if self.private {
            let p = self.palette;
            mouse_area(
                container(
                    column![
                        symbol::<M>(symbol::VISIBILITY_OFF, 40.0, p.on_surface_variant),
                        Space::new().height(mk::spacing::S),
                        txt("Privat", mk::typo::SUBTITLE, p.on_surface),
                        Space::new().height(mk::spacing::XXS),
                        txt("Klick öffnet wieder", mk::typo::CAPTION, p.on_surface_variant),
                    ]
                    .spacing(0)
                    .align_x(iced::Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .style(move |_| container::Style {
                    background: Some(color(p.surface_container).into()),
                    border: iced::Border { radius: mk::CORNER_RADIUS.into(), ..Default::default() },
                    ..Default::default()
                }),
            )
            .on_press(emit(FrameMsg::Shelf))
            .interaction(iced::mouse::Interaction::Pointer)
            .into()
        } else {
            content
        };
        // Rolled up (classic extract): only the title row — the content
        // vanishes with its height, the window is the header bar.
        let content: Element<'a, M> = if self.shade {
            Space::new().width(Length::Fill).height(Length::Fixed(0.0)).into()
        } else {
            content
        };
        let window = app_window(
            title,
            self.palette,
            content,
            emit(FrameMsg::Drag),
            emit(FrameMsg::Close),
            move |dir| emit_r(FrameMsg::Resize(dir)),
            emit(FrameMsg::RootToggle),
            emit(FrameMsg::Shelf),
            emit(FrameMsg::Maximize),
            root,
            &self.window,
            move |h| emit_h(FrameMsg::LightsHover(h)),
            Some((emit(FrameMsg::Shade), self.shade)),
        );
        // Scroll-ghost layer (Matrix original): the frame tracks the mouse
        // for the whole window and lays the floating display on top.
        let with_mouse = mouse_area(window).on_move(move |pt| emit_m(FrameMsg::Mouse(pt)));
        iced::widget::stack![with_mouse, scroll_ghost(&self.ghost, self.palette)].into()
    }

    /// Scroll surface in ghost style: edge fades, NO edge bar — the
    /// display is handled by the frame's scroll ghost. The standard for
    /// all scrolling content in frame apps.
    /// R190b: How far is it scrolled, as a fraction 0…1? The frame
    /// tracks the position anyway (RollPhysics) — an app that
    /// needs it should READ it instead of inventing a message of its own
    /// and keeping two truths.
    /// Can it be scrolled at all? An effect that lives on
    /// scrolling must not kick in when there is nothing to scroll
    /// — otherwise it hides elements that stand there visibly
    /// (exactly what happened on 04.08.2026).
    pub fn scrollable(&self) -> bool {
        self.roll.max > 1.0
    }

    pub fn scroll_fraction(&self) -> f32 {
        if self.roll.max <= 0.0 {
            return 0.0;
        }
        (self.roll.is / self.roll.max).clamp(0.0, 1.0)
    }

    pub fn scroll_area<'a, M: Clone + 'a>(
        &self,
        content: Element<'a, M>,
        emit: impl Fn(FrameMsg) -> M + Clone + 'a,
    ) -> Element<'a, M> {
        let emit_scroll = emit.clone();
        let emit_wheel = emit.clone();
        let emit_in = emit.clone();
        let emit_pull = emit.clone();
        let emit_end = emit.clone();
        let core = scroll_region_ghost(
            content,
            self.roll_id.clone(),
            move |vp| emit_scroll(FrameMsg::GhostScroll(vp)),
            move |d| emit_wheel(FrameMsg::Wheel(d)),
            emit_in(FrameMsg::GhostIn),
            emit(FrameMsg::GhostOut),
            self.ghost.pockets(),
            self.palette,
        );
        // R59: the swipe surface turns this into direct manipulation — the
        // finger drags the content, release coasts out (UIScrollView).
        swipe_area(
            core,
            move |dy| emit_pull(FrameMsg::Pull(dy)),
            move |v| emit_end(FrameMsg::PullEnd(v)),
        )
    }
}

/// The short side of a duo panel (logical px) — the largest square a new
/// window may ask for there (R209).
const PANEL_SHORT: f32 = 480.0;

/// Window settings in the MatrixKit standard: own decoration,
/// Wayland app id = name of the .desktop file (mandatory rule!).
pub fn window_settings(app_id: &str, width: f32, height: f32) -> iced::window::Settings {
    // R209: a duo panel (489 × 652 lp) is smaller than most apps' desktop
    // size — that size was also the MINIMUM, so the window could not fit
    // the panel and hung over its edges. On a duo the panel decides: no
    // minimum, no remembered desktop size (the forms take over, R207).
    if mk::device::is_duo() {
        return iced::window::Settings {
            size: iced::Size::new(width.min(PANEL_SHORT), height.min(PANEL_SHORT)),
            min_size: None,
            decorations: false,
            platform_specific: iced::window::settings::PlatformSpecific {
                application_id: String::from(app_id),
                ..Default::default()
            },
            ..Default::default()
        };
    }
    // Window memory (reference): remembered size wins, minimum stays.
    let (b, h) = mk::window::size_read(app_id)
        .map(|(gb, gh)| (gb.max(width), gh.max(height)))
        .unwrap_or((width, height));
    iced::window::Settings {
        size: iced::Size::new(b, h),
        min_size: Some(iced::Size::new(width, height)),
        decorations: false,
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: String::from(app_id),
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Executor-agnostic ticker (std::thread + channel) — works in
/// the winit AND the layershell runtime. MatrixKit standard: NEVER
/// iced::time::every (needs Tokio, which we do not run).
pub fn tick(id: &'static str, period: std::time::Duration) -> iced::Subscription<std::time::Instant> {
    iced::Subscription::run_with((id, period.as_millis() as u64), make_ticker)
}

/// TICK WHEN (R209 family): a tick that runs fast only while something needs
/// frames (`busy`) and rests otherwise. Every message redraws the app's
/// surfaces and makes the compositor compose again — a fixed 120 ms tick
/// in the notification daemon alone kept a duo panel composing eight
/// times a second with nothing on screen. Callers name both periods.
pub fn tick_when(
    id: &'static str,
    busy: bool,
    fast: std::time::Duration,
    rest: std::time::Duration,
) -> iced::Subscription<std::time::Instant> {
    tick(id, if busy { fast } else { rest })
}

/// Processes of the Kit — iced-free in `matrixkit_theme::process` (R209).
pub use mk::process;

fn make_ticker(
    input: &(&'static str, u64),
) -> impl futures::Stream<Item = std::time::Instant> {
    let period = std::time::Duration::from_millis(input.1);
    let (tx, rx) = futures::channel::mpsc::unbounded();
    std::thread::spawn(move || loop {
        std::thread::sleep(period);
        if tx.unbounded_send(std::time::Instant::now()).is_err() {
            break; // App ended — let the thread die cleanly
        }
    });
    rx
}

/// Palette-fade subscription: while the transition runs it ticks at 30 ms —
/// the app hangs it onto its pulse and thereby fades smoothly. Outside
/// the fade it costs nothing (Subscription::none).
/// Raw fade ticks — the caller maps them itself (non-capturing):
/// `mkw::palette_fade_subscription().map(|_| Msg::Puls)`.
pub fn palette_fade_subscription() -> iced::Subscription<std::time::Instant> {
    if mk::transition::active() {
        tick("palette-fade", std::time::Duration::from_millis(30))
    } else {
        iced::Subscription::none()
    }
}

/// The MatrixKit switch: DMS pill (track 52×30, circle 24) from the tokens.
pub fn toggle<'a, M: Clone + 'a>(on: bool, p: mk::Palette, on_press: M) -> Element<'a, M> {
    toggle_guarded(on, p, Some(on_press))
}

/// Switch with lock support: without a message it is inert
/// (muted colors, no pointer) — the look of the locked.
pub fn toggle_guarded<'a, M: Clone + 'a>(
    on: bool,
    p: mk::Palette,
    on_press: Option<M>,
) -> Element<'a, M> {
    let locked = on_press.is_none();
    let mut track = if on { p.primary } else { p.on_surface.over(p.surface_container_high, 0.12) };
    let mut circle = if on { p.on_primary } else { p.outline };
    if locked {
        track = track.over(p.surface_container_high, 0.45);
        circle = circle.over(p.surface_container_high, 0.45);
    }
    let pad_page = 3.0;
    let thumb = container(Space::new().width(Length::Fixed(24.0)).height(Length::Fixed(24.0)))
        .style(move |_| container::Style {
            background: Some(color(circle).into()),
            border: iced::Border { radius: mk::radius::NORMAL.into(), ..Default::default() },
            ..Default::default()
        });
    let inner = if on {
        row![Space::new().width(Length::Fill), thumb]
    } else {
        row![thumb, Space::new().width(Length::Fill)]
    };
    let pill = container(inner.align_y(iced::Alignment::Center))
        .padding(pad_page)
        .width(Length::Fixed(mk::control::TOGGLE_TRACK_W))
        .height(Length::Fixed(mk::control::TOGGLE_TRACK_H))
        .style(move |_| container::Style {
            background: Some(color(track).into()),
            border: iced::Border { radius: (mk::control::TOGGLE_TRACK_H / 2.0).into(), ..Default::default() },
            ..Default::default()
        });
    match on_press {
        Some(msg) => magnifier(
            mouse_area(pill)
                .on_press(msg)
                .interaction(iced::mouse::Interaction::Pointer),
        ),
        None => pill.into(),
    }
}

/// The animated entrance of the root layer: a spring drives veil
/// opacity and panel glide. Apps hold this state and forward
/// AnimTicks while `animiert()` — no more wiring is needed.
pub struct RootState {
    target: bool,
    spring: mk::motion::Spring,
    reduced: bool,
    /// Keyboard focus within the root layer (permissions + Done).
    pub focus: Focus,
    /// Permission lock: changes only after password confirmation.
    pub unlocked: bool,
    pub password: String,
    pub check_running: bool,
    pub failed_attempt: bool,
    /// Wobble spring of the lock (wrong password) — reference feedback.
    wobble: mk::motion::Spring,
}

impl RootState {
    pub fn new() -> Self {
        // Dev help: MATRIXKIT_ROOT_OFFEN=1 starts with the root layer open
        let open = std::env::var("MATRIXKIT_ROOT_OFFEN").is_ok();
        Self {
            target: open,
            spring: mk::motion::Spring::snappy(if open { 1.0 } else { 0.0 }),
            reduced: mk::motion_reduced(),
            focus: Focus::new(0),
            unlocked: false,
            password: String::new(),
            check_running: false,
            failed_attempt: false,
            wobble: mk::motion::Spring::new(0.0),
        }
    }

    pub fn toggle(&mut self) {
        self.target = !self.target;
        self.focus.delete();
        // On open as on close: locked (reference lock model)
        self.unlocked = false;
        self.password.clear();
        self.check_running = false;
        self.failed_attempt = false;
        let z = if self.target { 1.0 } else { 0.0 };
        if self.reduced {
            // Accessibility: states jump, nothing glides
            self.spring = mk::motion::Spring::snappy(z);
        } else {
            self.spring.retarget(z);
        }
    }

    pub fn close(&mut self) {
        if self.target {
            self.toggle();
        }
    }

    pub fn is_open(&self) -> bool {
        self.target
    }

    pub fn tick(&mut self) {
        self.spring.tick(1.0 / 60.0);
        self.wobble.tick(1.0 / 60.0);
    }

    pub fn is_animating(&self) -> bool {
        !self.spring.is_settled() || !self.wobble.is_settled()
    }

    /// Current wobble deflection (-1..1) for the lock symbol.
    pub fn wobble_value(&self) -> f32 {
        self.wobble.value
    }

    /// While visible the app must render the overlay (also on exit).
    pub fn visible(&self) -> bool {
        self.target || self.spring.value > 0.01
    }

    pub fn progress(&self) -> f32 {
        self.spring.value
    }

    /// Adopt the result of the password check.
    pub fn unlock_result(&mut self, ok: bool) {
        self.check_running = false;
        self.unlocked = ok;
        self.failed_attempt = !ok;
        if ok {
            self.password.clear();
        } else if !self.reduced {
            // The lock wobbles: stiff spring, little damping = oscillation
            self.wobble = mk::motion::Spring {
                value: 1.0,
                velocity: 0.0,
                target: 0.0,
                stiffness: 900.0,
                damping: 7.0,
            };
        }
    }
}

/// Data for the About section of the root layer.
pub struct RootInfo<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub description: &'a str,
    /// The living app icon (rendered live) — like the reference's About.
    pub icon: Option<iced::widget::image::Handle>,
}

/// The root layer of a MatrixKit app: clicking the app name dims the
/// app out and shows this surface — About (like the reference) and the
/// BINDING PermissionSet. Clicking the gray veil closes.
pub use matrixkit_scene::ROOT_WIDTH;

pub fn root_view<'a, M: Clone + 'a>(
    info: RootInfo<'a>,
    p: mk::Palette,
    permissions: &mk::permissions::PermissionSet,
    relevant: &'a [mk::permissions::Permission],
    on_permission: impl Fn(mk::permissions::Permission, bool) -> M,
    on_close: M,
    state: &RootState,
    on_password: impl Fn(String) -> M + 'a,
    on_unlock: M,
    on_help: M,
) -> Element<'a, M> {
    use iced::widget::stack;
    let focus = &state.focus;
    let f = state.progress().clamp(0.0, 1.2);
    let unlocked = state.unlocked;

    // The gray veil fades in with the spring (clicking it closes)
    let veil_alpha = 0.55 * f.min(1.0);
    let veil = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill)).style(move |_| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.08, 0.08, 0.08, veil_alpha).into()),
                ..Default::default()
            }
        }),
    )
    .on_press(on_close.clone());

    // About section: icon left (like the reference), title + version beside
    let header: Element<'a, M> = match info.icon.clone() {
        Some(handle) => row![
            iced::widget::image(handle)
                .width(Length::Fixed(56.0))
                .height(Length::Fixed(56.0)),
            Space::new().width(mk::spacing::M),
            column![
                text(info.name).size(20).color(color(p.on_surface)),
                Space::new().height(mk::spacing::XXS),
                text(format!("Version {} · MatrixKit-App", info.version))
                    .size(mk::font_size::SMALL)
                    .color(color(p.on_surface_variant)),
                text("Matrix Hardline · matrixhardline@ik.me")
                    .size(mk::font_size::SMALL)
                    .color(color(p.on_surface_variant)),
            ]
            .spacing(0),
        ]
        .align_y(iced::Alignment::Center)
        .into(),
        None => column![
            text(info.name).size(20).color(color(p.on_surface)),
            Space::new().height(mk::spacing::XXS),
            text(format!("Version {} · MatrixKit-App", info.version))
                .size(mk::font_size::SMALL)
                .color(color(p.on_surface_variant)),
            text("Matrix Hardline · matrixhardline@ik.me")
                .size(mk::font_size::SMALL)
                .color(color(p.on_surface_variant)),
        ]
        .spacing(0)
        .into(),
    };
    let mut card = column![
        header,
        Space::new().height(mk::spacing::S),
        text(info.description)
            .size(mk::font_size::MEDIUM)
            .color(color(p.on_surface_variant)),
        Space::new().height(mk::spacing::L),
        separator(p),
        Space::new().height(mk::spacing::M),
        row![
            symbol::<M>(symbol::SHIELD, mk::font_size::MEDIUM, p.on_surface_variant),
            Space::new().width(mk::spacing::XS),
            text("BERECHTIGUNGEN")
                .size(mk::font_size::SMALL)
                .color(color(p.on_surface_variant)),
        ]
        .align_y(iced::Alignment::Center),
        Space::new().height(mk::spacing::S),
    ]
    .spacing(0);

    if relevant.is_empty() {
        card = card.push(
            text("Diese App benötigt keine PermissionSet.")
                .size(mk::font_size::MEDIUM)
                .color(color(p.on_surface_variant)),
        );
    }
    for (i, r) in relevant.iter().enumerate() {
        let allowed = permissions.allowed(*r);
        let in_focus = unlocked && focus.is(i);
        card = card.push(
            container(
                row![
                    text(r.display_name())
                        .size(mk::font_size::MEDIUM)
                        .color(color(if unlocked { p.on_surface } else { p.on_surface_variant })),
                    Space::new().width(Length::Fill),
                    container(toggle_guarded(
                        allowed,
                        p,
                        unlocked.then(|| on_permission(*r, !allowed)),
                    ))
                    .padding(2)
                    .style(move |_| container::Style {
                        border: focus_ring(in_focus, mk::control::TOGGLE_TRACK_H / 2.0 + 2.0, p),
                        ..Default::default()
                    }),
                ]
                .align_y(iced::Alignment::Center),
            )
            .padding([mk::spacing::XS as u16, 0]),
        );
    }

    // The lock (reference model): changes only after password confirmation
    // by the user account — binding, no confirmation, no change.
    if !relevant.is_empty() {
        card = card.push(Space::new().height(mk::spacing::S));
        if unlocked {
            card = card.push(
                row![
                    symbol::<M>(symbol::LOCK_OPEN, mk::font_size::MEDIUM, p.primary),
                    Space::new().width(mk::spacing::XS),
                    text("Entsperrt — Änderungen möglich")
                        .size(mk::font_size::SMALL)
                        .color(color(p.on_surface_variant)),
                ]
                .align_y(iced::Alignment::Center),
            );
        } else {
            let field = iced::widget::text_input(
                if state.check_running { "Prüfe …" } else { "Passwort zum Ändern" },
                &state.password,
            )
            .secure(true)
            .on_input(on_password)
            .on_submit(on_unlock.clone())
            .size(mk::font_size::MEDIUM)
            .padding([mk::spacing::XS as u16, mk::spacing::S as u16])
            .style(move |_, status| iced::widget::text_input::Style {
                background: color(p.surface_container).into(),
                border: iced::Border {
                    radius: (mk::CORNER_RADIUS - 4.0).into(),
                    width: if matches!(status, iced::widget::text_input::Status::Focused { .. }) {
                        2.0
                    } else {
                        1.0
                    },
                    color: color(if matches!(status, iced::widget::text_input::Status::Focused { .. }) {
                        p.primary
                    } else {
                        p.outline.over(p.surface_container_high, 0.4)
                    }),
                },
                icon: color(p.on_surface_variant),
                placeholder: color(p.on_surface_variant),
                value: color(p.on_surface),
                selection: color(p.primary_container),
            });
            let deflection = (state.wobble_value() * 6.0).clamp(-6.0, 6.0);
            card = card.push(
                row![
                    container(symbol::<M>(symbol::LOCK, mk::font_size::MEDIUM, p.on_surface_variant))
                        .padding(iced::Padding {
                            left: (6.0 + deflection).max(0.0),
                            right: (6.0 - deflection).max(0.0),
                            ..iced::Padding::ZERO
                        }),
                    field,
                ]
                .align_y(iced::Alignment::Center),
            );
            if state.failed_attempt {
                card = card.push(Space::new().height(mk::spacing::XXS));
                card = card.push(
                    text("Passwort falsch")
                        .size(mk::font_size::SMALL)
                        .color(color(p.error)),
                );
            }
        }
    }

    card = card.push(Space::new().height(mk::spacing::S));
    if !relevant.is_empty() {
        card = card.push(
            text("Bindend: Ohne Berechtigung erhält die App keinen Zugriff.")
                .size(mk::font_size::SMALL)
                .color(color(p.on_surface_variant)),
        );
    }
    // HelpLink (reference grammar): the round ? button sits like in
    // reference dialogs at the bottom left and opens Matrix Hilfe with app context.
    let help_button = iced::widget::button(
        text(symbol::HELP.to_string())
            .font(iced::Font::with_name(symbol_family()))
            .size(mk::font_size::MEDIUM)
            .color(color(p.on_surface_variant))
            .center(),
    )
    .width(Length::Fixed(40.0))
    .height(Length::Fixed(40.0))
    .on_press(on_help)
    .style(move |_, status| {
        let base = p.surface_container_high;
        let bg = match status {
            iced::widget::button::Status::Hovered => p.on_surface.over(base, mk::state_layer::HOVER),
            iced::widget::button::Status::Pressed => p.on_surface.over(base, mk::state_layer::PRESSED),
            _ => base,
        };
        iced::widget::button::Style {
            background: Some(color(bg).into()),
            border: iced::Border {
                radius: mk::radius::capsule(40.0).into(),
                width: 1.0,
                color: color(p.on_surface_variant.over(p.surface_container_high, 0.35)),
            },
            ..Default::default()
        }
    });
    card = card
        .push(Space::new().height(mk::spacing::L))
        .push(row![
            help_button,
            Space::new().width(mk::spacing::S),
            iced::widget::button(
                text("Fertig")
                    .size(mk::font_size::MEDIUM)
                    .color(color(p.on_primary))
                    .center(),
            )
            .width(Length::Fill)
            .height(Length::Fixed(40.0))
            .on_press(on_close)
            .style({
                let done_focus = focus.is(relevant.len());
                move |_, status| {
                    let base = p.primary;
                    let bg = match status {
                        iced::widget::button::Status::Hovered => p.on_primary.over(base, mk::state_layer::HOVER),
                        iced::widget::button::Status::Pressed => p.on_primary.over(base, mk::state_layer::PRESSED),
                        _ => base,
                    };
                    let mut border = focus_ring(done_focus, mk::CORNER_RADIUS, p);
                    if done_focus {
                        border.color = color(p.on_surface);
                    }
                    iced::widget::button::Style {
                        background: Some(color(bg).into()),
                        border,
                        ..Default::default()
                    }
                }
            }),
        ]);

    // Entrance: the panel glides in from below (Material: emphasized
    // decelerate); the spring overshoot gives the last millimeters life.
    let offset = (80.0 * (1.0 - f)).max(0.0);
    // R178 (user): the root level is not a slip of paper but the
    // DASHBOARD of the app — here it says what it is, what it may do
    // and what it gets from the system. At 320 points it looked like
    // a notice window; now it takes space, but stays bounded towards
    // the centre (ROOT_WIDTH), so the lines stay readably short. It
    // grows with the window and caps at ROOT_WIDTH — on a
    // small surface it may be narrower instead of overflowing.
    let panel = container(
        container(
            container(iced::widget::scrollable(card).height(Length::Shrink))
                .padding(mk::spacing::XXL)
                .width(Length::Fill)
                .max_width(ROOT_WIDTH)
                .style(move |_| container::Style {
                    background: Some(color(p.surface_container_high).into()),
                    border: iced::Border { radius: mk::CORNER_RADIUS.into(), ..Default::default() },
                    shadow: elevation::floating(),
                    ..Default::default()
                }),
        )
        .padding(iced::Padding {
            top: offset + mk::spacing::XL,
            bottom: mk::spacing::XL,
            left: mk::spacing::XL,
            right: mk::spacing::XL,
        })
        // The card now carries Fill (so it grows along) — as a result
        // its carrier fills the whole width, and the centring of the
        // outer frame would come to nothing. So the carrier centres
        // itself. (Bug from the rebuild into the dashboard, R178.)
        .center_x(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    stack![veil, panel].into()
}

/// Empty state — reference ContentUnavailableView grammar (reference SDK):
/// large symbol, title, description, centered in the free surface.
/// For all "nothing here" moments: no stick inserted, no accounts,
/// empty search. Never place naked text into an empty surface.
pub fn empty_state<'a, M: 'a>(
    glyph: char,
    title: &'a str,
    description: &'a str,
    p: mk::Palette,
) -> Element<'a, M> {
    container(
        column![
            symbol::<M>(glyph, 44.0, p.on_surface_variant),
            Space::new().height(mk::spacing::M),
            text(title)
                .size(mk::font_size::LARGE)
                .color(color(p.on_surface)),
            Space::new().height(mk::spacing::XXS),
            text(description)
                .size(mk::font_size::MEDIUM)
                .color(color(p.on_surface_variant))
                .center(),
        ]
        .align_x(iced::Alignment::Center)
        .spacing(0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

/// State of a modal dialog (confirmation) — same mechanics as the
/// root layer: spring entrance, reduce-motion respected.
pub struct DialogState {
    target: bool,
    spring: mk::motion::Spring,
    reduced: bool,
}

impl DialogState {
    pub fn new() -> Self {
        Self {
            target: false,
            spring: mk::motion::Spring::snappy(0.0),
            reduced: mk::motion_reduced(),
        }
    }
    pub fn open(&mut self) {
        self.target = true;
        if self.reduced {
            self.spring = mk::motion::Spring::snappy(1.0);
        } else {
            self.spring.retarget(1.0);
        }
    }
    pub fn close(&mut self) {
        self.target = false;
        if self.reduced {
            self.spring = mk::motion::Spring::snappy(0.0);
        } else {
            self.spring.retarget(0.0);
        }
    }
    pub fn is_open(&self) -> bool {
        self.target
    }
    /// While visible the app must render the overlay (also on exit).
    pub fn visible(&self) -> bool {
        self.target || self.spring.value > 0.01
    }
    pub fn progress(&self) -> f32 {
        self.spring.value
    }
    pub fn tick(&mut self) {
        self.spring.tick(1.0 / 60.0);
    }
    pub fn is_animating(&self) -> bool {
        !self.spring.is_settled()
    }
}

/// Confirmation dialog — reference UI confirmationDialog grammar (reference SDK):
/// Destructive actions NEVER run directly — always through title + message +
/// red action + cancel. Clicking the veil or Esc = cancel
/// (the app wires Esc via keys_subscription). Reference rule: the safe
/// path is the convenient one — cancel comes first, the red action last.
pub fn confirmation<'a, M: Clone + 'a>(
    title: &'a str,
    note: String,
    action: &'a str,
    on_confirm: M,
    on_cancel: M,
    state: &DialogState,
    p: mk::Palette,
) -> Element<'a, M> {
    use iced::widget::stack;
    let f = state.progress().clamp(0.0, 1.2);

    let veil_alpha = 0.55 * f.min(1.0);
    let veil = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill)).style(move |_| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.08, 0.08, 0.08, veil_alpha).into()),
                ..Default::default()
            }
        }),
    )
    .on_press(on_cancel.clone());

    let card = confirmation_card(title, note, action, on_confirm, on_cancel.clone(), p);

    let offset = (80.0 * (1.0 - f)).max(0.0);
    let panel = container(
        container(
            container(card)
                .padding(mk::spacing::XL)
                .width(Length::Fixed(360.0))
                .style(move |_| container::Style {
                    background: Some(color(p.surface_container_high).into()),
                    border: iced::Border { radius: mk::CORNER_RADIUS.into(), ..Default::default() },
                    shadow: elevation::floating(),
                    ..Default::default()
                }),
        )
        .padding(iced::Padding { top: offset, ..iced::Padding::ZERO }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    stack![veil, panel].into()
}

/// The FACE of a confirmation (R209 family): warning sign + title, note,
/// then «Abbrechen» first and the red action last (reference grammar: the
/// safe path is the convenient one). No background of its own — inside an
/// app it sits on `confirmation`'s card, in the shell on compositor glass
/// (`glass_dialog`).
pub fn confirmation_card<'a, M: Clone + 'a>(
    title: &'a str,
    note: String,
    action: &'a str,
    on_confirm: M,
    on_cancel: M,
    p: mk::Palette,
) -> Element<'a, M> {
    column![
        row![
            symbol::<M>(symbol::WARNING, mk::font_size::LARGE, p.error),
            Space::new().width(mk::spacing::XS),
            txt(title, mk::typo::SUBTITLE, p.on_surface),
        ]
        .align_y(iced::Alignment::Center),
        Space::new().height(mk::spacing::S),
        txt(note, mk::typo::BODY, p.on_surface_variant),
        Space::new().height(mk::spacing::L),
        row![
            button("Abbrechen", buttons::Style::Tinted, buttons::Role::Normal, buttons::Size::Normal, p, Some(on_cancel))
                .width(Length::Fill),
            Space::new().width(mk::spacing::S),
            button(action, buttons::Style::Prominent, buttons::Role::Destructive, buttons::Size::Normal, p, Some(on_confirm))
                .width(Length::Fill),
        ],
    ]
    .spacing(0)
    .into()
}

/// The duo part of an app card: what one tap does with the app's windows.
pub struct AppCardDuo<'a, M> {
    /// «Panels tauschen» (windows on both panels) or «Aufs andere Panel».
    pub label: &'a str,
    pub hint: &'a str,
    pub on_press: M,
}

/// The Krabbler section of the app card (R211): the switch and one row
/// per species — (name, appetite, neon, on, message).
pub struct AppCardCrawl<'a, M> {
    pub on: bool,
    pub on_switch: M,
    pub species: Vec<(&'a str, &'a str, mk::Rgba, bool, M)>,
}

/// APP CARD (R210 family, redesigned R211 as a list — the owner 02.11.2026:
/// «the card needs a complete redesign», chosen: «a list like
/// Einstellungen»): the glass card the top bar opens for the focused app.
/// Its name and a line about it, then rows like Matrix Einstellungen —
/// Einstellungen ›, the duo's panel swap, Krabbler (switch + species with
/// their neon and appetite, ticked), and Schliessen in red at the end.
/// Size it with `app_card_size` (same arguments' shape).
pub struct AppCardFace<'a, M> {
    pub title: &'a str,
    pub about: &'a str,
    pub on_settings: Option<M>,
    pub duo: Option<AppCardDuo<'a, M>>,
    pub crawl: Option<AppCardCrawl<'a, M>>,
    pub on_close: M,
    /// Wide enough for two columns (app | Krabbler) — then nothing scrolls.
    pub wide: bool,
}

/// A round colour dot (R211 family): a species, a series, a tag colour.
pub fn dot<'a, M: 'a>(tint: mk::Rgba, size: f32) -> Element<'a, M> {
    container(Space::new().width(size).height(size))
        .style(move |_| container::Style { background: Some(color(tint).into()), border: iced::Border { radius: (size / 2.0).into(), ..Default::default() }, ..Default::default() })
        .into()
}

/// PRESS ROW (R211 family — the app card's grammar, public): the whole row
/// presses; a symbol leads, title (+ description), an optional trailing
/// element (a chevron, a value). `ink` colours the title (error = red).
#[allow(clippy::too_many_arguments)]
pub fn row_press<'a, M: Clone + 'a>(glyph: char, tint: mk::Rgba, title: &'a str, description: Option<&'a str>, ink: mk::Rgba, trailing: Option<Element<'a, M>>, p: mk::Palette, on: M) -> Element<'a, M> {
    let mut text_column = column![txt(title, mk::typo::BODY, ink)].spacing(2);
    if let Some(d) = description {
        text_column = text_column.push(txt(d, mk::typo::CAPTION, p.on_surface_variant));
    }
    let mut line = row![symbol(glyph, mk::icon_size::MEDIUM, tint), container(text_column).width(Length::Fill)].spacing(mk::spacing::M).align_y(iced::Alignment::Center);
    if let Some(t) = trailing {
        line = line.push(t);
    }
    iced::widget::button(container(line).padding([mk::spacing::S + 2.0, mk::spacing::M]).width(Length::Fill))
        .padding(0)
        .on_press(on)
        .style(move |_, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER).with_alpha(0.5)).into()),
                iced::widget::button::Status::Pressed => Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::PRESSED).with_alpha(0.5)).into()),
                _ => None,
            };
            iced::widget::button::Style { background: bg, border: iced::Border { radius: mk::radius::NORMAL.into(), ..Default::default() }, ..Default::default() }
        })
        .into()
}

/// MARK ROW (R211 family): a colour dot, title and description, a tick on
/// the right — choosing several of a kind (the card's Krabbler species,
/// chart series, calendar colours). `indent` moves it under a parent row.
#[allow(clippy::too_many_arguments)]
pub fn row_mark<'a, M: Clone + 'a>(tint: mk::Rgba, title: &'a str, description: Option<&'a str>, marked: bool, indent: f32, p: mk::Palette, on: M) -> Element<'a, M> {
    let mut text_column = column![txt(title, mk::typo::BODY, p.on_surface)].spacing(1).width(Length::Fill);
    if let Some(d) = description {
        text_column = text_column.push(txt(d, mk::typo::CAPTION, p.on_surface_variant));
    }
    let tick = symbol::<M>(if marked { symbol::CHECK_BOX } else { symbol::CHECK_BOX_BLANK }, mk::icon_size::MEDIUM, if marked { p.primary } else { p.on_surface_variant });
    let line = row![container(dot::<M>(if marked { tint } else { tint.with_alpha(0.35) }, 10.0)).width(24.0).center_x(24.0), text_column, tick].spacing(mk::spacing::M).align_y(iced::Alignment::Center);
    iced::widget::button(container(line).padding(iced::Padding { top: mk::spacing::XS + 1.0, bottom: mk::spacing::XS + 1.0, left: mk::spacing::M + indent, right: mk::spacing::M }).width(Length::Fill))
        .padding(0)
        .on_press(on)
        .style(|_, _| iced::widget::button::Style::default())
        .into()
}

pub fn app_card<'a, M: Clone + 'a>(face: AppCardFace<'a, M>, p: mk::Palette) -> Element<'a, M> {
    // The app's side: name, settings, panels, close.
    let mut app = column![
        container(column![txt(mk::elide(face.title, 30), mk::typo::SUBTITLE, p.on_surface), txt(face.about, mk::typo::CAPTION, p.on_surface_variant)].spacing(2))
            .padding([mk::spacing::XS, mk::spacing::M]),
        Space::new().height(mk::spacing::S),
        separator(p),
    ]
    .spacing(0);
    let chevron = || symbol::<M>(symbol::CHEVRON_RIGHT, mk::font_size::MEDIUM, p.on_surface_variant);
    if let Some(m) = face.on_settings {
        app = app.push(row_press(symbol::SETTINGS, p.on_surface_variant, "Einstellungen", None, p.on_surface, Some(chevron()), p, m));
    }
    if let Some(duo) = face.duo {
        app = app.push(row_press(symbol::SWAP_VERT, p.on_surface_variant, duo.label, Some(duo.hint), p.on_surface, Some(chevron()), p, duo.on_press));
    }
    app = app.push(separator(p));
    app = app.push(row_press(symbol::CLOSE, p.error, "Schliessen", None, p.error, None, p, face.on_close));
    // The Krabbler side: the switch and the species.
    let crawl = face.crawl.map(|crawl| {
        let mut c = column![row_toggle("Krabbler", Some("Wortwesen laufen über diese App"), Some(symbol::<M>(symbol::BUG, mk::icon_size::MEDIUM, if crawl.on { p.primary } else { p.on_surface_variant })), crawl.on, p, Some(crawl.on_switch))].spacing(0);
        for (name, diet, neon, on, msg) in crawl.species {
            c = c.push(row_mark(neon, name, Some(diet), on, if face.wide { 0.0 } else { mk::spacing::L }, p, msg));
        }
        c
    });
    match crawl {
        // A wide window: side by side — it fits without scrolling (a drag
        // on a row does not scroll; the owner 02.11.2026: «the card is
        // hard to scroll»).
        Some(c) if face.wide => row![container(app).width(Length::Fill), container(c).width(Length::Fill)].spacing(mk::spacing::L).into(),
        Some(c) => column![app, separator(p), c].spacing(0).into(),
        None => app.into(),
    }
}

/// The card's size for what it shows (settings row, duo row, Krabbler).
pub fn app_card_size(settings: bool, duo: bool, crawl: bool) -> (u32, u32) {
    let mut h = 22 + 70 + 52; // glass padding, head, Schliessen
    if settings {
        h += 48;
    }
    if duo {
        h += 62;
    }
    if crawl {
        h += 66 + 4 * 46;
    }
    (360, h)
}

/// GLASS STRIP (R210 family): a glass band along the bottom edge of a
/// panel, `margin` from the three edges, reserving its own height (the
/// compositor adds the margin: windows end one gap above it). The dock's
/// subtitle strip in cinema mode; any ticker or caption band.
pub mod glass_strip {
    use super::*;
    use crate::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption};

    pub fn settings(output: OutputOption, height: u32, margin: i32) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((0, height)),
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right,
            margin: Some((0, margin, margin, margin)),
            layer: Layer::Top,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: Some(height as i32),
            namespace: Some(String::from("matrix-card")),
            output_option: output,
            ..Default::default()
        }
    }

    /// Centred text on the strip (`px` logical, wrapped to its width).
    pub fn face<'a, M: 'a>(text: String, px: f32, ink: mk::Rgba) -> Element<'a, M> {
        container(
            iced::widget::text(text)
                .size(px)
                .color(color(ink))
                .align_x(iced::alignment::Horizontal::Center)
                .width(Length::Fill),
        )
        .padding([8, 16])
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

/// GLASS DIALOG (R209 family): a shell-level question as two layer
/// surfaces of a daemon — the VEIL (whole screen, dimmed, takes the
/// keyboard for Enter/Esc, a tap cancels) and the CARD (exactly sized,
/// centered, namespace `matrix-card` = compositor glass). Shells that end
/// the session ask here instead of leaving it to the compositor's own
/// «Press Enter to confirm».
pub mod glass_dialog {
    use super::*;
    use crate::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption};

    /// Card size (logical px): wide enough for two finger buttons.
    pub const WIDTH: u32 = 340;
    pub const HEIGHT: u32 = 200;

    pub fn veil_settings(output: OutputOption) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((0, 0)),
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            exclusive_zone: Some(-1),
            namespace: Some(String::from("matrix-veil")),
            output_option: output,
            ..Default::default()
        }
    }

    pub fn card_settings(output: OutputOption) -> NewLayerShellSettings {
        card_settings_sized(output, WIDTH, HEIGHT)
    }

    /// A glass card of its own size (R210: the top bar's app card grows
    /// by its duo section).
    pub fn card_settings_sized(output: OutputOption, width: u32, height: u32) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((width, height)),
            // No anchor: the compositor centers the card.
            anchor: Anchor::empty(),
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: Some(-1),
            namespace: Some(String::from("matrix-card")),
            output_option: output,
            ..Default::default()
        }
    }

    /// A card exactly over a window (R214): anchored top-left at the
    /// window's place on its output, the window's size; the bar's reserved
    /// zone is ignored (-1), so the margins count from the output's edge.
    pub fn card_settings_at(output: OutputOption, x: i32, y: i32, width: u32, height: u32) -> NewLayerShellSettings {
        NewLayerShellSettings {
            size: Some((width.max(1), height.max(1))),
            anchor: Anchor::Top | Anchor::Left,
            margin: Some((y.max(0), 0, 0, x.max(0))),
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: Some(-1),
            namespace: Some(String::from("matrix-card")),
            output_option: output,
            ..Default::default()
        }
    }

    /// The veil's face: dimmed, a tap anywhere cancels.
    pub fn veil<'a, M: Clone + 'a>(on_cancel: M) -> Element<'a, M> {
        mouse_area(
            container(Space::new().width(Length::Fill).height(Length::Fill)).style(|_| container::Style {
                background: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.45).into()),
                ..Default::default()
            }),
        )
        .on_press(on_cancel)
        .into()
    }

    /// The card's face on glass (`glass` false = reduced transparency:
    /// the card draws its own opaque body).
    pub fn card<'a, M: Clone + 'a>(face: Element<'a, M>, glass: bool, p: mk::Palette) -> Element<'a, M> {
        container(face)
            .padding(mk::spacing::XL)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| {
                if glass {
                    container::Style::default()
                } else {
                    container::Style {
                        background: Some(color(p.surface_container_high).into()),
                        border: iced::Border { radius: 22.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                }
            })
            .into()
    }

    /// Enter / Escape from the veil's keyboard (true = confirm).
    pub fn key(event: &iced::Event) -> Option<bool> {
        use iced::keyboard::{key::Named, Event, Key};
        match event {
            iced::Event::Keyboard(Event::KeyPressed { key: Key::Named(Named::Enter), .. }) => Some(true),
            iced::Event::Keyboard(Event::KeyPressed { key: Key::Named(Named::Escape), .. }) => Some(false),
            _ => None,
        }
    }
}

/// The MatrixKit footer: a quiet status row at the card bottom —
/// mode hints, copy feedback, permission messages. A fixed
/// part of the app vocabulary (introduced with Matrix Farben).
pub fn footer<'a, M: 'a>(content: String, p: mk::Palette) -> Element<'a, M> {
    column![
        Space::new().height(mk::spacing::M),
        txt(content, mk::typo::CAPTION, p.on_surface_variant),
    ]
    .spacing(0)
    .into()
}

/// Material Symbols — our SF Symbols counterpart, loaded at runtime
/// from /usr/share/fonts/matrix/. Codepoints instead of ligatures:
/// every constant is a Private Use Area address, the LOADED FONT
/// decides which picture lives there (R102 — symbol sets are a font
/// swap, not a code change).
pub mod flat;
pub mod glass;

pub use matrixkit_scene::symbol;
pub use matrixkit_scene::{
    arm_mono, load_symbol_fonts, mono_family, symbol_family, symbol_family_sized,
    symbol_fill_family, symbol_fill_family_sized, symbol_font_fill, symbol_font_main,
};
// ===========================================================================
// Mono and card families (R65) — born from real duplicates:
// Terminal and Installer both loaded Maple Mono by hand, Installer and
// Overview built their own cards, the boot-medium capsule and the
// device chip were one-offs. From now on: ONE code path per shape.
// ===========================================================================

/// The present-mode canon (R148): mailbox unless the user overrides —
/// fifo blocks on vsync and starves layer surfaces (the staleness
/// saga's stage). ONE call site per app, ONE definition here.
/// Stage 1 of the iced replacement (GRABUNG-ICED-ABLOESUNG.md): ONE
/// runtime gate. Before, every app ran its runtime itself — 24
/// callers of `iced::application`, each with the same litany
/// (single instance, present mode, window, fonts). From here on only
/// THIS file knows that iced exists; the apps say WHAT
/// they are (Program), not HOW they run. When our own
/// mk core comes, the implementation behind this gate changes —
/// and no app notices. The same pattern as mk::canvas for the
/// compositor (R45).
pub mod app {
    use matrixkit_theme as mk;

    /// What a Matrix window app IS: four verbs and a title.
    /// Deliberately iced types in the signatures — they move along
    /// when the foundation changes; the apps write them anyway.
    pub trait Program: Sized + 'static {
        type Msg: Clone + std::fmt::Debug + Send + 'static;
        fn new() -> (Self, iced::Task<Self::Msg>);
        fn update(&mut self, msg: Self::Msg) -> iced::Task<Self::Msg>;
        fn view(&self) -> iced::Element<'_, Self::Msg>;
        fn title(&self) -> String;
        fn subscription(&self) -> iced::Subscription<Self::Msg> {
            iced::Subscription::none()
        }
        /// Camp 65: a separate iced colour space, in case an app
        /// needs one. Exactly ONE does — Markdown: the
        /// Markdown widget paints with the iced theme; without its own
        /// that would give foreign colours on our surface
        /// (proof screenshot 18.7.: brown text on dark).
        /// All others say nothing and get the default —
        /// which is why nothing changes for them.
        fn theme(&self) -> Option<iced::Theme> {
            None
        }
    }

    /// The few REAL differences between the 24 mains — the
    /// survey yielded no more.
    pub struct Options {
        /// Almost all apps: one instance, a second start focuses the
        /// first. The terminal deliberately runs several times.
        pub single_instance: bool,
        /// Only the terminal loads the mono font as well.
        pub mono_font: bool,
        /// Own window settings instead of the kit defaults — so far
        /// only Matrix Web needs this (frameless, minimum size).
        pub settings: Option<iced::window::Settings>,
    }

    impl Default for Options {
        fn default() -> Self {
            Self { single_instance: true, mono_font: false, settings: None }
        }
    }

    /// The normal case: window app with the kit defaults.
    pub fn window<P: Program>(
        app_id: &'static str,
        width: f32,
        height: f32,
    ) -> iced::Result {
        window_with::<P>(app_id, width, height, Options::default())
    }

    /// The special case with options — terminal & co.
    pub fn window_with<P: Program>(
        app_id: &'static str,
        width: f32,
        height: f32,
        opt: Options,
    ) -> iced::Result {
        window_boot::<P>(app_id, width, height, opt, P::new)
    }

    /// Boot with payload: Decimen gets its bundle from the
    /// arguments and hands it to the app — everything else is the
    /// normal way through the gate.
    pub fn window_boot<P: Program>(
        app_id: &'static str,
        width: f32,
        height: f32,
        opt: Options,
        boot: impl Fn() -> (P, iced::Task<P::Msg>) + 'static,
    ) -> iced::Result {
        if opt.single_instance && !mk::window::single_instance(app_id) {
            return Ok(());
        }
        super::present_mailbox();
        let window = opt
            .settings
            .unwrap_or_else(|| super::window_settings(app_id, width, height));
        let runner = iced::application(boot, P::update, P::view)
            .title(|p: &P| p.title())
            // With iced, None means: "take the system colour scheme" — exactly
            // the behaviour of the 22 apps that never called .theme().
            .theme(|p: &P| p.theme())
            .subscription(P::subscription)
            .window(window)
            .font(super::symbol_font_main())
            .font(super::symbol_font_fill())
            .default_font(iced::Font::with_name("Inter Variable"));
        let runner = if opt.mono_font {
            runner.font(super::load_mono_font())
        } else {
            runner
        };
        runner.run()
    }
}

/// Stage 1b: the DAEMON gate — the canvas side of `app`.
/// Four callers carried the same litany (transparent ground,
/// Inter, symbol fonts, background start); now it lives here.
/// The Msg bound is that of the to_layer_message macro: a
/// daemon message can ALWAYS also be a layer instruction.
pub mod app_canvas {
    /// What a canvas daemon IS: four verbs, view per surface.
    pub trait Daemon: Sized + 'static {
        type Msg: Clone
            + std::fmt::Debug
            + Send
            + 'static
            + TryInto<
                iced_layershell::actions::LayerShellCustomActionWithId,
                Error = Self::Msg,
            >;
        fn new() -> (Self, iced::Task<Self::Msg>);
        fn update(&mut self, msg: Self::Msg) -> iced::Task<Self::Msg>;
        fn view(&self, window: iced::window::Id) -> iced::Element<'_, Self::Msg>;
        fn subscription(&self) -> iced::Subscription<Self::Msg> {
            iced::Subscription::none()
        }
    }

    /// The normal case: a daemon without a standing surface — every surface
    /// comes into being via NewLayerShell (dock pattern since R174).
    pub fn background() -> iced_layershell::settings::LayerShellSettings {
        iced_layershell::settings::LayerShellSettings {
            start_mode: iced_layershell::settings::StartMode::Background,
            ..Default::default()
        }
    }

    pub fn run<D: Daemon>(
        name: &'static str,
        layers: iced_layershell::settings::LayerShellSettings,
    ) -> Result<(), iced_layershell::Error> {
        iced_layershell::build_pattern::daemon(D::new, move || String::from(name), D::update, D::view)
            .subscription(D::subscription)
            .style(|_state, _theme| iced::theme::Style {
                background_color: iced::Color::TRANSPARENT,
                text_color: iced::Color::WHITE,
            })
            .settings(iced_layershell::settings::Settings {
                layer_settings: layers,
                default_font: iced::Font::with_name("Inter Variable"),
                fonts: super::load_symbol_fonts(),
                ..Default::default()
            })
            .run()
    }
}

/// GRAPHICS PATH (every Matrix app calls this first): how frames reach the
/// screen. Default: Mailbox (tear-free, low latency). R209 duo: Turnip
/// (Adreno 640) offers only Fifo under Wayland, and wgpu must take Vulkan —
/// left to choose, the daemons (bar, dock: background start, no surface to
/// match) picked OpenGL, which /etc/environment routes through zink; zink's
/// threaded context deadlocked on surface switches (bar frozen opening the
/// glass dialog, 01.11.2026: tc_flush vs. threaded_context_get_renderpass_info).
pub fn present_mailbox() {
    let duo = mk::device::is_duo();
    if std::env::var("ICED_PRESENT_MODE").is_err() {
        std::env::set_var("ICED_PRESENT_MODE", if duo { "fifo" } else { "mailbox" });
    }
    // Vulkan, and Turnip found (no direct-display extensions): the iced-free
    // family, shared with the house's GPU painter.
    mk::gpu_path::prepare();
}

/// The ONE mono font of the system. R165b: Adwaita Mono — the mono
/// the Matrix base actually ships. Maple Mono was the earlier choice;
/// stage 2 dropped the package (dropped.txt), and the old hard path
/// then failed SILENTLY into a garbled fallback (the "circuit board"
/// wordmark in Matrix Terminal, 29.07.2026).
pub fn mono() -> iced::Font {
    iced::Font::with_name("Adwaita Mono")
}

pub use matrixkit_scene::load_mono_font;

/// Editor family (R167): ONE look for every multiline text editor
/// (Markdown, Konstrukt). Extracted from Konstrukt's hand styling;
/// the mono font stays a builder call (`.font(mkw::mono())`) because
/// text_editor owns its font outside the style closure.
pub fn editor_style(
    p: mk::Palette,
) -> impl Fn(&iced::Theme, iced::widget::text_editor::Status) -> iced::widget::text_editor::Style {
    move |_t, _s| iced::widget::text_editor::Style {
        background: iced::Background::Color(color(p.surface_container)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: mk::radius::NORMAL.into(),
        },
        placeholder: color(p.on_surface_variant),
        value: color(p.on_surface),
        selection: iced::Color { a: 0.35, ..color(p.primary) },
    }
}

/// Card family: the quiet content card — fill by size class
/// GROSS, Kit-Hairline, NORMAL-Radius, M-Innenluft.
pub fn card<'a, M: 'a>(content: Element<'a, M>, p: mk::Palette) -> Element<'a, M> {
    container(content)
        .padding(mk::spacing::M)
        .style(move |_| container::Style {
            background: Some(color(p.fill(mk::FillLevel::Large)).into()),
            border: iced::Border {
                radius: mk::radius::NORMAL.into(),
                width: 1.0,
                color: color(p.outline.with_alpha(0.25)),
            },
            ..Default::default()
        })
        .into()
}

/// Card with a meaning color (warning = error, success = primary):
/// tinted ground, colored hairline — the red wall of the installer.
pub fn card_tinted<'a, M: 'a>(
    content: Element<'a, M>,
    tint: mk::Rgba,
    p: mk::Palette,
) -> Element<'a, M> {
    container(content)
        .padding(mk::spacing::L)
        .style(move |_| container::Style {
            background: Some(color(tint.over(p.surface_container_high, 0.10)).into()),
            border: iced::Border {
                radius: mk::radius::NORMAL.into(),
                width: 1.0,
                color: color(tint),
            },
            ..Default::default()
        })
        .into()
}

/// Status capsule: the quiet badge ("Boot-Medium — gesperrt",
/// "Aktiv", counter) — LABEL typo in a THIN-filled capsule.
pub fn status_capsule<'a, M: 'a>(
    text_content: impl iced::widget::text::IntoFragment<'a>,
    p: mk::Palette,
) -> Element<'a, M> {
    container(txt(text_content, mk::typo::LABEL, p.text_level(2)))
        .padding(iced::Padding { top: 3.0, right: 10.0, bottom: 3.0, left: 10.0 })
        .style(move |_| container::Style {
            background: Some(color(p.fill(mk::FillLevel::Thin)).into()),
            border: iced::Border {
                radius: mk::radius::capsule(22.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Code chip: a technical name (device, path, key) in mono on a thin
/// fill — unmistakably clear what is meant.
pub fn code_chip<'a, M: 'a>(
    text_content: impl iced::widget::text::IntoFragment<'a>,
    p: mk::Palette,
) -> Element<'a, M> {
    container(
        text(text_content)
            .font(mono())
            .size(mk::font_size::SMALL)
            .color(color(p.text_level(1))),
    )
    .padding(iced::Padding { top: 2.0, right: 8.0, bottom: 2.0, left: 8.0 })
    .style(move |_| container::Style {
        background: Some(color(p.fill(mk::FillLevel::Thin)).into()),
        border: iced::Border { radius: mk::radius::SMALL.into(), ..Default::default() },
        ..Default::default()
    })
    .into()
}

/// Console card: log/command output in mono on the darkest surface —
/// the terminal kinship for every app that shows logs.
pub fn console<'a, M: 'a>(
    text_content: impl iced::widget::text::IntoFragment<'a>,
    width: f32,
    p: mk::Palette,
) -> Element<'a, M> {
    container(
        text(text_content)
            .font(mono())
            .size(mk::font_size::SMALL)
            .color(color(p.text_level(2))),
    )
    .padding(mk::spacing::M)
    .width(Length::Fixed(width))
    .style(move |_| container::Style {
        background: Some(color(p.surface).into()),
        border: iced::Border {
            radius: mk::radius::NORMAL.into(),
            width: 1.0,
            color: color(p.outline.with_alpha(0.25)),
        },
        ..Default::default()
    })
    .into()
}

/// A symbol in token size and palette color.
///
/// R60 (measured with fonttools, 2026-07-16): Material Symbols Rounded
/// carries ascent 1056 / descent 96 at 960 upm — the natural line box
/// is 1.2 em tall, iced's default makes 1.3 em of it. In tight bars
/// (the bar caps buttons at 32 px) the box overflowed and iced let the
/// BOTTOM padding collapse: symbols slid visibly lower than the text
/// (user finding). With line_height 1.0 the box sits exactly
/// baseline..960 and the glyph center (480) IS the box center —
/// centered by construction, in every row.
pub fn symbol<'a, M: 'a>(glyph: char, size: f32, tint: mk::Rgba) -> Element<'a, M> {
    symbol_weight(glyph, size, tint, mk::typo::Weight::Normal)
}

/// Symbol with text weight (R65b, SF Symbols extract): next to semibold
/// text the reference shows semibold symbols — stroke width follows the
/// typo. Material Symbols Rounded is a VARIABLE font (wght axis),
/// the desired weight travels as a font attribute to the text stack.
pub fn symbol_weight<'a, M: 'a>(
    glyph: char,
    size: f32,
    tint: mk::Rgba,
    weight: mk::typo::Weight,
) -> Element<'a, M> {
    text(glyph.to_string())
        .font(iced::Font {
            weight: font_weight(weight),
            ..iced::Font::with_name(symbol_family_sized(size))
        })
        .size(size * mk::typo::factor())
        .line_height(iced::widget::text::LineHeight::Relative(1.0))
        .color(color(tint))
        .into()
}

/// A FILLED symbol (R102, SF Symbols grammar: the selected sidebar
/// entry carries the fill variant). Uses the FILL=1 instance of the
/// loaded set; without one it degrades to the outline glyph.
pub fn symbol_fill<'a, M: 'a>(glyph: char, size: f32, tint: mk::Rgba) -> Element<'a, M> {
    text(glyph.to_string())
        .font(iced::Font::with_name(symbol_fill_family_sized(size)))
        .size(size * mk::typo::factor())
        .line_height(iced::widget::text::LineHeight::Relative(1.0))
        .color(color(tint))
        .into()
}

/// Styled scroll area: thin, round bar in token colors — the last
/// foreign element in MatrixKit windows disappears.
pub fn scroll_region<'a, M: 'a>(content: Element<'a, M>, p: mk::Palette) -> Element<'a, M> {
    use iced::widget::scrollable;
    scrollable(content)
        .style(move |_, status| {
            let grip = match status {
                scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => {
                    p.on_surface.over(p.surface_container, 0.45)
                }
                _ => p.on_surface.over(p.surface_container, 0.25),
            };
            let rail = scrollable::Rail {
                background: None,
                border: iced::Border::default(),
                scroller: scrollable::Scroller {
                    background: color(grip).into(),
                    border: iced::Border { radius: mk::radius::MINI.into(), ..Default::default() },
                },
            };
            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: rail,
                horizontal_rail: rail,
                gap: None,
                auto_scroll: scrollable::AutoScroll {
                    background: color(p.surface_container_high).into(),
                    border: iced::Border { radius: mk::radius::NORMAL.into(), ..Default::default() },
                    shadow: iced::Shadow::default(),
                    icon: color(p.primary),
                },
            }
        })
        .height(Length::Fill)
        .into()
}

/// The MatrixKit navigation keys — ONE subscription for all apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Escape,
    /// Tab — focus to the next element
    Next,
    /// Shift+Tab — focus to the previous element
    Prev,
    /// Enter — trigger the focused element
    Activate,
    /// Ctrl+F — focus the search (reference keyboardShortcut grammar, round 11)
    Search,
    /// Ctrl+, — open Matrix Einstellungen (reference settings scene, round 12).
    /// The frame consumes the key globally; apps never see it.
    Settings,
    /// Ctrl+Z — take back the latest change (reference UndoManager, round 13).
    Undo,
    /// Ctrl+R — reload the content (reference refreshable, round 14).
    Refresh,
    /// Space — the peek (Quick Look grammar, R98): fleeting preview
    /// of the selected. Apps without a peek ignore the key.
    Peek,
}

/// Keyboard subscription: delivers the navigation keys as app messages.
/// Window activity (reference `appearsActive`, reference SDK): true if
/// the window is the active one ("key window"). The frame uses it to dim
/// lights and title — like the reference, where inactive windows lose color.
pub fn focus_subscription<M: Send + 'static>(map: fn(bool) -> M) -> iced::Subscription<M> {
    iced::window::events()
        .filter_map(|(_id, event)| match event {
            iced::window::Event::Focused => Some(true),
            iced::window::Event::Unfocused => Some(false),
            _ => None,
        })
        .with(map)
        .map(|(map, active)| map(active))
}

/// Tooltip (reference `.help()`): a short hint on mouse hover.
/// For symbol buttons whose meaning is not self-explanatory.
pub fn tooltip<'a, M: 'a>(
    content: Element<'a, M>,
    hint: &'a str,
    p: mk::Palette,
) -> Element<'a, M> {
    iced::widget::tooltip(
        content,
        container(
            text(hint)
                .size(mk::font_size::SMALL)
                .color(color(p.on_surface)),
        )
        .padding([4, 8])
        .style(move |_| container::Style {
            background: Some(color(p.surface_container_high).into()),
            border: iced::Border {
                radius: mk::radius::SMALL.into(),
                width: 1.0,
                color: color(p.outline.over(p.surface_container_high, 0.4)),
            },
            ..Default::default()
        }),
        iced::widget::tooltip::Position::Bottom,
    )
    .gap(6)
    .into()
}

/// Redaction (reference `.redacted(.placeholder)`): a quiet gray
/// placeholder bar for content still loading — the surface shows the
/// SHAPE of what is coming, never a spinner circus.
pub fn redaction<'a, M: 'a>(width: f32, height: f32, p: mk::Palette) -> Element<'a, M> {
    container(Space::new().width(Length::Fixed(width)).height(Length::Fixed(height)))
        .style(move |_| container::Style {
            background: Some(color(p.on_surface.over(p.surface_container, 0.10)).into()),
            border: iced::Border { radius: (height / 2.0).into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

pub use key_subscription as keys_subscription;
pub fn key_subscription<M: Send + 'static>(map: fn(Key) -> M) -> iced::Subscription<M> {
    // iced demands capture-free map/filter_map closures — the fn pointer
    // therefore travels as data via Subscription::with.
    iced::keyboard::listen()
        .filter_map(|event| {
            use iced::keyboard::key::Named;
            use iced::keyboard::{Event, Key};
            let Event::KeyPressed { key, modifiers, .. } = event else {
                return None;
            };
            Some(match key {
                Key::Named(Named::Escape) => crate::Key::Escape,
                Key::Named(Named::Tab) if modifiers.shift() => crate::Key::Prev,
                Key::Named(Named::Tab) => crate::Key::Next,
                // Arrow keys navigate lists like the reference (round 15).
                Key::Named(Named::ArrowUp) => crate::Key::Prev,
                Key::Named(Named::ArrowDown) => crate::Key::Next,
                Key::Named(Named::Enter) => crate::Key::Activate,
                Key::Named(Named::Space) => crate::Key::Peek,
                Key::Character(ref c) if c.as_str() == "f" && modifiers.control() => crate::Key::Search,
                Key::Character(ref c) if c.as_str() == "," && modifiers.control() => {
                    crate::Key::Settings
                }
                Key::Character(ref c) if c.as_str() == "z" && modifiers.control() => {
                    crate::Key::Undo
                }
                Key::Character(ref c) if c.as_str() == "r" && modifiers.control() => {
                    crate::Key::Refresh
                }
                _ => return None,
            })
        })
        .with(map)
        .map(|(map, key)| map(key))
}

pub use matrixkit_scene::Focus;

/// The focus ring: two signals (color + strength), as the design system
/// demands for focus. Applicable to any border definition.
pub fn focus_ring(focused: bool, radius: f32, p: mk::Palette) -> iced::Border {
    if focused {
        iced::Border {
            color: color(p.primary),
            width: 2.0,
            radius: radius.into(),
        }
    } else {
        iced::Border { radius: radius.into(), ..Default::default() }
    }
}

// ---------------------------------------------------------------------------
// Form grammar — the reference-27 settings vocabulary (2026-07-05 study):
// section titles small OUTSIDE the card, cards with hairlines between
// rows, descriptions small-gray right under the row title.
// ---------------------------------------------------------------------------

/// Sections of THIS process float on a compositor glass card (R125):
/// their paint thins to a whisper so the refraction carries the body.
/// Only card-window hosts (Zentrale) opt in.
static SECTION_GLASS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn sections_on_glass() {
    SECTION_GLASS.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// A settings section: title + card, rows separated by hairlines.
pub fn section<'a, M: 'a>(
    title: &'a str,
    lines: Vec<Element<'a, M>>,
    p: mk::Palette,
) -> Element<'a, M> {
    let mut card = column![].spacing(0);
    let n = lines.len();
    for (i, z) in lines.into_iter().enumerate() {
        card = card.push(z);
        if i + 1 < n {
            card = card.push(
                container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                    .padding(iced::Padding { left: mk::spacing::M, ..iced::Padding::ZERO })
                    .style(move |_| container::Style {
                        background: Some(
                            color(p.outline.over(p.surface_container_high, 0.18)).into(),
                        ),
                        ..Default::default()
                    }),
            );
        }
    }
    let mut column = column![].spacing(mk::spacing::XS);
    if !title.is_empty() {
        column = column.push(
            container(txt(title, mk::typo::LABEL, p.on_surface_variant))
                .padding(iced::Padding { left: mk::spacing::S, ..iced::Padding::ZERO }),
        );
    }
    let on_glass =
        SECTION_GLASS.load(std::sync::atomic::Ordering::Relaxed) && ui::glass_on();
    let card_alpha = if on_glass { 0.22 } else { 1.0 };
    column
        .push(
            container(card)
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(
                        color(mk::Rgba { a: card_alpha, ..p.surface_container_high }).into(),
                    ),
                    border: iced::Border { radius: mk::CORNER_RADIUS.into(), ..Default::default() },
                    shadow: if on_glass { Default::default() } else { elevation::card() },
                    ..Default::default()
                }),
        )
        .into()
}

/// Base row: [leading element] title (+ description below) … trailing element.
pub fn row_item<'a, M: 'a>(
    title: &'a str,
    description: Option<&'a str>,
    leading: Option<Element<'a, M>>,
    right: Option<Element<'a, M>>,
    p: mk::Palette,
) -> Element<'a, M> {
    let mut text_column = column![
        txt(title, mk::typo::BODY, p.on_surface),
    ]
    .spacing(2);
    if let Some(b) = description {
        text_column = text_column.push(txt(b, mk::typo::CAPTION, p.on_surface_variant));
    }
    let mut r = row![].spacing(mk::spacing::M).align_y(iced::Alignment::Center);
    if let Some(f) = leading {
        r = r.push(f);
    }
    // The text column takes the remaining space and WRAPS — long
    // descriptions must never crush the trailing element.
    r = r.push(container(text_column).width(Length::Fill));
    if let Some(re) = right {
        r = r.push(re);
    }
    container(r)
        .padding(iced::Padding {
            top: mk::spacing::S + 2.0,
            right: mk::spacing::M,
            bottom: mk::spacing::S + 2.0,
            left: mk::spacing::M,
        })
        .width(Length::Fill)
        .into()
}

/// Toggle row (Option: None = locked/dull).
pub fn row_toggle<'a, M: Clone + 'a>(
    title: &'a str,
    description: Option<&'a str>,
    leading: Option<Element<'a, M>>,
    active: bool,
    p: mk::Palette,
    on: Option<M>,
) -> Element<'a, M> {
    row_item(title, description, leading, Some(toggle_guarded(active, p, on)), p)
}

/// Choice family (R167): the reference's radio — several options,
/// ONE circle marked. Symbol-drawn (the loaded font decides the
/// picture, R102), instant-press mechanics, magnifier like every
/// control. A locked item (msg None) grays out like toggle_guarded.
pub fn radio_item<'a, M: Clone + 'a>(
    label: &'a str,
    selected: bool,
    p: mk::Palette,
    on: Option<M>,
) -> Element<'a, M> {
    choice_item(
        if selected { symbol::RADIO_CHECKED } else { symbol::RADIO_UNCHECKED },
        label,
        selected,
        p,
        on,
    )
}

/// Checkbox: the reference uses it for independent yes/no marks in
/// dialogs and lists — a switch is for settings that apply instantly.
pub fn check_item<'a, M: Clone + 'a>(
    label: &'a str,
    checked: bool,
    p: mk::Palette,
    on: Option<M>,
) -> Element<'a, M> {
    choice_item(
        if checked { symbol::CHECK_BOX } else { symbol::CHECK_BOX_BLANK },
        label,
        checked,
        p,
        on,
    )
}

fn choice_item<'a, M: Clone + 'a>(
    glyph: char,
    label: &'a str,
    marked: bool,
    p: mk::Palette,
    on: Option<M>,
) -> Element<'a, M> {
    let locked = on.is_none();
    let mut tint = if marked { p.primary } else { p.on_surface_variant };
    let mut ink = p.on_surface;
    if locked {
        tint = tint.over(p.surface_container_high, 0.45);
        ink = ink.over(p.surface_container_high, 0.45);
    }
    let line = row![
        symbol(glyph, mk::icon_size::MEDIUM, tint),
        txt(label, mk::typo::BODY, ink),
    ]
    .spacing(mk::spacing::S)
    .align_y(iced::Alignment::Center);
    match on {
        Some(msg) => magnifier(instant_button(line, msg)),
        None => line.into(),
    }
}

/// Form row carrying a radio group: the options stack in the detail
/// column, exactly the reference's settings grammar.
pub fn row_radio<'a, M: Clone + 'a>(
    title: &'a str,
    description: Option<&'a str>,
    options: &'a [&'a str],
    selected: usize,
    p: mk::Palette,
    pick: impl Fn(usize) -> M + 'a,
) -> Element<'a, M> {
    let mut group = column![];
    for (i, label) in options.iter().enumerate() {
        group = group.push(radio_item(label, i == selected, p, Some(pick(i))));
    }
    row_item(
        title,
        description,
        None,
        Some(group.spacing(mk::spacing::XS).into()),
        p,
    )
}

/// Value row: gray value on the right (display only).
///
/// R209: in the phone form of the window (duo panel, narrow window) the
/// value moves under its label — side by side they overlapped. The form
/// comes from the frame (`mk::form::current`), no caller changes.
pub fn row_value<'a, M: 'a>(
    title: &'a str,
    description: Option<&'a str>,
    value: impl Into<String>,
    p: mk::Palette,
) -> Element<'a, M> {
    let value = txt(value.into(), mk::typo::BODY, p.on_surface_variant);
    if mk::form::current() == mk::form::Form::Phone {
        let mut lines = iced::widget::column![txt(title, mk::typo::BODY, p.on_surface)].spacing(2);
        if let Some(d) = description {
            lines = lines.push(txt(d, mk::typo::CAPTION, p.on_surface_variant));
        }
        return container(lines.push(value))
            .padding([mk::spacing::S + 2.0, mk::spacing::M])
            .width(Length::Fill)
            .into();
    }
    row_item(title, description, None, Some(value.into()), p)
}

/// Button row: gray pill on the right (reference "Manage …").
pub fn row_button<'a, M: Clone + 'a>(
    title: &'a str,
    description: Option<&'a str>,
    button: &'a str,
    p: mk::Palette,
    on: M,
) -> Element<'a, M> {
    let pill = iced::widget::button(
        text(button).size(mk::font_size::SMALL).color(color(p.on_surface)),
    )
    .padding([4, mk::spacing::M as u16])
    .on_press(on)
    .style(move |_, status| {
        let base = p.on_surface.over(p.surface_container_high, 0.08);
        let bg = match status {
            iced::widget::button::Status::Hovered => p.on_surface.over(base, mk::state_layer::HOVER),
            iced::widget::button::Status::Pressed => p.on_surface.over(base, mk::state_layer::PRESSED),
            _ => base,
        };
        iced::widget::button::Style {
            background: Some(color(bg).into()),
            border: iced::Border { radius: mk::radius::NORMAL.into(), ..Default::default() },
            ..Default::default()
        }
    });
    row_item(title, description, None, Some(pill.into()), p)
}

/// Navigation row: the whole row clickable, › on the right.
pub fn row_nav<'a, M: Clone + 'a>(
    title: &'a str,
    description: Option<&'a str>,
    p: mk::Palette,
    on: M,
) -> Element<'a, M> {
    iced::widget::button(row_item::<M>(
        title,
        description,
        None,
        Some(symbol(symbol::CHEVRON_RIGHT, mk::font_size::MEDIUM, p.on_surface_variant)),
        p,
    ))
    .padding(0)
    .on_press(on)
    .style(move |_, status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => {
                Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER)).into())
            }
            iced::widget::button::Status::Pressed => {
                Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::PRESSED)).into())
            }
            _ => None,
        };
        iced::widget::button::Style { background: bg, ..Default::default() }
    })
    .into()
}

/// Choice row: value + ⌃⌄ on the right (pulldown look; opens app logic).
pub fn row_select<'a, M: Clone + 'a>(
    title: &'a str,
    description: Option<&'a str>,
    value: &'a str,
    p: mk::Palette,
    on: M,
) -> Element<'a, M> {
    let right = iced::widget::button(
        row![
            text(value).size(mk::font_size::MEDIUM).color(color(p.on_surface)),
            Space::new().width(mk::spacing::XS),
            symbol::<M>(symbol::UNFOLD_MORE, mk::font_size::SMALL, p.on_surface_variant),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([2, mk::spacing::S as u16])
    .on_press(on)
    .style(move |_, status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => {
                Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER)).into())
            }
            _ => None,
        };
        iced::widget::button::Style {
            background: bg,
            border: iced::Border { radius: mk::radius::SMALL.into(), ..Default::default() },
            ..Default::default()
        }
    });
    row_item(title, description, None, Some(right.into()), p)
}

// ---------------------------------------------------------------------------
// Settings controls — reference system-settings vocabulary (round 6):
// Slider, segmented choice, stepper, collapsible group. The building
// blocks of a future Matrix Einstellungen app.
// ---------------------------------------------------------------------------

/// Slider (reference slider): accent-filled track, round grip.
/// `bereich` = value range, `wert` = current value, `schritt` = step size.
pub fn slider<'a, M: Clone + 'a>(
    area: std::ops::RangeInclusive<f32>,
    value: f32,
    step: f32,
    on_change: impl Fn(f32) -> M + 'a,
    p: mk::Palette,
) -> Element<'a, M> {
    slider_impl(area, value, step, on_change, None, p)
}

/// Slider that also reports the END of a drag — for values that are
/// expensive to commit (a media seek restarts playback, so the scrub
/// only updates the display and the release commits).
pub fn slider_bis<'a, M: Clone + 'a>(
    area: std::ops::RangeInclusive<f32>,
    value: f32,
    step: f32,
    on_change: impl Fn(f32) -> M + 'a,
    on_release: M,
    p: mk::Palette,
) -> Element<'a, M> {
    slider_impl(area, value, step, on_change, Some(on_release), p)
}

fn slider_impl<'a, M: Clone + 'a>(
    area: std::ops::RangeInclusive<f32>,
    value: f32,
    step: f32,
    on_change: impl Fn(f32) -> M + 'a,
    on_release: Option<M>,
    p: mk::Palette,
) -> Element<'a, M> {
    let mut s = iced::widget::slider(area, value, on_change);
    if let Some(m) = on_release {
        s = s.on_release(m);
    }
    s
        .step(step)
        .height(22.0)
        .style(move |_, status| {
            // Two-channel state feedback (guidelines): the grip grows on hover,
            // while dragging the rest track additionally brightens.
            let (grip, track) = match status {
                iced::widget::slider::Status::Dragged => (10.0, 0.24),
                iced::widget::slider::Status::Hovered => (10.0, 0.15),
                _ => (9.0, 0.15),
            };
            iced::widget::slider::Style {
                rail: iced::widget::slider::Rail {
                    backgrounds: (
                        color(p.primary).into(),
                        color(p.on_surface.over(p.surface_container, track)).into(),
                    ),
                    width: 4.0,
                    border: iced::Border { radius: mk::radius::MINI.into(), ..Default::default() },
                },
                handle: iced::widget::slider::Handle {
                    shape: iced::widget::slider::HandleShape::Circle { radius: grip },
                    background: color(p.primary).into(),
                    border_width: 2.0,
                    border_color: color(p.surface_container),
                },
            }
        })
        .into()
}

/// Segmented choice (reference picker `.segmented`): a tinted track with
/// equal-width segments; the active one is accent-filled. For few,
/// equal-ranking options (light/dark/auto, size steps …).
pub fn segments<'a, M: Clone + 'a>(
    options: &'a [&'a str],
    active: usize,
    on_select: impl Fn(usize) -> M + 'a,
    p: mk::Palette,
) -> Element<'a, M> {
    let mut row = row![].spacing(2);
    for (i, opt) in options.iter().enumerate() {
        let is_active = i == active;
        let font = if is_active { p.on_primary } else { p.on_surface };
        row = row.push(
            iced::widget::button(txt(*opt, mk::typo::CAPTION, font).center().width(Length::Fill))
                .padding([mk::spacing::XS as u16, mk::spacing::S as u16])
                .width(Length::Fill)
                .on_press(on_select(i))
                .style(move |_, status| {
                    let bg = if is_active {
                        Some(color(p.primary).into())
                    } else {
                        match status {
                            iced::widget::button::Status::Hovered => {
                                Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER)).into())
                            }
                            iced::widget::button::Status::Pressed => {
                                Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::PRESSED)).into())
                            }
                            _ => None,
                        }
                    };
                    iced::widget::button::Style {
                        background: bg,
                        border: iced::Border { radius: mk::radius::inner(mk::radius::SMALL, 2.0).into(), ..Default::default() },
                        ..Default::default()
                    }
                }),
        );
    }
    container(row)
        .padding(2)
        .style(move |_| container::Style {
            background: Some(color(p.on_surface.over(p.surface_container, 0.08)).into()),
            border: iced::Border { radius: mk::radius::SMALL.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

/// Stepper (reference stepper): − [value] + for small numeric tweaks.
/// `None` = limit reached — the key dims and ignores input (like the reference).
pub fn stepper<'a, M: Clone + 'a>(
    value: impl iced::widget::text::IntoFragment<'a>,
    on_minus: Option<M>,
    on_plus: Option<M>,
    p: mk::Palette,
) -> Element<'a, M> {
    let key = |glyph: char, msg: Option<M>| {
        let active = msg.is_some();
        let glyph_color = if active {
            p.on_surface
        } else {
            p.on_surface_variant.over(p.surface_container_high, 0.5)
        };
        iced::widget::button(symbol::<M>(glyph, mk::font_size::MEDIUM, glyph_color))
            .padding([2, mk::spacing::S as u16])
            .on_press_maybe(msg)
            .style(move |_, status| {
                let base = p.on_surface.over(p.surface_container_high, if active { 0.08 } else { 0.04 });
                let bg = match status {
                    iced::widget::button::Status::Hovered => base.over(p.surface_container, mk::state_layer::HOVER),
                    iced::widget::button::Status::Pressed => base.over(p.surface_container, mk::state_layer::PRESSED),
                    _ => base,
                };
                iced::widget::button::Style {
                    background: Some(color(bg).into()),
                    border: iced::Border { radius: mk::radius::SMALL.into(), ..Default::default() },
                    ..Default::default()
                }
            })
    };
    row![
        magnifier(key(symbol::REMOVE, on_minus)),
        container(txt(value, mk::typo::BODY, p.on_surface))
            .width(Length::Fixed(56.0))
            .align_x(iced::alignment::Horizontal::Center),
        magnifier(key(symbol::ADD, on_plus)),
    ]
    .spacing(mk::spacing::XS)
    .align_y(iced::Alignment::Center)
    .into()
}

/// An entry in the context menu (reference ContextMenu grammar): label,
/// optional symbol on the left, destructive role in the error colour.
#[derive(Clone)]
pub struct MenuDot<'a, M> {
    pub label: &'a str,
    pub symbol: Option<char>,
    pub destructive: bool,
    pub msg: M,
}

/// Estimated size of the menu card (for edge flips at window and screen
/// borders): a family row is ~31 pt tall, plus shell and pill padding.
/// An estimate suffices — flipping happens BEFORE it gets tight.
pub fn menu_height(items: usize, with_header: bool) -> f32 {
    let lines = items as f32 * 31.0;
    let header = if with_header { 30.0 } else { 0.0 };
    lines + header + 4.0 * mk::spacing::S
}

/// Context menu (reference ContextMenu, round 10 — the overlay build): lays
/// an invisible close surface over the content and the menu card at the
/// click position. The app holds `Option<Point>` (right click opens, see
/// mouse_area::on_right_press + on_move for position tracking), Esc and
/// clicking beside it close.
pub fn context_menu<'a, M: Clone + 'a>(
    content: Element<'a, M>,
    items: Vec<MenuDot<'a, M>>,
    open_at: Option<iced::Point>,
    on_close: M,
    p: mk::Palette,
) -> Element<'a, M> {
    use iced::widget::stack;
    let Some(pos) = open_at else { return content };

    // Close surface: invisible, catches left AND right clicks beside the menu.
    let veil = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill)),
    )
    .on_press(on_close.clone())
    .on_right_press(on_close);

    // MatrixUI MenuFamily: in-app context menus, too, speak the ONE
    // menu language (destructive = error color of the family).
    let count = items.len();
    let entries: Vec<ui::MenuItem<M>> = items
        .into_iter()
        .map(|item| ui::MenuItem::Item {
            glyph: item.symbol,
            title: String::from(item.label),
            color: item.destructive.then_some(p.error),
            msg: item.msg,
        })
        .collect();

    // Anchor the card at the click position (slightly offset like the
    // reference) — and FLIP at the edges like the reference: if space below/
    // right runs out, the menu folds above/left of the click point.
    let height = menu_height(count, false);
    let placed = iced::widget::responsive(move |area| {
        let entries: Vec<ui::MenuItem<M>> = entries.clone();
        let card = ui::menu_family(None, entries, p);
        let mut x = pos.x - 4.0;
        let mut y = pos.y - 4.0;
        if x + ui::MENU_WIDTH > area.width {
            x = pos.x - ui::MENU_WIDTH + 4.0;
        }
        if y + height > area.height {
            y = pos.y - height + 4.0;
        }
        container(card)
            .padding(iced::Padding {
                left: x.max(0.0),
                top: y.max(0.0),
                ..iced::Padding::ZERO
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    });

    stack![content, veil, placed].into()
}

/// Choice menu (reference picker/dropdown): tinted pill with the current
/// value and ⌄; a click opens the option list. For more than three
/// equal-ranking options — with two/three, `segmente` stays first choice.
pub fn select_menu<'a, T, M>(
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
    p: mk::Palette,
) -> Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    select_menu_prompt(options, selected, "", on_select, p)
}

/// select_menu with an invitation shown while nothing is chosen yet.
pub fn select_menu_prompt<'a, T, M>(
    options: &'a [T],
    selected: Option<T>,
    prompt: &'a str,
    on_select: impl Fn(T) -> M + 'a,
    p: mk::Palette,
) -> Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    iced::widget::pick_list(options, selected, on_select)
        .placeholder(prompt)
        .text_size(mk::font_size::MEDIUM)
        .padding([4, mk::spacing::S as u16])
        .style(move |_, status| iced::widget::pick_list::Style {
            text_color: color(p.on_surface),
            placeholder_color: color(p.on_surface_variant),
            handle_color: color(p.on_surface_variant),
            background: color(p.on_surface.over(
                p.surface_container_high,
                if matches!(status, iced::widget::pick_list::Status::Hovered) {
                    mk::state_layer::HOVER
                } else {
                    0.08
                },
            ))
            .into(),
            border: iced::Border { radius: mk::radius::SMALL.into(), ..Default::default() },
        })
        .menu_style(move |_| iced::widget::overlay::menu::Style {
            background: color(p.surface_container_high).into(),
            border: iced::Border {
                radius: mk::radius::SMALL.into(),
                width: 1.0,
                color: color(p.outline.over(p.surface_container_high, 0.3)),
            },
            text_color: color(p.on_surface),
            selected_text_color: color(p.on_primary),
            selected_background: color(p.primary).into(),
            shadow: elevation::floating(),
        })
        .into()
}

/// Form row with a choice menu on the right.
pub fn row_menu<'a, T, M>(
    title: &'a str,
    description: Option<&'a str>,
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
    p: mk::Palette,
) -> Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    row_item(title, description, None, Some(select_menu(options, selected, on_select, p)), p)
}

/// Labeled text field (reference TextField grammar): LABEL above,
/// tinted field with double focus signal (primary color + thicker border),
/// optional error row below (error role instead of wobbling).
pub fn text_field<'a, M: Clone + 'a>(
    label: &'a str,
    value: &str,
    placeholder: &str,
    on_input: impl Fn(String) -> M + 'a,
    on_submit: Option<M>,
    error: Option<&'a str>,
    secret: bool,
    p: mk::Palette,
) -> Element<'a, M> {
    let is_error = error.is_some();
    let field = iced::widget::text_input(&mk::language::tr(placeholder), value)
        .on_input(on_input)
        .on_submit_maybe(on_submit)
        .secure(secret)
        .size(mk::font_size::MEDIUM)
        .padding([mk::spacing::XS as u16, mk::spacing::S as u16])
        .style(move |_, status| {
            let focus = matches!(status, iced::widget::text_input::Status::Focused { .. });
            if focus {
                // R58: a focused field summons the on-screen keyboard —
                // the cursor blink supplies the 500 ms heartbeat for free.
                keyboard::transmit();
            }
            iced::widget::text_input::Style {
                background: color(p.on_surface.over(p.surface_container, 0.08)).into(),
                border: iced::Border {
                    radius: mk::radius::SMALL.into(),
                    width: if focus || is_error { 1.5 } else { 1.0 },
                    color: color(if is_error {
                        p.error
                    } else if focus {
                        p.primary
                    } else {
                        p.outline.over(p.surface_container, 0.35)
                    }),
                },
                icon: color(p.on_surface_variant),
                placeholder: color(p.on_surface_variant),
                value: color(p.on_surface),
                selection: color(p.primary.over(p.surface_container, 0.35)),
            }
        });
    let mut column = column![
        txt(label, mk::typo::LABEL, p.on_surface_variant),
        Space::new().height(mk::spacing::XXS),
        field,
    ]
    .spacing(0);
    if let Some(f) = error {
        column = column
            .push(Space::new().height(mk::spacing::XXS))
            .push(txt(f, mk::typo::CAPTION, p.error));
    }
    column.into()
}

/// The Matrix pulse as the universal loading sign (indeterminate progress —
/// the same three dots as at system start). `phase` is driven by the app
/// via its animation tick (0..1, cyclic); with reduced motion the dots
/// stand evenly dimmed.
pub fn pulse<'a, M: 'a>(phase: f32, base_color: mk::Rgba) -> Element<'a, M> {
    let mut row = row![].spacing(mk::spacing::XS).align_y(iced::Alignment::Center);
    for i in 0..3 {
        let deck = if mk::motion_reduced() {
            0.55
        } else {
            let ph = phase * std::f32::consts::TAU - i as f32 * 0.9;
            0.30 + 0.70 * (0.5 + 0.5 * ph.sin())
        };
        let tint = mk::Rgba { a: base_color.a * deck, ..base_color };
        row = row.push(
            container(Space::new().width(Length::Fixed(8.0)).height(Length::Fixed(8.0))).style(
                move |_| container::Style {
                    background: Some(color(tint).into()),
                    border: iced::Border { radius: mk::radius::capsule(8.0).into(), ..Default::default() },
                    ..Default::default()
                },
            ),
        );
    }
    row.into()
}

/// Simple notice dialog (reference alert): title, message, ONE button.
/// For confirmations with a cancel path keep using `bestaetigung`.
pub fn alert<'a, M: Clone + 'a>(
    title: &'a str,
    note: String,
    on_ok: M,
    state: &DialogState,
    p: mk::Palette,
) -> Element<'a, M> {
    use iced::widget::stack;
    let f = state.progress().clamp(0.0, 1.2);
    let veil_alpha = 0.55 * f.min(1.0);
    let veil = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill)).style(move |_| {
            container::Style {
                background: Some(iced::Color::from_rgba(0.08, 0.08, 0.08, veil_alpha).into()),
                ..Default::default()
            }
        }),
    )
    .on_press(on_ok.clone());

    let card = column![
        txt(title, mk::typo::SUBTITLE, p.on_surface),
        Space::new().height(mk::spacing::S),
        txt(note, mk::typo::BODY, p.on_surface_variant),
        Space::new().height(mk::spacing::L),
        button("OK", buttons::Style::Prominent, buttons::Role::Normal, buttons::Size::Normal, p, Some(on_ok))
            .width(Length::Fill),
    ]
    .spacing(0);

    let offset = (80.0 * (1.0 - f)).max(0.0);
    let panel = container(
        container(
            container(card)
                .padding(mk::spacing::XL)
                .width(Length::Fixed(360.0))
                .style(move |_| container::Style {
                    background: Some(color(p.surface_container_high).into()),
                    border: iced::Border { radius: mk::CORNER_RADIUS.into(), ..Default::default() },
                    shadow: elevation::floating(),
                    ..Default::default()
                }),
        )
        .padding(iced::Padding { top: offset, ..iced::Padding::ZERO }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    stack![veil, panel].into()
}

/// Search field (reference `.searchable`): magnifier, capsule field, × clear.
/// The Esc semantics (dismissSearch) are wired by the app in its key
/// subscription with the same message as `on_leeren`.
/// Input field family (R47): THE text field of the kit — one padding, one
/// radius, one focus language (primary border). Before, nine apps built
/// eleven text_inputs of their own. `geheim` = password dots.
pub fn input_field<'a, M: Clone + 'a>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    on_submit: Option<M>,
    secret: bool,
    p: mk::Palette,
) -> Element<'a, M> {
    let mut field = iced::widget::text_input(&mk::language::tr(placeholder), value)
        .on_input(on_input)
        .secure(secret)
        .size(mk::font_size::MEDIUM)
        .padding(iced::Padding {
            top: 6.0,
            right: mk::spacing::S,
            bottom: 6.0,
            left: mk::spacing::S,
        })
        .style(move |_, status| {
            let focus = matches!(status, iced::widget::text_input::Status::Focused { .. });
            if focus {
                // R58: a focused field summons the on-screen keyboard —
                // the cursor blink supplies the 500 ms heartbeat for free.
                keyboard::transmit();
            }
            iced::widget::text_input::Style {
                background: color(p.on_surface.over(p.surface_container, 0.08)).into(),
                border: iced::Border {
                    color: color(if focus {
                        p.primary
                    } else {
                        p.outline.over(p.surface_container, 0.35)
                    }),
                    width: if focus { 2.0 } else { 1.0 },
                    radius: mk::radius::SMALL.into(),
                },
                icon: color(p.on_surface_variant),
                placeholder: color(p.on_surface_variant),
                value: color(p.on_surface),
                selection: color(p.primary.over(p.surface_container, 0.35)),
            }
        });
    if let Some(m) = on_submit {
        field = field.on_submit(m);
    }
    field.into()
}

/// Notice family (R47): the result banner — primary = success,
/// error = failed. Before, hand-rolled twice (dev,
/// verbindungen) plus a third receipt variant.
pub fn notice<'a, M: 'a>(ok: bool, text: String, p: mk::Palette) -> Element<'a, M> {
    let tint = if ok { p.primary } else { p.error };
    container(txt(text, mk::typo::BODY, tint))
        .padding(mk::spacing::S)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(color(tint.over(p.surface_container, 0.12)).into()),
            border: iced::Border { radius: mk::radius::SMALL.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

pub fn search_field<'a, M: Clone + 'a>(
    value: &str,
    placeholder: &str,
    on_input: impl Fn(String) -> M + 'a,
    on_clear: M,
    p: mk::Palette,
) -> Element<'a, M> {
    let inner = iced::widget::text_input(&mk::language::tr(placeholder), value)
        .id(iced::advanced::widget::Id::new("mkw-suchfeld"))
        .on_input(on_input)
        .size(mk::font_size::MEDIUM)
        .padding(0)
        .width(Length::Fill)
        .style(move |_, status| {
            if matches!(status, iced::widget::text_input::Status::Focused { .. }) {
                keyboard::transmit(); // R58: the search summons the keyboard too
            }
            iced::widget::text_input::Style {
                background: iced::Color::TRANSPARENT.into(),
                border: iced::Border::default(),
                icon: color(p.on_surface_variant),
                placeholder: color(p.on_surface_variant),
                value: color(p.on_surface),
                selection: color(p.primary.over(p.surface_container, 0.35)),
            }
        });
    let mut line = row![
        symbol::<M>(symbol::SEARCH, mk::font_size::MEDIUM, p.on_surface_variant),
        Space::new().width(mk::spacing::XS),
        inner,
    ]
    .align_y(iced::Alignment::Center);
    if !value.is_empty() {
        line = line.push(
            iced::widget::button(symbol::<M>(symbol::CLOSE, mk::font_size::SMALL, p.on_surface_variant))
                .padding(2)
                .on_press(on_clear)
                .style(move |_, status| iced::widget::button::Style {
                    background: matches!(status, iced::widget::button::Status::Hovered)
                        .then(|| color(p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER)).into()),
                    border: iced::Border { radius: mk::radius::capsule(20.0).into(), ..Default::default() },
                    ..Default::default()
                }),
        );
    }
    container(line)
        .padding(iced::Padding {
            top: 6.0,
            right: mk::spacing::S,
            bottom: 6.0,
            left: mk::spacing::S,
        })
        .style(move |_| container::Style {
            background: Some(color(p.on_surface.over(p.surface_container, 0.08)).into()),
            border: iced::Border { radius: mk::radius::capsule(32.0).into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

/// Focus the search (Ctrl+F, reference keyboardShortcut grammar):
/// sets the input focus into the app's mkw::suchfeld.
pub fn focus_search<M: Send + 'static>() -> Task<M> {
    iced::advanced::widget::operate(iced::advanced::widget::operation::focusable::focus(
        iced::advanced::widget::Id::new("mkw-suchfeld"),
    ))
}

/// Is this session running under the canvas compositor? Runtime TRUTH
/// instead of configuration belief: we look for the niri-leinwand process
/// of our own user in /proc (comm is truncated to 15 chars).
pub fn session_is_canvas() -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    for e in entries.flatten() {
        let name = e.file_name();
        let Some(pid) = name.to_str().filter(|s| s.chars().all(|c| c.is_ascii_digit()))
        else {
            continue;
        };
        if let Ok(comm) = std::fs::read_to_string(format!("/proc/{pid}/comm")) {
            if comm.trim() == "niri-leinwan" || comm.trim() == "niri-leinwand" {
                return true;
            }
        }
    }
    false
}

pub use matrixkit_scene::help_open;

/// Open Matrix Einstellungen (reference settings scene / Ctrl+, — round 12).
/// If the app already runs, the single instance merely focuses it.
pub fn settings_open() {
    let candidates = [
        std::env::var_os("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".local/bin/matrix-settings")),
        Some(std::path::PathBuf::from("/usr/bin/matrix-settings")),
    ];
    let program = candidates
        .into_iter()
        .flatten()
        .find(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("matrix-settings"));
    let _ = mk::process::spawn_reaped(&mut std::process::Command::new(program)); // round 51
}

/// Badge (reference badge, round 12): small number capsule for list
/// rows — "how many articles sit in this category?"
pub fn badge<'a, M: 'a>(count: usize, p: mk::Palette) -> Element<'a, M> {
    container(
        text(count.to_string())
            .size(mk::font_size::SMALL)
            .color(color(p.on_surface_variant)),
    )
    .padding(iced::Padding {
        top: 1.0,
        right: 7.0,
        bottom: 1.0,
        left: 7.0,
    })
    .style(move |_| container::Style {
        background: Some(color(p.on_surface.over(p.surface_container, 0.08)).into()),
        border: iced::Border {
            radius: mk::radius::capsule(20.0).into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// A table column (reference Table/TableColumn, round 12).
pub struct TableColumn {
    pub title: &'static str,
    /// Share of the total width (FillPortion) — 0 = fixed narrow (80px).
    pub portion: u16,
    /// Number columns align right (reference convention).
    pub right: bool,
}

/// Sortable table (reference Table + sortOrder): clickable column heads
/// with a sort arrow, zebra rows. The app holds the sorting as state
/// (column + direction) and sorts its data itself — like the
/// Leitbild-UI-sortOrder-Binding.
pub fn table<'a, M: Clone + 'a>(
    columns: &'a [TableColumn],
    lines: Vec<Vec<String>>,
    sort_column: usize,
    descending: bool,
    on_sort: impl Fn(usize) -> M + 'a,
    p: mk::Palette,
) -> Element<'a, M> {
    let cell_width = |s: &TableColumn| {
        if s.portion == 0 {
            Length::Fixed(80.0)
        } else {
            Length::FillPortion(s.portion)
        }
    };

    // Header row: a click sorts, the arrow shows the active direction
    let mut header = iced::widget::Row::new().spacing(mk::spacing::S);
    for (i, s) in columns.iter().enumerate() {
        let active = i == sort_column;
        let arrow = if !active {
            ""
        } else if descending {
            " ↓"
        } else {
            " ↑"
        };
        let caption = text(format!("{}{}", s.title, arrow))
            .size(mk::font_size::SMALL)
            .color(color(if active { p.primary } else { p.on_surface_variant }));
        let content = container(caption).width(cell_width(s)).align_x(if s.right {
            iced::alignment::Horizontal::Right
        } else {
            iced::alignment::Horizontal::Left
        });
        header = header.push(
            mouse_area(content)
                .interaction(iced::mouse::Interaction::Pointer)
                .on_press(on_sort(i)),
        );
    }

    let mut column = column![
        container(header).padding(iced::Padding {
            top: 4.0,
            right: mk::spacing::S,
            bottom: 4.0,
            left: mk::spacing::S,
        }),
        container(Space::new().width(Length::Fill).height(1.0)).style(move |_| {
            container::Style {
                background: Some(color(p.on_surface.over(p.surface_container, 0.10)).into()),
                ..Default::default()
            }
        }),
    ];

    // Zebra rows (reference alternatingRowBackgrounds)
    for (zi, cell_texts) in lines.into_iter().enumerate() {
        let mut line = iced::widget::Row::new().spacing(mk::spacing::S);
        for (i, value) in cell_texts.into_iter().enumerate() {
            let Some(s) = columns.get(i) else { break };
            line = line.push(
                container(
                    text(value)
                        .size(mk::font_size::SMALL)
                        .color(color(p.on_surface)),
                )
                .width(cell_width(s))
                .align_x(if s.right {
                    iced::alignment::Horizontal::Right
                } else {
                    iced::alignment::Horizontal::Left
                }),
            );
        }
        let striped = zi % 2 == 1;
        column = column.push(
            container(line)
                .padding(iced::Padding {
                    top: 3.0,
                    right: mk::spacing::S,
                    bottom: 3.0,
                    left: mk::spacing::S,
                })
                .style(move |_| container::Style {
                    background: striped
                        .then(|| color(p.on_surface.over(p.surface_container, 0.04)).into()),
                    ..Default::default()
                }),
        );
    }
    column.into()
}

/// Chart kinds (Swift Charts, round 13): LineMark+AreaMark or BarMark.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChartKind {
    Line,
    Bars,
}

/// History chart (Swift Charts): values 0..1, youngest value right.
/// Charts aesthetics: subtle grid lines, baseline (RuleMark), line
/// with an area gradient OR rounded bars — everything from the palette.
/// `capacity` = slots of the time window (the curve grows leftwards).
pub fn chart<'a, M: 'a>(
    values: Vec<f32>,
    capacity: usize,
    art: ChartKind,
    p: mk::Palette,
) -> Element<'a, M> {
    iced::widget::canvas(Chart {
        values,
        capacity: capacity.max(2),
        art,
        p,
        _m: std::marker::PhantomData,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

struct Chart<M> {
    values: Vec<f32>,
    capacity: usize,
    art: ChartKind,
    p: mk::Palette,
    _m: std::marker::PhantomData<M>,
}

impl<M> iced::widget::canvas::Program<M> for Chart<M> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Stroke};
        let mut frame = Frame::new(renderer, bounds.size());
        let (w, h) = (bounds.width, bounds.height);
        let p = self.p;

        // Grid (chartYAxis defaults): three quiet lines at 25/50/75 %.
        let grid = color(p.on_surface.over(p.surface_container, 0.06));
        for portion in [0.25f32, 0.5, 0.75] {
            let y = h - 2.0 - portion * (h - 4.0);
            let mut g = iced::widget::canvas::path::Builder::new();
            g.move_to(iced::Point::new(0.0, y));
            g.line_to(iced::Point::new(w, y));
            frame.stroke(&g.build(), Stroke::default().with_color(grid).with_width(1.0));
        }
        // Baseline (RuleMark at 0).
        let ground = color(p.on_surface.over(p.surface_container, 0.15));
        let mut g = iced::widget::canvas::path::Builder::new();
        g.move_to(iced::Point::new(0.0, h - 2.0));
        g.line_to(iced::Point::new(w, h - 2.0));
        frame.stroke(&g.build(), Stroke::default().with_color(ground).with_width(1.0));

        let n = self.values.len();
        if n >= 1 {
            let step = w / (self.capacity - 1) as f32;
            let x0 = w - step * (n.saturating_sub(1)) as f32;
            let pt = |i: usize| -> iced::Point {
                let v = self.values[i].clamp(0.0, 1.0);
                iced::Point::new(x0 + step * i as f32, h - 2.0 - v * (h - 4.0))
            };
            match self.art {
                ChartKind::Line if n >= 2 => {
                    let mut line_path = iced::widget::canvas::path::Builder::new();
                    line_path.move_to(pt(0));
                    for i in 1..n {
                        line_path.line_to(pt(i));
                    }
                    let mut area = iced::widget::canvas::path::Builder::new();
                    area.move_to(iced::Point::new(x0, h - 2.0));
                    for i in 0..n {
                        area.line_to(pt(i));
                    }
                    area.line_to(iced::Point::new(x0 + step * (n - 1) as f32, h - 2.0));
                    area.close();
                    frame.fill(
                        &area.build(),
                        color(p.primary.over(p.surface_container, 0.15)),
                    );
                    frame.stroke(
                        &line_path.build(),
                        Stroke::default().with_color(color(p.primary)).with_width(1.5),
                    );
                }
                ChartKind::Bars => {
                    let thickness = (step - 2.0).max(1.0);
                    for i in 0..n {
                        let peak = pt(i);
                        frame.fill(
                            &iced::widget::canvas::Path::rounded_rectangle(
                                iced::Point::new(peak.x - thickness / 2.0, peak.y),
                                iced::Size::new(thickness, (h - 2.0 - peak.y).max(0.0)),
                                iced::border::Radius::from(2.0),
                            ),
                            color(p.primary),
                        );
                    }
                }
                _ => {}
            }
        }
        vec![frame.into_geometry()]
    }
}

/// Ring gauge (reference Gauge, round 14): a share 0..1 as a circle
/// segment on a quiet track — countdown, fill level, progress.
/// Starts at the top, runs clockwise (the clock convention).
pub fn ring<'a, M: 'a>(
    portion: f32,
    size: f32,
    tint: mk::Rgba,
    p: mk::Palette,
) -> Element<'a, M> {
    iced::widget::canvas(RingDisplay {
        portion: portion.clamp(0.0, 1.0),
        fill: color(tint),
        track: color(p.on_surface.over(p.surface_container_high, 0.12)),
        _m: std::marker::PhantomData,
    })
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .into()
}

struct RingDisplay<M> {
    portion: f32,
    fill: Color,
    track: Color,
    _m: std::marker::PhantomData<M>,
}

impl<M> iced::widget::canvas::Program<M> for RingDisplay<M> {
    type State = ();
    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{path::Arc, path::Builder, Frame, Stroke};
        let mut frame = Frame::new(renderer, bounds.size());
        let center = iced::Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let width = (bounds.width.min(bounds.height) * 0.103).max(2.5);
        let radius = bounds.width.min(bounds.height) / 2.0 - width;
        let mut full = Builder::new();
        full.circle(center, radius);
        frame.stroke(&full.build(), Stroke::default().with_color(self.track).with_width(width));
        if self.portion > 0.001 {
            let start = -std::f32::consts::FRAC_PI_2;
            let mut arc = Builder::new();
            arc.arc(Arc {
                center,
                radius,
                start_angle: iced::Radians(start),
                end_angle: iced::Radians(start + std::f32::consts::TAU * self.portion),
            });
            frame.stroke(
                &arc.build(),
                Stroke::default().with_color(self.fill).with_width(width),
            );
        }
        vec![frame.into_geometry()]
    }
}

/// Collapsible group (reference DisclosureGroup): clickable header with
/// a chevron that shows/hides its content. For deeper settings levels.
pub fn expand<'a, M: Clone + 'a>(
    title: &'a str,
    open: bool,
    on_toggle: M,
    content: Element<'a, M>,
    p: mk::Palette,
) -> Element<'a, M> {
    let chevron = if open { symbol::EXPAND_MORE } else { symbol::CHEVRON_RIGHT };
    let header = iced::widget::button(
        row![
            txt(title, mk::typo::HEADLINE, p.on_surface),
            Space::new().width(Length::Fill),
            symbol::<M>(chevron, mk::font_size::LARGE, p.on_surface_variant),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([mk::spacing::S as u16, mk::spacing::M as u16])
    .width(Length::Fill)
    .on_press(on_toggle)
    .style(move |_, status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => {
                Some(color(p.on_surface.over(p.surface_container_high, mk::state_layer::HOVER)).into())
            }
            _ => None,
        };
        iced::widget::button::Style {
            background: bg,
            border: iced::Border { radius: mk::radius::SMALL.into(), ..Default::default() },
            ..Default::default()
        }
    });
    let mut column = column![header].spacing(0);
    if open {
        column = column.push(
            container(content).padding(iced::Padding {
                left: mk::spacing::M,
                top: mk::spacing::XS,
                bottom: mk::spacing::XS,
                ..iced::Padding::ZERO
            }),
        );
    }
    column.into()
}

/// Scroll area with a top edge fade (reference scrollEdgeEffect): content
/// visibly runs out under the header instead of ending hard.
pub fn scroll_region_with_fade<'a, M: 'a>(
    content: Element<'a, M>,
    p: mk::Palette,
) -> Element<'a, M> {
    use iced::gradient::Linear;
    use iced::widget::stack;
    // Edge fades TOP and BOTTOM (reference scrollEdgeEffect): clipped rows
    // run out softly instead of looking hard-cut.
    // Angle 0 = bottom to top; stop 0 sits at the BOTTOM end.
    let top = container(Space::new().width(Length::Fill).height(Length::Fixed(18.0))).style(
        move |_| {
            let g = Linear::new(iced::Radians(0.0))
                .add_stop(0.0, color(mk::Rgba { a: 0.0, ..p.surface_container }))
                .add_stop(1.0, color(p.surface_container));
            container::Style {
                background: Some(iced::Background::Gradient(g.into())),
                ..Default::default()
            }
        },
    );
    let bottom = container(Space::new().width(Length::Fill).height(Length::Fixed(18.0))).style(
        move |_| {
            let g = Linear::new(iced::Radians(0.0))
                .add_stop(0.0, color(p.surface_container))
                .add_stop(1.0, color(mk::Rgba { a: 0.0, ..p.surface_container }));
            container::Style {
                background: Some(iced::Background::Gradient(g.into())),
                ..Default::default()
            }
        },
    );
    // Inner padding of fade height: at the scroll stop the content
    // begins/ends OUTSIDE the fade zone — nothing is clipped at rest;
    // only while scrolling do rows run softly under the edges.
    let padded = container(content).padding(iced::Padding {
        top: crate::SCROLL_PAD,
        bottom: crate::SCROLL_PAD,
        ..iced::Padding::ZERO
    });
    stack![
        scroll_region(padded.into(), p),
        container(top)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Top),
        container(bottom)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Bottom),
    ]
    .into()
}

// ---------------------------------------------------------------------------
// The scroll ghost — a Matrix invention (user, 2026-07-07): the
// scrollbar does not stick to the edge of the app but appears NEXT TO
// THE MOUSE as soon as it hovers over a scrollable area. It is pure
// display — "you can scroll here" (affordance) plus "this far along"
// (position + viewport share). Scrolling happens with wheel/trackpad;
// on leave the ghost vanishes, the edge stays free.
// ---------------------------------------------------------------------------

/// Scroll physics (the reference feel): wheel detents do not JUMP, they
/// glide softly to the target (exponential trailing, ~300 ms); trackpad
/// deltas grip 1:1 and coast out with momentum after release
/// (momentum with exponential decay). Lives in the frame; the scroll
/// surface supplies wheel events and viewport position.
#[derive(Debug, Clone)]
pub struct RollPhysics {
    /// Where the surface wants to go (px, downward positive).
    pub target: f32,
    /// Where it currently is.
    pub is: f32,
    /// Maximum offset (content − viewport).
    pub max: f32,
    /// Coasting velocity (px/s, trackpad/finger momentum).
    v: f32,
    /// Momentum decay per second — trackpad (reference) and finger (tablet
    /// reference) coast out differently; the input chooses.
    decay: f32,
    last_input: std::time::Instant,
}

impl RollPhysics {
    /// One wheel detent scrolls three row heights — leisurely like the reference.
    const DETENTS_PX: f32 = 96.0;
    /// Trackpad coast-out (~1.3 s, reference feel).
    const DECAY_TRACKPAD: f32 = 0.045;
    /// Finger coasting: UIScrollViewDecelerationRateNormal = 0.998 per ms
    /// (touch reference SDK extract R59) → 0.998^1000 per second.
    const DECAY_FINGER: f32 = 0.135;

    pub fn new() -> Self {
        Self {
            target: 0.0,
            is: 0.0,
            max: 0.0,
            v: 0.0,
            decay: Self::DECAY_TRACKPAD,
            last_input: std::time::Instant::now() - std::time::Duration::from_secs(60),
        }
    }

    /// Finger drags (swipe surface, R59): the content sticks 1:1 to the
    /// finger — `dy` is the finger movement (downward positive).
    pub fn touch_drag(&mut self, dy: f32) {
        self.last_input = std::time::Instant::now();
        self.is = (self.is - dy).clamp(0.0, self.max);
        self.target = self.is;
        self.v = 0.0;
    }

    /// Finger released: the content keeps rolling at finger velocity
    /// and decays like UIScrollView.
    pub fn touch_end(&mut self, v_finger: f32) {
        if mk::motion_reduced() {
            return;
        }
        self.v = -v_finger;
        self.decay = Self::DECAY_FINGER;
        // No spin-up delay: momentum may grip on the next tick.
        self.last_input = std::time::Instant::now() - std::time::Duration::from_millis(60);
    }

    /// Adopt position/limits from the viewport (at rest the physics follow
    /// reality — window size changes etc. shift nothing).
    pub fn sync(&mut self, vp: iced::widget::scrollable::Viewport) {
        self.max = (vp.content_bounds().height - vp.bounds().height).max(0.0);
        if !self.running() {
            self.is = vp.absolute_offset().y.clamp(0.0, self.max);
            self.target = self.is;
        }
    }

    /// Wheel/trackpad input. Wheel (lines) = soft glide to the target;
    /// trackpad (pixels) = direct + momentum for the coast-out afterwards.
    pub fn input(&mut self, delta: iced::mouse::ScrollDelta) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_input).as_secs_f32().max(0.001);
        self.last_input = now;
        match delta {
            iced::mouse::ScrollDelta::Lines { y, .. } => {
                if mk::motion_reduced() {
                    self.is = (self.is - y * Self::DETENTS_PX).clamp(0.0, self.max);
                    self.target = self.is;
                } else {
                    self.target = (self.target - y * Self::DETENTS_PX).clamp(0.0, self.max);
                }
                self.v = 0.0;
            }
            iced::mouse::ScrollDelta::Pixels { y, .. } => {
                // 1:1 under the fingers (reference trackpad), remember momentum
                self.decay = Self::DECAY_TRACKPAD;
                self.is = (self.is - y).clamp(0.0, self.max);
                self.target = self.is;
                if dt < 0.2 {
                    self.v = 0.75 * self.v + 0.25 * (-y / dt);
                } else {
                    self.v = 0.0;
                }
            }
        }
    }

    /// 60 fps tick: trailing toward the target + momentum coast-out.
    pub fn tick(&mut self, dt: f32) {
        // Momentum: after a brief trackpad standstill it coasts out
        let since = self.last_input.elapsed().as_secs_f32();
        if !mk::motion_reduced() && since > 0.05 && self.v.abs() > 40.0 {
            self.target = (self.target + self.v * dt).clamp(0.0, self.max);
            // exponential decay — decay rate per input kind (R59)
            self.v *= self.decay.powf(dt);
            if self.target <= 0.0 || self.target >= self.max {
                self.v = 0.0;
            }
        } else if since > 0.5 {
            self.v = 0.0;
        }
        // Soft trailing (time constant ~90 ms → calm, never sluggish)
        let k = 1.0 - (-dt / 0.09f32).exp();
        self.is += (self.target - self.is) * k;
        if (self.target - self.is).abs() < 0.4 && self.v.abs() <= 40.0 {
            self.is = self.target;
        }
    }

    /// Must the tick run / a scroll_to go out?
    pub fn running(&self) -> bool {
        (self.target - self.is).abs() >= 0.4 || self.v.abs() > 40.0
    }
}

/// State of the scroll ghost — lives in the frame (every app gets it
/// for free); fade in/out springs gently.
#[derive(Debug, Clone)]
pub struct ScrollGhost {
    /// Mouse position in window coordinates (the frame tracks it).
    pub mouse: iced::Point,
    /// Mouse hovers over the scroll area.
    pub inside: bool,
    /// Scroll position 0..1.
    pub offset: f32,
    /// Viewport share of the content (0..1); ~1 = nothing to scroll.
    pub portion: f32,
    /// Received a real scroll position yet? (Before that the ghost shows
    /// a neutral track — better than no affordance at all.)
    pub known: bool,
    /// Opacity spring (0 = gone, 1 = there) — soft appearing/disappearing.
    pub opacity: mk::motion::Spring,
    /// Rectangle of the scroll area in the window — for the edge flip like
    /// the window ghost (tips to the other side at the edge).
    pub bounds: iced::Rectangle,
}

impl ScrollGhost {
    pub fn new() -> Self {
        let mut g = Self {
            mouse: iced::Point::ORIGIN,
            inside: false,
            offset: 0.0,
            portion: 0.5,
            known: false,
            opacity: mk::motion::Spring::new(0.0),
            bounds: iced::Rectangle { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
        };
        // Dev hook for screenshots: ghost pinned visible.
        if std::env::var("MATRIXKIT_GEIST").is_ok() {
            g.inside = true;
            g.known = true;
            g.portion = 0.45;
            g.offset = 0.35;
            g.mouse = iced::Point::new(210.0, 260.0);
            g.opacity = mk::motion::Spring::new(1.0);
        }
        g
    }

    /// Adopt the scroll position from an iced viewport.
    pub fn scroll(&mut self, vp: iced::widget::scrollable::Viewport) {
        self.offset = vp.relative_offset().y.clamp(0.0, 1.0);
        let (b, c) = (vp.bounds(), vp.content_bounds());
        self.portion = if c.height > 0.0 { (b.height / c.height).clamp(0.05, 1.0) } else { 1.0 };
        self.bounds = b;
        self.known = true;
    }

    pub fn enter(&mut self) {
        self.inside = true;
        if mk::motion_reduced() {
            self.opacity = mk::motion::Spring::new(1.0);
        } else {
            self.opacity.retarget(1.0);
        }
    }

    pub fn leave(&mut self) {
        if std::env::var("MATRIXKIT_GEIST").is_ok() {
            return;
        }
        self.inside = false;
        if mk::motion_reduced() {
            self.opacity = mk::motion::Spring::new(0.0);
        } else {
            self.opacity.retarget(0.0);
        }
    }

    pub fn tick(&mut self) {
        self.opacity.tick(1.0 / 60.0);
    }

    pub fn is_animating(&self) -> bool {
        !self.opacity.is_settled()
    }

    /// Drawn as long as opacity remains (also while fading out) — and
    /// only if there is something to scroll.
    pub fn visible(&self) -> bool {
        self.opacity.value > 0.02 && (!self.known || self.portion < 0.999)
    }

    /// R112 pocket state (NSScrollPocket extract): is content hidden
    /// above / below the viewport? Drives the pocket bands.
    pub fn pockets(&self) -> (bool, bool) {
        if !self.known || self.portion >= 0.999 {
            return (false, false);
        }
        (self.offset > 0.005, self.offset < 0.995)
    }
}

/// Scroll area for the scroll ghost: edge fades as usual, but WITHOUT
/// an edge bar — the position reports via on_scroll, enter/leave via the
/// two messages. The ghost itself is laid over the window via
/// `scrollgeist` (stack, like the context menu).
pub fn scroll_region_ghost<'a, M: Clone + 'a>(
    content: Element<'a, M>,
    id: iced::advanced::widget::Id,
    on_scroll: impl Fn(iced::widget::scrollable::Viewport) -> M + 'a,
    on_wheel: impl Fn(iced::mouse::ScrollDelta) -> M + 'a,
    on_enter: M,
    on_leave: M,
    pockets: (bool, bool),
    p: mk::Palette,
) -> Element<'a, M> {
    use iced::gradient::Linear;
    use iced::widget::stack;
    // R112 pocket (NSScrollPocket extract): the band only appears when
    // content is actually hidden in that direction — gradient into the
    // surface tone plus a hairline at the inner edge, so content slides
    // "into the pocket" instead of being cut off.
    let pocket_band = move |up: bool, active: bool| -> Element<'a, M> {
        if !active {
            return Space::new().width(Length::Fill).height(Length::Fixed(0.0)).into();
        }
        let grad = container(Space::new().width(Length::Fill).height(Length::Fixed(17.0)))
            .style(move |_| {
                let (from, to) = if up {
                    (color(mk::Rgba { a: 0.0, ..p.surface_container }), color(p.surface_container))
                } else {
                    (color(p.surface_container), color(mk::Rgba { a: 0.0, ..p.surface_container }))
                };
                let g = Linear::new(iced::Radians(0.0)).add_stop(0.0, from).add_stop(1.0, to);
                container::Style {
                    background: Some(iced::Background::Gradient(g.into())),
                    ..Default::default()
                }
            });
        let hairline = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(move |_| container::Style {
                background: Some(color(mk::Rgba { a: 0.18, ..p.on_surface_variant }).into()),
                ..Default::default()
            });
        if up {
            column![grad, hairline].into()
        } else {
            column![hairline, grad].into()
        }
    };
    let top = pocket_band(true, pockets.0);
    let bottom = pocket_band(false, pockets.1);
    let padded = container(content).padding(iced::Padding {
        top: 18.0,
        bottom: 18.0,
        ..iced::Padding::ZERO
    });
    // Edge bar to zero — the ghost takes over the display.
    let scroll_area = iced::widget::scrollable(padded)
        .id(id)
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::new().width(0).scroller_width(0).margin(0),
        ))
        .on_scroll(on_scroll)
        .width(Length::Fill)
        .height(Length::Fill);
    let area = stack![
        scroll_area,
        container(top)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Top),
        container(bottom)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(iced::alignment::Vertical::Bottom),
    ];
    // on_scroll captures the wheel EXCLUSIVELY (capture_event) — the
    // physics in the frame take over and drive the surface via scroll_to.
    mouse_area(area)
        .on_enter(on_enter)
        .on_exit(on_leave)
        .on_scroll(on_wheel)
        .into()
}

/// The floating display of the scroll ghost: a narrow track right of the
/// cursor, thumb in accent color (length = viewport share, position =
/// scroll position). Lay it over the window as the topmost stack layer.
pub fn scroll_ghost<'a, M: 'a>(g: &ScrollGhost, p: mk::Palette) -> Element<'a, M> {
    if !g.visible() {
        return Space::new().into();
    }
    const HEIGHT: f32 = 56.0;
    const WIDTH: f32 = 6.0;
    let thumb_h = (HEIGHT * g.portion).clamp(12.0, HEIGHT);
    let thumb_y = (HEIGHT - thumb_h) * g.offset.clamp(0.0, 1.0);
    // Spring opacity: the ghost fades softly in and out.
    let a = g.opacity.value.clamp(0.0, 1.0);

    let thumb = container(Space::new().width(Length::Fixed(WIDTH)).height(Length::Fixed(thumb_h)))
        .style(move |_| container::Style {
            background: Some(color(mk::Rgba { a: p.primary.a * a, ..p.primary }).into()),
            border: iced::Border { radius: mk::radius::capsule(WIDTH).into(), ..Default::default() },
            ..Default::default()
        });
    let track_color = p.on_surface.over(p.surface_container, 0.22);
    let track = container(column![Space::new().height(Length::Fixed(thumb_y)), thumb].spacing(0))
        .width(Length::Fixed(WIDTH))
        .height(Length::Fixed(HEIGHT))
        .style(move |_| container::Style {
            background: Some(color(mk::Rgba { a: track_color.a * a, ..track_color }).into()),
            border: iced::Border { radius: mk::radius::capsule(WIDTH).into(), ..Default::default() },
            ..Default::default()
        });

    // Edge flip like the window ghost: right-below the cursor (mouse+26);
    // at the right edge flip left, at the bottom edge clamp up.
    const MARGIN: f32 = 8.0;
    let right = if g.bounds.width > 0.0 { g.bounds.x + g.bounds.width } else { g.mouse.x + 1e6 };
    let bottom = if g.bounds.height > 0.0 { g.bounds.y + g.bounds.height } else { g.mouse.y + 1e6 };
    let mut left = g.mouse.x + 26.0;
    if left + WIDTH > right - MARGIN {
        left = (g.mouse.x - 26.0 - WIDTH).max(0.0);
    }
    let mut top = g.mouse.y + 26.0;
    if top + HEIGHT > bottom - MARGIN {
        top = (bottom - HEIGHT - MARGIN).max(0.0);
    }

    container(track)
        .padding(iced::Padding {
            left: left.max(0.0),
            top: top.max(0.0),
            ..iced::Padding::ZERO
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_lock_state_machine() {
        let mut r = RootState::new();
        // Fresh: closed, invisible, locked
        assert!(!r.is_open());
        assert!(!r.visible());
        assert!(!r.unlocked);

        // Open: visible but still locked (reference lock)
        r.toggle();
        assert!(r.is_open());
        assert!(r.visible());
        assert!(!r.unlocked, "Root öffnet IMMER verriegelt");

        // Wrong password: failed attempt, still locked
        r.check_running = true;
        r.unlock_result(false);
        assert!(!r.unlocked);
        assert!(r.failed_attempt);
        assert!(!r.check_running);

        // Right password: unlocked, password cleared
        r.password = "geheim".into();
        r.unlock_result(true);
        assert!(r.unlocked);
        assert!(r.password.is_empty(), "Passwort wird nach Erfolg gelöscht");

        // Closing locks again
        r.toggle();
        assert!(!r.is_open());
        assert!(!r.unlocked, "Schließen verriegelt erneut");
    }

    #[test]
    fn dialog_state_machine() {
        let mut d = DialogState::new();
        assert!(!d.is_open());
        assert!(!d.visible());
        d.open();
        assert!(d.is_open());
        assert!(d.visible());
        d.close();
        assert!(!d.is_open());
        // Still visible during the fade-out spring (overlay must render)
        assert!(d.visible() || !d.is_animating());
    }

    #[test]
    fn redaction_is_empty_placeholder() {
        // No crash at 0 sizes; produces an element.
        let p = mk::Palette::default();
        let _: Element<'_, ()> = redaction(0.0, 0.0, p);
        let _: Element<'_, ()> = redaction(120.0, 14.0, p);
    }
}

/// The niri bridge — ONE source of compositor knowledge for the bar
/// family (dock, bar): window list, focus, focusing. All speak `niri msg`,
/// nobody parses by hand.
pub mod canvas {
    use matrixkit_theme as mk;

    /// An open window as the compositor sees it.
    #[derive(Debug, Clone)]
    pub struct OpenWindow {
        pub id: u64,
        pub app_id: String,
        pub title: String,
        pub focus: bool,
        /// Position in the workspace view (for docking couplings like Matrix Web).
        pub pos: Option<(f64, f64)>,
        pub size: Option<(f64, f64)>,
    }

    fn json(args: &[&str]) -> Option<serde_json::Value> {
        let out = mk::canvas::raw(args)?;
        out.status.success().then(|| ())?;
        serde_json::from_slice(&out.stdout).ok()
    }

    fn window_from(w: &serde_json::Value) -> OpenWindow {
        let paar = |v: &serde_json::Value| {
            Some((v.get(0)?.as_f64()?, v.get(1)?.as_f64()?))
        };
        OpenWindow {
            id: w["id"].as_u64().unwrap_or(0),
            app_id: w["app_id"].as_str().unwrap_or("?").to_string(),
            title: w["title"].as_str().unwrap_or("").to_string(),
            focus: w["is_focused"].as_bool().unwrap_or(false),
            pos: paar(&w["layout"]["tile_pos_in_workspace_view"]),
            size: paar(&w["layout"]["tile_size"]),
        }
    }

    /// All open windows; an error means an empty list, never a crash.
    pub fn windows() -> Vec<OpenWindow> {
        json(&["msg", "--json", "windows"])
            .and_then(|v| v.as_array().map(|a| a.iter().map(window_from).collect()))
            .unwrap_or_default()
    }

    /// The focused window, if any holds focus.
    pub fn focus() -> Option<OpenWindow> {
        let v = json(&["msg", "--json", "focused-window"])?;
        v.is_object().then(|| window_from(&v))
    }

    /// Bring a window into focus (the canvas drives it fully into view).
    /// Focus a window — fetched from the shelf first if minimized
    /// (`mk::canvas::window_bring`, R209), so the shelf never shows.
    pub fn focus_window(id: u64) {
        mk::canvas::window_bring(id);
    }
}

/// The bar family (bar, dock): shared surface grammar.
/// Bars are layer-shell surfaces in the top layer (bottom starves in the
/// canvas session — finding of task #39), live from the live palette and
/// take no keyboard.
pub mod panel {
    use super::*;

    /// PANEL TARGET (R209): which screen a shell surface belongs on. On a
    /// duo device the settings key `setting` (or `duo_default`) names the
    /// panel; elsewhere the compositor's choice stands.
    pub fn target(setting: &str, duo_default: &str) -> crate::iced_layershell::settings::StartMode {
        if mk::device::is_duo() {
            crate::iced_layershell::settings::StartMode::TargetScreen(mk::device::duo_panel(setting, duo_default))
        } else {
            crate::iced_layershell::settings::StartMode::default()
        }
    }

    /// The same choice for a DAEMON's surfaces (NewLayerShell, dock/bar
    /// pattern R174): on a duo the panel named in `setting`, else
    /// `duo_default`; elsewhere the compositor decides (as before).
    pub fn output_option(setting: &str, duo_default: &str) -> crate::iced_layershell::reexport::OutputOption {
        if mk::device::is_duo() {
            crate::iced_layershell::reexport::OutputOption::OutputName(mk::device::duo_panel(setting, duo_default))
        } else {
            crate::iced_layershell::reexport::OutputOption::None
        }
    }

    /// CLAIM THE KEYBOARD (R209 family): a popup that types (search) on a
    /// finger device — summon the on-screen keyboard (touch word, beats the
    /// physical-keyboard veto), let this surface's fields keep it alive
    /// (an overlay is no `Frame`, so it must say it is the active one), and
    /// on a device with several panels make the popup's panel the focused
    /// one: an exclusive keyboard surface gets keys only there, and a tap on
    /// a shell surface no longer moves that focus (niri 0010). `setting` /
    /// `duo_default` name the popup's panel like `target`. Pair it with
    /// `keyboard::heartbeat()` while the popup is open.
    pub fn claim_keyboard(setting: &str, duo_default: &str) {
        if !mk::device::finger_first() {
            return;
        }
        mk::onscreen_keyboard::claim(setting, duo_default);
        crate::keyboard::window_active(true);
    }

    /// The value of `--output NAME` — a surface pinned to one screen.
    pub fn output_arg() -> Option<String> {
        let args: Vec<String> = std::env::args().collect();
        args.iter().position(|a| a == "--output").and_then(|i| args.get(i + 1).cloned())
    }

    /// PER PANEL (R209): this program once per panel (`--output <panel>`,
    /// remaining arguments passed on); waits for all, never returns. For
    /// surfaces that belong on EVERY screen (wallpaper).
    pub fn per_panel(panels: &[&str]) -> ! {
        let me = mk::process::own_exe();
        let rest: Vec<String> = std::env::args().skip(1).collect();
        let mut children = Vec::new();
        for panel in panels {
            let mut command = std::process::Command::new(&me);
        command.args(["--output", panel]).args(&rest);
        // A painter must not outlive its conductor (process::die_with_parent).
        unsafe {
            use std::os::unix::process::CommandExt;
            command.pre_exec(|| {
                crate::process::die_with_parent();
                Ok(())
            });
        }
        match command.spawn() {
                Ok(child) => children.push(child),
                Err(e) => eprintln!("[panel] {panel}: start failed: {e}"),
            }
        }
        for mut child in children {
            let _ = child.wait();
        }
        std::process::exit(0);
    }

    /// Opacity of the bar surfaces — translucent enough for the canvas to
    /// breathe, dense enough for readability.
    pub const COVERAGE: f32 = 0.94;

    /// Living opacity: `transparency = reduced` (accessibility aid,
    /// accessibilityReduceTransparency extract) makes surfaces opaque.
    pub fn coverage() -> f32 {
        if mk::transparency_reduced() {
            1.0
        } else {
            COVERAGE
        }
    }
    /// Hairline opacity of the outline.
    pub const OUTLINE: f32 = 0.25;

    fn area(p: mk::Palette, radius: f32) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(color(p.surface_container.with_alpha(coverage())).into()),
            border: iced::Border {
                color: color(p.outline.with_alpha(OUTLINE)),
                width: 1.0,
                radius: radius.into(),
            },
            // Own shadow WITH breathing room: the bar surfaces are larger than
            // the pill by SHADOW_MARGIN (user findings 2026-07-08:
            // first clipped corners, then compositor shadows around the
            // rectangular surface — both wrong; this way is right).
            // R55b: the TIGHT base layer of the triple shadow
            // (schatten_schichten stacks two more above it) — iced's
            // single-layer shadow falls off almost linearly and feels abrupt
            // (user: "much too abrupt"); only the sum of three layers
            // approaches the gauss of the window shadows.
            shadow: iced::Shadow {
                color: iced::Color { a: 0.32, ..iced::Color::BLACK },
                offset: iced::Vector::new(0.0, 3.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        }
    }

    /// The continuous band (bar): full edge, no rounding.
    pub fn band(p: mk::Palette) -> iced::widget::container::Style {
        area(p, 0.0)
    }

    /// The floating pill (dock): radius concentric to the content —
    /// `inner radius` of the largest child + its distance to the edge.
    pub fn pill(p: mk::Palette, inner_radius: f32, inset: f32) -> iced::widget::container::Style {
        area(p, inner_radius + inset)
    }

    /// R106 liquid glass: the pill outline WITHOUT a fill — the glass
    /// shader behind it paints refraction, tint and rim light.
    pub fn glass_pill(p: mk::Palette, inner_radius: f32, inset: f32) -> iced::widget::container::Style {
        // R108 shape family, completed: under compositor glass the app
        // contributes NOTHING to the silhouette — no fill, no shadow and
        // no outline either (user finding: the 1 px stroke beside the
        // field's own edge read as two layers at the corners). The edge
        // sheen comes from the rim light in the field, nowhere else.
        let _ = (p, inner_radius, inset);
        iced::widget::container::Style::default()
    }

    /// Triple shadow (R55b): two additional shells around a pill —
    /// medium and wide, with sinking alpha. Together with the tight base
    /// layer of the surface this creates the soft gauss fade of the window
    /// shadows that iced's single shadow cannot do.
    pub fn shadow_layers<'a, M: 'a>(
        content: iced::Element<'a, M, iced::Theme, iced::Renderer>,
        radius: f32,
    ) -> iced::Element<'a, M, iced::Theme, iced::Renderer> {
        let hull = |el: iced::Element<'a, M, iced::Theme, iced::Renderer>,
                      blur: f32,
                      alpha: f32,
                      oy: f32| {
            iced::widget::container(el)
                .style(move |_| iced::widget::container::Style {
                    border: iced::Border { radius: radius.into(), ..Default::default() },
                    shadow: iced::Shadow {
                        color: iced::Color { a: alpha, ..iced::Color::BLACK },
                        offset: iced::Vector::new(0.0, oy),
                        blur_radius: blur,
                    },
                    ..Default::default()
                })
                .into()
        };
        hull(hull(content, 26.0, 0.20, 6.0), 46.0, 0.10, 9.0)
    }

    /// Breathing room for the pill shadow: surfaces are larger than the
    /// content by this margin so the 18 px blur is never clipped.
    pub const SHADOW_MARGIN: f32 = 40.0;

    /// The ONE hover tone of the bar family (share of on_surface over
    /// surface_container) — for buttons, menu entries, selection rows.
    pub const HOVER: f32 = 0.10;

    /// Start a Matrix app: first the dev state in ~/.local/bin, then the
    /// system (PATH). The child is REAPED in the background — otherwise
    /// long-lived bars collect zombies whose comm later fools every
    /// leiste_toggle (launcher bug of 2026-07-07).
    pub fn app_start(name: &str) {
        app_start_with(name, &[]);
    }

    /// Like `app_starten`, with arguments (R69: "matrix-capture film-stopp").
    pub fn app_start_with(name: &str, args: &[&str]) {
        // One rule for the whole kit (round 27): mk::process::app_start.
        mk::process::app_start(name, args);
    }

    /// R103 popup grammar — ONE drop position for every bar popup:
    /// directly below the bar, right-aligned (user finding 2026-07-21:
    /// four widgets opened at four different places). Popups run as
    /// fullscreen overlays that leave the bar strip free; the content
    /// sits at these offsets inside the overlay.
    /// Air between the bar pill and a popup — the SAME breath as
    /// between the dock pill and its menus (user calibration
    /// 2026-07-21: the gaps differed per popup).
    pub const POPUP_TOP: f32 = 8.0;
    /// Right offset = the bar's side gap, so popup and pill edges align.
    pub const POPUP_RIGHT: f32 = 10.0;
    /// The bar strip the popup overlay must NOT cover — top margin (10)
    /// plus pill height (44); clicks on the bar keep working, so one
    /// click switches between popups.
    pub const BAR_STRIP: i32 = 54;

    /// R103 mutual exclusion: opening one bar popup closes the others.
    /// Pure-UI popups end by process; the notification daemon (it owns
    /// the DBus service and the history) only folds its panel.
    pub fn close_other_popups(keep: &str) {
        for name in ["matrix-central", "matrix-start"] {
            if name != keep {
                process_end_by_comm(name);
            }
        }
        if keep != "matrix-capture" {
            process_end_by_cmdline("matrix-capture", "panel");
        }
        if keep != "matrix-notifications" {
            let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
            let _ = std::fs::write(format!("{dir}/matrix-notifications-hide"), "1");
        }
    }

    /// End every live process with exactly this comm (not ourselves).
    fn process_end_by_comm(name: &str) {
        let me = std::process::id();
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for e in entries.flatten() {
                if let Ok(pid) = e.file_name().to_string_lossy().parse::<u32>() {
                    if pid != me && super::process_is_alive(pid, name) {
                        let _ = std::process::Command::new("kill").arg(pid.to_string()).status();
                    }
                }
            }
        }
    }

    /// End processes whose argv0 basename matches AND whose first
    /// argument is exactly `arg` (the capture panel — never a running
    /// recording process of the same binary).
    fn process_end_by_cmdline(bin: &str, arg: &str) {
        let me = std::process::id();
        let Ok(entries) = std::fs::read_dir("/proc") else { return };
        for e in entries.flatten() {
            let Ok(pid) = e.file_name().to_string_lossy().parse::<u32>() else { continue };
            if pid == me {
                continue;
            }
            let Ok(raw) = std::fs::read(format!("/proc/{pid}/cmdline")) else { continue };
            let parts: Vec<&str> = raw
                .split(|b| *b == 0)
                .filter_map(|s| std::str::from_utf8(s).ok())
                .filter(|s| !s.is_empty())
                .collect();
            let argv0 = parts.first().map(|p| p.rsplit('/').next().unwrap_or(p));
            if argv0 == Some(bin) && parts.get(1).copied() == Some(arg) {
                let _ = std::process::Command::new("kill").arg(pid.to_string()).status();
            }
        }
    }

    /// R103 dismiss backdrop: the transparent click-catcher behind a
    /// popup — one press anywhere outside closes it. Wrap the popup
    /// content: `stack![backdrop(msg), content]`.
    pub fn backdrop<'a, M: Clone + 'a>(on_close: M) -> Element<'a, M> {
        iced::widget::mouse_area(
            iced::widget::container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(on_close)
        .into()
    }

    /// Button style of the bar family: calm-transparent, hover tone,
    /// rounded. Radius per element (SMALL for row buttons,
    /// LARGE for dock tiles).
    pub fn button_style(
        p: mk::Palette,
        status: iced::widget::button::Status,
        radius: f32,
    ) -> iced::widget::button::Style {
        use iced::widget::button::{Status, Style};
        let bg = match status {
            Status::Hovered | Status::Pressed => {
                Some(color(p.on_surface.over(p.surface_container, HOVER)).into())
            }
            _ => None,
        };
        Style {
            background: bg,
            border: iced::border::rounded(radius),
            ..Default::default()
        }
    }
}

/// Panel toggle of the bar family: the same call opens and closes.
/// true = no other instance ran, we are the panel (keep running);
/// false = the running instance was ended — the caller exits quietly.
/// (Layer surfaces have no window to focus — unlike
/// mk::window::einzelinstanz, closing is the desired echo here.)
pub fn panel_toggle() -> bool {
    let me = std::process::id();
    let name = std::fs::read_to_string(format!("/proc/{me}/comm")).unwrap_or_default();
    let name = name.trim().to_string();
    if name.is_empty() {
        return true;
    }
    let mut other = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for e in entries.flatten() {
            if let Ok(pid) = e.file_name().to_string_lossy().parse::<u32>() {
                if pid != me && process_is_alive(pid, &name) {
                    other.push(pid);
                }
            }
        }
    }
    if other.is_empty() {
        return true;
    }
    for pid in other {
        let _ = std::process::Command::new("kill").arg(pid.to_string()).status();
    }
    false
}

/// Round 76: single instance for a DAEMON (not a popup): if another copy
/// lives, wait up to 3 s — a restart kills the old one just before — and
/// step back only if it is still there. `panel_toggle` instead killed the
/// dying old copy AND left itself (a toggle), so a quick restart left no
/// daemon at all (the notifications, three times on 03./04.11.).
pub fn daemon_single() -> bool {
    // Round 103: an exec'ed image (panel change) reaps what the last one left.
    matrixkit_theme::process::adopt_strays();
    let me = std::process::id();
    let name = std::fs::read_to_string(format!("/proc/{me}/comm")).unwrap_or_default();
    let name = name.trim().to_string();
    if name.is_empty() {
        return true;
    }
    // Round 80: only an OLDER copy counts (start time, then pid). Two
    // copies started at once (the watchdog and a manual restart, 04.11.)
    // saw each other and BOTH stepped back — or, without a guard, both ran.
    let started = |pid: u32| -> Option<u64> {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        stat.rsplit(')').next()?.split_whitespace().nth(19)?.parse().ok()
    };
    let mine = (started(me).unwrap_or(u64::MAX), me);
    let others = || -> bool {
        std::fs::read_dir("/proc").map_or(false, |entries| {
            entries.flatten().any(|e| {
                e.file_name().to_string_lossy().parse::<u32>().is_ok_and(|pid| {
                    pid != me && process_is_alive(pid, &name) && (started(pid).unwrap_or(0), pid) < mine
                })
            })
        })
    };
    for _ in 0..30 {
        if !others() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    false
}

/// Same name AND truly alive — zombies (`<defunct>`) keep their comm
/// and have fooled a launcher toggle before.
fn process_is_alive(pid: u32, name: &str) -> bool {
    let Ok(c) = std::fs::read_to_string(format!("/proc/{pid}/comm")) else {
        return false;
    };
    if c.trim() != name {
        return false;
    }
    // /proc/PID/stat: "pid (comm) STATE …" — comm may contain ')',
    // therefore read behind the LAST parenthesis.
    std::fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()
        .and_then(|s| {
            s.rsplit(')')
                .next()
                .and_then(|rest| rest.split_whitespace().next())
                .map(|state| state != "Z")
        })
        .unwrap_or(false)
}

/// Slider in MatrixKit style: track and grip from the palette,
/// never framework blue. Width/on_release is attached by the caller.
pub fn bar_slider<'a, M: Clone + 'a>(
    area: std::ops::RangeInclusive<f32>,
    value: f32,
    step: f32,
    p: mk::Palette,
    on_change: impl Fn(f32) -> M + 'a,
) -> iced::widget::Slider<'a, f32, M> {
    iced::widget::slider(area, value, on_change)
        .step(step)
        .style(move |_theme, status| {
            use iced::widget::slider::{Handle, HandleShape, Rail, Status, Style};
            let grip = match status {
                Status::Dragged => p.primary.over(p.on_surface, 0.15),
                Status::Hovered => p.primary.over(p.on_surface, 0.08),
                _ => p.primary,
            };
            Style {
                rail: Rail {
                    backgrounds: (
                        color(p.primary).into(),
                        color(p.on_surface.over(p.surface_container, 0.12)).into(),
                    ),
                    width: 4.0,
                    border: iced::border::rounded(mk::radius::capsule(4.0)),
                },
                handle: Handle {
                    shape: HandleShape::Circle { radius: 8.0 },
                    background: color(grip).into(),
                    border_width: 0.0,
                    border_color: iced::Color::TRANSPARENT,
                },
            }
        })
}

/// OSD building blocks of the bar family (Dynamic Dock, ch. 10/11):
/// ONE symbol mapping and ONE level bar — whoever shows the OSD channel
/// (mk::osd) renders it with these.
pub mod osd_display {
    use super::*;

    /// Symbol + "full" flag (false = dimmed rendering when muted).
    pub fn glyph(state: mk::osd::State) -> (char, bool) {
        match (state.kind, state.muted) {
            (mk::osd::Kind::Sound, false) => (symbol::VOLUME_UP, true),
            (mk::osd::Kind::Sound, true) => (symbol::VOLUME_OFF, false),
            (mk::osd::Kind::Mic, false) => (symbol::MIC, true),
            (mk::osd::Kind::Mic, true) => (symbol::MIC_OFF, false),
            (mk::osd::Kind::Light, _) => (symbol::BRIGHTNESS, true),
        }
    }

    /// The level bar: slider color world without a grip (track + fill),
    /// capsule rounding. `voll = false` dims the fill (muted).
    pub fn level_bar<'a, M: 'a>(
        p: mk::Palette,
        portion: f32,
        full: bool,
        width: f32,
    ) -> Element<'a, M> {
        let portion = portion.clamp(0.0, 1.0);
        let fill = if full {
            p.primary
        } else {
            p.on_surface_variant.with_alpha(0.6)
        };
        container(
            container(Space::new())
                .width(Length::Fixed(width * portion))
                .height(Length::Fixed(8.0))
                .style(move |_| container::Style {
                    background: Some(color(fill).into()),
                    border: iced::border::rounded(mk::radius::capsule(8.0)),
                    ..Default::default()
                }),
        )
        .width(Length::Fixed(width))
        .style(move |_| container::Style {
            background: Some(color(p.on_surface.over(p.surface_container, 0.12)).into()),
            border: iced::border::rounded(mk::radius::capsule(8.0)),
            ..Default::default()
        })
        .into()
    }
}

/// The niri event stream: ONE `niri msg event-stream` process per app,
/// every burst of events becomes exactly ONE ping (100 ms throttle).
/// Bars thus react IMMEDIATELY to window/focus changes instead of
/// polling; a rare poll remains as fallback good manners. If the stream
/// tears (compositor restart), it reconnects itself.
pub fn canvas_stream() -> iced::Subscription<()> {
    use iced::futures::{SinkExt, StreamExt};
    iced::Subscription::run(|| {
        iced::stream::channel(64, |mut out: iced::futures::channel::mpsc::Sender<()>| async move {
            let (tx, mut rx) = iced::futures::channel::mpsc::unbounded::<()>();
            // Round 50: the kit's event follow (reconnect, kill AND reap).
            mk::canvas::follow_events(false, move |line| {
                if line.is_none() {
                    return !tx.is_closed();
                }
                if tx.unbounded_send(()).is_err() {
                    return false; // app is gone — the stream goes with it
                }
                // Burst throttle: follow-up events of the next 100 ms
                // bundle up in the buffer.
                std::thread::sleep(std::time::Duration::from_millis(100));
                true
            });
            while let Some(()) = rx.next().await {
                // Drain the buffer: one burst = one ping.
                #[allow(deprecated)]
                while rx.try_next().ok().flatten().is_some() {}
                let _ = out.send(()).await;
            }
        })
    })
}

/// The reference `Label` extract (reference round 17): symbol + text are
/// ONE named pair with fixed grammar (spacing, baseline) — no more
/// ad-hoc assembled rows in bars, menus and rows.
pub fn tag<'a, M: 'a>(
    glyph: char,
    content: impl iced::widget::text::IntoFragment<'a>,
    style: mk::typo::TextStyle,
    tint: mk::Rgba,
) -> Element<'a, M> {
    row![
        // R65b: the symbol carries the WEIGHT of its text (SF Symbols).
        symbol_weight(glyph, style.size + 3.0, tint, style.weight),
        txt(content, style, tint),
    ]
    .spacing(mk::spacing::XS)
    .align_y(iced::Alignment::Center)
    .into()
}

/// The reference `Divider` extract: the ONE quiet separator.
pub fn separator<'a, M: 'a>(p: mk::Palette) -> Element<'a, M> {
    // R103f (user finding): the padding used to sit INSIDE the tinted
    // container — the background filled line + air into a 5 px band.
    // Tint only the 1 px line itself (like the Datenträger hairline),
    // the air stays transparent.
    container(
        container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
            move |_| container::Style {
                background: Some(color(p.outline.with_alpha(0.35)).into()),
                ..Default::default()
            },
        ),
    )
    .padding(iced::Padding {
        left: mk::spacing::S,
        right: mk::spacing::S,
        top: 2.0,
        bottom: 2.0,
    })
    .into()
}

/// The `TimelineView(.everyMinute)` extract: one tick EXACTLY on the
/// full minute, then per minute — clocks never show an expired minute
/// and nobody polls every second.
pub fn tick_to_the_minute() -> iced::Subscription<()> {
    use iced::futures::{SinkExt, StreamExt};
    iced::Subscription::run(|| {
        iced::stream::channel(4, |mut out: iced::futures::channel::mpsc::Sender<()>| async move {
            let (tx, mut rx) = iced::futures::channel::mpsc::unbounded::<()>();
            std::thread::spawn(move || loop {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let to_the_minute = 60 - (now % 60);
                std::thread::sleep(std::time::Duration::from_secs(to_the_minute.max(1)));
                if tx.unbounded_send(()).is_err() {
                    return;
                }
            });
            while let Some(()) = rx.next().await {
                let _ = out.send(()).await;
            }
        })
    })
}

// ===========================================================================
// The magnifier — kit-wide hover magnification (user, 2026-07-08).
// A wrapper widget: its content SCALES up springily on hover
// (Renderer::with_transformation, around the center) WITHOUT shifting
// the layout — the click area stays exact, only the look "pops".
// bewegung=reduziert switches it off hard. Everything clickable can
// wrap itself in it: mkw::zoom(button(...)).
// ===========================================================================

// ===========================================================================
// The island (R121) — the Dynamic-Island appearance extract: content
// POPS into place with a springy scale morph (0.86 → 1, clear
// overshoot) when it appears or changes identity, instead of hard
// swapping. Wrap any element: mkw::island(content, key) — a new key
// replays the morph (kind changes yes, value steps no).
// ===========================================================================

pub fn island<'a, M: 'a>(content: impl Into<Element<'a, M>>, key: u64) -> Element<'a, M> {
    Island { content: content.into(), key }.into()
}

struct Island<'a, M> {
    content: Element<'a, M>,
    key: u64,
}

struct IslandState {
    key: u64,
    scale: mk::motion::Spring,
    last: Option<std::time::Instant>,
}

impl IslandState {
    fn fresh(key: u64) -> Self {
        let mut scale = mk::motion::Spring::with(0.86, 0.35, 0.3);
        scale.retarget(1.0);
        Self { key, scale, last: None }
    }
}

impl<'a, M> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for Island<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<IslandState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(IslandState::fresh(self.key))
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        let z = tree.state.downcast_mut::<IslandState>();
        if z.key != self.key {
            // New identity — replay the pop (reduced motion snaps).
            *z = IslandState::fresh(self.key);
            if mk::motion_reduced() {
                z.scale = mk::motion::Spring::new(1.0);
            } else {
                shell.request_redraw();
            }
        }
        if let iced::Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            let dt = z
                .last
                .map(|l| now.duration_since(l).as_secs_f32())
                .unwrap_or(0.0)
                .clamp(0.0, 0.05);
            z.last = Some(*now);
            if z.scale.tick(dt) {
                shell.request_redraw();
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let z = tree.state.downcast_ref::<IslandState>();
        let s = z.scale.value;
        let draw = |r: &mut iced::Renderer| {
            self.content
                .as_widget()
                .draw(&tree.children[0], r, theme, style, layout, cursor, viewport);
        };
        if (s - 1.0).abs() <= 0.001 {
            draw(renderer);
        } else {
            let b = layout.bounds();
            let (cx, cy) = (b.center_x(), b.center_y());
            let t = iced::Transformation::translate((1.0 - s) * cx, (1.0 - s) * cy)
                * iced::Transformation::scale(s);
            renderer.with_transformation(t, draw);
        }
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, M: 'a> From<Island<'a, M>> for Element<'a, M> {
    fn from(w: Island<'a, M>) -> Self {
        Self::new(w)
    }
}

// ===========================================================================
// The morph (R124) — the missing transition piece for edit flows:
// TWO children occupy the same spot, and a snappy spring (reference
// preset, measured live 23.07.2026: response 0.5, damping 0.85)
// scale-swaps between them when `second` flips. The outgoing child
// shrinks to nothing while the incoming one springs up — a toggle
// morphs into a reorder handle instead of hard-swapping. Events and
// cursor go ONLY to the active child; reduced motion snaps.
// ===========================================================================

pub fn morph<'a, M: 'a>(
    first: impl Into<Element<'a, M>>,
    second: impl Into<Element<'a, M>>,
    second_active: bool,
) -> Element<'a, M> {
    Morph { children: [first.into(), second.into()], second: second_active }.into()
}

struct Morph<'a, M> {
    children: [Element<'a, M>; 2],
    second: bool,
}

struct MorphState {
    second: bool,
    /// 0 = first child fully present, 1 = second child.
    t: mk::motion::Spring,
    last: Option<std::time::Instant>,
}

impl MorphState {
    fn fresh(second: bool) -> Self {
        Self {
            second,
            t: mk::motion::Spring::snappy(if second { 1.0 } else { 0.0 }),
            last: None,
        }
    }
}

impl<'a, M> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for Morph<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<MorphState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(MorphState::fresh(self.second))
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.children.iter().map(iced::advanced::widget::Tree::new).collect()
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(&self.children);
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.children[self.second as usize].as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let [a, b] = &mut self.children;
        let na = a.as_widget_mut().layout(&mut tree.children[0], renderer, limits);
        let nb = b.as_widget_mut().layout(&mut tree.children[1], renderer, limits);
        let size = iced::Size::new(
            na.size().width.max(nb.size().width),
            na.size().height.max(nb.size().height),
        );
        let center = |n: iced::advanced::layout::Node| {
            let s = n.size();
            n.move_to(iced::Point::new(
                (size.width - s.width) / 2.0,
                (size.height - s.height) / 2.0,
            ))
        };
        iced::advanced::layout::Node::with_children(size, vec![center(na), center(nb)])
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        let z = tree.state.downcast_mut::<MorphState>();
        if z.second != self.second {
            z.second = self.second;
            let target = if self.second { 1.0 } else { 0.0 };
            if mk::motion_reduced() {
                z.t = mk::motion::Spring::snappy(target);
            } else {
                z.t.retarget(target);
                shell.request_redraw();
            }
        }
        if let iced::Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            let dt = z
                .last
                .map(|l| now.duration_since(l).as_secs_f32())
                .unwrap_or(0.0)
                .clamp(0.0, 0.05);
            z.last = Some(*now);
            if z.t.tick(dt) {
                shell.request_redraw();
            }
        }
        // Only the ACTIVE child lives — the outgoing one is scenery.
        let active = self.second as usize;
        if let Some(child_layout) = layout.children().nth(active) {
            self.children[active].as_widget_mut().update(
                &mut tree.children[active],
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let z = tree.state.downcast_ref::<MorphState>();
        let t = z.t.value.clamp(0.0, 1.5);
        let layouts: Vec<_> = layout.children().collect();
        for (n, scale) in [(0usize, 1.0 - t), (1usize, t)] {
            if scale <= 0.02 {
                continue;
            }
            let Some(child_layout) = layouts.get(n) else { continue };
            let draw = |r: &mut iced::Renderer| {
                self.children[n].as_widget().draw(
                    &tree.children[n],
                    r,
                    theme,
                    style,
                    *child_layout,
                    cursor,
                    viewport,
                );
            };
            if (scale - 1.0).abs() <= 0.001 {
                draw(renderer);
            } else {
                let b = child_layout.bounds();
                let (cx, cy) = (b.center_x(), b.center_y());
                let t = iced::Transformation::translate((1.0 - scale) * cx, (1.0 - scale) * cy)
                    * iced::Transformation::scale(scale);
                renderer.with_transformation(t, draw);
            }
        }
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        let active = self.second as usize;
        match layout.children().nth(active) {
            Some(child_layout) => self.children[active].as_widget().mouse_interaction(
                &tree.children[active],
                child_layout,
                cursor,
                viewport,
                renderer,
            ),
            None => iced::advanced::mouse::Interaction::default(),
        }
    }
}

impl<'a, M: 'a> From<Morph<'a, M>> for Element<'a, M> {
    fn from(w: Morph<'a, M>) -> Self {
        Self::new(w)
    }
}

// ===========================================================================
// The floating anchor (R124b) — zero layout footprint: the child is
// drawn at an offset from the anchor point, and the layout around it
// never learns about it. This is how the reference edit badges HOVER
// outside their row instead of shoving the content aside.
// ===========================================================================

pub fn floating<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    dx: f32,
    dy: f32,
) -> Element<'a, M> {
    Floating { content: content.into(), dx, dy }.into()
}

struct Floating<'a, M> {
    content: Element<'a, M>,
    dx: f32,
    dy: f32,
}

impl<'a, M> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for Floating<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.content.as_widget().tag()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        self.content.as_widget().state()
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.content.as_widget().children()
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        self.content.as_widget().diff(tree);
    }
    fn size(&self) -> iced::Size<iced::Length> {
        // 1x1, not 0x0: parents cull draw against the viewport with
        // Rectangle::intersects, and an empty rect never intersects.
        iced::Size::new(iced::Length::Fixed(1.0), iced::Length::Fixed(1.0))
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let _ = limits;
        let free = iced::advanced::layout::Limits::new(iced::Size::ZERO, iced::Size::INFINITE);
        let child = self
            .content
            .as_widget_mut()
            .layout(tree, renderer, &free)
            .move_to(iced::Point::new(self.dx, self.dy));
        iced::advanced::layout::Node::with_children(iced::Size::new(1.0, 1.0), vec![child])
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        if let Some(child) = layout.children().next() {
            self.content
                .as_widget_mut()
                .update(tree, event, child, cursor, renderer, clipboard, shell, viewport);
        }
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        if let Some(child) = layout.children().next() {
            self.content
                .as_widget()
                .draw(tree, renderer, theme, style, child, cursor, viewport);
        }
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        match layout.children().next() {
            Some(child) => self
                .content
                .as_widget()
                .mouse_interaction(tree, child, cursor, viewport, renderer),
            None => iced::advanced::mouse::Interaction::default(),
        }
    }
}

impl<'a, M: 'a> From<Floating<'a, M>> for Element<'a, M> {
    fn from(w: Floating<'a, M>) -> Self {
        Self::new(w)
    }
}

pub use matrixkit_scene::MAGNIFIER_MAX;

pub fn magnifier<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Magnifier { content: content.into(), max: MAGNIFIER_MAX }.into()
}

/// Like `zoom`, but with its own max scale (dock tiles like more).
pub fn magnifier_strong<'a, M: 'a>(content: impl Into<Element<'a, M>>, max: f32) -> Element<'a, M> {
    Magnifier { content: content.into(), max }.into()
}

struct Magnifier<'a, M> {
    content: Element<'a, M>,
    max: f32,
}

struct MagnifierState {
    hover: bool,
    /// R112 largeSize/actualLargeSize (Dock extract): the magnification
    /// is a spring toward a target, never set — slight overshoot pops.
    scale_row: mk::motion::Spring,
    last: Option<std::time::Instant>,
}

impl Default for MagnifierState {
    fn default() -> Self {
        Self {
            hover: false,
            scale_row: mk::motion::Spring::with(1.0, 0.3, 0.18),
            last: None,
        }
    }
}

impl<'a, M> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for Magnifier<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<MagnifierState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(MagnifierState::default())
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        let hard = mk::motion_reduced();
        let z = tree.state.downcast_mut::<MagnifierState>();
        let over = cursor.is_over(layout.bounds());
        if over != z.hover {
            z.hover = over;
            let target = if over { self.max } else { 1.0 };
            if hard {
                z.scale_row = mk::motion::Spring::with(target, 0.3, 0.18);
            } else {
                z.scale_row.retarget(target);
                shell.request_redraw();
            }
        }
        // Pull the spring along on every frame (RedrawRequested).
        if let iced::Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            if !hard {
                let dt = z
                    .last
                    .map(|l| now.duration_since(l).as_secs_f32())
                    .unwrap_or(0.0)
                    .clamp(0.0, 0.05);
                z.last = Some(*now);
                if z.scale_row.tick(dt) {
                    shell.request_redraw();
                }
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let z = tree.state.downcast_ref::<MagnifierState>();
        let s = z.scale_row.value;
        let draw = |r: &mut iced::Renderer| {
            self.content
                .as_widget()
                .draw(&tree.children[0], r, theme, style, layout, cursor, viewport);
        };
        if s <= 1.001 {
            draw(renderer);
        } else {
            let b = layout.bounds();
            let (cx, cy) = (b.center_x(), b.center_y());
            let t = iced::Transformation::translate((1.0 - s) * cx, (1.0 - s) * cy)
                * iced::Transformation::scale(s);
            renderer.with_transformation(t, draw);
        }
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: 'a> From<Magnifier<'a, M>> for Element<'a, M> {
    fn from(l: Magnifier<'a, M>) -> Self {
        Element::new(l)
    }
}

// ===========================================================================
// The touch round (R59) — UIScrollView/UIKeyboard extracts from the
// touch reference SDK 27.0 (UIScrollView.h on the reference system, 2026-07-16):
// * "if the user then drags far enough, we switch back to dragging and
//   cancel any tracking in the subview" — a drag starts after a slop
//   and then CANCELS the tracking of the subview. That is why a flick
//   on the tablet reference never accidentally clicks a switch.
// * After release the content coasts out with decelerationRate
//   (UIScrollViewDecelerationRateNormal — 0.998 per millisecond).
// * The reference keyboard types on TOUCH, not on release —
//   half the felt latency ("this has no effect on presses").
// iced knows no touch-cancel event; the swipe surface emulates the
// cancel with a synthetic FingerLost — same effect: the child
// forgets the finger without firing.
// ===========================================================================

/// Finger slop: this far a tap may wander before it becomes a drag.
const SWIPE_SLOP: f32 = 10.0;

/// Swipe surface: turns any content into direct manipulation. `on_zug`
/// receives the finger movement (dy per move, downward positive),
/// `on_ende` the release velocity in px/s for the coast-out.
/// Mice and trackpads pass the widget unchanged.
pub fn swipe_area<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    on_pull: impl Fn(f32) -> M + 'a,
    on_end: impl Fn(f32) -> M + 'a,
) -> Element<'a, M> {
    Element::new(SwipeArea {
        content: content.into(),
        on_pull: Box::new(on_pull),
        on_end: Box::new(on_end),
    })
}

struct SwipeArea<'a, M> {
    content: Element<'a, M>,
    on_pull: Box<dyn Fn(f32) -> M + 'a>,
    on_end: Box<dyn Fn(f32) -> M + 'a>,
}

#[derive(Default)]
struct SwipeState {
    finger: Option<iced::touch::Finger>,
    start_y: f32,
    last_y: f32,
    last_time: Option<std::time::Instant>,
    drags: bool,
    /// Smoothed finger velocity (px/s, downward positive).
    v: f32,
}

impl<'a, M: Clone> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for SwipeArea<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<SwipeState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(SwipeState::default())
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        use iced::touch::Event as T;
        if let iced::Event::Touch(t) = event {
            let z = tree.state.downcast_mut::<SwipeState>();
            match t {
                T::FingerPressed { id, position } => {
                    if z.finger.is_none() && layout.bounds().contains(*position) {
                        z.finger = Some(*id);
                        z.start_y = position.y;
                        z.last_y = position.y;
                        z.last_time = Some(std::time::Instant::now());
                        z.drags = false;
                        z.v = 0.0;
                    }
                    // pass through — a tap should act normally
                }
                T::FingerMoved { id, position } if z.finger == Some(*id) => {
                    if !z.drags && (position.y - z.start_y).abs() > SWIPE_SLOP {
                        z.drags = true;
                        // UIScrollView cancel: the child loses the finger before
                        // the drag lets it fire accidentally.
                        let lost =
                            iced::Event::Touch(T::FingerLost { id: *id, position: *position });
                        self.content.as_widget_mut().update(
                            &mut tree.children[0],
                            &lost,
                            layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    }
                    if z.drags {
                        let dy = position.y - z.last_y;
                        let now = std::time::Instant::now();
                        if let Some(prev) = z.last_time {
                            let dt = now.duration_since(prev).as_secs_f32();
                            if dt > 0.0005 {
                                z.v = 0.7 * z.v + 0.3 * (dy / dt);
                            }
                        }
                        z.last_y = position.y;
                        z.last_time = Some(now);
                        if dy != 0.0 {
                            shell.publish((self.on_pull)(dy));
                        }
                        shell.capture_event();
                        return; // the child does not see the drag
                    }
                }
                T::FingerLifted { id, .. } | T::FingerLost { id, .. }
                    if z.finger == Some(*id) =>
                {
                    let zog = z.drags;
                    let v = z.v;
                    z.finger = None;
                    z.drags = false;
                    z.v = 0.0;
                    if zog {
                        // child was already canceled at drag start.
                        shell.publish((self.on_end)(v));
                        shell.capture_event();
                        return;
                    }
                }
                _ => {}
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: Clone + 'a> From<SwipeArea<'a, M>> for Element<'a, M> {
    fn from(w: SwipeArea<'a, M>) -> Self {
        Element::new(w)
    }
}

/// TAP ANSWER (R209, the owner: «the dock reacts well thanks to the haptics
/// and the selection mark — use THAT instead of the magnifier»): the
/// element answers a touch, not a hover. On press a haptic tick (duo
/// devices, `mk::device::haptic`), on release a ring in the accent that
/// fades over `TAP_FLASH_S` (`mk::motion::tap_glow`). The child keeps its
/// own pressed state; mice and keys pass through unchanged.
pub fn tap_answer<'a, M: 'a>(content: impl Into<Element<'a, M>>, radius: f32, p: mk::Palette) -> Element<'a, M> {
    Element::new(TapAnswer { content: content.into(), radius, p })
}

/// How long the ring of a tap answer glows (seconds) — the dock's flash.
pub const TAP_FLASH_S: f32 = mk::motion::TAP_FLASH_S;

/// TAP FLASH — lives iced-free in `matrixkit_theme::motion` (R209), shared
/// with the house.
pub use mk::motion::TapFlash;

struct TapAnswer<'a, M> {
    content: Element<'a, M>,
    radius: f32,
    p: mk::Palette,
}

#[derive(Default)]
struct TapAnswerState {
    pressed: bool,
    released: Option<std::time::Instant>,
}

impl<'a, M> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for TapAnswer<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<TapAnswerState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(TapAnswerState::default())
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        let z = tree.state.downcast_mut::<TapAnswerState>();
        match event {
            iced::Event::Touch(iced::touch::Event::FingerPressed { position, .. })
                if layout.bounds().contains(*position) =>
            {
                z.pressed = true;
                mk::feel::tap();
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
                if cursor.is_over(layout.bounds()) =>
            {
                z.pressed = true;
            }
            iced::Event::Touch(iced::touch::Event::FingerLifted { .. })
            | iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left))
                if z.pressed =>
            {
                z.pressed = false;
                z.released = Some(std::time::Instant::now());
                shell.request_redraw();
            }
            iced::Event::Touch(iced::touch::Event::FingerLost { .. }) => z.pressed = false,
            iced::Event::Window(iced::window::Event::RedrawRequested(_)) => {
                if z.released.is_some_and(|t| t.elapsed().as_secs_f32() < TAP_FLASH_S) {
                    shell.request_redraw();
                } else {
                    z.released = None;
                }
            }
            _ => {}
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
        let z = tree.state.downcast_ref::<TapAnswerState>();
        let glow = mk::motion::tap_glow(z.released, TAP_FLASH_S, None);
        if glow > 0.01 {
            renderer.fill_quad(
                iced::advanced::renderer::Quad {
                    bounds: layout.bounds(),
                    border: iced::Border {
                        radius: self.radius.into(),
                        width: 1.0 + glow,
                        color: color(self.p.primary.with_alpha(0.3 + 0.6 * glow)),
                    },
                    ..Default::default()
                },
                iced::Color::TRANSPARENT,
            );
        }
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(&mut tree.children[0], layout, renderer, viewport, translation)
    }
}

/// Window flick (R209, duo devices): a finger pulled across the window
/// header and released sends the window to the neighbouring panel —
/// the canvas moves it (`move-window-to-monitor-<side>`). winit cannot
/// hand a touch to the compositor as a window move (its drag_window only
/// knows pointer serials), so the header recognises the flick itself.
/// Touching the header activated the window, so the action hits it.
/// Mice and short taps pass through unchanged.
pub fn flick_area<'a, M: Clone + 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    Element::new(FlickArea { content: content.into() })
}

/// This far (logical px) the finger must travel for a flick.
const FLICK_MIN: f32 = 60.0;

struct FlickArea<'a, M> {
    content: Element<'a, M>,
}

#[derive(Default)]
struct FlickState {
    finger: Option<iced::touch::Finger>,
    start: iced::Point,
    last: iced::Point,
    drags: bool,
}

/// The neighbouring panel in the direction of the larger movement.
fn flick_side(dx: f32, dy: f32) -> &'static str {
    if dx.abs() >= dy.abs() {
        if dx > 0.0 { "right" } else { "left" }
    } else if dy > 0.0 {
        "down"
    } else {
        "up"
    }
}

impl<'a, M: Clone> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for FlickArea<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<FlickState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(FlickState::default())
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        use iced::touch::Event as T;
        if let iced::Event::Touch(t) = event {
            let z = tree.state.downcast_mut::<FlickState>();
            match t {
                T::FingerPressed { id, position } => {
                    if z.finger.is_none() && layout.bounds().contains(*position) {
                        z.finger = Some(*id);
                        z.start = *position;
                        z.last = *position;
                        z.drags = false;
                    }
                }
                T::FingerMoved { id, position } if z.finger == Some(*id) => {
                    z.last = *position;
                    if !z.drags && z.start.distance(*position) > SWIPE_SLOP {
                        z.drags = true;
                        // the header buttons must not fire on a flick
                        let lost = iced::Event::Touch(T::FingerLost { id: *id, position: *position });
                        self.content.as_widget_mut().update(
                            &mut tree.children[0],
                            &lost,
                            layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            viewport,
                        );
                    }
                    if z.drags {
                        shell.capture_event();
                        return;
                    }
                }
                T::FingerLifted { id, position } if z.finger == Some(*id) => {
                    let drags = z.drags;
                    let (dx, dy) = (position.x - z.start.x, position.y - z.start.y);
                    z.finger = None;
                    z.drags = false;
                    if drags {
                        if dx.hypot(dy) >= FLICK_MIN {
                            let side = flick_side(dx, dy);
                            std::thread::spawn(move || mk::canvas::window_to_neighbour(side));
                        }
                        shell.capture_event();
                        return;
                    }
                }
                T::FingerLost { id, .. } if z.finger == Some(*id) => {
                    z.finger = None;
                    z.drags = false;
                }
                _ => {}
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// Instant button (UIKeyboard extract): fires on TOUCH instead of on
/// release — for keys where speed matters (on-screen keyboard).
/// The child (the button) keeps showing its pressed state but does
/// not fire itself for touch (FingerLost instead of FingerLifted);
/// mouse and trackpad pass through the child unchanged.
pub fn instant_button<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    msg: M,
) -> Element<'a, M> {
    Element::new(InstantButton { content: content.into(), msg })
}

struct InstantButton<'a, M> {
    content: Element<'a, M>,
    msg: M,
}

#[derive(Default)]
struct InstantState {
    finger: Option<iced::touch::Finger>,
}

impl<'a, M: Clone> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for InstantButton<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<InstantState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(InstantState::default())
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        use iced::touch::Event as T;
        if let iced::Event::Touch(t) = event {
            let z = tree.state.downcast_mut::<InstantState>();
            match t {
                T::FingerPressed { id, position } => {
                    if z.finger.is_none() && layout.bounds().contains(*position) {
                        z.finger = Some(*id);
                        // Fire NOW — not only on release.
                        shell.publish(self.msg.clone());
                        // The button below may show "pressed".
                    }
                }
                T::FingerLifted { id, position } | T::FingerLost { id, position }
                    if z.finger == Some(*id) =>
                {
                    z.finger = None;
                    // Give the child a FingerLost instead of the Lifted —
                    // reset the look without letting it fire itself.
                    let lost =
                        iced::Event::Touch(T::FingerLost { id: *id, position: *position });
                    self.content.as_widget_mut().update(
                        &mut tree.children[0],
                        &lost,
                        layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                    shell.capture_event();
                    return;
                }
                _ => {}
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: Clone + 'a> From<InstantButton<'a, M>> for Element<'a, M> {
    fn from(s: InstantButton<'a, M>) -> Self {
        Element::new(s)
    }
}

// ===========================================================================
// The kit sidebar (user unification, 2026-07-08): ONE building block
// for all sidebars — dark ground (the Hilfe look), magnifier on every
// entry (the canvas haptics), active state in primary container,
// optional number badge (reference badge) and focus ring for the
// keyboard model. Apps only describe their entries.
// ===========================================================================
pub struct SidebarItem<'a> {
    pub glyph: char,
    pub title: &'a str,
    /// Number on the right (e.g. articles per category) — None = no badge.
    pub count: Option<usize>,
}

/// ONE sidebar entry — the family building block used by `sidebar()`
/// AND every app with its own bar anatomy (Dateien): same height,
/// same palette pill (primary_container) for the active one, same
/// hover, same magnifier. ONE code path = guaranteed identical.
pub fn sidebar_entry<'a, M: Clone + 'a>(
    item: SidebarItem<'a>,
    is_active: bool,
    in_focus: bool,
    msg: M,
    p: mk::Palette,
) -> Element<'a, M> {
    let text_color = if is_active { p.on_primary_container } else { p.on_surface };
    // R102 (SF Symbols grammar): the active entry carries the FILLED glyph.
    let glyph = if is_active {
        symbol_fill::<M>(item.glyph, mk::font_size::LARGE, text_color)
    } else {
        symbol::<M>(item.glyph, mk::font_size::LARGE, text_color)
    };
    let mut line = row![
        glyph,
        Space::new().width(mk::spacing::S),
        txt(item.title, mk::typo::BODY, text_color),
        Space::new().width(Length::Fill),
    ]
    .align_y(iced::Alignment::Center);
    if let Some(n) = item.count {
        line = line.push(badge::<M>(n, p));
    }
    magnifier(
        iced::widget::button(line)
            .width(Length::Fill)
            .padding([mk::spacing::S as u16, mk::spacing::M as u16])
            .on_press(msg)
            .style(move |_, status| {
                let base = if is_active { p.primary_container } else { p.surface };
                let bg = match status {
                    iced::widget::button::Status::Hovered if !is_active => {
                        p.on_surface.over(base, mk::state_layer::HOVER)
                    }
                    iced::widget::button::Status::Pressed if !is_active => {
                        p.on_surface.over(base, mk::state_layer::PRESSED)
                    }
                    _ => base,
                };
                iced::widget::button::Style {
                    background: (is_active
                        || !matches!(status, iced::widget::button::Status::Active))
                    .then(|| color(bg).into()),
                    border: if in_focus {
                        focus_ring(true, mk::CORNER_RADIUS, p)
                    } else {
                        iced::Border {
                            radius: mk::CORNER_RADIUS.into(),
                            ..Default::default()
                        }
                    },
                    ..Default::default()
                }
            }),
    )
}

pub fn sidebar<'a, M: Clone + 'a>(
    items: Vec<SidebarItem<'a>>,
    active: usize,
    focus: Option<usize>,
    width: f32,
    on_choice: impl Fn(usize) -> M + 'a,
    p: mk::Palette,
) -> Element<'a, M> {
    let mut page = column![].spacing(mk::spacing::XS);
    for (i, item) in items.into_iter().enumerate() {
        page = page.push(sidebar_entry(item, i == active, focus == Some(i), on_choice(i), p));
    }
    sidebar_area(page.into(), width, p)
}

/// The dark sidebar ground of the Hilfe look as its own family building
/// block — for `sidebar()` AND apps with their own bar structure
/// (Dateien: sections). Same surface, same left rounding, ONE code path.
pub fn sidebar_area<'a, M: 'a>(
    content: Element<'a, M>,
    width: f32,
    p: mk::Palette,
) -> Element<'a, M> {
    container(content)
        .padding(mk::spacing::S)
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(color(p.surface).into()),
            border: iced::Border {
                // R103b (user finding): the sidebar meets the HEADER at the
                // top — a top rounding punched a notch into the frame there.
                // Only the true window corner (bottom left) stays round.
                radius: iced::border::Radius {
                    top_left: 0.0,
                    bottom_left: mk::CORNER_RADIUS,
                    top_right: 0.0,
                    bottom_right: 0.0,
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

// ===========================================================================
// MatrixUI — the pattern layer above the MatrixKit building blocks
// (user directive, 2026-07-08): while mkw supplies the BLOCKS,
// MatrixUI supplies the binding FAMILIES — block + anatomy in one
// grip, so unification does not depend on discipline.
// First member: the SidebarFamily.
// ===========================================================================

pub mod ui {
    use super::*;

    /// The SidebarFamily: dark kit sidebar bare at the window edge,
    /// the bright detail card on the right (padding L, radius) — the
    /// binding anatomy from Hilfe/Leinwand. Apps supply entries and
    /// detail content; width and embedding belong to the family.
    pub const SIDEBAR_WIDTH: f32 = 190.0;

    pub fn sidebar_family<'a, M: Clone + 'a>(
        items: Vec<SidebarItem<'a>>,
        active: usize,
        focus: Option<usize>,
        on_choice: impl Fn(usize) -> M + 'a,
        detail: Element<'a, M>,
        p: mk::Palette,
    ) -> Element<'a, M> {
        let panel = sidebar(items, active, focus, SIDEBAR_WIDTH, on_choice, p);
        row![panel, detail_card(detail, p)].into()
    }

    /// The SidebarFamily in its FORMS (R207, R209 Duo): from tablet width up
    /// exactly `sidebar_family`; in the phone form (duo panel, narrow window)
    /// ONE column — `menu_open` shows the section list over the whole width,
    /// otherwise the section fills the window as a detail card. The app keeps
    /// `menu_open`, sets it to false on a choice and leads ◀ back to the list
    /// with [`phone_back`].
    #[allow(clippy::too_many_arguments)]
    pub fn sidebar_family_form<'a, M: Clone + 'a>(
        width: f32,
        menu_open: bool,
        items: Vec<SidebarItem<'a>>,
        active: usize,
        focus: Option<usize>,
        on_choice: impl Fn(usize) -> M + 'a,
        detail: Element<'a, M>,
        p: mk::Palette,
    ) -> Element<'a, M> {
        if mk::form::of(width) != mk::form::Form::Phone {
            sidebar_family(items, active, focus, on_choice, detail, p)
        } else if menu_open {
            sidebar(items, active, focus, width, on_choice, p)
        } else {
            detail_card(detail, p)
        }
    }

    /// ◀ of the toolbar in the SidebarFamily forms: in the phone form an open
    /// section leads back to the list (`to_list`), otherwise the history.
    pub fn phone_back<M>(width: f32, menu_open: bool, to_list: M, history: Option<M>) -> Option<M> {
        if mk::form::of(width) == mk::form::Form::Phone && !menu_open {
            Some(to_list)
        } else {
            history
        }
    }

    /// CardRowFamily: cards side by side as long as they fit (`min_card`
    /// logical px each, gap L), otherwise in as many rows as needed — on a
    /// duo panel two live cards stand below each other instead of cutting
    /// their text.
    pub fn card_row<'a, M: 'a>(cards: Vec<Element<'a, M>>, width: f32, min_card: f32) -> Element<'a, M> {
        let per_row = mk::form::columns(width, min_card, mk::spacing::L);
        let mut rows = iced::widget::column![].spacing(mk::spacing::L);
        let mut current = row![].spacing(mk::spacing::L);
        let mut n = 0;
        for card in cards {
            current = current.push(card);
            n += 1;
            if n == per_row {
                rows = rows.push(current);
                current = row![].spacing(mk::spacing::L);
                n = 0;
            }
        }
        if n > 0 {
            rows = rows.push(current);
        }
        rows.into()
    }

    /// The bright detail card of the SidebarFamily as its own block —
    /// the same view ground for Hilfe AND apps with their own structure.
    pub fn detail_card<'a, M: 'a>(content: Element<'a, M>, p: mk::Palette) -> Element<'a, M> {
        container(content)
            .padding(mk::spacing::L)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(color(p.surface_container).into()),
                border: iced::Border {
                    // R103b (user finding): the card meets the header above
                    // and the sidebar on the left — roundings there broke
                    // the frame's continuity. Only the true window corner
                    // (bottom right) stays round.
                    radius: iced::border::Radius {
                        top_left: 0.0,
                        top_right: 0.0,
                        bottom_left: 0.0,
                        bottom_right: mk::CORNER_RADIUS,
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// The full content area of a window WITHOUT a sidebar (R103c):
    /// the same view ground as detail_card, spanning the whole width.
    /// Square at the header seam above, round only at the two real
    /// bottom window corners — the frame stays continuous.
    pub fn content_area<'a, M: 'a>(
        content: Element<'a, M>,
        padding: f32,
        p: mk::Palette,
    ) -> Element<'a, M> {
        container(content)
            .padding(padding)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(color(p.surface_container).into()),
                border: iced::Border {
                    radius: iced::border::Radius {
                        top_left: 0.0,
                        top_right: 0.0,
                        bottom_left: mk::CORNER_RADIUS,
                        bottom_right: mk::CORNER_RADIUS,
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Image-tile family, widget side: 112x63 thumb (16:9, corners baked
    /// into the image — recipe in mkw::bild), name below it (max. 16
    /// characters), selection = 2px primary border, optional badge top
    /// right. First born in the wallpaper section; the gallery in
    /// Matrix Dateien carries the same anatomy.
    pub const IMAGE_TILE_W: f32 = 112.0;
    pub const IMAGE_TILE_H: f32 = 63.0;

    /// Responsive grid (R87): how many tiles fit side by side?
    /// `available` = width of the stage (window width minus sidebar and
    /// padding), `schritt` = tile width plus spacing. Never below 1 —
    /// and with the window width still unknown the caller keeps computing
    /// with its start width (Rahmen.groesse is ZERO then).
    pub fn grid_columns(available: f32, step: f32) -> usize {
        ((available / step) as usize).max(1)
    }

    /// Grid family (R88): builds the grid from tiles, whose column count
    /// breathes with the stage — rows and spacing come from one hand,
    /// no app counts by itself anymore. Rigid `chunks(N)` in views
    /// are a police case since R88.
    pub fn raster<'a, M: 'a>(
        tiles: Vec<Element<'a, M>>,
        available: f32,
        step: f32,
    ) -> Element<'a, M> {
        let columns = grid_columns(available, step);
        let mut grid = iced::widget::Column::new().spacing(mk::spacing::S);
        let mut rest = tiles.into_iter();
        loop {
            let row: Vec<Element<'a, M>> = rest.by_ref().take(columns).collect();
            if row.is_empty() {
                break;
            }
            grid = grid.push(iced::widget::Row::with_children(row).spacing(mk::spacing::S));
        }
        grid.into()
    }

    pub fn image_tile<'a, M: 'a>(
        handle: Option<iced::widget::image::Handle>,
        placeholder: char,
        name: String,
        selected: bool,
        badge: Option<String>,
        p: mk::Palette,
    ) -> Element<'a, M> {
        use iced::widget::stack;
        let area: Element<'a, M> = match handle {
            Some(h) => iced::widget::image(h)
                .width(Length::Fixed(IMAGE_TILE_W))
                .height(Length::Fixed(IMAGE_TILE_H))
                .content_fit(iced::ContentFit::Fill)
                .into(),
            None => container(symbol::<M>(placeholder, 28.0, p.on_surface_variant))
                .center_x(Length::Fixed(IMAGE_TILE_W))
                .center_y(Length::Fixed(IMAGE_TILE_H))
                .style(move |_| container::Style {
                    background: Some(color(p.surface_container_high).into()),
                    border: iced::Border { radius: (mk::radius::SMALL - 2.0).into(), ..Default::default() },
                    ..Default::default()
                })
                .into(),
        };
        let mut layers = stack![area];
        if let Some(b) = badge {
            layers = layers.push(
                container(
                    container(txt(b, mk::typo::LABEL, p.on_primary))
                        .padding(iced::Padding { top: 1.0, right: 6.0, bottom: 1.0, left: 6.0 })
                        .style(move |_| container::Style {
                            background: Some(color(p.primary).into()),
                            border: iced::Border { radius: mk::radius::capsule(18.0).into(), ..Default::default() },
                            ..Default::default()
                        }),
                )
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right)
                .padding(3),
            );
        }
        let mut short: String = name.chars().take(16).collect();
        if name.chars().count() > 16 {
            short.push('\u{2026}');
        }
        let name_color = if selected { p.primary } else { p.on_surface_variant };
        iced::widget::column![
            container(layers).padding(2).style(move |_| container::Style {
                border: iced::Border {
                    color: color(p.primary),
                    width: if selected { 2.0 } else { 0.0 },
                    radius: mk::radius::SMALL.into(),
                },
                ..Default::default()
            }),
            txt(short, mk::typo::LABEL, name_color),
        ]
        .spacing(2)
        .align_x(iced::Alignment::Center)
        .into()
    }

    /// Tool button of the toolbar family: symbol button, dimmed and
    /// dead when `msg` is missing (reference grammar: inactive chevrons stay
    /// visible). First born in Matrix Dateien, now a family building block.
    pub fn tool_button<'a, M: Clone + 'a>(
        glyph: char,
        msg: Option<M>,
        p: mk::Palette,
    ) -> Element<'a, M> {
        let on = msg.is_some();
        let mut b = iced::widget::button(symbol::<M>(
            glyph,
            18.0,
            if on { p.on_surface } else { p.outline },
        ))
        .padding(iced::Padding { left: 6.0, right: 6.0, top: 4.0, bottom: 4.0 })
        .style(move |_, status| panel::button_style(p, status, mk::radius::SMALL));
        if let Some(m) = msg {
            b = b.on_press(m);
        }
        magnifier(b)
    }

    /// Toolbar family (reference toolbar extract, first born in
    /// Matrix Dateien): ◀ ▶ [own navigation] title … [right buttons]
    /// [search field 200 px]. Belongs at the HEAD of the detail card.
    #[allow(clippy::too_many_arguments)]
    pub fn toolbar<'a, M: Clone + 'a>(
        title: String,
        back: Option<M>,
        forward: Option<M>,
        navigation_extra: Vec<Element<'a, M>>,
        right: Vec<Element<'a, M>>,
        search: &str,
        on_search: impl Fn(String) -> M + 'a,
        search_clear: M,
        p: mk::Palette,
    ) -> Element<'a, M> {
        let mut line = row![
            tool_button(symbol::ARROW_BACK, back, p),
            tool_button(symbol::ARROW_FORWARD, forward, p),
        ]
        .spacing(mk::spacing::XS)
        .align_y(iced::Alignment::Center);
        for e in navigation_extra {
            line = line.push(e);
        }
        line = line
            .push(Space::new().width(mk::spacing::S))
            .push(txt(title, mk::typo::HEADLINE, p.on_surface))
            .push(Space::new().width(Length::Fill));
        for e in right {
            line = line.push(e);
        }
        line
            .push(
                container(search_field(search, "Suchen", on_search, search_clear, p))
                    .width(Length::Fixed(200.0)),
            )
            .into()
    }

    /// The SidebarFamily entry look as a free building block (user,
    /// 2026-07-08: "top-bar widgets same as the sidebar elements"):
    /// symbol+text grammar, state-layer hover, the active pill in
    /// primary container, CORNER_RADIUS, magnifier. For bar widgets and
    /// everywhere entries should look like sidebar entries.
    pub fn family_button<'a, M: Clone + 'a>(
        glyph: Option<char>,
        title: Option<&'a str>,
        active: bool,
        msg: M,
        p: mk::Palette,
        style: mk::typo::TextStyle,
    ) -> Element<'a, M> {
        let text_color = if active { p.on_primary_container } else { p.on_surface };
        let mut line = row![].spacing(mk::spacing::S).align_y(iced::Alignment::Center);
        if let Some(z) = glyph {
            line = line.push(symbol::<M>(z, mk::font_size::LARGE, text_color));
        }
        if let Some(t) = title {
            // R60: tight row box — in capped bars (bar: 32 px)
            // the button must never want to be taller than allowed; otherwise
            // iced silently collapses the bottom padding (lopsidedness).
            line = line.push(
                txt(t, style, text_color)
                    .line_height(iced::widget::text::LineHeight::Relative(1.0)),
            );
        }
        magnifier(
            iced::widget::button(line)
                // Exactly the sidebar entry height (user, 2026-07-08):
                // same padding as SidebarPunkt buttons.
                .padding([mk::spacing::S as u16, mk::spacing::M as u16])
                .on_press(msg)
                .style(move |_, status| {
                    let base = if active { p.primary_container } else { p.surface };
                    let bg = match status {
                        iced::widget::button::Status::Hovered if !active => {
                            p.on_surface.over(base, mk::state_layer::HOVER)
                        }
                        iced::widget::button::Status::Pressed if !active => {
                            p.on_surface.over(base, mk::state_layer::PRESSED)
                        }
                        _ => base,
                    };
                    iced::widget::button::Style {
                        background: (active
                            || !matches!(status, iced::widget::button::Status::Active))
                        .then(|| color(bg).into()),
                        border: iced::Border {
                            radius: mk::CORNER_RADIUS.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
        )
    }

    /// The MenuFamily (user, 2026-07-08): ONE menu look for ALL menus —
    /// session menu, dock context, canvas context, in-app context menus.
    /// Entries carry the MatrixUI grammar (symbol+text, state layer,
    /// CORNER_RADIUS, magnifier), danger entries come in error color, the
    /// shell is the kit pill with shadow. Width is family norm.
    pub const MENU_WIDTH: f32 = 230.0;

    #[derive(Clone)]
    pub enum MenuItem<M> {
        Item {
            glyph: Option<char>,
            title: String,
            /// None = on_surface; Some(error) for danger/confirmation.
            color: Option<mk::Rgba>,
            msg: M,
        },
        /// Item with key shortcut on the right (reference MenuItem.keyEquivalent,
        /// round 23) — use only for REAL binds.
        ItemWithShortcut {
            glyph: Option<char>,
            title: String,
            shortcut: String,
            color: Option<mk::Rgba>,
            msg: M,
        },
        /// Grayed item (measurement round 36): the reference system NEVER hides
        /// disabled entries — they remain visible in gray so the menu keeps
        /// its shape and you learn what would be available.
        Disabled {
            glyph: Option<char>,
            title: String,
        },
        /// Item with a live badge on the right (Apple-menu grammar, round 28:
        /// "Systemeinstellungen …  [1 Update]") — the menu carries status.
        ItemWithBadge {
            glyph: Option<char>,
            title: String,
            badge: String,
            color: Option<mk::Rgba>,
            msg: M,
        },
        Separator,
    }

    /// The ONE floating shell of the MenuFamily — for menus AND panels
    /// (Zentrale, notification history): same radius, same
    /// padding, same shadow. Never again three pill dialects.
    /// R122: menu glyphs sit in soft tinted circles like the
    /// reference control-center rows.
    pub fn menu_glyph_circle<'a, M: 'a>(z: char, tint: mk::Rgba, p: mk::Palette) -> Element<'a, M> {
        container(symbol(z, mk::icon_size::SMALL + 2.0, tint))
            .center_x(Length::Fixed(28.0))
            .center_y(Length::Fixed(28.0))
            .style(move |_| container::Style {
                background: Some(
                    color(p.on_surface.over(p.surface_container_high, 0.08)).into(),
                ),
                border: iced::Border { radius: 999.0_f32.into(), ..Default::default() },
                ..Default::default()
            })
            .into()
    }

    /// The reference minus badge (R122 phase 2): the small filled
    /// error circle that removes a row while a surface is editing —
    /// firstborn in the Zentrale's control-edit flow.
    pub fn minus_badge<'a, M: Clone + 'a>(on: M, p: mk::Palette) -> Element<'a, M> {
        const BADGE: f32 = 22.0;
        iced::widget::button(
            container(symbol::<M>(symbol::REMOVE, mk::icon_size::SMALL - 2.0, p.surface))
                .center_x(Length::Fixed(BADGE))
                .center_y(Length::Fixed(BADGE)),
        )
        .padding(0)
        .style(move |_, _| iced::widget::button::Style {
            background: Some(color(p.error).into()),
            border: iced::Border {
                radius: mk::radius::capsule(BADGE).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .on_press(on)
        .into()
    }

    pub use matrixkit_scene::ui::{glass_on, menu_glass_flag, menu_glass_on, menus_on_glass};

    /// Exact-enough footprint of a menu card window (R125 glass
    /// menus): mirrors menu_family's metrics — glyph rows, bare rows,
    /// disabled rows, XXS row gaps, S shell padding, M island gaps.
    pub fn menu_size<M>(entries: &[MenuItem<M>]) -> (f32, f32) {
        let mut h = 0.0f32;
        let mut groups = 0u32;
        let mut open = false;
        let mut rows_in_group = 0u32;
        for e in entries {
            match e {
                MenuItem::Separator => {
                    open = false;
                }
                other => {
                    if !open {
                        open = true;
                        groups += 1;
                        rows_in_group = 0;
                    }
                    if rows_in_group > 0 {
                        h += mk::spacing::XXS;
                    }
                    rows_in_group += 1;
                    h += match other {
                        MenuItem::Disabled { .. } => 30.0,
                        MenuItem::Item { glyph, .. }
                        | MenuItem::ItemWithShortcut { glyph, .. }
                        | MenuItem::ItemWithBadge { glyph, .. } => {
                            if glyph.is_some() { 40.0 } else { 33.0 }
                        }
                        MenuItem::Separator => 0.0,
                    };
                }
            }
        }
        h += groups as f32 * 2.0 * mk::spacing::S;
        if groups > 1 {
            h += (groups - 1) as f32 * mk::spacing::M;
        }
        (MENU_WIDTH, h)
    }

    /// R122 island shell for menus: rounder than the generic panel,
    /// faintly translucent (a hint of the reference glass), inner air.
    pub fn menu_shell<'a, M: 'a>(
        content: Element<'a, M>,
        width: f32,
        p: mk::Palette,
    ) -> Element<'a, M> {
        // On a compositor glass card the shell mostly VANISHES — the
        // glass body IS the shell, like the bar pill: only a whisper
        // of tint remains for legibility (user wish 23.07: the island
        // itself is the glass, not a slab around it).
        let on_glass = menu_glass_flag();
        let shell_alpha = if on_glass && glass_on() { 0.22 } else { 0.94 };
        panel::shadow_layers(
            container(container(content).padding(mk::spacing::S))
                .width(Length::Fixed(width))
                .style(move |_| container::Style {
                    background: Some(
                        color(mk::Rgba { a: shell_alpha, ..p.surface_container_high }).into(),
                    ),
                    border: iced::Border {
                        radius: (mk::CORNER_RADIUS + 6.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into(),
            mk::CORNER_RADIUS + 6.0,
        )
    }

    pub fn panel_shell<'a, M: 'a>(
        content: Element<'a, M>,
        width: f32,
        p: mk::Palette,
    ) -> Element<'a, M> {
        // R55c (user): menus, too, carry the triple shadow — the family
        // inherits the gauss fade at the root, not every shell
        // individually.
        panel::shadow_layers(
            container(container(content).padding(mk::spacing::S))
                .width(Length::Fixed(width))
                .style(move |_| panel::pill(p, mk::radius::SMALL, mk::spacing::S))
                .into(),
            mk::radius::SMALL + mk::spacing::S,
        )
    }

    /// The panel head of the family: title left, action buttons right
    /// (apps build them with ui::nav_button or ui::header_text_button),
    /// the separator below.
    pub fn panel_header<'a, M: Clone + 'a>(
        title: &'a str,
        actions: Vec<Element<'a, M>>,
        p: mk::Palette,
    ) -> Element<'a, M> {
        let mut line = row![
            txt(title, mk::typo::HEADLINE, p.on_surface),
            Space::new().width(Length::Fill),
        ]
        .spacing(mk::spacing::XS)
        .align_y(iced::Alignment::Center);
        for a in actions {
            line = line.push(a);
        }
        iced::widget::column![line, separator(p)]
            .spacing(mk::spacing::XS)
            .into()
    }

    /// Text action for panel heads ("Leeren") — family grammar + magnifier.
    pub fn header_text_button<'a, M: Clone + 'a>(
        label: &'a str,
        msg: M,
        p: mk::Palette,
    ) -> Element<'a, M> {
        magnifier(
            iced::widget::button(txt(label, mk::typo::CAPTION, p.on_surface_variant))
                .padding([2, mk::spacing::S as u16])
                .on_press(msg)
                .style(move |_, status| panel::button_style(p, status, mk::radius::SMALL)),
        )
    }

    /// The menu item button (R66, reference menu extract — measured on the
    /// reference system): a chosen entry BLINKS twice (~0.2 s) before its
    /// message fires — the reference click acknowledgement since 1984, in every
    /// menu. iced buttons cannot delay the click without losing hover;
    /// hence this own widget: it paints hover/flash ground itself,
    /// catches the click, plays the flash (off — on — off) and only THEN
    /// publishes. `bewegung reduziert` (reduced motion)
    /// fires immediately.
    pub fn menu_item_button<'a, M: Clone + 'a>(
        content: Element<'a, M>,
        msg: M,
        p: mk::Palette,
    ) -> Element<'a, M> {
        Element::new(MenuItemButton { content, msg, p })
    }

    struct MenuItemButton<'a, M> {
        content: Element<'a, M>,
        msg: M,
        p: mk::Palette,
    }

    #[derive(Default)]
    struct FlashState {
        hover: bool,
        /// Blitz-Beginn; None = ruhend.
        since: Option<std::time::Instant>,
    }

    impl<'a, M: Clone> iced::advanced::Widget<M, iced::Theme, iced::Renderer>
        for MenuItemButton<'a, M>
    {
        fn tag(&self) -> iced::advanced::widget::tree::Tag {
            iced::advanced::widget::tree::Tag::of::<FlashState>()
        }
        fn state(&self) -> iced::advanced::widget::tree::State {
            iced::advanced::widget::tree::State::new(FlashState::default())
        }
        fn children(&self) -> Vec<iced::advanced::widget::Tree> {
            vec![iced::advanced::widget::Tree::new(&self.content)]
        }
        fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
            tree.diff_children(std::slice::from_ref(&self.content));
        }
        fn size(&self) -> iced::Size<iced::Length> {
            self.content.as_widget().size()
        }
        fn layout(
            &mut self,
            tree: &mut iced::advanced::widget::Tree,
            renderer: &iced::Renderer,
            limits: &iced::advanced::layout::Limits,
        ) -> iced::advanced::layout::Node {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        }
        fn update(
            &mut self,
            tree: &mut iced::advanced::widget::Tree,
            event: &iced::Event,
            layout: iced::advanced::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            renderer: &iced::Renderer,
            clipboard: &mut dyn iced::advanced::Clipboard,
            shell: &mut iced::advanced::Shell<'_, M>,
            viewport: &iced::Rectangle,
        ) {
            let z = tree.state.downcast_mut::<FlashState>();
            let over = cursor.is_over(layout.bounds());
            if over != z.hover {
                z.hover = over;
                shell.request_redraw();
            }
            let pressed = matches!(
                event,
                iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Left
                ))
            ) || matches!(event, iced::Event::Touch(iced::touch::Event::FingerPressed { .. }));
            if pressed && over && z.since.is_none() {
                if mk::motion_reduced() {
                    shell.publish(self.msg.clone());
                } else {
                    z.since = Some(std::time::Instant::now());
                    shell.request_redraw();
                }
                shell.capture_event();
                return;
            }
            if let iced::Event::Window(iced::window::Event::RedrawRequested(_)) = event {
                if let Some(since) = z.since {
                    if since.elapsed().as_millis() as u64 >= mk::input::MENU_FLASH_MS {
                        z.since = None;
                        shell.publish(self.msg.clone());
                    } else {
                        shell.request_redraw();
                    }
                }
            }
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        fn draw(
            &self,
            tree: &iced::advanced::widget::Tree,
            renderer: &mut iced::Renderer,
            theme: &iced::Theme,
            style: &iced::advanced::renderer::Style,
            layout: iced::advanced::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            viewport: &iced::Rectangle,
        ) {
            use iced::advanced::Renderer as _;
            let z = tree.state.downcast_ref::<FlashState>();
            // Flash phases (off — ON — off) or the resting hover ground.
            let ground = if let Some(since) = z.since {
                let t = since.elapsed().as_millis() as u64;
                let on = t >= mk::input::MENU_FLASH_MS / 3
                    && t < 2 * mk::input::MENU_FLASH_MS / 3;
                on.then(|| {
                    self.p
                        .on_surface
                        .over(self.p.surface_container_high, mk::state_layer::PRESSED)
                })
            } else if z.hover {
                Some(
                    self.p
                        .on_surface
                        .over(self.p.surface_container_high, mk::state_layer::HOVER),
                )
            } else {
                None
            };
            if let Some(g) = ground {
                renderer.fill_quad(
                    iced::advanced::renderer::Quad {
                        bounds: layout.bounds(),
                        border: iced::Border {
                            radius: mk::CORNER_RADIUS.into(),
                            ..Default::default()
                        },
                        shadow: iced::Shadow::default(),
                        snap: false,
                    },
                    color(g),
                );
            }
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }
        fn mouse_interaction(
            &self,
            _tree: &iced::advanced::widget::Tree,
            layout: iced::advanced::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            _viewport: &iced::Rectangle,
            _renderer: &iced::Renderer,
        ) -> iced::advanced::mouse::Interaction {
            if cursor.is_over(layout.bounds()) {
                iced::advanced::mouse::Interaction::Pointer
            } else {
                iced::advanced::mouse::Interaction::default()
            }
        }
        fn operate(
            &mut self,
            tree: &mut iced::advanced::widget::Tree,
            layout: iced::advanced::Layout<'_>,
            renderer: &iced::Renderer,
            operation: &mut dyn iced::advanced::widget::Operation,
        ) {
            self.content
                .as_widget_mut()
                .operate(&mut tree.children[0], layout, renderer, operation);
        }
        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut iced::advanced::widget::Tree,
            layout: iced::advanced::Layout<'b>,
            renderer: &iced::Renderer,
            viewport: &iced::Rectangle,
            translation: iced::Vector,
        ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>>
        {
            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    }

    pub fn menu_family<'a, M: Clone + 'a>(
        header: Option<Element<'a, M>>,
        entries: Vec<MenuItem<M>>,
        p: mk::Palette,
    ) -> Element<'a, M> {
        // R122 bento menus (user wish 23.07): the reference control
        // center speaks in floating islands — menus follow. Every
        // separator becomes AIR: the groups render as separate island
        // cards, the header gets its own, and the stack pops in with
        // the island spring.
        let mut groups: Vec<iced::widget::Column<'a, M>> = Vec::new();
        let mut column = iced::widget::column![].spacing(mk::spacing::XXS);
        let mut column_used = false;
        if let Some(k) = header {
            groups.push(
                iced::widget::column![container(k).padding([4, mk::spacing::S as u16])]
                    .spacing(mk::spacing::XXS),
            );
        }
        for e in entries {
            match e {
                MenuItem::Separator => {
                    if column_used {
                        groups.push(std::mem::replace(
                            &mut column,
                            iced::widget::column![].spacing(mk::spacing::XXS),
                        ));
                        column_used = false;
                    }
                }
                MenuItem::Disabled { glyph, title } => {
                    let f = p.outline;
                    let content: Element<'a, M> = match glyph {
                        Some(z) => tag(z, title, mk::typo::BODY, f),
                        None => txt(title, mk::typo::BODY, f).into(),
                    };
                    column_used = true;
                    column = column.push(
                        container(content)
                            .width(Length::Fill)
                            .padding([5, mk::spacing::M as u16]),
                    );
                }
                MenuItem::ItemWithBadge { glyph, title, badge, color: tint, msg } => {
                    let f = tint.unwrap_or(p.on_surface);
                    let left: Element<'a, M> = match glyph {
                        Some(z) => row![
                            menu_glyph_circle(z, f, p),
                            txt(title, mk::typo::BODY, f),
                        ]
                        .spacing(mk::spacing::S)
                        .align_y(iced::Alignment::Center)
                        .into(),
                        None => txt(title, mk::typo::BODY, f).into(),
                    };
                    // Badge pill right, tonal — like "1 Update" in the Apple menu.
                    let pill = container(txt(badge, mk::typo::CAPTION, p.on_surface))
                        .padding(iced::Padding {
                            left: mk::spacing::S,
                            right: mk::spacing::S,
                            top: 1.0,
                            bottom: 1.0,
                        })
                        .style(move |_| container::Style {
                            background: Some(
                                color(p.on_surface.over(p.surface_container_high, 0.10)).into(),
                            ),
                            border: iced::Border {
                                radius: 999.0_f32.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        });
                    let content = row![left, Space::new().width(Length::Fill), pill]
                        .align_y(iced::Alignment::Center);
                    // R66: the family button with the menu flash.
                    column_used = true;
                    column = column.push(magnifier(menu_item_button(
                        container(content)
                            .width(Length::Fill)
                            .padding([6, mk::spacing::S as u16])
                            .into(),
                        msg,
                        p,
                    )));
                }
                MenuItem::ItemWithShortcut { glyph, title, shortcut, color: tint, msg } => {
                    let f = tint.unwrap_or(p.on_surface);
                    let left: Element<'a, M> = match glyph {
                        Some(z) => row![
                            menu_glyph_circle(z, f, p),
                            txt(title, mk::typo::BODY, f),
                        ]
                        .spacing(mk::spacing::S)
                        .align_y(iced::Alignment::Center)
                        .into(),
                        None => txt(title, mk::typo::BODY, f).into(),
                    };
                    let content = row![
                        left,
                        Space::new().width(Length::Fill),
                        txt(shortcut, mk::typo::CAPTION, p.on_surface_variant),
                    ]
                    .align_y(iced::Alignment::Center);
                    // R66: the family button with the menu flash.
                    column_used = true;
                    column = column.push(magnifier(menu_item_button(
                        container(content)
                            .width(Length::Fill)
                            .padding([6, mk::spacing::S as u16])
                            .into(),
                        msg,
                        p,
                    )));
                }
                MenuItem::Item { glyph, title, color: tint, msg } => {
                    let f = tint.unwrap_or(p.on_surface);
                    let content: Element<'a, M> = match glyph {
                        Some(z) => row![
                            menu_glyph_circle(z, f, p),
                            txt(title, mk::typo::BODY, f),
                        ]
                        .spacing(mk::spacing::S)
                        .align_y(iced::Alignment::Center)
                        .into(),
                        None => txt(title, mk::typo::BODY, f).into(),
                    };
                    // R66: the family button with the menu flash.
                    column_used = true;
                    column = column.push(magnifier(menu_item_button(
                        container(content)
                            .width(Length::Fill)
                            .padding([6, mk::spacing::S as u16])
                            .into(),
                        msg,
                        p,
                    )));
                }
            }
        }
        if column_used {
            groups.push(column);
        }
        let mut stack = iced::widget::column![].spacing(mk::spacing::M);
        for g in groups {
            stack = stack.push(menu_shell(g.into(), MENU_WIDTH, p));
        }
        let stack: Element<'a, M> = stack.into();
        // On a compositor glass card the island pop stays OFF: the
        // glass body cannot scale with the content, so the shells must
        // cover the window exactly (user finding 23.07 — the frozen
        // 0.86 scale left a visible glass rim around the menu).
        if menu_glass_flag() && glass_on() {
            stack
        } else {
            super::island(stack, 0)
        }
    }

    /// HarnessFamily: the window navigation button (‹ › ⟳ …) — symbol
    /// + kit button style + magnifier. For headers (web navigation,
    /// history arrows) — inactive buttons are inert and magnifier-free.
    pub fn nav_button<'a, M: Clone + 'a>(
        glyph: char,
        active: bool,
        msg: M,
        p: mk::Palette,
    ) -> Element<'a, M> {
        let tint = if active {
            p.on_surface
        } else {
            p.on_surface_variant.with_alpha(0.4)
        };
        let mut k = iced::widget::button(symbol::<M>(
            glyph,
            mk::icon_size::SMALL + 2.0,
            tint,
        ))
        .padding([2, mk::spacing::S as u16])
        .style(move |_, status| panel::button_style(p, status, mk::radius::SMALL));
        if active {
            k = k.on_press(msg);
            magnifier(k)
        } else {
            k.into()
        }
    }
}

// ===========================================================================
// Image-tile family, pixel side (feature "bild"): the binding thumb
// recipe from the wallpaper section (R38/R44) — centred
// 16:9 crop, 448x252 CatmullRom, corners baked DIRECTLY into the image
// (iced does not clip raster images to rounded frames). The tile is
// shown at 112x63 — the 24px baked rounding acts as 6px there.
// ===========================================================================

pub mod image {
    /// IMAGE FITTED TO THE SCREEN (R209): decode `path` and shrink it so its
    /// longest side is at most `long` px (never enlarged). A wallpaper of
    /// 8400 × 5600 decoded whole is 188 MB — per surface; on a duo panel
    /// (1800 px) the fitted copy is ~10 MB. None if the file does not decode.
    pub fn fitted(path: &str, long: u32) -> Option<iced::widget::image::Handle> {
        let (w, h, rgba) = matrixkit_image::fitted_rgba(std::path::Path::new(path), long)?;
        Some(iced::widget::image::Handle::from_rgba(w, h, rgba))
    }

    /// Punch circular-arc corners with 1px feathering into the alpha channel.
    pub fn round_corners(image: &mut image::RgbaImage, radius: f32) {
        let (w, h) = image.dimensions();
        let r = radius.min(w as f32 / 2.0).min(h as f32 / 2.0);
        for y in 0..h {
            for x in 0..w {
                let fx = x as f32 + 0.5;
                let fy = y as f32 + 0.5;
                let dx = if fx < r {
                    r - fx
                } else if fx > w as f32 - r {
                    fx - (w as f32 - r)
                } else {
                    continue;
                };
                let dy = if fy < r {
                    r - fy
                } else if fy > h as f32 - r {
                    fy - (h as f32 - r)
                } else {
                    continue;
                };
                let d = (dx * dx + dy * dy).sqrt();
                if d > r {
                    image.get_pixel_mut(x, y).0[3] = 0;
                } else if d > r - 1.0 {
                    let old = image.get_pixel(x, y).0[3] as f32;
                    image.get_pixel_mut(x, y).0[3] = (old * (r - d)) as u8;
                }
            }
        }
    }

    /// The family thumb: 16:9 crop centered, 448x252, baked corners
    /// (24px). Returns (width, height, RGBA) for Handle::from_rgba.
    pub fn tile_thumb(image: image::DynamicImage) -> (u32, u32, Vec<u8>) {
        let (bw, bh) = (image.width(), image.height());
        let target_page = 16.0 / 9.0;
        let (cw, ch) = if (bw as f32 / bh.max(1) as f32) > target_page {
            (((bh as f32) * target_page) as u32, bh)
        } else {
            (bw, ((bw as f32) / target_page) as u32)
        };
        let cut = image.crop_imm((bw - cw) / 2, (bh - ch) / 2, cw.max(1), ch.max(1));
        let thumb = cut.resize_exact(448, 252, image::imageops::FilterType::CatmullRom);
        let mut rgba = thumb.to_rgba8();
        round_corners(&mut rgba, 24.0);
        let (w, h) = rgba.dimensions();
        (w, h, rgba.into_raw())
    }
}

// ---------------------------------------------------------------------------
// The control registry (R122 phase 2). The Zentrale no longer hard-wires
// its rows: every control the panel can host is DECLARED here with key,
// wording, category and glyph — the reference control center's "edit"
// flow (minus badges, add gallery with categories and search) works on
// this list. The user's choice and order live in the settings key
// `central-controls` (space-separated keys, live like bar-widgets).
// ---------------------------------------------------------------------------

/// The island-fleet family (R134): a dismiss veil plus one glass window
/// per island, extracted from the Zentrale. The fleet owns window ids,
/// the structure signature and the prefix-keep diff; the APP owns all
/// geometry knowledge and maps `Shell` records onto its layer-shell
/// messages — mkw stays layershell-agnostic.
pub mod fleet {
    use iced::Task;

    /// Which way the fleet lays its panes out. The fleet only ever
    /// advances ALONG this axis; the other axis is the app's business
    /// (`Spec::cross`), because only the app knows which screen edge
    /// it anchors to.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Flow {
        /// Stack downwards, pane after pane (control centre).
        Down,
        /// Lay out sideways, pane beside pane (dock shelf).
        Across,
    }

    /// Geometry the fleet asks the app to open a window with. The app
    /// translates this 1:1 into its NewLayerShellSettings (overlay
    /// layer, exclusive zone -1 by convention) and decides which edges
    /// `main`/`cross` count from.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Shell {
        pub width: u32,
        pub height: u32,
        /// Offset along the flow axis, already stacked by the fleet.
        pub main: i32,
        /// Offset across the flow axis — passed through from the spec.
        pub cross: i32,
        /// Glass namespace (matrix-tile, matrix-card, ...).
        pub namespace: &'static str,
        /// On-demand keyboard focus (search panes).
        pub keyboard: bool,
    }

    /// One pane's recipe in the app's plan.
    pub struct Spec<P> {
        pub pane: P,
        pub width: u32,
        pub height: u32,
        /// Offset across the flow axis (see `Shell::cross`).
        pub cross: i32,
        pub namespace: &'static str,
        pub keyboard: bool,
        /// Structure signature: equal prefixes keep their windows (and
        /// with them all in-flight widget state like morphs) alive.
        pub sig: (u8, u32),
    }

    pub struct Fleet<P> {
        veil: Option<iced::window::Id>,
        panes: Vec<(iced::window::Id, P)>,
        /// The geometry each pane was opened with, parallel to `panes`.
        /// The app needs it to move ONE surface (a pane growing under
        /// the pointer) without recomputing the whole plan.
        shells: Vec<Shell>,
        sig: Vec<(u8, u32)>,
        /// First pane's offset along the flow, and the gap between panes.
        start: i32,
        gap: i32,
        flow: Flow,
    }

    impl Shell {
        /// The layer surface this pane opens as (R209 family — dock and bar
        /// spelled it out each): `anchor` names the corner the fleet hangs
        /// from; `cross` goes to the anchored top/bottom edge, `main` to the
        /// anchored left/right edge. Top layer, never reserving, keyboard only
        /// for search panes.
        pub fn layer_settings(
            &self,
            anchor: crate::iced_layershell::reexport::Anchor,
            output: crate::iced_layershell::reexport::OutputOption,
        ) -> crate::iced_layershell::reexport::NewLayerShellSettings {
            use crate::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
            let at = |edge: Anchor, value: i32| if anchor.contains(edge) { value } else { 0 };
            NewLayerShellSettings {
                size: Some((self.width, self.height)),
                anchor,
                // (top, right, bottom, left)
                margin: Some((
                    at(Anchor::Top, self.cross),
                    at(Anchor::Right, self.main),
                    at(Anchor::Bottom, self.cross),
                    at(Anchor::Left, self.main),
                )),
                layer: Layer::Top,
                keyboard_interactivity: if self.keyboard {
                    KeyboardInteractivity::OnDemand
                } else {
                    KeyboardInteractivity::None
                },
                exclusive_zone: Some(-1),
                namespace: Some(String::from(self.namespace)),
                output_option: output,
                ..Default::default()
            }
        }
    }

    impl<P: Clone> Fleet<P> {
        /// A downward fleet — the original shape (control centre).
        pub fn new(top: i32, gap: i32) -> Self {
            Self::flowing(top, gap, Flow::Down)
        }

        /// A sideways fleet — panes side by side (dock shelf).
        pub fn across(left: i32, gap: i32) -> Self {
            Self::flowing(left, gap, Flow::Across)
        }

        pub fn flowing(start: i32, gap: i32, flow: Flow) -> Self {
            Fleet {
                veil: None,
                panes: Vec::new(),
                shells: Vec::new(),
                sig: Vec::new(),
                start,
                gap,
                flow,
            }
        }

        /// Record the dismiss veil the app opened.
        pub fn set_veil(&mut self, id: iced::window::Id) {
            self.veil = Some(id);
        }

        pub fn is_veil(&self, id: iced::window::Id) -> bool {
            self.veil == Some(id)
        }

        /// The pane shown in this window — None for the veil and for
        /// windows on their way out.
        pub fn pane(&self, id: iced::window::Id) -> Option<&P> {
            self.panes.iter().find(|(w, _)| *w == id).map(|(_, p)| p)
        }

        /// The geometry this surface was opened with — the anchor for
        /// any per-surface animation the app wants to run.
        /// All surfaces of the fleet, in plan order. R192: whoever
        /// wants to move the fleet AFTERWARDS (scrolling,
        /// entrance) needs the list — reopening would flicker
        /// and kill every running animation.
        pub fn ids(&self) -> Vec<iced::window::Id> {
            self.panes.iter().map(|(id, _)| *id).collect()
        }

        pub fn shell_of(&self, id: iced::window::Id) -> Option<Shell> {
            let n = self.panes.iter().position(|(w, _)| *w == id)?;
            self.shells.get(n).copied()
        }

        /// Diff the plan against the fleet: the common structural
        /// prefix keeps its windows (positions cannot shift — later
        /// panes only stack below), the rest is closed and reopened.
        /// `open`/`close` map to the app's layer-shell messages.
        pub fn sync<M: 'static + Send>(
            &mut self,
            plan: Vec<Spec<P>>,
            open: impl Fn(Shell, iced::window::Id) -> Task<M>,
            close: impl Fn(iced::window::Id) -> Task<M>,
        ) -> Task<M> {
            let sig: Vec<(u8, u32)> = plan.iter().map(|s| s.sig).collect();
            let keep = sig
                .iter()
                .zip(self.sig.iter())
                .take_while(|(a, b)| a == b)
                .count();
            if keep == sig.len() && keep == self.sig.len() {
                for (slot, spec) in self.panes.iter_mut().zip(plan.into_iter()) {
                    slot.1 = spec.pane;
                }
                return Task::none();
            }
            self.sig = sig;
            let mut tasks: Vec<Task<M>> = self
                .panes
                .split_off(keep)
                .into_iter()
                .map(|(id, _)| close(id))
                .collect();
            self.shells.truncate(keep);
            let mut plan = plan.into_iter();
            let (flow, gap) = (self.flow, self.gap);
            let step = move |spec: &Spec<P>| {
                (match flow {
                    Flow::Down => spec.height,
                    Flow::Across => spec.width,
                }) as i32
                    + gap
            };
            let mut at = self.start;
            for n in 0..keep {
                let spec = plan.next().expect("prefix within plan");
                at += step(&spec);
                self.panes[n].1 = spec.pane;
            }
            for spec in plan {
                let id = iced::window::Id::unique();
                let shell = Shell {
                    width: spec.width,
                    height: spec.height,
                    main: at,
                    cross: spec.cross,
                    namespace: spec.namespace,
                    keyboard: spec.keyboard,
                };
                tasks.push(open(shell, id));
                at += step(&spec);
                self.panes.push((id, spec.pane));
                self.shells.push(shell);
            }
            Task::batch(tasks)
        }
    }
}

/// The event family (R135): system state FLOWS to the app instead of
/// being polled — Apple's zero-polling doctrine (WiFi delegates,
/// CoreAudio listeners; the Bento dig found no "polling" in the whole
/// reference control center). A watcher tails a monitor command and
/// yields one unit per burst of matching lines; the app re-reads the
/// state it cares about on each event: the listener fires, you query.
pub mod events {
    use std::io::BufRead as _;

    /// Watch ONE file and emit only when it actually changes.
    ///
    /// The polling stays in a background thread — a stat() every
    /// `period` costs nothing, whereas one MESSAGE per period costs a
    /// full redraw of every window the app owns. Since R174 the dock
    /// owns seventeen of them, which is how a harmless 150 ms
    /// heartbeat turned into 59 % of a CPU.
    pub fn file_changes(
        id: &'static str,
        path: String,
        period: std::time::Duration,
    ) -> iced::Subscription<()> {
        iced::Subscription::run_with((id, path, period), |input| {
            let (_, path, period) = input.clone();
            let (tx, rx) = futures::channel::mpsc::unbounded();
            std::thread::spawn(move || {
                let stamp = || std::fs::metadata(&path).and_then(|m| m.modified()).ok();
                let mut last = stamp();
                loop {
                    std::thread::sleep(period);
                    let now = stamp();
                    if now != last {
                        last = now;
                        if tx.unbounded_send(()).is_err() {
                            return; // App gone — die with it.
                        }
                    }
                }
            });
            rx
        })
    }

    /// Tail `program args` and emit once per burst (leading edge,
    /// 150 ms coalescing — pw-mon prints dozens of lines per change).
    /// A dying monitor restarts after 2 s; a missing tool stays
    /// silent — the caller keeps a slow fallback tick for that.
    pub fn watch(
        id: &'static str,
        program: &'static str,
        args: &'static [&'static str],
        filter: fn(&str) -> bool,
    ) -> iced::Subscription<()> {
        iced::Subscription::run_with((id, program, args, filter), make_watcher)
    }

    fn make_watcher(
        input: &(
            &'static str,
            &'static str,
            &'static [&'static str],
            fn(&str) -> bool,
        ),
    ) -> impl futures::Stream<Item = ()> {
        let (_, program, args, filter) = *input;
        let (tx, rx) = futures::channel::mpsc::unbounded();
        std::thread::spawn(move || loop {
            let child = std::process::Command::new(program)
                .args(args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn();
            let Ok(mut child) = child else {
                return; // Tool missing — the fallback tick covers it.
            };
            let Some(out) = child.stdout.take() else { return };
            let reader = std::io::BufReader::new(out);
            let mut last = std::time::Instant::now() - std::time::Duration::from_secs(1);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                if !filter(&line) {
                    continue;
                }
                if last.elapsed() >= std::time::Duration::from_millis(150) {
                    last = std::time::Instant::now();
                    if tx.unbounded_send(()).is_err() {
                        matrixkit_theme::process::end(&mut child); // round 48: kill AND reap — no zombie
                        return; // App gone — die with it.
                    }
                }
            }
            matrixkit_theme::process::end(&mut child); // round 48: kill AND reap — no zombie
            if tx.is_closed() {
                return;
            }
            // Monitor ended (audio server restart?) — try again shortly.
            std::thread::sleep(std::time::Duration::from_secs(2));
        });
        rx
    }
}

/// Moved into the iced-free kit (round 39): `matrixkit_scene::registry`.
pub use matrixkit_scene::registry;

/// Moved into the iced-free kit (round 39): `matrixkit_scene::central` —
/// the house top bar's control card reads it without iced.
pub use matrixkit_scene::central;

// ──────────────────── The touch round (R168) ────────────────────
// Dug from the living UIKit headers (CLT SDK 26.2, iOSSupport):
// UILongPressGestureRecognizer holds 0.5 s within 10 pt of drift, and
// once recognized the movement is free (mk::touch carries the
// numbers). The LIFT is the reference's visual promise — while the
// press charges, the element rises slightly: "I am holding a menu."

/// Right-click AND long-press open the same context menu — ONE handle
/// for both input worlds. The message carries the anchor point in
/// window coordinates, ready for `context_menu(open_at: ...)`.
/// R188: THE clickable piece of text. A real `button` underneath —
/// which is the whole point: `press_area` listens for a touch press
/// and hands mouse events to its child, so a bare text child never
/// sees a click. That defect hit the dock tiles this morning (R179)
/// and came straight back in everything built after it, because
/// `press_area` reads like the obvious tool. It is not. Anything a
/// finger AND a mouse must hit uses this.
///
/// Invisible by default: no background, no border — the caller's
/// text is the whole appearance. The magnifier comes along, so the
/// thing lifts under the pointer like every other target in the kit.
pub fn text_button<'a, M: Clone + 'a>(content: impl Into<Element<'a, M>>, msg: M) -> Element<'a, M> {
    magnifier(
        iced::widget::button(content)
            .padding(0)
            .style(|_, _| iced::widget::button::Style::default())
            .on_press(msg),
    )
}

/// The QUIET text button: the same grammar as `text_button`, but
/// without the magnifier. For buttons IN THE READING FLOW — the owner (28.08.2026): text
/// that grows under the pointer is irritating while reading. The magnifier belongs
/// on controls, not in paragraphs; the oracle therefore speaks
/// this variant throughout.
pub fn text_button_ruhig<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    msg: M,
) -> Element<'a, M> {
    iced::widget::button(content)
        .padding(0)
        .style(|_, _| iced::widget::button::Style::default())
        .on_press(msg)
        .into()
}

/// R188: one tab of a tab strip — a real tile, so the active one is
/// visible at a glance instead of only differently coloured. Title
/// and the × are separate targets; both are real buttons.
pub fn tab_chip<'a, M: Clone + 'a>(
    label: String,
    active: bool,
    on_pick: M,
    on_close: M,
    p: mk::Palette,
) -> Element<'a, M> {
    // The active tab wears the primary container — the same pill the
    // sidebar uses for "you are here". The others stay flat, so the
    // strip reads as one row with one raised card.
    let (bg, ink) = if active {
        (p.primary_container, p.on_primary_container)
    } else {
        (p.on_surface.over(p.surface_container, 0.06), p.on_surface_variant)
    };
    let body = row![
        text_button(txt(label, mk::typo::CAPTION, ink), on_pick),
        Space::new().width(mk::spacing::S),
        text_button(txt("×", mk::typo::CAPTION, ink), on_close),
    ]
    .align_y(iced::Alignment::Center);
    container(body)
        .padding(iced::Padding {
            left: mk::spacing::S as f32,
            right: mk::spacing::S as f32,
            top: 4.0,
            bottom: 4.0,
        })
        .style(move |_| iced::widget::container::Style {
            background: Some(color(bg).into()),
            border: iced::Border {
                radius: mk::radius::SMALL.into(),
                width: 0.0,
                color: color(bg),
            },
            ..Default::default()
        })
        .into()
}

pub fn press_area<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    on_open: impl Fn(iced::Point) -> M + 'a,
) -> Element<'a, M> {
    Element::new(PressArea { content: content.into(), on_open: Box::new(on_open) })
}

struct PressArea<'a, M> {
    content: Element<'a, M>,
    on_open: Box<dyn Fn(iced::Point) -> M + 'a>,
}

struct PressState {
    finger: Option<iced::touch::Finger>,
    start: iced::Point,
    began: Option<std::time::Instant>,
    fired: bool,
    lift: mk::motion::Spring,
    last: Option<std::time::Instant>,
}

impl Default for PressState {
    fn default() -> Self {
        Self {
            finger: None,
            start: iced::Point::ORIGIN,
            began: None,
            fired: false,
            lift: mk::motion::Spring::new(1.0),
            last: None,
        }
    }
}

impl<'a, M: Clone> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for PressArea<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<PressState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(PressState::default())
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        use iced::touch::Event as T;
        match event {
            // The mouse world: right click IS the long press.
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right)) => {
                if let Some(pos) = cursor.position_over(layout.bounds()) {
                    shell.publish((self.on_open)(pos));
                    shell.capture_event();
                    return;
                }
            }
            iced::Event::Touch(t) => {
                let z = tree.state.downcast_mut::<PressState>();
                match t {
                    T::FingerPressed { id, position } => {
                        if z.finger.is_none() && layout.bounds().contains(*position) {
                            z.finger = Some(*id);
                            z.start = *position;
                            z.began = Some(std::time::Instant::now());
                            z.fired = false;
                            shell.request_redraw();
                        }
                        // pass through — a tap should act normally
                    }
                    T::FingerMoved { id, position } if z.finger == Some(*id) => {
                        let moved = position.distance(z.start);
                        if !z.fired && moved > mk::touch::SLOP {
                            // Drifted before recognition: the press dies,
                            // the element settles back.
                            z.began = None;
                            z.lift.retarget(1.0);
                            shell.request_redraw();
                        }
                    }
                    T::FingerLifted { id, .. } | T::FingerLost { id, .. }
                        if z.finger == Some(*id) =>
                    {
                        let fired = z.fired;
                        z.finger = None;
                        z.began = None;
                        z.fired = false;
                        z.lift.retarget(1.0);
                        shell.request_redraw();
                        if fired {
                            // The menu already opened; the child was
                            // canceled at recognition.
                            shell.capture_event();
                            return;
                        }
                    }
                    _ => {}
                }
            }
            iced::Event::Window(iced::window::Event::RedrawRequested(now)) => {
                let z = tree.state.downcast_mut::<PressState>();
                let dt = z
                    .last
                    .map(|l| now.duration_since(l).as_secs_f32())
                    .unwrap_or(0.0)
                    .clamp(0.0, 0.05);
                z.last = Some(*now);
                let springing = z.lift.tick(dt);
                if let Some(began) = z.began {
                    let held = began.elapsed().as_secs_f32();
                    if !z.fired && held >= mk::touch::LIFT_DELAY_SECS && !mk::motion_reduced() {
                        z.lift.retarget(mk::touch::LIFT_SCALE);
                    }
                    if !z.fired && held >= mk::touch::LONG_PRESS_SECS {
                        z.fired = true;
                        z.began = None;
                        z.lift.retarget(1.0);
                        // UIScrollView cancel: the child loses the finger
                        // so the press does not also click.
                        if let Some(id) = z.finger {
                            let start = z.start;
                            let lost = iced::Event::Touch(T::FingerLost { id, position: start });
                            self.content.as_widget_mut().update(
                                &mut tree.children[0],
                                &lost,
                                layout,
                                cursor,
                                renderer,
                                clipboard,
                                shell,
                                viewport,
                            );
                        }
                        shell.publish((self.on_open)(z.start));
                    }
                    shell.request_redraw();
                } else if springing {
                    shell.request_redraw();
                }
            }
            _ => {}
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let s = tree.state.downcast_ref::<PressState>().lift.value;
        let draw = |r: &mut iced::Renderer| {
            self.content
                .as_widget()
                .draw(&tree.children[0], r, theme, style, layout, cursor, viewport);
        };
        if (s - 1.0).abs() <= 0.001 {
            draw(renderer);
        } else {
            let b = layout.bounds();
            let (cx, cy) = (b.center_x(), b.center_y());
            let t = iced::Transformation::translate((1.0 - s) * cx, (1.0 - s) * cy)
                * iced::Transformation::scale(s);
            renderer.with_transformation(t, draw);
        }
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// Touch for text: translates finger events into the synthetic mouse
/// language the text widgets already speak — tap places the cursor,
/// drag selects, a long press selects the word (a synthetic double
/// click, so iced's own word logic answers). `on_menu` fires when a
/// selection gesture ends or a long press lands: the moment for the
/// edit pill.
pub fn touch_text<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    on_menu: impl Fn(iced::Point) -> M + 'a,
) -> Element<'a, M> {
    Element::new(TouchText { content: content.into(), on_menu: Box::new(on_menu) })
}

struct TouchText<'a, M> {
    content: Element<'a, M>,
    on_menu: Box<dyn Fn(iced::Point) -> M + 'a>,
}

#[derive(Default)]
struct TouchTextState {
    finger: Option<iced::touch::Finger>,
    start: iced::Point,
    at: iced::Point,
    began: Option<std::time::Instant>,
    moved: bool,
    fired: bool,
}

impl<'a, M: Clone> TouchText<'a, M> {
    /// Feed the child a synthetic mouse event, with the cursor placed
    /// at the finger (text widgets read the CURSOR, not the event).
    #[allow(clippy::too_many_arguments)]
    fn synth(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: iced::mouse::Event,
        at: iced::Point,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        let cursor = iced::advanced::mouse::Cursor::Available(at);
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            &iced::Event::Mouse(event),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
}

impl<'a, M: Clone> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for TouchText<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<TouchTextState>()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(TouchTextState::default())
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        use iced::mouse::{Button, Event as Mo};
        use iced::touch::Event as T;
        match event {
            iced::Event::Touch(t) => {
                let z = tree.state.downcast_mut::<TouchTextState>();
                match t {
                    T::FingerPressed { id, position } => {
                        if z.finger.is_none() && layout.bounds().contains(*position) {
                            *z = TouchTextState {
                                finger: Some(*id),
                                start: *position,
                                at: *position,
                                began: Some(std::time::Instant::now()),
                                moved: false,
                                fired: false,
                            };
                            let at = *position;
                            self.synth(tree, Mo::CursorMoved { position: at }, at, layout, renderer, clipboard, shell, viewport);
                            self.synth(tree, Mo::ButtonPressed(Button::Left), at, layout, renderer, clipboard, shell, viewport);
                            shell.capture_event();
                            shell.request_redraw();
                            return;
                        }
                    }
                    T::FingerMoved { id, position } => {
                        if z.finger == Some(*id) {
                            z.at = *position;
                            if !z.moved && position.distance(z.start) > mk::touch::SLOP {
                                z.moved = true;
                                z.began = None;
                            }
                            if z.moved && !z.fired {
                                let at = *position;
                                self.synth(tree, Mo::CursorMoved { position: at }, at, layout, renderer, clipboard, shell, viewport);
                            }
                            shell.capture_event();
                            return;
                        }
                    }
                    T::FingerLifted { id, position } | T::FingerLost { id, position } => {
                        if z.finger == Some(*id) {
                            let was_selection = z.moved && !z.fired;
                            let fired = z.fired;
                            let at = *position;
                            *z = TouchTextState::default();
                            if !fired {
                                self.synth(tree, Mo::ButtonReleased(Button::Left), at, layout, renderer, clipboard, shell, viewport);
                            }
                            if was_selection {
                                shell.publish((self.on_menu)(at));
                            }
                            shell.capture_event();
                            return;
                        }
                    }
                }
            }
            iced::Event::Window(iced::window::Event::RedrawRequested(_)) => {
                let z = tree.state.downcast_mut::<TouchTextState>();
                if let Some(began) = z.began {
                    if !z.fired && began.elapsed().as_secs_f32() >= mk::touch::LONG_PRESS_SECS {
                        z.fired = true;
                        z.began = None;
                        let at = z.start;
                        // A synthetic double click: iced's own word
                        // selection answers (reference: long press
                        // selects the word under the finger).
                        self.synth(tree, Mo::ButtonReleased(Button::Left), at, layout, renderer, clipboard, shell, viewport);
                        self.synth(tree, Mo::ButtonPressed(Button::Left), at, layout, renderer, clipboard, shell, viewport);
                        self.synth(tree, Mo::ButtonReleased(Button::Left), at, layout, renderer, clipboard, shell, viewport);
                        shell.publish((self.on_menu)(at));
                    } else {
                        shell.request_redraw();
                    }
                }
            }
            _ => {}
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

/// The floating edit pill (UIEditMenuInteraction extract): horizontal
/// actions hovering ABOVE the selection, flipping below near the top
/// edge — the second menu language beside the context menu, reserved
/// for text moments (copy, paste, transform).
pub fn edit_menu<'a, M: Clone + 'a>(
    content: Element<'a, M>,
    actions: Vec<(String, M)>,
    open_at: Option<iced::Point>,
    on_close: M,
    p: mk::Palette,
) -> Element<'a, M> {
    use iced::widget::stack;
    let Some(pos) = open_at else { return content };

    let veil = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill)),
    )
    .on_press(on_close.clone())
    .on_right_press(on_close);

    // Estimated pill width for edge clamping (BODY ≈ 7.2 px/char + pads).
    let est: f32 = actions
        .iter()
        .map(|(label, _)| label.chars().count() as f32 * 7.2 + 2.0 * mk::spacing::M)
        .sum::<f32>();
    let height = 36.0;

    let placed = iced::widget::responsive(move |area| {
        let mut lane = row![].align_y(iced::Alignment::Center);
        for (i, (label, msg)) in actions.iter().enumerate() {
            if i > 0 {
                lane = lane.push(
                    container(Space::new().width(Length::Fixed(1.0)).height(Length::Fixed(16.0)))
                        .style(move |_| iced::widget::container::Style {
                            background: Some(color(p.outline.over(p.surface_container_high, 0.35)).into()),
                            ..Default::default()
                        }),
                );
            }
            lane = lane.push(instant_button(
                container(txt(label.clone(), mk::typo::BODY, p.on_surface))
                    .padding([8.0, mk::spacing::M]),
                msg.clone(),
            ));
        }
        let pill = container(lane).style(move |_| iced::widget::container::Style {
            background: Some(color(p.surface_container_high).into()),
            border: iced::Border {
                radius: mk::radius::capsule(height).into(),
                width: 1.0,
                color: color(p.outline.over(p.surface_container_high, 0.3)),
            },
            shadow: elevation::floating(),
            ..Default::default()
        });
        let x = (pos.x - est / 2.0).clamp(mk::spacing::S, (area.width - est - mk::spacing::S).max(0.0));
        // Above the anchor; below it when the top edge is near.
        let mut y = pos.y - height - 12.0;
        if y < mk::spacing::S {
            y = pos.y + 20.0;
        }
        container(pill)
            .padding(iced::Padding { left: x, top: y, ..iced::Padding::ZERO })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    });

    stack![content, veil, placed].into()
}

/// Slide-in (R168e): draws its content shifted by `offset` pixels on
/// the y axis — the wake animation of auto-hidden surfaces. The bar
/// floats DOWN from the top edge (negative offset fading to 0), the
/// dock floats UP from the bottom (positive offset). Pure drawing:
/// layout and events stay untouched.
pub fn slide_in<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    offset: f32,
) -> Element<'a, M> {
    Element::new(SlideIn { content: content.into(), offset })
}

struct SlideIn<'a, M> {
    content: Element<'a, M>,
    offset: f32,
}

impl<'a, M: Clone> iced::advanced::Widget<M, iced::Theme, iced::Renderer> for SlideIn<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::stateless()
    }
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }
    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, M>,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let draw = |r: &mut iced::Renderer| {
            self.content
                .as_widget()
                .draw(&tree.children[0], r, theme, style, layout, cursor, viewport);
        };
        if self.offset.abs() < 0.5 {
            draw(renderer);
        } else {
            renderer.with_translation(iced::Vector::new(0.0, self.offset), draw);
        }
    }
    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

// ═══════════════════════════════════════════════════════════════
// R172: THE TEXT TILE — an app announces itself in words, not a
// picture. Two lines carry the identity, and everything the app
// wants to offer grows UNDERNEATH them.
//
// Why the name is on top and the maker below (and not the other
// way round): the first, boldest line is the one the eye reads
// before it has decided to read anything. If that line says
// "Matrix" on every tile, the most prominent thing on screen is
// the thing that distinguishes nothing. The name sorts, the maker
// qualifies — and for a foreign app the second line finally says
// something true ("Westwood", not "Matrix").
//
// The widget slot is the second half of the idea: a tile is not a
// launcher, it is a small surface the app (or a third-party plug-
// in) may furnish — shortcut chips into a specific place, a live
// number, a spark of data. Nothing is drawn there unless someone
// asked for it, so an unfurnished tile stays exactly two lines
// tall.

/// One shortcut inside a tile: a word that goes somewhere precise.
pub fn tile_chip<'a, M: Clone + 'a>(label: &'a str, msg: M, p: mk::Palette) -> Element<'a, M> {
    instant_button(
        container(txt(label, mk::typo::CAPTION, p.on_surface))
            .padding([mk::spacing::XS as u16 / 2, mk::spacing::S as u16])
            .style(move |_| container::Style {
                background: Some(color(p.on_surface.over(p.surface_container, 0.10)).into()),
                border: iced::Border {
                    radius: mk::radius::capsule(22.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        msg,
    )
}

/// One furnished row inside a tile. Whatever the app or a plug-in
/// puts here is CLIPPED to the tile and scrolls sideways — the same
/// gesture the dock row uses, so nothing can ever stick out past
/// the tile's edge (found on the Surface: a second chip hung over
/// the border because the row simply grew).
pub fn tile_row<'a, M: Clone + 'a>(items: Vec<Element<'a, M>>, width: f32) -> Element<'a, M> {
    let mut line = iced::widget::row![].spacing(mk::spacing::XS);
    for item in items {
        line = line.push(item);
    }
    container(
        iced::widget::scrollable(line)
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::new()
                    .width(0)
                    .scroller_width(0)
                    .margin(0),
            ))
            .width(Length::Shrink),
    )
    .max_width(width)
    .clip(true)
    .into()
}

/// The tile itself — the successor to the app icon (R172, decided
/// by looking at both arrangements on a Surface): **the name leads,
/// the origin qualifies below it.** With the maker on top, the
/// boldest line on every tile of a first-party system reads the
/// same word, and the most prominent thing on screen distinguishes
/// nothing.
///
/// `rows` is what the app or a plug-in furnished — an empty vector
/// means "just the two lines". Each row scrolls on its own, so a
/// tile can carry more shortcuts than it is wide.
///
/// `on_menu` is the long press: the tile's own context menu (pin it
/// to the bottom edge, remove it, …), delivered through the kit's
/// press_area so touch and right-click behave alike.
pub fn app_tile<'a, M: Clone + 'a>(
    name: impl iced::widget::text::IntoFragment<'a>,
    maker: impl iced::widget::text::IntoFragment<'a>,
    rows: Vec<Element<'a, M>>,
    on_open: M,
    on_menu: Option<impl Fn(iced::Point) -> M + 'a>,
    p: mk::Palette,
) -> Element<'a, M> {
    app_tile_sized(name, maker, rows, on_open, on_menu, TILE_WIDTH, p)
}

/// The same tile at a chosen width — the dock runs narrower than a
/// launcher grid, and a tile must not force one metric on both.
pub fn app_tile_sized<'a, M: Clone + 'a>(
    name: impl iced::widget::text::IntoFragment<'a>,
    maker: impl iced::widget::text::IntoFragment<'a>,
    rows: Vec<Element<'a, M>>,
    on_open: M,
    on_menu: Option<impl Fn(iced::Point) -> M + 'a>,
    width: f32,
    p: mk::Palette,
) -> Element<'a, M> {
    // Only the two lines open the app. A shortcut in a row must
    // never be a mis-tap on the app itself.
    let head: Element<'a, M> = instant_button(
        container(
            iced::widget::column![
                txt(name, mk::typo::SUBTITLE, p.on_surface),
                txt(maker, mk::typo::CAPTION, p.on_surface_variant),
            ]
            .spacing(1),
        )
        .width(Length::Fill),
        on_open,
    );

    let mut stack = iced::widget::column![head].spacing(mk::spacing::XS);
    for row in rows {
        stack = stack.push(row);
    }

    let body = container(stack)
        .padding(mk::spacing::M)
        .width(Length::Fixed(width))
        .clip(true)
        .style(move |_| container::Style {
            background: Some(color(p.surface_container).into()),
            border: iced::Border {
                radius: mk::radius::LARGE.into(),
                width: 1.0,
                color: color(p.outline.over(p.surface_container, 0.25)),
            },
            ..Default::default()
        });

    let element: Element<'a, M> = body.into();
    match on_menu {
        Some(open) => press_area(element, open),
        None => element,
    }
}

/// The glass body a free-floating thing wears in the shelf-less
/// dock — ONE recipe, so a tile and a widget are visibly the same
/// kind of object (user: "the widgets need the same outline").
pub fn tile_chrome<'a, M: 'a>(content: Element<'a, M>, p: mk::Palette) -> Element<'a, M> {
    container(content)
        .padding(iced::Padding {
            top: mk::spacing::S,
            bottom: mk::spacing::S,
            left: mk::spacing::M,
            right: mk::spacing::M,
        })
        .style(move |_| container::Style {
            background: Some(color(p.surface_container.with_alpha(0.62)).into()),
            border: iced::Border {
                radius: mk::radius::LARGE.into(),
                width: 1.0,
                color: color(p.on_surface.with_alpha(0.14)),
            },
            ..Default::default()
        })
        .into()
}

/// The same body WITHOUT padding — for content that already brings
/// its own (the dock widgets do). Wrapping them in the padded
/// version stacks two paddings and the glass balloons around a tiny
/// glyph (user, on the Surface).
pub fn tile_chrome_bare<'a, M: 'a>(content: Element<'a, M>, p: mk::Palette) -> Element<'a, M> {
    container(content)
        .style(move |_| container::Style {
            background: Some(color(p.surface_container.with_alpha(0.62)).into()),
            border: iced::Border {
                radius: mk::radius::LARGE.into(),
                width: 1.0,
                color: color(p.on_surface.with_alpha(0.14)),
            },
            ..Default::default()
        })
        .into()
}

/// A free-floating tile: as wide as its words, no wider (user, on
/// the Surface — "Dateien" wasted half its tile while "Einstellungen"
/// was cramped). It wears glass itself, because in the shelf-less
/// dock there is no pane behind it to carry the material.
pub fn app_tile_auto<'a, M: Clone + 'a>(
    name: impl iced::widget::text::IntoFragment<'a>,
    maker: impl iced::widget::text::IntoFragment<'a>,
    on_open: M,
    on_menu: Option<impl Fn(iced::Point) -> M + 'a>,
    p: mk::Palette,
) -> Element<'a, M> {
    // Same correction as app_tile_naked: a real button, because
    // instant_button ignores the mouse and the child is bare text.
    let head: Element<'a, M> = iced::widget::button(
        iced::widget::column![
            txt(name, mk::typo::SUBTITLE, p.on_surface),
            txt(maker, mk::typo::CAPTION, p.on_surface_variant),
        ]
        .spacing(1),
    )
    .padding(0)
    .style(|_, _| iced::widget::button::Style::default())
    .on_press(on_open)
    .into();

    let element: Element<'a, M> = tile_chrome(head, p);
    match on_menu {
        Some(open) => press_area(element, open),
        None => element,
    }
}

/// The tile's face with the SAME air as `tile_chrome` but no painted
/// body: for a tile that is its own layer-shell surface, where the
/// compositor lays the real glass underneath (R174). `tile_chrome`
/// stays the fallback for tiles living inside someone else's surface
/// (previews, screenshots, the icon dock).
pub fn tile_chrome_naked<'a, M: 'a>(content: Element<'a, M>) -> Element<'a, M> {
    container(content)
        .padding(iced::Padding {
            top: mk::spacing::S,
            bottom: mk::spacing::S,
            left: mk::spacing::M,
            right: mk::spacing::M,
        })
        .into()
}

/// `app_tile_auto` without the painted body — the windowed twin.
///
/// R209 (Duo, «the tiles must feel better when I start them»): the WHOLE
/// tile is the button — before only the text was, and a finger on the
/// tile's air did nothing. Pressed, the tile darkens under the finger;
/// `glow` (0..1) lights its edge in the accent while the app starts or
/// right after a tap, so the answer is visible the moment the finger lifts.
pub fn app_tile_naked<'a, M: Clone + 'a>(
    name: impl iced::widget::text::IntoFragment<'a>,
    maker: impl iced::widget::text::IntoFragment<'a>,
    lines: Vec<String>,
    on_open: M,
    on_menu: Option<impl Fn(iced::Point) -> M + 'a>,
    glow: f32,
    p: mk::Palette,
) -> Element<'a, M> {
    // A REAL (invisible) button, not `instant_button`: the instant
    // variant fires only for TOUCH and passes the mouse through to its
    // child — which here is bare text, so mouse clicks died unheard
    // (user, on the Surface: touch worked, mouse did not). Release-fire
    // is the reference dock grammar anyway.
    let mut body = iced::widget::column![
        txt(name, mk::typo::SUBTITLE, p.on_surface),
        txt(maker, mk::typo::CAPTION, p.on_surface_variant),
    ]
    .spacing(1);
    // R174j (user): the tile names its WINDOWS — one caption line per
    // open window, so a full shelf reads like a window list.
    for line in lines {
        body = body.push(txt(line, mk::typo::CAPTION, p.on_surface_variant));
    }
    let element = chip_button(tile_chrome_naked(body.into()), on_open, glow, p);
    match on_menu {
        Some(open) => press_area(element, open),
        None => element,
    }
}

/// CHIP BUTTON (R209): a glass chip that IS its button — the dock tile's
/// touch answer for every chip (dock tiles, bar chips): the whole surface
/// darkens while pressed and its edge lights up by `glow` (0..1, the app
/// fades it out after the tap, `mk::motion::tap_glow`). The button fills
/// its surface, so the lit edge IS the glass edge.
pub fn chip_button<'a, M: Clone + 'a>(
    face: impl Into<Element<'a, M>>,
    on_press: M,
    glow: f32,
    p: mk::Palette,
) -> Element<'a, M> {
    let glow = glow.clamp(0.0, 1.0);
    let face = container(face).center_x(Length::Fill).center_y(Length::Fill);
    iced::widget::button(face)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .style(move |_, status| {
            let pressed = matches!(status, iced::widget::button::Status::Pressed);
            iced::widget::button::Style {
                background: pressed.then(|| color(p.on_surface.with_alpha(0.14)).into()),
                border: iced::Border {
                    radius: mk::radius::LARGE.into(),
                    width: if glow > 0.01 { 1.0 + glow } else { 0.0 },
                    color: color(p.primary.with_alpha(0.3 + 0.6 * glow)),
                },
                ..Default::default()
            }
        })
        .on_press(on_press)
        .into()
}

/// How wide this text runs, ESTIMATED — no renderer required.
///
/// A layer-shell surface must state its size before anything is laid
/// out, so a tile that is its own window cannot ask the text shaper
/// how wide it turned out. The estimate uses per-class advances for
/// Inter (measured against rendered strings): narrow letters, digits,
/// wide letters, capitals. It is deliberately generous — a few pixels
/// too much show up as a hair more side air, whereas too little would
/// clip the word.
pub fn text_advance(text: &str, size: f32) -> f32 {
    let em = text
        .chars()
        .map(|c| match c {
            'i' | 'l' | 'j' | 'I' | '.' | ',' | ':' | ';' | '\'' | '!' | '|' => 0.28,
            'f' | 't' | 'r' | '(' | ')' | '[' | ']' | '-' | ' ' => 0.36,
            'm' | 'w' | 'M' | 'W' => 0.90,
            c if c.is_ascii_uppercase() => 0.68,
            c if c.is_ascii_digit() => 0.58,
            c if c.is_ascii_lowercase() => 0.54,
            // Umlauts and anything else: assume a full lowercase body.
            _ => 0.58,
        })
        .sum::<f32>();
    em * size
}

/// The width a two-line tile wants as its own window: the longer of
/// its two lines plus the chrome's air on both sides.
pub fn tile_auto_width(name: &str, maker: &str) -> f32 {
    let head = text_advance(name, mk::typo::SUBTITLE.size);
    let foot = text_advance(maker, mk::typo::CAPTION.size);
    head.max(foot) + 2.0 * mk::spacing::M + TEXT_SAFETY
}

/// Headroom on top of an estimated text width. An estimate that falls
/// short CUTS A WORD (seen on the Surface: "Einstellungen" lost its
/// last letter); one that overshoots costs a little air. So it always
/// overshoots.
pub const TEXT_SAFETY: f32 = 10.0;

/// Tile width in a grid (launcher, home screen); the inner room a
/// furnished row may use is this minus the padding on both sides.
pub const TILE_WIDTH: f32 = 190.0;
pub const TILE_ROW_WIDTH: f32 = TILE_WIDTH - 2.0 * mk::spacing::M;
/// Tile width in the dock — narrower, because the dock is a strip
/// and the eye scans it sideways, not in a field.
pub const TILE_DOCK_WIDTH: f32 = 132.0;
pub const TILE_DOCK_ROW_WIDTH: f32 = TILE_DOCK_WIDTH - 2.0 * mk::spacing::M;

/// Camp 6 of the iced replacement (GRABUNG-ICED-ABLOESUNG.md): the
/// FAMILY MOVE begins. Here the first family — the
/// buttons — moves onto the iced-free building blocks: the same grammar as
/// `mkw::button` (Style × Role × Size, all colours from the palette,
/// state layers via mk::state_layer), only the floor changes.
/// Instead of iced widgets, mk::scene nodes come into being; measuring goes through
/// matrixkit-text, rasterising by the client. NO iced type enters
/// this module — and because both versions read the same tokens,
/// the family keeps its face through the move.
pub use matrixkit_scene::scene;
