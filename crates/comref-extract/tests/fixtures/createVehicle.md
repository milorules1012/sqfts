1.34 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Creates an empty object of given classname type. See Arma 3: Assets / Arma 3: createVehicle/vehicles, or Category:CfgVehicles for earlier games.

ⓘ

- To create objects of type `"Logic"`, use createUnit.
- To create an editable Eden Editor entity, see create3DENEntity.

To avoid vehicle randomisation in Arma 3, set the `BIS_enableRandomization` variable immediately after creating the vehicle:

``` sqf
private "_vehicle";
isNil { // run unscheduled
_vehicle = "C_Offroad_01_F" createVehicle getPosATL player;
_vehicle setVariable ["BIS_enableRandomization", false];
};
// the _vehicle variable is available after that
```

See the Arma 3: Vehicle Customisation page for more information and settings.

Groups:  
Object Manipulation

### Syntax

Syntax:  
type createVehicle position

Parameters:  
type: String - vehicle/object className

  
position: Array format Position - desired placement position. If the exact position is occupied, nearest empty position is used.

Return Value:  
Object

### Alternative Syntax

Syntax:  
createVehicle \[type, position, markers, placement, special\]

Parameters:  
type: String - vehicle/object className

  
position: Object; Array format Position2D or PositionATL (PositionAGL if watercraft or amphibious) - desired placement position

  
markers: Array of Strings - (Optional, default `[]`) if the markers array contains any markers, the position is randomly picked from array of given markers plus desired placement position. If any of the markers were given z coordinate with setMarkerPos, the vehicle will also be created at given z coordinate.

  
placement: Number - (Optional, default 0) the vehicle is placed inside a circle with given position as center and placement as its radius.

  
special: String - (Optional, default "NONE") can be one of the following:

- `"NONE"` - will look for suitable empty position near given position (subject to other placement params) before placing vehicle there.
- `"CAN_COLLIDE"` - places vehicle at given position (subject to other placement params), without checking if others objects can cross its 3D model.
- `"FLY"` - if vehicle is capable of flying and has crew, it will be made airborne at default height.

If *special* is "" or not specified, default `"NONE"` is used.

Return Value:  
Object - created vehicle or objNull if failed

### Examples

Example 1:  
``` sqf
_jeep = "Jeep" createVehicle position player;
```

Example 2:  
``` sqf
_heli = "AH1Z" createVehicle getMarkerPos "hspawn";
```

Example 3:  
``` sqf
_veh = createVehicle ["ah1w", position player, [], 0, "FLY"];
```

Example 4:  
``` sqf
_veh = createVehicle ["2S6M_Tunguska", getMarkerPos "marker1", ["marker2", "marker3"], 0, "NONE"];
```

Example 5:  
Objects such as

- "test_EmptyObjectForBubbles"
- "test_EmptyObjectForFireBig"
- "test_EmptyObjectForSmoke"

create additional emitters, which are stored in "effects" variable on the object. Since Arma 3 v1.72 these emitters are automatically deleted when object is deleted

``` sqf
0 spawn
{
private _fire = "test_EmptyObjectForFireBig" createVehicle position player;
sleep 5;
deleteVehicle _fire;
};
```

Example 6:  
The following explosives (ending with `_Scripted`) can be set off by applying setDamage 1 to them for ease of scripting:

- "DemoCharge_Remote_Ammo_Scripted"
- "SatchelCharge_Remote_Ammo_Scripted"
- "ClaymoreDirectionalMine_Remote_Ammo_Scripted"

``` sqf
_claymore = "ClaymoreDirectionalMine_Remote_Ammo_Scripted" createVehicle position player;
_claymore spawn
{
sleep 5;
_this setDamage 1;
};
```

Example 7:  
Add inventory to objects without inventory:

``` sqf
_boxes = "Land_Pallet_MilBoxes_F" createVehicle position player;
_cargo = "Supply500" createVehicle [0,0,0];
_cargo attachTo [_boxes, [0,0,0.85]];
// optional for objects that can take damage
_boxes addEventHandler ["Killed",
{
{
detach _x,
deleteVehicle _x;
}
forEach attachedObjects (_this select 0);
}];
```

Example 8:  
Drop player's weapon:

``` sqf
_weaponHolder = "GroundWeaponHolder_Scripted" createVehicle position player;
player action ["DropWeapon", _weaponHolder, currentWeapon player];
```

Example 9:  
The following weapon holders (ending with *\_Scripted*) do **not** auto-delete when empty. It is up to the mission maker to take care of these:

- "GroundWeaponHolder_Scripted"
- "WeaponHolderSimulated_Scripted"
- "Weapon_Empty" (a special weaponholder that displays only a single weapon, even if it contains magazines for this weapon)

``` sqf
0 spawn
{
private _weaponHolder = createVehicle ["Weapon_Empty", getPosATL player, [], 0, "CAN_COLLIDE"];
_weaponHolder addWeaponCargo ["arifle_Katiba_F", 1];
hint "You have 5 seconds to grab this weapon";
sleep 5;
deleteVehicle _weaponHolder;
};
```

### Additional Information

See also:  
createVehicleLocal create3DENEntity createVehicleCrew createAgent createTrigger createUnit createMine deleteVehicle createGroup createCenter setVehiclePosition

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

MrSanchez - c  
Posted on Aug 22, 2015 - 13:04 (UTC)

  
GroundWeaponHolder class is automatically deleted when empty after 0.5 to 1 seconds in A3 1.48. The exact delay is random but never lower than 0.50 secs after creation. You can stop deletion by adding something (cargo) to it within 0.5 seconds.

<!-- -->

AgentRev - c  
Posted on May 16, 2017 - 09:05 (UTC)

  
For the alternative syntax, if the vehicle has `canFloat`` ``=`` ``1``;` in its config class (e.g. boats and wheeled APCs) the command expects PositionAGL, otherwise always PositionATL.

<!-- -->

demellion - c  
Posted on Nov 02, 2018 - 12:16 (UTC)

  
**WARNING:** Do not instigate createVehicle or createVehicleLocal within a server function executed with preInit flag.  
This will cause *"You cannot play/edit this mission"* for a vehicles compiled from a -mod and make server skip/loop that mission init.

<!-- -->

R3vo - c  
Posted on Aug 10, 2019 - 09:10 (UTC)

  
The main syntax creates vehicles at ground level ignoring the Z in *pos* and is also faster than the alternative syntax.

``` sqf
"vehclass" createVehicle pos;
```

This is equivalent to

``` sqf
createVehicle ["vehclass", [pos select 0, pos select 1, 0], [], 0, "NONE"];
```

<!-- -->

DreadedEntity - c  
Posted on Mar 13, 2022 - 17:22 (UTC)

  
Objects are created with a vectorUp of terrain surface normal. If you are creating new buildings to add to the map, you will probably want to call setVectorUp on the newly-spawned object.

``` sqf
_veh = createVehicle [/*etc...*/];
_veh setVectorUp [0,0,1];
```
