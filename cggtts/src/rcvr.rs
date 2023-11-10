/// GNSS Receiver description
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rcvr {
    /// Manufacturer of this hardware
    pub manufacturer: String,
    /// Type of receiver
    pub recv_type: String,
    /// Receiver's serial number
    pub serial_number: String,
    /// Receiver manufacturing year
    pub year: u16,
    /// Receiver software revision number
    pub release: String,
}

impl Rcvr {
    pub fn manufacturer(&self, man: &str) -> Self {
        let mut s = self.clone();
        s.manufacturer = man.to_string();
        s
    }
    pub fn receiver(&self, rcvr: &str) -> Self {
        let mut s = self.clone();
        s.recv_type = rcvr.to_string();
        s
    }
    pub fn serial_number(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.serial_number = sn.to_string();
        s
    }
    pub fn year(&self, y: u16) -> Self {
        let mut s = self.clone();
        s.year = y;
        s
    }
    pub fn release(&self, release: &str) -> Self {
        let mut s = self.clone();
        s.release = release.to_string();
        s
    }
}

impl std::fmt::UpperHex for Rcvr {
    /*
     * Formats Self as in file Header
     */
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&format!(
            "{} {} {} {} {}",
            self.manufacturer, self.recv_type, self.serial_number, self.year, self.release
        ))
    }
}

impl std::fmt::LowerHex for Rcvr {
    /*
     * Formats Self as used in file name generation
     */
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let max_offset = std::cmp::min(self.serial_number.len(), 2);
        fmt.write_str(&self.serial_number[0..max_offset])
    }
}
