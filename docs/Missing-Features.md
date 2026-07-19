# Missing Features

Non-normative inventory of TypeScript-inspired type-system gaps and soundness holes in SQFts. Use this page to plan implementations **one bundle at a time**. It is **not** part of the v1 conformance surface and is not a full bug tracker for every unused diagnostic.

For the short roadmap pointer, see [Future Work](Future-Work).

## How to read

| Tag | Meaning |
|---|---|
| **missing** | Not in the language / checker |
| **partial** | Present in a limited form |
| **planned** | Called out in handbook / prior Future Work |
| **soundness hole** | Feature looks shipped, but checking is too weak |

| Priority | Meaning |
|---|---|
| **P0** | High value; unblocks other work; or docs claim it already |
| **P1** | Clear TypeScript analogue that fits SQF well |
| **P2** | Valuable after foundations land |
| **P3** | Optional parity polish |

**Bundle IDs** (e.g. `B-ArraySoundness`) group features that should ship together. Prefer implementing a whole bundle in one effort rather than splitting related checks across PRs.

## Highlighted holes

These three are the most visible day-to-day gaps relative to typed arrays and TypeScript callables.

### Opaque `code` (planned · P0 · B-TypedCode)

`code` is opaque in v1: the checker does not track what a block expects in `_this` or returns. See [Everyday Types](Everyday-Types).

```sqfts
// Today: both are just `code` — no mismatch
private _pred: code = { true };
private _onKilled: code = { hint str _this };

// Target: parameterized forms
// private _pred: code(unit: object) => boolean = { alive _this };
```

### Array mutation ignores element types (soundness hole · P0 · B-ArraySoundness)

Homogeneous `T[]` exists, but mutating commands treat the element as `any` and do not check assignability to `T`.

```sqfts
private _names: string[] = ["a", "b"];
_names pushBack 1;       // should error; currently accepted
_names set [0, 99];      // should error; currently accepted
_names append [1, 2];    // should error; currently accepted
```

### Collection commands accept non-arrays (soundness hole · P0 · B-ArraySoundness)

Expected wiki type `array` is soft-matched, so a wrong collection (e.g. `player`) often passes.

```sqfts
{ hint "x" } forEach player;   // should error; currently accepted
player apply { 1 };            // should error; currently accepted
player select 0;               // should error; currently accepted
player pushBack 1;             // should error; currently accepted
```

## Bundles (implement together)

| Bundle | Contents | Why together |
|---|---|---|
| **B-ArraySoundness** | Reject non-array collections for `forEach`, `apply`, `select`, `pushBack`, `set`, `append`, …; check mutation values against `T` for `T[]`; tighten soft-match for expected `array` | Same root cause in gradual overload matching |
| **B-ArrayGenerics** | Preserve / map element types through `select` / `apply` / `findIf`-style commands; fix `T[] select code` returning `T` instead of `T[]` | Needs sound collection typing first |
| **B-TypedCode** | `code(_this: T, …) => R` syntax + assignability; bind `_this` / params for literals from context; check `code \| string` DB payloads | Unlocks handlers and callable richness |
| **B-EventHandlers** | `addEventHandler ["Killed", …]` (and similar) payload tables from wiki event data | Depends on typed `code` |
| **B-Narrowing** | Implement documented `isNil` / `isEqualType`; add `isNull` / `typeName` / `isEqualTypeArray`; enforce [`strictNil`](Strictness-Flags) (`STS2202`); `never` + `switch` exhaustiveness; discriminated tuples | One control-flow story; docs already promise v1 narrowing |
| **B-HashMapTyping** | `hashMap<K, V>`, interface `extends`, index signatures, `keyof` | Completes HashMap typing beyond [Interfaces](Interfaces) |
| **B-CallableRichness** | User `declare` overloads; stricter `call` / `spawn` against typed `code` | After B-TypedCode |
| **B-DeclPackaging** | Declaration merging, emit `.d.sqfts` from sources, multi-root packaging | Ecosystem, not core theory |
| **B-IdeDepth** | Type-at-position, go-to-definition | Needs checker type maps |
| **B-ParityPolish** | `unknown`, boolean literal types, `satisfies`, intrinsic utilities | Optional TypeScript parity |

### Suggested implementation order

1. **B-ArraySoundness**
2. **B-Narrowing**
3. **B-TypedCode**
4. **B-ArrayGenerics**
5. **B-EventHandlers**
6. **B-HashMapTyping**
7. **B-CallableRichness** → **B-DeclPackaging** / **B-IdeDepth** / **B-ParityPolish** as needed

---

## Inventory by category

### Core types

| Feature | TS analogue | Status | Priority | Bundle |
|---|---|---|---|---|
| `unknown` (top type requiring narrowing) | `unknown` | missing | P2 | B-ParityPolish |
| `never` (bottom / exhaustiveness) | `never` | missing | P1 | B-Narrowing |
| Boolean literal types | `true` \| `false` | missing | P3 | B-ParityPolish |
| `enum` keyword | `enum` | missing (literal unions cover the need) | P3 | B-ParityPolish |
| User-defined brands / opaque aliases | branded nominal types | partial (engine brands only) | P3 | B-ParityPolish |
| Distinct `void` / `undefined` / `null` | same | partial by design — single [`nothing`](Everyday-Types) | — | — |

### Callables (`code` / functions)

| Feature | TS analogue | Status | Priority | Bundle |
|---|---|---|---|---|
| Parameterized `code(…) => R` | `(…args) => R` | planned / missing | P0 | B-TypedCode |
| `_this` / param typing from call context | `this` parameter | partial (literals checked as scopes only) | P0 | B-TypedCode |
| Event-handler name → payload typing | overload / mapped handlers | planned | P0 | B-EventHandlers |
| User `declare function` overloads | overload signatures | missing (engine DB has overloads) | P2 | B-CallableRichness |
| Type predicates / asserts | `x is T` / `asserts` | missing | P2 | B-Narrowing |
| `remoteExec` non-literal names | dynamic dispatch | partial (string literals only) | P2 | B-EventHandlers |

### Generics

| Feature | TS analogue | Status | Priority | Bundle |
|---|---|---|---|---|
| Type parameters (`<T>`) on aliases / interfaces | generics | planned / missing | P1 | B-ArrayGenerics / B-HashMapTyping |
| `T[] select code` preserving `T` | generic array methods | planned / partial (wrong return for code `select`) | P0 | B-ArrayGenerics |
| `apply` mapping types | `.map` inference | planned | P0 | B-ArrayGenerics |
| `hashMap<K, V>` | `Map<K, V>` / `Record` | planned | P1 | B-HashMapTyping |
| Constrained params / defaults | `T extends U`, `<T = any>` | missing | P2 | B-ArrayGenerics |

### Unions, narrowing, control flow

| Feature | TS analogue | Status | Priority | Bundle |
|---|---|---|---|---|
| Union types `A \| B` | unions | present | — | — |
| Intersection types `A & B` | intersections | missing | P1 | B-HashMapTyping |
| `isNil` / `isEqualType` narrowing | null / typeof checks | **documented v1, not enforced** | P0 | B-Narrowing |
| `strictNil` enforcement | `strictNullChecks` | partial / inert | P0 | B-Narrowing |
| `isNull` / `typeName` / `isEqualTypeArray` | richer guards | planned / missing | P1 | B-Narrowing |
| Discriminated tuples | discriminated unions | planned | P1 | B-Narrowing |
| `switch` exhaustiveness | exhaustiveness | planned | P1 | B-Narrowing |

### Arrays and mutation

| Feature | TS analogue | Status | Priority | Bundle |
|---|---|---|---|---|
| Homogeneous `T[]` / tuples | `T[]` / `[A, B]` | present | — | — |
| Mutation element checking (`pushBack`, `set`, …) | mutable array element checks | **soundness hole** | P0 | B-ArraySoundness |
| Collection argument checking (`forEach`, `apply`, …) | receiver typing | **soundness hole** | P0 | B-ArraySoundness |
| Soft-match of expected `array` | gradual call checking | **soundness hole** | P0 | B-ArraySoundness |
| `readonly T[]` | `ReadonlyArray<T>` | missing | P2 | B-ArraySoundness (follow-on) |
| Safer indexing (`T \| nothing` for `#`) | `noUncheckedIndexedAccess` | missing | P2 | B-ArraySoundness (follow-on) |
| Empty `[]` assignable to `T[]` | `[] as T[]` / contextual typing | partial (often rejected today) | P1 | B-ArrayGenerics |

### HashMaps / interfaces

| Feature | TS analogue | Status | Priority | Bundle |
|---|---|---|---|---|
| Interface key typing for `get` / `set` | object types | partial (present) | — | — |
| `interface` extends | `extends` | missing | P1 | B-HashMapTyping |
| Index signatures | `[key: string]: T` | missing | P1 | B-HashMapTyping |
| `keyof` / indexed access | `keyof T`, `T[K]` | missing | P2 | B-HashMapTyping |
| Structural interface assignability | structural typing | partial (named / nominal today) | P2 | B-HashMapTyping |

### Strictness and casts

| Feature | TS analogue | Status | Priority | Bundle |
|---|---|---|---|---|
| `no_implicit_any` | `noImplicitAny` | partial | P2 | B-ParityPolish |
| Checked `as` casts (`STS2108`) | type assertions | partial (erase-only today) | P1 | B-Narrowing |
| `strict` umbrella flag | `strict` | missing | P3 | B-ParityPolish |

### Declarations and tooling

| Feature | TS analogue | Status | Priority | Bundle |
|---|---|---|---|---|
| `.d.sqfts` ambient decls | `.d.ts` | present | — | — |
| Declaration merging | interface merge | missing | P2 | B-DeclPackaging |
| Emit `.d.sqfts` from `.sqfts` | declaration emit | missing | P2 | B-DeclPackaging |
| Type-at-position / go-to-definition | tsserver | missing | P2 | B-IdeDepth |
| Diagnostics / hover / completion | basics | present | — | — |

---

## Out of scope

TypeScript features that do not map cleanly onto SQF / SQFts. Do not prioritize these for language parity.

| Feature | Why skip |
|---|---|
| JSX / TSX | No UI tree model in SQF |
| Decorators | No class / decorator runtime |
| Classes, heritage, field modifiers | SQF OOP is objects + HashMaps + functions |
| ES `import` / `export` | Use `#include`, CfgFunctions, `.d.sqfts` |
| TS `namespace` / `module` merging | Conflicts with SQF `namespace` **values** |
| `bigint`, `symbol` | Not in the Arma value model |
| `async` / `await` / `Promise` | SQF uses `spawn` / `scriptHandle` |
| Emit to downlevel JS / polyfills | Erase target is SQF |

---

## Docs vs implementation caveats

1. **Narrowing / `strictNil`:** [Unions and Narrowing](Unions-and-Narrowing) and the archived SPEC describe v1 `isNil` / `isEqualType` narrowing. The checker still treats this as future work (“narrowing later”); `STS2202` is not emitted. Treat as incomplete, not done.
2. **Literal types:** Shipped now ([Literal Types](Literal-Types)). Older SPEC §9 listing them as future is stale.
3. **Macro annotations:** Checking runs after preprocess; do not list as missing ([FAQ](FAQ)).
4. **Array holes:** Typed arrays are documented as working for `#` / assignability, but mutation and collection soft-matching are undocumented soundness holes (see above).
5. **Several `STS` codes** exist in the diagnostics enum without emitters yet (e.g. return-body mismatch, use-of-`nothing`, illegal cast). Closing those belongs with the related bundles, not as a separate laundry list.

## Related

- [Future Work](Future-Work) — short roadmap pointer
- [Everyday Types](Everyday-Types) — opaque `code`
- [Arrays, Tuples, and Brands](Arrays-Tuples-and-Brands) — `T[]` surface
- [Unions and Narrowing](Unions-and-Narrowing) — promised vs richer narrowing
- [Interfaces](Interfaces) — HashMap typing today
