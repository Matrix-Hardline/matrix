//! Matrix Mitteilungen — app #19, the notification daemon (round 2).
//!
//! The UNUserNotificationCenter extract: ONE system service takes the
//! Mitteilungen aller Programme entgegen (org.freedesktop.Notifications
//! via DBus) and shows them as kit pills top right. Round 2 brings:
//!   • history — nothing is lost, the bell in the bar opens it
//!   • do not disturb — popups stay silent, the history keeps collecting
//!   • action buttons — notify actions become buttons; a click sends
//!     the ActionInvoked signal back (the freedesktop app contract)
//!
//! The DBus part runs blocking in a side thread; a back channel
//! (command) lets the iced surface emit signals.

mod licht;

use matrixkit_widgets::iced;
use matrixkit_widgets::iced_layershell;
use std::sync::mpsc;

use matrixkit_widgets::iced::widget::{button, column, container, image, mouse_area, row, Space};
use matrixkit_widgets::iced::{Color, Element, Font, Length, Subscription, Task};
use matrixkit_widgets::iced_layershell::to_layer_message;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;
use matrixkit_widgets::color;

pub(crate) const WIDTH: u32 = 380;
pub(crate) const DWELL_TIME_MS: u128 = 5000;
pub(crate) const HISTORY_MAX: usize = 40;

#[derive(Debug, Clone)]
pub(crate) struct Notification {
    pub(crate) id: u32,
    pub(crate) app: String,
    pub(crate) title: String,
    pub(crate) text: String,
    /// (key, label) — "default" is not shown as a button.
    pub(crate) actions: Vec<(String, String)>,
    /// Unix seconds when it arrived (round 31: the inbox shows the age).
    pub(crate) at: u64,
    /// Round 109: what a tap opens (hint `x-matrix-open`, mk::notify::Note).
    pub(crate) open: Option<String>,
}

/// ROUND 31: the history as the shared inbox (mk::inbox) — published after
/// every change, the commands of the bar's user card applied here. Returns
/// true when the history changed. Both front ends (iced, house) call it.
pub(crate) fn inbox_sync(history: &mut Vec<Notification>, command: Option<&mpsc::Sender<Command>>, changed: bool) -> bool {
    let mut dirty = changed;
    for c in mk::inbox::take() {
        match c {
            mk::inbox::Command::Open(id) => {
                // Round 109: a target of its own — the daemon opens it.
                let target = history.iter().find(|m| m.id == id).and_then(|m| m.open.clone());
                if let Some(t) = &target {
                    if !mk::open::open(t) {
                        eprintln!("[mitteilung] nichts öffnet «{t}»");
                    }
                }
                if let Some(cmd) = command {
                    if target.is_none() {
                        let _ = cmd.send(Command::Action(id, String::from("default")));
                    }
                    let _ = cmd.send(Command::Closed(id, 2));
                }
                history.retain(|m| m.id != id);
            }
            mk::inbox::Command::Dismiss(id) => {
                if let Some(cmd) = command {
                    let _ = cmd.send(Command::Closed(id, 2));
                }
                history.retain(|m| m.id != id);
            }
            mk::inbox::Command::Clear => history.clear(),
        }
        dirty = true;
    }
    if dirty {
        let items: Vec<mk::inbox::Item> = history
            .iter()
            .map(|m| mk::inbox::Item {
                id: m.id,
                app: m.app.clone(),
                title: m.title.clone(),
                text: m.text.clone(),
                at: m.at,
                opens: m.open.is_some() || m.actions.iter().any(|(k, _)| k == "default"),
            })
            .collect();
        mk::inbox::publish(&items);
    }
    dirty
}

pub(crate) enum Event {
    New(Notification),
    Gone(u32),
}

/// Back channel: the app asks the DBus thread to emit signals.
pub(crate) enum Command {
    Action(u32, String),
    Closed(u32, u32),
}

fn runtime_dir() -> String {
    std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"))
}

/// R105/R103: -show is the protocol; -zeige is accepted as legacy
/// (an old bar may still write it mid-upgrade).
pub(crate) fn show_path() -> String {
    format!("{}/matrix-notifications-show", runtime_dir())
}

pub(crate) fn show_path_legacy() -> String {
    format!("{}/matrix-notifications-zeige", runtime_dir())
}

/// R103 mutual exclusion: another popup asks us to fold the history.
pub(crate) fn hide_path() -> String {
    format!("{}/matrix-notifications-hide", runtime_dir())
}

pub(crate) fn newest_mtime(paths: &[String]) -> Option<std::time::SystemTime> {
    paths
        .iter()
        .filter_map(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
        .max()
}

pub(crate) fn dnd_on() -> bool {
    matches!(mk::settings::read("do-not-disturb").as_deref(), Some("on") | Some("an"))
}

// ------------------------------------------------------------- DBus

struct Service {
    /// Where the events flow — mpsc (iced path) or the
    /// loom of the window house (stage 26: Loom::feed).
    send: Box<dyn Fn(Event) + Send + Sync>,
    next_id: std::sync::atomic::AtomicU32,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl Service {
    #[allow(clippy::too_many_arguments)]
    fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        _app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
        _expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id != 0 {
            replaces_id
        } else {
            self.next_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        };
        // actions is flat [key1, label1, key2, label2, …] → pairs.
        let actions: Vec<(String, String)> = actions
            // family-exception: DBus pair parsing, no tile grid
            .chunks(2)
            .filter_map(|c| match c {
                [k, l] => Some((k.clone(), l.clone())),
                _ => None,
            })
            .collect();
        (self.send)(Event::New(Notification {
            id,
            app: app_name,
            title: summary,
            text: body,
            actions,
            at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
            open: hints.get(mk::notify::OPEN_HINT).and_then(|v| String::try_from(v.try_clone().ok()?).ok()).filter(|s| !s.is_empty()),
        }));
        id
    }

    fn close_notification(&self, id: u32) {
        (self.send)(Event::Gone(id));
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            String::from("body"),
            String::from("actions"),
            String::from("persistence"),
        ]
    }

    fn get_server_information(&self) -> (String, String, String, String) {
        (
            String::from("Matrix Mitteilungen"),
            String::from("Matrix"),
            String::from(env!("CARGO_PKG_VERSION")),
            String::from("1.2"),
        )
    }
}

pub(crate) fn service_start(
    send: impl Fn(Event) + Send + Sync + 'static,
    command_rx: mpsc::Receiver<Command>,
    claim_name: bool,
) {
    let send: Box<dyn Fn(Event) + Send + Sync> = Box::new(send);
    std::thread::spawn(move || {
        let make = move |send: Box<dyn Fn(Event) + Send + Sync>| -> zbus::Result<zbus::blocking::Connection> {
            let conn = zbus::blocking::connection::Builder::session()?
                .serve_at(
                    "/org/freedesktop/Notifications",
                    Service {
                        send,
                        next_id: std::sync::atomic::AtomicU32::new(1),
                    },
                )?
                .build()?;
            // Shadow probe (claim_name=false): do NOT take over the
            // well-known name — the session's running service
            // stays untouched; the probe addresses the
            // unique name directly.
            if claim_name {
                use zbus::fdo::RequestNameFlags;
                let dbus = zbus::blocking::fdo::DBusProxy::new(&conn)?;
                let _ = dbus.request_name(
                    "org.freedesktop.Notifications".try_into().unwrap(),
                    RequestNameFlags::AllowReplacement | RequestNameFlags::ReplaceExisting,
                );
            }
            eprintln!(
                "[mitteilung] DBus liegt an — eindeutiger Name {}",
                conn.unique_name().map(|n| n.as_str()).unwrap_or("?")
            );
            Ok(conn)
        };
        match make(send) {
            Ok(conn) => loop {
                // Back channel: emit signals (ActionInvoked/Closed).
                while let Ok(b) = command_rx.try_recv() {
                    let _ = match b {
                        Command::Action(id, key) => conn.emit_signal(
                            None::<()>,
                            "/org/freedesktop/Notifications",
                            "org.freedesktop.Notifications",
                            "ActionInvoked",
                            &(id, key),
                        ),
                        Command::Closed(id, reason) => conn.emit_signal(
                            None::<()>,
                            "/org/freedesktop/Notifications",
                            "org.freedesktop.Notifications",
                            "NotificationClosed",
                            &(id, reason),
                        ),
                    };
                }
                std::thread::sleep(std::time::Duration::from_millis(80));
            },
            Err(e) => eprintln!("matrix-notifications: DBus-Start scheiterte: {e}"),
        }
    });
}

// ----------------------------------------------------------------- App

fn main() -> Result<(), iced_layershell::Error> {
    // Stage 26 of the iced replacement: --erstlicht is the window-house
    // path (shadow doctrine).
    // Round 31 (02.11.2026): the HOUSE is the default (R216, «without iced»);
    // iced only as the fallback: `--iced`, or the setting `shell-iced=on`.
    let iced = std::env::args().any(|a| a == "--iced") || mk::settings::read("shell-iced").as_deref() == Some("on");
    if !iced {
        // The guard applies to BOTH paths. It only stood in the
        // iced branch, so the light path lacked the open/close gesture: a second
        // key press would have opened a SECOND panel instead of closing
        // the first
        // (finger probe 16.08.2026).
        // Round 76: a daemon, not a popup — the bell opens the panel through
        // the show file; a restart must not toggle the dying copy away.
        if !mkw::daemon_single() {
            return Ok(());
        }
        if let Err(e) = licht::run() {
            eprintln!("[mitteilung] Fensterhaus-Fehler: {e}");
        }
        return Ok(());
    }
    matrixkit_widgets::present_mailbox();
    use matrixkit_widgets::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
    use matrixkit_widgets::iced_layershell::settings::{LayerShellSettings, Settings};
    if !mkw::panel_toggle() {
        return Ok(());
    }
    iced_layershell::application(
        App::new,
        || String::from("matrix-mitteilung"),
        App::update,
        App::view,
    )
    .subscription(App::subscription)
    .style(|_state, _theme| iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
    })
    .settings(Settings {
        layer_settings: LayerShellSettings {
            size: Some((1, 1)),
            anchor: Anchor::Top | Anchor::Right,
            // R103 popup grammar: below the bar pill (54) with the same
            // 8 px air as every popup, right edge on the pill edge (10).
            margin: (mkw::panel::BAR_STRIP + mkw::panel::POPUP_TOP as i32, mkw::panel::POPUP_RIGHT as i32, 0, 0),
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: -1,
            ..Default::default()
        },
        default_font: Font::with_name("Inter Variable"),
        fonts: mkw::load_symbol_fonts(),
        ..Default::default()
    })
    .run()
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Msg {
    Pulse,
    Click(u32),
    Action(u32, String),
    HistoryClose,
    ClearAll,
    DndToggle,
    /// R114 stack grammar (NotificationCenter extract): a click on a
    /// collapsed group expands it, "Weniger anzeigen" folds it back.
    GroupExpand(String),
    GroupCollapse(String),
}

struct App {
    palette: mk::Palette,
    watcher: mk::PaletteWatcher,
    rx: mpsc::Receiver<Event>,
    command: mpsc::Sender<Command>,
    /// Aktive Popups (Mitteilung + Geburtszeit).
    stack: Vec<(Notification, std::time::Instant)>,
    /// History — youngest first.
    history: Vec<Notification>,
    history_open: bool,
    /// R114: apps whose history group is expanded (expandCollapseGroup);
    /// everything else shows as a collapsed stack behind its top card.
    expanded: std::collections::HashSet<String>,
    dnd: bool,
    icons: std::collections::HashMap<String, Option<image::Handle>>,
    last_show: Option<std::time::SystemTime>,
    last_hide: Option<std::time::SystemTime>,
    last_height: u32,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let (tx, rx) = mpsc::channel();
        let (btx, brx) = mpsc::channel();
        service_start(move |e| { let _ = tx.send(e); }, brx, true);
        (
            App {
                palette: mk::Palette::load().unwrap_or_default(),
                watcher: mk::PaletteWatcher::new(),
                rx,
                command: btx,
                stack: Vec::new(),
                history: Vec::new(),
                history_open: false,
                expanded: std::collections::HashSet::new(),
                dnd: dnd_on(),
                icons: std::collections::HashMap::new(),
                last_show: newest_mtime(&[show_path(), show_path_legacy()]),
                last_hide: std::fs::metadata(hide_path()).and_then(|m| m.modified()).ok(),
                last_height: 1,
            },
            Task::none(),
        )
    }

    fn icon_backen(&mut self, app: &str) {
        let p = self.palette;
        self.icons.entry(app.to_string()).or_insert_with(|| {
            matrixkit_icons::render_png(app, &p).map(image::Handle::from_bytes)
        });
    }

    /// R114: byApp groups over the history, newest group first, order
    /// inside a group = newest first (the history is youngest-first).
    fn history_groups(&self) -> Vec<(&str, Vec<&Notification>)> {
        let mut groups: Vec<(&str, Vec<&Notification>)> = Vec::new();
        for m in &self.history {
            if let Some(g) = groups.iter_mut().find(|(a, _)| *a == m.app.as_str()) {
                g.1.push(m);
            } else {
                groups.push((m.app.as_str(), vec![m]));
            }
        }
        groups
    }

    /// Surface height for the grouped history: collapsed stacks add
    /// their peek edges, expanded groups their fold-back row.
    fn history_height(&self) -> u32 {
        let mut h = 44 + 16;
        for (app, items) in self.history_groups() {
            if items.len() == 1 {
                h += 84;
            } else if self.expanded.contains(app) {
                h += items.len() as u32 * 84 + 30;
            } else {
                h += 84 + 13;
            }
        }
        h.max(44 + 84 + 16)
    }

    fn mass(&mut self) -> Task<Msg> {
        let (width, height) = if self.history_open {
            (WIDTH, self.history_height())
        } else if !self.stack.is_empty() {
            (WIDTH, (self.stack.len() as u32) * 108 + 8)
        } else {
            (1, 1)
        };
        if height != self.last_height {
            self.last_height = height;
            return Task::done(Msg::SizeChange((width, height.min(1400))));
        }
        Task::none()
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Pulse => {
                if self.watcher.changed() {
                    if let Some(new) = mk::Palette::load() {
                        self.palette = new;
                    }
                    if !mk::transition::active() {
                        self.icons.clear();
                    }
                }
                self.dnd = dnd_on();
                // Bell: open/close the history panel via file toggle.
                let show = newest_mtime(&[show_path(), show_path_legacy()]);
                if show.is_some() && show != self.last_show {
                    self.last_show = show;
                    self.history_open = !self.history_open;
                }
                // R103: a sibling popup opened — fold the history quietly.
                let hide = std::fs::metadata(hide_path()).and_then(|m| m.modified()).ok();
                if hide.is_some() && hide != self.last_hide {
                    self.last_hide = hide;
                    self.history_open = false;
                }
                let mut fresh = false;
                while let Ok(e) = self.rx.try_recv() {
                    match e {
                        Event::New(m) => {
                            fresh = true;
                            self.icon_backen(&m.app);
                            self.history.retain(|v| v.id != m.id);
                            self.history.insert(0, m.clone());
                            self.history.truncate(HISTORY_MAX);
                            // Do not disturb: only the history, no popup.
                            if !self.dnd {
                                // Nutzer-Fund (15.7.): niris Screenshot-
                                // the message is a TRIGGER, not mail —
                                // it sounds as 09, not as a notice.
                                let photo = m.title.contains("creenshot")
                                    || m.title.contains("ildschirmfoto")
                                    || m.app.contains("niri");
                                // Round 105: the bar shows it as its island —
                                // no popup of our own (sound and history stay).
                                if !mk::island::active() {
                                    self.stack.retain(|(old, _)| old.id != m.id);
                                    self.stack.push((m, std::time::Instant::now()));
                                }
                                if photo {
                                    mk::feedback::now("screenshot", "09-screenshot.wav");
                                } else {
                                    mk::feedback::hint();
                                }
                            }
                        }
                        Event::Gone(id) => self.stack.retain(|(m, _)| m.id != id),
                    }
                }
                self.stack
                    .retain(|(_, since)| since.elapsed().as_millis() < DWELL_TIME_MS);
                inbox_sync(&mut self.history, Some(&self.command), fresh);
                return self.mass();
            }
            Msg::Click(id) => {
                self.stack.retain(|(m, _)| m.id != id);
                let _ = self.command.send(Command::Closed(id, 2)); // dismissed
                return self.mass();
            }
            Msg::Action(id, key) => {
                let _ = self.command.send(Command::Action(id, key));
                let _ = self.command.send(Command::Closed(id, 2));
                self.stack.retain(|(m, _)| m.id != id);
                return self.mass();
            }
            Msg::HistoryClose => {
                self.history_open = false;
                return self.mass();
            }
            Msg::ClearAll => {
                self.history.clear();
                inbox_sync(&mut self.history, Some(&self.command), true);
                return self.mass();
            }
            Msg::DndToggle => {
                self.dnd = !self.dnd;
                mk::settings::write("do-not-disturb", if self.dnd { "on" } else { "off" });
                if self.dnd {
                    self.stack.clear();
                }
                return self.mass();
            }
            Msg::GroupExpand(app) => {
                self.expanded.insert(app);
                return self.mass();
            }
            Msg::GroupCollapse(app) => {
                self.expanded.remove(&app);
                return self.mass();
            }
            _ => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Msg> {
        // Every pulse redraws the surface and makes the compositor compose
        // again: 8 per second around the clock cost the duo ~3 % CPU here and
        // a share of niri's 12 % idle (01.11.2026). Fast only while something
        // shows (popups fade, history open); resting, 400 ms still catches a
        // new message or a bell tap without a noticeable wait.
        mkw::tick_when(
            "mitteilungen",
            !self.stack.is_empty() || self.history_open,
            std::time::Duration::from_millis(120),
            std::time::Duration::from_millis(400),
        )
        .map(|_| Msg::Pulse)
    }

    /// An app icon (living) or initial tile.
    fn icon<'a>(&'a self, app: &str, edge: f32) -> Element<'a, Msg> {
        let p = self.palette;
        match self.icons.get(app).cloned().flatten() {
            Some(h) => image(h).width(edge).height(edge).into(),
            None => container(mkw::txt(
                app.chars().next().unwrap_or('•').to_uppercase().to_string(),
                mk::typo::HEADLINE,
                p.on_primary_container,
            ))
            .center_x(Length::Fixed(edge))
            .center_y(Length::Fixed(edge))
            .style(move |_| container::Style {
                background: Some(color(p.primary_container).into()),
                border: iced::border::rounded(mk::radius::SMALL + 2.0),
                ..Default::default()
            })
            .into(),
        }
    }

    fn view(&self) -> Element<'_, Msg> {
        if self.history_open {
            self.history_view()
        } else {
            self.stack_view()
        }
    }

    fn stack_view(&self) -> Element<'_, Msg> {
        let _p = self.palette;
        if self.stack.is_empty() {
            return Space::new().into();
        }
        let mut column = column![].spacing(mk::spacing::S);
        for (m, _) in &self.stack {
            column = column.push(self.pill(m, true));
        }
        container(column)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .into()
    }

    /// A notification pill. `mit_aktionen` = popup (show buttons).
    fn pill<'a>(&'a self, m: &'a Notification, with_actions: bool) -> Element<'a, Msg> {
        let p = self.palette;
        let mut text_column = column![mkw::txt(&m.title, mk::typo::HEADLINE, p.on_surface)];
        if !m.text.is_empty() {
            let short: String = m.text.chars().take(140).collect();
            text_column = text_column.push(mkw::txt(short, mk::typo::CAPTION, p.on_surface_variant));
        }
        // Action buttons (without "default").
        if with_actions {
            let mut button_row = row![].spacing(mk::spacing::XS);
            let mut found = false;
            for (key, label) in &m.actions {
                if key == "default" {
                    continue;
                }
                found = true;
                let id = m.id;
                let k = key.clone();
                button_row = button_row.push(
                    button(mkw::txt(label, mk::typo::CAPTION, p.on_surface))
                        .padding([2, mk::spacing::S as u16])
                        .style(move |_, s| mkw::panel::button_style(p, s, mk::radius::SMALL))
                        .on_press(Msg::Action(id, k.clone())),
                );
            }
            if found {
                text_column = text_column
                    .push(Space::new().height(mk::spacing::XXS))
                    .push(button_row);
            }
        }
        let content = row![self.icon(&m.app, 36.0), text_column]
            .spacing(mk::spacing::M)
            .align_y(iced::Alignment::Center);
        let area = mkw::ui::panel_shell(content.into(), WIDTH as f32 - 8.0, p);
        mouse_area(area).on_press(Msg::Click(m.id)).into()
    }

    /// R114 collapsed stack (isTopCollapsedForGroup): the newest card
    /// carries a "+n" capsule, the older cards peek out beneath as two
    /// narrowing edges — a click expands the group, not the card.
    fn stack_card<'a>(&'a self, top: &'a Notification, extra: usize) -> Element<'a, Msg> {
        let p = self.palette;
        let mut text_column = column![mkw::txt(&top.title, mk::typo::HEADLINE, p.on_surface)];
        if !top.text.is_empty() {
            let short: String = top.text.chars().take(140).collect();
            text_column = text_column.push(mkw::txt(short, mk::typo::CAPTION, p.on_surface_variant));
        }
        let content = row![self.icon(&top.app, 36.0), text_column]
            .spacing(mk::spacing::M)
            .align_y(iced::Alignment::Center);
        let card = mkw::ui::panel_shell(content.into(), WIDTH as f32 - 8.0, p);
        // The stacked summary capsule, top right on the card.
        let badge = container(
            container(mkw::txt(format!("+{extra}"), mk::typo::LABEL, p.on_primary))
                .padding([0, 6])
                .style(move |_| container::Style {
                    background: Some(color(p.primary).into()),
                    border: iced::border::rounded(mk::radius::capsule(16.0)),
                    ..Default::default()
                }),
        )
        .align_x(iced::alignment::Horizontal::Right)
        .width(Length::Fill)
        .padding(iced::Padding { top: 6.0, right: 10.0, ..iced::Padding::ZERO });
        let card: Element<'_, Msg> = iced::widget::stack![card, badge].into();
        // Peek edges: each following card narrower, only its rim shows.
        let edge = |w: f32, h: f32, a: f32| {
            container(Space::new().width(Length::Fixed(w)).height(Length::Fixed(h))).style(
                move |_| container::Style {
                    background: Some(color(mk::Rgba { a, ..p.surface_container }).into()),
                    border: iced::border::rounded(6.0),
                    ..Default::default()
                },
            )
        };
        let mut pile = column![card].spacing(2.0).align_x(iced::Alignment::Center);
        pile = pile.push(edge(WIDTH as f32 - 36.0, 5.0, 0.8));
        if extra > 1 {
            pile = pile.push(edge(WIDTH as f32 - 64.0, 4.0, 0.5));
        }
        mouse_area(pile).on_press(Msg::GroupExpand(top.app.clone())).into()
    }

    fn history_view(&self) -> Element<'_, Msg> {
        let p = self.palette;
        // MatrixUI MenuFamily: panel head in the ONE language.
        let dnd_char = if self.dnd { mkw::symbol::VOLUME_OFF } else { mkw::symbol::NOTIFICATIONS };
        let header = mkw::ui::panel_header(
            "Mitteilungen",
            vec![
                mkw::ui::nav_button(dnd_char, true, Msg::DndToggle, p),
                mkw::ui::header_text_button("Leeren", Msg::ClearAll, p),
                mkw::ui::nav_button(mkw::symbol::CLOSE, true, Msg::HistoryClose, p),
            ],
            p,
        );
        let mut list = column![header].spacing(mk::spacing::XS);
        if self.history.is_empty() {
            list = list.push(
                container(mkw::txt("Keine Mitteilungen", mk::typo::CALLOUT, p.on_surface_variant))
                    .center_x(Length::Fill)
                    .padding(mk::spacing::L),
            );
        } else {
            // R114 stack grammar (NotificationCenter extract): the
            // history groups byApp; singles render flat, groups fold
            // behind their newest card until expanded.
            for (app, items) in self.history_groups() {
                if items.len() == 1 {
                    list = list.push(self.pill(items[0], false));
                } else if self.expanded.contains(app) {
                    for m in items {
                        list = list.push(self.pill(m, false));
                    }
                    let app_owned = app.to_string();
                    list = list.push(
                        mouse_area(
                            container(mkw::txt(
                                "Weniger anzeigen",
                                mk::typo::CAPTION,
                                p.primary,
                            ))
                            .center_x(Length::Fill)
                            .padding([4, 0]),
                        )
                        .on_press(Msg::GroupCollapse(app_owned)),
                    );
                } else {
                    list = list.push(self.stack_card(items[0], items.len() - 1));
                }
            }
        }
        mkw::ui::panel_shell(list.into(), WIDTH as f32, p)
    }
}
