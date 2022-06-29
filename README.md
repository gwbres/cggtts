CGGTTS 
======
Rust package to parse and crate CGGTTS data files.

[![crates.io](https://img.shields.io/crates/v/cggtts.svg)](https://crates.io/crates/cggtts)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/cggtts/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/cggtts/blob/main/LICENSE-MIT) 
[![crates.io](https://img.shields.io/crates/d/cggtts.svg)](https://crates.io/crates/cggtts)    
[![Rust](https://github.com/gwbres/cggtts/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/cggtts/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/cggtts/badge.svg)](https://docs.rs/cggtts/badge.svg)

CGGTTS is a file format to exchange Common View Time transfer data.  
It is specified by the
Bureau International des Poids & des Mesures (BIPM).

[CGGTTS 2E specifications](https://www.bipm.org/documents/20126/52718503/G1-2015.pdf/f49995a3-970b-a6a5-9124-cc0568f85450)

This library only supports revision **2E**, and will _reject_
other revisions.

Several structures exist in this crate:

* [Cggtts](doc/cggtts.md) is the main crate, it comprises measurement system
information and measurement data. It is a file parser (reader) and producer
(data generator).
* [Tracks](doc/track.md) are actual `Cggtts` measurements
* [Delay](doc/delay.md) represent the measurement internal delays
