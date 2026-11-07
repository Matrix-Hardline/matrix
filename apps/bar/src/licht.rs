//! Matrix Bar, First Light version (iced replacement, camp 30 —
//! GRABUNG-ICED-ABLOESUNG.md): the seventh app move — the BAR,
//! and the house learns the exclusive zone.
//!
//! The widget culture stays verbatim: three zones from
//! `~/.config/matrix/bar-widgets`, the readers (focus_read,
//! volume_read, battery_read, net_read, widgets_read) are
//! the SAME functions as in the iced path. The 2 s pulse measures,
//! a snapshot comparison keeps the calm (unchanged =
//! Calm, no diff, no pixel).
//!
//! Shadow mode: exclusive 0 — the bar floats, reserves
//! nothing. LICHT_RESERVE=1 arms the zone (house growth
//! camp 30: Mount::Layer::exclusive), then the windows move
//! below the bar.
//!
//! Since camp 43 the user pill carries the SESSION MENU: the
//! surface grows to screen height (extent), the free area is
//! the close ground, and anything final (restart, shutdown)
//! turns red on the first click — only the second counts
//! (Action verbatim from the iced path, execute() included).
//!
//! Reminders: tray, media, recording, glass tint (vibrant),
//! hover brightening of the pills, Matrix menu of the apps glyph.

use crate::{battery_read, focus_read, net_read, volume_read, widgets_read, Action, Widget, HEIGHT, INSET};
use matrixkit_shell::{self as shell, Refresh, Scene, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{lay_out, Keep, Node, Placed, Rect};
use matrixkit_widgets as mkw;
use matrixkit_widgets::scene as fam;

/// What a hit means.
#[derive(Clone, Copy)]
enum Deed {
    Matrix,
    Bell,
    Central,
    User,
    /// Camp 43: a row of the session menu.
    Act(Action),
    /// The free area below the open menu — a click closes.
    Backdrop,
}

/// Menu dimensions (following the panel grammar).
const MENU_W: f32 = 230.0;
const MENU_ROW: f32 = 40.0;
/// Like iced: the surface grows to screen height, the free area
/// is the close ground.
const MENU_HEIGHT: u32 = 2400;

#[derive(Default)]
struct Build {
    scene: Vec<Placed>,
    roles: Vec<Option<usize>>,
    texts: Vec<Text>,
}

/// The measurement snapshot: unchanged means calm.
#[derive(PartialEq, Clone, Default)]
struct Stand {
    focus: Option<String>,
    clock: String,
    volume: Option<String>,
    net: Option<String>,
    battery: Option<(u8, bool)>,
    cpu: u8,
}

struct Bar {
    palette: mk::Palette,
    zones: (Vec<Widget>, Vec<Widget>, Vec<Widget>),
    /// The session menu (camp 43) and the two-click weapon.
    menu: bool,
    confirmation: Option<Action>,
    stand: Stand,
    sys: sysinfo::System,
    clock_format: &'static str,
    counter: u8,
    user: String,
    deeds: Vec<Deed>,
}

impl Bar {
    fn new() -> Self {
        // Reference-design clock (R33/R35): 12 h only if the locale really
        // fills %p — checked once, not per pulse.
        let am_pm = mk::command::first_line("date", &["+%p"])
            .map(|p| !p.trim().is_empty())
            .unwrap_or(false);
        Bar {
            palette: mk::Palette::load().unwrap_or_default(),
            zones: widgets_read(),
            menu: false,
            confirmation: None,
            stand: Stand::default(),
            sys: sysinfo::System::new(),
            clock_format: if am_pm { "+%a. %-I:%M %p" } else { "+%a. %H:%M" },
            counter: 0,
            user: std::env::var("USER").unwrap_or_else(|_| String::from("nutzer")),
            deeds: Vec::new(),
        }
    }

    fn measure(&mut self) -> Stand {
        self.sys.refresh_cpu_usage();
        let mut neu = Stand {
            focus: focus_read(),
            clock: mk::command::first_line("date", &[self.clock_format]).unwrap_or_default(),
            volume: volume_read(),
            cpu: self.sys.global_cpu_usage() as u8,
            ..self.stand.clone()
        };
        // Network and battery change slowly — every 5th pulse (10 s).
        self.counter = self.counter.wrapping_add(1);
        if self.counter % 5 == 0 || neu.net.is_none() {
            neu.net = net_read();
            neu.battery = battery_read();
        }
        neu
    }
}

/// A bar pill: symbol and/or text, vertically centred.
#[allow(clippy::too_many_arguments)]
fn pill(
    engine: &mut TextEngine,
    b: &mut Build,
    p: mk::Palette,
    glyph: Option<char>,
    text: Option<&str>,
    x: f32,
    role: Option<usize>,
) -> f32 {
    let clear = mk::Rgba { a: 0.0, ..p.surface };
    let cal = mk::typo::CALLOUT;
    let symfam = mkw::symbol_family();
    let mut children = Vec::new();
    let mut gm = (0.0, 0.0);
    if let Some(g) = glyph {
        gm = engine.measure_family(symfam, &g.to_string(), mk::typo::symbol_px(16.0), mk::typo::Weight::Normal);
        children.push(Node::Quad { w: gm.0, h: gm.1, color: clear, radius: 0.0 });
    }
    let mut tm = (0.0, 0.0);
    if let Some(t) = text {
        tm = engine.measure_styled(t, cal.px(), cal.weight);
        children.push(Node::Quad { w: tm.0, h: tm.1, color: clear, radius: 0.0 });
    }
    let node = Node::Wrap {
        pad_x: 9.0,
        pad_y: 5.0,
        background: Some((clear, mk::radius::SMALL)),
        child: Box::new(Node::Row { gap: 6.0, padding: 0.0, background: None, children }),
    };
    let mut tmp = Vec::new();
    let (pw, ph) = lay_out(&node, 0.0, 0.0, &mut tmp);
    let first = b.scene.len();
    let mut placed = Vec::new();
    lay_out(&node, x, (HEIGHT as f32 - ph) / 2.0, &mut placed);
    let n = placed.len();
    b.scene.extend(placed);
    b.roles.extend(std::iter::repeat(role).take(n));
    let home = Rect::of(&b.scene[first]);
    let mut slot = first + 1;
    if let Some(g) = glyph {
        let q = b.scene[slot];
        b.texts.push(Text {
            s: g.to_string(),
            px: mk::typo::symbol_px(16.0),
            weight: mk::typo::Weight::Normal,
            family: symfam,
            color: p.on_surface_variant,
            x: q.x,
            y: q.y,
            home,
            anchor: slot,
            clip: None,
        });
        slot += 1;
        let _ = gm;
    }
    if let Some(t) = text {
        let q = b.scene[slot];
        b.texts.push(Text {
            s: t.to_string(),
            px: cal.px(),
            weight: cal.weight,
            family: matrixkit_text::FAMILY,
            color: p.on_surface,
            x: q.x,
            y: q.y + (gm.1 - tm.1).max(0.0) / 2.0,
            home,
            anchor: slot,
            clip: None,
        });
    }
    pw
}

impl shell::Client for Bar {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn view(
        &mut self,
        engine: &mut TextEngine,
        _keep: &mut Keep<f32>,
        _atlas: &mut matrixkit_image::ImageAtlas,
        w: u32,
        _h: u32,
    ) -> Scene {
        let p = self.palette;
        let mut b = Build::default();
        self.deeds = Vec::new();

        // The bar's ground: ONE rounded strip (ARGB — the edges
        // of the mount let the wall show through).
        let mut placed = Vec::new();
        lay_out(
            &Node::Quad {
                w: w as f32,
                h: HEIGHT as f32,
                color: p.surface_container,
                radius: 14.0,
            },
            0.0,
            0.0,
            &mut placed,
        );
        b.scene.extend(placed);
        b.roles.push(None);

        // A widget → pill (symbol/text/deed); whatever has not moved yet
        // (tray, media, recording) silently drops out — reminder.
        let s = self.stand.clone();
        let user = self.user.clone();
        let zones = self.zones.clone();

        // Left, centre, right — plain and honest.
        let (left, mid, right) = (zones.0, zones.1, zones.2);
        let mut x = INSET + 4.0;
        for item in items_of(&left, &s, &user) {
            let role = item.2.map(|d| self_deed(&mut self.deeds, d));
            let pw = pill(engine, &mut b, p, item.0, opt(&item.1), x, role);
            x += pw + 4.0;
        }
        // Centre: measure first, then place centred.
        let mid_items = items_of(&mid, &s, &user);
        let mut mid_w = 0.0;
        for item in &mid_items {
            let mut probe = Build::default();
            mid_w += pill(engine, &mut probe, p, item.0, opt(&item.1), 0.0, None) + 4.0;
        }
        let mut mx = (w as f32 - mid_w) / 2.0;
        for item in mid_items {
            let role = item.2.map(|d| self_deed(&mut self.deeds, d));
            let pw = pill(engine, &mut b, p, item.0, opt(&item.1), mx, role);
            mx += pw + 4.0;
        }
        // Right: from the edge inward.
        let mut rx = w as f32 - INSET - 4.0;
        let mut right_items = items_of(&right, &s, &user);
        right_items.reverse();
        for item in right_items {
            let role = item.2.map(|d| self_deed(&mut self.deeds, d));
            let mut probe = Build::default();
            let pw = pill(engine, &mut probe, p, item.0, opt(&item.1), 0.0, None);
            rx -= pw;
            pill(engine, &mut b, p, item.0, opt(&item.1), rx, role);
            rx -= 4.0;
        }

        // The SESSION MENU (camp 43): the surface has grown to
        // screen height, the free area closes, the
        // panel hangs on the right below the user pill. Anything final
        // turns red on the first click — only the second counts.
        if self.menu {
            let back = self_deed(&mut self.deeds, Deed::Backdrop);
            let mut placed = Vec::new();
            lay_out(
                &Node::Quad {
                    w: w as f32,
                    h: MENU_HEIGHT as f32,
                    color: mk::Rgba { a: 0.0, ..p.surface },
                    radius: 0.0,
                },
                0.0,
                HEIGHT as f32,
                &mut placed,
            );
            b.scene.extend(placed);
            b.roles.push(Some(back));

            let mx = w as f32 - MENU_W - INSET;
            let mut my = HEIGHT as f32 + 8.0;
            let panel_first = b.scene.len();
            let mut placed = Vec::new();
            lay_out(
                &Node::Quad { w: MENU_W, h: 10.0, color: p.surface, radius: 14.0 },
                mx,
                my,
                &mut placed,
            );
            b.scene.extend(placed);
            b.roles.push(None);
            my += 8.0;
            let symfam = mkw::symbol_family();
            let body = mk::typo::BODY;
            for a in Action::ALLE {
                let armed = self.confirmation == Some(a);
                let deed_i = self_deed(&mut self.deeds, Deed::Act(a));
                let tint = if armed { p.error } else { p.on_surface };
                let title = if armed { "Wirklich?" } else { a.title() };
                let first = b.scene.len();
                let mut placed = Vec::new();
                lay_out(
                    &Node::Quad {
                        w: MENU_W - 12.0,
                        h: MENU_ROW - 4.0,
                        color: if armed {
                            p.error.over(p.surface_container_high, 0.15)
                        } else {
                            mk::Rgba { a: 0.0, ..p.surface }
                        },
                        radius: mk::radius::SMALL,
                    },
                    mx + 6.0,
                    my,
                    &mut placed,
                );
                b.scene.extend(placed);
                b.roles.push(Some(deed_i));
                let home = Rect::of(&b.scene[first]);
                let gs = a.glyph().to_string();
                let (_, gh) = engine.measure_family(symfam, &gs, mk::typo::symbol_px(17.0), mk::typo::Weight::Normal);
                b.texts.push(Text {
                    s: gs,
                    px: mk::typo::symbol_px(17.0),
                    weight: mk::typo::Weight::Normal,
                    family: symfam,
                    color: tint,
                    x: mx + 18.0,
                    y: my + (MENU_ROW - 4.0 - gh) / 2.0,
                    home,
                    anchor: first,
                    clip: None,
                });
                let (_, th) = engine.measure_styled(title, body.px(), body.weight);
                b.texts.push(Text {
                    s: title.to_string(),
                    px: body.px(),
                    weight: body.weight,
                    family: matrixkit_text::FAMILY,
                    color: tint,
                    x: mx + 48.0,
                    y: my + (MENU_ROW - 4.0 - th) / 2.0,
                    home,
                    anchor: first,
                    clip: None,
                });
                my += MENU_ROW;
            }
            my += 4.0;
            b.scene[panel_first].h = my - (HEIGHT as f32 + 8.0);
        }

        Scene { placed: b.scene, roles: b.roles, texts: b.texts, picts: Vec::new(), curves: Vec::new() }
    }

    fn extent(&mut self) -> Option<(u32, u32)> {
        // The menu needs room: the surface grows to screen height
        // (like iced MENU_HEIGHT), closed it shrinks to the strip.
        Some((0, if self.menu { MENU_HEIGHT + HEIGHT } else { HEIGHT }))
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, source: Source) -> Refresh {
        let Some(deed) = role.and_then(|r| self.deeds.get(r)) else { return Refresh::Calm };
        let origin = source.label();
        match deed {
            Deed::Bell => {
                // The bell protocol: the notification history
                // open/closed (file mtime, as before).
                let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| String::from("/tmp"));
                let _ = std::fs::write(format!("{dir}/matrix-notifications-show"), b"");
                eprintln!("[bar] Treffer ({origin}): Glocke — Historie umgeschaltet");
            }
            Deed::Central => {
                eprintln!("[bar] Treffer ({origin}): Zentrale");
                let _ = mk::process::spawn_reaped(&mut std::process::Command::new("matrix-central")); // round 51
            }
            Deed::User => {
                self.menu = !self.menu;
                self.confirmation = None;
                eprintln!(
                    "[bar] Treffer ({origin}): Sitzungs-Menü {}",
                    if self.menu { "auf" } else { "zu" }
                );
                return Refresh::Dirty;
            }
            Deed::Act(a) => {
                let a = *a;
                if a.permanent() && self.confirmation != Some(a) {
                    // Anything final wants the second, red click.
                    eprintln!("[bar] Menü ({origin}): „{}“ — Wirklich? (zweiter Klick gilt)", a.title());
                    self.confirmation = Some(a);
                    return Refresh::Dirty;
                }
                eprintln!("[bar] Menü ({origin}): „{}“ wird ausgeführt", a.title());
                a.execute();
                self.menu = false;
                self.confirmation = None;
                return Refresh::Dirty;
            }
            Deed::Backdrop => {
                eprintln!("[bar] Menü: freie Fläche — schließt");
                self.menu = false;
                self.confirmation = None;
                return Refresh::Dirty;
            }
            Deed::Matrix => {
                eprintln!("[bar] Treffer ({origin}): Matrix-Menü (Merkposten)");
            }
        }
        Refresh::Calm
    }

    fn pulse(&mut self) -> Refresh {
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
            self.palette = fresh;
            self.zones = widgets_read();
            self.stand = self.measure();
            return Refresh::Full;
        }
        self.zones = widgets_read();
        let neu = self.measure();
        if neu != self.stand {
            self.stand = neu;
            return Refresh::Dirty;
        }
        Refresh::Calm
    }
}

fn opt(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn self_deed(deeds: &mut Vec<Deed>, d: Deed) -> usize {
    deeds.push(d);
    deeds.len() - 1
}

/// Widgets of a zone → pill ingredients (symbol, text, deed).
fn items_of(list: &[Widget], s: &Stand, user: &str) -> Vec<(Option<char>, String, Option<Deed>)> {
    let mut out = Vec::new();
    for wgt in list {
        let item = match wgt {
            Widget::Matrix => Some((Some(mkw::symbol::APPS), String::new(), Some(Deed::Matrix))),
            Widget::Focus => {
                Some((None, s.focus.clone().unwrap_or_else(|| String::from("Matrix")), None))
            }
            Widget::Clock => Some((None, s.clock.clone(), None)),
            Widget::Pulse => {
                let mut parts: Vec<String> = vec![format!("{} %", s.cpu)];
                if let Some(v) = &s.volume {
                    parts.push(format!("Ton {v}"));
                }
                if let Some(n) = &s.net {
                    parts.push(n.clone());
                }
                Some((None, parts.join("  ·  "), None))
            }
            Widget::Battery => s.battery.map(|(pct, loading)| {
                let g = if loading {
                    mkw::symbol::BATTERY_CHARGING
                } else {
                    mkw::symbol::BATTERY_FULL
                };
                (Some(g), format!("{pct} %"), None)
            }),
            Widget::Bell => Some((Some(mkw::symbol::NOTIFICATIONS), String::new(), Some(Deed::Bell))),
            Widget::Central => Some((Some(mkw::symbol::TUNE), String::new(), Some(Deed::Central))),
            Widget::User => Some((Some(mkw::symbol::PERSON), user.to_string(), Some(Deed::User))),
            _ => None,
        };
        if let Some(it) = item {
            out.push(it);
        }
    }
    out
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[bar] Matrix Bar: siebter App-Umzug — die Bar im Ersten Licht, pid={}", std::process::id());
    let reserve = std::env::var("LICHT_RESERVE").is_ok();
    shell::run(
        Bar::new(),
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-bar"),
                layer: shell::Layer::Top,
                keyboard: shell::KeyboardWish::None,
                anchor: shell::Anchors { top: true, right: true, bottom: false, left: true },
                size: (0, HEIGHT),
                margin: (10, 10, 0, 10),
                // Shadow mode floats; LICHT_RESERVE arms
                // the zone (windows move below the bar).
                exclusive: if reserve { HEIGHT as i32 + 10 } else { 0 },
            },
            argb: true,
            tag: "bar",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
