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

## System delays definition

**Delays are always specified in [ns]**

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

* total delay is defined as ANT + cable + A + B
* (A+B) is defined as internal delay, internaly delays inside
the receiver & the antenna
* if (A+B) granularity is not known, we then refer to (A+B)=system delay
in case it is known
* cable delay refers to the RF cable delay
* ref delay is the time offset between the time reference (whose name is "Reference" field),
and the receiver internal clock

Three scenarios:
(0) total unknown = 0 is what you get with ::new()
(1) system delay + ref delay (basic)
(1*) int delay +

## System delays in dual frequency context
In dual frequency context, total and system delays are related
to carrier frequency. Therefore in must be specified for which
GNSS constellation and which carrier.
This is why **Cggtts.get_total_delay()** is returns a calibrated delay object
