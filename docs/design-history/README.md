# SQFts Design History

This directory preserves the major architecture decisions and implementation plans that shaped SQFts. The documents are retained as historical context; current user and contributor documentation lives in the parent [`docs`](..) directory.

Private project names, local filesystem paths, and personal information have been removed from imported planning documents.

## Documents

- [Original project README](original-readme.md) — project status and usage at the end of the initial implementation phases
- [Language specification](language-specification.md) — detailed language design and erasure rules
- [Phase 1 and language design](phase-1-and-language-design.md) — initial project proposal and engine-command database plan
- [Original Phase 3 toolchain plan](phase-3-original-toolchain-plan.md) — superseded parser-fork approach
- [Phase 3 compiler toolchain](phase-3-compiler-toolchain.md) — implemented scanner-and-eraser architecture
- [Phase 4 implementation](phase-4-implementation.md) — language server, editor extension, declaration generator, and wiki integration

The two Phase 3 documents are both retained because they record an important architecture change: the original parser-fork proposal was replaced by a trivia-preserving annotation scanner and byte-stable eraser.
