# SQFts

A TypeScript-style gradually-typed **superset of Arma 3 SQF**.

Every valid SQF file is a valid SQFts file. Optional type annotations, function
signature declarations, and typedefs are checked then **erased** by a transpiler
that emits plain SQF.

## Status

**Phase 1 (done):** `comref-extract` — parse the Bohemia COMREF markdown corpus and
emit an engine-command type database in arma3-wiki YAML form, then cross-check
against the upstream [arma3-wiki](https://github.com/acemod/arma3-wiki) `dist`
branch. 2,655/2,685 engine pages parsed (98.9%), 98.2% signature agreement.

**Phase 2 (done):** language design — see the [language specification](language-specification.md).

**Phase 3 (done):** compiler toolchain — `sqfts check` / `sqfts build` CLI,
annotation scanner + byte-stable eraser, type checker against the Phase 1
command database and `.d.sqfts` declaration files.

**Phase 4 (done):** LSP (`sqfts-language-server`), VS Code/Cursor extension
([`editors/vscode`](../../editors/vscode)), `sqfts declgen` for CfgFunctions →
`.d.sqfts`, and `comref-extract emit-wiki-patches` for arma3-wiki upstream PRs.

## Workspace

| Crate | Role |
|---|---|
| [`crates/comref-extract`](../../crates/comref-extract) | COMREF → typed command YAML extractor |
| [`crates/hemtt-sqf`](../../crates/hemtt-sqf) | Vendored HEMTT SQF parser (parser feature) |
| [`crates/sqfts-syntax`](../../crates/sqfts-syntax) | Annotation scanner, type parser, eraser |
| [`crates/sqfts-db`](../../crates/sqfts-db) | Engine-command type database |
| [`crates/sqfts-check`](../../crates/sqfts-check) | Type checker |
| [`crates/sqfts-project`](../../crates/sqfts-project) | Config, discovery, project session, declgen |
| [`crates/sqfts-cli`](../../crates/sqfts-cli) | `sqfts` binary (`check` / `build` / `declgen`) |
| [`crates/sqfts-lsp`](../../crates/sqfts-lsp) | `sqfts-language-server` (tower-lsp) |
| [`editors/vscode`](../../editors/vscode) | VS Code / Cursor extension |

## Quick start

```bash
# Extract engine-command signatures (Phase 1)
cargo run -p comref-extract --release -- extract \
  --comref "../COMREF-md" \
  --out ./out --diff-wiki

# Prepare arma3-wiki upstream patch files
cargo run -p comref-extract -- emit-wiki-patches --out ./out

# Type-check a project (requires out/commands from Phase 1)
cargo run -p sqfts-cli -- check ./path/to/project

# Erase annotations → .sqf
cargo run -p sqfts-cli -- build ./path/to/project --out ./out/sqf

# Generate skeleton declarations from Functions.h / CfgFunctions
cargo run -p sqfts-cli -- declgen path/to/Functions.h \
  --root path/to/mission --tag-default TAG --out mission.d.sqfts

# Language server
cargo run -p sqfts-lsp --release

# Tests
cargo test -p sqfts-syntax
cargo test -p sqfts-db -p sqfts-check -p sqfts-project --lib
cargo test -p sqfts-cli -p sqfts-lsp
```

Optional mass identity regression against a real SQF tree:

```bash
# PowerShell
$env:SQFTS_TEST_CORPUS = "C:\path\to\your-mission"
cargo test -p sqfts-cli corpus_identity -- --nocapture
```

### `sqfts.toml`

```toml
sources = ["."]
declarations = ["."]
out_dir = "out/sqf"
emit_runtime_params = false

[flags]
no_implicit_any = false
strict_nil = false
no_position_brands = false
strict_hash_map = false
check_plain_sqf = false
```

## Architecture

Annotations are scanned and erased **before** preprocessing (byte-stable spans).
Checking runs on the erased plain SQF through the HEMTT preprocessor + parser;
diagnostics map back to original `.sqfts` locations. Engine overloads come from
Phase 1 `out/commands/*.yml`. The LSP and CLI share `sqfts-project::Project`.

**v1 restriction:** annotations must appear literally in source (not produced by
macro expansion). See SPEC §6 / Phase 3 notes.

## Scope

- Engine commands and operators only (`BIS_fnc_*` / `BIN_fnc_*` typed via `.d.sqfts`).
- License: GPL-2.0 (HEMTT-derived toolchain).

## Data source

The COMREF-md corpus is a scraped Bohemia Interactive community wiki export.
[`arma3-wiki`](https://github.com/acemod/arma3-wiki) is the canonical typed
database consumed by HEMTT; `comref-extract` verifies and enriches it. The
checker loads Phase 1 YAML by default (`out/commands`, or `SQFTS_COMMANDS_DIR`).
