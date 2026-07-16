1.00 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Executes the given command(s) on every item of an Array or a HashMap.

Since Arma 2, the variable \_x is always private to the forEach block so it is safe to nest them (see Example 8).

Groups:  
Program FlowArraysHashMap

### Syntax

Syntax:  
code forEach array

Parameters:  
<table class="wikitable align-center float-right">
<tbody>
<tr>
<th>Game</th>
<td> 1.00</td>
<td> 1.00</td>
<td> 1.00</td>
<td> 1.50</td>
<td> 1.00</td>
<td> 1.00</td>
</tr>
<tr>
<th>String support</th>
<td colspan="2"></td>
<td colspan="4"></td>
</tr>
<tr>
<th>Code support</th>
<td></td>
<td colspan="5"></td>
</tr>
</tbody>
</table>

code: String only in Operation Flashpoint and Armed Assault, 1.00 Code since Armed Assault - available variables:

- `_x`: iterated item
-  1.55 `_forEachIndex`: item's index

  
array: Array - the array to iterate over

Return Value:  
Anything - will return the value of last executed statement

2.02

### Alternative Syntax

Syntax:  
code forEach hashMap

Parameters:  
code: Code - code applied to each key-value pair - available variables:

- `_x`: key
- `_y`: value
- `_forEachIndex`: iteration number

  
hashMap : HashMap - the HashMap to iterate over

Return Value:  
Anything - will return the value of last executed statement

### Examples

Example 1:  
``` sqf
// SQF
{ _x setDamage 1 } forEach units player;
```

``` sqf
; SQS
"_x setDammage 1" forEach units player
```

Example 2:  
This command can also easily be used to execute a single command multiple times without respect to the array items - see also for

``` sqf
{ player addMagazine "30Rnd_556x45_Stanag" } forEach [1, 2, 3, 4];
// equivalent to
for "_i" from 1 to 4 do { player addMagazine "30Rnd_556x45_Stanag" };
```

Example 3:  
You can also use multiple commands in the same block:

``` sqf
{
_x setCaptive true;
removeAllWeapons _x;
doStop _x;
} forEach units group this;
```

Example 4:  
To get the index of a forEach loop, use \_forEachIndex:

``` sqf
{ systemChat str _forEachIndex; } forEach ["a", "b", "c"]; // will return: "0", "1", "2" in systemChat messages
```

Example 5:  
Iterating a HashMap's \_forEachIndex:

``` sqf
// shows "0, k1, v1", "1, k2, v2" in systemChat messages
{
systemChat format ["%1, %2, %3", _forEachIndex, _x, _y];
} forEach createHashMapFromArray [
["k1", "v1"],
["k2", "v2"]
];
```

Example 6:  
findIf equivalent for HashMap:

``` sqf
private _resultKey = {
if (_y isEqualTo "wantedValue") exitWith { _x };
""
} forEach _hashmap;
```

Example 7:  
Array is edited by reference:

``` sqf
_arr1 = [1,2,3];
_arr2 = [6,7,8];
_arr3 = [0];
{ _x set [1, "changed"] } forEach [_arr1, _arr2, _arr3];
// _arr1 = [1, "changed", 3]
// _arr2 = [6, "changed", 8]
// _arr3 = [0, "changed"]
```

Example 8:  
``` sqf
{
private _verticalValue = _x; // needed, otherwise _horizontalValues' _x made this one inaccessible
{
[_x, _verticalValue] call TAG_fnc_doSomething;
} forEach _horizontalValues;
} forEach _verticalValues;
```

### Additional Information

See also:  
Control Structures for apply while select findIf count forEachReversed

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Dedmen - c  
Posted on Nov 28, 2017 - 13:46 (UTC)

  
Be careful when deleting (deleteAt) elements from an Array while you iterate over it.  
\_forEachIndex will not move to reflect your change.  
The forEach code is doing the same as

``` sqf
private _forEachIndex = 0;
while { _forEachIndex < count _array } do
{
(_array select _forEachIndex) call code;
_forEachIndex = _forEachIndex + 1;
};
```

So if you delete your current element from the array the other elements will shift forward. Meaning you skip one element.  
Example:

``` sqf
_array = [1,2,3,4,5,6];
{ _array deleteAt _forEachIndex } forEach _array;
```

After the first iteration your Array will be \[2,3,4,5,6\] and the \_forEachIndex will be 1.  
So on next iteration you get the element at index 1 which will be 3. So you've just skipped the 2.  
So in the end you will only iterate over 1, 3 and 6.

<!-- -->

Sa-Matra - c  
Posted on Apr 02, 2023 - 09:04 (UTC)

  
Use new forEachReversed command for deleting array items with deleteAt. Check its examples for details.
