//! Matrix Einkauf in the window house (no iced).
//!
//! Top: the lists as pills (tap = switch, «+» = a new one), the two tabs
//! and the sort. «Produkte»: a search field, category filters and product
//! cards — a tap puts a product on the list (each further tap one step
//! more), a long press takes it off; a search without a match offers the
//! typed name as an own product. «Einkaufsliste»: grouped rows with − and
//! +, the amount typed in after a tap on it, the unit cycling on a tap; a
//! tap on the row strikes it into the basket (and back).

use crate::model::{self, Sort, Store, CATEGORIES, SORTS, UNITS};
use matrixkit_image::ImageAtlas;
use matrixkit_scene::scene as fam;
use matrixkit_shell::{self as shell, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::{Keep, Rect};

pub const APP_ID: &str = "matrix-shopping";

const CHROME_BASE: usize = 500;
const ROOT_BASE: usize = 700;
const LIST_BASE: usize = 1000; // + list index (< 100)
const LIST_NEW: usize = 1100;
const TAB_LIST: usize = 1101;
const TAB_PRODUCTS: usize = 1102;
const SEARCH: usize = 1103;
const SEARCH_CLEAR: usize = 1104;
const NAME_FIELD: usize = 1105;
const NAME_OK: usize = 1106;
const NAME_CANCEL: usize = 1107;
const CLEAR_DONE: usize = 1108;
const DELETE_LIST: usize = 1109;
const RENAME_LIST: usize = 1110;
const ADD_OWN: usize = 1111;
const COPY_LIST: usize = 1112;
const FILTER_BASE: usize = 1200; // + 0 = all, + 1 + k = category k
const SORT_BASE: usize = 1260;
const PRODUCT_BASE: usize = 2000; // + catalogue index (< 3000)
const ROW_BASE: usize = 5000; // + item index (< 1000) each
const LESS_BASE: usize = 6000;
const MORE_BASE: usize = 7000;
const QTY_BASE: usize = 8000;
const UNIT_BASE: usize = 9000;

const G_ADD: char = '\u{e145}';
const G_REMOVE: char = '\u{e15b}';
const G_LIST: char = '\u{e896}';
const G_GRID: char = '\u{e854}';
const G_SEARCH: char = '\u{e8b6}';
const G_OPEN: char = '\u{e836}';
const G_DONE: char = '\u{e86c}';
const G_EDIT: char = '\u{e3c9}';
const G_DELETE: char = '\u{e872}';
const G_SWEEP: char = '\u{e16c}';
const G_COPY: char = '\u{e14d}';

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    List,
    Products,
}

/// Where the keyboard goes.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    None,
    Search,
    /// A list name: new (true) or the active one renamed (false).
    ListName(bool),
    /// The amount of an item.
    Qty(usize),
}

struct Shop {
    store: Store,
    tab: Tab,
    sort: Sort,
    filter: Option<usize>,
    search: String,
    focus: Focus,
    /// The text of a list name or an amount being typed.
    edit: String,
    /// «Liste löschen» (delete list) asks once more (round 177: the kit's two taps, with a time limit).
    delete_armed: mk::confirm::Armed<()>,
    palette: mk::Palette,
    chrome: fam::Chrome,
    root: fam::RootPane,
    receipt: mk::Flash,
    press: Option<mk::touch::Press<Option<usize>>>,
    roll: f32,
    roll_max: f32,
    beats: u32,
}

fn sym(b: &mut SceneBuilder, engine: &mut TextEngine, g: char, size: f32, color: mk::Rgba, cx: f32, cy: f32, role: Option<usize>, clip: Option<Rect>) {
    let fam_name = matrixkit_scene::symbol_family();
    let px = mk::typo::symbol_px(size);
    let (gw, gh) = engine.measure_family(fam_name, &g.to_string(), px, mk::typo::Weight::Normal);
    b.text(engine, g.to_string(), px, mk::typo::Weight::Normal, fam_name, color, cx - gw / 2.0, cy - gh / 2.0, role, clip);
}

impl Shop {
    fn new() -> Self {
        Shop {
            store: Store::load(),
            tab: Tab::Products,
            sort: Sort::Category,
            filter: None,
            search: String::new(),
            focus: Focus::None,
            edit: String::new(),
            delete_armed: mk::confirm::Armed::new(),
            palette: mk::Palette::load().unwrap_or_default(),
            chrome: fam::Chrome::new("Matrix Einkauf", CHROME_BASE),
            root: fam::RootPane::new(APP_ID, &[]),
            receipt: mk::Flash::new(3.0),
            press: None,
            roll: 0.0,
            roll_max: 0.0,
            beats: 0,
        }
    }

    fn note(&mut self, s: impl Into<String>) {
        self.receipt.set(s);
    }

    fn save(&self) {
        self.store.save();
    }

    fn caret(&self) -> bool {
        self.beats % 2 == 0
    }

    fn filter_name(&self) -> Option<&'static str> {
        self.filter.and_then(|k| CATEGORIES.get(k).copied())
    }

    /// A row of pills that wraps; returns the y below it.
    #[allow(clippy::too_many_arguments)]
    fn pills(&self, b: &mut SceneBuilder, engine: &mut TextEngine, items: &[(Option<char>, String, usize, bool)], x0: f32, y: f32, w: f32, clip: Option<Rect>) -> f32 {
        let p = self.palette;
        let gap = mk::spacing::S;
        let (mut x, mut y) = (x0, y);
        let mut row_h: f32 = 0.0;
        for (g, label, role, on) in items {
            let pw = fam::pill_width(engine, *g, label).min(w);
            if x > x0 && x + pw > x0 + w {
                x = x0;
                y += row_h + gap;
                row_h = 0.0;
            }
            let tone = if *on { fam::PillTone::Accent } else { fam::PillTone::Quiet };
            let (pw, ph) = fam::lay_pill(b, engine, *g, label, x, y, Some(pw), Some(*role), tone, p);
            let _ = clip;
            x += pw + gap;
            row_h = row_h.max(ph);
        }
        y + row_h
    }

    fn section_label(&self, b: &mut SceneBuilder, engine: &mut TextEngine, text: &str, x: f32, y: f32, clip: Option<Rect>) -> f32 {
        let (_, h) = b.styled(engine, text.to_uppercase(), mk::typo::LABEL, self.palette.on_surface_variant, x, y, None, clip);
        y + h + mk::spacing::S
    }

    fn view_inner(&mut self, engine: &mut TextEngine, w: u32, h: u32) -> Scene {
        let (wf, hf) = (w as f32, h as f32);
        let p = self.palette;
        let mut b = SceneBuilder::new();
        let top = self.chrome.build(engine, &mut b, w, h, p);
        if self.chrome.rolled {
            return b.finish();
        }
        let body = Rect { x: 0.0, y: top, w: wf, h: (hf - top).max(0.0) };
        let clip = Some(body);
        let m = mk::spacing::L;
        let cw = (wf - 2.0 * m).min(760.0);
        let cx = (wf - cw) / 2.0;
        let mut y = top + m - self.roll;

        // The lists.
        let mut lists: Vec<(Option<char>, String, usize, bool)> = self
            .store
            .lists
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let open = l.items.iter().filter(|it| !it.done).count();
                let label = if open > 0 { format!("{} · {open}", l.name) } else { l.name.clone() };
                (None, label, LIST_BASE + i, i == self.store.active)
            })
            .collect();
        lists.push((Some(G_ADD), String::from("Neue Liste"), LIST_NEW, false));
        y = self.pills(&mut b, engine, &lists, cx, y, cw, clip) + mk::spacing::M;

        // A list name being typed (new or renamed).
        if let Focus::ListName(new) = self.focus {
            let ok_w = fam::pill_width(engine, None, "OK");
            let cancel_w = fam::pill_width(engine, None, "Abbrechen");
            let fw = (cw - ok_w - cancel_w - 2.0 * mk::spacing::S).max(120.0);
            let place = if new { "Name der neuen Liste" } else { "Neuer Name" };
            fam::lay_text_field(&mut b, engine, Rect { x: cx, y, w: fw, h: 44.0 }, &self.edit, true, self.caret(), place, Some(G_EDIT), (Some(NAME_FIELD), None), clip, p);
            fam::lay_pill(&mut b, engine, None, "Abbrechen", cx + fw + mk::spacing::S, y + 2.0, None, Some(NAME_CANCEL), fam::PillTone::Quiet, p);
            fam::lay_pill(&mut b, engine, None, "OK", cx + fw + 2.0 * mk::spacing::S + cancel_w, y + 2.0, None, Some(NAME_OK), fam::PillTone::Accent, p);
            y += 44.0 + mk::spacing::M;
        }

        // The tabs and the sort — one row (the Duo's panel is short).
        let open = self.store.list().items.iter().filter(|i| !i.done).count();
        let mut row = vec![
            (Some(G_GRID), String::from("Produkte"), TAB_PRODUCTS, self.tab == Tab::Products),
            (Some(G_LIST), if open > 0 { format!("Liste · {open}") } else { String::from("Liste") }, TAB_LIST, self.tab == Tab::List),
        ];
        row.extend(
            SORTS
                .iter()
                .enumerate()
                .filter(|(_, (s, _))| self.tab == Tab::Products || *s != Sort::Often)
                .map(|(k, (s, n))| (None, n.to_string(), SORT_BASE + k, *s == self.sort)),
        );
        y = self.pills(&mut b, engine, &row, cx, y, cw, clip) + mk::spacing::L;

        let end = match self.tab {
            Tab::Products => self.lay_products(&mut b, engine, cx, y, cw, clip),
            Tab::List => self.lay_list(&mut b, engine, cx, y, cw, clip),
        };
        self.roll_max = (end + self.roll + mk::spacing::XL * 2.0 - hf).max(0.0);
        self.roll = self.roll.min(self.roll_max);

        if let Some(note) = self.receipt.text() {
            let note = note.to_string();
            let wn = fam::pill_width(engine, Some('\u{e5ca}'), &note).min(wf - 32.0);
            fam::lay_pill(&mut b, engine, Some('\u{e5ca}'), &note, (wf - wn) / 2.0, hf - 56.0, Some(wn), None, fam::PillTone::Accent, p);
        }
        let mut scene = b.finish();
        if self.root.visible() {
            let mut rb = SceneBuilder::new();
            rb.scene = scene;
            let about = fam::RootAbout {
                name: "Matrix Einkauf",
                version: env!("CARGO_PKG_VERSION"),
                description: "Einkaufslisten: Produkte antippen, Menge wählen, im Laden abstreichen — im Fensterhaus, ohne iced.",
            };
            self.root.lay(engine, &mut rb, &about, wf, hf, p, ROOT_BASE);
            scene = rb.finish();
        }
        scene
    }

    fn lay_products(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, cx: f32, mut y: f32, cw: f32, clip: Option<Rect>) -> f32 {
        let p = self.palette;
        fam::lay_text_field(b, engine, Rect { x: cx, y, w: cw, h: 46.0 }, &self.search, self.focus == Focus::Search, self.caret(), "Produkt suchen oder neu eintippen", Some(G_SEARCH), (Some(SEARCH), Some(SEARCH_CLEAR)), clip, p);
        y += 46.0 + mk::spacing::M;
        let mut filters = vec![(None, String::from("Alle"), FILTER_BASE, self.filter.is_none())];
        filters.extend(CATEGORIES.iter().enumerate().map(|(k, c)| (None, c.to_string(), FILTER_BASE + 1 + k, self.filter == Some(k))));
        y = self.pills(b, engine, &filters, cx, y, cw, clip) + mk::spacing::L;

        let shown = self.store.products(self.filter_name(), &self.search, self.sort);
        let typed = self.search.trim().to_string();
        let exact = self.store.catalog.iter().any(|q| q.name.eq_ignore_ascii_case(&typed));
        let gap = mk::spacing::S;
        let cols = (((cw + gap) / (150.0 + gap)).floor() as usize).max(2);
        let card_w = (cw - gap * (cols - 1) as f32) / cols as f32;
        let card_h = 74.0;
        let grouped = self.sort == Sort::Category && self.filter.is_none();
        let mut col = 0usize;
        let mut last_cat: Option<String> = None;
        if !typed.is_empty() && !exact {
            // The typed name as an own product.
            let x = cx;
            b.quad(x, y, card_w.max(220.0).min(cw), card_h, p.primary_container.with_alpha(0.55), 16.0, Some(ADD_OWN), clip);
            sym(b, engine, G_ADD, 22.0, p.on_primary_container, x + 24.0, y + card_h / 2.0, Some(ADD_OWN), clip);
            let name = fam::ellipsize(engine, &format!("«{typed}»"), mk::typo::HEADLINE.px(), mk::typo::HEADLINE.weight, card_w.max(220.0).min(cw) - 60.0);
            b.styled(engine, name, mk::typo::HEADLINE, p.on_primary_container, x + 44.0, y + 14.0, Some(ADD_OWN), clip);
            let cat = self.filter_name().unwrap_or(model::OTHER);
            b.styled(engine, format!("Als eigenes Produkt · {cat}"), mk::typo::CAPTION, p.on_primary_container.with_alpha(0.8), x + 44.0, y + 40.0, Some(ADD_OWN), clip);
            y += card_h + mk::spacing::L;
        }
        if shown.is_empty() && typed.is_empty() {
            b.styled(engine, "Keine Produkte in dieser Auswahl.", mk::typo::CALLOUT, p.on_surface_variant, cx, y, None, clip);
            return y + 24.0;
        }
        for &i in &shown {
            let prod = self.store.catalog[i].clone();
            if grouped && last_cat.as_deref() != Some(prod.category.as_str()) {
                if col > 0 {
                    y += card_h + gap;
                    col = 0;
                }
                if last_cat.is_some() {
                    y += mk::spacing::S;
                }
                y = self.section_label(b, engine, &prod.category, cx, y, clip);
                last_cat = Some(prod.category.clone());
            }
            let x = cx + col as f32 * (card_w + gap);
            let role = Some(PRODUCT_BASE + i);
            let on = self.store.on_list(&prod.name).map(|it| (it.qty, it.unit.clone()));
            let (fill, ink) = if on.is_some() { (p.primary_container, p.on_primary_container) } else { (p.surface_container, p.on_surface) };
            b.quad(x, y, card_w, card_h, fill, 16.0, role, clip);
            let name = fam::ellipsize(engine, &prod.name, mk::typo::HEADLINE.px(), mk::typo::HEADLINE.weight, card_w - 24.0);
            b.styled(engine, name, mk::typo::HEADLINE, ink, x + 12.0, y + 12.0, role, clip);
            let sub = match &on {
                Some((q, u)) => format!("✓ {} {u} auf der Liste", model::qty_text(*q)),
                None if grouped => format!("+ {} {}", model::qty_text(model::first(&prod.unit)), prod.unit),
                None => prod.category.clone(),
            };
            let sub = fam::ellipsize(engine, &sub, mk::typo::CAPTION.px(), mk::typo::CAPTION.weight, card_w - 24.0);
            b.styled(engine, sub, mk::typo::CAPTION, ink.with_alpha(0.75), x + 12.0, y + 42.0, role, clip);
            col += 1;
            if col == cols {
                col = 0;
                y += card_h + gap;
            }
        }
        if col > 0 {
            y += card_h + gap;
        }
        b.styled(engine, "Antippen legt auf die Liste, nochmals antippen = mehr, lange drücken nimmt es weg.", mk::typo::CAPTION, p.on_surface_variant, cx, y + mk::spacing::S, None, clip);
        y + 40.0
    }

    #[allow(clippy::too_many_arguments)]
    fn lay_row(&self, b: &mut SceneBuilder, engine: &mut TextEngine, i: usize, cx: f32, y: f32, cw: f32, clip: Option<Rect>) -> f32 {
        let p = self.palette;
        let it = &self.store.list().items[i];
        let rh = 56.0;
        let row = Some(ROW_BASE + i);
        let fill = if it.done { p.surface_container.with_alpha(0.5) } else { p.surface_container };
        b.quad(cx, y, cw, rh, fill, 14.0, row, clip);
        let ink = if it.done { p.on_surface_variant.with_alpha(0.7) } else { p.on_surface };
        sym(b, engine, if it.done { G_DONE } else { G_OPEN }, 24.0, if it.done { p.primary } else { p.on_surface_variant }, cx + 26.0, y + rh / 2.0, row, clip);
        // The amount on the right: − [2] [Stk] +
        let qty = if self.focus == Focus::Qty(i) {
            format!("{}{}", self.edit, if self.caret() { "|" } else { " " })
        } else {
            model::qty_text(it.qty)
        };
        let st = mk::typo::HEADLINE;
        let (qw, qh) = engine.measure_styled(&qty, st.px(), st.weight);
        let (uw, uh) = engine.measure_styled(&it.unit, mk::typo::CALLOUT.px(), mk::typo::CALLOUT.weight);
        let btn = 36.0;
        let mut rx = cx + cw - 10.0;
        let mut name_room = cw - 60.0;
        if !it.done {
            // +
            rx -= btn;
            b.quad(rx, y + (rh - btn) / 2.0, btn, btn, p.surface_container_high, btn / 2.0, Some(MORE_BASE + i), clip);
            sym(b, engine, G_ADD, 20.0, p.on_surface, rx + btn / 2.0, y + rh / 2.0, Some(MORE_BASE + i), clip);
            // unit
            let ub = uw + 20.0;
            rx -= ub + 6.0;
            b.quad(rx, y + (rh - 32.0) / 2.0, ub, 32.0, p.on_surface.with_alpha(0.06), 10.0, Some(UNIT_BASE + i), clip);
            b.styled(engine, it.unit.clone(), mk::typo::CALLOUT, p.on_surface_variant, rx + 10.0, y + (rh - uh) / 2.0, Some(UNIT_BASE + i), clip);
            // amount (tap to type)
            let qb = qw.max(28.0) + 16.0;
            rx -= qb + 6.0;
            let editing = self.focus == Focus::Qty(i);
            let qfill = if editing { p.primary_container } else { p.on_surface.with_alpha(0.06) };
            b.quad(rx, y + (rh - 36.0) / 2.0, qb, 36.0, qfill, 10.0, Some(QTY_BASE + i), clip);
            b.styled(engine, qty, st, if editing { p.on_primary_container } else { p.on_surface }, rx + (qb - qw) / 2.0, y + (rh - qh) / 2.0, Some(QTY_BASE + i), clip);
            // −
            rx -= btn + 6.0;
            b.quad(rx, y + (rh - btn) / 2.0, btn, btn, p.surface_container_high, btn / 2.0, Some(LESS_BASE + i), clip);
            sym(b, engine, G_REMOVE, 20.0, p.on_surface, rx + btn / 2.0, y + rh / 2.0, Some(LESS_BASE + i), clip);
            name_room = rx - (cx + 50.0) - 8.0;
        } else {
            let t = format!("{} {}", model::qty_text(it.qty), it.unit);
            let (tw, th) = engine.measure_styled(&t, mk::typo::CALLOUT.px(), mk::typo::CALLOUT.weight);
            b.styled(engine, t, mk::typo::CALLOUT, ink, cx + cw - 16.0 - tw, y + (rh - th) / 2.0, row, clip);
            name_room = name_room.min(cw - 80.0 - tw);
        }
        let name = fam::ellipsize(engine, &it.name, mk::typo::BODY.px(), mk::typo::BODY.weight, name_room.max(40.0));
        let (nw, nh) = engine.measure_styled(&name, mk::typo::BODY.px(), mk::typo::BODY.weight);
        let ny = y + (rh - nh) / 2.0;
        b.styled(engine, name, mk::typo::BODY, ink, cx + 50.0, ny, row, clip);
        if it.done {
            // struck through: it is in the basket
            b.quad(cx + 48.0, ny + nh / 2.0, nw + 4.0, 1.6, ink, 0.8, row, clip);
        }
        y + rh + mk::spacing::S
    }

    fn lay_list(&mut self, b: &mut SceneBuilder, engine: &mut TextEngine, cx: f32, mut y: f32, cw: f32, clip: Option<Rect>) -> f32 {
        let p = self.palette;
        let (open, done) = self.store.items(self.sort);
        if open.is_empty() && done.is_empty() {
            b.styled(engine, "Noch leer — tippe unter «Produkte» auf das, was du brauchst.", mk::typo::CALLOUT, p.on_surface_variant, cx, y, None, clip);
            return y + 40.0;
        }
        let grouped = self.sort == Sort::Category;
        let mut last: Option<String> = None;
        for &i in &open {
            let cat = self.store.list().items[i].category.clone();
            if grouped && last.as_deref() != Some(cat.as_str()) {
                if last.is_some() {
                    y += mk::spacing::S;
                }
                y = self.section_label(b, engine, &cat, cx, y, clip);
                last = Some(cat);
            }
            y = self.lay_row(b, engine, i, cx, y, cw, clip);
        }
        if open.is_empty() {
            b.styled(engine, "Alles im Korb ✓", mk::typo::HEADLINE, p.primary, cx, y, None, clip);
            y += 28.0;
        }
        if !done.is_empty() {
            y += mk::spacing::M;
            y = self.section_label(b, engine, &format!("Im Korb · {}", done.len()), cx, y, clip);
            for &i in &done {
                y = self.lay_row(b, engine, i, cx, y, cw, clip);
            }
        }
        y += mk::spacing::M;
        let mut tools = Vec::new();
        if !done.is_empty() {
            tools.push((Some(G_SWEEP), String::from("Erledigte entfernen"), CLEAR_DONE, false));
        }
        if !open.is_empty() {
            tools.push((Some(G_COPY), String::from("Liste kopieren"), COPY_LIST, false));
        }
        tools.push((Some(G_EDIT), String::from("Liste umbenennen"), RENAME_LIST, false));
        let del = if self.delete_armed.is(&()) {
            String::from("Wirklich löschen?")
        } else if self.store.lists.len() > 1 {
            String::from("Liste löschen")
        } else {
            String::from("Liste leeren")
        };
        tools.push((Some(G_DELETE), del, DELETE_LIST, self.delete_armed.is(&())));
        self.pills(b, engine, &tools, cx, y, cw, clip) + mk::spacing::L
    }

    /// Leave a text being typed: an amount is taken, a search stays.
    fn unfocus(&mut self) {
        if let Focus::Qty(i) = self.focus {
            if let Some(q) = model::parse_qty(&self.edit) {
                if let Some(it) = self.store.list_mut().items.get_mut(i) {
                    it.qty = q;
                }
                self.save();
            }
        }
        self.focus = Focus::None;
        self.edit.clear();
    }

    fn commit_name(&mut self) {
        let Focus::ListName(new) = self.focus else { return };
        let name = self.edit.clone();
        if new {
            self.store.new_list(&name);
            self.tab = Tab::Products;
            self.note(format!("Liste «{}» angelegt", self.store.list().name));
        } else {
            self.store.rename_list(&name);
            self.note(format!("Liste heisst jetzt «{}»", self.store.list().name));
        }
        self.save();
        self.focus = Focus::None;
        self.edit.clear();
        self.roll = 0.0;
    }

    fn act(&mut self, role: usize) -> Refresh {
        // A tap elsewhere ends an amount being typed (and a list name).
        let keeps = match self.focus {
            Focus::Qty(i) => role == QTY_BASE + i,
            Focus::ListName(_) => matches!(role, NAME_FIELD | NAME_OK | NAME_CANCEL),
            Focus::Search => matches!(role, SEARCH | SEARCH_CLEAR | ADD_OWN) || (PRODUCT_BASE..ROW_BASE).contains(&role) || (FILTER_BASE..FILTER_BASE + 60).contains(&role) || (SORT_BASE..SORT_BASE + 3).contains(&role),
            Focus::None => true,
        };
        if !keeps {
            if matches!(self.focus, Focus::ListName(_)) {
                self.focus = Focus::None;
                self.edit.clear();
            } else {
                self.unfocus();
            }
        }
        if role != DELETE_LIST {
            self.delete_armed.clear();
        }
        match role {
            r if (LIST_BASE..LIST_BASE + 100).contains(&r) => {
                let i = r - LIST_BASE;
                if i < self.store.lists.len() && i != self.store.active {
                    self.store.active = i;
                    self.roll = 0.0;
                    self.save();
                }
            }
            LIST_NEW => {
                self.focus = Focus::ListName(true);
                self.edit.clear();
            }
            RENAME_LIST => {
                self.focus = Focus::ListName(false);
                self.edit = self.store.list().name.clone();
                self.roll = 0.0;
            }
            NAME_OK => self.commit_name(),
            NAME_CANCEL => {
                self.focus = Focus::None;
                self.edit.clear();
            }
            NAME_FIELD => {}
            TAB_LIST => {
                self.tab = Tab::List;
                self.roll = 0.0;
                if self.sort == Sort::Often {
                    self.sort = Sort::Category;
                }
            }
            TAB_PRODUCTS => {
                self.tab = Tab::Products;
                self.roll = 0.0;
            }
            SEARCH => self.focus = Focus::Search,
            SEARCH_CLEAR => self.search.clear(),
            ADD_OWN => {
                let cat = self.filter_name().unwrap_or(model::OTHER);
                if let Some(i) = self.store.own_product(&self.search, cat) {
                    self.store.add(i);
                    let name = self.store.catalog[i].name.clone();
                    self.note(format!("«{name}» auf der Liste"));
                    self.search.clear();
                    self.focus = Focus::None;
                    self.save();
                }
            }
            r if (FILTER_BASE..FILTER_BASE + 1 + CATEGORIES.len()).contains(&r) => {
                self.filter = (r > FILTER_BASE).then(|| r - FILTER_BASE - 1);
            }
            r if (SORT_BASE..SORT_BASE + SORTS.len()).contains(&r) => self.sort = SORTS[r - SORT_BASE].0,
            r if (PRODUCT_BASE..ROW_BASE).contains(&r) => {
                let i = r - PRODUCT_BASE;
                self.store.add(i);
                if let Some(it) = self.store.catalog.get(i).and_then(|p| self.store.on_list(&p.name)) {
                    let t = format!("{} · {} {}", it.name, model::qty_text(it.qty), it.unit);
                    self.note(t);
                }
                self.save();
            }
            r if (ROW_BASE..LESS_BASE).contains(&r) => {
                if let Some(it) = self.store.list_mut().items.get_mut(r - ROW_BASE) {
                    it.done = !it.done;
                }
                self.save();
            }
            r if (LESS_BASE..MORE_BASE).contains(&r) => {
                let i = r - LESS_BASE;
                let mut gone = None;
                if let Some(it) = self.store.list_mut().items.get_mut(i) {
                    it.qty -= model::step(&it.unit);
                    if it.qty <= 0.0001 {
                        gone = Some(it.name.clone());
                    }
                }
                if let Some(name) = gone {
                    self.store.list_mut().items.remove(i);
                    self.note(format!("«{name}» von der Liste genommen"));
                }
                self.save();
            }
            r if (MORE_BASE..QTY_BASE).contains(&r) => {
                if let Some(it) = self.store.list_mut().items.get_mut(r - MORE_BASE) {
                    it.qty += model::step(&it.unit);
                }
                self.save();
            }
            r if (QTY_BASE..UNIT_BASE).contains(&r) => {
                let i = r - QTY_BASE;
                if self.focus != Focus::Qty(i) {
                    if let Some(it) = self.store.list().items.get(i) {
                        self.edit = model::qty_text(it.qty);
                        self.focus = Focus::Qty(i);
                    }
                }
            }
            r if (UNIT_BASE..UNIT_BASE + 1000).contains(&r) => {
                if let Some(it) = self.store.list_mut().items.get_mut(r - UNIT_BASE) {
                    let k = UNITS.iter().position(|u| *u == it.unit).map_or(0, |k| (k + 1) % UNITS.len());
                    it.unit = UNITS[k].to_string();
                    it.qty = model::first(&it.unit);
                }
                self.save();
            }
            COPY_LIST => {
                // Round 85: the house's clipboard — paste it into a message.
                shell::board::write(self.store.share_text());
                self.note("Liste kopiert — zum Einfügen in eine Nachricht");
            }
            CLEAR_DONE => {
                self.store.clear_done();
                self.note("Erledigte entfernt");
                self.save();
            }
            DELETE_LIST => {
                if self.delete_armed.tap(()) {
                    let name = self.store.list().name.clone();
                    let last = self.store.lists.len() <= 1;
                    self.store.delete_list();
                    self.note(if last { format!("«{name}» geleert") } else { format!("«{name}» gelöscht") });
                    self.roll = 0.0;
                    self.save();
                }
            }
            _ => {}
        }
        Refresh::Dirty
    }
}

impl shell::Client for Shop {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn base(&mut self, _atlas: &mut ImageAtlas, buf: &mut [u8], _w: u32, _h: u32) {
        let p = self.palette;
        let (b, g, r) = ((p.surface.b * 255.0) as u8, (p.surface.g * 255.0) as u8, (p.surface.r * 255.0) as u8);
        for px in buf.chunks_exact_mut(4) {
            px[0] = b;
            px[1] = g;
            px[2] = r;
            px[3] = 0xff;
        }
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, _atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        self.view_inner(engine, w, h)
    }

    fn animating(&self) -> bool {
        self.chrome.animating() || self.root.animating()
    }

    fn glide(&mut self, dt: f32) -> Refresh {
        if self.chrome.glide(dt) | self.root.glide(dt) {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn beat(&mut self) -> Refresh {
        self.beats += 1;
        // A «Wirklich löschen?» (really delete?) that ran out goes back to its plain word.
        let lapsed = self.delete_armed.lapsed();
        if lapsed || self.focus != Focus::None || self.receipt.alive() {
            Refresh::Dirty
        } else {
            Refresh::Calm
        }
    }

    fn pulse(&mut self) -> Refresh {
        let fresh = mk::Palette::load().unwrap_or_default();
        if fresh != self.palette {
            self.palette = fresh;
            return Refresh::Full;
        }
        Refresh::Calm
    }

    fn zoomed(&mut self, on: bool) {
        self.chrome.set_zoomed(on);
    }

    fn extent(&mut self) -> Option<(u32, u32)> {
        self.chrome.extent()
    }

    fn headline(&self) -> f32 {
        self.chrome.headline()
    }

    fn motion(&mut self, x: f64, y: f64) -> Refresh {
        self.chrome.motion(x, y)
    }

    fn depart(&mut self) -> Refresh {
        self.chrome.depart()
    }

    fn strike(&mut self, role: Option<usize>, x: f64, y: f64, source: Source) -> Refresh {
        if let Some(deed) = self.root.strike_at(role, ROOT_BASE) {
            return match deed {
                fam::RootDeed::Help => {
                    matrixkit_scene::help_open("Matrix Einkauf");
                    Refresh::Calm
                }
                fam::RootDeed::Calm => Refresh::Calm,
                _ => Refresh::Dirty,
            };
        }
        match self.chrome.strike(role) {
            Some(fam::ChromeDeed::Close) => {
                eprintln!("[einkauf] Ampel × ({})", source.label());
                self.unfocus();
                return Refresh::Close;
            }
            Some(fam::ChromeDeed::Answer(r)) => return r,
            Some(fam::ChromeDeed::Title) => {
                self.root.toggle();
                return Refresh::Dirty;
            }
            None => {}
        }
        self.press = Some(mk::touch::Press::new(x as f32, y as f32, self.roll, role));
        Refresh::Calm
    }

    fn travel(&mut self, x: f64, y: f64, _source: Source) -> Refresh {
        let Some(pr) = &mut self.press else { return Refresh::Calm };
        match pr.travel(x as f32, y as f32, self.roll_max) {
            Some(roll) => {
                self.roll = roll;
                Refresh::Dirty
            }
            None => Refresh::Calm,
        }
    }

    fn release(&mut self, _role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        match self.press.take().and_then(mk::touch::Press::release) {
            Some(Some(r)) => self.act(r),
            // A tap on the empty page ends typing.
            Some(None) if self.focus != Focus::None => {
                if matches!(self.focus, Focus::ListName(_)) {
                    self.focus = Focus::None;
                    self.edit.clear();
                } else {
                    self.unfocus();
                }
                Refresh::Dirty
            }
            _ => Refresh::Calm,
        }
    }

    fn hold(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        let Some(r) = role else { return Refresh::Calm };
        // A long press takes a product (or a row) off the list.
        let name = if (PRODUCT_BASE..ROW_BASE).contains(&r) {
            self.store.catalog.get(r - PRODUCT_BASE).map(|p| p.name.clone())
        } else if (ROW_BASE..LESS_BASE).contains(&r) {
            self.store.list().items.get(r - ROW_BASE).map(|i| i.name.clone())
        } else {
            None
        };
        let Some(name) = name else { return Refresh::Calm };
        self.press = None;
        if (ROW_BASE..LESS_BASE).contains(&r) {
            self.store.list_mut().items.remove(r - ROW_BASE);
        } else {
            if self.store.on_list(&name).is_none() {
                return Refresh::Calm;
            }
            self.store.remove(&name);
        }
        self.note(format!("«{name}» von der Liste genommen"));
        self.save();
        Refresh::Dirty
    }

    fn axis(&mut self, _x: f64, _y: f64, value: f64, _keep: &mut Keep<f32>) -> Refresh {
        self.roll = (self.roll + value as f32).clamp(0.0, self.roll_max);
        Refresh::Dirty
    }

    fn key(&mut self, key: shell::Key, _shift: bool) -> Refresh {
        let deed = match self.focus {
            Focus::None => return Refresh::Calm,
            Focus::Search => fam::text_key(&mut self.search, &key, 40, |_| true),
            Focus::ListName(_) => fam::text_key(&mut self.edit, &key, 30, |_| true),
            Focus::Qty(_) => fam::text_key(&mut self.edit, &key, 7, |c| c.is_ascii_digit() || c == ',' || c == '.'),
        };
        match deed {
            fam::TextKey::Enter => match self.focus {
                Focus::ListName(_) => self.commit_name(),
                _ => self.unfocus(),
            },
            fam::TextKey::Escape => {
                if self.focus == Focus::Search {
                    self.search.clear();
                }
                self.focus = Focus::None;
                self.edit.clear();
            }
            fam::TextKey::Changed => {
                if self.focus == Focus::Search {
                    self.roll = 0.0;
                }
            }
            fam::TextKey::Ignored => return Refresh::Calm,
        }
        Refresh::Dirty
    }

    fn wants_keyboard(&self) -> bool {
        self.focus != Focus::None
    }

    fn wants_text(&self) -> bool {
        self.focus != Focus::None
    }

    fn root(&mut self) -> Refresh {
        self.root.toggle();
        Refresh::Dirty
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    if !mk::window::single_instance(APP_ID) {
        return Ok(());
    }
    // The GPU paints: scrolling the cards repaints the whole page (on the
    // CPU 46–105 ms a picture on the Duo — «the app lags», 04.11.2026).
    shell::prefer_gpu();
    eprintln!("[einkauf] Matrix Einkauf im Fensterhaus, pid={}", std::process::id());
    fam::run(
        Shop::new(),
        shell::Options {
            mount: shell::Mount::Window { app_id: String::from(APP_ID), title: String::from("Matrix Einkauf"), width: 720, height: 760 },
            argb: false,
            tag: "einkauf",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    )
}
