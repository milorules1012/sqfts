1.00 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Selects an element from an array, config entry from Config or substring from a string or a range from an array.

Groups:  
ArraysStringsConfig

### Syntax 1

Syntax:  
array select index

Parameters:  
array: Array

  
index: Number - index 0 denotes the first element, 1 the second, etc. If index has decimal places it gets rounded down for fractions less than or equal .5, otherwise it gets rounded up.  
2.12 negative index can be used to select from the end of the array, i.e. -1 means last array element.

Return Value:  
Anything - a <u>reference</u> to the array element given by its index

⚠

When the index equals the size of the array, there is no error for out of range selection and nil is returned - see Example 8.

### Syntax 2

Syntax:  
array select boolean

Parameters:  
array: Array

  
boolean: Boolean - false selects the **first** element of the Array, true the **second** one

Return Value:  
Anything - a <u>reference</u> to the array element

1.00

### Syntax 3

Syntax:  
config select index

Parameters:  
config: Config

  
index: Number - index 0 denotes the first element, 1 the second, etc. If index has decimal places it gets rounded down for fractions less than or equal .5, otherwise it gets rounded up

Return Value:  
Config

1.28

### Syntax 4

Syntax:  
string select \[start, length\]

Parameters:  
string: String

  
start: Number - string position to start selection from. 0 denotes the first character of the string, 1 the second, etc. If passed number has decimal places it gets rounded down for fractions less than or equal .5, otherwise it gets rounded up

  
length: Number - (Optional, default *string*'s length) number of characters to select

Return Value:  
String

ⓘ

Substring version of select operates with the ANSI charset; if Unicode support is desired, see forceUnicode.

1.32

### Syntax 5

Syntax:  
array select \[start, count\]

Parameters:  
array: Array

  
start: Number - array index to start selection from

  
count: Number - (Optional after 2.14) number of array elements to select. If the selected range exceeds source array boundaries, selection will be made up to the last element of the array.

ⓘ

Since v2.14 'count' is optional, if omitted the selection is made from *start* until the end of the array.

Return Value:  
Array - a <u>new array</u> from selection

1.56

### Syntax 6

Syntax:  
array select expression

Parameters:  
array: Array

  
expression: Code - expression that is expected to return Boolean or Nothing. If true is returned, the original array value of currently tested element \_x will be added to the output array

Return Value:  
Array - a <u>new array</u> of all elements from the original array that satisfied expression condition

### Examples

Example 1:  
``` sqf
["a", "b", "c", "d"] select 2; // result is "c"
position player select 2; // result is Z coordinate of player position
```

Example 2:  
``` sqf
["", currentWeapon player] select alive player; // if player is dead, "" is selected
```

Example 3:  
``` sqf
(configFile >> "cfgVehicles" >> typeOf vehicle player >> "Turrets") select 0 >> "gunnerAction";
```

Example 4:  
``` sqf
hint str ("japa is the man!" select [8]); // the man!
hint str ("japa is the man!" select [0, 7]); // japa is
```

Example 5:  
``` sqf
hint str ([1,2,3,4,5,6] select [1, 4]); // [2,3,4,5]
```

Example 6:  
``` sqf
_even = [1,2,3,4,5,6,7,8,9,0] select { _x % 2 == 0 }; // returns [2, 4, 6, 8, 0]
```

Example 7:  
JavaScript endsWith() alternative:

``` sqf
private _fnc_endsWith =
{
params ["_string", "_endswith"];
_string select [count _string - count _endswith] isEqualTo _endswith
};
["Arma 3", "3"] call _fnc_endsWith; // true
["Arma 3", "4"] call _fnc_endsWith; // false
```

Example 8:  
select index traps:

``` sqf
private _array = ["a", "b", "c", "d"];
_array select 0; // "a"
_array select 3; // "d"
_array select 4; // nil - no error shown
_array select 5; // error
// can sometimes be useful
private _firstEnemyNearMe = allUnits opfor select { player distance _x < 10 } select 0; // nil if no enemies nearby
if (isNil "_firstEnemyNearMe") exitWith { systemChat "no enemy found" };
systemChat format ["enemy found: %1", name _firstEnemyNearMe];
// get the last element properly
_array select (count _array); // wrong - nil is returned
_array select (count _array - 1); // correct - "d" is returned
```

Example 9:  

Since 2.12 `-1` can be used to select the last element of an array.

``` sqf
private _array = [1, 2, 3, 4, 5];
hint str (_array select -1); // 5
```

### Additional Information

See also:  
\# selectRandom selectRandomWeighted set resize reverse in find findIf toArray toString forEach count deleteAt deleteRange append sort param params splitString joinString pushBack pushBackUnique apply forceUnicode

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

General Barron - c  
Posted on Mar 03, 2009 - 02:02 (UTC)

  
When combined with the count command, this can be used to read all entries out of a config; even when you don't know exactly how many entries there will be. See the notes under count for more info.

<!-- -->

Killzone_Kid - c  
Posted on Sep 28, 2013 - 00:53 (UTC)

  
Rounding of fractions with select is not the same as when you use round command:

``` sqf
_roundThis = 0.5;
hint str ([0,1] select _roundThis); // 0
hint str ([0,1] select round _roundThis); // 1
```

<!-- -->

Pierre MGI - c  
Posted on Jul 14, 2016 - 04:18 (UTC)

  
You can substract array from array using select:

``` sqf
_array = [[1], [2], [3]];
_sub = [2];
_array - _sub; // [[1], [2], [3]]
_array select { !(_x isEqualTo _sub) }; // [[1], [3]]
[[1],[2],[2],[2],[2],[3]] select { !(_x isEqualTo _sub) }; // [[1], [3]]
```

<!-- -->

Commy2 - c  
Posted on Nov 12, 2016 - 22:36 (UTC)

  
It is not safe to escape the code block of alternative syntax \#5 with exitWith, breakOut etc:

``` sqf
x3 = [1,2,3,4,5] select {
if (_x == 3) exitWith
{
false;
};
true
};
// could be expected to be: x3 = [1,2,4,5]
// actual result: x3 = false
```

<!-- -->

Igneous01 - c  
Posted on Feb 14, 2017 - 16:26 (UTC)

  
Syntax 5 is the equivalent of passing in a predicate that returns a boolean. In SQF, a piece of code will always return what the last executed command returned.

``` sqf
myAliveUnits = allUnits select { alive _x }; // alive returns a boolean, the last statement run was alive _x,
// therefore this piece of code will return a true/false to the select command
myEastGroups = allGroups select { side _x == east }; // returns all east groups
my4ManGroups = allGroups select { count units _x == 4 }; // returns all groups that have 4 men in them
unitsThatDetectedMe = allUnits select { _x knowsAbout player > 0.1 }; // returns a list of units that have detected the player
```
