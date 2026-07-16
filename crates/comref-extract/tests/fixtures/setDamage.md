 ? 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
<table class="wikitable float-right align-center-col-2">
<tbody>
<tr>
<th>Event Handler</th>
<th>Triggered</th>
</tr>
&#10;<tr>
<td><var>Hit</var></td>
<td></td>
</tr>
<tr>
<td><var>HandleDamage</var></td>
<td></td>
</tr>
<tr>
<td><var>Killed</var></td>
<td></td>
</tr>
<tr>
<td><var>MPKilled</var></td>
<td></td>
</tr>
</tbody>
</table>

Sets the damage (or lack thereof) of an object/unit. The alternative syntax allows to skip destruction effects for vehicles and buildings and set a tree/pole/wall fall direction (away from *killer*).

Alias:  
setDammage

Groups:  
Object Manipulation

### Syntax

Syntax:  
object setDamage damage

Parameters:  
object: Object

  
damage: Number - range 0..1, 0 = pristine/healthy condition, 1 = entirely destroyed/killed

Return Value:  
Nothing

1.68

### Alternative Syntax

Syntax:  
object setDamage \[damage, useEffects, killer, instigator, allowResurrection\]

Parameters:  
object: Object

  
damage: Number - range 0..1, 0 = pristine/healthy condition, 1 = entirely destroyed/killed

  
useEffects: Boolean (Optional, default true) - false to skip destruction effects

since  2.12  
killer: Object - (Optional, default objNull) the entity that caused the damage (can be anything). If the damage leads to the death of the unit, the killer will be used as the object that caused the kill.

- it can be used to show "killed by player" in debriefing statistics and kill messages in the chat (if death messages are enabled).
- it will alter the killer's rating as if the killer directly killed the unit
- it will be listed as `_killer` parameter in the Killed event handler
- it influences a tree/pole/wall fall direction (it will fall *away* from *killer*)
- *killer* is ignored if setDamage is called on a player client machine unless *object* is one of these three entity types - see Example 3
- MP restricted similar to setShotParents, will be ignored unless executed on the server or headless client in MP

since  2.12  
instigator: Object - (Optional, default objNull) the person that instigated the damage.

- if a tank is a killer, the tank gunner that pulled the trigger is instigator
- it will be listed as `_instigator` parameter in the Killed event handler
- MP restricted similar to setShotParents, will be ignored unless executed on the server or headless client in MP
  - the above MP restriction does not apply to trees, poles and walls

since  2.22  
allowResurrection: Boolean - (Optional, default false) - Will re-set alive state of previously dead object.

Return Value:  
Nothing

### Examples

Example 1:  
``` sqf
_soldier1 setDamage 1;
```

Example 2:  
``` sqf
_house1 setDamage [1, false];
```

Example 3:  
``` sqf
// executed on a client
_remoteVehicle setDamage [1, true, player]; // killer (here, player) is not considered if the command is not called on the server
// the vehicle still gets destroyed
_tree setDamage [1, true, player]; // the tree is destroyed and falls away from the player
_pole setDamage [1, true, player]; // these three cases are the only ones where killer is considered
_wall setDamage [1, true, player]; // when setDamage is called on a non-server machine (neither server or headless client)
```

### Additional Information

See also:  
setVehicleArmor damage setHit getHit getHitIndex setHitIndex getHitPointDamage setHitPointDamage

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Fragorl - c  
Posted on Apr 17, 2006 - 07:36 (UTC)

  
1.00

In **Operation Flashpoint**, setting a unit's damage to a negative value will set it is health to full, but impair their aim.

<!-- -->

KamikazeXeX - c  
Posted on May 29, 2015 - 11:23 (UTC)

  
Using this possible overrides individual hit damages such as setHitPointDamage \["HitHead", \_value\]; if you're having issues try setting hitdamage *after* setdamage.

<!-- -->

Sarogahtyp - c  
Posted on Jun 24, 2021 - 10:31 (UTC)

  
You are able to repair buildings with this command if you just store the original object and use setDamage on this and not on the wreck which is shown after the building was destroyed. This works in debug console when pointing a house:

``` sqf
0 spawn
{
private _house = cursorObject;
_house setDamage 1;
sleep 5;
_house setDamage 0;
};
```
