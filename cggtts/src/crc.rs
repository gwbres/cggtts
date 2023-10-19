use thiserror::Error;

#[derive(PartialEq, Debug, Error)]
pub enum Error {
    #[error("can only calculate over valid utf8 data")]
    NonUtf8Data,
    #[error("checksum error, got \"{0}\" but \"{1}\" locally computed")]
    ChecksumError(u8, u8),
}

/// computes crc for given str content
pub(crate) fn calc_crc(content: &str) -> Result<u8, CrcError> {
    match content.is_ascii() {
        true => {
            let mut ck: u8 = 0;
            let mut ptr = content.encode_utf16();
            for _ in 0..ptr.clone().count() {
                ck = ck.wrapping_add(ptr.next().unwrap() as u8)
            }
            Ok(ck)
        },
        false => return Err(Error::NonUtf8Data),
    }
}
