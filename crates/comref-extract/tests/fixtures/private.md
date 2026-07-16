1.00 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Sets a variable to the innermost scope (see Example 3). See also param and params.

ⓘ

private variables **must** start with an underscore: `private`` `**`_`**`myVar1 = "myVar";` - see Identifier.

⚠

**Always** make your local variables **private** (through private or params) in order to avoid overwriting a local variable of the same name.

Use local in Arma 2.

Groups:  
Variables

### Syntax 1

Syntax:  
private variableName

Parameters:  
variableName: String - e.g `"_myVar"`

Return Value:  
Nothing

### Syntax 2

Syntax:  
private variableNameList

Parameters:  
variableNameList: Array of Strings - e.g `["_target", "_damage"]`

Return Value:  
Nothing

1.54

### Syntax 3

Syntax:  
private \_identifier = value

Parameters:  
\_identifier: underscored variable name, for example `_myVar`

  
value: Anything: value to assign to the variable

Return Value:  
Nothing

### Examples

Example 1:  
``` sqf
private _varname = "this is my new variable"; // since Arma 3 v1.54
// identical, but less performant
private "_varname";
_varname = "this is my new variable";
```

Example 2:  
``` sqf
private ["_varname1", "_varname2"];
_varname1 = "variable 1";
_varname2 = "variable 2";
```

Example 3:  
``` sqf
_lol = 123; call { hint str [_lol] }; // [123]
_lol = 123; call { private "_lol"; hint str [_lol] }; // [any]
```

Example 4:  
``` sqf
_myvar = 123;
systemChat str [_myvar]; // [123]
call {
systemChat str [_myvar]; // [123]
private "_myvar";
systemChat str [_myvar]; // [any]
_myvar = 345;
systemChat str [_myvar]; // [345]
};
systemChat str [_myvar]; // [123]
```

### Additional Information

See also:  
param params privateAll Variables - Scopes

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Faguss - c  
Posted on Aug 04, 2010 - 13:30 (UTC)

  
The higher scope is also the script from which the function has been called.  
in **script2.sqf**:

``` sqf
_a = 2;
```

in **script1.sqf**:

``` sqf
_a = 1;
call compile preprocessFileLineNumbers "script2.sqf";
hint format ["%1", _a];
```

Game will display 2.  
Inserting `private "_a"` in the function prevents the change and so number 1 will be displayed on the screen.

<!-- -->

DreadedEntity - c  
Posted on Feb 25, 2015 - 17:06 (UTC)

  
Recursive loops require the use of private. Without it, your variables will be overwritten.

<!-- -->

654wak654 - c  
Posted on Jan 31, 2018 - 10:37 (UTC)

  
This command is *similar* to javascript's let keyword.  
**EDIT:** in the way that it scopes the variable to the innermost scope. Otherwise, let and private can behave differently - Lou Montana (talk)
