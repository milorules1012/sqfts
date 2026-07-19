//! Shared project session for CLI and LSP.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use sqfts_check::{check_source_with, load_declarations, CheckResult, DeclarationSet};
use sqfts_db::CommandDb;

use crate::config::SqftsConfig;
use crate::discovery::collect_decls;

/// Loaded project: config + command DB + declarations.
#[derive(Debug, Clone)]
pub struct Project {
    pub config: SqftsConfig,
    pub db: Arc<CommandDb>,
    pub decls: DeclarationSet,
}

impl Project {
    /// Load config from `path`, command DB, and all declaration files.
    pub fn load(path: &str) -> Result<Self> {
        let config = SqftsConfig::load(path)?;
        Self::from_config(config)
    }

    /// Build a project from an already-loaded config.
    pub fn from_config(config: SqftsConfig) -> Result<Self> {
        let db = Arc::new(CommandDb::load_default().context("loading command database")?);
        let decl_paths = collect_decls(&config)?;
        let decls = load_declarations(&decl_paths).context("loading declarations")?;
        Ok(Self { config, db, decls })
    }

    /// Reload declaration files (e.g. after a `.d.sqfts` save).
    pub fn reload_declarations(&mut self) -> Result<()> {
        let decl_paths = collect_decls(&self.config)?;
        self.decls = load_declarations(&decl_paths).context("reloading declarations")?;
        Ok(())
    }

    /// Reload `sqfts.toml` and declarations from the project root.
    pub fn reload_config(&mut self) -> Result<()> {
        let root = self.config.root.clone();
        self.config = SqftsConfig::load_path(&root)?;
        self.reload_declarations()
    }

    /// Type-check a single file's source text.
    pub fn check_file(&self, path: &Path, source: &str) -> Result<CheckResult> {
        let workspace = sqfts_check::CheckWorkspace {
            source_path: Some(path.to_path_buf()),
            project_root: Some(self.config.root.clone()),
            include_paths: self.config.resolved_include_paths(),
        };
        check_source_with(
            source,
            path.to_string_lossy().as_ref(),
            &self.db,
            &self.decls,
            &self.config.flags,
            &workspace,
        )
        .map_err(Into::into)
    }

    /// Project root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.config.root
    }
}
