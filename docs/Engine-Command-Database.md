# Engine Command Database

SQFts treats the Arma 3 engine command set as a typed standard library. Types come from Phase 1 extraction, verified against [arma3-wiki](https://github.com/acemod/arma3-wiki).

## What is typed

Every **nular**, **unary**, and **binary** command and operator has typed overloads. The checker consults these when you write:

```sqfts
private _pos = getPosATL player;     // positionATL
private _d = player distance _veh;   // number
_veh setDamage 1;                    // nothing
```

User / library functions (`TAG_fnc_*`, `BIS_fnc_*`, …) are **not** in this database — they get types from [`.d.sqfts`](Declaration-Files).

## Building the database

```bash
cargo run -p comref-extract --release -- extract \
  --comref "../COMREF-md" \
  --out ./out --diff-wiki
```

Output lives under `out/commands/*.yml` (one page per command). Cross-check stats against arma3-wiki’s `dist` branch are printed when `--diff-wiki` is set.

### Upstream patches

```bash
cargo run -p comref-extract -- emit-wiki-patches --out ./out
```

Prepares patch material for arma3-wiki contributions where COMREF and wiki disagree.

## How the checker loads it

By default the toolchain looks for `out/commands` relative to the working tree. Override with:

```bash
# PowerShell
$env:SQFTS_COMMANDS_DIR = "C:\path\to\out\commands"
```

Without a database, engine call checking cannot resolve overloads (unknown commands surface as `STS2401` / degraded typing).

## Overload matching

See [Type Checking](Type-Checking). Summary: first matching overload wins; with `any` arguments, returns may union; with no match and no `any`, `STS2003`.

## Data sources

| Source | Role |
|---|---|
| COMREF markdown corpus | Scraped Bohemia community wiki export — input to `comref-extract` |
| arma3-wiki | Canonical typed DB also consumed by HEMTT; verification target |

License note: the toolchain is GPL-2.0 (HEMTT-derived). Respect upstream wiki licensing when redistributing extracted YAML.
