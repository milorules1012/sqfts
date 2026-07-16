1.10 1.00 1.00 1.50 1.00

### Description

Description:  
Adds an entry to the action menu of an object. The action can only be activated when in proximity to the object **and** looking at it. Adding an action to the player makes that action available to the player at all times. For event handling of user interaction see inGameUISetEventHandler.

Multiplayer:  
The command will be ignored on dedicated servers, probably because no UI exists.

Problems:  

Due to a bug in Operation Flashpoint, actions added via addAction do not get properly updated after mounting vehicles. When several actions are available while mounting a vehicle and it drives away from the actions' position, they will still be shown in the menu until dismounting/boarding the vehicle again.

Groups:  
Interaction

### Syntax

Syntax:  
object addAction \[title, script, arguments, priority, showWindow, hideOnUse, shortcut, condition, positionInModel, radius, radiusView, showIn3D, available, textDefault, textToolTip\]

Parameters:  
**object**: Object - unit, vehicle or static object. **No agents!**

  
**title**: String - the action name which is displayed in the action menu, may contain Structured Text. Because of that **\<** and **\>** symbols will be interpreted as opening and closing XML tags. To avoid this use `&lt;` for **\<** and `&gt;` for **\>**. The title text appearance can be changed with setUserActionText

  
**script**: String or 1.00 Code - path to the script file relative to the mission folder, or code to execute. It will run in scheduled environment, i.e. it is ok to use sleep. Parameters array passed to the script upon activation in *\_this* variable is:

``` sqf
params ["_target", "_caller", "_actionId", "_arguments"];
```

- **target**: Object - the object which the action is assigned to
- **caller**: Object - the unit that activated the action
- **actionID**: Number - activated action's ID (same as addAction's return value)
- **arguments**: Anything - arguments given to the script if you are using the extended syntax

since  1.00  
**arguments**: Anything - (Optional, default nil) arguments to pass to the script. Accessible with `_this select 3` inside the script. If Array is used as an argument for example, its first element reference would be `_this select 3 select 0`

since  1.00  
**priority**: Number - (Optional, default 1.5) priority value of the action. Actions will be arranged in descending order according to this value. Every game action has a preset priority value. Value can be negative or decimal fraction. Actions with same values will be arranged in order which they were made, newest at the bottom. The bigger the number the higher the action will be positioned on the menu. Typical range is 0 to 6

since  1.00  
**showWindow**: Boolean - (Optional, default true) if set to true, players see "Titletext" at mid-lower screen, as they approach the object. Only the title text for the action with highest priority and *showWindow* set to true will be shown

since  1.00  
**hideOnUse**: Boolean - (Optional, default true) if set to true, it will hide the action menu after selecting it. If set to false, it will leave the action menu open and visible after selecting the action, leaving the same action highlighted, for the purpose of allowing you to re-select that same action quickly, or to select another action

since  1.00  
**shortcut**: String - (Optional, default "") one of the key names defined in bin.pbo (e.g. "moveForward"). Adding available shortcut will bind corresponding keyboard key to this action. Shortcut availability can be tested with inputAction command

since  1.00  
**condition**: String - (Optional, default "true") expression that must return true for the action to be shown. **Special variables** passed to the script code are:

- *\_target*: Object - The object to which action is attached or, if the object is a unit inside of vehicle, the vehicle
- *\_this*: Object - Caller person to whom the action is shown (or not shown if *condition* returns false)
- *\_originalTarget*: Object - The original object to which the action is attached, regardless if the object/unit is in a vehicle or not

⚠

- *condition* is evaluated on each frame in unscheduled environment.
- *condition* is **not** evaluated if a dialog is open.
- If action is added to an object and not to the player, *condition* will only get evaluated if the player is closer than ~50m to the object surface and is looking at the object.
- If action is added to player, *condition* is evaluated all the time.

since  1.00  
**positionInModel**: String - (Optional, default "") name of the named selection in the model for positioning the action in 3D space; typically a memory point

since  1.00  
**radius**: Number - (Optional, default 50) maximum 3D distance in meters between the activating unit's eyePos and the *object's* *memoryPoint*, *selection* or position. -1 disables the radius

since  1.00  
**radiusView**: Number - (Optional, default **unknown**) maximum distance in meters the cursor can be away from the 3D action to activate it. -1 disables this radius

since  1.00  
**showIn3D**: Number - (Optional, default **unknown**) condition for showing the action in 3D space. Can be combined e.g. `2 + 32`

- 1 - show
- 2 - draw if unit is pilot
- 4 - draw if unit is inside vehicle
- 8 - draw if unit is outside vehicle
- 16 - draw if not in external camera
- 32 - draw if not in internal camera
- 64 - draw if not in gunner camera (turret optics)

since  1.00  
**available**: Number - (Optional, default **unknown**) condition for being able to activate the action. Can be combined e.g. `1 + 4`

- 0 - disabled
- 1 - unit is pilot or copilot
- 2 - unit is inside target
- 4 - unit is not inside target

since  1.00  
**textDefault**: String - (Optional, default "") structured Text which is shown as the 3D action (so it can be an icon), or in the center of the screen when the action is highlighted in the action menu for a 2D action

since  1.00  
**textToolTip**: String - (Optional, default "") structured Text which is faded in under the textDefault when hovering over the action in 3D space

Return Value:  
Number - the added action's ID. Action can be removed with removeAction (see also removeAllActions). IDs are incrementing, the first given action to each unit has the ID 0, the second the ID 1, etc. IDs are also passed to the called script (see the *script* parameter)

### Examples

Example 1:  
``` sqf
// short and sweet
player addAction ["a useless action that does nothing", {}];
player addAction ["<t color='#FF0000'>This Useless Action Is RED</t>", { hint "RED" }];
player addAction ["Hint Hello!", { hint format ["Hello %1!", name player] }];
player addAction ["String Exec", "hint 'this is also compiled'"];
```

Example 2:  
``` sqf
_actionID = player addAction ["Exec the file", "scriptFile.sqf"];
```

**scriptFile.sqf:**

``` sqf
hint str _this;
```

Example 3:  
Take On Helicopters:

``` sqf
_heli addAction
[
"Test",
"myTest.sqf",
"",
1,
true,
true,
"",
"true",
"display1",
2,
0.25,
9,
0,
"<img image='\HSim\UI_H\data\ui_action_autohover_ca.paa' size='1.8' shadow=0 />",
"<br />My test tooltip"
];
```

### Additional Information

See also:  
actionIDs actionParams setUserActionText inGameUISetEventHandler showHUD inputAction removeAction removeAllActions action

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

0.50

### Description

Description:  
Adds an entry to the action menu of an object. The action can only be activated when in proximity to the object **and** looking at it. Adding an action to the player makes that action available to the player at all times. For event handling of user interaction see inGameUISetEventHandler.

Multiplayer:  
The command will be ignored on dedicated servers, probably because no UI exists.

Groups:  
Interaction

### Syntax

Syntax:  
object addAction \[title, script, arguments, priority, showWindow, hideOnUse, shortcut, condition, radius, unconscious, selection, memoryPoint\]

Parameters:  
**object**: Object - unit, vehicle or static object. **No agents and simple objects!**

  
**title**: String - the action name displayed in the action menu, may contain Structured Text. Because of that **\<** and **\>** symbols will be interpreted as opening and closing XML tags. To avoid this use `&lt;` for **\<** and `&gt;` for **\>**. The title text appearance can be changed with setUserActionText

  
**script**: String or Code - either path to the script file, relative to the mission folder or string with code or the actual script code. If the string is a path to script file, the script file **must** have extension .SQS or .SQF. The script, whether it is a file or a code, will run in scheduled environment, i.e. it is ok to use sleep. Parameters array passed to the script upon activation in *\_this* variable is:

``` sqf
params ["_target", "_caller", "_actionId", "_arguments"];
```

- **target**: Object - the object which the action is assigned to
- **caller**: Object - the unit that activated the action
- **actionID**: Number - activated action's ID (same as addAction's return value)
- **arguments**: Anything - arguments given to the script if you are using the extended syntax

  
**arguments**: Anything - (Optional, default nil) arguments to pass to the script. Accessible with `_this select 3` inside the script. If Array is used as an argument for example, its first element reference would be `_this select 3 select 0`

  
**priority**: Number - (Optional, default 1.5) priority value of the action. Actions will be arranged in descending order according to this value. Every game action has a preset priority value. Value can be negative or decimal fraction. Actions with same values will be arranged in order which they were made, newest at the bottom. The bigger the number the higher the action will be positioned on the menu. Typical range is 0 to 6

  
**showWindow**: Boolean - (Optional, default true) if set to true, players see "Titletext" at mid-lower screen, as they approach the object. Only the title text for the action with highest priority and *showWindow* set to true will be shown

  
**hideOnUse**: Boolean - (Optional, default true) if set to true, it will hide the action menu after selecting it. If set to false, it will leave the action menu open and visible after selecting the action, leaving the same action highlighted, for the purpose of allowing you to re-select that same action quickly, or to select another action

  
**shortcut**: String - (Optional, default "") one of the key names defined in bin.pbo (e.g. "moveForward"). Adding available shortcut will bind corresponding keyboard key to this action. Shortcut availability can be tested with inputAction command

  
**condition**: String - (Optional, default "true") expression that must return true for the action to be shown. **Special variables** passed to the script code are:

- *\_target*: Object - The object to which action is attached or, if the object is a unit inside of vehicle, the vehicle
- *\_this*: Object - Caller person to whom the action is shown (or not shown if *condition* returns false)
- *\_originalTarget*: Object - The original object to which the action is attached, regardless if the object/unit is in a vehicle or not
- *\_actionId*: Number - checked action's ID (same as addAction's return value)

⚠

If the player is the group leader, has an action added to them, and selects a subordinate, *`_this`* alternatively switches between the player and this selected unit - RPT logging `Unknown attribute itemsCmd` in the process.

⚠

- *condition* is evaluated on each frame in unscheduled environment.
- *condition* is **not** evaluated if a dialog is open.
- If action is added to an object and not to the player, *condition* will only get evaluated if the player is closer than ~50m to the object surface and is looking at the object.
- If action is added to player, *condition* is evaluated all the time.

since  1.64  
**radius**: Number - (Optional, default 50) maximum 3D distance in meters between the activating unit's eyePos and *object*'s *memoryPoint*, *selection* or position. -1 disables the radius; hardcoded limit is **50**

since  1.64  
**unconscious**: Boolean - (Optional, default false) if true will be shown to incapacitated player. See also setUnconscious and lifeState

since  1.70  
**selection**: String - (Optional, default "") *object*'s geometry LOD's named selection

since  1.82  
**memoryPoint**: String - (Optional, default "") *object*'s memory point. If *selection* is supplied, *memoryPoint* is not used

Return Value:  
Number - the added action's ID. Action can be removed with removeAction (see also removeAllActions). IDs are incrementing, the first given action to each unit has the ID 0, the second the ID 1, etc. IDs are also passed to the called script (see the *script* parameter)

### Examples

Example 1:  
``` sqf
// short and sweet
player addAction ["a useless action that does nothing", {}];
player addAction ["<t color='#FF0000'>This Useless Action Is RED</t>", { hint "RED" }];
player addAction ["Hint Hello!", { hint format ["Hello %1!", name player] }];
player addAction ["String Exec", "hint 'this is also compiled'"];
```

Example 2:  
``` sqf
_actionID = player addAction ["Exec the file", "scriptFile.sqf"];
```

**scriptFile.sqf:**

``` sqf
hint str _this;
```

Example 3:  
``` sqf
// create object on the server and add action to the object on every client
if (isServer) then
{
private _object = "some_obj_class" createVehicle [1234, 1234, 0];
[_object, ["Greetings!", { hint "Hello!"; }]] remoteExec ["addAction"]; // Note: does not return action id
};
```

Example 4:  
Default parameters:

``` sqf
this addAction
[
"title",
{
params ["_target", "_caller", "_actionId", "_arguments"];
},
nil,
1.5,
true,
true,
"",
"true",
50,
false,
"",
""
];
```

Example 5:  
Default parameters with comments:

``` sqf
this addAction
[
"title", // title
{
params ["_target", "_caller", "_actionId", "_arguments"]; // script
},
nil, // arguments
1.5, // priority
true, // showWindow
true, // hideOnUse
"", // shortcut
"true", // condition
50, // radius
false, // unconscious
"", // selection
"" // memoryPoint
];
```

### Additional Information

See also:  
actionIDs actionParams setUserActionText inGameUISetEventHandler showHUD inputAction removeActionremoveAllActionsactionBIS_fnc_holdActionAdd

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Dedmen - c  
Posted on May 02, 2018 - 13:44 (UTC)

  
If you want to replicate vanilla Actions like "Treat yourself" where the scroll menu only shows text and it displays the icon mid-screen you can use

``` sqf
private _actionId = player addAction ["Heal", {}];
player setUserActionText [_actionId, "Heal", "<img size='2' image='\a3\ui_f\data\IGUI\Cfg\Actions\heal_ca'/>"];
```

<!-- -->

PierreMGI - c  
Posted on May 12, 2020 - 10:42 (UTC)

  
addAction on unit or player stays on corpse after kill. So this action is lost for the new body after respawn. If you want a persistent addAction, you need to add it in the respawn script (onPlayerRespawn.sqf or addMissionEventHandler "entityRespawned" or addMPEventHandler "MPrespawn"... You can use removeAllActions on corpse.

<!-- -->

Leopard20 - c  
Posted on Aug 28, 2022 - 21:52 (UTC)

  
If the server blocks execVM in CfgRemoteExec, you can't use path to script:

``` sqf
_obj addAction ["Test", "path\script.sqf"]; // "path\script.sqf" won't execute
```

You can, however, do this:

``` sqf
_obj addAction ["Test", { _this spawn compile preprocessFileLineNumbers "path\script.sqf" }];
```

But it's recommended to use functions if the script is called frequently.
