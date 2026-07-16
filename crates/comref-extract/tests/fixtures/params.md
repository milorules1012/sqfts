1.48

### Description

Description:  
Parses input argument into array of private variables. When used without argument, as shown in main syntax, internal variable \_this, which is usually available inside functions and event handlers, is used as argument.  
  
In addition to simple parsing directly into variables, input can be tested in case it is undefined, of the wrong type or of the wrong size (if array) and substituted if necessary with default values. Since Arma 3 v1.54, onscreen errors are displayed for when the input is of the wrong type or size.

⚠

All variables names must start with underscore and be enclosed in quotes: `params`` [``"_``myVar1``"``, ``"_``myVar2``"``];`

ⓘ

It is a good practice to make your local variables **private** (through private or params) in order to avoid overwriting a local variable of the same name.

Groups:  
VariablesArrays

### Syntax

Syntax:  
params \[element1, element2, ...\]

Parameters:  
elementN: String or Array

- String:name of the private variable (must begin with underscore \_, e.g. "\_myVar", or can be an empty string "", in which case the element is skipped)
- Array format \[variableName, defaultValue, expectedDataTypes, expectedArrayCount\]:
  - variableName: String - name of a private variable (must begin with underscore \_, e.g. "\_myVar")
  - defaultValue: Anything - default value to return if input element is undefined, of the wrong type or of the wrong size (if array).
  - expectedDataTypes: Array of direct Data Types - (Optional) checks if passed value is one of listed Data Types. If not, default value is used instead. Empty array \[\] means every data type is accepted.
  - expectedArrayCount: Number or Array - (Optional) a single size or array of sizes. If passed input value is an array, checks that it has a certain number of elements. If not, default value is used instead. Empty array \[\] means any size is accepted.

Return Value:  
Boolean - false if error occurred or default value has been used, otherwise true

### Alternative Syntax

Syntax:  
argument params \[element1, element2, ...\]

Parameters:  
argument: Anything - a usual array of params is expected. If a non-array argument is passed, it will be converted to 1 element array

  
elementN: String or Array

- String:name of the private variable (must begin with underscore \_, e.g. "\_myVar", or can be an empty string "", in which case the element is skipped)
- Array format \[variableName, defaultValue, expectedDataTypes, expectedArrayCount\]:
  - variableName: String - name of a private variable (must begin with underscore \_, e.g. "\_myVar")
  - defaultValue: Anything - default value to return if input element is undefined, of the wrong type or of the wrong size (if array).
  - expectedDataTypes: Array of direct Data Types - (Optional) checks if passed value is one of listed Data Types. If not, default value is used instead. Empty array \[\] means every data type is accepted.
  - expectedArrayCount: Number or Array - (Optional) a single size or array of sizes. If passed input value is an array, checks that it has a certain number of elements. If not, default value is used instead. Empty array \[\] means any size is accepted.

Return Value:  
Boolean - false if error occurred or default value has been used, otherwise true

### Examples

Example 1:  
``` sqf
[1, 2, 3] call {
private ["_one", "_two", "_three"];
_one = _this select 0;
_two = _this select 1;
_three = _this select 2;
// ...
};
// Same as above, only using params
[1, 2, 3] call {
params ["_one", "_two", "_three"];
// ...
};
```

Example 2:  
``` sqf
[123] call {
params ["_myvar"];
};
// Below would produce the same result as above
123 call {
params ["_myvar"];
};
```

Example 3:  
Skipping some array elements:

``` sqf
position player params ["", "", "_z"];
if (_z > 10) then {
hint "YOU ARE FLYING!";
};
```

Example 4:  
``` sqf
[1, nil, 2] params ["_var1", "_var2", "_var3"];
// All 3 variables are made private but only _var1 and _var3 are defined
[1, nil, 2] params ["_var1", ["_var2", 23], "_var3"];
// All 3 variables are private and defined
```

Example 5:  
``` sqf
[1, 2] call {
if (!params ["_var1", "_var2", ["_var3", true, [true]]]) exitWith {
hint str [_var1, _var2, _var3];
};
};
// The hint shows [1,2,true]
// Script exits, default value was used due to missing value
[1, 2, 3] call {
if (!params ["_var1", "_var2", ["_var3", true, [true]]]) exitWith {
hint str [_var1, _var2, _var3];
};
};
// The hint shows [1,2,true]
// Script exits, default value was used due incorrect value type
```

Example 6:  
``` sqf
[1, "ok", [1, 2, 3]] call {
if (!params [
["_var1", 0, [0]],
["_var2", "", [""]],
["_var3", [0,0,0], [[], objNull, 0], [2,3]]
]) exitWith {};
hint "ok";
};
// Passes validation
[1, 2, [3, 4, 5]] call {
if (!params ["_var1", "_var2", ["_var3", [], [[], objNull, 0], 0]]) exitWith {};
hint "ok";
};
// Fails, because passed array is expected to be of 0 length, i.e. empty
```

Example 8:  
``` sqf
[1, 2, 3, [4, 5, 6]] call {
params ["_one", "_two", "_three"];
_this select 3 params ["_four", "_five", "_six"];
};
```

Example 9:  
``` sqf
{
_x params ["_group", "_index"];
// ...
} forEach waypoints group player;
fn_someFnc = {
params ["_position", ["_direction", 0], ["_name", ""]];
// Extract the x, y, and z from "_position" array:
_position params ["_x", "_y", "_z"];
// ...
};
[position player, direction player, name player] call fn_someFnc;
```

Example 10:  
``` sqf
player addEventHandler ["HitPart", {
_this select 0 params ["_target", "_shooter", "_projectile"];
}];
```

### Additional Information

See also:  
param select \# \_this isEqualTypeAll isEqualType isEqualTypeParams isEqualTypeArray isEqualTypeAny

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Dedmen - c  
Posted on Nov 03, 2016 - 04:07 (UTC)

  
With a function only taking one Parameter, it doesn't matter whether the parameter is in an array or not:  
Example:

``` sqf
1 call {
params [["_number",0, [0]]];
};
```

or

``` sqf
[1] call {
params [["_number",0, [0]]];
};
```

But when the one Parameter is an array that parameter has to be inside of an array when the function is called  
Example:

``` sqf
[1, 2] call {
params [["_array", [], [[]], 2]];
}; // Fails
[[1,2]] call {
params [["_array", [], [[]], 2]];
}; // Succeeds
```

<!-- -->

7erra - c  
Posted on Jul 04, 2019 - 16:54 (UTC)

  
It is valid to redefine the \_this variable and use params again like this:

``` sqf
[1, 2, [3, 4]] call {
params ["_one", "_two", "_this"];
params ["_three", "_four"];
};
```

<!-- -->

AgentRev - c  
Posted on Nov 01, 2021 - 16:00 (UTC)

  
Here's how to validate HashMap parameters:

``` sqf
_myHashMap = createHashMapFromArray [["a",1],["b",2],["c",3]];
[_myHashMap] call {
params [["_theHashMap",createHashMap,[createHashMap]]];
};
```
