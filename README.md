CGGTTS 
======

Rust package to parse and generate CGGTTS data.

[![crates.io](https://img.shields.io/crates/v/cggtts.svg)](https://crates.io/crates/cggtts)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/cggtts/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/cggtts/blob/main/LICENSE-MIT) 
[![crates.io](https://img.shields.io/crates/d/cggtts.svg)](https://crates.io/crates/cggtts)    
[![Rust](https://github.com/gwbres/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/cggtts/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/cggtts/badge.svg)](https://docs.rs/cggtts/badge.svg)

CGGTTS is a file format to describe a local clock behavior against a single or the combination of clocks embedded in Satellite Vehicles (SV).  
Exchanging CGGTTS files enables so called "Common View" Time Transfer.

CGGTTS is specified by the Bureau International des Poids & des Mesures (BIPM):
[CGGTTS 2E specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

This library only supports revision **2E**, and will _reject_ other revisions.

## Ecosystem

`CGGTTS` heavily relies on `Hifitime` for accurate _Epoch_ representation.  
Check out Christopher's amazing libraries [right here](https://github.com/nyx-space/hifitime).

The [RNX2CGGTTS application](https://github.com/georust/rinex) is the "goto" application when it comes
to generate CGTTTS files. Use it to generate such files from coherent RINEX contexts.
This application is part of the GeoRust toolsuite and is the combination of this crate
and the [GNSS-RTK solver](https://github.com/rtk-rs/gnss-rtk).

## Crate achitecture

* [CGGTTS](doc/cggtts.md) is the main structure, it comprises measurement system
information and measurement data. It is a file parser (reader) and producer
(data generator).
* a CGGTTS file contains several [Tracks](doc/track.md) 
* The number of tracks in a CGGTTS is defined by the tracking specification
* CGGTTS requires accurate [delay](doc/delay.md) specifications and compensation,
because it targets 0.1 ns residual errors. 

## Synchronous / Asynchronous CGGTTS

Usually CGGTTS files are synchronous and use the scheduling table
defined by BIPM. Originaly, the scheduler was defined from the GPS ephemerides 
and hardware (GNSS signal trackers) limitations.  

Nowadays the scheduling is kept and serves as an easy method to have synchronous CGGTTS.
Also note that _Epochs_ are expressed in UTC in CGGTTS. The combination of both has a lot of benefits,
synchronous CGGTTS files can be directly compared to one another (remote clock comparison).

## CGGTTS track production

The `SkyTracker` object is there to help generate `CGGTTS Tracks` from GNSS observations.  

The SkyTracker only implements the BIPM reference point at the moment, but the API allows using a different
tracking duration (faster tracking = tighter clock comparison).

By default the Tracking duration is 16' (13' Ephemerides + 3' historical warmup) so you can only get 90 
track from a typical Observation RINEX spanning 24 h.

`SkyTracker` allows real time modification of the tracking duration, although that is not recommended,
unless you have means to change the tracking duration on the remote site at the same time.

## CGGTTS-CLI

[A command line application](gnss_cli/README.md) is developed to process one or two CGGTTS file for clock comparison.
