//! STACK FAMILY (round 148, 05.11.2026): a vertical stack of pages like a
//! phone's widget stack — one page shows, a vertical swipe moves the pages
//! 1:1 with the finger, a fifth of a page turns it, letting go glides to
//! the page; dots on the side say where you are. Born in the lock's clock
//! sheet (Dicke, Deckkraft); any card with several faces can use it.
//!
//! ```ignore
//! // touch:   stack.press(y)  →  stack.drag(y, area.h)  →  stack.release()
//! // frame:   if stack.tick(dt) { redraw }
//! // drawing: for (k, off) in stack.visible(area.h) { lay page k shifted by off, clipped to area }
//! //          stack::lay_dots(&mut b, area, stack.pos, stack.len, fade, home, anchor);
//! ```

use matrixkit_shell::SceneBuilder;
use matrixkit_theme as mk;
use matrixkit_theme::scene::Rect;

/// A fifth of a page turns it.
pub const TURN: f32 = 0.2;
/// Past the first and last page the pages move a quarter as far.
pub const RUBBER: f32 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stack {
    /// How many pages.
    pub len: usize,
    /// The page chosen (where the stack glides to).
    pub page: usize,
    /// Where the stack is (fractions while it glides or follows a finger).
    pub pos: f32,
    /// A finger holds the pages: where it landed and the position then.
    held: Option<(f32, f32)>,
}

impl Stack {
    pub fn new(len: usize) -> Stack {
        Stack { len: len.max(1), page: 0, pos: 0.0, held: None }
    }

    /// Resting on its page (a slider on the page may take a finger).
    pub fn settled(&self) -> bool {
        self.held.is_none() && (self.pos - self.page as f32).abs() < 0.02
    }

    pub fn held(&self) -> bool {
        self.held.is_some()
    }

    /// A finger lands on the pages at `y`.
    pub fn press(&mut self, y: f32) {
        self.held = Some((y, self.pos));
    }

    /// The finger is at `y`; a page is `h` tall.
    pub fn drag(&mut self, y: f32, h: f32) {
        let Some((y0, pos0)) = self.held else { return };
        let last = (self.len - 1) as f32;
        let p = pos0 - (y - y0) / h.max(1.0);
        self.pos = if p < 0.0 {
            p * RUBBER
        } else if p > last {
            last + (p - last) * RUBBER
        } else {
            p
        };
    }

    /// The finger lifted: the page to glide to. None if no finger held it.
    pub fn release(&mut self) -> Option<usize> {
        let (_, pos0) = self.held.take()?;
        let from = pos0.round();
        let moved = self.pos - from;
        let to = if moved > TURN {
            from + 1.0
        } else if moved < -TURN {
            from - 1.0
        } else {
            from
        };
        self.page = to.clamp(0.0, (self.len - 1) as f32) as usize;
        Some(self.page)
    }

    /// Go to `page` (gliding).
    pub fn show(&mut self, page: usize) {
        self.page = page.min(self.len - 1);
    }

    /// One frame: the stack glides to its page. True while it moves.
    pub fn tick(&mut self, dt: f32) -> bool {
        if self.held.is_some() {
            return false;
        }
        let to = self.page as f32;
        if (self.pos - to).abs() < 0.002 {
            self.pos = to;
            return false;
        }
        self.pos += (to - self.pos) * (1.0 - (-dt * 16.0).exp());
        true
    }

    /// The pages in sight and their offsets (px) for a page `h` tall.
    pub fn visible(&self, h: f32) -> Vec<(usize, f32)> {
        (0..self.len).filter_map(|k| {
            let off = (k as f32 - self.pos) * h;
            (off.abs() < h).then_some((k, off))
        }).collect()
    }
}

/// The page dots on the right edge of `area`, centred: the page at `pos`
/// bright, the others faint. A shape picture (quads hide under a glass
/// card, which is a picture itself — round 149).
pub fn lay_dots(b: &mut SceneBuilder, area: Rect, pos: f32, len: usize, fade: f32, home: Rect, anchor: usize) {
    use matrixkit_shell::shapes::{Cmd, Shape, Style};
    if len < 2 {
        return;
    }
    let white = mk::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    let now = pos.round().clamp(0.0, (len - 1) as f32) as usize;
    let (dot, gap) = (6.0, 10.0);
    let total = len as f32 * dot + (len - 1) as f32 * gap;
    let mut shape = Shape::new(dot, total.ceil());
    for k in 0..len {
        let cy = k as f32 * (dot + gap) + dot / 2.0;
        shape = shape.op(vec![Cmd::Circle(dot / 2.0, cy, dot / 2.0)], Style::Fill(white.with_alpha(if k == now { 0.9 } else { 0.32 } * fade)));
    }
    b.pict(crate::scene::pict_of(&shape, area.x + area.w - 14.0, area.y + (area.h - total) / 2.0, home, anchor));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fifth_turns_a_page_and_it_glides_there() {
        let mut s = Stack::new(3);
        s.press(500.0);
        s.drag(500.0 - 30.0, 100.0); // 0.3 page up
        assert!((s.pos - 0.3).abs() < 1e-5, "1:1 with the finger");
        assert_eq!(s.release(), Some(1));
        let mut frames = 0;
        while s.tick(0.016) {
            frames += 1;
        }
        assert_eq!(s.pos, 1.0);
        assert!(frames > 3 && frames < 60, "{frames}");
        assert!(s.settled());
    }

    #[test]
    fn a_twitch_stays_and_the_ends_resist() {
        let mut s = Stack::new(2);
        s.press(500.0);
        s.drag(490.0, 100.0);
        assert_eq!(s.release(), Some(0));
        while s.tick(0.016) {}
        assert_eq!(s.pos, 0.0, "it glides back");
        s.press(500.0);
        s.drag(600.0, 100.0); // a page down past the first
        assert!((s.pos + RUBBER).abs() < 1e-5);
        assert_eq!(s.release(), Some(0));
        assert_eq!(Stack::new(2).release(), None, "no finger, no turn");
    }

    #[test]
    fn only_neighbours_are_visible() {
        let mut s = Stack::new(3);
        assert_eq!(s.visible(100.0), vec![(0, 0.0)]);
        s.pos = 0.5;
        assert_eq!(s.visible(100.0), vec![(0, -50.0), (1, 50.0)]);
    }
}
