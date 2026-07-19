//! `sqfts.toml` project configuration.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use sqfts_check::CheckFlags;

/// Project configuration loaded from `sqfts.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SqftsConfig {
    /// Project root (directory containing sqfts.toml or the path given).
    #[serde(skip)]
    pub root: PathBuf,
    /// Glob-like roots for `.sqfts` sources (relative to root).
    pub sources: Vec<String>,
    /// Paths to `.d.sqfts` declaration files or directories.
    pub declarations: Vec<String>,
    /// Extra HEMTT include roots for `#include` (relative to project root).
    ///
    /// `None` (field omitted) → auto-add `./include` when that directory exists.
    /// `Some([])` → no include roots. `Some([...])` → those paths only.
    #[serde(default)]
    pub include_paths: Option<Vec<String>>,
    /// Output directory for `sqfts build`.
    pub out_dir: PathBuf,
    /// Emit runtime params guards (SPEC §7.4).
    pub emit_runtime_params: bool,
    /// When true, the language server erases and writes `.sqf` on each `.sqfts` save.
    pub build_on_save: bool,
    /// Type-checker strictness flags.
    pub flags: CheckFlags,
    /// Options for `sqfts declgen`.
    pub declgen: DeclgenConfig,
}

/// `[declgen]` section of `sqfts.toml`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DeclgenConfig {
    /// Leading path segments to strip from `file = "..."` before resolving under
    /// `--root` (e.g. `["addon_name/"]`). Set per-project when config paths
    /// include a PBO/addon prefix that is not present on disk.
    pub strip_prefixes: Vec<String>,
}

impl Default for SqftsConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            sources: vec![".".into()],
            declarations: vec![],
            include_paths: None,
            out_dir: PathBuf::from("out/sqf"),
            emit_runtime_params: false,
            build_on_save: false,
            flags: CheckFlags::default(),
            declgen: DeclgenConfig::default(),
        }
    }
}

impl SqftsConfig {
    /// Load from `path` (file or directory). Missing file → defaults.
    pub fn load(path: &str) -> Result<Self> {
        let p = PathBuf::from(path);
        let (root, toml_path) = if p.is_file() {
            (
                p.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(),
                p,
            )
        } else {
            let candidate = p.join("sqfts.toml");
            (p, candidate)
        };
        let mut cfg = if toml_path.is_file() {
            let text = std::fs::read_to_string(&toml_path)
                .with_context(|| format!("reading {}", toml_path.display()))?;
            toml::from_str::<SqftsConfig>(&text)
                .with_context(|| format!("parsing {}", toml_path.display()))?
        } else {
            SqftsConfig::default()
        };
        cfg.root = root;
        Ok(cfg)
    }

    /// Load from an absolute/relative project root path.
    pub fn load_path(path: &Path) -> Result<Self> {
        Self::load(path.to_string_lossy().as_ref())
    }

    /// Resolve a config-relative path against the project root.
    #[must_use]
    pub fn resolve(&self, rel: &str) -> PathBuf {
        let p = Path::new(rel);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.root.join(p)
        }
    }

    /// Absolute include directories for the HEMTT preprocessor.
    ///
    /// When `include_paths` is omitted from `sqfts.toml`, adds `./include` if it
    /// exists (HEMTT convention). Explicit lists are resolved and filtered to
    /// existing directories.
    #[must_use]
    pub fn resolved_include_paths(&self) -> Vec<PathBuf> {
        match &self.include_paths {
            Some(paths) => paths
                .iter()
                .map(|p| self.resolve(p))
                .filter(|p| p.is_dir())
                .collect(),
            None => {
                let include = self.root.join("include");
                if include.is_dir() {
                    vec![include]
                } else {
                    vec![]
                }
            }
        }
    }
}
