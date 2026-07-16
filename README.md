# SQFts

SQFts is a TypeScript-style, gradually typed superset of Arma 3 SQF. Every valid SQF file remains valid SQFts, while optional annotations, declarations, and type aliases provide compile-time checks before being erased to plain SQF.

## Features

- Gradual typing with `any` for untyped code
- Typed variables, parameters, arrays, tuples, unions, interfaces, and casts
- `.d.sqfts` declaration files for incrementally typing existing projects
- Engine-command overloads sourced from the Arma 3 community wiki data model
- `check`, `build`, and `declgen` CLI commands
- Language server with diagnostics, hover, and completion
- VS Code and Cursor extension support
- Source-span mapping through annotation erasure and preprocessing

## Project status

The core compiler toolchain, language server, editor extension, and declaration generator are implemented. The language remains under active development; see [Future Work](docs/Future-Work.md) for planned additions.

## Quick start

Build and test the workspace:

```bash
cargo build --workspace
cargo test --workspace
```

Check or build a project:

```bash
cargo run -p sqfts-cli -- check path/to/project
cargo run -p sqfts-cli -- build path/to/project --out out/sqf
```

Generate declaration skeletons from `CfgFunctions`:

```bash
cargo run -p sqfts-cli -- declgen path/to/Functions.h \
  --root path/to/project --tag-default TAG --out project.d.sqfts
```

Run the language server:

```bash
cargo run -p sqfts-lsp --release
```

## Configuration

Projects can use an `sqfts.toml` file:

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

[declgen]
# strip_prefixes = ["addon_name/"]
```

## Workspace

- `crates/comref-extract` — engine-command documentation extractor
- `crates/hemtt-sqf` — vendored HEMTT SQF parser
- `crates/sqfts-syntax` — annotation scanner, type parser, and eraser
- `crates/sqfts-db` — engine-command type database
- `crates/sqfts-check` — type checker and diagnostics
- `crates/sqfts-project` — configuration, discovery, project sessions, and declaration generation
- `crates/sqfts-cli` — command-line interface
- `crates/sqfts-lsp` — language server
- `editors/vscode` — VS Code and Cursor extension

## Documentation

- [Handbook](docs/Home.md)
- [Getting Started](docs/Getting-Started.md)
- [Architecture](docs/Architecture.md)
- [CLI Reference](docs/CLI-Reference.md)
- [Editor Support](docs/Editor-Support.md)
- [Language specification and design history](docs/design-history/README.md)

## License

SQFts is licensed under GPL-2.0. The toolchain builds on HEMTT-derived components; provenance is recorded in [`crates/hemtt-sqf/UPSTREAM.md`](crates/hemtt-sqf/UPSTREAM.md).
