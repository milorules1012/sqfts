# CLI Reference

The `sqfts` binary provides three subcommands.

```bash
cargo build --release -p sqfts-cli
# → target/release/sqfts
```

## `sqfts check`

Type-check `.sqfts` / `.d.sqfts` (and optionally plain `.sqf`).

```bash
sqfts check [PATH]
```

| Argument | Default | Description |
|---|---|---|
| `PATH` | `.` | Path to `sqfts.toml` or a project root |

Exit codes: `0` if clean, non-zero if errors. Diagnostics print with `STSxxxx` codes.

## `sqfts build`

Erase annotations and emit `.sqf` files.

```bash
sqfts build [PATH] [--out DIR]
```

| Argument / flag | Description |
|---|---|
| `PATH` | Project root or `sqfts.toml` (default `.`) |
| `-o`, `--out DIR` | Output directory (overrides `out_dir` in config) |

Emitted files follow [Erasure](Erasure) rules. Declaration files produce no output.

## `sqfts declgen`

Generate skeleton `.d.sqfts` from `Functions.h` or `CfgFunctions` in `config.cpp`.

```bash
sqfts declgen <CONFIG_FILE> --out <FILE> [options]
```

| Flag | Default | Description |
|---|---|---|
| `--root DIR` | Parent of config file | Resolve `file = "…"` paths |
| `--project PATH` | `.` | Load `[declgen]` options from `sqfts.toml` |
| `--tag-default TAG` | `TAG` | Fallback tag when a group has no `tag = "…"` |
| `--cfg-functions` | off | Only parse inside `class CfgFunctions { … }` |
| `--strip-prefix PREFIX` | from `[declgen]` | Strip a leading addon/PBO prefix from `file = "…"` (repeatable; overrides config) |
| `-o`, `--out FILE` | required | Output `.d.sqfts` path |

Generated lines look like:

```sqfts
declare function TAG_fnc_payFine(_unit: object): any;
```

Param types are inferred from leading `params` guard arrays when the source `.sqf` is found; otherwise `(): any`. Tighten by hand.

Full details: [Declgen](Declgen).

## Related tools

| Tool | Crate | Purpose |
|---|---|---|
| `sqfts-language-server` | `crates/sqfts-lsp` | LSP for editors |

```bash
cargo run -p sqfts-lsp --release
```
