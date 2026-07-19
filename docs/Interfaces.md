# Interfaces

An `interface` describes the known keys of a `hashMap`. At runtime the value is an ordinary HashMap; the interface only constrains `get` / `set` usage.

## Syntax

```sqfts
interface PlayerStats {
    cash: number;
    bank: number;
    licenses: string[];
    gang?: string;          // optional key: get returns string | nothing
}
```

## Usage

```sqfts
private _stats: PlayerStats = createHashMapFromArray [
    ["cash", 0], ["bank", 5000], ["licenses", []]
];

_stats set ["cash", 100];          // OK
_stats get "bank" + 1;             // number
_stats set ["cash", "lots"];       // error: string not assignable to number
_stats get "unknownKey";           // error under strictHashMap; any otherwise
```

## Assignability

- `PlayerStats` (interface / named type) is assignable **to** `hashMap`
- `hashMap` is **not** assignable to `PlayerStats` without a [cast](Casts)

## Optional keys

Members marked `?` mean:

- The key may be absent
- `get` yields `T | nothing`

## Strictness

With [`strict_hash_map`](Strictness-Flags), `get` with a key not present on the interface is an error (`STS2203`) instead of `any`.

## Erasure

Interfaces are erased entirely, like `type` statements.

## Future

Parameterized `hashMap<K, V>` beyond interfaces is tracked under [Missing Features](Missing-Features) (bundle **B-HashMapTyping**).
