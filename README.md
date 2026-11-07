# Matrix

**English** · [Deutsch](#deutsch)

The apps of Matrix, an independent Linux desktop for the first-generation Microsoft Surface Duo — built with
**MatrixKit**, without a third-party widget library. This repository holds the source of everything Morpheus puts on a
Duo: bar, dock, launcher, control centre, notifications, lock screen, login screen (with installer, recovery and first
setup), settings, files, terminal, Tube, player, camera, screen capture, shopping list, help and more — plus
**Morpheus** itself (the laptop app) and `matrix-trust` («Trust this laptop?»).

To install Matrix on a Duo, see the companion repository
[`matrix-duo`](https://github.com/Matrix-Hardline/matrix-duo) — its *Releases* hold Morpheus as one ready file.

> [!WARNING]
> **For experienced users only, at your own risk:** installing Matrix can leave a Surface Duo unusable («bricked»).
> No warranty — read the warning in [`matrix-duo`](https://github.com/Matrix-Hardline/matrix-duo) first.

| Folder | Contents |
|---|---|
| `apps/` | the apps (one crate `matrix-<name>` each) |
| `apps/morpheus-duo` | Morpheus: install, start, back up and restore Matrix on a Duo; files and photos |
| `apps/trust` | `matrix-trust`: the Duo asks whether to trust a laptop |
| `matrixkit-*` | MatrixKit (layout, text, image and vector engines, GPU painting, window house, families) |
| `canvas/` | the changes to the niri window manager (patches on pinned niri and Smithay revisions) |
| `vendor/` | two adapted libraries (layershellev, iced_layershell) for the remaining iced parts |

## Building

    cargo build --release -p matrix-bar        # one app
    cargo build --release                      # all of them

Toolchain: see `rust-toolchain.toml` (tested with Rust 1.95). Only the Wayland and libxkbcommon development packages
are needed (Fedora: `dnf install cargo rust gcc pkgconf-pkg-config wayland-devel libxkbcommon-devel`). `cargo test` also
needs a system font (e.g. `google-noto-sans-fonts`), because the layout tests measure real text. For the Duo
(aarch64) the apps are built in a Fedora 42 environment, see `matrix-duo`.

## Licence

The apps and Morpheus: **GPL-3.0-or-later** (`LICENSE`). The `matrixkit-*` crates are **MIT** (`LICENSE-MIT`, as in
the standalone `matrixkit` repository). The patches in `canvas/` follow niri (GPL-3.0), the libraries in `vendor/`
their own licences.

Status: alpha. Tested on a Surface Duo (Android 2022.902.48), a Surface Laptop 7 (ARM) and a Surface Book 2 (Intel).

---

## Deutsch

Die Apps von Matrix, einem eigenständigen Linux-Desktop für das Microsoft Surface Duo (erste Generation) — gebaut mit
**MatrixKit**, ohne fremde Widget-Bibliothek. Dieses Repo enthält den Quellcode von allem, was Morpheus auf ein Duo
bringt: Leiste, Dock, Start, Zentrale, Mitteilungen, Sperre, Anmeldebildschirm (mit Installer, Recovery und
Ersteinrichtung), Einstellungen, Dateien, Terminal, Tube, Player, Kamera, Aufnahme, Einkauf, Hilfe und weitere — dazu
**Morpheus** selbst (die Laptop-App) und `matrix-trust` («Diesem Laptop vertrauen?»).

> [!WARNING]
> **Nur für erfahrene Nutzerinnen und Nutzer, auf eigene Verantwortung:** Die Installation kann ein Surface Duo
> unbrauchbar machen («bricken»). Ohne Gewährleistung — zuerst den Hinweis in `matrix-duo` lesen.

Matrix auf ein Duo installieren: siehe das Begleit-Repo
[`matrix-duo`](https://github.com/Matrix-Hardline/matrix-duo) — unter *Releases* liegt Morpheus als fertige Datei.

| Ordner | Inhalt |
|---|---|
| `apps/` | die Apps (je eine Kiste `matrix-<name>`) |
| `apps/morpheus-duo` | Morpheus: Matrix auf ein Duo installieren, starten, sichern, wiederherstellen; Dateien und Fotos |
| `apps/trust` | `matrix-trust`: das Duo fragt, ob es einem Laptop vertrauen soll |
| `matrixkit-*` | MatrixKit (Layout-, Text-, Bild- und Vektormaschine, GPU-Malerei, Fensterhaus, Familien) |
| `canvas/` | die Änderungen am Fenstermanager niri (Patches auf festen Ständen von niri und Smithay) |
| `vendor/` | zwei angepasste Bibliotheken (layershellev, iced_layershell) für die verbleibenden iced-Teile |

**Bauen:** `cargo build --release` (alle) oder `-p matrix-bar` (eine App). Rust 1.95; nur die Entwicklerpakete von
Wayland und libxkbcommon; für `cargo test` zusätzlich eine Schrift (z. B. `google-noto-sans-fonts`). Für das Duo wird
in Fedora 42 gebaut, siehe `matrix-duo`.

**Lizenz:** Apps und Morpheus **GPL-3.0-or-later**, die Kisten `matrixkit-*` **MIT**, `canvas/` wie niri (GPL-3.0).

Stand: Alpha. Getestet auf einem Surface Duo, einem Surface Laptop 7 (ARM) und einem Surface Book 2 (Intel).
