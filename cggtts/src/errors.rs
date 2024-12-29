//! CGGTTS errors
use thiserror::Error;

use crate::track::Error as TrackError;

/// Errors related to CRC parsing
/// and calculations specifically.
#[derive(PartialEq, Debug, Error)]
pub enum CrcError {
    #[error("can only calculate over valid utf8 data")]
    NonUtf8Data,
    #[error("checksum error, got \"{0}\" but \"{1}\" locally computed")]
    ChecksumError(u8, u8),
}

/// Errors strictly related to file parsing.
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
    #[error("mixing constellations is not allowed in CGGTTS")]
    MixedConstellation,
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
    #[error("invalid crc value")]
    ChecksumValue,
    #[error("missing crc field")]
    CrcMissing,
    #[error("track parsing error")]
    TrackParsing(#[from] TrackError),
}

/// Errors strictly related to CGGTTS formatting
#[derive(Debug, Error)]
pub enum FormattingError {
    #[error("bad utf-8 data")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("i/o error: {0}")]
    Stdio(#[from] std::io::Error),
}
