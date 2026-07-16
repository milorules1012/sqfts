# Future Work

Non-normative roadmap. Recorded so syntax stays forward-compatible; not part of the v1 conformance surface.

## Planned / considered

### Typed code values

```text
code(_unit: object) => boolean
```

For event-handler and `addAction` parameters. Would let the database’s `code | string` unions check their payloads.

### Richer narrowing

- `typeName` / `isEqualTypeArray` maps
- Discriminated tuples
- Exhaustiveness for `switch`

### Generics

- `T[] select code` preserving `T`
- `apply` mapping types

### Parameterized HashMaps

`hashMap<K, V>` beyond [interfaces](Interfaces).

### Event-handler name typing

`addEventHandler ["Killed", …]` selecting a payload tuple from an event table (arma3-wiki ships event data usable for this).

### Erasure through preprocessor maps

Allow annotations introduced only by macro expansion while preserving byte-stable erasure (lifts the v1 “literal annotations only” restriction).

### Project include roots

Wire HEMTT / `sqfts.toml` include paths so `#include "macro.h"` resolves in large missions without `STS1004` preprocess failures.

## Conformance reminder

An implementation is conforming for v1 if it satisfies SPEC §10 (accept all SQF unchanged, parse grammar deltas, check per §1–§5, erase per §7). Future features above must not break those guarantees for existing SQFts programs.
