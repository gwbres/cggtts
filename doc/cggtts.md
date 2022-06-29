# CGGTTS 
Rust package to manipulate CGGTTS (BIPM) data files

[![crates.io](https://img.shields.io/crates/v/cggtts.svg)](https://crates.io/crates/cggtts)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/cggtts/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/cggtts/blob/main/LICENSE-MIT) 
[![crates.io](https://img.shields.io/crates/d/cggtts.svg)](https://crates.io/crates/cggtts)    
[![Rust](https://github.com/gwbres/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/cggtts/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/cggtts/badge.svg)](https://docs.rs/cggtts/badge.svg)

CGGTTS is a file format to exchange Common View Time transfer data.  
It is specified by the
Bureau International des Poids & des Mesures (BIPM, Paris).

[CGGTTS Specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

This library specifically supports revision **2E**, and will _reject_
other revisions.

Several structures exist in this crate:

* [Cggtts](doc/cggtts.md) if the main crate, it comprises measurement system
information and measurement data. It is a file parser (reader) and producer
(generate data).
* [Tracks](doc/track.md) are actual `Cggtts` measurements
* [Delay](doc/delay.md) represent the measurement internal delays

## parser

* file naming conventions must be respected.
* We tolerate fields /lines order of apperance (only in header section) to differ
from BIPM examples, except for first line (header introduction).
* revision must be 2E or we reject the given file

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

