/// Describes whether this common view is based on a unique
/// or a combination of SV
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum CommonViewClass {
    /// Single Channel
    SingleChannel,
    /// Multi Channel
    MultiChannel,
}

impl std::fmt::UpperHex for CommonViewClass {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommonViewClass::MultiChannel => write!(fmt, "FF"),
            CommonViewClass::SingleChannel => write!(fmt, "99"),
        }
    }
}


impl std::str::FromStr for CommonViewClass {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("FF") {
            Ok(Self::MultipleChannel)
        } else if s.eq("99") {
            Ok(Self::SingleChannel)
        } else {
            Err(Error::BadCommonViewClass)
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn cv_class() {
        assert_eq!(format!("{:X}", CommonViewClass::MultiChannel), "FF");
        assert_eq!(format!("{:X}", CommonViewClass::SingleChannel), "99");
        assert_eq!(CommonViewClas::from_str("FF"), Ok(CommonViewClass::MultiChannel));
        assert_eq!(CommonViewClas::from_str("FF"), Ok(CommonViewClass::MultiChannel));
    }
}
