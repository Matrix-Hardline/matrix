//! Matrix Einkauf — shopping lists in the window house (no iced).
//!
//! Products as cards by category (or A–Z, or how often), filter and
//! search, an amount per item, struck off once in the basket. Several
//! lists; everything in ~/.config/matrix/shopping-lists.

mod haus;
mod model;

fn main() {
    if let Err(e) = haus::run() {
        eprintln!("[einkauf] Fensterhaus-Fehler: {e}");
    }
}
