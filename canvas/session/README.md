# Matrix canvas session (session candidate)

Additive greetd session that starts the patched niri (`niri-leinwand`) with the
normal environment + `gestures { canvas-drag }` (legacy: leinwand-drag). The regular
niri session stays completely untouched (rollback = pick the normal session).

## Live installation (on the PC, sudo)
```
sudo cp ~/canvas/niri/target/release/niri /usr/local/bin/niri-canvas
sudo install -m755 niri-canvas-session /usr/local/bin/
sudo mkdir -p /usr/local/share/wayland-sessions
sudo cp matrix-canvas.desktop /usr/local/share/wayland-sessions/
cp canvas-session.kdl ~/.config/niri/canvas-session.kdl
cp canvas-start.sh ~/.config/niri/canvas-start.sh   # shell + services (graphical-session fix)
chmod +x ~/.config/niri/canvas-start.sh
niri-leinwand validate -c ~/.config/niri/canvas-session.kdl   # must say "valid"
```
**Grey screen after login?** Starting niri directly does not activate
graphical-session.target → DMS/wallpaper/bar do not start. Fix:
canvas-start.sh (above) is called by the config via spawn-at-startup
and launches the shell + services. (A cleaner solution — a dedicated niri-leinwand.service
bound to graphical-session — is TODO.)

At the next login, pick the session **„Matrix Leinwand"** in the greeter
(the greeter reads XDG_DATA_DIRS, which contains /usr/local/share).

## Image integration (DONE 7.7.)
In the Containerfile: stage `niri-builder` clones niri @a30ca798, applies
0001-leinwand-drag.patch, builds it → /usr/bin/niri-leinwand. Wrapper +
.desktop go to /usr/bin and /usr/share/wayland-sessions respectively. Canvas config
+ start script in /etc/skel/.config/niri (new accounts). After an image update
the session „Matrix Leinwand" is there permanently without re-adding /usr/local.

### Old (before image integration)
Build the patched niri in the Containerfile (CI) → /usr/bin/niri-leinwand,
wrapper + .desktop to /usr/share. Until then it lives in /usr/local
(bootc-writable, does NOT survive updates — set it again after an image update).
