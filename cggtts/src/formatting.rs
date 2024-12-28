use crate::prelude::{Version, CGGTTS};

use std::io::{BufWriter, Write};

/// CGGTTS production is currently permitted by "Displaying"
/// the [CGGTTS] structure.
///
/// Basic [CGGTTS] example:
///
/// To produce valid advanced CGGTTS, one should specify:
/// - IMS [Hardware]
/// - ionospheric parameters
/// - System delay definitions, per signal carrier
///
/// ```
/// use cggtts::prelude::CGGTTS;
/// use std::io::Write;
///
/// let rcvr = Hardware::default()
/// .with_manufacturer("SEPTENTRIO")  
/// .with_model("POLARRx5")
///  .with_serial_number("#12345")
///  .with_release_year(2023)
///  .with_release_version("v1");
///
///  let mut cggtts = CGGTTS::default()
///    .with_station("AJACFR")
///    .with_receiver_hardware(rcvr)
///    .with_apc_coordinates(Coordinates {
///       x: 0.0_f64,
///       y: 0.0_f64,
///       z: 0.0_f64,
///    })
///    .with_reference_time(ReferenceTime::UTCk("LAB".to_string()))
///    .with_reference_frame("ITRF");
///         
///
///     // TrackData is mandatory
///     let data = TrackData {
///         refsv: 0.0_f64,
///         srsv: 0.0_f64,
///         refsys: 0.0_f64,
///         srsys: 0.0_f64,
///         dsg: 0.0_f64,
///         ioe: 0_u16,
///         smdt: 0.0_f64,
///         mdtr: 0.0_f64,
///         mdio: 0.0_f64,
///         smdi: 0.0_f64,
///     };
///
///     // tracking parameters
///     let epoch = Epoch::default();
///     let sv = SV::default();
///     let (elevation, azimuth) = (0.0_f64, 0.0_f64);
///     let duration = Duration::from_seconds(780.0);
///
///     // receiver channel being used
///     let rcvr_channel = 0_u8;
///
///     // option 1: track resulting from a single SV observation
///     let track = Track::new(
///         sv,
///         epoch,
///         duration,
///         CommonViewClass::SingleChannel,
///         elevation,
///         azimuth,
///         data,
///         None,
///         rcvr_channel,
///         "L1C",
///     );

///     cggtts.tracks.push(track);
///     let mut fd = std::fs::File::create("test.txt") // does not respect naming conventions
///         .unwrap();
///     write!(fd, "{}", cggtts).unwrap();
/// }
/// ```
impl CGGTTS {
    /// Formats this [CGGTTS] following standard specifications.
    pub fn format<W: Write>(&self, writer: &mut BufWriter<W>) -> std::io::Result<()> {
        const TRACK_LABELS_WITH_IONOSPHERIC_DATA: &str =
            "SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS DSG IOE MDTR SMDT MDIO SMDI MSIO SMSI ISG FR HC FRC CK";

        const UNIT_LABELS_WITH_IONOSPHERIC : &str = "             hhmmss  s  .1dg .1dg    .1ns     .1ps/s     .1ns    .1ps/s .1ns     .1ns.1ps/s.1ns.1ps/s.1ns.1ps/s.1ns";

        const TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA: &str =
            "SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS  DSG IOE MDTR SMDT MDIO SMDI FR HC FRC CK";

        let mut content = String::new();

        writeln!(
            writer,
            "CGGTTS GENERIC DATA FORMAT VERSION = {}",
            Version::Version2E,
        )?;

        writeln!(writer, "REV DATE = {:x}", &self.version)?;

        if let Some(gnss_rx) = &self.receiver {
            writeln!(writer, "RCVR = {:x}", gnss_rx)?;
        }

        writeln!(writer, "CH = {}", self.nb_channels)?;

        if let Some(ims) = &self.ims_hardware {
            writeln!(writer, "IMS = {:x}", ims)?;
        }

        writeln!(writer, "LAB = {}", self.station)?;

        writeln!(writer, "X = {}", self.apc_coordinates.x)?;
        writeln!(writer, "Y = {}", self.apc_coordinates.y)?;
        writeln!(writer, "Z = {}", self.apc_coordinates.z)?;

        if let Some(frame) = &self.reference_frame {
            writeln!(writer, "FRAME = {}", frame)?;
        }

        if let Some(comments) = &self.comments {
            writeln!(writer, "COMMENTS = {}", comments.trim())?;
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

        writeln!(writer, "CAB DLY = {:.1} ns", self.delay.antenna_cable_delay)?;
        writeln!(writer, "REF DLY = {:.1} ns", self.delay.local_ref_delay)?;
        writeln!(writer, "REF = {}", self.reference_time,)?;

        // CKSUM + BLANK
        writeln!(writer, "{}", "CKSUM = 00\n")?;

        if self.has_ionospheric_data() {
            writeln!(writer, "{}", TRACK_LABELS_WITH_IONOSPHERIC_DATA,)?;

            writeln!(writer, "{}", UNIT_LABELS_WITH_IONOSPHERIC,)?;
        } else {
            content.push_str(TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA);
            content.push_str("             hhmmss  s  .1dg .1dg    .1ns     .1ps/s     .1ns    .1ps/s .1ns     .1ns.1ps/s.1ns.1ps/s\n");
        }

        for track in self.tracks.iter() {
            writeln!(writer, "{}", track)?;
        }

        Ok(())
    }
}
