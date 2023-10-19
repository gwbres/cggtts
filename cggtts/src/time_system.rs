use hifitime::TimeScale;

/// Reference Time Systems
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TimeSystem {
    /// TAI: International Atomic Time
    TAI,
    /// UTC: Universal Coordinate Time
    UTC,
    /// UTC(k): Laboratory local official
    /// UTC image, agency name
    /// and optionnal |offset| to universal UTC
    /// in nanoseconds
    UTCk(String, Option<f64>),
    /// Unknown Time system
    Unknown(String),
}

impl Default for TimeSystem {
    fn default() -> TimeSystem {
        TimeSystem::UTC
    }
}

impl TimeSystem {
    pub fn from_str(s: &str) -> TimeSystem {
        if s.eq("TAI") {
            TimeSystem::TAI
        } else if s.contains("UTC") {
            // UTCk with lab + offset
            if let (Some(lab), Some(offset)) = scan_fmt!(s, "UTC({},{})", String, f64) {
                TimeSystem::UTCk(lab, Some(offset))
            }
            // UTCk with only agency name
            else if let Some(lab) = scan_fmt!(s, "UTC({})", String) {
                TimeSystem::UTCk(lab, None)
            } else {
                TimeSystem::UTC
            }
        } else {
            TimeSystem::Unknown(s.to_string())
        }
    }
}

impl From<TimeScale> for TimeSystem {
    fn from(ts: TimeScale) -> Self {
        match ts {
            TimeScale::UTC => Self::UTC,
            TimeScale::TAI => Self::TAI,
            _ => Self::TAI, /* incorrect usage */
        }
    }
}

impl std::fmt::Display for TimeSystem {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TimeSystem::TAI => fmt.write_str("TAI"),
            TimeSystem::UTC => fmt.write_str("UTC"),
            TimeSystem::UTCk(lab, _) => write!(fmt, "UTC({})", lab),
            TimeSystem::Unknown(s) => fmt.write_str(s),
        }
    }
}

