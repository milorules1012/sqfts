1.85 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Adds given set of compiled instructions to the current stack and waits for it to finish and return, provides an option to pass arguments to the executed Code. See Scheduler to learn more about how the code is executed and behaves.

Groups:  
Program Flow

### Syntax 1

Syntax:  
call code

Parameters:  
code: Code - compiled instructions

Operation Flashpoint takes String.

ⓘ

This way of calling has access to its parent scope's `_this`, as all it does is creating a scope.

Return Value:  
Anything - the last value given in the function is returned - see the topic Function for more information

### Syntax 2

Syntax:  
args call code

Parameters:  
args: Anything - arguments that are passed to the function in the `_this` variable.

  
code: Code - compiled instructions

Operation Flashpoint takes String.

Return Value:  
Anything - the last value given in the function is returned. See the topic Function for more information.

2.14

### Syntax 3

Syntax:  
hashMapObj call \[methodName, arguments\]

Parameters:  
hashMapObj: HashMap - the HashMap (not necessarily created through createHashMapObject) to call the method on

  
methodName: String - the name of the method to call (the method must be defined in *hashMapObj*; see createHashMapObject)

  
arguments: Anything - (Optional, default parent scope's `_this`) arguments passed to the method in the `_this` variable

Return Value:  
Anything - the value returned by the *methodName* method

### Examples

Example 1:  
``` sqf
call { hint str 123; };
```

Example 2:  
``` sqf
123 call { hint str _this; };
```

Example 3:  
``` sqf
_sum = [1, 2] call { (_this select 0) + (_this select 1); };
hint str _sum; // displays 3
```

Example 4:  
``` sqf
123 call compile "hint str _this;";
```

Example 5:  
``` sqf
_result = 123 call compile preprocessFileLineNumbers "myFile.sqf";
```

Example 6:  
``` sqf
private _hashMapObj = createHashMapObject [[
["MyMethod", { systemChat ("MyMethod arguments: " + str _this); }]
]];
_hashMapObj call ["MyMethod", ["Hello there", player, 123]]; // hints 'MyMethod arguments: ["Hello there", player, 123]'
```

### Additional Information

See also:  
spawn execVM canSuspend compile compileScript preprocessFile remoteExec remoteExecCall

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Leopard20 - c  
Posted on Nov 05, 2021 - 10:52 (UTC)

  
Contrary to a widespread misconception in the community, call doesn't necessarily execute the code in the Unscheduled Environment. Call simply means: "execute the code here", just like how you execute a code using then (`if (true) then _code`) or do. Therefore it does not change the environment:

<table class="wikitable">
<tbody>
<tr>
<th><em>call</em> executed in environment:</th>
<th>Resulting environment of the code:</th>
</tr>
&#10;<tr>
<td>Scheduled</td>
<td>Scheduled</td>
</tr>
<tr>
<td>Unscheduled</td>
<td>Unscheduled</td>
</tr>
</tbody>
</table>

This means that doing something like this (which is misused very often):

``` sqf
private _handle = _params spawn MY_fnc_Function;
waitUntil { scriptDone _handle };
```

is not needed, because a lot of performance would be wasted compared to simply doing this:

``` sqf
_params call MY_fnc_Function;
```

which does the exact same thing as the previous code, except no new *"scheduler thread"* is created and the function executes seamlessly, plus the added performance benefit as mentioned before.  
If it is necessary to execute a code in the Unscheduled Environment, one can use isNil instead:

``` sqf
isNil {_params call MY_fnc_Function}; // MY_fnc_Function is always executed unscheduled, regardless of the current environment
```

or if there are no parameters, one can simply do this:

``` sqf
isNil MY_fnc_Function
```
