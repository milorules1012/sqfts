# Unions and Narrowing

## Union types

`A | B` accepts a value of either type. Unions appear constantly in the engine database — for example `code | string` for event-handler parameters, or wide unions for `allVariables`.

```sqfts
type moneyTarget = object | group;
private _t: moneyTarget = player;
```

A value of union type may only be used in ways valid for **every** member. Use narrowing or a [cast](Casts) otherwise.

## Assignability

- A value of type `A` is assignable to `A | B`.
- A value of type `A | B` is assignable to `C` only if **both** `A` and `B` are assignable to `C`.
- `any` still short-circuits both directions.

## Narrowing (v1)

Minimal narrowing is specified for v1. Inside an `if` guarded by:

| Guard | Effect |
|---|---|
| `isNil "_x"` | Selects `nothing` in the true branch; removes `nothing` in the false branch (and vice versa for `!isNil`) |
| `_x isEqualType X` | Selects the union member matching exemplar `X` |

the narrowed type applies in the corresponding branch.

```sqfts
params ["_note"?: string];

if (!isNil "_note") then {
    // _note: string
    hint _note;
};
```

With [`strictNil`](Strictness-Flags), using `_note` as `string` outside a narrowing context is an error (`STS2202`).

## What is not in v1

Richer narrowing is tracked under [Missing Features](Missing-Features) (bundle **B-Narrowing**):

- `typeName` / `isEqualTypeArray` maps
- Discriminated tuples
- Exhaustiveness checking for `switch`

Until then, use casts when you know more than the checker.

> **Caveat:** even the minimal `isNil` / `isEqualType` narrowing described above is not fully enforced by the checker yet; see Missing Features.
