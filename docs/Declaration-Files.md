# Declaration Files

A `.d.sqfts` file contains only `declare`, `type`, and `interface` statements (plus comments). It produces **no output** and exists so plain-SQF codebases can be typed incrementally without touching sources.

## Example

```sqfts
// mission.d.sqfts — hand-written or generated
type moneyTarget = object | group;

declare project_playerCash: number;
declare project_serverReady: boolean;

declare function TAG_fnc_checkPayment(_unit: object, _amount: number): boolean;
declare function TAG_fnc_weaponShopCfg(_shop: string): array;
declare function SERVER_fnc_updateVehicleMods(
    _vehicle: object,
    _insurance: boolean,
    _unused: array,
    _mods: string[]
): nothing;
```

## Resolution

1. The compiler loads every `.d.sqfts` reachable from [`sqfts.toml`](Configuration) (`declarations` paths, plus any under `sources`).
2. Duplicate declarations for the same symbol must be identical (`STS1002`).
3. A declaration whose symbol is also *defined* in a checked `.sqfts` file is verified against the definition.

## Adoption path

Typical workflow for a large mission:

1. Generate skeletons with [`sqfts declgen`](Declgen) (`(): any` or inferred from `params` guards).
2. Enable [`check_plain_sqf`](Strictness-Flags) and fix call-site mismatches against declarations.
3. Tighten param/return types in `.d.sqfts` by hand.
4. Optionally rename hot files to `.sqfts` and add inline annotations.

See [Adoption Guide](Adoption-Guide).

## What may appear

| Allowed | Forbidden |
|---|---|
| `declare …` | Executable SQF statements |
| `type …` | `private` / assignments |
| `interface …` | Anything that would emit code |
| Comments | |

Malformed declaration files report `STS1003`.
