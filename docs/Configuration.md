# Configuration

Project settings live in `sqfts.toml` at the project root (or a path you pass to the CLI).

## Full example

```toml
sources = ["."]
declarations = ["."]
# Optional: HEMTT include roots for #include (relative to project root).
# Omit to auto-add ./include when that directory exists.
# include_paths = ["include"]
out_dir = "out/sqf"
emit_runtime_params = true
build_on_save = true

[flags]
no_implicit_any = false
strict_nil = false
no_position_brands = false
strict_hash_map = false
check_plain_sqf = false

[declgen]
# Optional: strip addon/PBO prefixes from CfgFunctions `file = "..."` paths
# strip_prefixes = ["addon_name/"]
```

## Fields

| Field | Default | Description |
|---|---|---|
| `sources` | `["."]` | Roots (relative to project root) to walk for `.sqfts` (and `.sqf` when `check_plain_sqf`) |
| `declarations` | `[]` | Paths to `.d.sqfts` files or directories containing them |
| `include_paths` | auto | Extra HEMTT include-layer directories. Omitted → add `./include` if present; `[]` → none. Headers there use `#include "\name.h"`. Relative `#include "..\..\macro.h"` resolves from the source file under the project root. |
| `out_dir` | `out/sqf` | Output directory for `sqfts build` |
| `emit_runtime_params` | `true` | Emit native `params` guard arrays from typed entries ([Erasure](Erasure)) |
| `build_on_save` | `true` | Language server erases and writes `.sqf` on each `.sqfts` save |
| `flags.*` | all `false` | Strictness — see [Strictness Flags](Strictness-Flags) |
| `declgen.strip_prefixes` | `[]` | Prefixes stripped from `file = "…"` during [`sqfts declgen`](Declgen) |

## Discovery rules

- Under each `sources` root, the walker collects `.sqfts` files (not `.d.sqfts`).
- With `check_plain_sqf`, sibling `.sqf` files are included.
- The configured `out_dir` name is skipped so build output is not re-checked.
- `.d.sqfts` files are collected from `declarations` **and** from under `sources`.

Missing `sqfts.toml` → defaults (current directory as root).

## Loading

```bash
sqfts check .                    # looks for ./sqfts.toml
sqfts check ./path/to/sqfts.toml # explicit file
sqfts build . --out ./dist       # overrides out_dir
```

## Environment

No environment variables are required for the engine-command database. Commands load from [arma3-wiki](Engine-Command-Database) (git cache with embedded fallback).
