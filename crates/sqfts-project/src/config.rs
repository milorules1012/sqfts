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
    /// Output directory for `sqfts build`.
    pub out_dir: PathBuf,
    /// Emit runtime params guards (SPEC §7.4).
    pub emit_runtime_params: bool,
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
            out_dir: PathBuf::from("out/sqf"),
            emit_runtime_params: false,
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
}
