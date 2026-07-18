# Getting Started

This page gets you from a clone of SQFts to a type-checked project.

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- Network access on first build (the `arma3-wiki` crate embeds command data at compile time)

Engine commands load automatically from [arma3-wiki](https://github.com/acemod/arma3-wiki) (same crate/path as HEMTT) at runtime — no separate extract step. See [Engine Command Database](Engine-Command-Database).

## Build the CLI

From the repository root:

```bash
cargo build --release -p sqfts-cli
```

The binary is at `target/release/sqfts` (or `sqfts.exe` on Windows). Put it on your `PATH`, or invoke it via `cargo run -p sqfts-cli -- …`.

## Minimal project

Create a folder with:

**`sqfts.toml`**

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

**`hello.sqfts`**

```sqfts
private _n: number = 1 + 2;
hint str _n;
```

**`lib.d.sqfts`** (optional ambient types)

```sqfts
declare function TAG_fnc_example(_x: number): boolean;
```

## Check

```bash
sqfts check .
```

Exit code `0` means no errors. Diagnostics use stable codes such as `STS2003` — see [Diagnostics](Diagnostics).

## Build (erase to SQF)

```bash
sqfts build . --out ./out/sqf
```

Annotated `.sqfts` files are erased to plain `.sqf` under `out_dir`. Plain SQF identity is preserved: files with no SQFts syntax are emitted byte-for-byte identical.

## Generate declarations for an existing mission

```bash
sqfts declgen path/to/Functions.h \
  --root path/to/mission \
  --tag-default TAG \
  --out mission.d.sqfts
```

For server `config.cpp` / `CfgFunctions`:

```bash
sqfts declgen path/to/config.cpp \
  --project . \
  --root path/to/addon_server \
  --tag-default SERVER \
  --cfg-functions \
  --out server.d.sqfts
```

Configure addon path prefixes in `sqfts.toml` when needed:

```toml
[declgen]
strip_prefixes = ["addon_name/"]
```

See [Declgen](Declgen) and [Adoption Guide](Adoption-Guide).

## Editor

For live diagnostics in VS Code or Cursor, see [Editor Support](Editor-Support).

## Next steps

- [Basic Concepts](Basic-Concepts) — how SQFts relates to SQF
- [Everyday Types](Everyday-Types) — the type vocabulary
- [Configuration](Configuration) — all `sqfts.toml` options
