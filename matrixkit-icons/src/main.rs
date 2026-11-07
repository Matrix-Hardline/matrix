//! MatrixKit living icons 2.0 — app icons that breathe with the wallpaper.
//!
//! Following the reference principle, icons here are SEPARATE LAYERS
//! instead of finished images: a tile (squircle with depth) and a glyph
//! (the motif). Variants arise from that — standard (colored) and tinted
//! (monochrome glyph, like the reference's tinted mode), chosen via
//!
//! The tile is a true superellipse (continuous curvature — the subliminal
//! reference look that rounded rectangles never hit) with a fine vertical
//! gradient and light edge; the glyph casts a soft core shadow.

use matrixkit_theme as mk;
use std::path::PathBuf;
use std::time::Duration;
use tiny_skia::{Pixmap, Transform};

/// Master canvas of the app icons; smaller hicolor sizes are
/// downscaled at high quality (launcher/dock/bars use different ones).
const SIZE: u32 = 256;
const ALL_SIZES: &[u32] = &[48, 64, 128, 256];

use matrixkit_icons::*;

fn main() {
    // System mode (image build): render all icons with the default palette
    // into an icon root — stable names, no versioned copies, no
    // desktop/avatar side effects. The image thus ships a static fallback
    // icon of all apps for EVERY (even new) account; the living recolor
    // is handled later by the per-user color sync.
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--system") {
        let root = PathBuf::from(args.get(i + 1).cloned().unwrap_or_else(|| "/usr/share/icons".into()));
        system_render(&root);
        return;
    }

    // Plymouth mode (image build): render boot sign + pulse dot for the
    // Matrix boot theme — palette-free (see lib::plymouth_logo_png).
    if let Some(i) = args.iter().position(|a| a == "--plymouth") {
        let dir = PathBuf::from(
            args.get(i + 1).cloned().unwrap_or_else(|| "/usr/share/plymouth/themes/matrix".into()),
        );
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("matrixkit-icons: Plymouth-Verzeichnis nicht anlegbar: {e}");
            std::process::exit(1);
        }
        let Some(logo) = plymouth_logo_png() else {
            eprintln!("matrixkit-icons: Plymouth-Zeichen nicht renderbar");
            std::process::exit(1);
        };
        let _ = std::fs::write(dir.join("watermark.png"), logo);
        // two-step convention (see spinner theme): throbber-* is the RUNNING
        // pulse animation, animation-* the end animation. Without throbber
        // the plugin only paints a still image top left.
        const FRAMES: u32 = 30;
        for f in 0..FRAMES {
            if let Some(png) = plymouth_animation_png(f, FRAMES) {
                let _ = std::fs::write(dir.join(format!("throbber-{:04}.png", f + 1)), &png);
                let _ = std::fs::write(dir.join(format!("animation-{:04}.png", f + 1)), &png);
            }
        }
        println!("{}", dir.join("watermark.png").display());
        return;
    }

    // Race protection: the sync fires while matugen is still writing the
    // color file. If need be (file entirely missing) draw with the fallback palette.
    let palette = mk::Palette::load_settled(Duration::from_secs(3)).unwrap_or_default();

    let dir = icon_dir(SIZE);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("matrixkit-icons: Icon-Verzeichnis nicht anlegbar: {e}");
        std::process::exit(1);
    }

    let glyph = match style() {
        Style::Standard => Glyph {
            a: palette.primary,
            b: palette.secondary,
            c: palette.tertiary,
        },
        // Tinted: monochrome glyph in shades of the text color
        Style::Tinted => Glyph {
            a: with_alpha(palette.on_surface, 1.0),
            b: with_alpha(palette.on_surface, 0.72),
            c: with_alpha(palette.on_surface, 0.5),
        },
    };

    let _ = &glyph;
    for spec in ICONS {
        // R207: ONE path for all — bitmap or tile + glyph.
        let Some(pixmap) = icon_pixmap(&palette, spec) else { continue };

        let png = match pixmap.encode_png() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("matrixkit-icons: {} nicht kodierbar: {e}", spec.name);
                continue;
            }
        };
        // Stable name in ALL hicolor sizes — for classic theme lookups.
        for &g in ALL_SIZES {
            let gdir = icon_dir(g);
            let _ = std::fs::create_dir_all(&gdir);
            let stable = gdir.join(format!("{}.png", spec.name));
            let bytes = if g == SIZE {
                png.clone()
            } else {
                let mut small = Pixmap::new(g, g).expect("Pixmap");
                let s = g as f32 / SIZE as f32;
                small.draw_pixmap(
                    0,
                    0,
                    pixmap.as_ref(),
                    &tiny_skia::PixmapPaint {
                        quality: tiny_skia::FilterQuality::Bilinear,
                        ..Default::default()
                    },
                    Transform::from_scale(s, s),
                    None,
                );
                match small.encode_png() {
                    Ok(b) => b,
                    Err(_) => continue,
                }
            };
            if let Err(e) = std::fs::write(&stable, &bytes) {
                eprintln!("matrixkit-icons: {} nicht schreibbar: {e}", stable.display());
            }
        }
        // Versioned name — cache buster for running shells: Quickshell/Qt
        // cache images PER PATH; a color hash in the file name turns every
        // palette state into a fresh path.
        let versioned_name = format!("{}-{:08x}.png", spec.name, fnv1a(&png));
        let versioned = dir.join(&versioned_name);
        if !versioned.exists() {
            let _ = std::fs::write(&versioned, &png);
        }
        prune_old_versions(&dir, spec.name, &versioned_name);
        // Point Icon= of the user desktop entry to the fresh path — this
        // also nudges DMS' desktop-entry watcher (re-resolution).
        update_desktop_icon(spec.name, &versioned);
    }

    // Living avatar (desktop, lock screen, greeter): only if the user
    // enabled it via marker — a personal photo in ~/.face is never
    // overwritten (delete the marker to use your own picture).
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    // R107: avatar-live is the contract; avatar-lebendig read as legacy.
    if std::path::Path::new(&format!("{home}/.config/matrix/avatar-live")).exists()
        || std::path::Path::new(&format!("{home}/.config/matrix/avatar-lebendig")).exists() {
        if let Some(png) = avatar_png(&palette) {
            let _ = std::fs::write(format!("{home}/.face"), png);
        }
    }

    // Refresh the icon theme cache (best effort).
    if let Some(icons_root) = dir.ancestors().nth(2) {
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .arg(icons_root)
            .output();
    }
}


/// Write all app icons to `<root>/hicolor/<g>x<g>/apps/<name>.png`.
/// For the image build (default palette, stable names, nothing else).
fn system_render(root: &std::path::Path) {
    let palette = mk::Palette::default();
    for spec in ICONS {
        let Some(pixmap) = icon_pixmap(&palette, spec) else { continue };
        let png_large = match pixmap.encode_png() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("matrixkit-icons: {} nicht kodierbar: {e}", spec.name);
                continue;
            }
        };
        for &g in ALL_SIZES {
            let gdir = root.join(format!("hicolor/{g}x{g}/apps"));
            if std::fs::create_dir_all(&gdir).is_err() {
                continue;
            }
            let target = gdir.join(format!("{}.png", spec.name));
            let bytes = if g == SIZE {
                png_large.clone()
            } else {
                let mut small = Pixmap::new(g, g).expect("Pixmap");
                let s = g as f32 / SIZE as f32;
                small.draw_pixmap(
                    0,
                    0,
                    pixmap.as_ref(),
                    &tiny_skia::PixmapPaint {
                        quality: tiny_skia::FilterQuality::Bilinear,
                        ..Default::default()
                    },
                    Transform::from_scale(s, s),
                    None,
                );
                match small.encode_png() {
                    Ok(b) => b,
                    Err(_) => continue,
                }
            };
            if let Err(e) = std::fs::write(&target, &bytes) {
                eprintln!("matrixkit-icons: {} nicht schreibbar: {e}", target.display());
            }
        }
    }
    let _ = std::process::Command::new("gtk-update-icon-cache")
        .arg(root.join("hicolor"))
        .output();
}

// --- management (cache buster, desktop entries) -----------------------------

/// FNV-1a over the PNG bytes — the same palette yields the same name,
/// so no file churn with an unchanged theme.
fn fnv1a(data: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in data {
        h ^= *b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Remove older versioned icon states of the same name.
fn prune_old_versions(dir: &std::path::Path, name: &str, keep: &str) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let prefix = format!("{name}-");
    for e in entries.flatten() {
        let f = e.file_name();
        let Some(f) = f.to_str() else { continue };
        if f.starts_with(&prefix) && f.ends_with(".png") && f != keep {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

/// Set `Icon=` of the user desktop entry to the versioned path.
/// If the user entry is missing, the system entry is copied as template.
/// Written atomically (tmp + rename) — half .desktop files would feed
/// the shell's desktop-entry watcher parse garbage.
fn update_desktop_icon(name: &str, icon_path: &std::path::Path) {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    let user_dir = PathBuf::from(&home).join(".local/share/applications");
    let user_entry = user_dir.join(format!("{name}.desktop"));
    let template = if user_entry.exists() {
        user_entry.clone()
    } else {
        // The entry that counts (any app dir, R215: /usr/local was missed).
        let Some(sys) = matrixkit_theme::apps::desktop_path(name) else {
            return; // app not (yet) installed — nothing to do
        };
        sys
    };
    let Ok(content) = std::fs::read_to_string(&template) else { return };
    let new_line = format!("Icon={}", icon_path.display());
    let mut replaced = false;
    let mut out: Vec<String> = content
        .lines()
        .map(|l| {
            if l.starts_with("Icon=") {
                replaced = true;
                new_line.clone()
            } else {
                l.to_string()
            }
        })
        .collect();
    if !replaced {
        out.push(new_line);
    }
    let new_content = out.join("\n") + "\n";
    if new_content == content {
        return; // already current — do not wake the watcher unnecessarily
    }
    if std::fs::create_dir_all(&user_dir).is_err() {
        return;
    }
    let tmp = user_dir.join(format!(".{name}.desktop.new"));
    if std::fs::write(&tmp, new_content).is_ok() {
        let _ = std::fs::rename(&tmp, &user_entry);
    }
}

fn icon_dir(size: u32) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    PathBuf::from(home).join(format!(".local/share/icons/hicolor/{size}x{size}/apps"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sysmon_icon_carries_palette_colors() {
        let p = mk::Palette::default();
        let g = Glyph { a: p.primary, b: p.secondary, c: p.tertiary };
        let mut pixmap = Pixmap::new(SIZE, SIZE).unwrap();
        tile(&p, &mut pixmap);
        glyph_sysmon(&g, &mut Painter::new(&mut pixmap));
        // Bar 1 (center) carries primary — with rounding tolerance, since
        // premultiplied alpha + antialiasing may cost ±4.
        // Icons 3.0 draws the glyph freely — instead of a fixed coordinate
        // we look for the invariant: SOMEWHERE in the motif area lies an
        // opaque pixel in primary (±4 rounding tolerance).
        let goal = (p.primary.r * 255.0).round() as i32;
        let mut found = false;
        for y in 40..220 {
            for x in 40..220 {
                let px = pixmap.pixel(x, y).unwrap();
                if px.alpha() > 200 && (px.red() as i32 - goal).abs() <= 4 {
                    found = true;
                }
            }
        }
        assert!(found, "no primary pixel in the glyph area");
        // Corners stay transparent (squircle tile)
        assert_eq!(pixmap.pixel(2, 2).unwrap().alpha(), 0);
        // The tile ground is a gradient: brighter on top than below
        let top = pixmap.pixel(128, 20).unwrap();
        let bottom = pixmap.pixel(128, 236).unwrap();
        assert!(top.red() > bottom.red() || top.green() > bottom.green());
    }
}
