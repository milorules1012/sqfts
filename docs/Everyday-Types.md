# Everyday Types

Surface type names are **lowerCamelCase** keywords. They map onto Arma’s value model (aligned with [arma3-wiki](https://github.com/acemod/arma3-wiki)).

## Primitive types

| SQFts type | Meaning |
|---|---|
| `any` | Gradual type; assignable to and from everything |
| `nothing` | Type of `nil` and of statements/commands with no value |
| `number` | Number |
| `string` | String |
| `boolean` | Boolean |
| `array` | Untyped array; shorthand for `any[]` |
| `code` | Code block — bare form is gradual; see parameterized `code` below |
| `object` | Object |
| `group` | Group |
| `side` | Side |
| `config` | Config |
| `control` | Control |
| `display` | Display |
| `task` | Task |
| `location` | Location |
| `namespace` | Namespace |
| `hashMap` | HashMap (refine with [Interfaces](Interfaces)) |
| `teamMember` | TeamMember |
| `scriptHandle` | Result of `spawn` / `execVM` |
| `diaryRecord` | DiaryRecord |
| `structuredText` | StructuredText |
| `exception` | Exception handle |
| `edenEntity` | Eden entity |
| `edenId` | Eden ID |
| `forType`, `ifType`, `switchType`, `whileType`, `withType` | Control-structure intermediates (rarely written by hand) |

### Built-in alias: `hashMapKey`

```sqfts
type hashMapKey = number | string | boolean | side | config | group
    | object | task | location | display | control | teamMember | array;
```

## `any`

- An expression with no better information has type `any`.
- `any` is assignable **to** every type and every type is assignable **to** `any` (bidirectional, like TypeScript’s `any`, not `unknown`).
- Plain `.sqf` and undeclared symbols are `any`.
- Database types of `Unknown` surface as `any`.

## `nothing` and `nil`

- `nil` has type `nothing`.
- Commands whose return type is Nothing (e.g. `setDamage`) have type `nothing`; using their result is an error (`STS2006`).
- Optional param `"_x"?: T` (no default) has body type `T | nothing`, because a missing SQF param is `nil`.
- Providing a default (`"_x": T = expr`) removes `nothing` from the body type.

With the [`strictNil`](Strictness-Flags) flag, `T | nothing` must be narrowed (e.g. via `isNil`) or defaulted before use where `T` is expected. Without the flag, `nothing` in a union is only reported when a value is *known* to be nil.

## `code`

Bare `code` is the gradual form: assignable to and from any parameterized `code(…) : R`.

### Parameterized form

```sqfts
private _pred: code(unit: object): boolean = { alive _this };
private _onKilled: code(): nothing = { hint "killed" };
```

- The param list describes the `_this` tuple (same convention as [`declare function`](Declaring-Functions-and-Globals)): names are documentation only.
- Return type uses `:` (same as `declare function …: R`).
- One required non-array/tuple param → `_this` has that bare type; multiple (or optional/array) params → `_this` is a tuple.
- Empty params (nular) leave `_this` unbound (`any`).
- When a `{ … }` literal is checked against a parameterized expected type, the checker binds `_this` and checks the block’s last expression against the return type (`STS2005` on mismatch).
- Assignability between parameterized forms: **contravariant** params, **covariant** return.
- Engine `code | string` unions accept opaque `code`, parameterized `code`, and `string`.

Event-handler name → payload tables (e.g. `addEventHandler ["Killed", …]`) are separate ([Missing Features](Missing-Features) **B-EventHandlers**).

## Examples

```sqfts
private _n: number = 0;
private _name: string = name player;
private _alive: boolean = alive player;
private _veh: object = vehicle player;
private _items: array = [];           // any[]
private _script: scriptHandle = [] spawn {};
private _pred: code(unit: object): boolean = { alive _this };
```

## Related

- [Arrays, Tuples, and Brands](Arrays-Tuples-and-Brands)
- [Unions and Narrowing](Unions-and-Narrowing)
- [Annotating Variables](Annotating-Variables)
