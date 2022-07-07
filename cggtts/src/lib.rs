//! Rust package to parse, analyze and generate CGGTTS data.   
//! CGGTTS data files are dedicated to common view (two way satellite)
//! time transfer.
//!
//! Official BIPM `Cggtts` specifications:  
//! <https://www.bipm.org/wg/CCTF/WGGNSS/Allowed/Format_CGGTTS-V2E/CGTTS-V2E-article_versionfinale_cor.pdf>
//!
//! Only "2E" Version (latest to date) is supported by this parser

#[cfg(feature = "use-serde")]
#[macro_use]
extern crate serde;

pub mod delay;
pub mod track;
pub mod cggtts;
pub mod scheduler;
pub mod processing;

pub use crate::{
    delay::Delay,
    track::Track,
    track::GlonassChannel,
    cggtts::Code,
    cggtts::Rcvr,
    cggtts::Cggtts,
    cggtts::TimeSystem,
};

