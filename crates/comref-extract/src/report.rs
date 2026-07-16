//! Human-readable coverage / agreement report.

use std::fs;
use std::path::Path;

use crate::diff::DiffReport;
use crate::model::ParseOutcome;

#[derive(Debug, Default)]
pub struct CoverageStats {
    pub total_engine_pages: usize,
    pub ok: usize,
    pub stub: usize,
    pub failed: usize,
    pub skipped: usize,
    pub failures: Vec<(String, String)>,
    pub stubs: Vec<(String, String)>,
}

impl CoverageStats {
    #[must_use]
    pub fn from_outcomes(outcomes: &[ParseOutcome]) -> Self {
        let mut stats = Self {
            total_engine_pages: outcomes.len(),
            ..Self::default()
        };
        for o in outcomes {
            match o {
                ParseOutcome::Ok(_) => stats.ok += 1,
                ParseOutcome::Stub { name, reason } => {
                    stats.stub += 1;
                    stats.stubs.push((name.clone(), reason.clone()));
                }
                ParseOutcome::Failed { name, reason } => {
                    stats.failed += 1;
                    stats.failures.push((name.clone(), reason.clone()));
                }
                ParseOutcome::Skipped { .. } => stats.skipped += 1,
            }
        }
        stats.failures.sort_by(|a, b| a.0.cmp(&b.0));
        stats.stubs.sort_by(|a, b| a.0.cmp(&b.0));
        stats
    }

    #[must_use]
    pub fn success_rate(&self) -> f64 {
        if self.total_engine_pages == 0 {
            return 0.0;
        }
        // Treat stubs as non-success for the denominator of "complete signature"
        (self.ok as f64) / (self.total_engine_pages as f64) * 100.0
    }
}

pub fn write_report(
    stats: &CoverageStats,
    diff: Option<&DiffReport>,
    out_dir: &Path,
) -> anyhow::Result<()> {
    let mut md = String::new();
    md.push_str("# COMREF extract report\n\n");
    md.push_str(&format!(
        "- Engine-command pages processed: **{}**\n",
        stats.total_engine_pages
    ));
    md.push_str(&format!("- Parsed OK: **{}**\n", stats.ok));
    md.push_str(&format!("- Stubs: **{}**\n", stats.stub));
    md.push_str(&format!("- Failed: **{}**\n", stats.failed));
    md.push_str(&format!("- Skipped: **{}**\n", stats.skipped));
    md.push_str(&format!(
        "- Success rate: **{:.2}%** (target ≥ 90%)\n\n",
        stats.success_rate()
    ));

    if let Some(d) = diff {
        md.push_str("## arma3-wiki agreement\n\n");
        md.push_str(&format!("- Wiki source: `{}`\n", d.wiki_source));
        md.push_str(&format!("- Commands compared: **{}**\n", d.compared));
        md.push_str(&format!("- Full agreements: **{}**\n", d.agreements));
        md.push_str(&format!(
            "- Agreement rate: **{:.2}%**\n",
            if d.compared == 0 {
                0.0
            } else {
                d.agreements as f64 / d.compared as f64 * 100.0
            }
        ));
        md.push_str(&format!(
            "- Enrichments (wiki Unknown / extra overload): **{}**\n",
            d.enrichments.len()
        ));
        md.push_str(&format!("- Type mismatches: **{}**\n", d.mismatches.len()));
        md.push_str(&format!(
            "- Present in COMREF but missing in wiki: **{}**\n",
            d.missing_in_wiki.len()
        ));
        md.push_str(&format!(
            "- Present in wiki but missing in COMREF extract: **{}**\n\n",
            d.missing_in_comref.len()
        ));

        if !d.enrichments.is_empty() {
            md.push_str("### Enrichment samples (first 50)\n\n");
            for e in d.enrichments.iter().take(50) {
                md.push_str(&format!(
                    "- `{}` @ `{}`: COMREF `{}` — {}\n",
                    e.command, e.location, e.comref_type, e.note
                ));
            }
            md.push('\n');
        }

        if !d.mismatches.is_empty() {
            md.push_str("### Mismatch samples (first 50)\n\n");
            for m in d.mismatches.iter().take(50) {
                md.push_str(&format!(
                    "- `{}` @ `{}`: COMREF `{}` vs wiki `{}`\n",
                    m.command, m.location, m.comref, m.wiki
                ));
            }
            md.push('\n');
        }
    }

    if !stats.failures.is_empty() {
        md.push_str("## Failures\n\n");
        for (name, reason) in &stats.failures {
            md.push_str(&format!("- `{name}`: {reason}\n"));
        }
        md.push('\n');
    }

    if !stats.stubs.is_empty() {
        md.push_str("## Stubs\n\n");
        for (name, reason) in &stats.stubs {
            md.push_str(&format!("- `{name}`: {reason}\n"));
        }
        md.push('\n');
    }

    fs::create_dir_all(out_dir)?;
    fs::write(out_dir.join("report.md"), md)?;
    Ok(())
}
