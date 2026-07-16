# SQFts Language Specification (Phase 2)

SQFts is a gradually-typed superset of Arma 3 SQF, in the same relationship to
SQF as TypeScript is to JavaScript:

1. **Superset** — every valid SQF file is a valid SQFts file with identical
   meaning. No existing script needs changes to compile.
2. **Erasable** — all SQFts-only syntax is checked at compile time and then
   erased. The emitted `.sqf` contains no runtime trace of the type system
   (with one explicit opt-in exception, see [§7.4](#74-optional-runtime-params-lowering)).
3. **Gradual** — untyped code means `any`, not an error. Types can be added
   file by file, or supplied externally via `.d.sqfts` declaration files
   without touching existing sources.

File extensions:

| Extension | Contents |
|---|---|
| `.sqfts` | Source: SQF plus optional annotations. Compiles to `.sqf`. |
| `.d.sqfts` | Declarations only (`declare` / `type` / `interface` statements). Never emits output. |
| `.sqf` | Plain SQF. Consumed as-is; treated as fully `any` unless covered by declarations. |

The engine-command type database produced in Phase 1 (`comref-extract` YAML,
verified against [arma3-wiki](https://github.com/acemod/arma3-wiki)) is the
built-in "standard library": every nular, unary, and binary command and
operator has typed overloads that the checker consults. User and library
functions (`TAG_fnc_*`, `BIS_fnc_*`, …) get types from declarations.

---

## 1. The type system

### 1.1 Primitive types

Surface type names are lowerCamelCase keywords. Each maps 1:1 onto the
canonical `SqfType` enum from Phase 1 (which is itself aligned with
arma3-wiki's value model):

| SQFts type | Phase 1 / wiki type | Notes |
|---|---|---|
| `any` | Anything | The gradual type; assignable to and from everything. |
| `nothing` | Nothing | Type of `nil` and of statements/commands with no value. |
| `number` | Number | |
| `string` | String | |
| `boolean` | Boolean | |
| `array` | Array | Shorthand for `any[]`. |
| `code` | Code | |
| `object` | Object | |
| `group` | Group | |
| `side` | Side | |
| `config` | Config | |
| `control` | Control | |
| `display` | Display | |
| `task` | Task | |
| `location` | Location | |
| `namespace` | Namespace | |
| `hashMap` | HashMap | See interfaces, [§4.4](#44-interfaces-typed-hashmaps). |
| `teamMember` | TeamMember | |
| `scriptHandle` | ScriptHandle | Result of `spawn` / `execVM`. |
| `diaryRecord` | DiaryRecord | |
| `structuredText` | StructuredText | |
| `exception` | ExceptionHandle | |
| `edenEntity` | EdenEntity | |
| `edenId` | EdenID | |
| `waypoint` | Waypoint | Branded `[group, number]`, see §1.5. |
| `turretPath` | TurretPath | Branded `number[]`. |
| `treePath` | Path | Branded `number[]`. |
| `unitLoadout` | UnitLoadoutArray | Branded `array`. |
| `color` | Color | Branded `[number, number, number, number?]`. |
| `forType`, `ifType`, `switchType`, `whileType`, `withType` | ForType … WithType | Control-structure intermediates; rarely written by hand but needed so `if (…)` / `then` compose. |

`hashMapKey` is a built-in alias, not a distinct primitive:

```sqfts
type hashMapKey = number | string | boolean | side | config | group
    | object | task | location | display | control | teamMember | array;
```

### 1.2 `any` and gradual semantics

- An expression with no better information has type `any`.
- `any` is assignable to every type, and every type is assignable to `any`
  (bidirectional, like TypeScript's `any`, not like `unknown`).
- All plain `.sqf` files and all undeclared global variables/functions are
  `any`. Errors can therefore never be introduced merely by *importing*
  untyped code — only by contradicting an explicit annotation or the engine
  command database.
- Database types of `Unknown` (from Phase 1 extraction) surface as `any`.

### 1.3 `nothing` and `nil`

- `nil` has type `nothing`.
- Commands whose database return type is Nothing (e.g. `setDamage`) have type
  `nothing`; using their result is an error.
- An optional parameter declared `_x?: T` (no default) has type `T | nothing`
  inside the body, because a missing SQF param is `nil`. Providing a default
  (`_x: T = expr`) removes `nothing` from the body type.
- Under the `strictNil` flag, `T | nothing` must be narrowed (e.g. via
  `isNil`) or defaulted before use where `T` is expected. Without the flag,
  `nothing` in a union is only reported when a value is *known* to be nil.

### 1.4 Arrays, typed arrays, and tuples

- `array` — the untyped SQF array, equivalent to `any[]`.
- `T[]` — homogeneous array (`string[]`, `object[]`, `number[][]`).
  Corresponds to Phase 1 `ArrayOf(T)`.
- `[T1, T2, …]` — fixed-shape tuple. Trailing elements may be optional:
  `[string, number, boolean?]`.
- Assignability: `[number, number, number]` is assignable to `number[]`,
  which is assignable to `array`/`any[]`. The reverse directions require a
  cast ([§4.5](#45-casts)).
- Array literals infer as tuples when every element type is known, and decay
  to `T[]`/`array` on assignment as needed.

### 1.5 Branded array types (positions, vectors, waypoints, colors)

Several engine concepts are plain arrays at runtime but semantically distinct.
SQFts models them as **branded aliases**: structurally they are tuples of
`number` (or `[group, number]` for waypoints), but the brand is tracked
nominally so that mixing coordinate spaces is an error.

| Type | Runtime shape |
|---|---|
| `position2D` | `[number, number]` |
| `position3D` | `[number, number, number]` (space unspecified) |
| `positionATL`, `positionASL`, `positionASLW`, `positionAGL`, `positionRelative` | `[number, number, number]` |
| `positionAGLS` | `[number, number, number?]` (surface-snapped; Z optional) |
| `position` | union of all of the above (what the wiki calls bare "Position") |
| `vector2D` | `[number, number]` |
| `vector3D` | `[number, number, number]` |
| `waypoint` | `[group, number]` |
| `color` | `[number, number, number, number?]` |

Rules:

- A **literal** tuple of the right shape is assignable to any matching brand:
  `[0, 0, 0]` is a valid `positionATL` (fresh literals carry no brand).
- A branded value is assignable to its structural shape (`positionASL` →
  `[number, number, number]` → `number[]`).
- One brand is **not** assignable to another: passing a `positionATL` where
  `positionASL` is expected is an error. Convert with the engine commands
  (`ATLToASL` returns `positionASL`, `AGLToASL`, `getPosASL`, …) or cast.
- Bare `position3D` accepts any 3D positional brand; the specific brands
  accept `position3D` only under a cast (it is the "I don't know the space"
  escape hatch, one step stricter than `any`).

This is directly motivated by Phase 1 findings: the wiki types 28 waypoint
commands' parameter as `Waypoint` while COMREF prose says `Array` — both are
right, and a brand captures it. Under the `noPositionBrands` flag all brands
collapse to their structural shapes for codebases that don't want this rigor.

### 1.6 Unions

`A | B` accepts a value of either type. Unions come up constantly in the
engine database — Phase 1 extracted e.g. `code | string` for `addEventHandler`
handlers and `namespace | object | group | task | location | teamMember` for
`allVariables`. A value of union type may only be used in ways valid for
*every* member; use narrowing or a cast otherwise.

Minimal narrowing is specified for v1 — inside an `if` guarded by:

- `isNil "_x"` / `!isNil "_x"` — removes/selects `nothing`,
- `_x isEqualType X` — selects the member matching exemplar `X`,

the narrowed type applies in the corresponding branch. Anything fancier
(typeof-style maps, discriminated tuples) is future work ([§9](#9-future-work-non-normative)).

### 1.7 Code values

`code` is opaque in v1: the checker does not track what a code block expects
in `_this` or returns. Parameterized code types (e.g.
`code(unit: object) => boolean` for event handlers) are future work. Code
*literals* are still checked internally like any other scope.

---

## 2. Where types come from

Priority order when the checker resolves a symbol or call:

1. **Explicit annotation** in the current file (`private _x: T`, typed
   `params`).
2. **Declarations** from `.d.sqfts` files and `declare` statements in scope
   (global variables, user functions).
3. **Engine command database** (Phase 1 YAML): every command/operator call is
   matched against its overloads; argument types are checked and the return
   type flows out.
4. **Initializer inference**: `private _u = vehicle player;` gives `_u` type
   `object` without an annotation.
5. Otherwise `any`.

A variable's declared or inferred type is fixed for its scope; assigning an
incompatible value is an error (`any` is always compatible). Reassignment does
not re-infer.

---

## 3. Annotation syntax

All SQFts syntax is chosen so it is **invalid in plain SQF** (no valid SQF
program changes meaning) and **erasable by local rewrite** ([§7](#7-erasure-rules)).

### 3.1 Typed `private`

```sqfts
// SQFts
private _tries: number = 0;
private _target: object | group = objNull;
private _names: string[] = [];

// declare-then-assign (type without initializer)
private _result: string;
if (_ok) then { _result = "yes"; } else { _result = "no"; };
```

Erases to:

```sqf
private _tries = 0;
private _target = objNull;
private _names = [];

private "_result";
if (_ok) then { _result = "yes"; } else { _result = "no"; };
```

Note the third form: `private _result: string;` (no `=`) erases to the
string-form declaration `private "_result";`, which is the only valid SQF
spelling of declare-without-assign. This is the one place typed `private`
erasure is a rewrite rather than a deletion.

The annotation is only recognized immediately after a `private` local (or in
`params`, below). A colon anywhere else is parsed as plain SQF (e.g. `case`
labels, `:` operator in `switch`).

### 3.2 Typed `params`

Plain SQF already has runtime-checked params:

```sqf
params [
    ["_vehicle", objNull, [objNull]],
    ["_fee", 0, [0]]
];
```

SQFts adds a compile-time-typed spelling:

```sqfts
params [
    "_vehicle": object,
    "_fee": number = 0,
    "_note"?: string
];
```

Semantics:

- `"_name": T` — required param of type `T`.
- `"_name": T = expr` — optional with default; `expr` must be assignable to
  `T`; body type is `T`.
- `"_name"?: T` — optional without default; body type is `T | nothing`.
- Typed and plain entries may be mixed in one `params` array; plain entries
  keep their SQF runtime semantics and contribute types from their guard
  arrays (`[0]` → `number`, etc.) or `any`.

Erasure (default mode — see [§7.4](#74-optional-runtime-params-lowering) for the
runtime-lowering option):

```sqf
params [
    "_vehicle",
    ["_fee", 0],
    "_note"
];
```

`"_x": T` with no default erases to the bare string; `= expr` erases to the
two-element `["_x", expr]` form; `?` erases to nothing.

### 3.3 `type` aliases

```sqfts
type moneyTarget = object | group;
type vehicleRow = [string, number, boolean];   // classname, price, insured
type gearList = string[];
```

- Statement form: `type Name = TypeExpr;` — erased entirely (whole statement,
  including trailing `;` and one newline).
- `type` is a **contextual keyword**: it is only treated as a keyword when
  followed by an identifier and `=` ... `;` forming a type expression. A plain
  SQF assignment to a global named `type` (`type = 5;`) still parses as SQF,
  preserving the superset property. The same rule applies to `declare` and
  `interface`.
- Alias names are their own namespace (they can't collide with variables) and
  are file-scoped unless exported from a `.d.sqfts` file, in which case they
  are project-visible.

### 3.4 Interfaces (typed HashMaps)

An `interface` describes the known keys of a `hashMap`. At runtime the value
is an ordinary HashMap; the interface only constrains `get`/`set` usage.

```sqfts
interface PlayerStats {
    cash: number;
    bank: number;
    licenses: string[];
    gang?: string;          // optional key: get returns string | nothing
}

private _stats: PlayerStats = createHashMapFromArray [
    ["cash", 0], ["bank", 5000], ["licenses", []]
];
_stats set ["cash", 100];          // OK
_stats get "bank" + 1;             // number
_stats set ["cash", "lots"];       // error: string not assignable to number
_stats get "unknownKey";           // error under strict; any otherwise
```

Interfaces are erased entirely, like `type` statements. `PlayerStats` is
assignable to `hashMap`; the reverse requires a cast.

### 3.5 Casts: `as`

```sqfts
private _crew = (_this select 0) as object[];
private _pos = (getMarkerPos _mk) as positionATL;
```

- `expr as T` asserts the type. It is compile-time only and erases to `expr`
  (the ` as T` token run is deleted).
- `as` has **lower precedence than any binary command**, so
  `_this select 0 as string[]` means `(_this select 0) as string[]`.
  Parenthesize for clarity anyway.
- A cast is only legal between types that overlap (either direction of
  assignability, or via `any`). `"abc" as number` is an error; go through
  `as any` if you really mean it.
- `as` is contextual: it is only a keyword in postfix expression position
  followed by a type expression; a global variable named `as` in plain SQF
  still parses.

### 3.6 Function declarations

`declare function` gives a type to a global function invoked with
`call` / `spawn` / `remoteExec`. The parameter list describes the `_this`
tuple; the return type describes the value of the function body.

```sqfts
declare function TAG_fnc_impoundVehicle(_vehicle: object, _fee?: number): boolean;
declare function SERVER_fnc_setMapMarker(_markerId: number): nothing;
declare function TAG_fnc_nearestUnits(): object[];   // nular _this
```

Call-site checking:

```sqfts
private _ok = [_veh, 500] call TAG_fnc_impoundVehicle;   // _ok: boolean
[_veh] call TAG_fnc_impoundVehicle;                      // OK, _fee optional
["a"] call TAG_fnc_impoundVehicle;                       // error: string vs object
[23] spawn SERVER_fnc_setMapMarker;                      // scriptHandle
```

Rules:

- A single required parameter also accepts the unwrapped form
  (`_veh call TAG_fnc_x`), matching SQF's convention that a 1-tuple and a
  bare value are often interchangeable; the checker accepts either when the
  declaration has exactly one required param whose type is not itself an
  array/tuple type.
- `spawn` with a declared function checks the argument tuple and returns
  `scriptHandle` (return type of the function is irrelevant to `spawn`).
- `remoteExec`/`remoteExecCall` with a **string literal** function name that
  has a declaration checks the argument tuple. Non-literal names stay `any`.
- If the function's defining file is part of the project and has typed
  `params`, the checker verifies declaration and definition agree (arity,
  types, optionality) and that the file's trailing-expression type is
  assignable to the declared return type.
- `declare function` statements erase entirely. They may appear in `.sqfts`
  files but conventionally live in `.d.sqfts`.

### 3.7 Global variable declarations

```sqfts
declare project_playerCash: number;
declare project_groupList: string[];
declare project_debug: boolean;
```

Assignments and reads of a declared global are checked everywhere in the
project. Undeclared globals remain `any`. Erased entirely.

---

## 4. Declaration files (`.d.sqfts`)

A `.d.sqfts` file contains only `declare`, `type`, and `interface` statements
(plus comments). It produces no output and exists so that plain-SQF codebases
can be typed incrementally, one declaration file at a time, without touching
sources.

```sqfts
// mission.d.sqfts — hand-written or generated
type moneyTarget = object | group;

declare project_playerCash: number;
declare project_serverReady: boolean;

declare function TAG_fnc_checkPayment(_unit: object, _amount: number): boolean;
declare function TAG_fnc_catalogConfig(_catalog: string): array;
declare function SERVER_fnc_updateVehicleMods(
    _vehicle: object,
    _insurance: boolean,
    _unused: array,
    _mods: string[]
): nothing;
```

Resolution:

- The compiler loads every `.d.sqfts` reachable from the project config
  (`sqfts.toml`, Phase 3 defines the exact discovery rules).
- Duplicate declarations for the same symbol must be identical; otherwise a
  conflict error names both files.
- A declaration whose symbol is also *defined* in a checked `.sqfts` file is
  verified against the definition ([§3.6](#36-function-declarations)).

Phase 4 plans a generator that emits skeleton `.d.sqfts` for existing
codebases (e.g. all mission `TAG_fnc_*`/`SERVER_fnc_*` from `Functions.h` and
`config.cpp`, with `any` types to be tightened by hand).

---

## 5. Checking model

- **Engine calls**: every nular/unary/binary command use is resolved against
  the Phase 1 database. Overloads are tried in order; the first whose
  parameter types accept the (possibly-`any`) arguments wins and supplies the
  return type. If none match and no argument is `any`, error. If arguments
  are `any`, the union of all overload returns is used (collapsing to `any`
  if they disagree).
- **Operators** (`+`, `select`, `#`, `>>`, …) are commands and are typed the
  same way — e.g. `_arr # 0` yields the element type of `_arr` when known.
- **Statement result**: SQF scopes evaluate to their last expression; the
  checker tracks this for function return types and `if/then/else` values
  (`if` with both branches yields the union of branch types; without `else`,
  `T | nothing`).
- **No implicit re-typing**: a variable has one type per scope (declared or
  inferred at first assignment); shadowing via a new `private` in an inner
  scope is a fresh variable, matching SQF semantics.

### 5.1 Strictness flags

Configured in `sqfts.toml`; all default **off** so an untouched SQF codebase
checks clean.

| Flag | Effect |
|---|---|
| `noImplicitAny` | Error when a `params` entry, declared function param, or first assignment yields `any` with no annotation. |
| `strictNil` | `T \| nothing` must be narrowed or defaulted before use as `T`. |
| `noPositionBrands` | Disable nominal position/vector/waypoint/color brands ([§1.5](#15-branded-array-types-positions-vectors-waypoints-colors)). |
| `strictHashMap` | `get` with a key not present on the interface is an error instead of `any`. |
| `checkPlainSqf` | Also run the checker (engine database only) over sibling `.sqf` files. |

### 5.2 Diagnostics

TypeScript-style: primary span plus related spans, through the preprocessor's
source maps so errors in macro expansions point at both the use site and the
macro definition. Error codes are stable (`STS1001` unknown type name,
`STS2003` argument type mismatch, `STS2107` brand mismatch, …; final numbering
in Phase 3).

---

## 6. Grammar deltas

Additions over the SQF grammar (all others unchanged):

```text
TypedPrivate   ::= "private" LocalVar ":" Type ( "=" Expr )? ";"
TypedParamEnt  ::= StringLit "?"? ":" Type ( "=" Expr )?          // inside params array literal
TypeAlias      ::= "type" Ident "=" Type ";"
Interface      ::= "interface" Ident "{" ( Ident "?"? ":" Type ";" )* "}"
DeclareVar     ::= "declare" Ident ":" Type ";"
DeclareFn      ::= "declare" "function" Ident "(" ParamList? ")" ":" Type ";"
CastExpr       ::= Expr "as" Type                                  // lowest precedence
Type           ::= UnionType
UnionType      ::= ArrayType ( "|" ArrayType )*
ArrayType      ::= AtomType ( "[" "]" )*
AtomType       ::= PrimitiveName | Ident | TupleType | "(" Type ")"
TupleType      ::= "[" Type "?"? ( "," Type "?"? )* "]"
```

Disambiguation (superset preservation):

| Token | Keyword when… | Plain SQF when… |
|---|---|---|
| `type`, `declare`, `interface` | statement-initial and followed by the grammar above | followed by `=` (global assignment) or any other SQF continuation |
| `as` | expression-postfix and followed by a type expression | anywhere else (e.g. a variable named `as`) |
| `:` | after a `private` local or a string in a `params` array literal | `case` labels, `switch` `:` operator |
| `?` | in typed param entries and interface members | nowhere in SQF (invalid character today), so no conflict |

All new constructs are invalid syntax in plain SQF, so no existing program
parses differently.

Preprocessing: SQFts checks run **after** the HEMTT preprocessor, so macros
may expand to annotated code; spans map back through the expansion.

**v1 implementation note (Phase 3):** erasure runs on the *unpreprocessed*
source so that E1–E3 (byte-identity / locality) hold. Consequently, type
annotations must appear **literally** in `.sqfts` source in v1 — annotations
produced only by macro expansion are out of scope until a later revision that
can erase through the preprocessor source map.

---

## 7. Erasure rules

### 7.1 Guarantees

- **E1 — Identity on plain SQF**: a file containing no SQFts syntax is
  emitted byte-for-byte identical.
- **E2 — Determinism**: same input and compiler version produce identical
  output bytes.
- **E3 — Locality**: erasure only touches the exact spans of SQFts
  constructs; all other bytes (comments, whitespace, line endings, `#include`
  directives — erasure operates on the *unpreprocessed* source) pass through.
- **E4 — Lint-clean**: for annotated code, the rewrites are chosen to satisfy
  HEMTT lints (e.g. typed `private` erases to the `private _x = v` /
  `private "_x"` forms that L-S16 expects).

### 7.2 Per-construct rules

| Construct | Erasure |
|---|---|
| `: Type` in `private` with `=` | delete `: Type` (and one preceding space if present) |
| `private _x: T;` (no `=`) | rewrite to `private "_x";` |
| `"_p": T` params entry | delete `: T` → `"_p"` |
| `"_p": T = expr` | rewrite entry to `["_p", expr]` |
| `"_p"?: T` | delete `?: T` → `"_p"` |
| `expr as T` | delete ` as T` |
| `type` / `interface` / `declare` statements | delete the whole statement plus one trailing newline |
| `.d.sqfts` file | no output at all |

### 7.3 Example

Input (`fn_impoundVehicle.sqfts`):

```sqfts
// File: fn_impoundVehicle.sqfts
// Author: Example Mission
// Description: Impounds a vehicle and charges the owner.
type feeTier = [number, number];

params [
    "_vehicle": object,
    "_fee": number = 0
];

private _owner: object = _vehicle getVariable ["project_owner", objNull];
if (isNull _owner) exitWith { false };

private _pos = getPosATL _vehicle;      // inferred positionATL
[_owner, _fee] call TAG_fnc_checkPayment
```

Output (`fn_impoundVehicle.sqf`):

```sqf
// File: fn_impoundVehicle.sqfts
// Author: Example Mission
// Description: Impounds a vehicle and charges the owner.

params [
    "_vehicle",
    ["_fee", 0]
];

private _owner = _vehicle getVariable ["project_owner", objNull];
if (isNull _owner) exitWith { false };

private _pos = getPosATL _vehicle;      // inferred positionATL
[_owner, _fee] call TAG_fnc_checkPayment
```

### 7.4 Optional runtime `params` lowering

With `emitRuntimeParams = true`, typed `params` entries additionally emit the
native SQF guard array, trading byte-minimalism for runtime defense (useful
for `remoteExec` targets per network-safety conventions):

```sqf
params [
    ["_vehicle", objNull, [objNull]],
    ["_fee", 0, [0]]
];
```

Type-to-exemplar mapping: `number`→`0`, `string`→`""`, `boolean`→`false`,
`object`→`objNull`, `group`→`grpNull`, `array`/tuples/brands→`[]`,
`code`→`{}`, `config`→`configNull`, `side`→`sideUnknown`,
`control`→`controlNull`, `display`→`displayNull`, `task`→`taskNull`,
`location`→`locationNull`, `namespace`→`missionNamespace`, unions → one
exemplar per member. Types with no exemplar (`any`, `hashMap` pre-2.02
targets, control-structure types) omit the guard. Tuples of known length also
emit the size list (`[3]` for `[number, number, number]`). This is the only
mode in which types influence emitted runtime behavior, and it only ever
*narrows* accepted inputs the way hand-written guards would.

---

## 8. Worked example: typing an existing file via declarations only

`mission.d.sqfts` (new file; no source edits):

```sqfts
declare project_serviceFee: number;
declare function TAG_fnc_checkPayment(_unit: object, _amount: number): boolean;
```

Existing `fn_payFee.sqf` (unchanged, plain SQF) — with `checkPlainSqf`
enabled the checker now reports on it:

```sqf
private _ok = [player, "500"] call TAG_fnc_checkPayment;
//                    ~~~~~ STS2003: argument 2 is string, expected number
project_serviceFee = true;
// ~~~~~~~~~~~~ STS2004: boolean not assignable to declared type number
```

This is the Phase 4 adoption path: generate declarations, tighten them, and
only later (if ever) rename files to `.sqfts` for inline annotations.

---

## 9. Future work (non-normative)

Deliberately out of scope for v1, recorded so syntax stays forward-compatible:

- **Typed code values** — `code(_unit: object) => boolean` for event-handler
  and `addAction` parameters; would let the database's `code | string` unions
  check their payloads.
- **Literal types** — `"west" | "east"` string unions, numeric enums for
  flag-style params (`showIn3D` in `addAction`).
- **Richer narrowing** — `typeName`/`isEqualTypeArray` maps, discriminated
  tuples, exhaustiveness for `switch`.
- **Generics** — `T[] select code` preserving `T`; `apply` mapping types.
- **`hashMap<K, V>`** — parameterized maps beyond interfaces.
- **Event-handler name typing** — `addEventHandler ["Killed", …]` selecting a
  payload tuple from an event table (arma3-wiki ships event data usable for
  this).

---

## 10. Conformance summary

An implementation is a conforming SQFts compiler if:

1. It accepts every valid SQF program unchanged and emits it byte-identically (E1).
2. It parses the grammar deltas of §6 with the disambiguation rules given.
3. It checks types per §1–§5 against the Phase 1 engine database and loaded
   declarations, reporting but never suppressing errors.
4. Its emitted SQF follows the erasure table of §7.2 exactly (E2–E4).
