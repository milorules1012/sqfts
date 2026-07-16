1.00 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Returns object or location position. If the argument is an object, the return value is in format PositionAGLS.  
The alternative syntax gets the position given distance and heading away from provided object or position - the command equivalent of BIS_fnc_relPos.

⚠

Do **not** use this command's first syntax to get an object's position in 3D format:

- the Z value from this command is **relative** to the surface underneath, and there is no compatible setter command to use it with;  
  the commonly misused code: `_obj1 setPos getPos _obj2` is **absolutely wrong**
- this command is significantly **slower** than other position commands because it has to calculate the surface below a position from objects in the (2D) area; its performance therefore suffers from areas with a high density of objects such as cities, and can easily be ~20x slower than other, simpler position commands - see this benchmark.

The **only** correct usage of this command is to determine the altitude of an object **from the surface below it** (see Example 4).

Groups:  
Positions

### Syntax 1

Syntax:  
getPos object

Parameters:  
object: Object

Return Value:  
Array format PositionAGLS - where Z is the height over the surface underneath

### Syntax 2

Syntax:  
getPos location

Parameters:  
location: Location

Return Value:  
Array format PositionAGL - see locationPosition

1.56

### Syntax 3

Syntax:  
origin getPos \[distance, heading\]

Parameters:  
origin: Object, Position2D or Position3D

  
distance: Number - distance from position

  
heading: Number - in which compass direction

Return Value:  
Array format PositionAGLS - the Z value is **terrain level Above Ground Level**, either 0 over land or getTerrainHeightASL (negative value) over water

### Examples

Example 1:  
``` sqf
hintSilent str getPos player;
```

Example 2:  
getPos vs. other methods (over sea). Pay attention to Z values:

``` sqf
getPos ship; // [2412.01, 6036.33, -0.839965]
getPosATL ship; // [2412.01, 6036.33, 19.4266]
getPosASL ship; // [2412.01, 6036.33, -0.920066]
getPosASLW ship; // [2412.01, 6036.33, -0.865981]
visiblePosition ship; // [2412.02, 6036.33, -0.837952]
visiblePositionASL ship; // [2412.02, 6036.33, -0.91798]
position ship; // [2412.01, 6036.33, -0.839965]
```

Example 3:  
getPos vs. other methods (over land, on top of a 100m high building). Pay attention to Z values:

``` sqf
getPos car; // [2508.64, 5681.47, 0.0609589]
getPosATL car; // [2508.64, 5681.47, 100.0356369]
getPosASL car; // [2508.64, 5681.47, 171.718]
getPosASLW car; // [2508.64, 5681.47, 171.718]
visiblePosition car; // [2508.64, 5681.47, 0.0609512]
visiblePositionASL car; // [2508.64, 5681.47, 171.718]
position car; // [2508.64, 5681.47, 0.0609589]
```

Example 4:  
Determine if a free-falling unit is close enough to the surface (including buildings, aircraft carriers etc) below to deploy the parachute:

``` sqf
waitUntil { sleep 1; getPos player select 2 < 200 };
hint "Deploying a parachute might be a good idea";
```

Example 5:  
Find position 100 metres and 45 degrees from player position:

``` sqf
player getPos [100, 45]; // e.g [1834, 2230, 0] if resulting pos is over land
// or [120, 256, -271] if resulting pos is over water
```

### Additional Information

See also:  
getPosVisual getRelPos setPos setPosAGLS position getPosATL getPosASL getPosASLW visiblePosition visiblePositionASL getMarkerPos

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Dr_Eyeball - c  
Posted on Feb 17, 2007 - 13:43 (UTC)

  
`getPos obj select 2` might return the vertical position above ground level, but for a stacked object, it returns the vertical position above the object beneath it. The same problem existed for getPosASL (**pre-Arma 2 games only**). There was a *discussion* <sub>(dead link)</sub> thread in the BIS forums which suggested the use of the command modelToWorld instead to get around this issue where an absolute vertical position is required. ArmA Ver **1.02**.
