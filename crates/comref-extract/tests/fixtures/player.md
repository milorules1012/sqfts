1.00 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
A person object controlled by user. In Intros and Outros this isn't set by default and must be assigned.

Multiplayer:  
In MP player is different on each computer and on dedicated server it is objNull by default (unless scripted). When user is joining a dedicated server mission, at the moment of onPlayerConnected the player object is remote to the joining client, as the player unit is created on the server, but once control of the unit is passed to the user moments later, the player objects becomes and remains local to the client. See Multiplayer Scripting's player topic for additional helpful information.

Problems:  
In multiplayer, the command is not initialised in functions called by initline or init eventhandlers.

Groups:  
Object Manipulation

### Syntax

Syntax:  
player

Return Value:  
Object

### Examples

Example 1:  
``` sqf
player addRating 500;
```

### Additional Information

See also:  
Multiplayer Scripting isPlayer playableUnits selectPlayer

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Galzohar - c  
Posted on Jun 20, 2010 - 03:11 (UTC)

  
Before you use the **player** object in init scripts, to avoid JIP issues all you need is to run:

``` sqf
waitUntil { not isNull player };
```

Anything else you see in other scripts is equivalent and/or redundant. (but you may need to add other conditions too, such as checking hasInterface before initiating that loop)  
Of course JIP players may need more than just the player to point at the actual JIP player unit, but that's script/mission-specific.

<!-- -->

Killzone_Kid - c  
Posted on Jun 26, 2014 - 18:49 (UTC)

  
player can actually be REMOTE object on player's PC: \[1\]
