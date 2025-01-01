use crate::{
    buffer::Utf8Buffer,
    errors::FormattingError,
    prelude::{Header, Version},
};

use std::io::{BufWriter, Write};

impl Header {
    /// Formats this [CGGTTS] following standard specifications.
    pub fn format<W: Write>(
        &self,
        writer: &mut BufWriter<W>,
        buf: &mut Utf8Buffer,
    ) -> Result<(), FormattingError> {
        // clear potential past residues
        buf.clear();

        buf.push_str(&format!(
            "CGGTTS GENERIC DATA FORMAT VERSION = {}\n",
            Version::Version2E,
        ));

        buf.push_str(&format!("REV DATE = {:x}\n", &self.version));

        if let Some(gnss_rx) = &self.receiver {
            buf.push_str(&format!("RCVR = {:x}\n", gnss_rx));
        }

        buf.push_str(&format!("CH = {}\n", self.nb_channels));

        if let Some(ims) = &self.ims_hardware {
            buf.push_str(&format!("IMS = {:x}\n", ims));
        }

        buf.push_str(&format!("LAB = {}\n", self.station));

        buf.push_str(&format!("X = {:12.3} m\n", self.apc_coordinates.x));
        buf.push_str(&format!("Y = {:12.3} m\n", self.apc_coordinates.y));
        buf.push_str(&format!("Z = {:12.3} m\n", self.apc_coordinates.z));

        if let Some(frame) = &self.reference_frame {
            buf.push_str(&format!("FRAME = {}\n", frame));
        }

        if let Some(comments) = &self.comments {
            buf.push_str(&format!("COMMENTS = {}\n", comments.trim()));
        } else {
            buf.push_str(&format!("COMMENTS = NO COMMENTS\n"));
        }

        // TODO system delay formatting
        // let delays = self.delay.delays.clone();
        // let constellation = if !self.tracks.is_empty() {
        //     self.tracks[0].sv.constellation
        // } else {
        //     Constellation::default()
        // };

        // if delays.len() == 1 {
        //     // Single frequency
        //     let (code, value) = delays[0];
        //     match value {
        //         Delay::Internal(v) => {
        //             content.push_str(&format!(
        //                 "INT DLY = {:.1} ns ({:X} {})\n",
        //                 v, constellation, code
        //             ));
        //         },
        //         Delay::System(v) => {
        //             content.push_str(&format!(
        //                 "SYS DLY = {:.1} ns ({:X} {})\n",
        //                 v, constellation, code
        //             ));
        //         },
        //     }
        //     if let Some(cal_id) = &self.delay.cal_id {
        //         content.push_str(&format!("       CAL_ID = {}\n", cal_id));
        //     } else {
        //         content.push_str("       CAL_ID = NA\n");
        //     }
        // } else if delays.len() == 2 {
        //     // Dual frequency
        //     let (c1, v1) = delays[0];
        //     let (c2, v2) = delays[1];
        //     match v1 {
        //         Delay::Internal(_) => {
        //             content.push_str(&format!(
        //                 "INT DLY = {:.1} ns ({:X} {}), {:.1} ns ({:X} {})\n",
        //                 v1.value(),
        //                 constellation,
        //                 c1,
        //                 v2.value(),
        //                 constellation,
        //                 c2
        //             ));
        //         },
        //         Delay::System(_) => {
        //             content.push_str(&format!(
        //                 "SYS DLY = {:.1} ns ({:X} {}), {:.1} ns ({:X} {})\n",
        //                 v1.value(),
        //                 constellation,
        //                 c1,
        //                 v2.value(),
        //                 constellation,
        //                 c2
        //             ));
        //         },
        //     }
        //     if let Some(cal_id) = &self.delay.cal_id {
        //         content.push_str(&format!("     CAL_ID = {}\n", cal_id));
        //     } else {
        //         content.push_str("     CAL_ID = NA\n");
        //     }
        // }

        buf.push_str(&format!(
            "CAB DLY = {:05.1} ns\n",
            self.delay.antenna_cable_delay,
        ));

        buf.push_str(&format!(
            "REF DLY = {:05.1} ns\n",
            self.delay.local_ref_delay
        ));

        buf.push_str(&format!("REF = {}\n", self.reference_time));

        // push last bytes contributing to CRC
        buf.push_str("CKSUM = ");

        // Run CK calculation
        let ck = buf.calculate_crc();

        // Append CKSUM
        buf.push_str(&format!("{:02X}\n", ck));

        // interprate
        let ascii_utf8 = buf.to_utf8_ascii()?;

        // forward to user
        write!(writer, "{}", ascii_utf8)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use std::io::BufWriter;
    use std::path::Path;

    use crate::{buffer::Utf8Buffer, CGGTTS};

    #[test]
    fn header_crc_buffering() {
        let mut utf8 = Utf8Buffer::new(1024);
        let mut buf = BufWriter::new(Utf8Buffer::new(1024));

        // This file does not have comments
        // run once
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("single")
            .join("GZSY8259.506");

        let cggtts = CGGTTS::from_file(path).unwrap();

        let header = &cggtts.header;

        header.format(&mut buf, &mut utf8).unwrap();

        let inner = buf.into_inner().unwrap_or_else(|_| panic!("oops"));
        let ascii_utf8 = inner.to_utf8_ascii().expect("generated invalid utf-8!");

        let expected = "CGGTTS GENERIC DATA FORMAT VERSION = 2E
REV DATE = 2014-02-20
RCVR = GORGYTIMING SYREF25 18259999 2018 v00
CH = 12
LAB = SY82
X =  4314137.334 m
Y =   452632.813 m
Z =  4660706.403 m
FRAME = ITRF
COMMENTS = NO COMMENTS
CAB DLY = 000.0 ns
REF DLY = 000.0 ns
REF = REF(SY82)
CKSUM = C7";

        for (content, expected) in ascii_utf8.lines().zip(expected.lines()) {
            assert_eq!(content, expected);
        }
    }
}
