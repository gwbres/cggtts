//! CGGTTS errors
use thiserror::Error;

use crate::{crc::Error as CrcError, track::Error as TrackError};

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("file i/o error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("only revision 2E is supported")]
    VersionMismatch,
    #[error("invalid version")]
    VersionFormat,
    #[error("invalid CGGTTS format")]
    InvalidFormat,
    #[error("invalid revision date")]
    RevisionDateFormat,
    #[error("non supported file revision")]
    NonSupportedRevision,
    #[error("invalid delay calibration ID#")]
    InvalidCalibrationId,
    // #[error("coordiantes parsing error")]
    // CoordinatesParsing,
    #[error("failed to identify delay value in line \"{0}\"")]
    DelayIdentificationError(String),
    #[error("failed to parse frequency dependent delay from \"{0}\"")]
    FrequencyDependentDelayParsingError(String),
    #[error("invalid common view class")]
    CommonViewClass,
    #[error("checksum format error")]
    ChecksumFormat,
    #[error("failed to parse checksum value")]
    ChecksumParsing,
    #[error("header crc error")]
    ChecksumError(#[from] CrcError),
    #[error("missing crc field")]
    CrcMissing,
    #[error("track parsing error")]
    TrackParsing(#[from] TrackError),
}
