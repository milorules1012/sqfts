1.00 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Check if given vehicle/person/building is alive (i.e. not dead or destroyed).

Groups:  
Object Manipulation

### Syntax

Syntax:  
alive object

Parameters:  
object: Object

Return Value:  
Boolean - true if alive, false if dead/destroyed

### Examples

Example 1:  
SQS:

``` sqf
? not alive player : exit
```

Example 2:  
SQF:

``` sqf
if (!alive player) exitWith {};
```

Example 3:  
``` sqf
alive objNull; // returns false
```

### Additional Information

See also:  
setDamage damage waterDamaged

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Pierre MGI - c  
Posted on Oct 19, 2015 - 23:48 (UTC)

  
Alive or not could be the question! in multi-player, missions come with respawn module(s). When a player is dead shot, `alive player` will return false, then almost immediately true if the "revive" respawn template is enabled, then could turn to false again if time for assistance is elapsed or if the player activates the respawn menu before; and finally true after player respawns. Just be aware that in that case (respawn + revive enabled), the status of the player is toggling: true → false → true → false → true. Then, alive status while player is waiting for being rescued could lead to some script errors as player is supposed to be alive but is in limbo and the dead entity "player" gets passed to server.
