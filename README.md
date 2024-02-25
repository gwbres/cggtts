CGGTTS 
======

Rust package to parse and generate CGGTTS data.

[![crates.io](https://img.shields.io/crates/v/cggtts.svg)](https://crates.io/crates/cggtts)
[![Rust](https://github.com/gwbres/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/cggtts/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/cggtts/badge.svg)](https://docs.rs/cggtts/)
[![crates.io](https://img.shields.io/crates/d/cggtts.svg)](https://crates.io/crates/cggtts)    

CGGTTS is a file format designed to describe a local clock state compared to GNSS time systems.  
Exchanging CGGTTS files allows comparison of remote clocks by means of common satellite clocks in sight.  
This is called the "common view" time transfer technique.

CGGTTS is specified by the Bureau International des Poids & des Mesures (BIPM):
[CGGTTS 2E specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

This library only supports revision **2E**, and will _reject_ other revisions.

## Set of tools

- `cggtts` is the main library. Compile it with the _scheduler_ option to unlock
full support of CGGTTS data production
- `cggtts-cli` is an application to analyze one or compare two CGGTTS files.  
Download its latest release from the [github portal](https://github.com/gwbres/cggtts/releases).

## Ecosystem

`CGGTTS` heavily relies on `Hifitime` for accurate _Epoch_ representation
and _Timescales_ knowledge. 
Check out Christopher's amazing libraries [right here](https://github.com/nyx-space/hifitime).

The [RNX2CGGTTS application](https://github.com/georust/rinex) is the _goto_ application when it comes
to generate CGTTTS files. Use it to generate synchronous CGGTTS tracks from coherent RINEX contexts.  
Checkout the RINEX Wiki for examples of CGGTTS file exchanges.

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
