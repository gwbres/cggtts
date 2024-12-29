use crate::{
    errors::ParsingError,
    header::{CalibrationID, Code, Coordinates, Delay, SystemDelay},
    prelude::{Epoch, Hardware, Header, ReferenceTime, Version},
};

use scan_fmt::scan_fmt;

use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

impl Header {
    /// Parse [Header] from any [Read]able input.
    pub fn parse<R: Read>(reader: &mut BufReader<R>) -> Result<Self, ParsingError> {
        let mut lines_iter = reader.lines();

        // init variables
        let mut system_delay = SystemDelay::default();

        let mut header_ck;
        let mut cksum = 0_u8;

        let (mut blank, mut field_labels, mut unit_labels) = (false, false, false);

        let mut release_date = Epoch::default();
        let mut nb_channels: u16 = 0;

        let mut receiver: Option<Hardware> = None;
        let mut ims_hardware: Option<Hardware> = None;

        let mut station = String::from("LAB");
        let mut comments: Option<String> = None;
        let mut reference_frame: Option<String> = None;
        let mut apc_coordinates = Coordinates::default();
        let mut reference_time = ReferenceTime::default();
        let (_x, _y, _z): (f64, f64, f64) = (0.0, 0.0, 0.0);

        // VERSION must come first
        let version = lines_iter.next().ok_or(ParsingError::VersionFormat)?;
        let version = version.map_err(|_| ParsingError::VersionFormat)?;

        let version = match scan_fmt!(&version, "CGGTTS GENERIC DATA FORMAT VERSION = {}", String) {
            Some(version) => Version::from_str(&version)?,
            _ => return Err(ParsingError::VersionFormat),
        };

        for line in lines_iter {
            if line.is_err() {
                continue;
            }

            let line = line.unwrap();

            if line.starts_with("REV DATE = ") {
                match scan_fmt!(&line, "REV DATE = {d}-{d}-{d}", i32, u8, u8) {
                    (Some(y), Some(m), Some(d)) => {
                        release_date = Epoch::from_gregorian_utc_at_midnight(y, m, d);
                    },
                    _ => {
                        return Err(ParsingError::RevisionDateFormat);
                    },
                }
            } else if line.starts_with("RCVR = ") {
                match scan_fmt!(
                    &line,
                    "RCVR = {} {} {} {d} {}",
                    String,
                    String,
                    String,
                    u16,
                    String
                ) {
                    (
                        Some(manufacturer),
                        Some(recv_type),
                        Some(serial_number),
                        Some(year),
                        Some(release),
                    ) => {
                        receiver = Some(
                            Hardware::default()
                                .with_manufacturer(&manufacturer)
                                .with_model(&recv_type)
                                .with_serial_number(&serial_number)
                                .with_release_year(year)
                                .with_release_version(&release),
                        );
                    },
                    _ => {},
                }
            } else if line.starts_with("CH = ") {
                match scan_fmt!(&line, "CH = {d}", u16) {
                    Some(n) => nb_channels = n,
                    _ => {},
                };
            } else if line.starts_with("IMS = ") {
                match scan_fmt!(
                    &line,
                    "IMS = {} {} {} {d} {}",
                    String,
                    String,
                    String,
                    u16,
                    String
                ) {
                    (
                        Some(manufacturer),
                        Some(recv_type),
                        Some(serial_number),
                        Some(year),
                        Some(release),
                    ) => {
                        ims_hardware = Some(
                            Hardware::default()
                                .with_manufacturer(&manufacturer)
                                .with_model(&recv_type)
                                .with_serial_number(&serial_number)
                                .with_release_year(year)
                                .with_release_version(&release),
                        );
                    },
                    _ => {},
                }
            } else if line.starts_with("LAB = ") {
                match line.strip_prefix("LAB = ") {
                    Some(s) => {
                        station = s.trim().to_string();
                    },
                    _ => {},
                }
            } else if line.starts_with("X = ") {
                match scan_fmt!(&line, "X = {f}", f64) {
                    Some(f) => {
                        apc_coordinates.x = f;
                    },
                    _ => {},
                }
            } else if line.starts_with("Y = ") {
                match scan_fmt!(&line, "Y = {f}", f64) {
                    Some(f) => {
                        apc_coordinates.y = f;
                    },
                    _ => {},
                }
            } else if line.starts_with("Z = ") {
                match scan_fmt!(&line, "Z = {f}", f64) {
                    Some(f) => {
                        apc_coordinates.z = f;
                    },
                    _ => {},
                }
            } else if line.starts_with("FRAME = ") {
                let frame = line.split_at(7).1.trim();
                if !frame.eq("?") {
                    reference_frame = Some(frame.to_string())
                }
            } else if line.starts_with("COMMENTS = ") {
                let c = line.strip_prefix("COMMENTS =").unwrap().trim();
                if !c.eq("NO COMMENTS") {
                    comments = Some(c.to_string());
                }
            } else if line.starts_with("REF = ") {
                if let Some(s) = scan_fmt!(&line, "REF = {}", String) {
                    reference_time = ReferenceTime::from_str(&s)
                }
            } else if line.contains("DLY = ") {
                let items: Vec<&str> = line.split_ascii_whitespace().collect();

                let dual_carrier = line.contains(',');

                if items.len() < 4 {
                    continue; // format mismatch
                }

                match items[0] {
                    "CAB" => system_delay.antenna_cable_delay = f64::from_str(items[3])?,
                    "REF" => system_delay.local_ref_delay = f64::from_str(items[3])?,
                    "SYS" => {
                        if line.contains("CAL_ID") {
                            let offset =
                                line.rfind('=').ok_or(ParsingError::InvalidCalibrationId)?;

                            if let Ok(cal_id) = CalibrationID::from_str(&line[offset + 1..]) {
                                system_delay = system_delay.with_calibration_id(cal_id);
                            }
                        }

                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::System(value)));
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[9].replace(')', "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::System(value)));
                                }
                            }
                        } else {
                            let value = f64::from_str(items[3]).unwrap();
                            let code = items[6].replace(')', "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay
                                    .freq_dependent_delays
                                    .push((code, Delay::System(value)));
                            }
                        }
                    },
                    "INT" => {
                        if line.contains("CAL_ID") {
                            let offset =
                                line.rfind('=').ok_or(ParsingError::InvalidCalibrationId)?;

                            if let Ok(cal_id) = CalibrationID::from_str(&line[offset + 1..]) {
                                system_delay = system_delay.with_calibration_id(cal_id);
                            }
                        }

                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::Internal(value)));
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[10].replace(')', "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::Internal(value)));
                                }
                            }
                        } else if let Ok(value) = f64::from_str(items[3]) {
                            let code = items[6].replace(')', "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay
                                    .freq_dependent_delays
                                    .push((code, Delay::Internal(value)));
                            }
                        }
                    },
                    "TOT" => {
                        if line.contains("CAL_ID") {
                            let offset =
                                line.rfind('=').ok_or(ParsingError::InvalidCalibrationId)?;

                            if let Ok(cal_id) = CalibrationID::from_str(&line[offset + 1..]) {
                                system_delay = system_delay.with_calibration_id(cal_id);
                            }
                        }

                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::System(value)));
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[9].replace(')', "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::System(value)));
                                }
                            }
                        } else if let Ok(value) = f64::from_str(items[3]) {
                            let code = items[6].replace(')', "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay
                                    .freq_dependent_delays
                                    .push((code, Delay::System(value)));
                            }
                        }
                    },
                    _ => {}, // non recognized delay type
                };
            } else if line.starts_with("CKSUM = ") {
                // CKSUM terminates this section

                // verify CK value
                header_ck = match scan_fmt!(&line, "CKSUM = {x}", String) {
                    Some(s) => match u8::from_str_radix(&s, 16) {
                        Ok(hex) => hex,
                        _ => return Err(ParsingError::ChecksumParsing),
                    },
                    _ => return Err(ParsingError::ChecksumFormat),
                };

                let end_pos = line.find("= ").unwrap();
                //cksum = cksum.wrapping_add(crc_calculation(line.split_at(end_pos + 2).0)?);

                //if cksum != header_ck {
                //    //return Err(Error::ChecksumError(crc::Error::ChecksumError(cksum, ck)));
                //}

                blank = true;
            } else if blank {
                // Field labels expected next
                blank = false;
                field_labels = true;
            } else if field_labels {
                // Unit labels expected next
                field_labels = false;
                unit_labels = true;
            } else if unit_labels {
                // last line that concludes this section
                break;
            } else {
                // every single line (except comments) contributes to CRC calculation
                if !line.starts_with("COMMENTS = ") {
                    //cksum = cksum.wrapping_add(crc_calculation(&line)?);
                }
            }
        }

        Ok(Self {
            version,
            release_date,
            nb_channels,
            receiver,
            ims_hardware,
            station,
            reference_frame,
            apc_coordinates,
            comments,
            delay: system_delay,
            reference_time,
        })
    }
}
