//! SQFts CLI — `sqfts check` / `sqfts build` / `sqfts declgen`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use sqfts_project::{generate_declarations, DeclgenOptions, SqftsConfig};

mod report;

#[derive(Parser, Debug)]
#[command(name = "sqfts", version, about = "Gradually-typed SQF toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Type-check .sqfts / .d.sqfts (and optionally plain .sqf) files.
    Check {
        /// Path to sqfts.toml or a project root (default: cwd).
        #[arg(default_value = ".")]
        path: String,
    },
    /// Erase annotations and emit .sqf files.
    Build {
        /// Path to sqfts.toml or a project root (default: cwd).
        #[arg(default_value = ".")]
        path: String,
        /// Output directory (overrides sqfts.toml).
        #[arg(long, short)]
        out: Option<String>,
    },
    /// Generate skeleton `.d.sqfts` from Functions.h / CfgFunctions.
    Declgen {
        /// Path to Functions.h or config.cpp.
        config_file: PathBuf,
        /// Directory used to resolve `file = "..."` paths.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Path to `sqfts.toml` or project root (loads `[declgen]` options).
        #[arg(long, default_value = ".")]
        project: String,
        /// Fallback / default tag (e.g. TAG).
        #[arg(long, default_value = "TAG")]
        tag_default: String,
        /// Only parse `class CfgFunctions { ... }` (for config.cpp).
        #[arg(long, default_value_t = false)]
        cfg_functions: bool,
        /// Strip a leading path prefix from `file = "..."` before resolving under
        /// `--root`. Repeatable. Overrides `[declgen].strip_prefixes` from
        /// `sqfts.toml` when any value is passed.
        #[arg(long = "strip-prefix", value_name = "PREFIX")]
        strip_prefixes: Vec<String>,
        /// Output `.d.sqfts` path.
        #[arg(long, short)]
        out: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { path } => {
            let cfg = SqftsConfig::load(&path)?;
            let code = report::run_check(&cfg)?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::Build { path, out } => {
            let mut cfg = SqftsConfig::load(&path)?;
            if let Some(o) = out {
                cfg.out_dir = o.into();
            }
            report::run_build(&cfg)?;
        }
        Commands::Declgen {
            config_file,
            root,
            project,
            tag_default,
            cfg_functions,
            strip_prefixes,
            out,
        } => {
            let project_cfg = SqftsConfig::load(&project)?;
            let resolve_root = root.unwrap_or_else(|| {
                config_file
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
            });
            let strip_prefixes = if strip_prefixes.is_empty() {
                project_cfg.declgen.strip_prefixes
            } else {
                strip_prefixes
            };
            let (text, decls) = generate_declarations(&DeclgenOptions {
                config_file: config_file.clone(),
                resolve_root,
                tag_default,
                cfg_functions_only: cfg_functions,
                strip_prefixes,
            })?;
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            std::fs::write(&out, &text).with_context(|| format!("writing {}", out.display()))?;
            eprintln!(
                "wrote {} ({} declarations) from {}",
                out.display(),
                decls.len(),
                config_file.display()
            );
        }
    }
    Ok(())
}
