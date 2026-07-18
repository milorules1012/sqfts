//! Dump engine command names from arma3-wiki as JSON for the VS Code grammar generator.
//!
//! ```bash
//! cargo run -p sqfts-db --example dump_command_names
//! ```

use sqfts_db::CommandDb;

fn main() {
    let db = CommandDb::load_default().expect("load arma3-wiki command database");
    let mut words = Vec::new();
    let mut symbols = Vec::new();
    for name in db.command_names() {
        if is_word_command(name) {
            words.push(name.to_string());
        } else if is_symbol_operator(name) {
            symbols.push(name.to_string());
        }
        // Skip MediaWiki junk / redirect titles that are not real tokens.
    }
    words.sort();
    symbols.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    let out = serde_json::json!({
        "words": words,
        "symbols": symbols,
    });
    println!("{}", serde_json::to_string_pretty(&out).expect("serialize"));
}

fn is_word_command(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_symbol_operator(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| matches!(c, '!' | '|' | '&' | '=' | '>' | '<' | '+' | '-' | '*' | '/' | '%' | '^' | ':' | '#'))
}
