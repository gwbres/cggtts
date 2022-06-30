CGGTTS
======

CGGTTS is the main structure, is can be parsed from a standard CGGTTS file, and can be dumped into a file following standards. 

Cggtts parser 
=============

## Known behavior 

* The parser supports current revision **2E** only, it will reject files that have different revision number.
* This parser does not care for file naming conventions

* While it is clearly specified in the standards, that header lines order matter, this parser is tolerant against line/data order, except for the first CGGTT REVISION header line which should always come first.

* We tolerate a missing BLANK between the header section and measurements

* This parser does not care for whitespaces, padding, it is not disturbed by their abscence

* This parser is case sensitive at the moment, as it appears all fields should be written in upper case, except for data units/scalings 

* We accept several \"COMMENTS =\" lines, although it is not specified in CGGTTS. Several comments are then parsed. 

Notes on System delays and this parser

* This parser follows standard specifications, 
if \"TOT\" = Total delay is specified, we actually discard
any other possibly provided delay value, and this one superceeds
and becomes the only known system delay.
Refer to the System Delay paragraph to understand what they
mean and how to specify them.

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
``` 

## CGGTT Measurements 

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

## Advanced CGGTTS

```rust
    let mut cggtts = Cggtts::from_file("data/ionospheric/RZOP0159.572");
    // Has ionospheric parameter estimates,
    // this is feasible on dual carrier receivers
    assert_eq!(cggtts.has_ionospheric_parameters(), true); 
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

