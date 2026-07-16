//! Check / build command implementations (CLI reporting).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use codespan_reporting::diagnostic::{Diagnostic as CsDiag, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{
    self,
    termcolor::{ColorChoice, StandardStream},
};

use sqfts_check::CheckResult;
use sqfts_project::{collect_sources, emit_file, Project, SqftsConfig};

/// Run `sqfts check`. Returns process exit code.
pub fn run_check(cfg: &SqftsConfig) -> Result<i32> {
    let project = Project::from_config(cfg.clone())?;
    let mut files = SimpleFiles::new();
    let mut had_error = false;

    for path in collect_sources(cfg, cfg.flags.check_plain_sqf)? {
        let src = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let result = project.check_file(&path, &src)?;
        if emit_diagnostics(&mut files, &path, &src, &result)? {
            had_error = true;
        }
    }
    Ok(if had_error { 1 } else { 0 })
}

/// Run `sqfts build` — erase annotations to `.sqf`.
pub fn run_build(cfg: &SqftsConfig) -> Result<()> {
    let out_root = cfg.resolve(cfg.out_dir.to_string_lossy().as_ref());
    std::fs::create_dir_all(&out_root)
        .with_context(|| format!("creating {}", out_root.display()))?;

    let sources = collect_sources(cfg, false)?;
    for path in sources {
        if path.extension().and_then(|e| e.to_str()) != Some("sqfts") {
            continue;
        }
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".d.sqfts"))
        {
            continue;
        }
        let src = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let dest = emit_file(cfg, &path, &src)?;
        eprintln!("wrote {}", dest.display());
    }
    Ok(())
}

fn emit_diagnostics(
    files: &mut SimpleFiles<String, String>,
    path: &Path,
    src: &str,
    result: &CheckResult,
) -> Result<bool> {
    let file_id = files.add(path.display().to_string(), src.to_string());
    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = term::Config::default();
    let mut had_error = false;
    for d in &result.diagnostics {
        if d.severity == sqfts_check::Severity::Error {
            had_error = true;
        }
        let severity = match d.severity {
            sqfts_check::Severity::Error => codespan_reporting::diagnostic::Severity::Error,
            sqfts_check::Severity::Warning => codespan_reporting::diagnostic::Severity::Warning,
            sqfts_check::Severity::Note => codespan_reporting::diagnostic::Severity::Note,
        };
        let mut cs = CsDiag::new(severity)
            .with_code(d.code.as_str())
            .with_message(&d.message);
        if let Some(span) = &d.span {
            cs = cs.with_labels(vec![Label::primary(file_id, span.start..span.end)]);
        }
        for (msg, span) in &d.related {
            cs = cs.with_labels(vec![
                Label::secondary(file_id, span.start..span.end).with_message(msg),
            ]);
        }
        term::emit(&mut writer.lock(), &config, files, &cs)?;
    }
    Ok(had_error)
}

/// Re-export for callers that still want PathBuf helpers.
#[allow(dead_code)]
pub type SourceList = Vec<PathBuf>;
