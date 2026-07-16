1.00 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Returns true only if both conditions are true. In case of the alternative syntax, lazy evaluation is used - if left operand is false, evaluation of the right side is ignored.

Alias:  
a and b

Groups:  
VariablesMath

### Syntax

Syntax:  
booleanA && booleanB

Parameters:  
booleanA: Boolean - test condition or variable

  
booleanB: Boolean - test condition or variable

Return Value:  
Boolean

1.62

### Alternative Syntax

Syntax:  
boolean && code

Parameters:  
boolean: Boolean - test condition or variable

  
code: Code - code that once executed returns a Boolean. The code is not evaluated if **boolean** is false.

Return Value:  
Boolean

### Examples

Example 1:  
``` sqf
private _allEnemiesKilled = true;
if (alive player && _allEnemiesKilled) then
{
hint "you win !";
};
```

Example 2:  
``` sqf
if ((count _array > 0) && { (_array select 0) == player }) then // an error would be thrown without lazy evaluation
{
hint "It works!";
};
```

Example 3:  
``` sqf
if ((alive player) && { player setDamage 0.5; true }) then // valid AS LONG AS the code block returns a Boolean
{
hint "It works!";
};
```

### Additional Information

See also:  
and or Operators

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note
