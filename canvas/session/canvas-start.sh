#!/bin/sh
# The canvas session keeps the desktop-mode note in sync.
mkdir -p "$HOME/.config/matrix" && printf leinwand > "$HOME/.config/matrix/leinwand-modus" && rm -f "$HOME/.config/matrix/desktop-modus"
# Matrix canvas session: start shell + services. With a direct niri
# start (not via niri.service) graphical-session.target does not become
# active automatically — hence the shell directly here and the services
# via systemctl. Idempotent. Place at ~/.config/niri/canvas-start.sh.
systemctl --user start --no-block graphical-session.target 2>/dev/null
# Matrix instead of DMS (2026-07-08): wallpaper + palette is done by
# matrix-hintergrund (spawn-at-startup in local.kdl); Quickshell no longer starts.
# Way back in need: the old line was
#   pgrep -u "$(id -u)" -f "dms run" || setsid dms run --session &
systemctl --user start matrix-sound-hooks.service 2>/dev/null || \
    systemctl --user start matrix-klang-hooks.service 2>/dev/null
systemctl --user start matrix-dock.service iio-niri.service 2>/dev/null
