# Adoption Guide

How to introduce SQFts into an existing Arma mission or addon **without** rewriting every file.

## Strategy

SQFts is designed for incremental adoption:

1. Keep shipping plain `.sqf`.
2. Add `.d.sqfts` declarations beside the project.
3. Turn on checking for plain SQF against those declarations.
4. Optionally convert hot paths to `.sqfts` for inline annotations.
5. Run `sqfts build` in CI if you author `.sqfts` sources.

## Step-by-step

### 1. Add a project file

```toml
# sqfts.toml
sources = ["mission.MyMap/core/actions"]
declarations = [".sqfts"]
out_dir = "out/sqf"
emit_runtime_params = false

[flags]
check_plain_sqf = true
```

Start with a **narrow** `sources` root so noise stays manageable.

### 2. Generate declaration skeletons

```bash
sqfts declgen mission.MyMap/Functions.h \
  --root mission.MyMap --tag-default TAG --out .sqfts/mission.d.sqfts

sqfts declgen addon_server/config.cpp \
  --project . \
  --root addon_server --tag-default SERVER --cfg-functions \
  --out .sqfts/server.d.sqfts
```

If `file = "…"` paths include an addon/PBO prefix that is not present under `--root`, set it in `sqfts.toml`:

```toml
[declgen]
strip_prefixes = ["addon_name/"]
```

See [Declgen](Declgen).

### 3. Ensure the engine database is available

`out/commands` from Phase 1, or `SQFTS_COMMANDS_DIR`. See [Engine Command Database](Engine-Command-Database).

### 4. Check

```bash
sqfts check .
```

Expect early noise: argument mismatches (`STS2003`) where declarations are still `any` or inferred incorrectly, and possible preprocess/`#include` issues (`STS1004`) if include roots are not yet wired.

### 5. Tighten declarations

Edit `.d.sqfts` by hand:

```sqfts
declare function TAG_fnc_checkPayment(_unit: object, _amount: number): boolean;
```

Re-check. Fix either the declaration or the call site.

### 6. (Optional) Convert individual files

Rename `fn_foo.sqf` → `fn_foo.sqfts`, add typed `private` / `params`, keep the rest as SQF. Wire `sqfts build` so mission packaging still consumes `.sqf`.

### 7. Raise strictness gradually

Follow the progression in [Strictness Flags](Strictness-Flags): `check_plain_sqf` first, then `strict_nil` / `strict_hash_map`, then `no_implicit_any` for new code.

## Declarations-only example

Without touching sources:

```sqfts
// mission.d.sqfts
declare project_serviceFee: number;
declare function TAG_fnc_checkPayment(_unit: object, _amount: number): boolean;
```

Existing `fn_payFee.sqf`:

```sqf
private _ok = [player, "500"] call TAG_fnc_checkPayment;
//                    ~~~~~ STS2003: expected number
project_serviceFee = true;
// ~~~~~~~~~~~~~~~~ STS2004: boolean not assignable to number
```

## First large check run

On a real mission tree, expect two recurring themes while declarations are still loose: include-path setup for `#include`, and gradual-`any` overload matching edge cases. Both are useful context when triaging the first noisy `sqfts check` pass — tighten high-traffic `*_fnc_*` declarations next so call-site errors become meaningful.
