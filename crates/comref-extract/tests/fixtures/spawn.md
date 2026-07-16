1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Adds given set of compiled instructions to the scheduler. Exactly when the code will be executed is unknown, it depends on how busy is the engine and how filled up is the scheduler. Therefore spawn does not wait for the supplied code to finish, instead, spawn returns a Script handle to the scheduler task. scriptDone command can be used to check the code completion. Additional arguments are passed to the code in local variable \_this. Since 1.56 the script handle also exists inside the code in `_thisScript` variable. To see what spawned scripts are currently in the scheduler, use diag_activeSQFScripts command.

⚠

When multiple Code is spawned in an order, there is <u>no guarantee</u> that the spawned Code will execute in the same order (see Example 2). If the order is important, use BIS_fnc_spawnOrdered.

Groups:  
Program Flow

### Syntax 1

Syntax:  
arguments spawn code

Parameters:  
arguments: Anything - arguments passed to the script, which later available in `_this` variable inside the script.

  
code: Code

Return Value:  
Script Handle - can be used to determine (via scriptDone (also via isNull in Arma 3)) when the spawned script has finished. In Arma 3, the handle is also available inside the spawned script in `_thisScript` variable

2.22

### Syntax 2

Syntax:  
spawn code

Parameters:  
code: Code

Return Value:  
Script Handle

2.22

### Syntax 3

Syntax:  
spawn name

Parameters:  
name: String - The name of a "Empty Handle". Behaves like scriptName inside a function.

Return Value:  
Script Handle - A "Empty Handle" to be used as a Promise

### Examples

Example 1:  
``` sqf
_handle = 0 spawn { player globalChat "Hello world!" };
```

Example 2:  
There is no guarantee that spawned scripts will be executed in the same order they spawned:

``` sqf
for "_i" from 0 to 100 do
{
_i spawn
{
diag_log _this;
};
}; // Result: 51,1,2...49,50,0,52,53...100
```

Example 3:  
Local variables declared in the main scope are not available in the spawned code. You have to pass them as parameters:

``` sqf
private _localVariable = 1234;
[_localVariable] spawn
{
systemChat format ["_localVariable does not exist: %1", isNil "_localVariable"]; // _localVariable does not exist: true
params ["_localVariable"];
systemChat format ["_localVariable is now: %1", _localVariable]; // _localVariable is now: 1234
};
```

### Additional Information

See also:  
call execVM execFSM exec compile preprocessFileLineNumbers preprocessFile terminate scriptDone remoteExec sleep uiSleep canSuspend

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Kronzky - c  
Posted on Mar 06, 2009 - 00:20 (UTC)

  
spawn cannot call other local functions on the same scope as itself.  
It can, however, call other global functions:

``` sqf
_addOne = { TST = TST + 1 };
TST_addOne = { TST = TST + 1 };
_add = {
TST = TST+1;
player sideChat format ["added: %1",TST];
call _addOne;
player sideChat format ["called local: %1",TST];
call TST_addOne;
player sideChat format ["called global: %1",TST];
};
TST = 0;
call _add;
0 spawn _add;
```

The call of \_addOne from the spawned function does not do anything.

<!-- -->

DreadedEntity - c  
Posted on Oct 21, 2014 - 23:33 (UTC)

  
0.50 spawn requires a script handle when used in the 2D-Editor.  
2.04 In Eden Editor this is no longer necessary.  
  
In scripts and in the debug console, it is not required, but very useful for keeping track of running scripts. Having a script handle also makes it easy to terminate scripts at any time.  
  
Since spawn creates a new scheduled environment, having an excess of open threads can make the scheduler queue extremely long, significantly increasing the execution time of each thread. (it takes an extremely large amount of threads, though)

<!-- -->

Nelis75733126 - c  
Posted on Aug 25, 2015 - 13:39 (UTC)

  
If you have a local (private) function that you want to access from within code created with `spawn`, you can pass that private function to the code of `spawn`, like so:

``` sqf
_someFunction = {};
[ _someFunction ] spawn { call( _this select 0 ) };
```
