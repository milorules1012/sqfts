//! Phase 1 enrichment overlay (concrete types where wiki has Unknown).

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use sqfts_syntax::Type;

use crate::convert::wiki_name_to_type;

/// Overlay patches keyed by command name (lowercase).
#[derive(Debug, Clone, Default)]
pub struct Overlay {
    /// command → list of param-index → type replacements (and return overlays).
    pub commands: HashMap<String, CommandOverlay>,
}

/// Per-command overlay entries.
#[derive(Debug, Clone, Default)]
pub struct CommandOverlay {
    /// Parameter type overrides: (syntax_index, param_index) → type.
    pub params: Vec<(usize, usize, Type)>,
    /// Return type override per syntax index.
    pub returns: Vec<(usize, Type)>,
}

#[derive(Debug, Deserialize)]
struct PatchFile {
    #[serde(default)]
    name: String,
    #[serde(default)]
    enrichments: Vec<Enrichment>,
}

#[derive(Debug, Deserialize)]
struct Enrichment {
    #[serde(default)]
    syntax: usize,
    #[serde(default)]
    param: Option<usize>,
    #[serde(default)]
    return_type: Option<String>,
    #[serde(default)]
    typ: Option<String>,
    #[serde(default)]
    #[serde(rename = "type")]
    type_alt: Option<String>,
}

impl Overlay {
    /// Load all YAML patches from a directory (Phase 1 `out/patches`).
    pub fn load_dir(dir: &Path) -> Result<Self> {
        let mut overlay = Self::default();
        if !dir.exists() {
            return Ok(overlay);
        }
        for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("yml")
                && path.extension().and_then(|e| e.to_str()) != Some("yaml")
            {
                continue;
            }
            let text = std::fs::read_to_string(&path)?;
            // Patches may be either a single object or our Phase 1 format.
            if let Ok(patch) = serde_yaml::from_str::<PatchFile>(&text) {
                let key = if patch.name.is_empty() {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_ascii_lowercase()
                } else {
                    patch.name.to_ascii_lowercase()
                };
                let slot = overlay.commands.entry(key).or_default();
                for e in patch.enrichments {
                    let ty_name = e.typ.or(e.type_alt).or(e.return_type.clone());
                    if let Some(name) = ty_name {
                        let ty = wiki_name_to_type(&name);
                        if let Some(pi) = e.param {
                            slot.params.push((e.syntax, pi, ty));
                        } else if e.return_type.is_some() || e.param.is_none() {
                            slot.returns.push((e.syntax, ty));
                        }
                    }
                }
            }
        }
        Ok(overlay)
    }
}
