//! matrix-trust (Morpheus, 06.11.2026): «Diesem Laptop vertrauen?» — like an
//! iPhone on a new computer. `matrix-kopplung` starts it when Morpheus on a
//! laptop at the USB cable asks to pair; the answer is the exit code
//! (0 = trust, 1 = not). No answer within two minutes is a «no».

use matrixkit_image::ImageAtlas;
use matrixkit_scene::scene as fam;
use matrixkit_scene::symbol;
use matrixkit_shell::{self as shell, Refresh, Scene, SceneBuilder, Source};
use matrixkit_text::TextEngine;
use matrixkit_theme as mk;
use matrixkit_theme::scene::Keep;
use std::time::{Duration, Instant};

const APP_ID: &str = "matrix-trust";
const TRUST: usize = 1;
const DENY: usize = 2;
const WAIT: Duration = Duration::from_secs(120);

fn l<'a>(de: &'a str, en: &'a str) -> &'a str {
    mk::language::pick(de, en)
}

struct Trust {
    laptop: String,
    palette: mk::Palette,
    since: Instant,
}

/// The answer, as the exit code — the window closes with it.
fn answer(yes: bool) -> ! {
    if yes {
        mk::feel::confirm();
    }
    std::process::exit(if yes { 0 } else { 1 });
}

impl shell::Client for Trust {
    type Card = ();
    type Work = ();

    fn dress(&mut self, engine: &mut TextEngine) {
        fam::arm_symbols(engine);
    }

    fn view(&mut self, engine: &mut TextEngine, _keep: &mut Keep<f32>, _atlas: &mut ImageAtlas, w: u32, h: u32) -> Scene {
        let p = self.palette;
        let (wf, hf) = (w as f32, h as f32);
        let mut b = SceneBuilder::new();
        b.quad(0.0, 0.0, wf, hf, p.surface, 0.0, None, None);
        let m = mk::spacing::L;
        let col = wf - 2.0 * m;
        let mut y = m;
        // The laptop symbol in a soft disc.
        let disc = 64.0;
        b.quad((wf - disc) / 2.0, y, disc, disc, p.primary_container, disc / 2.0, None, None);
        let glyph = symbol::LAPTOP.to_string();
        let gpx = mk::typo::symbol_px(34.0);
        let (gw, gh) = engine.measure_family(matrixkit_scene::symbol_family(), &glyph, gpx, mk::typo::Weight::Normal);
        b.text(engine, glyph, gpx, mk::typo::Weight::Normal, matrixkit_scene::symbol_family(), p.on_primary_container, (wf - gw) / 2.0, y + (disc - gh) / 2.0, None, None);
        y += disc + mk::spacing::M;
        y += fam::lay_centred(&mut b, engine, l("Diesem Laptop vertrauen?", "Trust this laptop?"), mk::typo::TITLE, p.on_surface, wf / 2.0, col, y) + mk::spacing::S;
        y += fam::lay_centred(&mut b, engine, &self.laptop, mk::typo::BODY, p.primary, wf / 2.0, col, y) + mk::spacing::M;
        y += fam::lay_centred(
            &mut b,
            engine,
            l(
                "Morpheus auf diesem Laptop kann dann über das USB-Kabel deine Dateien und Fotos sehen und kopieren und Matrix sichern. Entziehen geht jederzeit in den Einstellungen.",
                "Morpheus on this laptop can then see and copy your files and photos over the USB cable and back up Matrix. You can withdraw it at any time in Settings.",
            ),
            mk::typo::CALLOUT,
            p.on_surface_variant,
            wf / 2.0,
            col,
            y,
        ) + mk::spacing::L;
        fam::lay_pill_row(
            &mut b,
            engine,
            &[
                (Some(symbol::CLOSE), l("Nicht vertrauen", "Don't trust"), Some(DENY), fam::PillTone::Quiet),
                (Some(symbol::CHECK), l("Vertrauen", "Trust"), Some(TRUST), fam::PillTone::Accent),
            ],
            wf / 2.0,
            y,
            mk::spacing::M,
            p,
        );
        b.finish()
    }

    fn strike(&mut self, role: Option<usize>, _x: f64, _y: f64, _source: Source) -> Refresh {
        match role {
            Some(TRUST) => answer(true),
            Some(DENY) => answer(false),
            _ => Refresh::Calm,
        }
    }

    fn beat(&mut self) -> Refresh {
        if self.since.elapsed() > WAIT {
            answer(false);
        }
        Refresh::Calm
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut laptop = String::from("Laptop");
    while let Some(a) = args.next() {
        if a == "--laptop" {
            laptop = args.next().unwrap_or(laptop);
        }
    }
    mk::feel::warn();
    let app = Trust { laptop, palette: mk::Palette::load().unwrap_or_default(), since: Instant::now() };
    if let Err(e) = shell::run(
        app,
        shell::Options {
            mount: shell::Mount::Window { app_id: String::from(APP_ID), title: String::from(l("Diesem Laptop vertrauen?", "Trust this laptop?")), width: 440, height: 420 },
            argb: false,
            tag: "trust",
            fleet_start: 0,
            fleet_gap: 0,
            fleet_flow: shell::fleet::Flow::Down,
        },
    ) {
        eprintln!("[trust] {e}");
        std::process::exit(1);
    }
    // The window closed without an answer.
    std::process::exit(1);
}
