# CGGTTS 
Rust package to manipulate CGGTTS (BIPM) data files

[![Rust](https://github.com/gwbres/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/cggtts/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/gwbres/cggtts/branch/main/graph/badge.svg)](https://codecov.io/gh/gwbres/cggtts)

CGGTTS is specified by BIPM, Paris 
(Bureau International des Poids & des Mesures)
and is dedicated to GNSS Time Transfer, Common View, Two Way
Satellites time transfers.

[CGGTTS Specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

Supported version: **2E**.   
Older CGGTTS format are _rejected_ by this library.

### Cggtts file analysis

Retrieve data from a local CGGTTS compliant file:

* File name must follow naming conventions, refer to specifications

```rust
    let cggtts = Cggtts::from_file("data/standard/GZSY8259.506");
    prinln!("{:#?}", cggtts);
    prinln!("{:#?}", cggtts.get_track(0));
    println!("{:?}", cggtts.get_antenna_coordinates());
    println!("{:#?}", cggtts.get_total_delay()); // refer to 'System Delays' section
    assert_eq!(cggtts.has_ionospheric_parameters(), false); // basic session
    
    let mut cggtts = Cggtts::from_file("data/ionospheric/RZOP0159.572");
    prinln!("{:#?}", cggtts);
    prinln!("{:#?}", cggtts.get_track(3));
    assert_eq!(cggtts.has_ionospheric_parameters(), true); // dual carrier session
```

#### Data analysis

Measurements are stored within the list of _CggttsTracks_

```rust
    let cggtts = Cggtts::from_file("data/standard/GZSY8259.506");
    let track = cggtts.get_track(0);
    prinln!("{:#?}", track);
    println!("{}", track.get_start_time());
    println!("{}", track.get_duration());
    prinln!("{:#?}", track.get_refsys_srsys());
```

_CggttsTracks_ are easily manipulated

```rust
    let t = cggtts.pop(); // grab 1
    assert_eq!(cggtts.len(), 0); // consumed
    assert_eq!(t.get_azimuth(), 180.0);
    assert_eq!(t.set_elevation(), 90.0);
```

### CGGTTS production

Using the basic constructor gets you started quickly

```rust
    let mut cggtts = Cggtts::new();
    cggtts.set_lab_agency("MyLab");
    cggtts.set_nb_channels(10);
    
    // Antenna phase center coordinates [m] 
    // is specified in IRTF spatial referencing
    cggtts.set_antenna_coordinates((+4027881.79.0,+306998.67,+4919499.36));
    println!("{:#?}", cggtts);
    assert_eq!(cggtts.get_total_delay(), 0.0); // system delays is not specified
    assert_eq!(cggtts.support_dual_frequency(), false); // not enough information
    cggtts.to_file("XXXX0159.572").unwrap(); // produce a CGGTTS
```

Add some measurements to a _Cggtts_

```rust
    let mut track = track::Cggttrack::new(); // basic track
    // customize a little
    track.set_azimuth(90.0);
    track.set_elevation(180.0);   
    track.set_duration(Cggtts::track::BIPM_SPECIFIED_TRACKING_DURATION); // standard
    cggtts.add_track(track);
```

Add ionospheric parameters estimates

```rust
    // read some data
    let cggtts = Cggtts::from_file("data/ionospheric/RZOP0159.572");
    assert_eq!(cggtts.has_ionospheric_parameters(), true); // yes
    // add some data
    let mut track = track::Cggttrack::new(); // basic track
    // customize
    track.set_duration(Cggtts::track::BIPM_SPECIFIED_TRACKING_DURATION); // respect standard
    track.set_refsys_srsys((1E-9,1E-12)); // got some data
    cggtts.push_track(track); // ionospheric_parameters not provided
          // will get blanked out on this line
    
    let params = (5.0E-9, 0.1E-12, 1E-9); // see (msio, smsi, isg) specifications
    track.set_ionospheric_parameters(params));
    cggtts.push_track(track);
    cggtts.to_file("RZOP0159.573"); // fully populated
```

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
when publishing a CGGTTS:

__(A)__ basic use: system delays are not known
```rust
let mut cggtts = Cggtts::new(); // default
assert_eq!(cggtts.get_total_delay(), 0.0); // no specifications
```

__(A*)__ basic use: usually the RF cable delay is easilly determined.
```rust
let mut cggtts = Cggtts::new(); // default
cggtts.set_cable_delay(10E-9); // [s] !!
assert_eq!(cggtts.get_total_delay(), 10E-9); // that's a starting point
cggtts.to_file("GZXXDD.DDD");
```

This definition does not match a standard definition.
To still generate a standardized CGGTTS, the lib in this context declares
a single frequency TOTAL DELAY of 10 ns.

__(A**)__ basic use: total delay is known.
Total delay is always tied to a GNSS constellation, like 
intermediate & advanced examples down below.
In case you don't know, use the default constructor:
```rust
let mut cggtts = Cggtts::new(); // default
cggtts.set_cable_delay(10E-9); // [s] !!
let mut delay = CalibratedDelay::default(); // default
delay.delays.push(100E-9); // [s]!!
assert_eq!(cggtts.get_total_delay(), 110E-9);
```

__(B)__ intermediate use: 
system delay is known and
we know the RF cable delay. System delay is always tied to a 
constellation:

```rust
let mut cggtts = Cggtts::new(); // default
cggtts.set_cable_delay(10E-9); // [s] !!

let delay = CalibratedDelay::new(
    constellation: track::Constellation::GPS,
    values: vec![150E-9],
    codes: vec!["C1"], // GPS::C1
    report: None
 );

cggtts.set_system_delay(delay); 
assert_eq!(cggtts.get_total_delay(), 150-9+10E-9);
cggtts.to_file("GZXXDD.DDD");
```

__(B*)__ same hypothesis but in a dual frequency context.
Therefore we specify a delay for each carrier frequency: 

```rust
let mut cggtts = Cggtts::new(); // default
cggtts.set_cable_delay(25E-9); // [s] !!

let delay = CalibratedDelay::new(
    constellation: track::Constellation::Glonass,
    values: vec![150E-9,345E-9],
    codes: vec!["C1", "P1"], // Glonass::C1&P1
    report: None
 );

cggtts.set_system_delay(delay); 
assert_eq!(cggtts.get_total_delay(), 150-9+10E-9);
cggtts.to_file("GZXXDD.DDD");
```

__(C)__ advance use: (A+B) intrinsic delays are known 

```rust
let mut cggtts = Cggtts::new(); // default
cggtts.set_cable_delay(25E-9); // [s] !!

let delay = CalibratedDelay::new(
    constellation: track::Constellation::Galileo,
    values: vec![50E-9],
    codes: vec!["E1"], // Galileo::E1
    report: "some-calibration-info"
 );

cggtts.set_internal_delay(delay); 
assert_eq!(cggtts.get_total_delay(), 50E-9+50E-9+25E-9);
cggtts.to_file("GZXXDD.DDD");
```
