# SQFts Handbook

SQFts is a gradually-typed **superset of Arma 3 SQF**, in the same relationship to SQF as TypeScript is to JavaScript.

- **Superset** — every valid SQF file is a valid SQFts file with identical meaning.
- **Erasable** — type annotations are checked at compile time, then stripped. Emitted `.sqf` has no runtime type system (unless you opt into runtime `params` guards).
- **Gradual** — untyped code means `any`, not an error. Add types file by file, or via `.d.sqfts` declaration files without touching sources.

```sqfts
params [
    "_vehicle": object,
    "_fee": number = 0
];

private _owner: object = _vehicle getVariable ["project_owner", objNull];
[_owner, _fee] call TAG_fnc_checkPayment
```

Erases to plain SQF:

```sqf
params [
    "_vehicle",
    ["_fee", 0]
];

private _owner = _vehicle getVariable ["project_owner", objNull];
[_owner, _fee] call TAG_fnc_checkPayment
```

---

## Handbook

### Start here

| Page | Description |
|---|---|
| [Getting Started](Getting-Started) | Install the toolchain and check your first project |
| [Configuring Your Project](Configuring-Your-Project.md) | End-to-end setup for an existing Arma project |
| [Basic Concepts](Basic-Concepts) | Superset, erasure, gradual typing, file extensions |
| [Everyday Types](Everyday-Types) | Primitives, `any`, `nothing`, `nil` |
| [Arrays, Tuples, and Brands](Arrays-Tuples-and-Brands) | `T[]`, tuples, positions, waypoints, colors |
| [Unions and Narrowing](Unions-and-Narrowing) | `A \| B` and `isNil` / `isEqualType` |
| [Literal Types](Literal-Types) | String and numeric literal refinements |

### Syntax

| Page | Description |
|---|---|
| [Annotating Variables](Annotating-Variables) | Typed `private` |
| [Typed Params](Typed-Params) | Compile-time `params` entries |
| [Type Aliases](Type-Aliases) | `type Name = …` |
| [Interfaces](Interfaces) | Typed HashMaps |
| [Casts](Casts) | `expr as T` |
| [Declaring Functions and Globals](Declaring-Functions-and-Globals) | `declare function` / `declare` |
| [Declaration Files](Declaration-Files) | `.d.sqfts` ambient types |

### Toolchain

| Page | Description |
|---|---|
| [Type Checking](Type-Checking) | Where types come from; engine overloads |
| [Strictness Flags](Strictness-Flags) | `noImplicitAny`, `strictNil`, … |
| [Erasure](Erasure) | How annotations become plain SQF |
| [Configuration](Configuration) | `sqfts.toml` |
| [CLI Reference](CLI-Reference) | `check` / `build` / `declgen` |
| [Diagnostics](Diagnostics) | Stable `STSxxxx` codes |
| [Editor Support](Editor-Support) | VS Code / Cursor / LSP |
| [Declgen](Declgen) | Generate skeletons from `Functions.h` |
| [Engine Command Database](Engine-Command-Database) | arma3-wiki engine command types |

### Reference & guides

| Page | Description |
|---|---|
| [Grammar](Grammar) | Formal grammar deltas over SQF |
| [Architecture](Architecture) | Crates and pipeline |
| [Adoption Guide](Adoption-Guide) | Typing an existing mission without rewriting it |
| [Future Work](Future-Work) | Non-normative roadmap |
| [FAQ](FAQ) | Common questions |

The normative [language specification](design-history/language-specification.md) is preserved with the project’s design history. This handbook is the human-oriented guide; if a handbook page and the specification disagree, **the specification wins**.
