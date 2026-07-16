//! Extract typed SQF engine-command signatures from a COMREF-md corpus.

pub mod corpus;
pub mod diff;
pub mod emit;
pub mod model;
pub mod parse;
pub mod report;
pub mod types;
pub mod wiki_upstream;

use std::path::Path;

use crate::corpus::parse_corpus;
use crate::diff::{diff_against_wiki, emit_patches};
use crate::emit::emit_commands;
use crate::model::{ExtractedCommand, ParseOutcome};
use crate::report::{write_report, CoverageStats};

#[derive(Debug)]
pub struct ExtractResult {
    pub stats: CoverageStats,
    pub commands: Vec<ExtractedCommand>,
    pub diff: Option<diff::DiffReport>,
}

/// Run a full extraction pipeline.
pub fn run_extract(comref_dir: &Path, out_dir: &Path, do_diff: bool) -> anyhow::Result<ExtractResult> {
    let outcomes = parse_corpus(comref_dir)?;
    let stats = CoverageStats::from_outcomes(&outcomes);

    let commands: Vec<ExtractedCommand> = outcomes
        .iter()
        .filter_map(|o| match o {
            ParseOutcome::Ok(cmd) => Some(cmd.clone()),
            _ => None,
        })
        .collect();

    emit_commands(&commands, out_dir)?;

    let diff = if do_diff {
        let cache = out_dir.join(".wiki-cache");
        let report = diff_against_wiki(&commands, &cache)?;
        emit_patches(&report, out_dir)?;
        Some(report)
    } else {
        None
    };

    write_report(&stats, diff.as_ref(), out_dir)?;

    Ok(ExtractResult {
        stats,
        commands,
        diff,
    })
}
