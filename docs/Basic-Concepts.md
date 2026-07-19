# Basic Concepts

SQFts sits on three design pillars, mirroring TypeScript’s relationship to JavaScript.

## 1. Superset of SQF

Every valid Arma 3 SQF program is a valid SQFts program. Meaning does not change. You can rename `fn_foo.sqf` → `fn_foo.sqfts` and the checker treats it as fully `any` until you add annotations or declarations.

New syntax is chosen so it is **invalid in plain SQF**. That guarantees no existing script parses differently when opened as SQFts.

## 2. Erasable types

Annotations exist only for the checker. After checking, the compiler **erases** them and emits plain `.sqf`:

| Construct | Fate |
|---|---|
| `: Type` on `private` / `params` | Removed (or rewritten to valid SQF forms) |
| `expr as T` | Becomes `expr` |
| `type` / `interface` / `declare` | Entire statement deleted |
| `.d.sqfts` | Never emits output |

By default [`emit_runtime_params`](Erasure#runtime-params-lowering) is on: typed `params` also emit native SQF guard arrays for runtime defense. Set it to `false` to strip types only.

See [Erasure](Erasure) for the full rules (byte identity, locality, determinism).

## 3. Gradual typing

Untyped means `any`, not an error:

- Plain `.sqf` files → all symbols `any` (unless covered by `.d.sqfts`)
- Undeclared globals / functions → `any`
- Engine database `Unknown` → surfaces as `any`

Errors appear only when you **contradict** an explicit annotation or a known engine/declaration type — never merely by importing untyped code.

## File extensions

| Extension | Role |
|---|---|
| `.sqfts` | Source: SQF plus optional annotations. Compiles to `.sqf`. |
| `.d.sqfts` | Declarations only (`declare` / `type` / `interface`). No output. |
| `.sqf` | Plain SQF. Consumed as-is; fully `any` unless declarations apply. |

## Where types come from

When the checker resolves a symbol or call, priority is:

1. Explicit annotation in the current file
2. `.d.sqfts` / `declare` statements
3. Engine command database (nular / unary / binary overloads)
4. Initializer inference (`private _u = vehicle player` → `object`)
5. Otherwise `any`

Details: [Type Checking](Type-Checking).

## Contextual keywords

`type`, `declare`, `interface`, and `as` are **contextual**. They are keywords only in the positions defined by the grammar; elsewhere they remain ordinary SQF identifiers (e.g. `type = 5;` still assigns a global named `type`).

## Preprocessor note

Checking runs **after** the HEMTT preprocessor. Annotations may be introduced by macro expansion; the checker scans the processed buffer and erases those constructs before parsing. Emit rewrites unpreprocessed files (including `#define` bodies) so byte-stable identity holds.

## Mental model

| TypeScript | SQFts |
|---|---|
| `.ts` | `.sqfts` |
| `.d.ts` | `.d.sqfts` |
| `.js` | `.sqf` |
| `tsc` | `sqfts check` / `sqfts build` |
| `any` | `any` (same gradual semantics) |
| erase types → JS | erase annotations → SQF |
