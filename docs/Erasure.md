# Erasure

SQFts annotations are compile-time only. The build step rewrites `.sqfts` → `.sqf` by erasing (or locally rewriting) typed constructs.

## Guarantees

| ID | Guarantee |
|---|---|
| **E1 — Identity** | A file containing no SQFts syntax is emitted byte-for-byte identical |
| **E2 — Determinism** | Same input and compiler version → identical output bytes |
| **E3 — Locality** | Only the exact spans of SQFts constructs change; comments, whitespace, line endings, and `#include` directives pass through. Emit operates on **unpreprocessed** source (including annotation text inside `#define` bodies) |
| **E4 — Lint-clean** | Rewrites target HEMTT-friendly forms (e.g. `private _x = v` / `private "_x"`) |

## Per-construct rules

| Construct | Erasure |
|---|---|
| `: Type` in `private` with `=` | Delete `: Type` (and one preceding space if present) |
| `private _x: T;` (no `=`) | Rewrite to `private "_x";` |
| `"_p": T` params entry | Delete `: T` → `"_p"` |
| `"_p": T = expr` | Rewrite entry to `["_p", expr]` |
| `"_p"?: T` | Delete `?: T` → `"_p"` |
| `expr as T` | Delete ` as T` |
| `type` / `interface` / `declare` statements | Delete the whole statement plus one trailing newline |
| `.d.sqfts` file | No output at all |

## Worked example

**Input** (`fn_impoundVehicle.sqfts`):

```sqfts
type feeTier = [number, number];

params [
    "_vehicle": object,
    "_fee": number = 0
];

private _owner: object = _vehicle getVariable ["project_owner", objNull];
if (isNull _owner) exitWith { false };

private _pos = getPosATL _vehicle;
[_owner, _fee] call TAG_fnc_checkPayment
```

**Output** (`fn_impoundVehicle.sqf`):

```sqf
params [
    "_vehicle",
    ["_fee", 0]
];

private _owner = _vehicle getVariable ["project_owner", objNull];
if (isNull _owner) exitWith { false };

private _pos = getPosATL _vehicle;
[_owner, _fee] call TAG_fnc_checkPayment
```

## Optional runtime params lowering

With `emit_runtime_params = true`, typed `params` also emit native SQF guard arrays:

```sqf
params [
    ["_vehicle", objNull, [objNull]],
    ["_fee", 0, [0]]
];
```

### Type → exemplar map

| Type | Exemplar |
|---|---|
| `number` | `0` |
| `string` | `""` |
| `boolean` | `false` |
| `object` | `objNull` |
| `group` | `grpNull` |
| `array` / tuples / brands | `[]` |
| `code` | `{}` |
| `config` | `configNull` |
| `side` | `sideUnknown` |
| `control` | `controlNull` |
| `display` | `displayNull` |
| `task` | `taskNull` |
| `location` | `locationNull` |
| `namespace` | `missionNamespace` |
| unions | one exemplar per member |

Types with no exemplar (`any`, some HashMap targets, control-structure types) omit the guard. Tuples of known length also emit a size list (`[3]` for `[number, number, number]`).

This is the only mode in which types influence emitted runtime behavior.
