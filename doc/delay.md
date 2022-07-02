Delay: measurement systems delay 
=================================

`delay.rs` exposes several structures

* `Delay` : a generic delay value, always specified in nanoseconds
* `CalibratedDelay`:  basically a `Delay` that was evaluated for a specific
GNSS constellation.
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
* System / internal delay

System delay is defined as trusted as long as carrier dependent
delays are calibrated against a specific GNSS constellation (not Mixed constellation: avoid that scenario).

```rust
// build descriptor
let mut delay = SystemDelay {
    rf_cable_delay: 5.0, // always in [ns]
    ref_cable: 10.0, // always in [ns]
    calib_delay: CalibratedDelay { // carrier dependant delay
        info: None, // extra calibration info
        constellation: Constellation::GPS,
        delay: Delay::System(15.0), // always in [ns]
    },
};
assert_eq!(delay.value(), 5.0 + 10.0 + 15.0);
assert_eq!(delay.trusted(), true);
```

`CalibratedDelay` supports (+):
* non feasible (value remains the same, (+) never fails), if we're trying to add a value calibrated against a different constellation

```rust
let new = CalibratedDelay {
    info: None,
    constellation: Constellation::Glonass, // not feasible 
    delay: Delay::System(10.0), // same kind
}

delay = delay + new;
assert_eq!(delay.value(), 5.0 + 10.0 + 15.0); // nothing changed
assert_eq!(delay.trusted(), true); // nothing changed
```

* natural, if new value is specified against same constellation
```rust
let new = CalibratedDelay {
    info: None,
    constellation: Constellation::GPS, // same kind
    delay: Delay::System(1.0), // same kind
}

delay = delay + new;
assert_eq!(delay.value(), 5.0 + 10.0 + 15.0 +1.0); // increment 
assert_eq!(delay.trusted(), true); // nothing changed
```

* becomes _untrusted_ if we're adding a value calibrated against Mixed constellation

```rust
let new = CalibratedDelay {
    info: String::from("Calibrated by myself"),
    constellation:: Constellation::Mixed, // taints previous declaration
    delay: Delay::System(2.0), // same kind
}

delay = delay + new;
assert_eq!(delay.value(), 5.0 + 10.0 + 15.0 +1.0 +2.0); // increment
assert_eq!(delay.trusted(), false);
```
