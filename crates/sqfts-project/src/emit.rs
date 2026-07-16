//! Shared erase-and-write helpers for CLI and LSP.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sqfts_syntax::{erase, erase_with_runtime_params, EraseOptions};

use crate::config::SqftsConfig;

/// Erase annotations from `src` and write the resulting `.sqf` under `cfg.out_dir`.
///
/// Destination is `out_dir` joined with the source path relative to the project
/// root, with the extension changed to `.sqf`. Returns the destination path.
pub fn emit_file(cfg: &SqftsConfig, path: &Path, src: &str) -> Result<PathBuf> {
    let erased = if cfg.emit_runtime_params {
        erase_with_runtime_params(src)?
    } else {
        erase(src, &EraseOptions::default())?
    };
    let out_root = cfg.resolve(cfg.out_dir.to_string_lossy().as_ref());
    let rel = path
        .strip_prefix(&cfg.root)
        .unwrap_or(path)
        .with_extension("sqf");
    let dest = out_root.join(rel);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&dest, erased.text).with_context(|| format!("writing {}", dest.display()))?;
    Ok(dest)
}
