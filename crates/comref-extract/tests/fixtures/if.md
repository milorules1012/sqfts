1.85 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
This operator creates a If Type which is used in the if-construct as described here.

Groups:  
Program Flow

### Syntax

Syntax:  
if condition

Parameters:  
condition: Boolean - if it evaluates as true, the then clause is executed. Otherwise, the else clause (if present) is executed

Return Value:  
If Type - predicate which will execute 1st or 2nd option when used. This predicate is used in then or exitWith commands

### Examples

Example 1:  
``` sqf
_retVal = if (1 > 0) then { "It's true" } else { "It's false" };
hint str _retVal;
```

Example 2:  
``` sqf
_val = if (true) then [{ "true" }, { "false" }];
hint _val;
```

### Additional Information

See also:  
else exitWith then Control Structures Lazy evaluation

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

Ceeeb - c  
Posted on Jan 31, 2007 - 04:08 (UTC)

  
Any \_local variables you declare within the body of an if/then statement (ie between the curly braces) are local to that 'if' statement, and are destroyed at the end of the statement. If you know you want to use the variable outside the 'if' statement, make sure your declare it before the 'if' statement.

<!-- -->

Galzohar - c  
Posted on Jan 17, 2010 - 02:40 (UTC)

  
1.05

If the condition is nil then neither the "then" nor the "else" section get executed, but the script will proceed with no error messages.  
Example code:

``` sqf
systemChat "script started"; // will get executed
if (nil) then
{
systemChat "true"; // will never get executed
}
else
{
systemChat "false"; // will never get executed
};
systemChat "script ended"; // will get executed
```

<!-- -->

AgentRev - c  
Posted on Jun 05, 2015 - 09:35 (UTC)

  
If you only need to choose between 2 raw values, it is possible to use the following trick to avoid using code blocks, as required by the if command, which results in greater atomicity and faster execution:

``` sqf
_result = [falseValue, trueValue] select condition;
```

The select command treats "false" as 0 and "true" as 1, therefore you can feed it a condition determining the array index of the value to be returned. Here is another example:

``` sqf
_result = [1,-1] select (_this < 0); // if _this is less than 0, _result will be equal to -1, otherwise it will be 1
```

<!-- -->

DreadedEntity - c  
Posted on May 19, 2022 - 01:11 (UTC)

  
When using **if** to assign values to a variable, if you not have an else block it is treated as nil and returned, setting the variable value to nil. A3 2.08.149102

``` sqf
private _myVar = "value";
private _condition = false;
_myVar = if (_condition) then { "another value" }; //perfectly valid sqf
systemChat str (isNil "_myVar"); //returns true
//_myVar is now nil
_myVar = if (_condition) then { "another value" } else { "value" };
systemChat _myVar; //returns "value"
```

This behavior can be both desirable and undesirable depending on your needs
