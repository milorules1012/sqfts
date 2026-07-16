1.85 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Repeats Code while the given condition is true. A part of while-do construct.

⚠

- A while loop does not have any suspension, meaning that if used in scheduled environment without any suspension (sleep or uiSleep) the code will run multiple times per frame and consumes the 3 ms per frame scheduler execution limit. This should be avoided if not intended (see Example 3).
- In non-scheduled environment, while do loop is limited to 10,000 iterations, after which it exits even if condition is still true.

Groups:  
Program Flow

### Syntax

Syntax:  
while condition

Parameters:  
condition: Code

Return Value:  
While Type

### Examples

Example 1:  
``` sqf
while { a < b } do { a = a + 1 };
```

Example 2:  
A practical example: Repair all members of a group to such a level that they are able to stand up:

``` sqf
{
if (alive _x) then
{
while { not canStand _x } do
{
_x setDamage (damage _x - 0.01);
};
};
} forEach units group unitname;
```

Example 3:  
``` sqf
0 spawn {
// warning: while loop without suspension executes multiple times per frame
private _counter = 0;
private _endTime = diag_tickTime + 5;
private _frameNo = diag_frameNo;
while { diag_tickTime < _endTime } do
{
_counter = _counter + 1;
};
// in an empty mission, the _counter may go well over 2000 times per frame!
hint format ["Average Execution: %1 times per frame", _counter / (diag_frameNo - _frameNo)];
// with suspension
private _counter = 0;
private _endTime = diag_tickTime + 5;
private _frameNo = diag_frameNo;
while { diag_tickTime < _endTime } do
{
_counter = _counter + 1;
uiSleep 0.001; // waits at least 1 frame
};
// _counter says one per frame, as expected
hint format ["Average Execution: %1 times per frame", _counter / (diag_frameNo - _frameNo)];
};
```

### Additional Information

See also:  
Control Structures waitUntil for do

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Kronzky - c  
Posted on May 14, 2008 - 08:40 (UTC)

  
The boolean code that is used to evaluate the while condition can be preceded by code that executes a regular command.

``` sqf
while { _a =_a + 1; _a < 10 } do { /* ... */ };
```
