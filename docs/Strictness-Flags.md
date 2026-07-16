# Strictness Flags

Configured under `[flags]` in [`sqfts.toml`](Configuration). **All default off** so an untouched SQF codebase checks clean.

| TOML key | Effect |
|---|---|
| `no_implicit_any` | Error (`STS2201`) when a `params` entry, declared function param, or first assignment yields `any` with no annotation |
| `strict_nil` | `T \| nothing` must be narrowed or defaulted before use as `T` (`STS2202`) |
| `no_position_brands` | Disable nominal position / vector / waypoint / color brands — collapse to structural shapes |
| `strict_hash_map` | `get` with a key not on the interface is an error (`STS2203`) instead of `any` |
| `check_plain_sqf` | Also run the checker (engine DB + declarations) over sibling `.sqf` files |

## Example

```toml
[flags]
no_implicit_any = false
strict_nil = false
no_position_brands = false
strict_hash_map = false
check_plain_sqf = true
```

## Recommended progression

1. Start with all flags off; add `.d.sqfts` skeletons via [declgen](Declgen).
2. Turn on `check_plain_sqf` to surface call-site mismatches against declarations.
3. Enable `strict_nil` / `strict_hash_map` on folders that already have good annotations.
4. Enable `no_implicit_any` last, as a “fully typed” gate for new code.

`no_position_brands` is for codebases that prefer not to track ATL vs ASL; leave it off if you want brand safety.
