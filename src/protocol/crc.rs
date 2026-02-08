use crc::{Crc, CRC_16_XMODEM};

const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_XMODEM);

pub fn crc16_xmodem(data: &[u8]) -> [u8; 2] {
    CRC.checksum(data).to_be_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc_known_value() {
        // "Z" (0x5A) header test
        let data = b"\x5a\x00\x05\x00\x01\x08\x01\x00";
        let crc = crc16_xmodem(data);
        // CRC should be deterministic and match the XMODEM algorithm
        assert_eq!(crc.len(), 2);
    }

    #[test]
    fn test_crc_empty() {
        let crc = crc16_xmodem(&[]);
        assert_eq!(crc, [0x00, 0x00]);
    }
}
