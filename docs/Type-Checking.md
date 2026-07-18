# Type Checking

How the checker decides types for expressions and whether a program is well-typed.

## Resolution priority

When resolving a symbol or call:

1. **Explicit annotation** in the current file (`private _x: T`, typed `params`)
2. **Declarations** from `.d.sqfts` and in-scope `declare` statements
3. **Engine command database** — every nular / unary / binary command matched against overloads
4. **Initializer inference** — `private _u = vehicle player` → `object`
5. Otherwise **`any`**

A variable’s type is fixed for its scope. Shadowing with a new `private` creates a fresh binding.

## Engine calls

Every command use is resolved against the [arma3-wiki](Engine-Command-Database) engine database:

1. Overloads are tried in order.
2. The first whose parameter types accept the (possibly-`any`) arguments wins and supplies the return type.
3. If none match and no argument is `any` → error (`STS2003`).
4. If arguments are `any`, the union of all overload returns is used (collapsing to `any` if they disagree).

Operators (`+`, `select`, `#`, `>>`, …) are commands and are typed the same way.

## Statement results

SQF scopes evaluate to their last expression. The checker tracks this for:

- Function return types
- `if` / `then` / `else` values — both branches → union of branch types; without `else` → `T | nothing`

## User functions

| Invocation | Checking |
|---|---|
| `args call Fn` | Argument tuple vs declaration; return type flows out |
| `args spawn Fn` | Argument tuple; result is always `scriptHandle` |
| `… remoteExec ["Fn", …]` | Args checked only if `"Fn"` is a string literal with a declaration |

See [Declaring Functions and Globals](Declaring-Functions-and-Globals).

## Diagnostics shape

TypeScript-style: primary span plus related spans. Through the preprocessor’s source maps, errors in macro expansions can point at both the use site and the macro definition. Codes are stable (`STS1001`, `STS2003`, …) — see [Diagnostics](Diagnostics).

## Pipeline (implementation)

1. Scan / erase annotations on **unpreprocessed** source (byte-stable spans)
2. Run HEMTT preprocessor + parser on erased SQF
3. Type-check against engine DB + loaded declarations
4. Map diagnostic spans back to original `.sqfts` locations

v1 restriction: annotations must appear literally in source (not only via macro expansion).
