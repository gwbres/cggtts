CGGTTS 
======

Rust package to parse and generate CGGTTS data.

[![crates.io](https://img.shields.io/crates/v/cggtts.svg)](https://crates.io/crates/cggtts)
[![Rust](https://github.com/gwbres/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/cggtts/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/cggtts/badge.svg)](https://docs.rs/cggtts/badge.svg)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/cggtts/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/cggtts/blob/main/LICENSE-MIT) 
[![crates.io](https://img.shields.io/crates/d/cggtts.svg)](https://crates.io/crates/cggtts)    

CGGTTS is a file format to describe a local clock behavior against a single or the combination of clocks embedded in Satellite Vehicles (SV).  
Exchanging CGGTTS files enables so called "Common View" Time Transfer.

CGGTTS is specified by the Bureau International des Poids & des Mesures (BIPM):
[CGGTTS 2E specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

This library only supports revision **2E**, and will _reject_ other revisions.

## Ecosystem

`CGGTTS` heavily relies on `Hifitime` for accurate _Epoch_ representation
and _Timescales_ knowledge. 
Check out Christopher's amazing libraries [right here](https://github.com/nyx-space/hifitime).

The [RNX2CGGTTS application](https://github.com/georust/rinex) is the "goto" application when it comes
to generate CGTTTS files. Use it to generate synchronous CGGTTS tracks from coherent RINEX contexts.
You can then use "cggtts-cli" to compare two remote clocks.

## Crate achitecture

* `CGGTTS` is the main structure, it supports construction from a local file
or dumping into a local file
* `CGGTTS` is made of several attributes and a list of `Tracks` which are
actuall data
* a `Track` is made of several attributes, the actual data that
allows clock comparison is store in its `TrackData`

## CGGTTS track scheduling

If you compiled the crate with the _scheduler_ feature, you can access the
`Scheduler` structure that helps you generate synchronous CGGTTS tracks.

Synchronous CGGTTS is convenient because it allows direct exchange of CGGTTS files
and therefore, direct remote clocks comparison.

The `Scheduler` structure works according to the BIPM definitions but we allow for a different
tracking duration. The default being 980s, you can use shorter tracking duration and faster
CGGTTS generation. You can only modify the tracking duration if you can do so on both remote clocks,
so they share the same production parameters at all times.

## System Time delays

A built in API allows accurate system delay description as defined in CGGTTS.

## CGGTTS-CLI

[A command line application](gnss_cli/README.md) is developed to process one or two CGGTTS file for clock comparison.
