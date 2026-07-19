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
| `code` | Code block (opaque in v1 — see below) |
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

## `code` (v1)

`code` is **opaque** in v1: the checker does not track what a code block expects in `_this` or returns. Parameterized forms such as `code(unit: object) => boolean` are tracked under [Missing Features](Missing-Features) (bundle **B-TypedCode**).

Code *literals* `{ … }` are still checked internally like any other scope.

## Examples

```sqfts
private _n: number = 0;
private _name: string = name player;
private _alive: boolean = alive player;
private _veh: object = vehicle player;
private _items: array = [];           // any[]
private _script: scriptHandle = [] spawn {};
```

## Related

- [Arrays, Tuples, and Brands](Arrays-Tuples-and-Brands)
- [Unions and Narrowing](Unions-and-Narrowing)
- [Annotating Variables](Annotating-Variables)
