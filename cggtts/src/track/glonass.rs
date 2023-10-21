#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::track::Error;

/// Describes Glonass Frequency channel,
/// in case this `Track` was estimated using Glonass
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GlonassChannel {
    /// Default value when not using Glonass constellation
    Unknown,
    /// Glonass Frequency channel number
    Channel(u8),
}

impl std::fmt::Display for GlonassChannel {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GlonassChannel::Unknown => write!(fmt, "00"),
            GlonassChannel::Channel(c) => write!(fmt, "{:02X}", c),
        }
    }
}

impl std::str::FromStr for GlonassChannel {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("0") {
            Ok(Self::Unknown)
        } else {
            let ch = s
                .trim()
                .parse::<u8>()
                .map_err(|_| Error::FieldParsing(String::from("FR")))?;
            Ok(Self::Channel(ch))
        }
    }
}

impl PartialEq for GlonassChannel {
    fn eq(&self, rhs: &Self) -> bool {
        match self {
            GlonassChannel::Unknown => match rhs {
                GlonassChannel::Unknown => true,
                _ => false,
            },
            GlonassChannel::Channel(c0) => match rhs {
                GlonassChannel::Channel(c1) => c0 == c1,
                _ => false,
            },
        }
    }
}

impl Default for GlonassChannel {
    /// Default Glonass Channel is `Unknown`
    fn default() -> Self {
        Self::Unknown
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::track::GlonassChannel;
    use gnss::prelude::{Constellation, SV};
    use hifitime::Duration;
    use std::str::FromStr;
    #[test]
    fn glonass_test() {
        let c = GlonassChannel::Unknown;
        assert_eq!(c.to_string(), "00");
        let c = GlonassChannel::Channel(1);
        assert_eq!(c.to_string(), "01");
        let c = GlonassChannel::Channel(10);
        assert_eq!(c.to_string(), "0A");
        assert_eq!(c, GlonassChannel::Channel(10));
        assert_eq!(c != GlonassChannel::Unknown, true);
        assert_eq!(GlonassChannel::default(), GlonassChannel::Unknown);
    }
}
