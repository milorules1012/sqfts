# Diagnostics

SQFts uses stable diagnostic codes of the form `STSxxxx`, analogous to TypeScript’s `TSxxxx`.

## Codes

### Syntax and declarations (1xxx)

| Code | Name | Meaning |
|---|---|---|
| `STS1001` | Unknown type | Unknown type name in an annotation or declaration |
| `STS1002` | Duplicate declaration | Conflicting declarations for the same symbol |
| `STS1003` | Bad declaration | Malformed `.d.sqfts` / `declare` content |
| `STS1004` | Syntax error | Scan / parse / preprocess failure |

### Type mismatches (2xxx)

| Code | Name | Meaning |
|---|---|---|
| `STS2003` | Argument mismatch | Call / command argument types do not match an overload |
| `STS2004` | Assignment mismatch | Value not assignable to the variable’s type |
| `STS2005` | Return mismatch | Function body type not assignable to declared return |
| `STS2006` | Use of nothing | Using a `nothing`-typed value where a value is required |

### Brands and casts (21xx)

| Code | Name | Meaning |
|---|---|---|
| `STS2107` | Brand mismatch | Incompatible branded types (e.g. ATL vs ASL) |
| `STS2108` | Illegal cast | Cast between non-overlapping types |

### Strictness (22xx)

| Code | Name | Meaning |
|---|---|---|
| `STS2201` | Implicit any | `any` inferred under `no_implicit_any` |
| `STS2202` | Strict nil | Un-narrowed `T \| nothing` under `strict_nil` |
| `STS2203` | Unknown HashMap key | Key not on interface under `strict_hash_map` |

### Consistency and commands (23xx–24xx)

| Code | Name | Meaning |
|---|---|---|
| `STS2301` | Decl/def mismatch | Declaration disagrees with typed definition |
| `STS2401` | Unknown command | Command not in the engine database (informational when args are typed) |

## Shape

Each diagnostic has:

- A **primary span** in the source
- Optional **related spans** (e.g. declaration site, macro definition)
- A **severity** (error / warning / note)
- A human-readable **message**

In the editor, codes appear on hover and in the Problems panel via the [language server](Editor-Support).

## Example

```text
fn_payFee.sqf:1:24 STS2003: argument 2 is string, expected number
private _ok = [player, "500"] call TAG_fnc_checkPayment;
                       ~~~~~
```
