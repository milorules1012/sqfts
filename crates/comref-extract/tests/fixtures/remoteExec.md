1.50

### Description

Description:  
Asks the server to execute the given function or script command on the given target machine(s).

- Functions are executed in the scheduled environment; suspension is allowed.
- Script commands are executed in the unscheduled environment; suspension is not allowed (see Example 7).

Read Arma 3: Remote Execution for more information about remote execution, security features and JIP techniques.

ⓘ

remoteExec/remoteExecCall can be used in single player as well, as it is considered as player-hosted multiplayer.

⚠

The direct execution of call or spawn via remoteExec (or remoteExecCall) should be avoided to prevent issues in cases where the remote execution of call or spawn is blocked by CfgRemoteExec. It is instead recommended to create a function to be itself remote-executed.

⚠

The order of persistent remote execution for JIP players is not guaranteed, i.e. the order in which multiple calls are added is not necessarily the order they will be executed for joining player.

Multiplayer:  
Remote executions are queued and are therefore executed in the same order on remote clients (see Example 8).

Groups:  
Multiplayer

### Syntax

Syntax:  
params remoteExec \[order, targets, JIP\]

Parameters:  
**params**: Anything **but** Structured Text - *order*'s parameter

⚠

Structured Text is **not** meant to be sent over network.

  
**order**: String - function or command name; while any function or command can be used here, only those allowed by CfgRemoteExec will actually be executed

  
**targets** - (Optional, default 0):

- Number (See also Machine network ID):
  - **0:** the order will be executed globally, i.e. on the server and every connected client, including the machine where remoteExec originated
  - **2:** the order will only be executed on the server - is both dedicated and hosted server. See for more info
  - **Other number:** the order will be executed on the machine where clientOwner matches the given number
  - **Negative number:** the effect is inverted: **-2** means every client but not the server, **-12** means the server and every client, except for the client where clientOwner returns 12
- Object - the order will be executed where the given object is local
- String - interpreted as an Identifier (variable name); the function / command will be executed where the object or group identified by the variable with the provided name is local
- Side - the order will be executed on machines where the player is on the specified side
- Group - the order will be executed on machines **where the player is in the specified group** (**not** where said group is local!)
- Array - array of any combination of the types listed above

  
**JIP** - (Optional, default false):

- Boolean - if true, a unique JIP ID is generated and the remoteExec statement is added to the JIP queue from which it will be executed for every JIP
- String:
  - if the string is empty, it is interpreted as false
  - if the string is in format "Number:Number" (e.g. "0:0"), it is interpreted as a netId (see below)
  - else the string is treated as a custom JIP ID and the remoteExec statement is added to the JIP queue, replacing statements that have the same JIP ID
- Object, Group or netId - the persistent execution of the remoteExec statement is attached to the given object or group, replacing any previous statement that has the same JIP ID.  
  When the object / group is deleted, the remoteExec statement is automatically removed from the JIP queue

The **JIP** parameter can only be used if the **targets** parameter is 0 or a negative number.  
See also Example 3 on how to remove statements from the JIP queue.

Return Value:  
- nil - In case of error.
- String - In case of success (see remoteExecutedJIPID).
  - If the **JIP** parameter was false or an empty string, the return value is "".
  - If the **JIP** parameter was true or a custom JIP ID, the JIP ID is returned.
  - If the **JIP** parameter was an Object, a Group or a netId, the (corresponding) netId is returned.

### Alternative Syntax

Syntax:  
remoteExec \[functionName, targets, JIP\]

Parameters:  
**functionName**: String - see the main syntax above for more details.

  
**targets**: Number, Object, String, Side, Group or Array - (Optional, default 0) see the main syntax above for more details.

  
**JIP**: Boolean, String, Object, Group or netId - (Optional, default false) see the main syntax above for more details.

Return Value:  
nil or String - see the main syntax above for more details.

### Examples

Example 1:  
How to write remoteExec/remoteExecCall with arguments - note the colours: `hint`` ``"Hello"``; ``// becomes`` [``"Hello"``] remoteExec ["``hint``"]; ``"Hello"`` remoteExec ["``hint``"]; ``// alternatively` `unit1`` ``setFace`` ``"Miller"``; ``// becomes`` [``unit1``, ``"Miller"``] remoteExec ["``setFace``"];` `cutRsc`` ``["", "BLACK OUT"]``; ``// becomes`` [``["", "BLACK OUT"]``] remoteExec ["``cutRsc``"]; ``// double brackets are needed as the unary command takes an array` `// functions, however, do not need double squared brackets`` ``["line 1", "line 2"]`` spawn ``BIS_fnc_infoText``; ``// becomes`` ``["line 1", "line 2"]`` remoteExec ["``BIS_fnc_infoText``"]; `

Example 2:  
send an order to specific machines:

``` sqf
"message" remoteExec ["hint"]; // sends a hint message to everyone
"message" remoteExec ["hint", 0]; // sends a hint message to everyone, identical to "message" remoteExec ["hint"]
"message" remoteExec ["hint", -2]; // sends a hint message to everybody but the server (also not hosted server)
"message" remoteExec ["hint", myCar]; // sends a hint message where myCar is local
"message" remoteExec ["hint", -clientOwner]; // sends a hint message to everybody but the current machine
```

Example 3:  
Add statements to the JIP queue:

``` sqf
private _jipId = ["mission state: the car is broken"] remoteExec ["systemChat", 0, true]; // adds the hint to the JIP queue and returns the JIP queue order id
waitUntil { canMove _car };
remoteExec ["", _jipId]; // the systemChat order is removed from the JIP queue
```

``` sqf
["mission state: the car is broken"] remoteExec ["systemChat", 0, _queueObject];
// ...
remoteExec ["", _queueObject]; // the order attached to _queueObject is removed
```

``` sqf
private _jipId = ["mission state: the car is broken"] remoteExec ["systemChat", 0, "MY_JIP_ID"]; // _jipId is actually "MY_JIP_ID" now
waitUntil { canMove _car };
["mission state: the car is repaired"] remoteExec ["systemChat", 0, "MY_JIP_ID"]; // this order replaces the previous one
// ...
remoteExec ["", "MY_JIP_ID"]; // the "MY_JIP_ID" order is removed from the JIP queue
```

Example 4:  
Some more complex examples:

``` sqf
["Open", true] remoteExec ["BIS_fnc_arsenal", MyTargetPlayer];
[MyCurator, [[MyObject1, MyObject2], false]] remoteExec ["addCuratorEditableObjects", 2];
```

Example 5:  
A tricky example: executing `player setAmmo [primaryWeapon player, 1];` (on machines where the player is in MyGroup):

``` sqf
[player, [primaryWeapon player, 1]] remoteExec ["setAmmo", MyGroup]; // WRONG: the local player object is used here!
[{ player setAmmo [primaryWeapon player, 1]; }] remoteExec ["call", MyGroup]; // CORRECT: the remote player object is used here
```

Example 6:  
**Multiplayer Scripting "performance trick"**  
This `[0, -2] select isDedicated` check is worth it to avoid **function** server-side calculations only. See also Example 9 for an advanced solution.

``` sqf
["message"] remoteExec ["BIS_fnc_infoText"]; // not ideal - the function will still run on the dedicated server for nothing
["message"] remoteExec ["BIS_fnc_infoText", [0, -2] select isDedicated]; // ideal - the dedicated server will not run the code, a player-hosted server will
["message"] remoteExec ["hint", [0, -2] select isDedicated]; // the check is too expensive to be worthy - it becomes worthy if the server logs an RPT warning
["message"] remoteExec ["hint"]; // the (dedicated) server will automatically ditch hint usage due to it not having an interface
private _allPlayersTarget = [0, -2] select isDedicated; // caching the result for multiple usages makes it worthy - think of headless clients as well
["message 1"] remoteExec ["hint", _allPlayersTarget];
["message 2"] remoteExec ["hint", _allPlayersTarget];
```

ⓘ

See Example 9 below for an advanced example.

Example 7:  
As said in the description: **commands** will be executed in an unscheduled environment

``` sqf
[{ sleep 1 }] remoteExec ["call"]; // will throw an error: it is forbidden to use sleep (or waitUntil, etc) in unscheduled environment
```

Example 8:  
``` sqf
"Message 1" remoteExec ["systemChat"];
"Message 2" remoteExec ["systemChat"];
// will result in
// "Message 1"
// "Message 2"
// in this exact order on clients
```

Example 9:  
It is possible to create a "to all players" remote exec target variable:

``` sqf
if (isServer) then
{
TO_ALL_PLAYERS = [0, -2] select isDedicated;
publicVariable "TO_ALL_PLAYERS";
};
```

Show HC-compatible version

If Headless Clients are involved:

``` sqf
if (isServer) then
{
TO_ALL_PLAYERS = [0, -2] select isDedicated;
private _allNegativeHCs = allPlayers apply { getPlayerID _x } select { _x != "-1" } // all valid playerIDs
apply { getUserInfo _x } select { _x select 7 } // filter by HC
apply { -(_x select 1) }; // get negative network ID
if (_allNegativeHCs isNotEqualTo []) then
{
TO_ALL_PLAYERS = [TO_ALL_PLAYERS] + _allNegativeHCs;
};
publicVariable "TO_ALL_PLAYERS";
addMissionEventHandler ["OnUserConnected", {
params ["_networkId"];
private _userInfo = getUserInfo _networkId;
if !(_userInfo select 7) exitWith {}; // not a HC
if (TO_ALL_PLAYERS isEqualType 0) then // number to array conversion
{
if (TO_ALL_PLAYERS == 0) then // player-hosted
{
TO_ALL_PLAYERS = [-(_userInfo select 1)];
}
else // -2, dedicated server
{
TO_ALL_PLAYERS = [TO_ALL_PLAYERS, -(_userInfo select 1)];
};
}
else // already an array
{
TO_ALL_PLAYERS pushBackUnique -(_userInfo select 1);
};
publicVariable "TO_ALL_PLAYERS";
}];
};
```

↑ Back to spoiler's top

``` sqf
// client or server will always target the good machines
["Yay!"] remoteExec ["hint", TO_ALL_PLAYERS];
```

### Additional Information

See also:  
Multiplayer Scripting remoteExecCall remoteExecutedOwner isRemoteExecuted isRemoteExecutedJIP Arma 3: Remote Execution canSuspend BIS_fnc_MP remoteExecutedJIPID

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

  

AgentRev - c  
Posted on Dec 29, 2015 - 20:28 (UTC)

  
remoteExec and remoteExecCall are currently filtered by BattlEye's remoteexec.txt, the string analyzed by BE is formatted the same way as the following example's output:

``` sqf
format ["%1 %2", functionName, str params]
```

If CfgRemoteExec `class Functions` is set to `mode = 1;`, the following remoteexec.txt exclusion can be used to safely allow all whitelisted \*\_fnc\_\* functions taking an array as parameter to go through:

``` sqf
!="\w+?_fnc_\w+? \[[\S\s]*\]"
```

Any attempt to exploit this exclusion using other RE methods like createUnit will run into "Error Missing ;" without any malicious code being executed. Mod makers should refrain from remote-executing raw commands from clients, as they require individual exclusions, and instead use \*\_fnc\_\* functions taking an array as parameter, which are covered by the above exclusion.

<!-- -->

Pierre MGI - c  
Posted on Jan 30, 2017 - 18:35 (UTC)

  
``` sqf
[someArgs] remoteExec ['someCommand', 2, true];
```

will fail, as you can't use JIP and remoteExec on server only

``` sqf
[someArgs] remoteExec ['someCommand', 2]; // works
```

<!-- -->

7erra - c  
Posted on Mar 05, 2021 - 00:48 (UTC)

  
The remoteExec'ed function only has to exist on the target machine. For example:

``` sqf
// initPlayerLocal.sqf
TAG_fnc_testRemote = {
hint "Remote Exec Received";
};
```

``` sqf
// executed on a DEDICATED server
remoteExec ["TAG_fnc_testRemote", -2];
```

Will display a hint for every client. This is especially useful for when the server is running a mod that is not required by clients.

<!-- -->

Sa-Matra - c  
Posted on Sep 19, 2024 - 08:33 (UTC)

  
It is not possible to use two negative owners to exclude two clients, you'll end up REing all clients:

``` sqf
args remoteExec ["func", [-2, -3]]; // Doesn't work, gonna execute on every client
```
