//! Render chosen app icons to PNG files for a look (round 58):
//! `cargo run -p matrixkit-icons --example preview -- <dir> <app>…`
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((dir, apps)) = args.split_first() else {
        eprintln!("Aufruf: preview <verzeichnis> <app>…");
        std::process::exit(2);
    };
    let palette = matrixkit_theme::Palette::load().unwrap_or_default();
    for app in apps {
        match matrixkit_icons::render_png(app, &palette) {
            Some(png) => {
                let path = std::path::Path::new(dir).join(format!("{app}.png"));
                let _ = std::fs::write(&path, png);
                println!("{}", path.display());
            }
            None => eprintln!("{app}: kein Icon"),
        }
    }
}
