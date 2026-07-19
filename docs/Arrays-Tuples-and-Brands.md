# Arrays, Tuples, and Brands

SQF arrays are the workhorse data structure. SQFts distinguishes untyped arrays, homogeneous arrays, fixed-shape tuples, and **branded** array types for positions and similar concepts.

## Untyped arrays

```sqfts
private _a: array = [];   // same as any[]
```

## Homogeneous arrays

```sqfts
private _names: string[] = ["a", "b"];
private _matrix: number[][] = [[1, 2], [3, 4]];
private _units: object[] = [];
```

Corresponds to the engine database’s `ArrayOf(T)`.

## Tuples

Fixed-shape lists. Trailing elements may be optional with `?`:

```sqfts
type vehicleRow = [string, number, boolean];
type rgba = [number, number, number, number?];

private _row: [string, number] = ["B_MRAP_01_F", 5000];
```

### Assignability

| From | To | OK? |
|---|---|---|
| `[number, number, number]` | `number[]` | Yes |
| `number[]` | `array` / `any[]` | Yes |
| `number[]` | `[number, number, number]` | No — needs a [cast](Casts) |
| `[string, number]` | `string[]` | No (element types differ) |

Array literals infer as tuples when every element type is known, and decay to `T[]` / `array` on assignment as needed.

## Branded array types

Several engine concepts are plain arrays at runtime but semantically distinct. SQFts models them as **brands**: structurally they are number tuples (or `[group, number]` for waypoints), but the brand is tracked nominally so mixing coordinate spaces is an error.

| Type | Runtime shape |
|---|---|
| `position2D` | `[number, number]` |
| `position3D` | `[number, number, number]` (space unspecified) |
| `positionATL` | `[number, number, number]` |
| `positionASL` | `[number, number, number]` |
| `positionASLW` | `[number, number, number]` |
| `positionAGL` | `[number, number, number]` |
| `positionAGLS` | `[number, number, number?]` (Z optional) |
| `positionRelative` | `[number, number, number]` |
| `position` | Union of the above (wiki “Position”) |
| `vector2D` | `[number, number]` |
| `vector3D` | `[number, number, number]` |
| `waypoint` | `[group, number]` |
| `color` | `[number, number, number, number?]` |
| `turretPath` | `number[]` |
| `treePath` | `number[]` |
| `unitLoadout` | branded `array` |

### Rules

1. A **literal** tuple of the right shape is assignable to any matching brand — fresh literals carry no brand:
   ```sqfts
   private _p: positionATL = [0, 0, 0];   // OK
   ```
2. A branded value is assignable to its structural shape:
   ```sqfts
   private _raw: number[] = getPosASL player;   // positionASL → number[]
   ```
3. One brand is **not** assignable to another:
   ```sqfts
   private _atl: positionATL = getPosATL player;
   private _asl: positionASL = _atl;   // error STS2107
   ```
   Convert with engine commands (`ATLToASL`, `getPosASL`, …) or [cast](Casts).
4. Bare `position3D` accepts any 3D positional brand; specific brands accept `position3D` only under a cast (escape hatch, one step stricter than `any`).

Disable brands entirely with [`no_position_brands`](Strictness-Flags) — they collapse to structural shapes.

## Operators

Array operators (`select`, `#`, `+`, …) are typed like other engine commands. When the array element type is known, `_arr # 0` yields that element type.

v1 still has soundness holes around mutation (`pushBack`, `set`, …) and collection commands accepting non-arrays; see [Missing Features](Missing-Features) (bundle **B-ArraySoundness**).
