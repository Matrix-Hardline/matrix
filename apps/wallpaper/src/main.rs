//! Matrix Hintergrund — app #21, the wallpaper service.
//!
//! Replaces the last big DMS role: paints the wallpaper onto the
//! background layer AND creates the system palette on image change
//! (matugen with our matrix-palette template) — the color chain thus
//! belongs to Matrix completely. Light/dark follows the sun slot.
//!
//! Image sources (first one wins):
//!   ~/.config/matrix/wallpaper-light|dark (own culture; legacy: hintergrund-*)
//!   session.json wallpaperPath (DMS legacy)
//!   /usr/share/backgrounds/matrix/matrix-standard.jpg (falcon)
//!
//! Surface guard: at session start (spawn-at-startup, sleep 3) the
//! service loses the race against output initialization — the layer
//! surface stays absent or hangs at 0x0, the background stays black.
//! The 2 s pulse therefore checks the surface state: 0x0 → SizeChange
//! nudge (set_size+commit forces a fresh configure); no surface at
//! all → exec restart after GUARD_LIMIT failed pulses, because the
//! single-window pattern cannot create a new surface (NewLayerShell
//! only exists in the multi macro, AllScreens forbids application()).

use matrixkit_widgets::iced;
use matrixkit_widgets::iced_layershell;
use matrixkit_widgets::iced::widget::{image, Space};
use matrixkit_widgets::iced::{Color, Element, Length, Subscription, Task};
use matrixkit_widgets::iced_layershell::to_layer_message;
use matrixkit_theme as mk;
use matrixkit_widgets as mkw;

mod haus;
mod licht;

/// The auto-mode switch, shared between both modes of operation.
pub(crate) fn wallpaper_mode_auto() -> bool {
    mk::settings::read("wallpaper-mode").as_deref() == Some("auto")
}

fn main() -> Result<(), iced_layershell::Error> {
    // Stage 2 of the iced replacement: --erstlicht paints WITHOUT iced
    // (shadow mode; the default stays until the finger probe
    // has decided).
    if std::env::args().nth(1).as_deref() == Some("--erstlicht") {
        // The guard applies to BOTH paths. It stood only in the
        // iced branch, so the light path lacked the open/close gesture: a second
        // key press would have opened a SECOND panel instead of closing
        // the first
        // (finger probe 16.08.2026).
        if !matrixkit_widgets::panel_toggle() {
            return Ok(());
        }
        if let Err(e) = licht::run() {
            eprintln!("[licht] Fehler: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }
    // R216: the HOUSE painter is the standard (no iced); `--iced` keeps
    // the old path as a fallback. Plain start on a duo = conductor that
    // runs one painter per panel, as before.
    if !std::env::args().any(|a| a == "--iced") {
        if matrixkit_widgets::panel::output_arg().is_none() {
            // Round 80: the wallpaper is a DAEMON (the watchdog restarts it) —
            // panel_toggle killed the dying old conductor AND left itself on a
            // quick restart (the notifications' bug of round 76).
            if !matrixkit_widgets::daemon_single() {
                return Ok(());
            }
            // Round 155: one painter per SCREEN — the panels and a TV on
            // USB-C (it got niri's grey only, 05.11.2026).
            // Round 166: the login screen shows the person's wallpaper and
            // lock clock — the conductor shares them (mk::login_share).
            std::thread::spawn(|| loop {
                mk::login_share::publish();
                std::thread::sleep(std::time::Duration::from_secs(5));
            });
            // Without the canvas (a foreign desktop) one painter, as before.
            if mk::device::panels().is_some() || !mk::canvas::outputs().is_empty() {
                mk::process::per_output(mk::device::panels().unwrap_or(&[]));
            }
        }
        matrixkit_shell::prefer_gpu();
        if let Err(e) = haus::run() {
            eprintln!("[wand] Fensterhaus-Fehler: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }
    matrixkit_widgets::present_mailbox();
    use matrixkit_widgets::iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
    use matrixkit_widgets::iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
    eprintln!("[mh] start pid={}", std::process::id());
    // R209: `--output NAME` paints exactly one screen. A duo device has two
    // panels, and one layer surface only ever lands on one of them (the
    // second panel stayed grey) -> the plain start becomes a small
    // conductor that runs one painter per panel.
    let output = matrixkit_widgets::panel::output_arg();
    if output.is_none() {
        if !matrixkit_widgets::daemon_single() {
            eprintln!("[mh] andere Instanz lebt — Ende");
            return Ok(());
        }
        if let Some(panels) = mk::device::panels() {
            matrixkit_widgets::panel::per_panel(panels);
        }
    }
    eprintln!("[mh] toggle ok, layershell startet");
    iced_layershell::application(
        App::new,
        || String::from("matrix-hintergrun"),
        App::update,
        App::view,
    )
    .subscription(App::subscription)
    .style(|_s, _t| iced::theme::Style {
        background_color: Color::BLACK,
        text_color: Color::WHITE,
    })
    .settings(Settings {
        layer_settings: LayerShellSettings {
            size: Some((0, 0)),
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Background,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: -1,
            start_mode: match output {
                Some(name) => StartMode::TargetScreen(name),
                None => StartMode::default(),
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .run()
    .inspect(|_| eprintln!("[mh] run() endete SAUBER (Surface zu?)"))
    .inspect_err(|e| eprintln!("[mh] run() Fehler: {e}"))
}

pub(crate) fn is_light() -> bool {
    std::fs::read_to_string(mk::session_path())
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("isLightMode").and_then(|b| b.as_bool()))
        .unwrap_or(false)
}

/// The currently desired image path — ONE definition for the family
/// (R152): the lock screen reads the very same one.
fn image_path() -> Option<String> {
    mk::wallpaper::current()
}

/// Decode a picture at screen size on a thread of its own.
fn decode_task(path: String) -> Task<Msg> {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    let p = path.clone();
    std::thread::spawn(move || {
        let _ = tx.send(fitted(&p));
    });
    Task::perform(async move { rx.await.ok().flatten() }, move |image| Msg::Decoded(path.clone(), image))
}

/// Only one panel painter makes the palette for a picture
/// (mk::runtime::claim — round 26).
pub(crate) fn palette_claim(path: &str) -> bool {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut h);
    is_light().hash(&mut h);
    mk::runtime::claim(&format!("matrix-palette-{:016x}", h.finish()), std::time::Duration::from_secs(30))
}

/// Create the palette from the image — matugen with our templates
/// (app palette + niri frame colors), then the Firefox bridge.
/// `-m` follows the mode so {{..default.hex}} in the niri template
/// hits the right side.
pub(crate) fn palette_generate(path: &str) {
    let mode = if is_light() { "light" } else { "dark" };
    // Detached (mk::process::spawn_detached): the painter may exec itself
    // anew right after a change, and a waiting thread would die with it.
    mk::process::spawn_detached(
        "sh",
        &["-c", "matugen image \"$1\" -t scheme-tonal-spot --source-color-index 0 -m \"$2\"; pywalfox update", "palette", path, mode],
    );
}

/// Is the sun above the horizon? (Round 71: the kit's `mk::sun`.)
pub(crate) fn sun_top() -> Option<bool> {
    Some(mk::sun::is_up())
}

/// The wallpaper decoded at screen size (R209: whole, an 8400 × 5600 photo
/// held 188 MB per panel painter).
fn fitted(path: &str) -> Option<image::Handle> {
    matrixkit_widgets::image::fitted(path, mk::device::image_long_side())
}

/// Has the palette already been made (matugen output of matrix-palette)?
fn palette_exists() -> bool {
    std::env::var("HOME")
        .map(|h| std::path::Path::new(&h).join(".cache/DankMaterialShell/dms-colors.json").exists())
        .unwrap_or(true)
}

/// After this many pulses without a healthy surface (~10 s) we restart.
const GUARD_LIMIT: u8 = 5;

/// Self-restart via exec: same PID — leiste_toggle does not see itself
/// as "another instance" —, fresh Wayland connection, fresh layer
/// surface. Resolution like leiste::app_starten: dev slot before system.
fn itself_restart() -> ! {
    use std::os::unix::process::CommandExt;
    let local = std::env::var("HOME")
        .map(|h| format!("{h}/.local/bin/matrix-wallpaper"))
        .unwrap_or_default();
    let program = if std::path::Path::new(&local).exists() {
        local
    } else {
        String::from("matrix-wallpaper")
    };
    let error = std::process::Command::new(program).args(std::env::args().skip(1)).exec();
    eprintln!("[mh] guard: exec restart failed: {error}");
    std::process::exit(1);
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Msg {
    Pulse,
    /// R210 cinema mode: a player started or paused (`mk::cinema`).
    Cinema,
    /// One step of the dimming fade.
    Fade,
    /// Answer of the surface guard: size of the youngest surface,
    /// None if no window exists (anymore).
    SurfaceSize(Option<iced::Size>),
    /// A new picture decoded off the drawing thread: (path, image).
    Decoded(String, Option<image::Handle>),
}

struct App {
    path: Option<String>,
    image: Option<image::Handle>,
    /// Consecutive pulses without a healthy surface (guard).
    failed_pulses: u8,
    /// The surface has been seen with a real size — the guard may rest.
    healthy: bool,
    /// Cinema mode (R210): a video plays somewhere — dim towards 1.0.
    cinema: bool,
    /// The dimming as it stands, 0 (bright) … 1 (cinema).
    dim: f32,
}

/// How dark the background gets in cinema mode.
const CINEMA_DARK: f32 = 0.78;

impl App {
    fn new() -> (Self, Task<Msg>) {
        let path = image_path();
        let image = path.as_deref().and_then(fitted);
        // R209: a missing palette (fresh account, matugen installed later)
        // was only ever made on an image CHANGE — make it at start too.
        if let Some(p) = &path {
            if !palette_exists() {
                palette_generate(p);
            }
        }
        let cinema = mk::cinema::active();
        (App { path, image, failed_pulses: 0, healthy: false, cinema, dim: if cinema { 1.0 } else { 0.0 } }, Task::none())
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Cinema => {
                self.cinema = mk::cinema::active();
                Task::none()
            }
            Msg::Fade => {
                // ~0.6 s from bright to cinema; reduced motion jumps.
                let target = if self.cinema { 1.0 } else { 0.0 };
                let step = if mk::motion_reduced() { 1.0 } else { 0.055 };
                self.dim = if self.dim < target { (self.dim + step).min(target) } else { (self.dim - step).max(target) };
                Task::none()
            }
            Msg::Pulse => {
                // A crashed player leaves no change behind — look again.
                self.cinema = mk::cinema::active();
                // REAL sun position (round 23, NSAppearance auto extract):
                // NOAA approximation from location (~/.config/matrix/ort = "lat,lon",
                // default Switzerland) — bright between sunrise and sunset.
                if wallpaper_mode_auto() {
                    if let Some(goal_light) = sun_top() {
                        if goal_light != is_light() {
                            mk::light_toggle();
                        }
                    }
                }
                let new = image_path();
                let mut tasks = Vec::new();
                if new != self.path {
                    self.path = new.clone();
                    if let Some(p) = new {
                        eprintln!("[mh] {} neues Bild: {p}", matrixkit_widgets::panel::output_arg().unwrap_or_default());
                        // Image change = palette change — made ONCE per
                        // change, not by every panel's painter.
                        if palette_claim(&p) {
                            palette_generate(&p);
                        }
                        // A picture swapped in place stayed BLACK on the lower
                        // duo panel (02.11.2026: decoded fine, never drawn; the
                        // guard only noticed after a minute). A fresh start
                        // always draws both panels — so the painter starts
                        // itself anew with the new picture (same pid, exec).
                        if self.image.is_some() {
                            itself_restart();
                        }
                        tasks.push(decode_task(p.clone()));
                    }
                }
                // Surface guard: query the state. Runs even without a window —
                // latest() then answers None.
                tasks.push(iced::window::latest().then(|id| match id {
                    Some(id) => iced::window::size(id).map(|g| Msg::SurfaceSize(Some(g))),
                    None => Task::done(Msg::SurfaceSize(None)),
                }));
                Task::batch(tasks)
            }
            Msg::Decoded(path, image) => {
                eprintln!("[mh] {} dekodiert: {} ({})", matrixkit_widgets::panel::output_arg().unwrap_or_default(), path, if image.is_some() { "ok" } else { "FEHLER" });
                // Only the picture still wanted (a quick second change wins).
                if self.path.as_deref() == Some(path.as_str()) && image.is_some() {
                    self.image = image;
                    // A new picture after start stayed black on the lower duo
                    // panel (measured 02.11.2026: decoded fine, never shown) —
                    // nudge the surface so it configures and draws afresh.
                    return Task::done(Msg::SizeChange((0, 0)));
                }
                Task::none()
            }
            Msg::SurfaceSize(size) => {
                if size.is_some_and(|g| g.width >= 1.0 && g.height >= 1.0) {
                    self.failed_pulses = 0;
                    self.healthy = true;
                    return Task::none();
                }
                self.healthy = false;
                self.failed_pulses += 1;
                eprintln!(
                    "[mh] guard: surface {:?} — missed pulse {}/{}",
                    size, self.failed_pulses, GUARD_LIMIT
                );
                if self.failed_pulses >= GUARD_LIMIT {
                    eprintln!("[mh] guard: surface stays absent — exec restart");
                    itself_restart();
                }
                if size.is_some() {
                    // Configured 0x0: nudge — the runtime sets the size anew and
                    // commits, the compositor answers with a fresh configure.
                    // with a fresh configure.
                    return Task::done(Msg::SizeChange((0, 0)));
                }
                Task::none()
            }
            // variants added by the to_layer_message macro
            _ => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Msg> {
        // R209: every pulse repainted the whole panel and made the compositor
        // compose — every 2 s, on both duo panels. The image now arrives by
        // the session file's change (the Kit's listener thread); the pulse
        // only guards the surface (fast until it is healthy) and follows the
        // sun (slow).
        Subscription::batch([
            mkw::events::file_changes(
                "hintergrund-sitzung",
                mk::session_path().to_string_lossy().into_owned(),
                std::time::Duration::from_millis(500),
            )
            .map(|_| Msg::Pulse),
            // The wallpaper contract itself (the house Settings write only
            // here) — before, a change waited for the minute pulse.
            mkw::events::file_changes(
                "hintergrund-dunkel",
                std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config/matrix").join(mk::wallpaper::key(false)).to_string_lossy().into_owned(),
                std::time::Duration::from_millis(300),
            )
            .map(|_| Msg::Pulse),
            mkw::events::file_changes(
                "hintergrund-hell",
                std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config/matrix").join(mk::wallpaper::key(true)).to_string_lossy().into_owned(),
                std::time::Duration::from_millis(300),
            )
            .map(|_| Msg::Pulse),
            mkw::events::file_changes(
                "hintergrund-kino",
                mk::cinema::path().to_string_lossy().into_owned(),
                std::time::Duration::from_millis(250),
            )
            .map(|_| Msg::Cinema),
            // The fade ticks only while it moves.
            mkw::tick_when(
                "hintergrund-kino-blende",
                (self.dim - if self.cinema { 1.0 } else { 0.0 }).abs() > 0.001,
                std::time::Duration::from_millis(33),
                std::time::Duration::from_secs(3600),
            )
            .map(|_| Msg::Fade),
            mkw::tick_when(
                "hintergrund",
                !self.healthy,
                std::time::Duration::from_secs(2),
                std::time::Duration::from_secs(60),
            )
            .map(|_| Msg::Pulse),
        ])
    }

    fn view(&self) -> Element<'_, Msg> {
        let picture: Element<'_, Msg> = match &self.image {
            Some(h) => image(h.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Cover)
                .into(),
            None => Space::new().into(),
        };
        if self.dim <= 0.0 {
            return picture;
        }
        // Cinema mode: a black veil over the picture.
        let veil = self.dim * CINEMA_DARK;
        iced::widget::stack![
            picture,
            iced::widget::container(Space::new().width(Length::Fill).height(Length::Fill)).style(move |_| {
                // family-exception: the cinema veil is black by definition (it darkens the picture), not a palette role
                iced::widget::container::Style { background: Some(Color::from_rgba(0.0, 0.0, 0.0, veil).into()), ..Default::default() }
            }),
        ]
        .into()
    }
}
