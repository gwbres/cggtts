//! CGGTTS Format
//! Only 2E Version (latest) supported at the moment
//! Refer to official doc: https://www.bipm.org/wg/CCTF/WGGNSS/Allowed/Format_CGGTTS-V2E/CGTTS-V2E-article_versionfinale_cor.pdf

use std::fmt;
use scan_fmt::scan_fmt;
use std::io::ErrorKind;

/// track description
mod track;

/// currently CGGTTS file version supported
const CGGTTS_VERSION: &str = "2E";

#[allow(dead_code)]
pub struct Cggtts {
    version: String, // file version info
    rev_date: chrono::NaiveDate, // header data rev. date
    date: chrono::NaiveDate, // production date
    lab: String, // LAB where measurements were performed (possibly unknown)
    recvr: Rcvr, // possible GNSS receiver infos
    nb_channels: u16, // nb of GNSS receiver channels
    ims: Option<Rcvr>, // IMS Ionospheric Measurement System (if any)
    xyz: (f32,f32,f32), // antenna phase center coordinates [in m]
    frame: String,
    comments: Option<String>, // comments (if any)
    sys_dly: f32, // total system delay (internal + cable delay) [in ns]
    cab_dly: f32, // delay in antenna to GNSS RX [in ns]
    ref_dly: f32, // local GNSS RX to local clock delay
    reference: String,
    tracks: Vec<track::CggttsTrack>
}

/*
 * GNSS receiver & other system descriptor
 */
#[derive(Clone, Debug)]
struct Rcvr {
    manufacturer: String,
    recv_type: String,
    serial_number: String,
    year: u16,
    software_number: String,
}

#[allow(dead_code)]
impl Cggtts {
    pub fn new (file_path: &str) -> Result<Cggtts, ErrorKind> {
        let mut chksum: u8 = 0;
        let file_content = std::fs::read_to_string(file_path).unwrap();
        let mut lines = file_content.split("\n").map(|x| x.to_string()).into_iter();

        let mut sys_dly: Option<f32> = Some(0.0);
        let mut ref_dly: Option<f32> = Some(0.0);
        let mut cab_dly: Option<f32> = Some(0.0);

        // Identify date from MJD contained in file name
        let point_index = file_path.find(".").unwrap(); // will panic if filename does not match naming convention
        let mjd_str = file_path.to_string().split_off(point_index-2);
        let mjd: f32 = mjd_str.parse().unwrap();

        // Version line
        let line = lines.next().unwrap();
        let version = scan_fmt!(&line, "CGGTTS GENERIC DATA FORMAT VERSION = {}", String).unwrap();
        if !version.eq(CGGTTS_VERSION) {
            println!("CGTTS Version '{}' is not supported", version);
            return Err(ErrorKind::InvalidData);
        }

        // CRC is the %256 summation
        // of all ASCII bytes contained in the header
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // REV DATE
        let line = lines.next().unwrap();
        let rev_date_str = scan_fmt!(&line, "REV DATE = {}", String).unwrap();
        let rev_date = chrono::NaiveDate::parse_from_str(rev_date_str.trim(), " %Y-%m-%d").unwrap();

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len() {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // RCVR
        let rcvr_infos: Rcvr;
        let line = lines.next().unwrap();
        match scan_fmt! (&line, "RCVR = {} {} {} {d} {}", String, String, String, String, String) {
            (Some(manufacturer),
            Some(recv_type),
            Some(serial_number),
            Some(year),
            Some(software_number)) => rcvr_infos = Rcvr{manufacturer, recv_type, serial_number, year: u16::from_str_radix(&year, 10).unwrap(), software_number},
            _ => {
                println!("Failed to parse RCVR infos from CGGTTS file");
                return Err(ErrorKind::InvalidData);
            }
        }

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // CHANNEL
        let line = lines.next().unwrap();
        let nb_channels = scan_fmt!(&line, "CH = {d}", u16).unwrap();

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // IMS
        let mut ims_infos: Option<Rcvr> = None;
        let line = lines.next().unwrap();
        if !line.contains("IMS = 99999") // IMS data is provided
        {
            match scan_fmt! (&line, "IMS = {} {} {} {d} {}", String, String, String, String, String)
            {
                (Some(manufacturer),
                    Some(recv_type),
                    Some(serial_number),
                    Some(year),
                    Some(software_number)) => ims_infos = Some(Rcvr{manufacturer, recv_type, serial_number, year: u16::from_str_radix(&year, 10).unwrap(), software_number}),
                _ => {
                    println!("Failed to parse IMS infos from CGGTTS file");
                    return Err(ErrorKind::InvalidData);
                }
            }
        }

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // LAB
        let line = lines.next().unwrap();

        let mut lab = scan_fmt!(&line, "LAB = {}", String).unwrap();
        if lab.eq("ABC")
        {
            lab = String::from("Unknown");
        }

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // X
        let line = lines.next().unwrap();
        let x = scan_fmt!(&line, "X = {f}", f32).unwrap();

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // Y
        let line = lines.next().unwrap();
        let y = scan_fmt!(&line, "Y = {f}", f32).unwrap();

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // Z
        let line = lines.next().unwrap();
        let z = scan_fmt!(&line, "Z = {f}", f32).unwrap(); // TODO: to be fixed in next release!

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // FRAME
        let line = lines.next().unwrap();
        let frame = scan_fmt!(&line, "FRAME = {}", String).unwrap();

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // COMMENTS
        let mut comments: Option<String> = None;
        let line = lines.next().unwrap();
        let cmts = line.split("=").nth(1).unwrap().trim();
        if !cmts.eq("NO COMMENTS")
        {
            comments = Some(cmts.to_string());
        }

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // SYS DLY
        let line = lines.next().unwrap();
        match scan_fmt!(&line, "SYS DLY = {f} {} {}", f32, String, String)
        {
            (Some(dly), Some(scaling), Some(_)) => {
                if scaling.eq("ms")
                {
                    sys_dly = Some(dly * 1e-3);
                } else if scaling.eq("us")
                {
                    sys_dly = Some(dly * 1e-6);
                } else if scaling.eq("ns") {
                    sys_dly = Some(dly * 1e-9);
                } else if scaling.eq("ps") {
                    sys_dly = Some(dly * 1e-12);
                } else if scaling.eq("fs") {
                    sys_dly = Some(dly * 1e-15);
                }
            },
            _ => {
                println!("Failed to parse SYS (system) Delay line");
                return Err(ErrorKind::InvalidData);
            }
        }

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // CAB DLY
        let line = lines.next().unwrap();
        match scan_fmt!(&line, "CAB DLY = {f} {}", f32, String)
        {
            (Some(dly), Some(scaling)) => {
                if scaling.eq("ms")
                {
                    cab_dly = Some(dly * 1e-3);
                } else if scaling.eq("us")
                {
                    cab_dly = Some(dly * 1e-6);
                } else if scaling.eq("ns") {
                    cab_dly = Some(dly * 1e-9);
                } else if scaling.eq("ps") {
                    cab_dly = Some(dly * 1e-12);
                } else if scaling.eq("fs") {
                    cab_dly = Some(dly * 1e-15);
                }
            },
            _ => {
                println!("Failed to parse CAB (cable) Delay line");
                return Err(ErrorKind::InvalidData);
            }
        }

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // REF DLY
        let line = lines.next().unwrap();
        match scan_fmt!(&line, "REF DLY = {f} {}", f32, String)
        {
            (Some(dly), Some(scaling)) => {
                if scaling.eq("ms")
                {
                    ref_dly = Some(dly * 1e-3);
                } else if scaling.eq("us")
                {
                    ref_dly = Some(dly * 1e-6);
                } else if scaling.eq("ns") {
                    ref_dly = Some(dly * 1e-9);
                } else if scaling.eq("ps") {
                    ref_dly = Some(dly * 1e-12);
                } else if scaling.eq("fs") {
                    ref_dly = Some(dly * 1e-15);
                }
            },
            _ => {
                println!("Failed to parse REF (Reference) Delay line");
                return Err(ErrorKind::InvalidData);
            }
        }

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // REFERENCE
        let line = lines.next().unwrap();
        let reference = scan_fmt!(&line, "REF = {}", String).unwrap();

        // CRC
        let bytes = line.clone().into_bytes();
        for i in 0..bytes.len()
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        // CKSUM
        let line = lines.next().unwrap();
        let cksum_parsed = scan_fmt!(&line, "CKSUM = {x}", String).unwrap();
        let cksum = u8::from_str_radix(&cksum_parsed, 16).unwrap();

        // CRC calc. ends on 'CHKSUM = ' (line 15)
        let end_pos = line.find("= ").unwrap();
        let bytes = line.clone().into_bytes();
        for i in 0..end_pos+2
        {
            chksum = chksum.wrapping_add(bytes[i]);
        }

        chksum = chksum.wrapping_add(15*10); // 15*'\n', TODO: faulty syref2-5.c ??

        // Verify Checksum
        if chksum != cksum
        {
            println!("CGGTTS file checksum error - Found '{}' while '{}' was locally computed", cksum, chksum);
            return Err(ErrorKind::InvalidData);
        }

        let _ = lines.next().unwrap(); // Blank line
        let _ = lines.next().unwrap(); // labels line
        let _ = lines.next().unwrap(); // units line currently discarded, TODO: to be improved

        // Parsing tracks
        let mut tracks: Vec<track::CggttsTrack> = Vec::new();
        loop {
            // grab new line
            let line = match lines.next() {
                Some(s) => s,
                _ => break // we're done parsing
            };
            if line.len() == 0 { // empty line
                break // we're done parsing
            }
            tracks.push (track::CggttsTrack::new(&line).unwrap());
        }

        return Ok(Cggtts {
                version: CGGTTS_VERSION.to_string(),
                rev_date,
                date: mjd_to_date((mjd * 1000.0) as u32), /* TODO @GBR conversion mjd est a revoir */
                nb_channels,
                recvr: rcvr_infos,
                ims: ims_infos,
                lab,
                xyz: (x,y,z),
                frame,
                comments,
                sys_dly: sys_dly.unwrap(),
                cab_dly: cab_dly.unwrap(),
                ref_dly: ref_dly.unwrap(),
                reference,
                tracks
            }
        );
    }

    pub fn get_date (&self) -> &chrono::NaiveDate { &self.date }

    /* retuns requested track in self */
    pub fn get_track (&self, index: usize) -> Option<&track::CggttsTrack> { self.tracks.get(index) }

    /* returns latest track to date */
    pub fn get_latest_track (&self) -> Option<&track::CggttsTrack> {
        self.tracks.get(self.tracks.len()-1)
    }

    /* returns earlist track in date */
    pub fn get_earliest_track (&self) -> Option<&track::CggttsTrack> {
        self.tracks.get(0)
    }
}

// custom display formatter
impl fmt::Display for Cggtts {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "Version: '{}' | REV DATE '{}' | LAB '{}' | Nb Channels {}\nRECVR: {:?}\nIMS: {:?}\nXYZ: {:?}\nFRAME: {}\nCOMMENTS: {:#?}\nDELAYS | SYS: {} ns | CAB: {} ns | TOTAL {} ns\nREFERENCE: {}\n",
            self.version, self.rev_date, self.lab, self.nb_channels,
            self.recvr,
            self.ims,
            self.xyz,
            self.frame,
            self.comments,
            self.sys_dly, self.cab_dly, self.ref_dly,
            self.reference,
        ).unwrap();
        write! (f, "-------------------------\n").unwrap();
        for i in 0..self.tracks.len() {
            write! (f, "MEAS #{}: {}\n",i, self.tracks[i]).unwrap()
        }
        write!(f, "\n")
    }
}

/* converts MJD to chrono::naivedate */
fn mjd_to_date (mjd: u32) -> chrono::NaiveDate {
    let z = 2400000.5 + 0.5 + mjd as f32; // julian day 0h
    let alpha: u32 = ((z - 867216.25) /  36524.25) as u32;
    let s: u32 = z as u32 +1 +alpha - alpha / 4;
    let b: u32 = s + 1524;
    let c: u32 = ((b as f32 -122.1)/365.25) as u32;
    let d: u32 = (365.25 * c as f32) as u32;
    let e: u32 = ((b-d) as f32 / 30.6001) as u32;
    let q = b - d - ((30.6001 * e as f32) as u32);
    let m: u32;
    let mut annee: u32 = c - 4715;

    if e < 14 {
        m = e-1;
    } else {
        m = e-13;
    }

    if m > 2 {
        annee = c - 4716;
    }

    return chrono::NaiveDate::from_ymd(annee as i32, m, q)
}
/*
#[cfg(test)]
mod test
{
    use chrono::Datelike;
    use crate::syref::cggtts::Cggtts;
    use crate::syref::cggtts::mjd_to_date;

    #[test]
    fn test_cggtts_with_real_data1 ()
    {   // Test constructor with sample data
        match Cggtts::new ("src/syref/cggtts/tests/GZSY8251.412")
        {
            Ok(c) => println!("Found {}", c),
            Err(_) => {
                println!("CGGTTS file parsing test failed!");
                assert!(false);
            }
        }
    }

    #[test]
    fn test_cggtts_with_real_data2 ()
    {   // Test constructor with sample data
        match Cggtts::new ("src/syref/cggtts/tests/GZSY8259.159")
        {
            Ok(c) => println!("Found {}", c),
            Err(_) => {
                println!("CGGTTS file parsing test failed!");
                assert!(false);
            }
        }
    }

    #[test]
    fn test_mdj_convertion ()
    {
        let date = mjd_to_date(57000);
        assert_eq!(date.year(), 2014);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 9);
    }
}
*/
