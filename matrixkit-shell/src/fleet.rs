//! The island fleet (R134) — at home in the window house since camp 17;
//! born in Central, generic since camp 12.

/// Camp 12: the FLEET moves house (R134) — Central's island
/// fleet, iced-free. mkw::fleet speaks iced::window::Id and
/// iced::Task; here the fleet is GENERIC over the
/// window handle W and, instead of tasks, returns a sync result
/// that the app carries out (open/close surfaces) and
/// acknowledges with adopt() for each opened surface — in
/// sync order. The contract is the same:
/// structure signature, PREFIX KEEPING (equal prefixes keep
/// their windows along with their live state), stacking happens ONLY
/// along the flow axis. The veil stays the app's business — in the
/// test field the main surface IS the veil.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// One below the other, surface by surface (Central).
    Down,
    /// Side by side (dock shelf).
    Across,
}

/// The geometry with which the app opens ONE surface. It
/// translates it 1:1 into its layer settings and
/// decides from which edges main/cross count.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shell {
    pub width: u32,
    pub height: u32,
    /// Offset along the flow axis, stacked by the
    /// fleet.
    pub main: i32,
    /// Offset across it — passed through from the spec.
    pub cross: i32,
    /// Glass namespace (matrix-tile, matrix-card, …).
    pub namespace: &'static str,
    /// Keyboard on demand (search surfaces).
    pub keyboard: bool,
}

/// The recipe of ONE surface in the app's plan.
pub struct Spec<P> {
    pub pane: P,
    pub width: u32,
    pub height: u32,
    pub cross: i32,
    pub namespace: &'static str,
    pub keyboard: bool,
    /// Structure signature: equal prefixes keep their
    /// windows (and with them live state such as morphs).
    pub sig: (u8, u32),
}

/// The diff result: who leaves, who comes.
pub struct Sync<P, W> {
    pub close: Vec<W>,
    pub open: Vec<(Shell, P)>,
}

pub struct Fleet<P, W> {
    panes: Vec<(W, P)>,
    shells: Vec<Shell>,
    sig: Vec<(u8, u32)>,
    start: i32,
    gap: i32,
    flow: Flow,
}

impl<P: Clone, W: Copy + PartialEq> Fleet<P, W> {
    /// The downward fleet — the original form (Central).
    pub fn new(top: i32, gap: i32) -> Self {
        Self::flowing(top, gap, Flow::Down)
    }

    /// The sideways fleet (dock shelf).
    pub fn across(left: i32, gap: i32) -> Self {
        Self::flowing(left, gap, Flow::Across)
    }

    pub fn flowing(start: i32, gap: i32, flow: Flow) -> Self {
        Fleet { panes: Vec::new(), shells: Vec::new(), sig: Vec::new(), start, gap, flow }
    }

    pub fn is_empty(&self) -> bool {
        self.panes.is_empty()
    }

    pub fn ids(&self) -> Vec<W> {
        self.panes.iter().map(|(w, _)| *w).collect()
    }

    pub fn pane(&self, w: W) -> Option<&P> {
        self.panes.iter().find(|(x, _)| *x == w).map(|(_, p)| p)
    }

    /// All surfaces with their FRESH cards (camp 35) — the
    /// house reconciles its pane copies against them.
    pub fn all(&self) -> &[(W, P)] {
        &self.panes
    }

    pub fn shell_of(&self, w: W) -> Option<Shell> {
        let n = self.panes.iter().position(|(x, _)| *x == w)?;
        self.shells.get(n).copied()
    }

    /// The fleet's flow — the house anchors by it (camp 38).
    pub fn flow(&self) -> Flow {
        self.flow
    }

    fn step(&self, s: &Shell) -> i32 {
        (match self.flow {
            Flow::Down => s.height,
            Flow::Across => s.width,
        }) as i32
            + self.gap
    }

    /// The plan against the fleet: the shared
    /// structure prefix keeps its windows (positions
    /// cannot slide — later ones only stack behind),
    /// the rest leaves and comes anew.
    pub fn sync(&mut self, plan: Vec<Spec<P>>) -> Sync<P, W> {
        let sig: Vec<(u8, u32)> = plan.iter().map(|s| s.sig).collect();
        // Round 178: never keep more than was really adopted (a pane skipped for want of an output would make the
        // signature and the panes disagree — `split_off` then panicked).
        let keep = sig
            .iter()
            .zip(self.sig.iter())
            .take_while(|(a, b)| a == b)
            .count()
            .min(self.panes.len());
        if keep == sig.len() && keep == self.sig.len() {
            for (slot, spec) in self.panes.iter_mut().zip(plan) {
                slot.1 = spec.pane;
            }
            return Sync { close: Vec::new(), open: Vec::new() };
        }
        self.sig = sig;
        // The kept prefix gets its FRESH cards too
        // (camp 35: interactive surfaces — the state lives in the
        // model, the card is its snapshot).
        for (slot, spec) in self.panes.iter_mut().zip(plan.iter()).take(keep) {
            slot.1 = spec.pane.clone();
        }
        let close: Vec<W> =
            self.panes.split_off(keep).into_iter().map(|(w, _)| w).collect();
        self.shells.truncate(keep);
        let mut main = self.start
            + self.shells.iter().map(|s| self.step(s)).sum::<i32>();
        let mut open = Vec::new();
        for spec in plan.into_iter().skip(keep) {
            let shell = Shell {
                width: spec.width,
                height: spec.height,
                main,
                cross: spec.cross,
                namespace: spec.namespace,
                keyboard: spec.keyboard,
            };
            main += self.step(&shell);
            open.push((shell, spec.pane));
        }
        Sync { close, open }
    }

    /// Round 80: the compositor CLOSED pane `w` (its output went away, or
    /// it was told to). Give it up together with every pane behind it
    /// (positions stack), so the next `sync` opens them anew. Returns the
    /// windows to destroy (`w` included).
    pub fn forget(&mut self, w: W) -> Vec<W> {
        let Some(n) = self.panes.iter().position(|(x, _)| *x == w) else {
            return Vec::new();
        };
        self.sig.truncate(n);
        self.shells.truncate(n);
        self.panes.split_off(n).into_iter().map(|(x, _)| x).collect()
    }

    /// Round 178 (stability audit): a planned pane could not open (a lock surface without its output — Duo panel
    /// folded, outputs not named yet). Forget the plan from here, so the next `sync` tries it again; before, the
    /// identical plan counted as «unchanged» and the second panel's lock never appeared.
    pub fn skipped(&mut self) {
        self.sig.truncate(self.panes.len());
    }

    /// Acknowledge an opened surface — in sync order.
    pub fn adopt(&mut self, window: W, shell: Shell, pane: P) {
        self.panes.push((window, pane));
        self.shells.push(shell);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skipped_pane_is_planned_again() {
        let mut f: Fleet<&str, u32> = Fleet::new(0, 0);
        let plan = || vec![spec("a", 1), spec("b", 2)];
        let s = f.sync(plan());
        assert_eq!(s.open.len(), 2);
        let mut opened = s.open.into_iter();
        let (shell, pane) = opened.next().unwrap();
        f.adopt(10, shell, pane);
        f.skipped(); // «b» had no output
        let again = f.sync(plan());
        assert_eq!(again.open.len(), 1, "b is tried again");
        assert!(again.close.is_empty());
    }

    fn spec(name: &'static str, sig: u32) -> Spec<&'static str> {
        Spec {
            pane: name,
            width: 260,
            height: 120,
            cross: 24,
            namespace: "matrix-card",
            keyboard: false,
            sig: (1, sig),
        }
    }

    #[test]
    fn a_closed_pane_is_reopened_with_its_followers() {
        let mut f: Fleet<&str, u32> = Fleet::across(10, 4);
        let plan = || vec![spec("A", 0), spec("B", 1), spec("C", 2)];
        for (i, (sh, p)) in f.sync(plan()).open.into_iter().enumerate() {
            f.adopt(i as u32, sh, p);
        }
        assert_eq!(f.forget(1), vec![1, 2]);
        assert_eq!(f.forget(9), Vec::<u32>::new());
        let s = f.sync(plan());
        assert!(s.close.is_empty());
        assert_eq!(s.open.iter().map(|(_, p)| *p).collect::<Vec<_>>(), vec!["B", "C"]);
    }

    #[test]
    fn the_fleet_keeps_the_common_prefix() {
        let mut f: Fleet<&str, u32> = Fleet::new(200, 12);
        let s = f.sync(vec![spec("Dateien", 0)]);
        assert!(s.close.is_empty());
        assert_eq!(s.open.len(), 1);
        assert_eq!(s.open[0].0.main, 200);
        for (sh, p) in s.open {
            f.adopt(10, sh, p);
        }
        // Second surface: the first stays UNTOUCHED.
        let s = f.sync(vec![spec("Dateien", 0), spec("Terminal", 1)]);
        assert!(s.close.is_empty());
        assert_eq!(s.open.len(), 1);
        assert_eq!(s.open[0].0.main, 200 + 120 + 12);
        for (sh, p) in s.open {
            f.adopt(11, sh, p);
        }
        // The head falls: from there everything leaves, the rest comes
        // newly stacked — positions never slide within what exists.
        let s = f.sync(vec![spec("Terminal", 1)]);
        assert_eq!(s.close, vec![10, 11]);
        assert_eq!(s.open.len(), 1);
        assert_eq!(s.open[0].0.main, 200);
        for (sh, p) in s.open {
            f.adopt(12, sh, p);
        }
        // Same plan: calm costs nothing.
        let s = f.sync(vec![spec("Terminal", 1)]);
        assert!(s.close.is_empty() && s.open.is_empty());
        assert_eq!(f.ids(), vec![12]);
    }

    #[test]
    fn the_fleet_flows_across_too() {
        let mut f: Fleet<&str, u32> = Fleet::across(48, 8);
        let s = f.sync(vec![spec("a", 0), spec("b", 1)]);
        assert_eq!(s.open[0].0.main, 48);
        assert_eq!(s.open[1].0.main, 48 + 260 + 8);
    }
}
