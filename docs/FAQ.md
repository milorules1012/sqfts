# FAQ

## Is SQFts a new language?

It is a **superset** of Arma 3 SQF: every valid SQF file is valid SQFts. Annotations are optional and erased. Think TypeScript, not a separate VM language.

## Does typed code run slower?

No. Erasure removes all type syntax. With [`emit_runtime_params`](Erasure) (on by default), typed `params` also emit the same kind of guard arrays you would write by hand — set it to `false` for type-stripping only.

## Do I have to rename every `.sqf`?

No. Prefer [`.d.sqfts` declarations](Declaration-Files) and `check_plain_sqf` first. Rename to `.sqfts` only where you want inline annotations.

## What about `BIS_fnc_*`?

Engine **commands** are in the [command database](Engine-Command-Database). Scripted functions (`BIS_fnc_*`, mission `*_fnc_*`) need [declarations](Declaring-Functions-and-Globals) — generate skeletons with [declgen](Declgen) or write them by hand.

## Where do engine command types come from?

From [arma3-wiki](https://github.com/acemod/arma3-wiki), loaded the same way HEMTT does (git refresh of the `dist` data, with an embedded snapshot as fallback). There is no separate COMREF extract step.

## How does parameterized `code` work?

Use `code(unit: object): boolean` (and similar) to describe `_this` and the return type. Param names document the `_this` tuple; the checker binds only `_this` when checking a `{ … }` literal against that expected type. Bare `code` remains gradual (assignable both ways). The return uses `:` like [`declare function`](Declaring-Functions-and-Globals). See [Everyday Types](Everyday-Types).

## Why did `positionATL` reject my `positionASL`?

Brands prevent mixing coordinate spaces. Convert with engine commands (`ATLToASL`, …) or cast deliberately. Or set `no_position_brands = true` if you do not want this rigor.

## Can macros inject types?

Yes. Checking runs after the HEMTT preprocessor, so annotations that appear only after macro expansion (including from `#include`d headers) are scanned, erased from the processed buffer for parsing, and used for type-checking.

Emit stays byte-local on unpreprocessed files (E1–E3). Annotations written in a `#define` body are erased there when that file is built. Purely synthetic expansion text with no original token span is type-checked but not rewritten at the call site — put the annotation text in the `#define` body so emit can strip it.

Relative `#include` paths resolve from the file under the project root; set `include_paths` in `sqfts.toml` (or keep a top-level `include/`) for include-layer headers as `#include "\header.h"`. See [Configuration](Configuration).

## Where is the normative spec?

The [language specification](https://github.com/milorules1012/sqfts/blob/main/docs/design-history/language-specification.md) in the design-history archive. This handbook is the wiki-oriented guide; the specification wins on conflicts.

## What license is the toolchain?

GPL-2.0 (HEMTT-derived). See the repository license file.

## How do I get live errors in the editor?

Install the [VS Code / Cursor extension](Editor-Support) and ensure `sqfts-language-server` is available (bundled or via `sqfts.serverPath`).
