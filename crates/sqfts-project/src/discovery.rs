//! Source and declaration file discovery.

use std::path::PathBuf;

use anyhow::{bail, Result};
use walkdir::WalkDir;

use crate::config::SqftsConfig;

/// Collect `.sqfts` (and optionally `.sqf`) source files.
pub fn collect_sources(cfg: &SqftsConfig, include_sqf: bool) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let out_dir_name = cfg
        .out_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("out_sqf");
    for root in &cfg.sources {
        let dir = cfg.resolve(root);
        if dir.is_file() {
            out.push(dir);
            continue;
        }
        if !dir.exists() {
            bail!("source path does not exist: {}", dir.display());
        }
        for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            if p.components()
                .any(|c| c.as_os_str().to_str().is_some_and(|s| s == out_dir_name))
            {
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with(".d.sqfts") {
                continue;
            }
            let ext = p.extension().and_then(|e| e.to_str());
            match ext {
                Some("sqfts") => out.push(p.to_path_buf()),
                Some("sqf") if include_sqf => out.push(p.to_path_buf()),
                _ => {}
            }
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// Collect `.d.sqfts` declaration files from config + source roots.
pub fn collect_decls(cfg: &SqftsConfig) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for root in &cfg.declarations {
        let dir = cfg.resolve(root);
        if dir.is_file() {
            out.push(dir);
            continue;
        }
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(".d.sqfts"))
            {
                out.push(p.to_path_buf());
            }
        }
    }
    for root in &cfg.sources {
        let dir = cfg.resolve(root);
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(".d.sqfts"))
            {
                out.push(p.to_path_buf());
            }
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}
