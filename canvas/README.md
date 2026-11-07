# The canvas compositor (matrix-leinwand)

Reinventing the user desktop: the infinite 2D desktop without scrollbars.
Full project journal: das lokale Projekt-Journal (local).

## Patches against niri (base a30ca798 = shipped version)

- `0001-leinwand-drag.patch`: config flag `gestures { leinwand-drag }` +
  a left-click drag on empty space starts the existing
  SpatialMovementGrab (2D pan across columns ↔ and workspaces ↕).
- `0002-smithay-keymap-per-wl_keyboard.patch`: applies NOT against niri,
  but against **Smithay** @ff5fa7df (niri's pinned revision — re-check on
  a NIRI_COMMIT bump). Fix for the keyboard bug ("." typed "q",
  R59b): send_keymap deduplicates seat-globally via SHA-256, but freshly
  bound wl_keyboards only get the seat keymap — windows that bind after
  the on-screen keyboard's vk upload interpret vk keys wrongly.
  The patch tracks the last sent keymap PER wl_keyboard.
  Integration in the niri-builder: clone+patch /smithay, a `[patch]` entry
  is appended to niri's Cargo.toml during the build (the 0001 patch deliberately
  leaves Cargo.toml/Lock out). After the deploy the client-side
  workaround in matrixkit/apps/tastatur/src/einspeisung.rs (alternating
  F19/F20 keymaps, commit 75a7c51) can be removed.
  Nested proof: /tmp/kbtest.sh on the Surface (old: "." → Escape,
  new: keymap event before the key → period). Upstream draft: no. 11 in
  das lokale Projekt-Journal.
- `0005-canvas-round-maximized.patch`: a MAXIMISED window keeps
  its corner radius. Upstream, in `src/layout/tile.rs`
  (`render`), pulls the radius down to 0 with `1. - expanded_progress`, and
  `expanded_progress()` returns 1 as soon as the `SizingMode` is `Maximized`
  OR `Fullscreen`. For fullscreen that is right — there the window lies
  borderless against the screen. On the canvas, however, "maximised" keeps
  the same air (`gaps`) to the screen edge and the shelf as any other
  window; it is a normal window, just big, and must keep its corners.
  The patch rescues ONLY the radius (border, shadow and
  focus ring still hang on `expanded_progress`, so the
  layout measures stay untouched). It is the only patch that touches
  `src/layout/tile.rs` — 0001 and 0003 leave the file alone,
  so the patch can be regenerated independently.
  Measurement (Surface, 17.08.2026): maximised, the window outline was an
  exact rectangle at EVERY configured radius (scanned line by line,
  first window pixel constantly at x=20), while the same app
  unmaximised rounded cleanly. What looked round at the top edge was the
  app's header bar (`fam::frame_header` rounds its top itself), not
  the window outline — that is why only the LOWER corners stood out.

## Building (on the PC)

```
distrobox enter leinwand -- bash -lc "cd ~/leinwand/niri && cargo build --release"
```

## Testing nested (safe)

```
~/leinwand/niri/target/release/niri -c /tmp/leinwand-test.kdl
# Apps into it: WAYLAND_DISPLAY=wayland-2 <app>
```
