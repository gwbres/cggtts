use crate::{errors::ParsingError, prelude::Epoch};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Version {
    #[default]
    Version2E,
}

impl Version {
    pub(crate) fn release_date(&self) -> Epoch {
        match self {
            Self::Version2E => Epoch::from_gregorian_utc_at_midnight(2014, 02, 20),
        }
    }
}

impl std::str::FromStr for Version {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("2E") {
            Ok(Self::Version2E)
        } else {
            Err(ParsingError::NonSupportedRevision)
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Version2E => write!(f, "2E"),
        }
    }
}

impl std::fmt::LowerHex for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Version2E => {
                write!(f, "2014-02-20")
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::Version;
    use std::str::FromStr;

    #[test]
    fn version_parsing() {
        let version_2e = Version::from_str("2E").unwrap();
        assert_eq!(version_2e.to_string(), "2E");
        assert_eq!(format!("{:x}", version_2e), "2014-02-20");
    }
}
