# FAQ

## Is SQFts a new language?

It is a **superset** of Arma 3 SQF: every valid SQF file is valid SQFts. Annotations are optional and erased. Think TypeScript, not a separate VM language.

## Does typed code run slower?

No. Default erasure removes all type syntax. The only opt-in runtime effect is [`emit_runtime_params`](Erasure), which emits the same kind of `params` guards you would write by hand.

## Do I have to rename every `.sqf`?

No. Prefer [`.d.sqfts` declarations](Declaration-Files) and `check_plain_sqf` first. Rename to `.sqfts` only where you want inline annotations.

## What about `BIS_fnc_*`?

Engine **commands** are in the [command database](Engine-Command-Database). Scripted functions (`BIS_fnc_*`, mission `*_fnc_*`) need [declarations](Declaring-Functions-and-Globals) — generate skeletons with [declgen](Declgen) or write them by hand.

## Where do engine command types come from?

From [arma3-wiki](https://github.com/acemod/arma3-wiki), loaded the same way HEMTT does (git refresh of the `dist` data, with an embedded snapshot as fallback). There is no separate COMREF extract step.

## Why is `code` opaque?

v1 does not yet track `_this` / return types of code blocks. Parameterized `code(…)` types are [future work](Future-Work). Code literals are still type-checked internally as scopes.

## Why did `positionATL` reject my `positionASL`?

Brands prevent mixing coordinate spaces. Convert with engine commands (`ATLToASL`, …) or cast deliberately. Or set `no_position_brands = true` if you do not want this rigor.

## Can macros inject types?

Checking sees post-preprocess code, but **erasure** runs on unpreprocessed source in v1. Annotations must appear literally in `.sqfts` files. See [Grammar](Grammar).

## Where is the normative spec?

The [language specification](https://github.com/milorules1012/sqfts/blob/main/docs/design-history/language-specification.md) in the design-history archive. This handbook is the wiki-oriented guide; the specification wins on conflicts.

## What license is the toolchain?

GPL-2.0 (HEMTT-derived). See the repository license file.

## How do I get live errors in the editor?

Install the [VS Code / Cursor extension](Editor-Support) and ensure `sqfts-language-server` is available (bundled or via `sqfts.serverPath`).
