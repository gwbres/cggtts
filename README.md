# CGGTTS 
Rust package to manipulate CGGTTS (BIPM) data files

[![Rust](https://github.com/gwbres/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/cggtts/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/gwbres/cggtts/branch/main/graph/badge.svg)](https://codecov.io/gh/gwbres/cggtts)

CGGTTS is specified by BIPM, Paris 
(Bureau International des Poids & des Mesures)
and is dedicated to GNSS Time Transfer, Common View, Two Way
Satellites time transfers.

[CGGTTS Specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

Supported version: "2E". Older versions are rejected by this library.

## Examples

For compelling examples, refer to the integrated test methods.

### Cggtts File Parsing

Retrieve Cggtts Data from a local file:

* File must be at least revision "2E"
* older revisions will be rejected
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

### Basic constructor

```rust
    let mut cggtts = Cggtts::new(); // Unknown system delays, see down below
    cggtts.set_lab_agency("MyLab");
    cggtts.set_nb_channels(10);
    // Antenna phase center coordinates [m] 
    // uses IRTF spatial referencing
    cggtts.set_antenna_coordinates(
        (+4027881.79.0,+306998.67,+4919499.36)
    );
    println!("{:#?}", cggtts);
    assert_eq!(cggtts.get_total_delay(), 0.0); // system delays are not known
    assert_eq!(cggtts.support_dual_frequency(), false); // not enough information
    cggtts.to_file("XXXX0159.572").unwrap();
```

### Tracks manipulation

Add some measurements to a previous **Cggtts** (basic)

```rust
    let mut cggtts = Cggtts::new(); // basic struct,
    // Unknown system delays, see down below
    // -> add some measurements
    let mut track = track::Cggttrack::new(); // basic track
    // customize a little
    track.set_azimuth(90.0);
    track.set_elevation(180.0);   
    track.set_duration(Cggtts::track::BIPM_SPECIFIED_TRACKING_DURATION); // standard
    cggtts.add_track(track);

    let t = cggtts.pop(); // grab 1
    assert_eq!(cggtts.len(), 0); // consumed
    assert_eq!(t.get_azimuth(), 180.0);
    assert_eq!(t.set_elevation(), 90.0);
```

Add some measurements to a previous **Cggtts** (advance)

```rust
    // read some data
    let cggtts = Cggtts::from_file("data/ionospheric/RZOP0159.572");
    assert_eq!(cggtts.has_ionospheric_parameters(), true); // yes
    // add some data
    let mut track = track::Cggttrack::new(); // basic track
    // customize
    track.set_duration(Cggtts::track::BIPM_SPECIFIED_TRACKING_DURATION); // respect standard
    track.set_elevation(90.0);
    track.set_azimuth(180.0);
    track.set_refsys_srsys((1E-9,1E-12));
    cggtts.push_track(track); // ionospheric_parameters not provided
                       // will get blanked out on this line
    
    track.set_ionospheric_parameters((alpha, beta)));
    cggtts.push_track(track); // respects previous context
    cggtts.to_file("RZOP0159.573"); // produce a new file
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
