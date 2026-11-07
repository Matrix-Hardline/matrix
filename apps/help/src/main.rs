//! Matrix Hilfe — everything worth knowing about MatrixKit, for users and devs.
//!
//! Built in the style of a help app: sidebar with categories, card
//! overview of the articles, reading view with back navigation. The
//! knowledge base lives in inhalte.rs — new articles need no UI code.

mod contents;
mod licht;

use matrixkit_widgets::iced;
use matrixkit_widgets::iced::widget::{column, container, row, Space};
use matrixkit_widgets::iced::{Element, Length, Subscription, Task};
use contents::CATEGORIES;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use mkw::color;
use std::time::Duration;

const APP_ID: &str = "matrix-help";

fn main() -> iced::Result {
    // Lager 28 of the iced replacement: --erstlicht is the window house
    // path (shadow doctrine).
    if std::env::args().nth(1).as_deref() == Some("--erstlicht") {
        if let Err(e) = licht::run() {
            eprintln!("[hilfe] Fensterhaus-Fehler: {e}");
        }
        return Ok(());
    }
    mkw::app::window::<App>(APP_ID, 760.0, 540.0)
}

impl mkw::app::Program for App {
    type Msg = Msg;
    fn new() -> (Self, Task<Msg>) {
        App::new()
    }
    fn update(&mut self, msg: Msg) -> Task<Msg> {
        App::update(self, msg)
    }
    fn view(&self) -> Element<'_, Msg> {
        App::view(self)
    }
    fn title(&self) -> String {
        String::from("Matrix Hilfe")
    }
    fn subscription(&self) -> iced::Subscription<Msg> {
        App::subscription(self)
    }
}

struct App {
    frame: mkw::Frame,
    category: usize,
    /// Some(i) = article i of the active category is open.
    article: Option<usize>,
    /// Entrance spring of the reading view (articles glide in).
    read_spring: mk::motion::Spring,
    /// Keyboard focus: categories (0..4), then article cards; in the
    /// reading view only the back button.
    focus: mkw::Focus,
    /// History navigation like the reference (‹ ›): visited views.
    history: Vec<(usize, Option<usize>)>,
    forward: Vec<(usize, Option<usize>)>,
    /// Full-text search over all categories (reference .searchable).
    search: String,
}

#[derive(Debug, Clone)]
enum Msg {
    Frame(mkw::FrameMsg),
    Tick,
    /// Own 60 fps tick for the reading spring (articles glide in).
    ReadTick,
    HistoryBack,
    HistoryForward,
    Key(mkw::Key),
    Category(usize),
    Article(usize),
    Back,
    Search(String),
    SearchClear,
    /// Open a search hit: (category, article).
    Hit(usize, usize),
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        // HelpLink (reference): `matrix-help --suche <term>` opens the help
        // directly with a filled search — the ? buttons of the apps thus
        // jump into their chapter. Dev hook MATRIX_HILFE_SUCHE stays.
        let args: Vec<String> = std::env::args().collect();
        let cli_search = args
            .iter()
            .position(|a| a == "--suche")
            .and_then(|i| args.get(i + 1))
            .cloned();
        let search = cli_search
            .or_else(|| std::env::var("MATRIX_HILFE_SUCHE").ok())
            .unwrap_or_default();

        // Memory (reference SceneStorage): the help opens where it was
        // closed — unless a search job overrides that.
        let (category, article) = if search.is_empty() {
            Self::state_read()
        } else {
            (0, None)
        };

        (
            Self {
                frame: mkw::Frame::new("matrix-help", &[]),
                category,
                article,
                read_spring: mk::motion::Spring::new(1.0),
                focus: if article.is_some() {
                    mkw::Focus::new(1)
                } else {
                    mkw::Focus::new(CATEGORIES.len() + CATEGORIES[category].article.len())
                },
                history: Vec::new(),
                forward: Vec::new(),
                search,
            },
            Task::none(),
        )
    }

    /// Load the remembered place ("k" or "k,a"), clamped against the
    /// current categories/articles — chapters may shrink between versions
    /// without the restore pointing into the void.
    fn state_read() -> (usize, Option<usize>) {
        let Some(value) = mk::settings::read("help-state") else {
            return (0, None);
        };
        let mut parts = value.splitn(2, ',');
        let k = parts
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&k| k < CATEGORIES.len())
            .unwrap_or(0);
        let a = parts
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&a| a < CATEGORIES[k].article.len());
        (k, a)
    }

    /// Remember the current place — called after every navigation.
    fn state_remember(&self) {
        let value = match self.article {
            Some(a) => format!("{},{a}", self.category),
            None => format!("{}", self.category),
        };
        mk::settings::write("help-state", &value);
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Frame(m) => self.frame.update(m).map(Msg::Frame),
            Msg::Tick => {
                self.frame.palette_changed();
                Task::none()
            }
            Msg::ReadTick => {
                self.read_spring.tick(1.0 / 60.0);
                Task::none()
            }
            Msg::Category(i) => {
                self.history.push((self.category, self.article));
                self.forward.clear();
                self.category = i.min(CATEGORIES.len() - 1);
                self.article = None;
                self.focus.set_count(CATEGORIES.len() + CATEGORIES[self.category].article.len());
                self.state_remember();
                Task::none()
            }
            Msg::Article(i) => {
                self.history.push((self.category, self.article));
                self.forward.clear();
                self.article = Some(i);
                self.focus = mkw::Focus::new(1); // only the back button
                // the reading view glides in (with reduced motion: instantly)
                self.read_spring = mk::motion::Spring::new(if mk::motion_reduced() { 1.0 } else { 0.0 });
                self.read_spring.retarget(1.0);
                self.state_remember();
                Task::none()
            }
            Msg::Back => {
                self.history.push((self.category, self.article));
                self.forward.clear();
                self.article = None;
                self.focus = mkw::Focus::new(CATEGORIES.len() + CATEGORIES[self.category].article.len());
                self.state_remember();
                Task::none()
            }
            Msg::HistoryBack => {
                if let Some((k, a)) = self.history.pop() {
                    self.forward.push((self.category, self.article));
                    self.category = k;
                    self.article = a;
                    self.focus = mkw::Focus::new(if a.is_some() { 1 } else { CATEGORIES.len() + CATEGORIES[k].article.len() });
                }
                self.state_remember();
                Task::none()
            }
            Msg::HistoryForward => {
                if let Some((k, a)) = self.forward.pop() {
                    self.history.push((self.category, self.article));
                    self.category = k;
                    self.article = a;
                    self.focus = mkw::Focus::new(if a.is_some() { 1 } else { CATEGORIES.len() + CATEGORIES[k].article.len() });
                }
                self.state_remember();
                Task::none()
            }
            Msg::Search(s) => {
                self.search = s;
                Task::none()
            }
            Msg::SearchClear => {
                self.search.clear();
                Task::none()
            }
            Msg::Hit(k, i) => {
                self.history.push((self.category, self.article));
                self.forward.clear();
                self.category = k.min(CATEGORIES.len() - 1);
                self.article = Some(i);
                self.search.clear();
                self.focus = mkw::Focus::new(1);
                self.read_spring = mk::motion::Spring::new(if mk::motion_reduced() { 1.0 } else { 0.0 });
                self.read_spring.retarget(1.0);
                self.state_remember();
                Task::none()
            }
            Msg::Key(t) => {
                // Root layer first; only if it does not consume the key does it
                // steer the app-own category/article navigation.
                if self.frame.handle_key(t) {
                    return Task::none();
                }
                match t {
                    mkw::Key::Escape => {
                        // dismissSearch (reference): Esc clears the search first
                        if !self.search.is_empty() {
                            self.search.clear();
                            return Task::none();
                        }
                        if self.article.is_some() {
                            return self.update(Msg::Back);
                        }
                    }
                    mkw::Key::Next => self.focus.onward(),
                    mkw::Key::Prev => self.focus.back(),
                    mkw::Key::Activate => {
                        if let Some(i) = self.focus.current() {
                            if self.article.is_some() {
                                return self.update(Msg::Back);
                            }
                            return self.update(if i < CATEGORIES.len() {
                                Msg::Category(i)
                            } else {
                                Msg::Article(i - CATEGORIES.len())
                            });
                        }
                    }
                    // Ctrl+F (reference keyboardShortcut): focus the search
                    mkw::Key::Search => return mkw::focus_search(),
                    mkw::Key::Settings => {}
                    mkw::Key::Undo => {}
                    mkw::Key::Refresh => {}
                    // New keys (e.g. Key::Peek) are no-ops here.
                    _ => {}
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Msg> {
        let mut subs = vec![
            self.frame.subscription().map(Msg::Frame),
            mkw::tick("help", Duration::from_secs(2)).map(|_| Msg::Tick),
            mkw::key_subscription(Msg::Key),
        ];
        // The reading view glides in with its own spring.
        if !self.read_spring.is_settled() {
            subs.push(mkw::tick("help-read", Duration::from_millis(16)).map(|_| Msg::ReadTick));
        }
        Subscription::batch(subs)
    }

    fn view(&self) -> Element<'_, Msg> {
        let p = self.frame.palette;
        let kat = &CATEGORIES[self.category];

        // MatrixUI SidebarFamily: supply entries, the anatomy comes along.
        let symbols = [mkw::symbol::ROCKET_LAUNCH, mkw::symbol::TOUCH_APP, mkw::symbol::SHIELD, mkw::symbol::PALETTE, mkw::symbol::CODE];
        let items: Vec<mkw::SidebarItem> = CATEGORIES
            .iter()
            .enumerate()
            .map(|(i, k)| mkw::SidebarItem {
                glyph: symbols[i.min(symbols.len() - 1)],
                title: k.name,
                count: Some(k.article.len()),
            })
            .collect();

        // --- content: search, card overview or reading view ---
        let detail: Element<'_, Msg> = if !self.search.is_empty() {
            // Full-text hits across ALL categories (reference .searchable)
            let request = self.search.to_lowercase();
            let mut list = column![].spacing(mk::spacing::M);
            let mut count = 0usize;
            for (ki, k) in CATEGORIES.iter().enumerate() {
                for (ai, a) in k.article.iter().enumerate() {
                    let fits = a.title.to_lowercase().contains(&request)
                        || a.teaser.to_lowercase().contains(&request)
                        || a.content.to_lowercase().contains(&request);
                    if !fits {
                        continue;
                    }
                    count += 1;
                    list = list.push(
                        iced::widget::button(
                            column![
                                mkw::txt(k.name, mk::typo::LABEL, p.primary),
                                mkw::txt(a.title, mk::typo::SUBTITLE, p.on_surface),
                                Space::new().height(mk::spacing::XS),
                                mkw::txt(a.teaser, mk::typo::BODY, p.on_surface_variant),
                            ]
                            .spacing(2)
                            .width(Length::Fill),
                        )
                        .width(Length::Fill)
                        .padding(mk::spacing::L)
                        .on_press(Msg::Hit(ki, ai))
                        .style(move |_, status| {
                            let base = p.surface_container_high;
                            let bg = match status {
                                iced::widget::button::Status::Hovered => {
                                    p.on_surface.over(base, mk::state_layer::HOVER)
                                }
                                iced::widget::button::Status::Pressed => {
                                    p.on_surface.over(base, mk::state_layer::PRESSED)
                                }
                                _ => base,
                            };
                            // family-exception: search hit card — clickable card with three-line content, mkw::knopf takes only a text label
                            iced::widget::button::Style {
                                background: Some(color(bg).into()),
                                border: iced::Border { radius: mk::CORNER_RADIUS.into(), ..Default::default() },
                                ..Default::default()
                            }
                        }),
                    );
                }
            }
            if count == 0 {
                list = list.push(mkw::txt(
                    format!("Keine Treffer für „{}“ — anders formulieren oder kürzer suchen.", self.search),
                    mk::typo::BODY,
                    p.on_surface_variant,
                ));
            }
            column![
                mkw::txt(
                    if count == 1 { String::from("1 Treffer") } else { format!("{count} Treffer") },
                    mk::typo::TITLE,
                    p.on_surface,
                ),
                Space::new().height(mk::spacing::M),
                self.frame.scroll_area(list.into(), Msg::Frame),
            ]
            .spacing(0)
            .into()
        } else { match self.article {
            None => {
                // Cards of the category (one column — calm and readable)
                let mut cards = column![].spacing(mk::spacing::M);
                for (i, a) in kat.article.iter().enumerate() {
                    let card_focus = self.focus.is(CATEGORIES.len() + i);
                    cards = cards.push(
                        iced::widget::button(
                            column![
                                mkw::txt(a.title, mk::typo::SUBTITLE, p.on_surface),
                                Space::new().height(mk::spacing::XS),
                                mkw::txt(a.teaser, mk::typo::BODY, p.on_surface_variant),
                            ]
                            .spacing(0)
                            .width(Length::Fill),
                        )
                        .width(Length::Fill)
                        .padding(mk::spacing::L)
                        .on_press(Msg::Article(i))
                        .style(move |_, status| {
                            let base = p.surface_container_high;
                            let bg = match status {
                                iced::widget::button::Status::Hovered => {
                                    p.on_surface.over(base, mk::state_layer::HOVER)
                                }
                                iced::widget::button::Status::Pressed => {
                                    p.on_surface.over(base, mk::state_layer::PRESSED)
                                }
                                _ => base,
                            };
                            // family-exception: article card — clickable card with multi-line content + keyboard focus ring, mkw::knopf takes only a text label
                            iced::widget::button::Style {
                                background: Some(color(bg).into()),
                                border: mkw::focus_ring(card_focus, mk::CORNER_RADIUS, p),
                                ..Default::default()
                            }
                        }),
                    );
                }
                column![
                    mkw::txt(kat.name, mk::typo::TITLE, p.on_surface),
                    Space::new().height(mk::spacing::M),
                    self.frame.scroll_area(cards.into(), Msg::Frame),
                ]
                .spacing(0)
                .into()
            }
            Some(i) => {
                let a = &kat.article[i.min(kat.article.len() - 1)];
                let lf = self.read_spring.value.clamp(0.0, 1.2);
                let indent = (24.0 * (1.0 - lf)).max(0.0);
                let back_focus = self.focus.is(0);
                let back = iced::widget::button(
                    row![
                        mkw::symbol::<Msg>(mkw::symbol::ARROW_BACK, mk::font_size::MEDIUM, p.primary),
                        Space::new().width(mk::spacing::XS),
                        mkw::txt(kat.name, mk::typo::BODY, p.primary),
                    ]
                    .align_y(iced::Alignment::Center),
                )
                .padding([mk::spacing::XS as u16, mk::spacing::S as u16])
                .on_press(Msg::Back)
                .style(move |_, status| {
                    let base = p.surface_container;
                    let bg = match status {
                        iced::widget::button::Status::Hovered => {
                            Some(color(p.on_surface.over(base, mk::state_layer::HOVER)).into())
                        }
                        iced::widget::button::Status::Pressed => {
                            Some(color(p.on_surface.over(base, mk::state_layer::PRESSED)).into())
                        }
                        _ => None,
                    };
                    // family-exception: back pill of the reading view — symbol+text content + keyboard focus ring, mkw::knopf takes only a text label
                    iced::widget::button::Style {
                        background: bg,
                        border: mkw::focus_ring(back_focus, mk::CORNER_RADIUS, p),
                        ..Default::default()
                    }
                });
                column![
                    back,
                    Space::new().height(Length::Fixed(mk::spacing::M + indent)),
                    mkw::txt(a.title, mk::typo::TITLE, p.on_surface),
                    Space::new().height(mk::spacing::M),
                    self.frame.scroll_area(
                        container(
                            mkw::txt(a.content, mk::typo::BODY, p.on_surface),
                        )
                        .padding(iced::Padding {
                            top: 0.0,
                            right: mk::spacing::L + mk::spacing::XS,
                            bottom: mk::spacing::L,
                            left: 0.0,
                        })
                        .into(),
                        Msg::Frame,
                    ),
                ]
                .spacing(0)
                .into()
            }
        } };

        // Reference history pill: ‹ › — the family nav button dims inactive
        // arrows itself (HarnessFamily, built for exactly these history arrows).
        let history_pill = row![
            mkw::ui::nav_button(mkw::symbol::ARROW_BACK, !self.history.is_empty(), Msg::HistoryBack, p),
            mkw::ui::nav_button(mkw::symbol::CHEVRON_RIGHT, !self.forward.is_empty(), Msg::HistoryForward, p),
        ]
        .spacing(2);
        let header_row = row![
            history_pill,
            Space::new().width(Length::Fill),
            container(mkw::search_field(&self.search, "Hilfe durchsuchen", Msg::Search, Msg::SearchClear, p))
                .width(Length::Fixed(240.0)),
        ]
        .align_y(iced::Alignment::Center);
        let detail = column![header_row, Space::new().height(mk::spacing::S), detail].spacing(0);

        let content = mkw::ui::sidebar_family(
            items,
            self.category,
            self.article.is_none().then(|| self.focus.current()).flatten(),
            Msg::Category,
            detail.into(),
            p,
        );

        self.frame.window(
            "Matrix Hilfe",
            env!("CARGO_PKG_VERSION"),
            "Alles Wissenswerte zu MatrixKit — für Nutzer und Entwickler.",
            content.into(),
            Msg::Frame,
        )
    }
}
