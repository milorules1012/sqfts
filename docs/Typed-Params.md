# Typed Params

Plain SQF already supports runtime-checked `params` with guard arrays. SQFts adds a compile-time-typed spelling that erases to the simple string / default forms.

## Plain SQF (still valid)

```sqf
params [
    ["_vehicle", objNull, [objNull]],
    ["_fee", 0, [0]]
];
```

## Typed spelling

```sqfts
params [
    "_vehicle": object,
    "_fee": number = 0,
    "_note"?: string
];
```

### Entry forms

| Syntax | Meaning | Body type |
|---|---|---|
| `"_name": T` | Required param of type `T` | `T` |
| `"_name": T = expr` | Optional with default; `expr` must be assignable to `T` | `T` |
| `"_name"?: T` | Optional without default | `T \| nothing` |

Typed and plain entries may be **mixed** in one `params` array. Plain entries keep SQF runtime semantics and contribute types from their guard arrays (`[0]` → `number`, etc.) or `any`.

## Default erasure

```sqf
params [
    "_vehicle",
    ["_fee", 0],
    "_note"
];
```

| Typed form | Erases to |
|---|---|
| `"_x": T` | `"_x"` |
| `"_x": T = expr` | `["_x", expr]` |
| `"_x"?: T` | `"_x"` |

## Runtime params lowering

Enabled by default (`emit_runtime_params = true` in [`sqfts.toml`](Configuration)). Typed entries additionally emit native guard arrays:

```sqf
params [
    ["_vehicle", objNull, [objNull]],
    ["_fee", 0, [0]]
];
```

Type → exemplar mapping includes `number`→`0`, `string`→`""`, `boolean`→`false`, `object`→`objNull`, `group`→`grpNull`, arrays/tuples/brands→`[]`, `code`→`{}`, and similar nulls for UI/config types. Unions emit one exemplar per member. Types with no exemplar (`any`, some HashMap targets, control-structure types) omit the guard.

This is the **only** mode in which types influence runtime behavior, and it only ever *narrows* accepted inputs the way hand-written guards would. Useful for `remoteExec` targets.

See [Erasure](Erasure#optional-runtime-params-lowering).

## Call-site checking

Typed params on a function that also has a [`declare function`](Declaring-Functions-and-Globals) are verified for agreement (arity, types, optionality). Call sites through `call` / `spawn` / literal `remoteExec` names are checked against the declaration.
