# Annotating Variables

Typed `private` is the most common inline annotation.

## Forms

```sqfts
// With initializer
private _tries: number = 0;
private _target: object | group = objNull;
private _names: string[] = [];

// Declare-then-assign (type without initializer)
private _result: string;
if (_ok) then {
    _result = "yes";
} else {
    _result = "no";
};
```

## Erasure

```sqf
private _tries = 0;
private _target = objNull;
private _names = [];

private "_result";
if (_ok) then {
    _result = "yes";
} else {
    _result = "no";
};
```

Notes:

- With `=`, the `: Type` span is deleted (plus one preceding space if present).
- Without `=`, `private _result: string;` rewrites to the string-form declaration `private "_result";` — the only valid SQF spelling of declare-without-assign. This is the one place typed `private` erasure is a rewrite rather than a pure deletion.

## Where annotations are recognized

The annotation is only recognized **immediately after a `private` local** (or inside typed [`params`](Typed-Params)). A colon anywhere else is plain SQF (`case` labels, `switch` `:` operator).

## Inference without annotation

```sqfts
private _u = vehicle player;   // inferred object
```

A variable’s declared or inferred type is **fixed for its scope**. Assigning an incompatible value is an error (`any` is always compatible). Reassignment does not re-infer.

Shadowing via a new `private` in an inner scope creates a fresh variable, matching SQF semantics.

## HEMTT lint alignment

Erasure targets the forms HEMTT’s `not_private` lint expects (`private _x = v` / `private "_x"`), so annotated code stays lint-clean after build.
