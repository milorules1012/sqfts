//! Discover and filter COMREF-md engine-command pages.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::ParseOutcome;
use crate::parse::{command_name_from_stem, parse_comref_page};

#[derive(Debug, Clone)]
pub struct CorpusEntry {
    pub path: PathBuf,
    pub stem: String,
    pub name: String,
}

/// Walk `comref_dir` and return engine-command pages only (BIS/BIN skipped, dupes collapsed).
pub fn discover_engine_commands(comref_dir: &Path) -> anyhow::Result<Vec<CorpusEntry>> {
    let mut by_name: HashMap<String, CorpusEntry> = HashMap::new();
    let mut seen_decoded_stems: HashSet<String> = HashSet::new();

    for entry in fs::read_dir(comref_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        if should_skip_stem(&stem) {
            continue;
        }

        // Prefer decoded filenames over %-encoded duplicates.
        let decoded = percent_encoding::percent_decode_str(&stem)
            .decode_utf8_lossy()
            .into_owned();
        let is_encoded = stem.contains('%');
        if is_encoded && seen_decoded_stems.contains(&decoded) {
            continue;
        }
        // If we already have an encoded entry and now see the decoded twin, replace.
        let name = command_name_from_stem(&stem);
        if let Some(existing) = by_name.get(&name) {
            let existing_encoded = existing.stem.contains('%');
            if existing_encoded && !is_encoded {
                // replace with decoded
            } else {
                continue;
            }
        }

        seen_decoded_stems.insert(decoded);
        by_name.insert(name.clone(), CorpusEntry { path, stem, name });
    }

    let mut entries: Vec<_> = by_name.into_values().collect();
    entries.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });
    Ok(entries)
}

#[must_use]
pub fn should_skip_stem(stem: &str) -> bool {
    let lower = stem.to_ascii_lowercase();
    if lower.starts_with("bis_fnc_") || lower.starts_with("bin_fnc_") {
        return true;
    }
    if lower.starts_with("bis_som_") {
        return true;
    }
    // Generated analysis reports, not wiki pages
    if matches!(
        stem,
        "commands_code_execution_parameters" | "BIS_fnc_code_execution_parameters"
    ) {
        return true;
    }
    false
}

/// Parse every discovered engine-command page.
pub fn parse_corpus(comref_dir: &Path) -> anyhow::Result<Vec<ParseOutcome>> {
    let entries = discover_engine_commands(comref_dir)?;
    let mut outcomes = Vec::with_capacity(entries.len());
    for entry in entries {
        let content = fs::read_to_string(&entry.path)?;
        outcomes.push(parse_comref_page(&entry.stem, &content));
    }
    Ok(outcomes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_bis_bin() {
        assert!(should_skip_stem("BIS_fnc_spawnGroup"));
        assert!(should_skip_stem("BIN_fnc_foo"));
        assert!(!should_skip_stem("setDamage"));
        assert!(!should_skip_stem("a_%26%26_b"));
    }
}
