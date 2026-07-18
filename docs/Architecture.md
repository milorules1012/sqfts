# Architecture

High-level map of the SQFts workspace for contributors and curious users.

## Design pipeline

```text
.sqfts source
    │
    ├─► sqfts-syntax: scan annotations ──► erase (byte-stable) ──► plain SQF text
    │                                              │
    │                                              ▼
    │                                    HEMTT preprocessor + parser
    │                                              │
    └─► span map ◄──────────────────────────── diagnostics
                                                   │
    sqfts-db (arma3-wiki)  ──►  sqfts-check ◄── .d.sqfts (sqfts-project)
                                                   │
                                                   ▼
                                          STS diagnostics / build output
```

- **Erasure before preprocess** — preserves E1–E3 identity/locality on unpreprocessed bytes.
- **Check after preprocess** — macros expand; types flow through real SQF ASTs.
- **v1** — annotations must appear literally in source (not only via macro expansion).

## Crates

| Crate | Role |
|---|---|
| `hemtt-sqf` | Vendored HEMTT SQF parser (parser feature) |
| `sqfts-syntax` | Annotation scanner, type parser, eraser |
| `sqfts-db` | Engine-command type database (arma3-wiki) |
| `sqfts-check` | Type checker, assignability, diagnostics |
| `sqfts-project` | `sqfts.toml`, discovery, project session, declgen |
| `sqfts-cli` | `sqfts` binary (`check` / `build` / `declgen`) |
| `sqfts-lsp` | `sqfts-language-server` (tower-lsp) |
| `editors/vscode` | VS Code / Cursor extension |

## Shared session

The CLI and LSP both drive `sqfts-project::Project`: load config, collect sources and declarations, erase, check, report.

## Tests

```bash
cargo test -p sqfts-syntax
cargo test -p sqfts-db -p sqfts-check -p sqfts-project --lib
cargo test -p sqfts-cli -p sqfts-lsp
```

Optional mass identity regression against a real SQF tree:

```powershell
$env:SQFTS_TEST_CORPUS = "C:\path\to\your-mission"
cargo test -p sqfts-cli corpus_identity -- --nocapture
```

## Normative spec

Language rules are defined in the archived [language specification](design-history/language-specification.md). This handbook paraphrases them for wiki use; if a handbook page and the specification disagree, **the specification wins** — except for engine-command **data sourcing**: the live toolchain uses [arma3-wiki](Engine-Command-Database) only (see [design history](design-history/README)).

## Phases (status)

| Phase | Status | Deliverable |
|---|---|---|
| 1 | Done | Engine command DB (arma3-wiki via `sqfts-db`) |
| 2 | Done | [Language design](design-history/language-specification.md) |
| 3 | Done | Compiler toolchain (`check` / `build`) |
| 4 | Done | LSP, editor extension, `declgen` |
