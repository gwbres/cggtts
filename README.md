# CGGTTS 
Rust package to manipulate CGGTTS (BIPM) data files

[![Rust](https://github.com/gwbres/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/cggtts/actions/workflows/rust.yml)

CGGTTS is specified by BIPM, Paris 
(Bureau International des Poids & des Mesures)
and is dedicated to GNSS Time Transfer, Common View, Two Way
Satellites time transfers.

[CGGTTS Specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

Supported version: "2E". Older versions are not managed by this lib,
mainly because it knows how to manipulate cable delays.

## Examples
For compelling examples, refer to the integrated test methods.

### Basic usage

Basic usage does not provide enough information for dual frequency CGGTTS,
but gets you started quickly

```rust
    let mut cggtts = Cggtts::new();
    cggtts.set_lab_agency("MyLab");
    cggtts.set_nb_channels(10);
    // Antenna phase center coordinates [m] IRTF referencing
    cggtts.set_antenna_coordinates(
        (+4027881.79.0,+306998.67,+4919499.36)
    );
    // basic usage, only total system delay is known
    cggtts.set_total_delay(300E-9);
```

Build from a file
```rust
    let cggtts = Cggtts::from_file("data/standard/GSOP0159.571");
    prinln!("{:?}", cggtts);
    prinln!("{:?}", cggtts.get_track(3));
    assert_eq!(cggtts.has_ionospheric_parameters(), false); // basic
    println!("{:?}", cggtts.get_antenna_coordinates());
    println!("{}", cggtts.get_total_delay().unwrap()); // always
    assert_eq!(cggtts.get_system_delay().is_err(), true); // basic usage
    assert_eq!(cggtts.get_cable_delay().is_err(), true); // basic usage
    
    let cggtts = Cggtts::from_file("data/ionospheric/GSOP0159.571");
    prinln!("{}", cggtts);
    prinln!("{:?}", cggtts.get_track(5));
    assert_eq!(cggtts.has_ionospheric_parameters(), true); // advanced
    println!("{}", cggtts.get_total_delay().unwrap()); // always
    assert_eq!(cggtts.get_system_delay().is_err(), false); // advanced usage
    assert_eq!(cggtts.get_cable_delay().is_err(), false); // advanced usage
    println!("{}", cggtts.get_cable_delay().unwrap());
```

CGGTTS Tracks manipulation
```rust
    let mut cggtts = [...]
    let mut track = track::Cggttrack::new();
    track.set_elevation(180.0);
    track.set_azimuth(180.0);
    track.set_duration(track::BIPM_SPECIFIED_TRACKING_DURATION);
    cggtts.add_track(track);

    let mut track = cggtts.pop(); // grab 1 please
    assert_eq!(track.get_satellite_id(), 0xFF); // Multi satellite PRN#
    track.set_satellite_id(0x01); // Single SatVehicule PRN#
```
