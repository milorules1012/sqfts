# Literal Types

String and numeric literal types refine unions and engine-command parameters beyond bare `string` / `number`.

## Syntax

```sqfts
private _side: "west" | "east" = "west";
private _flags: 0 | 1 | 2 | 4 | 8 = 1;
type compass = "N" | "E" | "S" | "W";
```

Literal atoms use the same quoting and number spellings as SQF value literals (`"…"`, `'…'`, optional `-` prefix, optional fractional part).

## Assignability

| From | To | OK? |
|---|---|---|
| `"west"` | `"west" \| "east"` | Yes |
| `"WEST"` | `"west"` | Yes (case-insensitive) |
| `"west"` | `string` | Yes |
| `string` | `"west"` | No — use [cast](Casts) |
| `1` | `0 \| 1 \| 2` | Yes |
| `3` | `0 \| 1 \| 2` | No |

Union rules from [Unions and Narrowing](Unions-and-Narrowing) apply unchanged.

## Inference and widening

Expression literals infer as literal types:

```sqfts
private _x = "west";   // inferred as string (widened at definition)
_x = "east";           // OK — unannotated locals widen to string
```

Annotated locals keep the declared type:

```sqfts
private _mode: "west" | "east" = "west";
_mode = "north";       // error
```

## Engine database

Wiki `StringEnum` / `NumberEnum` values (e.g. `showIn3D` on `addAction`) map to literal unions in the command database when the wiki data provides them.
