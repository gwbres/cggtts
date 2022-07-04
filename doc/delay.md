Delay: measurement systems delay 
=================================

`delay.rs` exposes several structures

* `Delay` : a generic delay value, always specified in nanoseconds
* `SystemDelay` : used by Cggtts to describe the measurement systems delay.

## `SystemDelay` object

```
+--------+               +---------- system ---------+ +++++++++++
+        +      cable    +------+                      + counter +
+  ANT   + ------------> + RCVR + -------------------> + DUT     +
+--------+               +------+                      +         +
                            ^                          +         +
 +++++++ -------------------| ref_dly                  +         +
 + REF + --------------------------------------------> + REF     +
 +++++++                                               +++++++++++
```

SystemDelay represents the summation of all delays:

* RF/cable delay
* Reference delay
* System or internal delay, that are carrier dependent

System or internal delays are calibrated against 
a specific GNSS constellation.
