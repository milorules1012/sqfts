# Casts

Use `as` to assert a type at compile time. Casts erase completely — no runtime check.

## Syntax

```sqfts
private _crew = (_this select 0) as object[];
private _pos = (getMarkerPos _mk) as positionATL;
```

## Rules

1. **Compile-time only** — `expr as T` erases to `expr` (the ` as T` token run is deleted).
2. **Precedence** — `as` has **lower precedence than any binary command**, so:
   ```sqfts
   _this select 0 as string[]
   ```
   means `(_this select 0) as string[]`. Parenthesize for clarity anyway.
3. **Overlap required** — a cast is legal only between types that overlap (either direction of assignability, or via `any`). `"abc" as number` is an error (`STS2108`); go through `as any` if you really mean it.
4. **Contextual keyword** — `as` is only a keyword in postfix expression position followed by a type expression. A global variable named `as` in plain SQF still parses.

## When to cast

- After `select` / `#` on untyped arrays when you know the element type
- Crossing brand boundaries when you intentionally discard coordinate-space safety
- Converting `hashMap` → interface after validating keys yourself
- Escaping gradual holes until richer narrowing exists

## Anti-patterns

Prefer engine converters over brand casts when possible:

```sqfts
// Prefer
private _asl = ATLToASL (getPosATL player);

// Avoid unless necessary
private _asl = (getPosATL player) as positionASL;
```
