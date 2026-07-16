1.00 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Can be used to count:

- The number of elements in an array (returns the already internally known array size)
- The number of elements in an array matching the condition
- The number of sub-entries in a config entry
-  1.28 The number of characters in an ANSI string
  ⓘ
  If Unicode support is desired, see forceUnicode.

Groups:  
ArraysStringsConfigHashMap

### Syntax

Syntax:  
count value

Parameters:  
value: Array, String, Config or HashMap

Return Value:  
Number

### Alternative Syntax

Syntax:  
condition count array

Parameters:  
condition: Code - condition that must return true for the tested element to be counted. The variable `_x` will contain the currently tested element

If the provided array contains different data types, use isEqualTo for item comparison instead of ==.

  
array: Array

Return Value:  
Number

### Examples

Example 1:  
``` sqf
count [0, 0, 1, 2]; // returns 4
count units group player; // returns number of units in player group
```

Example 2:  
``` sqf
private _cnt = { _x == 4 } count [1, 9, 8, 3, 4, 4, 4, 5, 6]; // returns 3
_cnt = { alive _x } count allUnits; // returns the number of alive units
```

Example 3:  
``` sqf
private _cnt = count (configFile >> "CfgVehicles");
```

Example 4:  
``` sqf
hint str count "japa is the man!"; // 16
```

Example 5:  
``` sqf
hint format ["There are %1 elements in the provided hashmap", count _myHashMap];
```

### Additional Information

See also:  
apply select in find countFriendly countEnemy countUnknown countSide countType findIf forceUnicode

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Hardrock - c  
Posted on Aug 03, 2006 - 14:27 (UTC)

  
*Notes from before the conversion:*  
Use this to calculate how many "M16" mags a soldier has left.

``` sqf
{ _x == "M16" } count magazines soldier1;
```

Take care when using count to determine how many units are left alive in a group: count units group player or count units groupname Will return the number of units the leader of the group thinks are alive. If some units have been killed out of sight of other members of the group then it may take sometime for this to be the actual numbers in the group. To determine exactly how many units are really alive in a group use:

``` sqf
{ alive _x } count units group player;
```

or

``` sqf
{ alive _x } count units groupname;
```

<!-- -->

Heeeere's Johnny! - c  
Posted on Dec 15, 2014 - 00:01 (UTC)

  
*count* can be (ab)used for a very fast and simple check if at least one element in an array fulfills a certain condition:

``` sqf
if ({ if (/* _x fulfills condition */) exitWith {1}; false } count _array isEqualTo 1) then
{
// do whatever here
};
```

This code will exit the *count* loop as soon as it finds an element fulfilling the condition, leaving the *count* with the value of 1, hence make the larger if-condition be *true*.  
If no array element fulfills the condition, the *count* will be 0 and the if-condition will be *false*.

<!-- -->

Killzone_Kid - c  
Posted on Dec 29, 2014 - 21:23 (UTC)

  
Quit loop at first fulfilled condition (same as above but faster):

``` sqf
{
if (_x == 4) exitWith {
// do something when we reach 4
}
} count [1,2,3,4,5,6];
```

<!-- -->

Heeeere's Johnny! - c  
Posted on Jan 02, 2015 - 22:32 (UTC)

  
Using exitWith inside a **count** loop will overwrite the default functionality and make **count** return whatever the **exitWith** returns:

``` sqf
_result = {
if (_x isEqualTo 3) exitWith { "Hello" }
} count [1,2,3,4,5];
// _result = "Hello"
```

<!-- -->

Ebay - c  
Posted on Aug 22, 2016 - 19:41 (UTC)

  
With the alternative syntax each iteration should result in an interior return of bool or nothing. Example:

``` sqf
createDialog "RscFunctionsViewer";
{ lbAdd [292901, _x]; } count ["first", "second", "third"];
```

lbAdd returns a number, so this throws "Error Type Number, expected Bool". Tested in A2OA 1.63.131129
