# Declaring Functions and Globals

`declare` attaches types to symbols that live outside the current file — typically mission globals and `*_fnc_*` functions.

## Function declarations

```sqfts
declare function TAG_fnc_impoundVehicle(_vehicle: object, _fee?: number): boolean;
declare function SERVER_fnc_setMapMarker(_markerId: number): nothing;
declare function TAG_fnc_nearestUnits(): object[];   // nular _this
```

The parameter list describes the `_this` tuple; the return type describes the value of the function body (or `nothing`).

### Call-site checking

```sqfts
private _ok = [_veh, 500] call TAG_fnc_impoundVehicle;   // _ok: boolean
[_veh] call TAG_fnc_impoundVehicle;                      // OK — _fee optional
["a"] call TAG_fnc_impoundVehicle;                       // error STS2003
[23] spawn SERVER_fnc_setMapMarker;                        // scriptHandle
```

### Rules

| Case | Behavior |
|---|---|
| Single required non-array param | Also accepts unwrapped form (`_veh call TAG_fnc_x`), matching SQF’s 1-tuple ↔ bare value convention |
| `spawn` | Checks argument tuple; always returns `scriptHandle` (function return type ignored) |
| `remoteExec` / `remoteExecCall` | Checks args only when the function name is a **string literal** with a declaration; non-literal names stay `any` |
| Definition in project | If the defining `.sqfts` has typed `params`, declaration and definition must agree; trailing expression type must be assignable to the declared return |

`declare function` statements erase entirely. They may appear in `.sqfts` files but conventionally live in [`.d.sqfts`](Declaration-Files).

## Global variable declarations

```sqfts
declare project_playerCash: number;
declare project_groupList: string[];
declare project_debug: boolean;
```

Assignments and reads of a declared global are checked everywhere in the project. Undeclared globals remain `any`. Erased entirely.

## Duplicate declarations

Two declarations for the same symbol must be **identical**. Otherwise `STS1002` names both files.

## Decl vs definition mismatch

When both a declaration and a typed definition exist, disagreement reports `STS2301`.
