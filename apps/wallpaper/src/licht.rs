//! First light (iced replacement — GRABUNG-ICED-ABLOESUNG.md).
//!
//! Since stage 17 this mode is the first CUSTOMER of the
//! window house (matrixkit-shell): the whole Wayland runtime —
//! registry, surfaces, buffer dance, diff, compose, input,
//! fleet — lives in the kit; only the APP lives here: background
//! (the wall from the atlas), tree (view as a pure function of the
//! model) and the answers to input.
//!
//! Shadow doctrine (like die frühere Wahl R116): `--erstlicht` is a
//! second mode BESIDE the iced path, not its replacement — the
//! default only changes after the owner's finger probe.

use matrixkit_shell::{self as shell, Curve, Pict, Refresh, Scene, Source, Text};
use matrixkit_text::TextEngine;
use matrixkit_theme::scene::{Keep, Rect};

/// Overridable image source for the measurement probe: with LICHT_BILD one
/// can prove WHO is painting (two clients, two images).
fn licht_path() -> Option<String> {
    std::env::var("LICHT_BILD").ok().or_else(crate::image_path)
}

/// The result of view(): the laid-out scene with everything the
/// painter and the hit test need to know about it.
struct View {
    scene: Vec<matrixkit_theme::scene::Placed>,
    roles: Vec<Option<usize>>,
    texts: Vec<Text>,
    /// The hull of the traffic-light GROUP (stage 13) - target of the
    /// hover check in pointer motion.
    lights: matrixkit_theme::scene::Rect,
    /// Stage 14: view box + scroll extent of the peek scroll.
    roll_view: matrixkit_theme::scene::Rect,
    roll_max: f32,
    /// Stage 15: the scene's images (the atlas blits them).
    picts: Vec<Pict>,
    /// Stage 16: the scene's vector curves.
    curves: Vec<Curve>,
    /// Stage 91: the five boxes of the header bar (three traffic lights, title,
    /// window key) at THEIR position — the client's magnifier aims
    /// at them. The probe field floats in the middle of the surface, so
    /// the client cannot compute them itself.
    head_boxes: [matrixkit_theme::scene::Rect; 5],
}

/// The disabled button in the probe field (family behaviour: inert).
const LOCKED: usize = 4;

/// Role namespace: roles from TILE_BASE are tiles, from
/// FORM_BASE form switches, below that buttons — three families,
/// one hit test.
const TILE_BASE: usize = 100;
const FORM_BASE: usize = 200;
const TOOL_BASE: usize = 300;
/// Tool sub-role: the bar's search field.
const SEARCH: usize = 9;
const PEEK_BASE: usize = 400;
const FRAME_BASE: usize = 500;

/// The key of the peek scroll in the state slot.
const ROLL_KEY: u64 = 1;

/// The scrolling content of the peek probe: the dig log.
const PEEK_TEXT: &str = "Lager 1: mk::scene - die Layoutmaschine\nLager 2: die Textmaschine (cosmic-text)\nLager 3: der Klient hoert zu\nLager 4: Zustandsbaum + Diffing\nLager 5: matrixkit-text, das Kit-Zuhause\nLager 6: die Knoepfe ziehen um\nLager 7: die Kacheln ziehen um\nLager 8: die Formulare ziehen um\nLager 9: die Werkzeugleisten sprechen Symbole\nStufe 0: mkw ist das EINE Tor\nLager 10: die Meldungen ziehen um\nLager 11: der Blick + Z-Ordnung\nLager 12: die Flotte - mehrere Flaechen\nLager 13: der Frame - Ampeln + Hover\nLager 14: der Roll, den du gerade drehst\nDanach: Stufe 4 - IME, Canvas, Bild-Atlas\nDie Vertraege wachsen. Sie wechseln nie.";

/// The probe's shelf: three tiles (R172 face).
const TILES: [(&str, &str); 3] =
    [("Dateien", "Matrix"), ("Terminal", "Matrix"), ("Rabbit", "Hardline")];

/// The probe's form rows; the third is disabled
/// (family behaviour like toggle_guarded without a message).
const FORM_ROWS: [&str; 3] = ["Funknetz", "Bluetooth", "Flugmodus"];
const FORM_LOCKED: usize = 2;

/// Stage 4: view() is a PURE function of the model — model
/// in, tree out, no side effects. Exactly the Elm loop
/// that mk::app will later drive without iced: input → update →
/// view → diff → paint.
///
/// Stages 6/7/8, the family move: the probe field consists of
/// REAL MatrixKit buttons, tiles and forms (mkw::scene).
/// The hit button carries the pressed state layer, the
/// disabled one stays mute; a tapped tile "opens a
/// window" (R174j); a tapped switch flips its thumb —
/// the disabled airplane mode stays dimmed.
/// The probe field's model: what the human has done so far.
/// Since stage 19 it carries a string (the search field) —
/// Copy went away, view takes a clone.
#[derive(Clone)]
struct Model {
    selected: Option<usize>,
    opened: [bool; 3],
    switches: [bool; 3],
    page: u8,
    starred: bool,
    peeked: bool,
    frame_active: bool,
    lights_hover: bool,
    /// The window bar's MAGNIFIER (stage 91) — the client springs,
    /// view() only gets the numbers.
    magnify: [f32; 5],
    search: String,
    cursor: usize,
    search_focus: bool,
    /// Stage 20: the selection anchor (shift ranges) and the
    /// blink phase of the caret (the house's 500 ms beat).
    anchor: Option<usize>,
    blink: bool,
    /// Stage 35: how often each fleet surface was touched — the
    /// fleet listens, the card shows it.
    poked: [u8; 3],
}

fn view(
    engine: &mut TextEngine,
    keep: &mut matrixkit_theme::scene::Keep<f32>,
    m: Model,
    wall: &Option<String>,
    samples: &[f32],
    bw: u32,
    bh: u32,
) -> View {
    let Model {
        selected,
        opened,
        switches,
        page,
        starred,
        peeked,
        frame_active,
        lights_hover,
        magnify,
        search,
        cursor,
        search_focus,
        anchor,
        blink,
        poked: _,
    } = m;
    use matrixkit_theme::scene::{lay_out, Node, Rect};
    use matrixkit_widgets::buttons::{Role, Size, Style};
    use matrixkit_widgets::scene as fam;
    let p = matrixkit_theme::Palette::load().unwrap_or_default();

    let defs: [(&str, Style, Role, bool); 5] = [
        ("Sichern", Style::Prominent, Role::Normal, true),
        ("Optionen", Style::Tinted, Role::Normal, true),
        ("Leise", Style::Borderless, Role::Normal, true),
        ("Löschen", Style::Prominent, Role::Destructive, true),
        ("Gesperrt", Style::Tinted, Role::Normal, false),
    ];
    let mut labels = Vec::new();
    let children: Vec<Node> = defs
        .iter()
        .enumerate()
        .map(|(i, (label, style, role, active))| {
            let b = fam::Button {
                label: (*label).to_string(),
                style: *style,
                role: *role,
                size: Size::Normal,
                active: *active, ..Default::default() };
            let status = if selected == Some(i) && *active {
                fam::Status::Pressed
            } else {
                fam::Status::Idle
            };
            let (node, label) = fam::button(engine, &b, status, p);
            labels.push(label);
            node
        })
        .collect();
    let panel = Node::Row {
        gap: matrixkit_theme::spacing::M,
        padding: 14.0,
        background: Some((p.surface_container, 14.0 + matrixkit_theme::CORNER_RADIUS)),
        children,
    };
    let mut scene = Vec::new();
    let (w, h) = lay_out(&panel, 0.0, 0.0, &mut scene);
    scene.clear();
    let x = (bw as f32 - w) / 2.0;
    let y = bh as f32 - h - 96.0;
    lay_out(&panel, x, y, &mut scene);
    // Roles in paint order: field, then per button background + text size.
    let mut roles = vec![None];
    for i in 0..defs.len() {
        roles.extend([Some(i); fam::PLACED_PER_BUTTON]);
    }
    // The buttons' labels: anchor from the laid-out scene,
    // home = the hull of the button background.
    let anchors = fam::label_anchors(&scene, 1, defs.len());
    let mut texts: Vec<Text> = labels
        .into_iter()
        .zip(anchors)
        .enumerate()
        .map(|(i, (l, (ax, ay)))| Text {
            s: l.text,
            px: l.px,
            weight: l.weight,
            family: l.family,
            color: l.color,
            x: ax,
            y: ay,
            home: Rect::of(&scene[1 + i * fam::PLACED_PER_BUTTON]),
            anchor: 1 + i * fam::PLACED_PER_BUTTON + 1,
            clip: None,
        })
        .collect();

    // Stage 7, the shelf: three tiles above the button panel — floating
    // freely as in the floe-less dock, each as wide as its
    // words; an "opened" one carries its window row.
    let tiles: Vec<fam::Tile> = TILES
        .iter()
        .zip(opened)
        .map(|((name, maker), open)| fam::Tile {
            name: (*name).to_string(),
            maker: (*maker).to_string(),
            lines: if open { vec!["1 Fenster".to_string()] } else { Vec::new() },
        })
        .collect();
    let mut tile_labels = Vec::new();
    let shelf_children: Vec<Node> = tiles
        .iter()
        .map(|t| {
            let (node, labels) = fam::tile(engine, t, p);
            tile_labels.push(labels);
            node
        })
        .collect();
    let shelf = Node::Row {
        gap: matrixkit_theme::spacing::S,
        padding: 0.0,
        background: None,
        children: shelf_children,
    };
    let mut shelf_placed = Vec::new();
    let (sw, sh) = lay_out(&shelf, 0.0, 0.0, &mut shelf_placed);
    shelf_placed.clear();
    let sx = (bw as f32 - sw) / 2.0;
    let sy = y - sh - 24.0;
    lay_out(&shelf, sx, sy, &mut shelf_placed);
    let shelf_first = scene.len();
    for (i, t) in tiles.iter().enumerate() {
        roles.extend(std::iter::repeat(Some(TILE_BASE + i)).take(fam::tile_placed_count(t)));
    }
    scene.extend(shelf_placed);
    let mut first = shelf_first;
    for (t, labels) in tiles.iter().zip(tile_labels) {
        let home = Rect::of(&scene[first]);
        let anchors = fam::tile_label_anchors(&scene, first, t);
        for (j, (l, (ax, ay))) in labels.into_iter().zip(anchors).enumerate() {
            texts.push(Text {
                s: l.text,
                px: l.px,
                weight: l.weight,
                family: l.family,
                color: l.color,
                x: ax,
                y: ay,
                home,
                anchor: first + 2 + j,
                clip: None,
            });
        }
        first += fam::tile_placed_count(t);
    }

    // The sign: hits + wall clock. The clock IN the image is the
    // freshness proof of the screenshot doctrine, painted by the client
    // itself — and because it sits in the label, every new
    // second damages the sign by itself.
    let clock = std::process::Command::new("date")
        .arg("+%H:%M:%S")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let label = match selected {
        Some(i) => format!("Treffer: Knopf {i} ({}) · {clock}", defs[i].0),
        None => format!("MatrixKit hört zu · {clock}"),
    };
    let px = 30.0;
    let (tw, th) = engine.measure(&label, px);
    let pad = 18.0;
    let node = Node::Row {
        gap: 0.0,
        padding: pad,
        background: Some((p.surface_container_high, 24.0)),
        children: vec![Node::Quad {
            w: tw,
            h: th,
            color: matrixkit_theme::Rgba { a: 0.0, ..p.surface },
            radius: 0.0,
        }],
    };
    let mut chip_placed = Vec::new();
    let (cw, _) = lay_out(&node, 0.0, 0.0, &mut chip_placed);
    chip_placed.clear();
    let cx = (bw as f32 - cw) / 2.0;
    // Since stage 7 the shelf lives between the sign and the panel.
    let cy = bh as f32 - 380.0;
    lay_out(&node, cx, cy, &mut chip_placed);
    texts.push(Text {
        s: label,
        px,
        weight: matrixkit_theme::typo::Weight::Normal,
        family: matrixkit_text::FAMILY,
        color: p.on_surface,
        x: cx + pad,
        y: cy + pad,
        home: Rect::of(&chip_placed[0]),
        anchor: scene.len() + 1,
        clip: None,
    });
    // Stage 11: the chip is the peek trigger (role PEEK_BASE).
    roles.extend(std::iter::repeat(Some(PEEK_BASE)).take(chip_placed.len()));
    scene.extend(chip_placed);

    // Stage 8, the form: the section "Verbindungen" above the
    // sign — three switch rows, the third disabled. The
    // bookkeeping comes from the family (label_slots/toggle_slots),
    // the app only translates it into roles and labels.
    let form_rows = [
        fam::FormRow {
            title: FORM_ROWS[0].to_string(),
            description: None,
            on: switches[0],
            locked: false,
        },
        fam::FormRow {
            title: FORM_ROWS[1].to_string(),
            description: Some("Sichtbar als Matrix".to_string()),
            on: switches[1],
            locked: false,
        },
        fam::FormRow {
            title: FORM_ROWS[2].to_string(),
            description: None,
            on: switches[2],
            locked: true,
        },
    ];
    let form = fam::form_section(engine, "Verbindungen", &form_rows, 340.0, p);
    let mut form_placed = Vec::new();
    let (fw, fh) = lay_out(&form.node, 0.0, 0.0, &mut form_placed);
    form_placed.clear();
    let fx = (bw as f32 - fw) / 2.0;
    let fy = cy - fh - 24.0;
    lay_out(&form.node, fx, fy, &mut form_placed);
    let form_first = scene.len();
    let mut form_roles: Vec<Option<usize>> = vec![None; form_placed.len()];
    for (i, slot) in form.toggle_slots.iter().enumerate() {
        for k in 0..fam::PLACED_PER_TOGGLE {
            form_roles[slot + k] = Some(FORM_BASE + i);
        }
    }
    roles.extend(form_roles);
    scene.extend(form_placed);
    for (l, slot) in form.labels.iter().zip(&form.label_slots) {
        let q = scene[form_first + slot];
        texts.push(Text {
            s: l.text.clone(),
            px: l.px,
            weight: l.weight,
            family: l.family,
            color: l.color,
            x: q.x,
            y: q.y,
            home: Rect::of(&q),
            anchor: form_first + slot,
            clip: None,
        });
    }

    // Stage 9, the toolbar: at the head of a card above the
    // form. ◀ ▶ page through the page title, ★ flips between
    // outline and filled; mute chevrons stay visible but carry
    // no role. The search field is deliberately missing (stage 4 land).
    let star = if starred {
        matrixkit_widgets::symbol::STAR
    } else {
        matrixkit_widgets::symbol::STAR_OUTLINE
    };
    let bar = fam::toolbar(
        engine,
        &format!("Seite {page}"),
        (page > 1, fam::Status::Idle),
        (page < 3, fam::Status::Idle),
        &[(star, true, fam::Status::Idle)],
        Some(&fam::FieldSpec {
            value: search.as_str(),
            cursor,
            anchor,
            focused: search_focus,
            blink,
            placeholder: "Suchen",
            leading: None,
        }),
        456.0,
        p,
    );
    let card = Node::Wrap {
        pad_x: matrixkit_theme::spacing::M,
        pad_y: matrixkit_theme::spacing::S,
        background: Some((p.surface_container, matrixkit_theme::radius::LARGE)),
        child: Box::new(bar.node),
    };
    let mut bar_placed = Vec::new();
    let (bw_, bh_) = lay_out(&card, 0.0, 0.0, &mut bar_placed);
    bar_placed.clear();
    let bx = (bw as f32 - bw_) / 2.0;
    let by = fy - bh_ - 24.0;
    lay_out(&card, bx, by, &mut bar_placed);
    let bar_first = scene.len();
    let mut bar_roles: Vec<Option<usize>> = vec![None; bar_placed.len()];
    // Roles only for live tools; +1 because the card background
    // lies in front of the bar.
    let alive = [page > 1, page < 3, true];
    for (i, slot) in bar.tool_slots.iter().enumerate() {
        if alive[i] {
            for k in 0..fam::PLACED_PER_TOOL {
                bar_roles[1 + slot + k] = Some(TOOL_BASE + i);
            }
        }
    }
    // Stage 19: the search field is a hit target (takes the focus).
    if let Some(slot) = bar.search_slot {
        for k in 0..fam::PLACED_PER_FIELD {
            bar_roles[1 + slot + k] = Some(TOOL_BASE + SEARCH);
        }
    }
    roles.extend(bar_roles);
    scene.extend(bar_placed);
    for (l, slot) in bar.labels.iter().zip(&bar.label_slots) {
        let q = scene[bar_first + 1 + slot];
        texts.push(Text {
            s: l.text.clone(),
            px: l.px,
            weight: l.weight,
            family: l.family,
            color: l.color,
            x: q.x,
            y: q.y,
            home: Rect::of(&q),
            anchor: bar_first + 1 + slot,
            clip: None,
        });
    }

    // Stage 13, the frame: the header row with the traffic lights above the
    // toolbar - appearsActive dims, hover calls up the glyphs,
    // the x light REALLY closes the probe field.
    let header = fam::frame_header(
        engine,
        "Erstes Licht",
        // A LAYER surface does not roll up — the key
        // only shows its resting glyph here.
        // The probe field is a layer surface with its own position; the
        // magnifier lives with the client, not in this free function.
        fam::HeaderState { active: frame_active, lights_hover, rolled_up: false, magnify },
        456.0,
        p,
    );
    let mut head_placed = Vec::new();
    let (hw, hh) = lay_out(&header.node, 0.0, 0.0, &mut head_placed);
    head_placed.clear();
    let hx = (bw as f32 - hw) / 2.0;
    let hy = by - hh - 16.0;
    lay_out(&header.node, hx, hy, &mut head_placed);
    // The bar's boxes at THEIR position — the magnifier aims at them.
    let head_box = |slot: usize| {
        head_placed
            .get(slot)
            .map(Rect::of)
            .unwrap_or(Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 })
    };
    let head_boxes = [
        head_box(header.light_slots[0]),
        head_box(header.light_slots[1]),
        head_box(header.light_slots[2]),
        head_box(header.title_slot),
        head_box(header.shade_slot),
    ];
    let head_first = scene.len();
    let mut head_roles: Vec<Option<usize>> = vec![None; head_placed.len()];
    for (k, slot) in header.light_slots.iter().enumerate() {
        head_roles[*slot] = Some(FRAME_BASE + k);
    }
    head_roles[header.title_slot] = Some(FRAME_BASE + 3);
    roles.extend(head_roles);
    scene.extend(head_placed);
    let tq = scene[head_first + header.title_slot];
    texts.push(Text {
        s: header.title.text.clone(),
        px: header.title.px,
        weight: header.title.weight,
        family: header.title.family,
        color: header.title.color,
        x: tq.x,
        y: tq.y,
        home: Rect::of(&scene[head_first]),
        anchor: head_first + header.title_slot,
        clip: None,
    });
    for (k, (g, dx, dy)) in header.glyphs.iter().enumerate() {
        let lq = scene[head_first + header.light_slots[k]];
        texts.push(Text {
            s: g.text.clone(),
            px: g.px,
            weight: g.weight,
            family: g.family,
            color: g.color,
            x: lq.x + dx,
            y: lq.y + dy,
            home: Rect::of(&scene[head_first]),
            anchor: head_first + header.light_slots[k],
            clip: None,
        });
    }
    let l0 = scene[head_first + header.light_slots[0]];
    let l2 = scene[head_first + header.light_slots[2]];
    let lights = Rect {
        x: l0.x - 2.0,
        y: l0.y - 2.0,
        w: (l2.x + l2.w) - l0.x + 4.0,
        h: l0.h + 4.0,
    };

    // Stage 10, the messages: the result banner derives PURELY
    // from the model — "Sichern" pressed = success (primary),
    // "Löschen" pressed = error world (error), otherwise no banner.
    // Below it, fixed in place, the quiet footer.
    let ny = y + h + 24.0;
    let said = match selected {
        Some(0) => Some((true, "Gespeichert — alles steht")),
        Some(3) => Some((false, "Löschen fehlgeschlagen — das Probefeld ist unzerstörbar")),
        _ => None,
    };
    if let Some((ok, text)) = said {
        let (node, label) = fam::notice(engine, ok, text, 456.0, p);
        let mut placed = Vec::new();
        lay_out(&node, (bw as f32 - 456.0) / 2.0, ny, &mut placed);
        let first = scene.len();
        roles.extend(std::iter::repeat(None).take(placed.len()));
        scene.extend(placed);
        let q = scene[first + 1];
        texts.push(Text {
            s: label.text.clone(),
            px: label.px,
            weight: label.weight,
            family: label.family,
            color: label.color,
            x: q.x,
            y: q.y,
            home: Rect::of(&scene[first]),
            anchor: first + 1,
            clip: None,
        });
    }
    let (fnode, flabel) = fam::footer(engine, "Bereit · Erstes-Licht-Probefeld", p);
    let mut fplaced = Vec::new();
    let (fw2, _) = lay_out(&fnode, 0.0, 0.0, &mut fplaced);
    fplaced.clear();
    lay_out(&fnode, (bw as f32 - fw2) / 2.0, ny + 42.0, &mut fplaced);
    let first = scene.len();
    roles.extend(std::iter::repeat(None).take(fplaced.len()));
    scene.extend(fplaced);
    let q = scene[first + 1];
    texts.push(Text {
        s: flabel.text.clone(),
        px: flabel.px,
        weight: flabel.weight,
        family: flabel.family,
        color: flabel.color,
        x: q.x,
        y: q.y,
        home: Rect::of(&q),
        anchor: first + 1,
        clip: None,
    });

    // Stage 11, the peek: the fleeting preview as the topmost
    // layer, centred — today's content is the hero glyph (the image comes with
    // the image atlas, stage 4). A tap on the card closes.
    let mut roll_view = Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
    let mut roll_max = 0.0f32;
    let mut picts: Vec<Pict> = Vec::new();
    let mut curves: Vec<Curve> = Vec::new();

    // Stage 16: the diff history curve - the vector machine paints
    // the cost of its own paint passes (self-measurement: every
    // interaction draws its price into the card).
    {
        // Stage 21: the card now comes from the FAMILY
        // (history_card) — born here, returned as a building block.
        let card = fam::history_card(engine, "Diff-Verlauf · % der Wand", 260.0, 90.0, p);
        let mut card_placed = Vec::new();
        lay_out(&card.node, 24.0, 200.0, &mut card_placed);
        let first = scene.len();
        roles.extend(std::iter::repeat(None).take(card_placed.len()));
        scene.extend(card_placed);
        let tq = scene[first + card.label_slot];
        texts.push(Text {
            s: card.label.text.clone(),
            px: card.label.px,
            weight: card.label.weight,
            family: card.label.family,
            color: card.label.color,
            x: tq.x,
            y: tq.y,
            home: Rect::of(&tq),
            anchor: first + card.label_slot,
            clip: None,
        });
        let bq = scene[first + card.curve_slot];
        curves.push(Curve {
            samples: samples.to_vec(),
            max: 10.0,
            tint: p.primary,
            x: bq.x,
            y: bq.y,
            w: bq.w,
            h: bq.h,
            home: Rect::of(&bq),
            anchor: first + card.curve_slot,
        });
    }
    if peeked {
        // Stage 15: the toolbar's pages are REAL pages
        // of the peek - 1: the scrolling log (stage 14), 2: the image
        // from the atlas, 3: the hero glyph.
        let off = *keep.visit(ROLL_KEY);
        let (name, sub, content) = match page {
            1 => ("Grabungs-Log", "Stufe-3-Probefeld · rollt ohne iced",
                fam::PeekContent::Text(PEEK_TEXT.to_string())),
            2 => ("Die Wand", "Bild · einmal dekodiert, aus dem Atlas geblittet",
                match wall {
                    Some(pth) => fam::PeekContent::Image(std::path::PathBuf::from(pth)),
                    None => fam::PeekContent::Glyph(matrixkit_widgets::symbol::STAR),
                }),
            _ => ("Erstes Licht", "Stufe-4-Basislager · der Atlas steht",
                fam::PeekContent::Glyph(matrixkit_widgets::symbol::STAR)),
        };
        let sheet = fam::peek(engine, name, sub, &content, off, p);
        let mut placed = Vec::new();
        let (pw, ph) = lay_out(&sheet.node, 0.0, 0.0, &mut placed);
        placed.clear();
        let px_ = (bw as f32 - pw) / 2.0;
        let py_ = (bh as f32 - ph) / 2.0;
        lay_out(&sheet.node, px_, py_, &mut placed);
        let first = scene.len();
        roles.extend(std::iter::repeat(Some(PEEK_BASE + 1)).take(placed.len()));
        scene.extend(placed);
        let home = Rect::of(&scene[first]);
        for (k, (l, slot)) in sheet.labels.iter().zip(&sheet.label_slots).enumerate() {
            let q = scene[first + slot];
            let (ty, clip, thome) = if k == 0 {
                if let Some(r) = &sheet.roll {
                    let vq = scene[first + r.viewport_slot];
                    let view = Rect { x: vq.x, y: vq.y, w: vq.w, h: vq.h };
                    (vq.y - off, Some(view), view)
                } else {
                    (q.y, None, home)
                }
            } else {
                (q.y, None, home)
            };
            texts.push(Text {
                s: l.text.clone(),
                px: l.px,
                weight: l.weight,
                family: l.family,
                color: l.color,
                x: q.x,
                y: ty,
                home: thome,
                anchor: first + slot,
                clip,
            });
        }
        if let Some(r) = &sheet.roll {
            let vq = scene[first + r.viewport_slot];
            roll_view = Rect { x: vq.x, y: vq.y, w: vq.w, h: vq.h };
            roll_max = r.content_h - r.view_h;
        }
        if let Some((slot, pth)) = &sheet.image {
            let q = scene[first + slot];
            picts.push(Pict {
                path: pth.clone(),
                box_w: q.w as u32,
                box_h: q.h as u32,
                fit: matrixkit_image::Fit::Contain,
                x: q.x,
                y: q.y,
                radius: matrixkit_theme::radius::SMALL,
                home: Rect::of(&q),
                anchor: first + slot,
                clip: None,
            });
        }
    }

    View { scene, roles, texts, lights, roll_view, roll_max, picts, curves, head_boxes }
}


/// The probe: the model, the wall, the self-measurement — the
/// first client of the window house.
struct Probe {
    model: Model,
    path: Option<String>,
    samples: Vec<f32>,
    frame_lights: Rect,
    roll_view: Rect,
    roll_max: f32,
    /// The header bar's MAGNIFIER (stage 91).
    head_zoom: matrixkit_widgets::scene::HeaderMotion,
}

impl Probe {
    fn new() -> Self {
        Probe {
            model: Model {
                selected: None,
                opened: [false; 3],
                switches: [true, false, false],
                page: 2,
                starred: false,
                peeked: false,
                frame_active: true,
                lights_hover: false,
                magnify: [1.0; 5],
                search: String::new(),
                cursor: 0,
                search_focus: false,
                anchor: None,
                blink: true,
            poked: [0; 3],
            },
            path: licht_path(),
            samples: Vec::new(),
            frame_lights: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            roll_view: Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            roll_max: 0.0,
            head_zoom: matrixkit_widgets::scene::HeaderMotion::new(),
        }
    }
}

impl shell::Client for Probe {
    type Card = String;
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        // Stage 9: load the symbol font — once per engine.
        matrixkit_widgets::scene::arm_symbols(engine);
    }

    fn base(&mut self, atlas: &mut matrixkit_image::ImageAtlas, buf: &mut [u8], w: u32, h: u32) {
        // Stage 15: the wall comes from the atlas.
        if let Some(pth) = self.path.clone() {
            atlas.paint(
                std::path::Path::new(&pth),
                w,
                h,
                matrixkit_image::Fit::Cover,
                0.0,
                0.0,
                0.0,
                None,
                buf,
                w,
                h,
            );
        }
    }

    fn view(&mut self, engine: &mut TextEngine, keep: &mut Keep<f32>,
        _atlas: &mut matrixkit_image::ImageAtlas, w: u32, h: u32) -> Scene {
        if std::env::var("LICHT_SZENE").is_err() {
            // Reiner Wand-Klient: leere Szene, Ruhe kostet nichts.
            self.frame_lights = Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
            self.roll_view = Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
            return Scene::default();
        }
        let v = view(engine, keep, self.model.clone(), &self.path, &self.samples, w, h);
        self.head_zoom.set_boxes(v.head_boxes);
        self.frame_lights = v.lights;
        self.roll_view = v.roll_view;
        self.roll_max = v.roll_max;
        Scene { placed: v.scene, roles: v.roles, texts: v.texts, picts: v.picts, curves: v.curves }
    }

    fn strike(&mut self, role: Option<usize>, x: f64, y: f64, source: Source) -> Refresh {
        let origin = source.label();
        match role {
            // Stage 13: the frame — the traffic lights act, the title
            // flips appearsActive.
            Some(t) if t >= FRAME_BASE => match t - FRAME_BASE {
                0 => {
                    eprintln!("[licht] Treffer ({origin}): Ampel × — das Probefeld schließt sich");
                    Refresh::Close
                }
                1 => {
                    eprintln!("[licht] Treffer ({origin}): Ampel − — Ablage (im Probefeld nur genannt)");
                    Refresh::Calm
                }
                2 => {
                    eprintln!("[licht] Treffer ({origin}): Ampel + — Maximieren (im Probefeld nur genannt)");
                    Refresh::Calm
                }
                _ => {
                    self.model.frame_active = !self.model.frame_active;
                    let state = if self.model.frame_active { "aktiv" } else { "inaktiv" };
                    eprintln!(
                        "[licht] Treffer ({origin}): Titel — das Fenster wird {state} (appearsActive)"
                    );
                    Refresh::Dirty
                }
            },
            // Stage 11: the peek — the chip opens, a card tap closes.
            Some(t) if t >= PEEK_BASE => {
                if t == PEEK_BASE {
                    self.model.peeked = !self.model.peeked;
                    let state = if self.model.peeked { "öffnet" } else { "schließt" };
                    eprintln!("[licht] Treffer ({origin}): der Chip {state} den Blick");
                } else {
                    self.model.peeked = false;
                    eprintln!("[licht] Treffer ({origin}): der Blick schließt — flüchtig wie das Leitbild");
                }
                Refresh::Dirty
            }
            // Stage 19: the search field takes the focus.
            Some(t) if t == TOOL_BASE + SEARCH => {
                if !self.model.search_focus {
                    self.model.search_focus = true;
                    eprintln!("[licht] Treffer ({origin}): das Suchfeld nimmt den Fokus");
                    Refresh::Dirty
                } else {
                    Refresh::Calm
                }
            }
            // Stage 9: the toolbar.
            Some(t) if t >= TOOL_BASE => {
                match t - TOOL_BASE {
                    0 => {
                        self.model.page -= 1;
                        eprintln!(
                            "[licht] Treffer ({origin}): Werkzeug ◀ → Seite {} bei ({x:.0}, {y:.0})",
                            self.model.page
                        );
                    }
                    1 => {
                        self.model.page += 1;
                        eprintln!(
                            "[licht] Treffer ({origin}): Werkzeug ▶ → Seite {} bei ({x:.0}, {y:.0})",
                            self.model.page
                        );
                    }
                    _ => {
                        self.model.starred = !self.model.starred;
                        let state = if self.model.starred { "gesetzt" } else { "gelöst" };
                        eprintln!("[licht] Treffer ({origin}): Werkzeug ★ {state} bei ({x:.0}, {y:.0})");
                    }
                }
                Refresh::Dirty
            }
            // Stage 8: the form switches.
            Some(t) if t >= FORM_BASE => {
                let t = t - FORM_BASE;
                if t == FORM_LOCKED {
                    eprintln!(
                        "[licht] Treffer ({origin}): Schalter {t} ({}) ist gesperrt — keine Wirkung",
                        FORM_ROWS[t]
                    );
                    Refresh::Calm
                } else {
                    self.model.switches[t] = !self.model.switches[t];
                    let state = if self.model.switches[t] { "an" } else { "aus" };
                    eprintln!(
                        "[licht] Treffer ({origin}): Schalter {t} ({}) → {state} bei ({x:.0}, {y:.0})",
                        FORM_ROWS[t]
                    );
                    Refresh::Dirty
                }
            }
            // Stage 7: the tiles open their window (R174j).
            Some(t) if t >= TILE_BASE => {
                let t = t - TILE_BASE;
                self.model.opened[t] = !self.model.opened[t];
                let verb = if self.model.opened[t] { "öffnet" } else { "schließt" };
                eprintln!(
                    "[licht] Treffer ({origin}): Kachel {t} ({}) {verb} ihr Fenster bei ({x:.0}, {y:.0})",
                    TILES[t].0
                );
                Refresh::Dirty
            }
            // Stage 6: the buttons (the disabled one is inert).
            Some(b) if b == LOCKED => {
                eprintln!("[licht] Treffer ({origin}): Knopf {b} ist gesperrt — keine Wirkung");
                Refresh::Calm
            }
            Some(b) => {
                eprintln!("[licht] Treffer ({origin}): Knopf {b} bei ({x:.0}, {y:.0})");
                self.model.selected = if self.model.selected == Some(b) { None } else { Some(b) };
                Refresh::Dirty
            }
            None => {
                eprintln!("[licht] Treffer ({origin}): das Feld bei ({x:.0}, {y:.0}), kein Knopf");
                Refresh::Calm
            }
        }
    }

    fn depart(&mut self) -> Refresh {
        let zoom = self.head_zoom.leave();
        self.model.magnify = self.head_zoom.factors();
        let war = self.model.lights_hover;
        self.model.lights_hover = false;
        if zoom || war {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn animating(&self) -> bool {
        self.head_zoom.running()
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        if self.head_zoom.tick(dt) {
            self.model.magnify = self.head_zoom.factors();
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn motion(&mut self, x: f64, y: f64) -> Refresh {
        // Stage 13: hover over the traffic-light GROUP.
        let r = self.frame_lights;
        let inside = r.w > 0.0
            && (x as f32) >= r.x
            && (x as f32) < r.x + r.w
            && (y as f32) >= r.y
            && (y as f32) < r.y + r.h;
        let zoom = self.head_zoom.point(x, y);
        self.model.magnify = self.head_zoom.factors();
        if zoom && inside == self.model.lights_hover {
            return Refresh::Dirty;
        }
        if inside != self.model.lights_hover {
            self.model.lights_hover = inside;
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn axis(&mut self, x: f64, y: f64, value: f64, keep: &mut Keep<f32>) -> Refresh {
        // Stage 14: the wheel axis scrolls the view box under the pointer.
        let r = self.roll_view;
        let inside = r.w > 0.0
            && (x as f32) >= r.x
            && (x as f32) < r.x + r.w
            && (y as f32) >= r.y
            && (y as f32) < r.y + r.h;
        if inside {
            if let Some(off) = keep.get_mut(ROLL_KEY) {
                let neu = (*off + value as f32).clamp(0.0, self.roll_max);
                if (neu - *off).abs() > f32::EPSILON {
                    *off = neu;
                    eprintln!("[licht] Roll: Versatz {neu:.0} von {:.0}", self.roll_max);
                    return Refresh::Dirty;
                }
            }
        }
        Refresh::Calm
    }

    fn key(&mut self, key: shell::Key, shift: bool) -> Refresh {
        // Stages 19/20: the keyboard writes into the search field — the
        // value belongs to the model, the caret counts in CHARACTERS,
        // shift spans the selection from the anchor to the caret.
        if !self.model.search_focus {
            return Refresh::Calm;
        }
        let m = &mut self.model;
        let at = |s: &str, i: usize| {
            s.char_indices().nth(i).map(|(b, _)| b).unwrap_or(s.len())
        };
        // Delete a standing selection (for characters/Backspace/
        // Delete): [lo, hi) goes away, the caret lands on lo.
        fn cut(m: &mut Model, at: &dyn Fn(&str, usize) -> usize) -> bool {
            let Some(a) = m.anchor else { return false };
            if a == m.cursor {
                m.anchor = None;
                return false;
            }
            let (lo, hi) = (a.min(m.cursor), a.max(m.cursor));
            let (b0, b1) = (at(&m.search, lo), at(&m.search, hi));
            m.search.replace_range(b0..b1, "");
            m.cursor = lo;
            m.anchor = None;
            true
        }
        // Movement: with shift extend, without shift collapse.
        let travel = |m: &mut Model, to: usize| {
            if shift {
                if m.anchor.is_none() {
                    m.anchor = Some(m.cursor);
                }
            } else {
                m.anchor = None;
            }
            m.cursor = to;
        };
        match key {
            shell::Key::Char(t) => {
                cut(m, &at);
                let b = at(&m.search, m.cursor);
                m.search.insert_str(b, &t);
                m.cursor += t.chars().count();
            }
            shell::Key::Backspace => {
                if !cut(m, &at) && m.cursor > 0 {
                    m.cursor -= 1;
                    let b = at(&m.search, m.cursor);
                    m.search.remove(b);
                }
            }
            shell::Key::Delete => {
                if !cut(m, &at) && m.cursor < m.search.chars().count() {
                    let b = at(&m.search, m.cursor);
                    m.search.remove(b);
                }
            }
            shell::Key::Left => {
                let to = m.cursor.saturating_sub(1);
                travel(m, to);
            }
            shell::Key::Right => {
                let to = (m.cursor + 1).min(m.search.chars().count());
                travel(m, to);
            }
            shell::Key::Home => travel(m, 0),
            shell::Key::End => {
                let to = m.search.chars().count();
                travel(m, to);
            }
            shell::Key::Enter => {
                eprintln!("[licht] Suche abgeschickt: \u{201e}{}\u{201c}", m.search);
                return Refresh::Calm;
            }
            shell::Key::Escape => {
                m.search_focus = false;
                m.anchor = None;
                eprintln!("[licht] Suchfeld gibt den Fokus ab");
                return Refresh::Dirty;
            }
            // Catch-all instead of enumeration: the key contract GROWS
            // (stage 86 brought Page Up/Down, Insert and the
            // F keys), and a search field has nothing to say
            // about them.
            _ => return Refresh::Calm,
        }
        match m.anchor {
            Some(a) if a != m.cursor => eprintln!(
                "[licht] Feld: \u{201e}{}\u{201c} Marke {} Auswahl {}..{}",
                m.search,
                m.cursor,
                a.min(m.cursor),
                a.max(m.cursor)
            ),
            _ => eprintln!(
                "[licht] Feld: \u{201e}{}\u{201c} Marke {}",
                m.search, m.cursor
            ),
        }
        Refresh::Dirty
    }

    fn beat(&mut self) -> Refresh {
        // Stage 20: the caret blinks in the house's 500 ms beat —
        // only while the field has the focus.
        if self.model.search_focus {
            self.model.blink = !self.model.blink;
            Refresh::Dirty
        } else if !self.model.blink {
            self.model.blink = true;
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn wants_text(&self) -> bool {
        self.model.search_focus
    }

    fn pulse(&mut self) -> Refresh {
        // Image change + sun position — the same verbs as in the
        // iced version, the same colour chain.
        if crate::wallpaper_mode_auto() {
            if let Some(goal_light) = crate::sun_top() {
                if goal_light != crate::is_light() {
                    matrixkit_theme::light_toggle();
                }
            }
        }
        let new = licht_path();
        if new != self.path {
            self.path = new;
            if let Some(p) = &self.path {
                crate::palette_generate(p);
            }
            // New image = new wall: the diff has no basis any more.
            return Refresh::Full;
        }
        Refresh::Calm
    }

    fn fleet(&mut self) -> Vec<matrixkit_shell::fleet::Spec<String>> {
        // Stage 12: the plan is a pure derivation of the model — one
        // surface per "opened" tile (R174j becomes physical).
        TILES
            .iter()
            .enumerate()
            .filter(|(i, _)| self.model.opened[*i])
            .map(|(i, (name, _))| matrixkit_shell::fleet::Spec {
                pane: if self.model.poked[i] > 0 {
                    format!("{name} — {}× berührt", self.model.poked[i])
                } else {
                    format!("{name} — Fenster 1")
                },
                width: 260,
                height: 120,
                cross: 24,
                namespace: "matrix-card",
                keyboard: false,
                sig: (1, i as u32),
            })
            .collect()
    }

    fn pane_strike(&mut self, card: &String, x: f64, y: f64, source: Source) -> shell::Refresh {
        // Stage 35: the fleet listens — the card says who.
        if let Some(i) = TILES.iter().position(|(name, _)| card.starts_with(name)) {
            self.model.poked[i] = self.model.poked[i].saturating_add(1);
            eprintln!(
                "[licht] Flotte berührt ({}): „{card}“ bei ({x:.0}, {y:.0}) — {}× — Nachanstrich",
                source.label(),
                self.model.poked[i]
            );
            return shell::Refresh::Panes;
        }
        shell::Refresh::Calm
    }

    fn pane(&mut self, card: &String, engine: &mut TextEngine, canvas: &mut [u8], w: u32, h: u32) {
        // The fleet card: tile-chrome look, the title leads.
        let title = card.clone();
        let p = matrixkit_theme::Palette::load().unwrap_or_default();
        let ring = matrixkit_theme::scene::Placed {
            x: 0.0,
            y: 0.0,
            w: w as f32,
            h: h as f32,
            color: p.outline.with_alpha(0.3),
            radius: matrixkit_theme::radius::LARGE,
            clip: None, shadow: None,
        };
        let fill = matrixkit_theme::scene::Placed {
            x: 1.0,
            y: 1.0,
            w: w as f32 - 2.0,
            h: h as f32 - 2.0,
            color: p.surface_container_high,
            radius: matrixkit_theme::radius::inner(matrixkit_theme::radius::LARGE, 1.0),
            clip: None, shadow: None,
        };
        shell::paint(&[ring, fill], canvas, w, h, None);
        let head = matrixkit_theme::typo::SUBTITLE;
        let cap = matrixkit_theme::typo::CAPTION;
        let m = matrixkit_theme::spacing::M;
        engine.paint_styled(&title, head.px(), head.weight, p.on_surface, m, m, canvas, w, h);
        engine.paint_styled(
            "offen · Flotten-Fläche ohne iced",
            cap.px(),
            cap.weight,
            p.on_surface_variant,
            m,
            m + head.px() * matrixkit_text::LINE_HEIGHT + 4.0,
            canvas,
            w,
            h,
        );
    }

    fn painted(&mut self, percent: f32) {
        // Stage 16: the self-measurement feeds the history curve.
        self.samples.push(percent);
        if self.samples.len() > 40 {
            self.samples.remove(0);
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[licht] Erstes Licht: erster Kunde des Fensterhauses, pid={}", std::process::id());
    let state = match std::env::var("LICHT_LAGE").as_deref() {
        Ok("top") => shell::Layer::Top,
        Ok("bottom") => shell::Layer::Bottom,
        _ => shell::Layer::Background,
    };
    shell::run(
        Probe::new(),
        shell::Options {
            mount: shell::Mount::Layer {
                namespace: String::from("matrix-hintergrun"),
                layer: state,
                // Stage 19: the search field wants keys (on_demand).
                keyboard: shell::KeyboardWish::OnDemand,
                anchor: shell::Anchors::all(),
                size: (0, 0),
                margin: (0, 0, 0, 0),
                exclusive: -1,
            },
            argb: false,
            tag: "licht",
            fleet_start: 200,
            fleet_gap: 12,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
