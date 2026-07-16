1.00 1.00 1.00 1.00 1.50 1.00 0.50

### Description

Description:  
Returns time elapsed since mission started (in seconds). The value is different on each client. If you need unified time, use serverTime. Unlike uiTime the time stops when game is paused.

Groups:  
TimeMission Information

### Syntax

Syntax:  
time

Return Value:  
Number

### Examples

Example 1:  
``` sqf
private _future = time + 30;
waitUntil { time >= _future }; // continue after 30 seconds
```

Example 2:  
Wait until mission fully started:

``` sqf
waitUntil { time > 0 };
```

### Additional Information

See also:  
uiTime sleep date dayTime serverTime skipTime accTime dateToNumber timeMultiplier setTimeMultiplier missionStart diag_tickTime diag_deltaTime systemTime systemTimeUTC BIS_fnc_timeToString

### Notes

  
Report bugs on the Feedback Tracker and/or discuss them on the Arma Discord or on the Forums.  
**Only post proven facts here!** Add Note

<!-- -->

  
Posted on 2006-08-04 - 12:02

hardrock  
*Notes from before the conversion:*  
Not to be confused with the **SQS** variable *\_time*. Within an **SQS** script, the reserved local variable *\_time* returns the time elapsed since the script started running. Note that the value of \_time is not saved when the mission is saved and so will be reset to zero when a mission is restarted from a saved position. The value of time is saved correctly and will resume from where it was.

*\_time* has only special meaning in SQS scripts, in SQF script it is just another variable. --Killzone_Kid

Posted on 2007-01-05 - 12:24

Giova

*Notes from before the conversion:*  
time works properly in sqf called with execVM command. In an other hand, \_time does not works in sqf called with execVM command.(Arma v1.02.5103GER)

Posted on 2010-10-02 - 12:02

teaCup

On overloaded servers (below ~10 server FPS), time readings are unreliable. Seconds actually take longer. While the clients keep a steady tempo, server time lags behind, resulting in considerable offset between client and server time (easily 30 minutes for a 2 hour game). Client time is synchronised to server time during JIP, but other than that it runs independently.

Posted on 30 Oct 2013

Dr Eyeball

**Arma 3 JIP bug:**  
As of Arma 3 v1.02, for JIP clients 'time' value will start off counting from 0, not the real 'time' value. After about 2.5sec (on average), it will then jump to a high value and synchronise with the real 'time' value, which could be 900, for example. Therefore, do not use 'time' for any start of mission init timeouts; it is unreliable. (It's odd that it doesn't synchronise at the same time as public variables.)

Posted on 2016-09-01 - 18:18 (UTC)

Demellion

**In MP**: Since per-client time and server time is unconsistant I strongly recommend execution of time-critical tasks from server-side scripts and with remoteExec or remoteExecCall (**Since only A3 1.50** alternative may be publicVariableClient with pre-defined handler) as this will eliminate any time calculation lags and make it reliable.
