# Engine Command Database

SQFts treats the Arma 3 engine command set as a typed standard library. Types come from [arma3-wiki](https://github.com/acemod/arma3-wiki) — the same data source HEMTT uses.

## What is typed

Every **nular**, **unary**, and **binary** command and operator has typed overloads. The checker consults these when you write:

```sqfts
private _pos = getPosATL player;     // positionATL
private _d = player distance _veh;   // number
_veh setDamage 1;                    // nothing
```

User / library functions (`TAG_fnc_*`, `BIS_fnc_*`, …) are **not** in this database — they get types from [`.d.sqfts`](Declaration-Files).

## How the database is loaded

`sqfts-db` loads commands the same way HEMTT does:

1. Try `Wiki::load_git(false)` — refreshes the [arma3-wiki](https://github.com/acemod/arma3-wiki) `dist` branch into the OS app-data cache (at most every ~6 hours unless forced).
2. On failure, fall back to `Wiki::load_dist()` — the snapshot embedded in the `arma3-wiki` crate at build time.

Uses the same `arma3-wiki` **0.4.x** API as HEMTT (`Syntax::params()`). No local YAML extract or `SQFTS_COMMANDS_DIR` is required.

## Overload matching

See [Type Checking](Type-Checking). Summary: first matching overload wins; with `any` arguments, returns may union; with no match and no `any`, `STS2003`.

## Data source

| Source | Role |
|---|---|
| [arma3-wiki](https://github.com/acemod/arma3-wiki) (`dist` + Rust crate) | Canonical typed command DB (also consumed by HEMTT) |

License note: the toolchain is GPL-2.0 (HEMTT-derived). The `arma3-wiki` crate is MIT; respect Bohemia wiki content licensing when redistributing command text.
