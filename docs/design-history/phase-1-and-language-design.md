---
name: Typed SQF Superset
overview: Build "SQFts", a TypeScript-style gradually-typed superset of SQF in Rust on forked HEMTT infrastructure, with v1 delivering an engine-command type database auto-generated from the COMREF-md corpus in arma3-wiki YAML format.
todos:
  - id: scaffold
    content: "Create sqfts repo: Cargo workspace, comref-extract crate skeleton, README, arma3-wiki crate dependency"
    status: completed
  - id: template-parser
    content: "Implement COMREF template parser: sections, syntax blocks, param lines, return values, versions"
    status: completed
  - id: type-normalizer
    content: Implement type vocabulary normalization onto canonical enum aligned with arma3-wiki model
    status: completed
  - id: yaml-emit
    content: Emit arma3-wiki-format YAML for engine commands and operators (skip BIS/BIN pages)
    status: completed
  - id: hard-cases
    content: "Handle hard cases: %-encoded dupes, HTML tables in params, bold syntax, prose union types, stubs"
    status: completed
  - id: validation
    content: Golden-file tests on ~20 hand-checked commands plus engine-command coverage report (target >=90%)
    status: completed
  - id: diff-wiki
    content: Cross-check extracted engine-command signatures against arma3-wiki dist and produce patch report
    status: completed
  - id: spec
    content: "Phase 2: write the language specification — type system, annotation syntax, declaration files, erasure rules"
    status: completed
isProject: false
---

# Typed SQF Superset (working name: SQFts)

## Goal and model

A TypeScript-analog for SQF: every valid SQF file is a valid SQFts file (new extension, e.g. `.sqfts`); optional type annotations, function signature declarations, and typedefs are checked then **erased** by a transpiler that emits plain SQF. Implementation in Rust, building on HEMTT's crates.

New Cargo workspace for SQFts, using `COMREF-md` as a read-only data source.

## Key research findings that shape the design

- [`arma3-wiki` crate](https://crates.io/crates/arma3-wiki) (MIT, crates.io v0.5.5) already provides ~2,693 engine commands as typed YAML (per-overload call shape, param types, optionality, defaults, since-versions, return types). **Do not re-derive engine commands.**
- `hemtt-sqf` (in [BrettMayson/HEMTT](https://github.com/BrettMayson/HEMTT), GPL-2.0, unpublished) has a chumsky parser with spanned AST, preprocessor with source-mapping through macros, and an existing set-based type-inference engine ("inspector") that validates args against arma3-wiki signatures. It has no annotation syntax and no cross-file user-function types — exactly the gap SQFts fills. Using it requires a **fork** and makes the toolchain GPL-2.0.
- COMREF-md corpus (4,769 files): ~2,663 engine commands plus 27 operator pages (with %-encoded duplicates), 2,077 `BIS_fnc_*`/`BIN_fnc_*` pages, 2 meta reports. One rigid template; ~3–5% of engine-command pages need special handling (HTML tables in params, bold syntax lines, prose union types). Nearly all stub pages are BIS/BIN, so the engine-command subset is high quality.
- **Scope decision**: `BIS_fnc_*`/`BIN_fnc_*` pages are excluded — they are library functions any mission/addon author can define, not part of the language. The type database covers engine commands and operators only; user/library functions get types later via `.d.sqfts` declaration files (Phase 2).

## Phase 1 — v1 deliverable: engine-command type database (`comref-extract`)

arma3-wiki is the base database (MIT, already consumed by `hemtt-sqf`). The COMREF corpus's job is to **verify and enrich it**: fill params typed `Unknown`, catch missing overloads/params, and confirm the type vocabulary the language will standardize on. A Rust CLI crate parses the ~2,690 engine-command/operator pages (BIS/BIN pages skipped by filename) and emits **arma3-wiki-format YAML**.

- **Parser for the wiki-export template**: version header line, `### Syntax` / `### Alternative Syntax` / `### Syntax 1..6` blocks, `Syntax:` / `Parameters:` / `Return Value:` labels, param lines (`name: Type - description`), `(Optional, default X)`, per-param `since  N.NN` lines, per-syntax bare version lines, `Array of X` / `X or Y` / `Array format PositionAGL` type idioms, escaped-bracket syntax lines, `Groups:` splitting via a known-group dictionary.
- **Normalization**: dedupe %-encoded filenames; map the prose type vocabulary (~35 names: Number, Object, String, Array, Boolean, Code, Side, Group, Config, Control, Display, Task, Location, Namespace, HashMap, Team Member, Script Handle, position formats, etc.) onto a canonical type enum aligned with arma3-wiki's model. This canonical enum becomes the primitive type set of the language (Phase 2).
- **Outputs**:
  - `commands/*.yml` — extracted signatures for every engine command and operator.
  - `patches/*.yml` — structured diffs against arma3-wiki dist: params where COMREF has a concrete type but arma3-wiki says `Unknown`, overloads/params present in one source but not the other (candidate upstream PRs).
  - `report.md` — coverage stats: parsed clean / failed, per-file failure reasons, and the agreement rate with arma3-wiki.
- **Quality gate**: golden-file tests on ~20 hand-checked commands (setDamage, getPos, addAction, forEach, an operator, a control structure); target ≥90% of engine-command pages producing a complete signature, with every failure listed in the report rather than silently dropped.
- Subagents: parallelize hard-case handling (HTML-table params, prose types) and validation sweeps across corpus slices.

## Phase 2 — language design (spec document, no code)

A language specification defining, with before/after examples:

- **Type system**: gradual (`any`-equivalent = SQF's Anything), the canonical primitive set from Phase 1, unions (`string | code`), typed arrays (`number[]`, tuples like `[number, number, number]` for positions), `nil`/Nothing, position subtypes as branded array types.
- **Annotation syntax** (all erasable): typed `params ["_x": number, "_y": object = objNull]`, `private _n: number = 0`, function signature declarations for `TAG_fnc_*`-style user functions, `type`/`interface` aliases, cast/assert syntax.
- **Declaration files** (`.d.sqfts`) so existing plain-SQF codebases can be typed incrementally without touching source.
- **Erasure rules**: transpiled output must be byte-stable, readable SQF that passes HEMTT lints.

## Phase 3 — compiler toolchain (Rust workspace)

- Fork `hemtt-sqf` (keep AST a strict superset so upstream inspector/lint improvements stay mergeable); take `hemtt-preprocessor`/`hemtt-workspace` as git deps. Accept GPL-2.0 for the project.
- Extend lexer/grammar with annotation tokens; parse `.sqfts` and `.d.sqfts`.
- **Checker**: build on the inspector's set-based `GameValue` engine, adding declared types as constraints, cross-file user-function signatures (loaded from Phase 1 YAML + declaration files), and TS-style diagnostics with spans through macros.
- **Emitter**: type erasure to `.sqf`.
- `sqfts check` / `sqfts build` CLI with codespan diagnostics.

## Phase 4 — adoption and editor support (later)

- LSP server following HEMTT's `hls` pattern (tower-lsp; hover types, inline errors), VS Code/Cursor extension.
- Pilot: generate `.d.sqfts` declarations for a private downstream codebase and type-check one subsystem end-to-end.
- Upstream arma3-wiki patches from Phase 1.

## Execution notes

- Phase 1 is self-contained and is the concrete build target now; Phases 2–4 proceed after v1 review.
- Only Phase 1 + repo scaffold are in scope for the initial implementation todos below.