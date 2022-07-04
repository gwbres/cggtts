CGGTTS
======

CGGTTS is the main structure, is can be parsed from a standard CGGTTS file, and can be dumped into a file following standards. 

Cggtts parser 
=============

## Known behavior 

* The parser supports current revision **2E** only, it will reject files that have different revision number.
* This parser does not care for file naming conventions

* While standard specifications says header lines order do matter,
this parser is tolerant and only expects the first CGGTT REVISION header to come first.

* BLANKs between header & measurements data must be respected
* This parser does not care for whitespaces, padding, it is not disturbed by their abscence
* This parser is case sensitive at the moment, all data fields and labels should be provided in upper case,
as specified in standards

* We accept several \"COMMENTS =\" lines, although it is not specified in CGGTTS. Several comments are then parsed. 

Notes on System delays and this parser

* This parser follows standard specifications, 
if \"TOT\" = Total delay is specified, we actually discard
any other possibly provided delay value, and this one superceeds
and becomes the only known system delay.
Refer to the System Delay documentation to understand what they
mean and how to specify/use them.

## Getting started

Parse a CGGTTS file:

```rust
let cggtts = Cggtts::from_file("data/standard/GZSY8259.506");
assert_eq!(cgtts.lab, Some("SY82"));
// more info on receiver hardware
println!("{:#?}", cggtts.rcvr);
// nb channels of this GNSS receiver
assert_eq!(cggtts.nb_channels, 12);
// IMS missing, see API/doc to understand what IMS is
assert_eq!(cggtts.ims, None);
// Coordinates reference system, used by X,Y,Z) 3D coordinates
assert_eq!(cggtts.coods_ref_system, Some("ITRF"));
assert_eq!(cggtts.comments, None); // no comments were identified
// basic CGGTTS (single carrier, see advanced usage..)
assert_eq!(cggts.has_ionospheric_data(), false);
// True if all measurements contained in this file
// follow BIPM recommendations for tracking duration
assert_eq!(track.follows_bipm_specs(), true);
``` 

## CGGTT Measurements 

Measurements are stored within the list of _CggttsTracks_

```rust
let cggtts = Cggtts::from_file("data/standard/GZSY8259.506");
let track = cggtts.first()
    .unwrap();
let duration = track.duration;
let (refsys, srsys) = (track.refsys, track.srsys);
assert_eq!(track.has_ionospheric_data(), false);
assert_eq!(track.follows_bipm_specs(), true);
```

## Advanced CGGTTS

```rust
let mut cggtts = Cggtts::from_file("data/ionospheric/RZOP0159.572");
// Has ionospheric parameter estimates,
// this is feasible on dual carrier receivers
assert_eq!(cggtts.has_ionospheric_parameters(), true); 
let Some(iono) = cggtts.ionospheric_data {
    let msio = iono.msio;
    let smsi = iono.smsi;
    let isg = iono.isg;
}

// IonosphericData supports unwrapping and wrapping
let data : IonosphericData = (1E-9, 1E-13, 1E-10).into();
// refer to [IonosphericData] to understand their meaning 
let (msio, smsi, isg) : (f64, f64, f64) = data.into();
```

## CGGTTS production

Use `to_string` methods to produce CGGTTS data

```rust
    let lab = String::from("MyAgency");
    let nb_channels = 16;
    let hardware = Rcvr {
        manufacturer: String::from("GoodManufacturer"),
        recv_type: String::from("Unknown"),
        serial_number: String::from("1234"),
        year: 2022,
        release: String::from("V1"),
    };
    let mut cggtts = Cggtts::new(
        Some(lab), 
        nb_channels, 
        Some(harware));

    write!(fd, "{}", cggtts).unwrap();
```

Add some measurements:

```rust
// standard CGGTTS (single carrier)
let class = CommonViewClass::Single;
// CGGTTS says, if we use many space vehicules
// to extrapolate data, we should set "99" as PRN
// to emphasize
let sv = rinex::sv::Sv {
    constellation: rinex::constellation::Constellation::GPS,
    prn: 99,
};

let mut track = Track::new(
    class,
    trktime,
    Track::BIPM_SPECIFIED_DURATION, // follow standards 
    space_vehicule: sv,
    elevation: 10.0,
    ...
);
cggtts.tracks.push(track);
write!(fd, "{}", cggtts).unwrap();
```

Use `to_string` methods to produce dump CGGTTS measurement

```rust
write!(fd, "{}", cggtts.tracks[0]).unwrap();
```

## Advanced CGGTTTS production

To produce advanced CGGTTS data correctly, one should specify / provide

* secondary hardware info (IMS)
* ionospheric parameters estimate
* specificy carrier dependent delay [see doc/delay.md]

```rust
let secondary_hw = Rcvr {
    manufacturer: String::from("ExtraGoodHardware"),
    recv_type: String::from("Unknown"),
    serial_number: String::from("1234"),
    year: 2022,
    release: String::from("V1"),
};
let cggtts = cggtts
    .with_ims_infos(secondary_hw);

cggtts.delay = SystemDelay {
    rf_cable_delay: 10.0,
    ref_delay: 5.0,
    calib_delay: CalibratedDelay {
        info: String::from("I did this calibration"),
        constellation: Constellation::GPS,
        delay: Delay::Internal(125.0),
    },
};

let track = Track::new();
/// see [IonosphericData in API]
track = track
    .with_ionospheric_data((1E-9,1E-13,1e-10));
```
