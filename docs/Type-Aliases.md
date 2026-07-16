# Type Aliases

Give a name to a type expression with `type`.

```sqfts
type moneyTarget = object | group;
type vehicleRow = [string, number, boolean];   // classname, price, insured
type gearList = string[];
```

## Rules

- Statement form: `type Name = TypeExpr;`
- Erased **entirely** (statement, trailing `;`, and one newline)
- Alias names live in their **own namespace** — they cannot collide with variables
- File-scoped unless exported from a [`.d.sqfts`](Declaration-Files) file (then project-visible)

## Contextual keyword

`type` is only a keyword when statement-initial and followed by `Ident = Type ;`. A plain SQF assignment to a global named `type` still parses as SQF:

```sqf
type = 5;   // OK — not a type alias
```

## When to use aliases

- Repeated unions (`object | group`)
- Tuple shapes shared across files (put them in `.d.sqfts`)
- Documenting intent (`feeTier` vs raw `[number, number]`)

For HashMap key sets, prefer [`interface`](Interfaces) over a bare alias to `hashMap`.
