# SQFts Design History

This directory preserves the major architecture decisions and implementation plans that shaped SQFts. The documents are retained as historical context; current user and contributor documentation lives in the parent [`docs`](..) directory.

Private project names, local filesystem paths, and personal information have been removed from imported planning documents.

## Current vs historical (engine commands)

**Current:** [`sqfts-db`](../../crates/sqfts-db) loads engine commands from [arma3-wiki](https://github.com/acemod/arma3-wiki) the same way HEMTT does (`Wiki::load_git` → `Wiki::load_dist`). There is no COMREF extract step and no `out/commands` YAML database.

**Historical:** Phase 1 originally planned (and temporarily shipped) a `comref-extract` crate that scraped Bohemia COMREF-md and emitted wiki-shaped YAML, with diffs/patches for upstreaming. That pipeline was removed as unreliable; older plans and the archived language specification still mention it.

## Documents

- [Original project README](original-readme.md) — project status and usage at the end of the initial implementation phases
- [Language specification](language-specification.md) — detailed language design and erasure rules (still normative for the type system; engine-DB sourcing descriptions are historical)
- [Phase 1 and language design](phase-1-and-language-design.md) — initial project proposal and COMREF-based engine-command database plan (superseded for data sourcing)
- [Original Phase 3 toolchain plan](phase-3-original-toolchain-plan.md) — superseded parser-fork approach
- [Phase 3 compiler toolchain](phase-3-compiler-toolchain.md) — implemented scanner-and-eraser architecture
- [Phase 4 implementation](phase-4-implementation.md) — language server, editor extension, declaration generator, and wiki-upstream tooling (wiki-upstream via COMREF is obsolete)

The two Phase 3 documents are both retained because they record an important architecture change: the original parser-fork proposal was replaced by a trivia-preserving annotation scanner and byte-stable eraser.
