//! CGGTTS
//!
//! Rust package to parse and generate CGGTTS data files.   
//! CGGTTS data files are dedicated to common view (two way satellite)
//! time transfer.
//!
//! Official BIPM `Cggtts` specifications:
//! <https://www.bipm.org/wg/CCTF/WGGNSS/Allowed/Format_CGGTTS-V2E/CGTTS-V2E-article_versionfinale_cor.pdf>
//!
//! Only "2E" Version (latest to date) supported
//!
//! Homepage: <https://github.com/gwbres/cggtts>
use regex::Regex;
use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;

//mod cggtts;
pub mod delay;
pub mod track;

pub use crate::{
    delay::Delay,
    delay::CalibratedDelay,
    track::Track,
};

/// supported `Cggtts` version,
/// non matching input files will be rejected
const VERSION: &str = "2E";

/// latest revision date
const LATEST_REV_DATE: &str = "2014-02-20";
