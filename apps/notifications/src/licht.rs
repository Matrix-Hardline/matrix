//! Matrix Mitteilungen (notifications), First Light version (iced replacement,
//! camp 26 — GRABUNG-ICED-ABLOESUNG.md): the fifth app move —
//! the first DAEMON, and the surface learns size and anchor.
//!
//! The DBus service is verbatim that of the iced version (the same
//! zbus thread, the same service interface) — only its sender
//! is now the LOOM: every notification flows via Loom::feed
//! into the loop and comes home as woven(). The surface anchors
//! top right (panel storey: anchors + size + margin) and GROWS
//! with its stack — view() measures, extent() wishes, the
//! configure brings the new size.
//!
//! Popups linger 5 s (beat sweep), the bar's bell opens
//! the history (file log as before), groups fold
//! behind their newest card (R114), action buttons send
//! ActionInvoked back — the freedesktop contract, without iced.
//!
//! Shadow probe: without LICHT_NAME the client does NOT take over the
//! well-known name — the session's running service
//! stays untouched, the probe addresses the unique name.
//!
//! Since camp 27 the notification text WRAPS (TextEngine::wrap,
//! word by word by measurement, at most four lines) — the pill
//! grows along, the surface follows via extent().
//!
//! Reminder: panel_toggle at the default switch.

use crate::{
    dnd_on, hide_path, inbox_sync, newest_mtime, service_start, show_path, show_path_legacy, Command, Event,
    Notification, DWELL_TIME_MS, HISTORY_MAX, WIDTH,
};
use matrixkit_shell::{self as shell, Pict, Refresh, Scene, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{lay_out, Keep, Node, Placed, Rect};
use matrixkit_widgets as mkw;
use matrixkit_widgets::scene as fam;
use std::sync::mpsc;

/// The pill body (the surface is WIDTH, air stays on the left).
const PILL_W: f32 = WIDTH as f32 - 8.0;
const AIR: f32 = 8.0;
const ICON: f32 = 36.0;
/// Maximum height of the surface (like the iced version).
const MAX_H: u32 = 1400;

/// What a hit means — the scene's roles point here.
enum Deed {
    Dismiss(u32),
    Action(u32, String),
    DndToggle,
    ClearAll,
    HistoryClose,
    Expand(String),
    Collapse(String),
}

/// The growing scene of one view() run, bundled.
#[derive(Default)]
struct Build {
    scene: Vec<Placed>,
    roles: Vec<Option<usize>>,
    texts: Vec<Text>,
    picts: Vec<Pict>,
}

struct Notifications {
    loom: Option<shell::Loom<Event>>,
    command: Option<mpsc::Sender<Command>>,
    palette: mk::Palette,
    stack: Vec<(Notification, std::time::Instant)>,
    history: Vec<Notification>,
    history_open: bool,
    expanded: std::collections::HashSet<String>,
    dnd: bool,
    /// Rendered app icons (36×36 RGBA) — None: no icon, the
    /// initial carries it.
    icons: std::collections::HashMap<String, Option<(u32, u32, Vec<u8>)>>,
    /// Which icons of this palette epoch are already in the atlas.
    puts: std::collections::HashSet<String>,
    /// Palette epoch: a change = new atlas keys, old ones fall
    /// to the sweep.
    epoch: u64,
    last_show: Option<std::time::SystemTime>,
    last_hide: Option<std::time::SystemTime>,
    /// The desired size — measured by view(), named by extent().
    want: (u32, u32),
    /// The hit meanings of the last scene (role = index).
    deeds: Vec<Deed>,
}

impl Notifications {
    fn new() -> Self {
        Notifications {
            loom: None,
            command: None,
            palette: mk::Palette::load().unwrap_or_default(),
            stack: Vec::new(),
            history: Vec::new(),
            history_open: false,
            expanded: std::collections::HashSet::new(),
            dnd: dnd_on(),
            icons: std::collections::HashMap::new(),
            puts: std::collections::HashSet::new(),
            epoch: 0,
            last_show: newest_mtime(&[show_path(), show_path_legacy()]),
            last_hide: std::fs::metadata(hide_path()).and_then(|m| m.modified()).ok(),
            want: (1, 1),
            deeds: Vec::new(),
        }
    }

    fn icon_backen(&mut self, app: &str) {
        let p = self.palette;
        self.icons.entry(app.to_string()).or_insert_with(|| {
            matrixkit_icons::render_png(app, &p)
                .and_then(|png| matrixkit_image::ImageAtlas::unpack(&png, ICON as u32, ICON as u32))
        });
    }

    fn icon_key(&self, app: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("\u{0}icon:{}:{}", app, self.epoch))
    }

    /// R114: byApp groups above the history, newest group first
    /// (cloned — view builds from the snapshot).
    fn history_groups(&self) -> Vec<(String, Vec<Notification>)> {
        let mut groups: Vec<(String, Vec<Notification>)> = Vec::new();
        for m in &self.history {
            if let Some(g) = groups.iter_mut().find(|(a, _)| *a == m.app) {
                g.1.push(m.clone());
            } else {
                groups.push((m.app.clone(), vec![m.clone()]));
            }
        }
        groups
    }

    fn kurz(s: &str, max: usize) -> String {
        mk::elide(s, max + 1)
    }

    /// Lay out a notification pill: ground, icon (atlas or
    /// initial), title, text, action buttons. Returns the height.
    #[allow(clippy::too_many_arguments)]
    fn pill(
        &mut self,
        engine: &mut TextEngine,
        atlas: &mut matrixkit_image::ImageAtlas,
        b: &mut Build,
        m: &Notification,
        with_actions: bool,
        deed: usize,
        x: f32,
        y: f32,
    ) -> f32 {
        let p = self.palette;
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let body = mk::typo::BODY;
        let cap = mk::typo::CAPTION;
        let head = mk::typo::HEADLINE;

        // The icon: rendered (atlas) or the initial on
        // primary_container.
        self.icon_backen(&m.app);
        let icon_data = self.icons.get(&m.app).cloned().flatten();
        let has_image = icon_data.is_some();
        if let Some((iw, ih, rgba)) = icon_data {
            if !self.puts.contains(&m.app) {
                atlas.put(&self.icon_key(&m.app), iw, ih, rgba);
                self.puts.insert(m.app.clone());
            }
        }
        let initial = m.app.chars().next().unwrap_or('•').to_uppercase().to_string();
        let (in_w, in_h) = engine.measure_styled(&initial, head.px(), head.weight);
        let icon_node = if has_image {
            Node::Quad { w: ICON, h: ICON, color: clear, radius: 0.0 }
        } else {
            Node::Wrap {
                pad_x: (ICON - in_w) / 2.0,
                pad_y: (ICON - in_h) / 2.0,
                background: Some((p.primary_container, mk::radius::SMALL + 2.0)),
                child: Box::new(Node::Quad { w: in_w, h: in_h, color: clear, radius: 0.0 }),
            }
        };
        let icon_n = if has_image { 1 } else { 2 };

        // Title (one line) + text WRAPPED (camp 27): the
        // column wraps word by word, at most four lines — then
        // the last one is truncated (the UNC extract of the iced version).
        // Like the reference: padding S on both sides (panel_shell), then
        // symbol and gap.
        let text_w = PILL_W - 2.0 * mk::spacing::S - ICON - mk::spacing::M;
        let title = Self::kurz(&m.title, 36);
        // measure_styled already returns the LINE height
        // (lines * px * LINE_HEIGHT), not the ink — recomputed
        // on 17.08.2026, when I wrongly multiplied it myself.
        let (tw, th) = engine.measure_styled(&title, body.px(), mk::typo::Weight::Semibold);
        let mut col: Vec<Node> = vec![Node::Quad { w: tw, h: th, color: clear, radius: 0.0 }];
        let text = {
            let wrapped = engine.wrap_styled(&m.text, cap.px(), cap.weight, text_w);
            let lines: Vec<&str> = wrapped.split('\n').collect();
            if lines.len() > 4 {
                let mut t = lines[..4].join("\n");
                t.push('…');
                t
            } else {
                wrapped
            }
        };
        let has_text = !text.is_empty();
        let (xw, xh) = engine.measure_styled(&text, cap.px(), cap.weight);
        if has_text {
            col.push(Node::Quad { w: xw, h: xh, color: clear, radius: 0.0 });
        }

        // Action buttons (without "default") — the button family.
        let mut labels = Vec::new();
        let actions: Vec<(String, String)> = if with_actions {
            m.actions.iter().filter(|(k, _)| k != "default").cloned().collect()
        } else {
            Vec::new()
        };
        if !actions.is_empty() {
            use matrixkit_widgets::buttons::{Role, Size, Style};
            let mut row = Vec::new();
            for (_, label) in &actions {
                let bt = fam::Button {
                    label: label.clone(),
                    style: Style::Tinted,
                    role: Role::Normal,
                    size: Size::Normal,
                    active: true, ..Default::default() };
                let (node, l) = fam::button(engine, &bt, fam::Status::Idle, p);
                labels.push(l);
                row.push(node);
            }
            col.push(Node::Row {
                gap: mk::spacing::XS,
                padding: 0.0,
                background: None,
                children: row,
            });
        }

        // The SHELL comes from the family (fam::panel_shell), as
        // in the reference (mkw::ui::panel_shell): full width,
        // padding S, ring and ground. Built by hand it was 36
        // logical pixels narrower and padded with 14 instead of 8 —
        // the text wrapped where the reference shows one line
        // (finger test 17.08.2026).
        let node = fam::panel_shell(
            Node::Row {
                gap: mk::spacing::M,
                padding: 0.0,
                background: None,
                children: vec![
                    icon_node,
                    Node::Column {
                        // The reference sets title and text WITHOUT
                        // a gap (column![..] with default 0);
                        // with XS the card was four physical pixels
                        // taller (finger test 17.08.2026).
                        gap: 0.0,
                        padding: 0.0,
                        background: None,
                        children: col,
                    },
                ],
            },
            PILL_W,
            p,
        );

        let first = b.scene.len();
        let mut placed = Vec::new();
        let (_, ph) = lay_out(&node, x, y, &mut placed);
        let n = placed.len();
        b.scene.extend(placed);
        // Everything on the pill means "swipe away" — only the buttons
        // overrule with their action. Placed order: ground,
        // icon (1/2), title, text (0/1), 2 per button.
        b.roles.extend(std::iter::repeat(Some(deed)).take(n));
        let text_n = if has_text { 1 } else { 0 };
        // fam::panel_shell puts ring AND ground before the content —
        // so the first child Placed lies two further on. ONE name
        // for it, instead of the number in four places (camp 70: the
        // anchor carries the depth; whoever miscounts it paints into
        // the void).
        let inhalt = 2usize;
        let btn0 = inhalt + icon_n + 1 + text_n;
        for (i, (key, _)) in actions.iter().enumerate() {
            let deed_i = self.deeds.len();
            self.deeds.push(Deed::Action(m.id, key.clone()));
            for k in 0..fam::PLACED_PER_BUTTON {
                b.roles[first + btn0 + i * fam::PLACED_PER_BUTTON + k] = Some(deed_i);
            }
        }

        let home = Rect::of(&b.scene[first]);
        if has_image {
            let iq = b.scene[first + inhalt];
            b.picts.push(Pict {
                path: self.icon_key(&m.app),
                box_w: ICON as u32,
                box_h: ICON as u32,
                fit: matrixkit_image::Fit::Cover,
                x: iq.x,
                y: iq.y,
                radius: mk::radius::SMALL + 2.0,
                home,
                anchor: first + inhalt,
                clip: None,
            });
        } else {
            let lq = b.scene[first + inhalt + 1];
            b.texts.push(Text {
                s: initial,
                px: head.px(),
                weight: head.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_primary_container,
                x: lq.x,
                y: lq.y,
                home,
                anchor: first + inhalt + 1,
                clip: None,
            });
        }
        let tq = b.scene[first + inhalt + icon_n];
        b.texts.push(Text {
            s: title,
            px: body.px(),
            weight: mk::typo::Weight::Semibold,
            family: matrixkit_text::FAMILY,
            color: p.on_surface,
            x: tq.x,
            y: tq.y,
            home,
            anchor: first + inhalt + icon_n,
            clip: None,
        });
        if has_text {
            let xq = b.scene[first + inhalt + icon_n + 1];
            b.texts.push(Text {
                s: text,
                px: cap.px(),
                weight: cap.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface_variant,
                x: xq.x,
                y: xq.y,
                home,
                anchor: first + inhalt + icon_n + 1,
                clip: None,
            });
        }
        for (i, l) in labels.into_iter().enumerate() {
            let slot = first + btn0 + i * fam::PLACED_PER_BUTTON;
            let lq = b.scene[slot + 1];
            b.texts.push(Text {
                s: l.text,
                px: l.px,
                weight: l.weight,
                family: l.family,
                color: l.color,
                x: lq.x,
                y: lq.y,
                home,
                anchor: slot + 1,
                clip: None,
            });
        }
        ph
    }

    /// A header button of the history (symbol or text run), aligned
    /// right; `bx` moves on to the left.
    #[allow(clippy::too_many_arguments)]
    fn kopf(
        &mut self,
        engine: &mut TextEngine,
        b: &mut Build,
        glyph: Option<char>,
        label: Option<&str>,
        deed: Deed,
        bx: &mut f32,
    ) {
        let p = self.palette;
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let deed_i = self.deeds.len();
        self.deeds.push(deed);
        let (node, l) = match (glyph, label) {
            (Some(g), _) => fam::tool_button(engine, g, true, fam::Status::Idle, p),
            (_, Some(t)) => {
                let cap = mk::typo::CAPTION;
                let (lw, lh) = engine.measure_styled(t, cap.px(), cap.weight);
                (
                    Node::Wrap {
                        pad_x: 8.0,
                        pad_y: 5.0,
                        background: Some((clear, mk::radius::SMALL)),
                        child: Box::new(Node::Quad { w: lw, h: lh, color: clear, radius: 0.0 }),
                    },
                    fam::Label {
                        text: t.to_string(),
                        px: cap.px(),
                        weight: cap.weight,
                        family: matrixkit_text::FAMILY,
                        color: p.primary,
                    },
                )
            }
            _ => unreachable!(),
        };
        let mut tmp = Vec::new();
        let (bw, _) = lay_out(&node, 0.0, 0.0, &mut tmp);
        *bx -= bw;
        let first = b.scene.len();
        let mut placed = Vec::new();
        lay_out(&node, *bx, mk::spacing::M - 5.0, &mut placed);
        let n = placed.len();
        b.scene.extend(placed);
        b.roles.extend(std::iter::repeat(Some(deed_i)).take(n));
        let lq = b.scene[first + 1];
        b.texts.push(Text {
            s: l.text,
            px: l.px,
            weight: l.weight,
            family: l.family,
            color: l.color,
            x: lq.x,
            y: lq.y,
            home: Rect::of(&b.scene[first]),
            anchor: first + 1,
            clip: None,
        });
        *bx -= mk::spacing::XS;
    }
}

impl shell::Client for Notifications {
    type Card = ();
    type Work = Event;

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn warp(&mut self, loom: &shell::Loom<Event>) {
        self.loom = Some(loom.clone());
        let (btx, brx) = mpsc::channel();
        self.command = Some(btx);
        // The service of the iced version, verbatim — only its sender
        // is the loom. Without LICHT_NAME the well-known
        // name stays with the running service (shadow probe).
        let feeder = loom.clone();
        // Round 31: the house is the default service — it claims the
        // well-known name; only a shadow probe (LICHT_SCHATTEN) leaves it
        // with the running service.
        let claim = std::env::var("LICHT_SCHATTEN").is_err();
        service_start(move |e| feeder.feed(e), brx, claim);
    }

    fn woven(&mut self, e: Event) -> Refresh {
        match e {
            Event::New(m) => {
                eprintln!("[mitteilung] Mitteilung {} von {}: „{}“", m.id, m.app, m.title);
                self.icon_backen(&m.app);
                self.history.retain(|v| v.id != m.id);
                self.history.insert(0, m.clone());
                self.history.truncate(HISTORY_MAX);
                inbox_sync(&mut self.history, self.command.as_ref(), true);
                if !self.dnd {
                    let photo = m.title.contains("creenshot")
                        || m.title.contains("ildschirmfoto")
                        || m.app.contains("niri");
                    // Round 105: the bar shows it as its island — no popup
                    // of our own (sound and history stay).
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
            Event::Gone(id) => {
                eprintln!("[mitteilung] Mitteilung {id} zurückgezogen");
                self.stack.retain(|(m, _)| m.id != id);
            }
        }
        Refresh::Dirty
    }

    fn view(
        &mut self,
        engine: &mut TextEngine,
        _keep: &mut Keep<f32>,
        atlas: &mut matrixkit_image::ImageAtlas,
        _w: u32,
        _h: u32,
    ) -> Scene {
        let p = self.palette;
        let clear = mk::Rgba { a: 0.0, ..p.surface };
        let mut b = Build::default();
        self.deeds = Vec::new();

        let want_h;
        if self.history_open {
            // ------------------------------------------ the HISTORY
            // The panel plate grows to the measured height at the end.
            let shell_first = b.scene.len();
            let mut placed = Vec::new();
            lay_out(
                &Node::Quad { w: WIDTH as f32, h: 10.0, color: p.surface, radius: 16.0 },
                0.0,
                0.0,
                &mut placed,
            );
            b.scene.extend(placed);
            b.roles.push(None);

            let head = mk::typo::HEADLINE;
            let (tw, th) = engine.measure_styled("Mitteilungen", head.px(), head.weight);
            let tfirst = b.scene.len();
            let mut placed = Vec::new();
            lay_out(
                &Node::Quad { w: tw, h: th, color: clear, radius: 0.0 },
                mk::spacing::M,
                mk::spacing::M,
                &mut placed,
            );
            b.scene.extend(placed);
            b.roles.push(None);
            b.texts.push(Text {
                s: String::from("Mitteilungen"),
                px: head.px(),
                weight: head.weight,
                family: matrixkit_text::FAMILY,
                color: p.on_surface,
                x: mk::spacing::M,
                y: mk::spacing::M,
                home: Rect::of(&b.scene[tfirst]),
                anchor: tfirst,
                clip: None,
            });

            // The header buttons, from the right: ×, clear, mute.
            let mut bx = WIDTH as f32 - mk::spacing::M;
            let dnd_glyph =
                if self.dnd { mkw::symbol::VOLUME_OFF } else { mkw::symbol::NOTIFICATIONS };
            self.kopf(engine, &mut b, Some(mkw::symbol::CLOSE), None, Deed::HistoryClose, &mut bx);
            self.kopf(engine, &mut b, None, Some("Leeren"), Deed::ClearAll, &mut bx);
            self.kopf(engine, &mut b, Some(dnd_glyph), None, Deed::DndToggle, &mut bx);

            let mut y = mk::spacing::M + th + mk::spacing::M;
            if self.history.is_empty() {
                let cap = mk::typo::CALLOUT;
                let s = "Keine Mitteilungen";
                let (lw, lh) = engine.measure_styled(s, cap.px(), cap.weight);
                let first = b.scene.len();
                let mut placed = Vec::new();
                lay_out(
                    &Node::Quad { w: lw, h: lh, color: clear, radius: 0.0 },
                    (WIDTH as f32 - lw) / 2.0,
                    y + mk::spacing::L,
                    &mut placed,
                );
                b.scene.extend(placed);
                b.roles.push(None);
                b.texts.push(Text {
                    s: s.to_string(),
                    px: cap.px(),
                    weight: cap.weight,
                    family: matrixkit_text::FAMILY,
                    color: p.on_surface_variant,
                    x: (WIDTH as f32 - lw) / 2.0,
                    y: y + mk::spacing::L,
                    home: Rect::of(&b.scene[first]),
                    anchor: first,
                    clip: None,
                });
                y += mk::spacing::L * 2.0 + lh;
            } else {
                for (app, items) in self.history_groups() {
                    if items.len() == 1 {
                        let deed_i = self.deeds.len();
                        self.deeds.push(Deed::Dismiss(items[0].id));
                        let ph =
                            self.pill(engine, atlas, &mut b, &items[0], false, deed_i, AIR, y);
                        y += ph + mk::spacing::XS;
                    } else if self.expanded.contains(&app) {
                        for m in &items {
                            let deed_i = self.deeds.len();
                            self.deeds.push(Deed::Dismiss(m.id));
                            let ph = self.pill(engine, atlas, &mut b, m, false, deed_i, AIR, y);
                            y += ph + mk::spacing::XS;
                        }
                        // "Weniger anzeigen" (show less) folds back.
                        let cap = mk::typo::CAPTION;
                        let s = "Weniger anzeigen";
                        let (lw, lh) = engine.measure_styled(s, cap.px(), cap.weight);
                        let deed_i = self.deeds.len();
                        self.deeds.push(Deed::Collapse(app.clone()));
                        let first = b.scene.len();
                        let mut placed = Vec::new();
                        lay_out(
                            &Node::Quad { w: lw, h: lh, color: clear, radius: 0.0 },
                            (WIDTH as f32 - lw) / 2.0,
                            y + 4.0,
                            &mut placed,
                        );
                        b.scene.extend(placed);
                        b.roles.push(Some(deed_i));
                        b.texts.push(Text {
                            s: s.to_string(),
                            px: cap.px(),
                            weight: cap.weight,
                            family: matrixkit_text::FAMILY,
                            color: p.primary,
                            x: (WIDTH as f32 - lw) / 2.0,
                            y: y + 4.0,
                            home: Rect::of(&b.scene[first]),
                            anchor: first,
                            clip: None,
                        });
                        y += lh + 8.0 + mk::spacing::XS;
                    } else {
                        // The folded group: newest card, "+n"
                        // capsule, peek edges below (R114).
                        let deed_i = self.deeds.len();
                        self.deeds.push(Deed::Expand(app.clone()));
                        let ph =
                            self.pill(engine, atlas, &mut b, &items[0], false, deed_i, AIR, y);
                        let label = mk::typo::LABEL;
                        let badge = format!("+{}", items.len() - 1);
                        let (bw2, bh2) = engine.measure_styled(&badge, label.px(), label.weight);
                        let bnode = Node::Wrap {
                            pad_x: 6.0,
                            pad_y: 1.0,
                            background: Some((p.primary, mk::radius::capsule(bh2 + 2.0))),
                            child: Box::new(Node::Quad {
                                w: bw2,
                                h: bh2,
                                color: clear,
                                radius: 0.0,
                            }),
                        };
                        let mut tmp = Vec::new();
                        let (bnw, _) = lay_out(&bnode, 0.0, 0.0, &mut tmp);
                        let first = b.scene.len();
                        let mut placed = Vec::new();
                        lay_out(&bnode, AIR + PILL_W - bnw - 10.0, y + 6.0, &mut placed);
                        let n = placed.len();
                        b.scene.extend(placed);
                        b.roles.extend(std::iter::repeat(Some(deed_i)).take(n));
                        b.texts.push(Text {
                            s: badge,
                            px: label.px(),
                            weight: label.weight,
                            family: matrixkit_text::FAMILY,
                            color: p.on_primary,
                            x: b.scene[first + 1].x,
                            y: b.scene[first + 1].y,
                            home: Rect::of(&b.scene[first]),
                            anchor: first + 1,
                            clip: None,
                        });
                        y += ph + 2.0;
                        // The edges: getting narrower, only the hem.
                        let edges: &[(f32, f32, f32)] = if items.len() > 2 {
                            &[(PILL_W - 28.0, 5.0, 0.8), (PILL_W - 56.0, 4.0, 0.5)]
                        } else {
                            &[(PILL_W - 28.0, 5.0, 0.8)]
                        };
                        for (ew, eh, ea) in edges {
                            let mut placed = Vec::new();
                            lay_out(
                                &Node::Quad {
                                    w: *ew,
                                    h: *eh,
                                    color: mk::Rgba { a: *ea, ..p.surface_container },
                                    radius: 6.0,
                                },
                                AIR + (PILL_W - ew) / 2.0,
                                y,
                                &mut placed,
                            );
                            b.scene.extend(placed);
                            b.roles.push(Some(deed_i));
                            y += eh + 2.0;
                        }
                        y += mk::spacing::XS;
                    }
                }
            }
            y += mk::spacing::S;
            // The panel plate below everything grows to the height.
            b.scene[shell_first].h = y;
            want_h = y.ceil() as u32;
        } else {
            // ------------------------------------------- the STACK
            let mut y = 0.0;
            for (m, _) in self.stack.clone() {
                let deed_i = self.deeds.len();
                self.deeds.push(Deed::Dismiss(m.id));
                let ph = self.pill(engine, atlas, &mut b, &m, true, deed_i, AIR, y);
                y += ph + AIR;
            }
            want_h = if y > 0.0 { y.ceil() as u32 } else { 1 };
        }

        self.want = if want_h > 1 { (WIDTH, want_h.min(MAX_H)) } else { (1, 1) };
        Scene { placed: b.scene, roles: b.roles, texts: b.texts, picts: b.picts, curves: Vec::new() }
    }

    fn extent(&mut self) -> Option<(u32, u32)> {
        Some(self.want)
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, source: Source) -> Refresh {
        let Some(deed) = role.and_then(|r| self.deeds.get(r)) else { return Refresh::Calm };
        let origin = source.label();
        match deed {
            Deed::Dismiss(id) => {
                eprintln!("[mitteilung] Treffer ({origin}): Mitteilung {id} weggewischt");
                let id = *id;
                self.stack.retain(|(m, _)| m.id != id);
                if let Some(c) = &self.command {
                    let _ = c.send(Command::Closed(id, 2));
                }
            }
            Deed::Action(id, key) => {
                eprintln!("[mitteilung] Treffer ({origin}): Aktion „{key}“ für {id} → Signal");
                if let Some(c) = &self.command {
                    let _ = c.send(Command::Action(*id, key.clone()));
                    let _ = c.send(Command::Closed(*id, 2));
                }
                let id = *id;
                self.stack.retain(|(m, _)| m.id != id);
            }
            Deed::DndToggle => {
                self.dnd = !self.dnd;
                mk::settings::write("do-not-disturb", if self.dnd { "on" } else { "off" });
                if self.dnd {
                    self.stack.clear();
                }
                eprintln!("[mitteilung] Nicht stören: {}", if self.dnd { "an" } else { "aus" });
            }
            Deed::ClearAll => {
                eprintln!("[mitteilung] Historie geleert");
                self.history.clear();
                inbox_sync(&mut self.history, self.command.as_ref(), true);
            }
            Deed::HistoryClose => {
                eprintln!("[mitteilung] Historie zu (×)");
                self.history_open = false;
            }
            Deed::Expand(app) => {
                eprintln!("[mitteilung] Gruppe „{app}“ entfaltet");
                let app = app.clone();
                self.expanded.insert(app);
            }
            Deed::Collapse(app) => {
                eprintln!("[mitteilung] Gruppe „{app}“ gefaltet");
                let app = app.clone();
                self.expanded.remove(app.as_str());
            }
        }
        Refresh::Dirty
    }

    fn beat(&mut self) -> Refresh {
        let mut dirty = inbox_sync(&mut self.history, self.command.as_ref(), false);
        // The dwell time: 5 s, then the popup gives way.
        let vorher = self.stack.len();
        self.stack.retain(|(_, since)| since.elapsed().as_millis() < DWELL_TIME_MS);
        if self.stack.len() != vorher {
            eprintln!("[mitteilung] Popup gewichen (Verweilzeit)");
            dirty = true;
        }
        // The bell: the bar's file log (open/closed), and the
        // fold signal of the sibling popups.
        let show = newest_mtime(&[show_path(), show_path_legacy()]);
        if show.is_some() && show != self.last_show {
            self.last_show = show;
            self.history_open = !self.history_open;
            eprintln!(
                "[mitteilung] Glocke: Historie {}",
                if self.history_open { "auf" } else { "zu" }
            );
            dirty = true;
        }
        let hide = std::fs::metadata(hide_path()).and_then(|m| m.modified()).ok();
        if hide.is_some() && hide != self.last_hide {
            self.last_hide = hide;
            if self.history_open {
                self.history_open = false;
                dirty = true;
            }
        }
        if dirty {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn pulse(&mut self) -> Refresh {
        self.dnd = dnd_on();
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
            self.palette = fresh;
            self.icons.clear();
            self.puts.clear();
            self.epoch += 1;
            return Refresh::Full;
        }
        Refresh::Calm
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!(
        "[mitteilung] Matrix Mitteilungen: fünfter App-Umzug — der erste Daemon, pid={}",
        std::process::id()
    );
    shell::run(
        Notifications::new(),
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-mitteilung"),
                layer: shell::Layer::Overlay,
                keyboard: shell::KeyboardWish::None,
                anchor: shell::Anchors { top: true, right: true, bottom: false, left: false },
                size: (1, 1),
                margin: (
                    mkw::panel::BAR_STRIP + mkw::panel::POPUP_TOP as i32,
                    mkw::panel::POPUP_RIGHT as i32,
                    0,
                    0,
                ),
                exclusive: -1,
            },
            argb: true,
            tag: "mitteilung",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
