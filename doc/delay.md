Delay: measurement system's delay 
=================================

`delay.rs` exposes two structures

* `Delay` : a generic delay value, always specified in nanoseconds
* `CalibratedDelay`:  basically a `Delay` that was evaluated for a specific
GNSS constellation.


### System delays

When we refer to system delays we refer to propagation induced delays.

In _Cggtts_ files, delays are always specified in **[ns]**.  
Ths library manipulates delays in seconds, but converts them
back to **[ns]** in file operations.

#### Definitions

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

* "total" delay is defined as ANT + cable + A + B
* (A+B) is defined as internal delay, propagation delay inside
the receiver & the antenna
* in case we do not have the (A+B) granularity, we then refer to (A+B)=system delay

* cable delay refers to the RF cable delay

* ref delay is the time offset between the time reference (whose name is "Reference" field),
and the receiver internal clock

* internal (A+B) or system delay are mutually exclusive.
User is expected to use either one of them.  
In case user specifies both, this lib will not crash but prefer (A+B) (advaced system definition)

* when internal (A+B) delay or system delay is provided,
the standard expects a Ref delay too. 
In case the user did not specify a Ref delay, we set it to 0 ns
to still have a valid Cggtts generated.

#### Case of Dual Frequency CGGTTS
In dual frequency context (two carriers), 
_total_, _system_, _internal_ should be specified
for each carrier frequency.

_cable_ and _ref_ delays are not tied to a carrier frequency.

#### Delays and CGGTTS production interface

This library provdes an easy to use interface to specify your system
