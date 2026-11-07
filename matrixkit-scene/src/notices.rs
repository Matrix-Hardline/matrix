//! NOTICES FAMILY (round 129): the lock screen's notification centre and
//! its player, as kit parts.
//!
//! * **Groups** — the inbox history (`mk::inbox`) stacked per app, the
//!   newest app first: a closed stack shows its newest card with one or
//!   two edges peeking below and the count on the icon; a tap opens it —
//!   a head («AliExpress», «Weniger», ✕) and every card. A card pushed to
//!   the left shows «Entfernen».
//! * **Player** — what plays (`mk::now_playing`): picture, title, artist,
//!   a time bar to touch, back / play-pause / next.
//!
//! Cards are LIVE GLASS over the scene's backdrop when a map is given
//! (`glass_clock::lens::card`), else a light frosted plate.

use matrixkit_image::Fit;
use matrixkit_shell::{Pict, SceneBuilder};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::inbox::Item;
use matrixkit_theme::now_playing::Playing;
use matrixkit_theme::scene::Rect;
use std::path::{Path, PathBuf};

pub const CARD_H: f32 = 66.0;
pub const GAP: f32 = 8.0;
pub const HEAD_H: f32 = 34.0;
const PEEK: f32 = 7.0;
pub const RADIUS: f32 = 18.0;
/// How far a pushed card stays open (the «Entfernen» button).
pub const OPEN_DX: f32 = -104.0;

const WHITE: mk::Rgba = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

/// A rounded plate as a computed PICTURE (kit: `scene::lay_plate`):
/// plates — quads — lie under every picture of their layer, so stack edges
/// go under a glass card and a count badge over an app icon.
#[allow(clippy::too_many_arguments)]
fn plate(b: &mut SceneBuilder, x: f32, y: f32, w: f32, h: f32, r: f32, color: mk::Rgba, anchor: usize, home: Rect, clip: Option<Rect>) {
    crate::scene::lay_plate(b, Rect { x, y, w, h }, r, color, home, anchor, clip);
}

/// One app's notifications, newest first.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub app: String,
    pub items: Vec<Item>,
}

/// The history in stacks per app, the newest app first.
pub fn group(items: &[Item]) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    let mut sorted: Vec<&Item> = items.iter().collect();
    sorted.sort_by(|a, b| b.at.cmp(&a.at).then(b.id.cmp(&a.id)));
    for it in sorted {
        match groups.iter_mut().find(|g| g.app == it.app) {
            Some(g) => g.items.push(it.clone()),
            None => groups.push(Group { app: it.app.clone(), items: vec![it.clone()] }),
        }
    }
    groups
}

/// «jetzt», «Vor 5 Min.», «Vor 2 Std.», «Gestern», «Vor 3 T.».
pub fn ago(at: u64, now: u64) -> String {
    let s = now.saturating_sub(at);
    match s {
        0..=59 => String::from("jetzt"),
        60..=3599 => format!("Vor {} Min.", s / 60),
        3600..=86_399 => format!("Vor {} Std.", s / 3600),
        86_400..=172_799 => String::from("Gestern"),
        _ => format!("Vor {} T.", s / 86_400),
    }
}

/// What the centre shows and how it is moved.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    /// The open stack (its app).
    pub open: Option<String>,
    /// How far the list is rolled up.
    pub roll: f32,
    /// The card pushed sideways: (id, offset ≤ 0).
    pub swipe: Option<(u32, f32)>,
}

/// One laid row of the centre.
#[derive(Debug, Clone, PartialEq)]
pub enum Row {
    /// A card: its group, the item, how many edges peek below (closed stack).
    Card { group: usize, item: Item, peek: usize, count: usize },
    Head { group: usize },
}

/// The rows from the top with their y and height; the whole height.
pub fn layout(groups: &[Group], state: &State) -> (Vec<(Row, f32, f32)>, f32) {
    let mut rows = Vec::new();
    let mut y = 0.0;
    for (g, grp) in groups.iter().enumerate() {
        if state.open.as_deref() == Some(grp.app.as_str()) && grp.items.len() > 1 {
            rows.push((Row::Head { group: g }, y, HEAD_H));
            y += HEAD_H;
            for it in &grp.items {
                rows.push((Row::Card { group: g, item: it.clone(), peek: 0, count: 1 }, y, CARD_H));
                y += CARD_H + GAP;
            }
        } else {
            let peek = (grp.items.len() - 1).min(2);
            let h = CARD_H + peek as f32 * PEEK;
            rows.push((Row::Card { group: g, item: grp.items[0].clone(), peek, count: grp.items.len() }, y, h));
            y += h + GAP;
        }
    }
    (rows, (y - GAP).max(0.0))
}

/// What a touch on the centre means.
#[derive(Debug, Clone, PartialEq)]
pub enum Hit {
    /// A closed stack: open it.
    Stack(String),
    /// A card (single, or inside an open stack).
    Note(u32),
    /// «Weniger»: close the open stack.
    Less,
    /// ✕ on a head: remove the whole stack.
    Clear(String),
    /// «Entfernen» of the pushed card.
    Remove(u32),
}

const R_LESS: usize = 0;
const R_REMOVE: usize = 1;
const R_GROUP: usize = 1_000;
const R_CLEAR: usize = 2_000;
const R_NOTE: usize = 10_000;
/// How many roles the centre uses above its base.
pub const ROLES: usize = 10_000_000;

pub fn hit(role: Option<usize>, base: usize, groups: &[Group], state: &State) -> Option<Hit> {
    let r = role?.checked_sub(base)?;
    if r >= ROLES {
        return None;
    }
    Some(match r {
        R_LESS => Hit::Less,
        R_REMOVE => Hit::Remove(state.swipe?.0),
        r if (R_GROUP..R_CLEAR).contains(&r) => {
            let g = groups.get(r - R_GROUP)?;
            if g.items.len() > 1 && state.open.as_deref() != Some(g.app.as_str()) {
                Hit::Stack(g.app.clone())
            } else {
                Hit::Note(g.items[0].id)
            }
        }
        r if (R_CLEAR..R_NOTE).contains(&r) => Hit::Clear(groups.get(r - R_CLEAR)?.app.clone()),
        r => Hit::Note((r - R_NOTE) as u32),
    })
}

/// How the centre is drawn this frame.
pub struct Look<'a> {
    /// The card's live-glass map (full width, CARD_H high); None = plate.
    pub glass: Option<&'a Path>,
    /// The app icons (by app name), already looked up.
    pub icon: &'a dyn Fn(&str) -> Option<PathBuf>,
    pub now: u64,
    pub fade: f32,
    pub role_base: usize,
}

/// Draw the centre into `area` (clipped, rolled by `state.roll`). Returns
/// the content height (for the roll's limit).
pub fn lay(b: &mut SceneBuilder, engine: &mut TextEngine, area: Rect, groups: &[Group], state: &State, look: &Look) -> f32 {
    let (rows, total) = layout(groups, state);
    let clip = Some(area);
    let fade = look.fade;
    let w = area.w.floor();
    for (row, ry, rh) in &rows {
        let y = area.y + ry - state.roll;
        if y + rh < area.y || y > area.y + area.h {
            continue;
        }
        match row {
            Row::Head { group } => {
                let grp = &groups[*group];
                let st = mk::typo::TITLE;
                b.text(engine, grp.app.clone(), st.px() * 1.05, mk::typo::Weight::Semibold, matrixkit_text::FAMILY, WHITE.with_alpha(fade), area.x + 4.0, y + 4.0, None, clip);
                // ✕ and «Weniger» — two glassy pills on the right.
                let (xw, xh) = (30.0, 26.0);
                let cx = area.x + w - xw;
                b.quad(cx, y + 3.0, xw, xh, WHITE.with_alpha(0.20 * fade), xh / 2.0, Some(look.role_base + R_CLEAR + group), clip);
                let sym = crate::symbol_family();
                let spx = mk::typo::symbol_px(16.0);
                let (gw, gh) = engine.measure_family(sym, "\u{e5cd}", spx, mk::typo::Weight::Normal);
                b.text(engine, String::from("\u{e5cd}"), spx, mk::typo::Weight::Normal, sym, WHITE.with_alpha(fade), cx + (xw - gw) / 2.0, y + 3.0 + (xh - gh) / 2.0, Some(look.role_base + R_CLEAR + group), clip);
                let less = "Weniger";
                let lpx = mk::typo::CALLOUT.px();
                let (lw, lh) = engine.measure_family(matrixkit_text::FAMILY, less, lpx, mk::typo::Weight::Medium);
                let px = cx - 8.0 - (lw + 22.0);
                b.quad(px, y + 3.0, lw + 22.0, xh, WHITE.with_alpha(0.20 * fade), xh / 2.0, Some(look.role_base + R_LESS), clip);
                b.text(engine, less, lpx, mk::typo::Weight::Medium, matrixkit_text::FAMILY, WHITE.with_alpha(fade), px + 11.0, y + 3.0 + (xh - lh) / 2.0, Some(look.role_base + R_LESS), clip);
            }
            Row::Card { group, item, peek, count } => {
                let in_stack = *peek > 0 || *count > 1;
                let role = Some(if in_stack { look.role_base + R_GROUP + group } else { look.role_base + R_NOTE + item.id as usize });
                // Edges of the stack first (pictures under the card's glass).
                if *peek > 0 {
                    let under = b.quad(area.x, y, w, CARD_H + *peek as f32 * PEEK, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, RADIUS, role, clip);
                    for k in (1..=*peek).rev() {
                        let inset = 12.0 * k as f32;
                        plate(b, area.x + inset, y + CARD_H - RADIUS + k as f32 * PEEK, w - 2.0 * inset, RADIUS, RADIUS / 2.0, WHITE.with_alpha((0.30 - 0.08 * k as f32) * fade), under, area, clip);
                    }
                }
                let dx = state.swipe.filter(|(id, _)| *id == item.id).map(|(_, d)| d).unwrap_or(0.0);
                if dx < -1.0 {
                    // «Entfernen» behind the card.
                    let bw = (-OPEN_DX - 12.0).min(-dx - 6.0).max(0.0);
                    if bw > 20.0 {
                        let bx = area.x + w - bw;
                        b.quad(bx, y, bw, CARD_H, mk::Rgba { r: 0.86, g: 0.24, b: 0.22, a: 0.85 * fade }, RADIUS, Some(look.role_base + R_REMOVE), clip);
                        let word = "Entfernen";
                        let wpx = mk::typo::CALLOUT.px();
                        let (ww, wh) = engine.measure_family(matrixkit_text::FAMILY, word, wpx, mk::typo::Weight::Semibold);
                        if ww + 8.0 < bw {
                            b.text(engine, word, wpx, mk::typo::Weight::Semibold, matrixkit_text::FAMILY, WHITE.with_alpha(fade), bx + (bw - ww) / 2.0, y + (CARD_H - wh) / 2.0, Some(look.role_base + R_REMOVE), clip);
                        }
                    }
                }
                let cx = area.x + dx;
                let anchor = b.quad(cx, y, w, CARD_H, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, RADIUS, role, clip);
                match look.glass {
                    Some(map) => b.pict(Pict { path: map.to_path_buf(), box_w: w as u32, box_h: CARD_H as u32, fit: Fit::Stretch, x: cx, y, radius: RADIUS, home: area, anchor, clip }),
                    None => {
                        b.quad(cx, y, w, CARD_H, WHITE.with_alpha(0.14 * fade), RADIUS, role, clip);
                    }
                }
                // Icon, count, title, time, text.
                let icon = 36.0;
                let (ix, iy) = (cx + 12.0, y + (CARD_H - icon) / 2.0);
                match (look.icon)(&item.app) {
                    Some(path) => {
                        let a = b.quad(ix, iy, icon, icon, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, 9.0, role, clip);
                        b.pict(Pict { path, box_w: icon as u32, box_h: icon as u32, fit: Fit::Contain, x: ix, y: iy, radius: 9.0, home: area, anchor: a, clip });
                    }
                    None => {
                        b.quad(ix, iy, icon, icon, WHITE.with_alpha(0.25 * fade), 9.0, role, clip);
                        let initial = item.app.chars().next().unwrap_or('?').to_uppercase().to_string();
                        let (iw, ih) = engine.measure_family(matrixkit_text::FAMILY, &initial, 17.0, mk::typo::Weight::Semibold);
                        b.text(engine, initial, 17.0, mk::typo::Weight::Semibold, matrixkit_text::FAMILY, WHITE.with_alpha(fade), ix + (icon - iw) / 2.0, iy + (icon - ih) / 2.0, role, clip);
                    }
                }
                let tx = ix + icon + 12.0;
                let when = ago(item.at, look.now);
                let cap = mk::typo::CAPTION;
                let (ww, _) = engine.measure_styled(&when, cap.px(), cap.weight);
                b.styled(engine, when, cap, WHITE.with_alpha(0.7 * fade), cx + w - 14.0 - ww, y + 12.0, role, clip);
                let room = (cx + w - 14.0 - ww - 8.0 - tx).max(20.0);
                let tpx = mk::typo::CALLOUT.px() * 1.05;
                let title = crate::scene::ellipsize(engine, if item.title.is_empty() { &item.app } else { &item.title }, tpx, mk::typo::Weight::Semibold, room);
                b.text(engine, title, tpx, mk::typo::Weight::Semibold, matrixkit_text::FAMILY, WHITE.with_alpha(fade), tx, y + 11.0, role, clip);
                let body = crate::scene::ellipsize(engine, &item.text.replace('\n', " "), mk::typo::CALLOUT.px(), mk::typo::Weight::Normal, cx + w - 14.0 - tx);
                b.text(engine, body, mk::typo::CALLOUT.px(), mk::typo::Weight::Normal, matrixkit_text::FAMILY, WHITE.with_alpha(0.86 * fade), tx, y + 34.0, role, clip);
                // The count on the icon — AFTER the texts: a plate laid after
                // text opens a new layer, so it lies over the icon picture
                // (pictures of a layer cover its plates).
                if *count > 1 {
                    let label = count.to_string();
                    let (lw, lh) = engine.measure_family(matrixkit_text::FAMILY, &label, 11.0, mk::typo::Weight::Semibold);
                    let bw = (lw + 8.0).max(17.0);
                    plate(b, ix - 5.0, iy - 5.0, bw, 17.0, 8.5, mk::Rgba { r: 0.95, g: 0.95, b: 0.97, a: 0.95 * fade }, anchor, area, clip);
                    b.text(engine, label, 11.0, mk::typo::Weight::Semibold, matrixkit_text::FAMILY, mk::Rgba { r: 0.1, g: 0.1, b: 0.12, a: fade }, ix - 5.0 + (bw - lw) / 2.0, iy - 5.0 + (17.0 - lh) / 2.0, role, clip);
                }
            }
        }
    }
    total
}

/// The id of the card at `role` (to push it sideways): a single card or
/// a stack's newest.
pub fn card_at(role: Option<usize>, base: usize, groups: &[Group], state: &State) -> Option<u32> {
    match hit(role, base, groups, state)? {
        Hit::Note(id) => Some(id),
        Hit::Stack(app) => groups.iter().find(|g| g.app == app).map(|g| g.items[0].id),
        _ => None,
    }
}

// ---- The player -----------------------------------------------------

pub const PLAYER_H: f32 = 132.0;

/// What a touch on the player means.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerHit {
    Previous,
    Toggle,
    Next,
    /// The time bar at this share (0…1).
    Seek(f32),
}

const P_PREV: usize = 0;
const P_TOGGLE: usize = 1;
const P_NEXT: usize = 2;
const P_BAR: usize = 3;

/// The player card in `r` (live glass when `glass` is given). Roles from
/// `base` (four). The time bar's rect is returned (for Seek).
#[allow(clippy::too_many_arguments)]
pub fn lay_player(b: &mut SceneBuilder, engine: &mut TextEngine, r: Rect, p: &Playing, glass: Option<&Path>, fade: f32, base: usize, home: Rect) -> Rect {
    let anchor = b.quad(r.x, r.y, r.w, r.h, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, RADIUS + 4.0, None, None);
    match glass {
        Some(map) => b.pict(Pict { path: map.to_path_buf(), box_w: r.w.floor() as u32, box_h: r.h.floor() as u32, fit: Fit::Stretch, x: r.x, y: r.y, radius: RADIUS + 4.0, home, anchor, clip: None }),
        None => {
            b.quad(r.x, r.y, r.w, r.h, WHITE.with_alpha(0.14 * fade), RADIUS + 4.0, None, None);
        }
    }
    let pad = 14.0;
    let art = 54.0;
    let (ax, ay) = (r.x + pad, r.y + pad);
    match &p.art {
        Some(path) => {
            let a = b.quad(ax, ay, art, art, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, 10.0, None, None);
            b.pict(Pict { path: PathBuf::from(path), box_w: art as u32, box_h: art as u32, fit: Fit::Cover, x: ax, y: ay, radius: 10.0, home, anchor: a, clip: None });
        }
        None => {
            // A plate PICTURE (over the card's glass) and a note on it.
            plate(b, ax, ay, art, art, 10.0, WHITE.with_alpha(0.20 * fade), anchor, home, None);
            let sym = crate::symbol_family();
            let spx = mk::typo::symbol_px(28.0);
            let (gw, gh) = engine.measure_family(sym, "\u{e405}", spx, mk::typo::Weight::Normal);
            b.text(engine, String::from("\u{e405}"), spx, mk::typo::Weight::Normal, sym, WHITE.with_alpha(0.85 * fade), ax + (art - gw) / 2.0, ay + (art - gh) / 2.0, None, None);
        }
    }
    let tx = ax + art + 12.0;
    let room = r.x + r.w - pad - tx;
    let tpx = mk::typo::BODY.px() * 1.05;
    let title = crate::scene::ellipsize(engine, &p.title, tpx, mk::typo::Weight::Semibold, room);
    b.text(engine, title, tpx, mk::typo::Weight::Semibold, matrixkit_text::FAMILY, WHITE.with_alpha(fade), tx, ay + 6.0, None, None);
    let sub = if p.artist.is_empty() { p.app.clone() } else { p.artist.clone() };
    let sub = crate::scene::ellipsize(engine, &sub, mk::typo::CALLOUT.px(), mk::typo::Weight::Normal, room);
    b.text(engine, sub, mk::typo::CALLOUT.px(), mk::typo::Weight::Normal, matrixkit_text::FAMILY, WHITE.with_alpha(0.75 * fade), tx, ay + 28.0, None, None);
    // The time bar.
    let by = ay + art + 10.0;
    let bar = Rect { x: r.x + pad, y: by - 8.0, w: r.w - 2.0 * pad, h: 20.0 };
    let pos = p.position_now();
    let share = if p.duration_s > 0.0 { (pos / p.duration_s).clamp(0.0, 1.0) as f32 } else { 0.0 };
    b.quad(bar.x, bar.y, bar.w, bar.h, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, 0.0, Some(base + P_BAR), None);
    // Track and fill as plate PICTURES — plates (quads) lie under the
    // card's glass picture of the same layer.
    plate(b, bar.x, by, bar.w, 4.0, 2.0, WHITE.with_alpha(0.28 * fade), anchor, home, None);
    if share > 0.0 {
        plate(b, bar.x, by, (bar.w * share).max(4.0), 4.0, 2.0, WHITE.with_alpha(0.92 * fade), anchor, home, None);
    }
    let cap = mk::typo::CAPTION;
    let left = mk::now_playing::clock(pos);
    let right = if p.duration_s > 0.0 { format!("−{}", mk::now_playing::clock(p.duration_s - pos)) } else { String::new() };
    b.styled(engine, left, cap, WHITE.with_alpha(0.7 * fade), bar.x, by + 8.0, None, None);
    let (rw, _) = engine.measure_styled(&right, cap.px(), cap.weight);
    b.styled(engine, right, cap, WHITE.with_alpha(0.7 * fade), bar.x + bar.w - rw, by + 8.0, None, None);
    // The buttons, in the middle under the bar.
    let sym = crate::symbol_family();
    let toggle = if p.playing { crate::symbol::PAUSE } else { crate::symbol::PLAY_ARROW };
    let cy = by + 12.0;
    let size = 34.0;
    let mid = r.x + r.w / 2.0;
    for (k, glyph, on, px) in [(P_PREV, crate::symbol::SKIP_PREVIOUS, p.can_previous, 24.0), (P_TOGGLE, toggle, true, 30.0), (P_NEXT, crate::symbol::SKIP_NEXT, p.can_next, 24.0)] {
        let x = mid + (k as f32 - 1.0) * 64.0 - size / 2.0;
        let role = on.then_some(base + k);
        b.quad(x, cy, size, size, mk::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }, size / 2.0, role, None);
        let spx = mk::typo::symbol_px(px);
        let g = glyph.to_string();
        let (gw, gh) = engine.measure_family(sym, &g, spx, mk::typo::Weight::Normal);
        b.text(engine, g, spx, mk::typo::Weight::Normal, sym, WHITE.with_alpha(if on { fade } else { 0.35 * fade }), x + (size - gw) / 2.0, cy + (size - gh) / 2.0, role, None);
    }
    bar
}

/// The player's touch: `bar` from `lay_player`, x for Seek.
pub fn player_hit(role: Option<usize>, base: usize, x: f32, bar: Rect) -> Option<PlayerHit> {
    Some(match role?.checked_sub(base)? {
        P_PREV => PlayerHit::Previous,
        P_TOGGLE => PlayerHit::Toggle,
        P_NEXT => PlayerHit::Next,
        P_BAR => PlayerHit::Seek(((x - bar.x) / bar.w.max(1.0)).clamp(0.0, 1.0)),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u32, app: &str, at: u64) -> Item {
        Item { id, app: app.into(), title: format!("T{id}"), text: String::new(), at, opens: true }
    }

    #[test]
    fn stacks_per_app_newest_first() {
        let items = vec![item(1, "A", 10), item(2, "B", 30), item(3, "A", 40), item(4, "C", 20)];
        let g = group(&items);
        assert_eq!(g.iter().map(|g| g.app.as_str()).collect::<Vec<_>>(), ["A", "B", "C"]);
        assert_eq!(g[0].items.iter().map(|i| i.id).collect::<Vec<_>>(), [3, 1]);
    }

    #[test]
    fn an_open_stack_shows_its_head_and_every_card() {
        let items = vec![item(1, "A", 10), item(2, "A", 20), item(3, "A", 30), item(4, "B", 5)];
        let g = group(&items);
        let closed = layout(&g, &State::default());
        assert_eq!(closed.0.len(), 2);
        assert!(matches!(closed.0[0].0, Row::Card { peek: 2, count: 3, .. }));
        let open = layout(&g, &State { open: Some("A".into()), ..Default::default() });
        assert_eq!(open.0.len(), 1 + 3 + 1);
        assert!(open.1 > closed.1);
    }

    #[test]
    fn touches_name_what_they_meant() {
        let g = group(&[item(1, "A", 10), item(2, "A", 20), item(5, "B", 5)]);
        let st = State::default();
        assert_eq!(hit(Some(100 + R_GROUP), 100, &g, &st), Some(Hit::Stack("A".into())));
        assert_eq!(hit(Some(100 + R_GROUP + 1), 100, &g, &st), Some(Hit::Note(5)));
        assert_eq!(hit(Some(100 + R_NOTE + 7), 100, &g, &st), Some(Hit::Note(7)));
        assert_eq!(hit(Some(100 + R_CLEAR), 100, &g, &st), Some(Hit::Clear("A".into())));
        let pushed = State { swipe: Some((2, OPEN_DX)), ..Default::default() };
        assert_eq!(hit(Some(100 + R_REMOVE), 100, &g, &pushed), Some(Hit::Remove(2)));
        assert_eq!(card_at(Some(100 + R_GROUP), 100, &g, &st), Some(2));
    }

    #[test]
    fn ages_read_like_a_phone() {
        assert_eq!(ago(100, 130), "jetzt");
        assert_eq!(ago(0, 27 * 60), "Vor 27 Min.");
        assert_eq!(ago(0, 2 * 3600 + 5), "Vor 2 Std.");
        assert_eq!(ago(0, 90_000), "Gestern");
    }
}
