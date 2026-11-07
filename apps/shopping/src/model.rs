//! The shopping model — products, lists, quantities; no drawing here.
//!
//! Stored as plain lines in ~/.config/matrix/shopping-lists (tab-separated,
//! written atomically): own products, the lists with their items, how often
//! a product was put on a list (the «often» order) and the active list.

use std::collections::HashMap;
use std::path::PathBuf;

/// Category order on screen (the walk through a Swiss shop).
pub const CATEGORIES: [&str; 9] = [
    "Früchte & Gemüse",
    "Brot & Backwaren",
    "Milch & Eier",
    "Fleisch & Fisch",
    "Vorrat",
    "Tiefkühl",
    "Getränke",
    "Haushalt",
    "Sonstiges",
];

/// Where own products go when no category is chosen.
pub const OTHER: &str = "Sonstiges";

pub const UNITS: [&str; 6] = ["Stk", "Pack", "g", "kg", "ml", "l"];

/// The built-in catalogue: (name, category, unit).
const BUILT_IN: &[(&str, &str, &str)] = &[
    ("Äpfel", "Früchte & Gemüse", "Stk"),
    ("Bananen", "Früchte & Gemüse", "Stk"),
    ("Orangen", "Früchte & Gemüse", "Stk"),
    ("Zitronen", "Früchte & Gemüse", "Stk"),
    ("Tomaten", "Früchte & Gemüse", "Stk"),
    ("Gurke", "Früchte & Gemüse", "Stk"),
    ("Salat", "Früchte & Gemüse", "Stk"),
    ("Karotten", "Früchte & Gemüse", "kg"),
    ("Kartoffeln", "Früchte & Gemüse", "kg"),
    ("Zwiebeln", "Früchte & Gemüse", "Stk"),
    ("Knoblauch", "Früchte & Gemüse", "Stk"),
    ("Peperoni", "Früchte & Gemüse", "Stk"),
    ("Brot", "Brot & Backwaren", "Stk"),
    ("Zopf", "Brot & Backwaren", "Stk"),
    ("Gipfeli", "Brot & Backwaren", "Stk"),
    ("Toastbrot", "Brot & Backwaren", "Pack"),
    ("Milch", "Milch & Eier", "l"),
    ("Eier", "Milch & Eier", "Stk"),
    ("Butter", "Milch & Eier", "Stk"),
    ("Joghurt", "Milch & Eier", "Stk"),
    ("Käse", "Milch & Eier", "g"),
    ("Rahm", "Milch & Eier", "ml"),
    ("Quark", "Milch & Eier", "Stk"),
    ("Poulet", "Fleisch & Fisch", "g"),
    ("Hackfleisch", "Fleisch & Fisch", "g"),
    ("Schinken", "Fleisch & Fisch", "Pack"),
    ("Lachs", "Fleisch & Fisch", "g"),
    ("Mehl", "Vorrat", "kg"),
    ("Zucker", "Vorrat", "kg"),
    ("Salz", "Vorrat", "Pack"),
    ("Reis", "Vorrat", "kg"),
    ("Teigwaren", "Vorrat", "Pack"),
    ("Öl", "Vorrat", "l"),
    ("Essig", "Vorrat", "l"),
    ("Hefe", "Vorrat", "Pack"),
    ("Tomatensauce", "Vorrat", "Stk"),
    ("Kaffee", "Vorrat", "Pack"),
    ("Tee", "Vorrat", "Pack"),
    ("Pizza", "Tiefkühl", "Stk"),
    ("Gemüse tiefgekühlt", "Tiefkühl", "Pack"),
    ("Glace", "Tiefkühl", "Pack"),
    ("Mineralwasser", "Getränke", "l"),
    ("Orangensaft", "Getränke", "l"),
    ("Bier", "Getränke", "Stk"),
    ("Toilettenpapier", "Haushalt", "Pack"),
    ("Abwaschmittel", "Haushalt", "Stk"),
    ("Waschmittel", "Haushalt", "Stk"),
    ("Abfallsäcke", "Haushalt", "Pack"),
];

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub name: String,
    pub category: String,
    pub unit: String,
    /// Typed in by the user (kept in the file; built-ins are not).
    pub own: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    pub category: String,
    pub qty: f32,
    pub unit: String,
    /// In the basket (struck through).
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub name: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Category,
    Alphabet,
    Often,
}

pub const SORTS: [(Sort, &str); 3] = [(Sort::Category, "Kategorie"), (Sort::Alphabet, "A–Z"), (Sort::Often, "Häufig")];

#[derive(Debug, Clone, PartialEq)]
pub struct Store {
    pub catalog: Vec<Product>,
    pub lists: Vec<List>,
    pub active: usize,
    /// How often a product went on a list.
    pub uses: HashMap<String, u32>,
}

/// The step a − / + makes in this unit.
pub fn step(unit: &str) -> f32 {
    match unit {
        "g" | "ml" => 100.0,
        "kg" | "l" => 0.5,
        _ => 1.0,
    }
}

/// What the first tap puts on the list.
pub fn first(unit: &str) -> f32 {
    match unit {
        "g" => 250.0,
        "ml" => 500.0,
        _ => 1.0,
    }
}

/// «2», «0,5», «250» — Swiss decimal comma, no trailing zeros.
pub fn qty_text(q: f32) -> String {
    if (q - q.round()).abs() < 0.001 {
        format!("{}", q.round() as i64)
    } else {
        format!("{:.2}", q).trim_end_matches('0').trim_end_matches('.').replace('.', ",")
    }
}

/// A typed quantity («2», «0,5», «1.5»); None for nothing sensible.
pub fn parse_qty(s: &str) -> Option<f32> {
    let v: f32 = s.trim().replace(',', ".").parse().ok()?;
    (v > 0.0 && v < 100_000.0).then_some(v)
}

/// The sort key of a name: lower case, umlauts as their base letter
/// («Äpfel» belongs to the A, not behind «Zwiebeln»).
pub fn sort_key(s: &str) -> String {
    s.to_lowercase().chars().map(|c| match c {
        'ä' | 'à' | 'â' => 'a',
        'ö' | 'ô' => 'o',
        'ü' | 'û' => 'u',
        'é' | 'è' | 'ê' => 'e',
        _ => c,
    }).collect()
}

pub fn category_rank(c: &str) -> usize {
    CATEGORIES.iter().position(|x| *x == c).unwrap_or(CATEGORIES.len())
}

fn path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"));
    base.join("matrix").join("shopping-lists")
}

fn clean(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ").trim().to_string()
}

impl Default for Store {
    fn default() -> Self {
        Store {
            catalog: built_in(),
            lists: vec![List { name: String::from("Einkauf"), items: Vec::new() }],
            active: 0,
            uses: HashMap::new(),
        }
    }
}

fn built_in() -> Vec<Product> {
    BUILT_IN
        .iter()
        .map(|(n, c, u)| Product { name: n.to_string(), category: c.to_string(), unit: u.to_string(), own: false })
        .collect()
}

impl Store {
    pub fn load() -> Store {
        std::fs::read_to_string(path()).map(|t| Store::parse(&t)).unwrap_or_default()
    }

    pub fn parse(text: &str) -> Store {
        let mut s = Store { catalog: built_in(), lists: Vec::new(), active: 0, uses: HashMap::new() };
        for line in text.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            match f.as_slice() {
                ["product", name, cat, unit] if !name.is_empty() => {
                    if !s.catalog.iter().any(|p| p.name.eq_ignore_ascii_case(name)) {
                        s.catalog.push(Product { name: name.to_string(), category: cat.to_string(), unit: unit.to_string(), own: true });
                    }
                }
                ["list", name] => s.lists.push(List { name: name.to_string(), items: Vec::new() }),
                ["item", name, cat, qty, unit, done] => {
                    if let Some(l) = s.lists.last_mut() {
                        l.items.push(Item {
                            name: name.to_string(),
                            category: cat.to_string(),
                            qty: qty.parse().unwrap_or(1.0),
                            unit: unit.to_string(),
                            done: *done == "1",
                        });
                    }
                }
                ["use", name, n] => {
                    s.uses.insert(name.to_string(), n.parse().unwrap_or(0));
                }
                ["active", n] => s.active = n.parse().unwrap_or(0),
                _ => {}
            }
        }
        if s.lists.is_empty() {
            s.lists.push(List { name: String::from("Einkauf"), items: Vec::new() });
        }
        s.active = s.active.min(s.lists.len() - 1);
        s
    }

    pub fn text(&self) -> String {
        let mut out = String::from("# Matrix Einkauf\n");
        for p in self.catalog.iter().filter(|p| p.own) {
            out += &format!("product\t{}\t{}\t{}\n", p.name, p.category, p.unit);
        }
        for l in &self.lists {
            out += &format!("list\t{}\n", l.name);
            for i in &l.items {
                out += &format!("item\t{}\t{}\t{}\t{}\t{}\n", i.name, i.category, i.qty, i.unit, if i.done { 1 } else { 0 });
            }
        }
        let mut uses: Vec<_> = self.uses.iter().collect();
        uses.sort();
        for (n, c) in uses {
            out += &format!("use\t{n}\t{c}\n");
        }
        out += &format!("active\t{}\n", self.active);
        out
    }

    /// Atomic (tmp + rename) — a half-written file would lose the lists.
    pub fn save(&self) {
        let p = path();
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = p.with_extension("new");
        if std::fs::write(&tmp, self.text()).is_ok() {
            let _ = std::fs::rename(&tmp, &p);
        }
    }

    pub fn list(&self) -> &List {
        &self.lists[self.active]
    }

    pub fn list_mut(&mut self) -> &mut List {
        &mut self.lists[self.active]
    }

    /// How much of `name` the active list holds (open items).
    pub fn on_list(&self, name: &str) -> Option<&Item> {
        self.list().items.iter().find(|i| i.name == name && !i.done)
    }

    /// A product card was tapped: put it on the list, or one step more.
    pub fn add(&mut self, product: usize) {
        let Some(p) = self.catalog.get(product).cloned() else { return };
        *self.uses.entry(p.name.clone()).or_default() += 1;
        let l = self.list_mut();
        if let Some(i) = l.items.iter_mut().find(|i| i.name == p.name && !i.done) {
            i.qty += step(&i.unit);
        } else {
            l.items.push(Item { name: p.name, category: p.category, qty: first(&p.unit), unit: p.unit, done: false });
        }
    }

    /// Take a product off the active list (a long press on its card).
    pub fn remove(&mut self, name: &str) {
        self.list_mut().items.retain(|i| i.name != name || i.done);
    }

    /// A product the user typed in; returns its catalogue index.
    pub fn own_product(&mut self, name: &str, category: &str) -> Option<usize> {
        let name = clean(name);
        if name.is_empty() {
            return None;
        }
        if let Some(i) = self.catalog.iter().position(|p| p.name.eq_ignore_ascii_case(&name)) {
            return Some(i);
        }
        let cat = if CATEGORIES.contains(&category) { category } else { OTHER };
        self.catalog.push(Product { name, category: cat.to_string(), unit: String::from("Stk"), own: true });
        Some(self.catalog.len() - 1)
    }

    pub fn new_list(&mut self, name: &str) {
        let mut name = clean(name);
        if name.is_empty() {
            name = format!("Liste {}", self.lists.len() + 1);
        }
        self.lists.push(List { name, items: Vec::new() });
        self.active = self.lists.len() - 1;
    }

    pub fn rename_list(&mut self, name: &str) {
        let name = clean(name);
        if !name.is_empty() {
            self.list_mut().name = name;
        }
    }

    /// The active list goes (the last one only empties).
    pub fn delete_list(&mut self) {
        if self.lists.len() <= 1 {
            self.lists[0].items.clear();
            return;
        }
        self.lists.remove(self.active);
        self.active = self.active.min(self.lists.len() - 1);
    }

    pub fn clear_done(&mut self) {
        self.list_mut().items.retain(|i| !i.done);
    }

    /// Catalogue indices to show: filtered by category and search, sorted.
    pub fn products(&self, filter: Option<&str>, search: &str, sort: Sort) -> Vec<usize> {
        let q = search.trim().to_lowercase();
        let mut v: Vec<usize> = (0..self.catalog.len())
            .filter(|&i| {
                let p = &self.catalog[i];
                filter.map_or(true, |c| p.category == c) && (q.is_empty() || p.name.to_lowercase().contains(&q))
            })
            .collect();
        let name = |i: &usize| sort_key(&self.catalog[*i].name);
        match sort {
            Sort::Category => v.sort_by_key(|i| (category_rank(&self.catalog[*i].category), name(i))),
            Sort::Alphabet => v.sort_by_key(name),
            Sort::Often => v.sort_by_key(|i| (std::cmp::Reverse(self.uses.get(&self.catalog[*i].name).copied().unwrap_or(0)), name(i))),
        }
        v
    }

    /// Item indices of the active list: open first (in `sort`), then the basket.
    pub fn items(&self, sort: Sort) -> (Vec<usize>, Vec<usize>) {
        let items = &self.list().items;
        let key = |i: &usize| {
            let it = &items[*i];
            let rank = if sort == Sort::Category { category_rank(&it.category) } else { 0 };
            (rank, sort_key(&it.name))
        };
        let mut open: Vec<usize> = (0..items.len()).filter(|&i| !items[i].done).collect();
        let mut done: Vec<usize> = (0..items.len()).filter(|&i| items[i].done).collect();
        open.sort_by_key(key);
        done.sort_by_key(key);
        (open, done)
    }
}

impl Store {
    /// The open items as text to paste into a message — grouped by
    /// category in shop order, one «• name – amount unit» a line.
    pub fn share_text(&self) -> String {
        let (open, _) = self.items(Sort::Category);
        let items = &self.list().items;
        let mut out = format!("{}\n", self.list().name);
        let mut last: Option<&str> = None;
        for i in open {
            let it = &items[i];
            if last != Some(it.category.as_str()) {
                out += &format!("\n{}\n", it.category);
                last = Some(it.category.as_str());
            }
            out += &format!("• {} – {} {}\n", it.name, qty_text(it.qty), it.unit);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_reads_as_a_message() {
        let mut s = Store::default();
        for n in ["Mehl", "Äpfel", "Mehl"] {
            let i = s.catalog.iter().position(|p| p.name == n).unwrap();
            s.add(i);
        }
        assert_eq!(s.share_text(), "Einkauf\n\nFrüchte & Gemüse\n• Äpfel – 1 Stk\n\nVorrat\n• Mehl – 1,5 kg\n");
    }

    #[test]
    fn quantities_read_and_write_the_swiss_way() {
        assert_eq!(qty_text(2.0), "2");
        assert_eq!(qty_text(0.5), "0,5");
        assert_eq!(qty_text(1.25), "1,25");
        assert_eq!(parse_qty("0,5"), Some(0.5));
        assert_eq!(parse_qty("1.5"), Some(1.5));
        assert_eq!(parse_qty("0"), None);
        assert_eq!(parse_qty("Brot"), None);
    }

    #[test]
    fn a_tap_adds_then_steps_and_the_file_round_trips() {
        let mut s = Store::default();
        let mehl = s.catalog.iter().position(|p| p.name == "Mehl").unwrap();
        let eier = s.catalog.iter().position(|p| p.name == "Eier").unwrap();
        s.add(mehl);
        s.add(mehl);
        s.add(eier);
        assert_eq!(s.on_list("Mehl").map(|i| i.qty), Some(1.5));
        assert_eq!(s.on_list("Eier").map(|i| (i.qty, i.unit.as_str())), Some((1.0, "Stk")));
        let own = s.own_product("Hafermilch", "Milch & Eier").unwrap();
        s.add(own);
        s.list_mut().items[1].done = true;
        s.new_list("Party");
        let back = Store::parse(&s.text());
        assert_eq!(back.lists, s.lists);
        assert_eq!(back.active, 1);
        assert!(back.catalog.iter().any(|p| p.name == "Hafermilch" && p.own));
        assert_eq!(back.uses.get("Mehl"), Some(&2));
    }

    #[test]
    fn filter_search_and_sort() {
        let mut s = Store::default();
        let brot = s.catalog.iter().position(|p| p.name == "Brot").unwrap();
        s.add(brot);
        let bake = s.products(Some("Brot & Backwaren"), "", Sort::Alphabet);
        assert!(bake.iter().all(|&i| s.catalog[i].category == "Brot & Backwaren"));
        let found = s.products(None, "mil", Sort::Category);
        assert!(found.iter().all(|&i| s.catalog[i].name.to_lowercase().contains("mil")));
        assert_eq!(s.products(None, "", Sort::Often)[0], brot);
        let cat = s.products(None, "", Sort::Category);
        assert_eq!(s.catalog[cat[0]].category, CATEGORIES[0]);
        assert_eq!(s.catalog[cat[0]].name, "Äpfel");
    }

    #[test]
    fn the_basket_goes_last_and_lists_survive_deleting() {
        let mut s = Store::default();
        for n in ["Mehl", "Äpfel", "Eier"] {
            let i = s.catalog.iter().position(|p| p.name == n).unwrap();
            s.add(i);
        }
        s.list_mut().items[0].done = true; // Mehl (flour)
        let (open, done) = s.items(Sort::Category);
        assert_eq!(open.iter().map(|&i| s.list().items[i].name.as_str()).collect::<Vec<_>>(), vec!["Äpfel", "Eier"]);
        assert_eq!(done.len(), 1);
        s.clear_done();
        assert_eq!(s.list().items.len(), 2);
        s.delete_list();
        assert_eq!(s.lists.len(), 1);
        assert!(s.list().items.is_empty());
    }
}
