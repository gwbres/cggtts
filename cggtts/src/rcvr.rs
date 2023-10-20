/// GNSS Receiver description
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rcvr {
    /// Manufacturer of this hardware
    manufacturer: String,
    /// Type of receiver
    recv_type: String,
    /// Receiver's serial number
    serial_number: String,
    /// Receiver manufacturing year
    year: u16,
    /// Receiver software revision number
    release: String,
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

impl std::fmt::Display for Rcvr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&self.manufacturer)?;
        fmt.write_str(" ")?;
        fmt.write_str(&self.recv_type)?;
        fmt.write_str(" ")?;
        fmt.write_str(&self.serial_number)?;
        fmt.write_str(" ")?;
        fmt.write_str(&self.year.to_string())?;
        fmt.write_str(" ")?;
        fmt.write_str(&self.release)?;
        Ok(())
    }
}
