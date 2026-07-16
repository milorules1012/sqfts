use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use comref_extract::run_extract;
use comref_extract::wiki_upstream::emit_wiki_upstream_from_out;

#[derive(Parser)]
#[command(
    name = "comref-extract",
    about = "Extract typed SQF engine-command signatures from COMREF-md"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse COMREF-md and emit command YAML (+ optional wiki diff)
    Extract {
        /// Path to the COMREF-md corpus directory
        #[arg(long)]
        comref: PathBuf,
        /// Output directory for commands/, patches/, report.md
        #[arg(long, default_value = "./out")]
        out: PathBuf,
        /// Cross-check against embedded arma3-wiki and write patches/
        #[arg(long, default_value_t = false)]
        diff_wiki: bool,
    },
    /// Apply high-confidence enrichments to wiki YAML under out/wiki-upstream/
    EmitWikiPatches {
        /// Output directory that already contains patches/ from extract --diff-wiki
        #[arg(long, default_value = "./out")]
        out: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Extract {
            comref,
            out,
            diff_wiki,
        } => match run_extract(&comref, &out, diff_wiki) {
            Ok(result) => {
                println!(
                    "Processed {} engine-command pages: {} ok, {} stub, {} failed ({:.2}% success)",
                    result.stats.total_engine_pages,
                    result.stats.ok,
                    result.stats.stub,
                    result.stats.failed,
                    result.stats.success_rate()
                );
                if let Some(d) = &result.diff {
                    println!(
                        "Wiki diff: {} compared, {} agreements, {} enrichments, {} mismatches",
                        d.compared,
                        d.agreements,
                        d.enrichments.len(),
                        d.mismatches.len()
                    );
                }
                println!("Wrote output to {}", out.display());
                if result.stats.success_rate() < 90.0 {
                    eprintln!(
                        "warning: success rate {:.2}% is below the 90% target",
                        result.stats.success_rate()
                    );
                    ExitCode::from(2)
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::FAILURE
            }
        },
        Commands::EmitWikiPatches { out } => match emit_wiki_upstream_from_out(&out) {
            Ok(result) => {
                println!(
                    "Wiki upstream: {} applied, {} skipped, {} command files → {}/wiki-upstream/",
                    result.applied,
                    result.skipped,
                    result.commands.len(),
                    out.display()
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e:#}");
                ExitCode::FAILURE
            }
        },
    }
}
