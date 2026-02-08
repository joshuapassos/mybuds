use std::collections::BTreeMap;

use anyhow::{bail, ensure, Result};

use super::crc::crc16_xmodem;

/// Magic byte at the start of every Huawei SPP packet.
const MAGIC: u8 = 0x5A;

/// A Huawei SPP protocol packet.
///
/// Wire format:
/// ```text
/// [0x5A] [length: 2 bytes BE] [0x00] [command_id: 2 bytes] [TLV params...] [CRC16: 2 bytes]
/// ```
///
/// Where `length` = size of (0x00 byte + command_id + params), i.e. body_len + 1.
/// TLV param: [type: 1 byte] [length: 1 byte] [value: `length` bytes]
#[derive(Debug, Clone)]
pub struct HuaweiSppPacket {
    pub command_id: [u8; 2],
    pub parameters: BTreeMap<u8, Vec<u8>>,
}

impl HuaweiSppPacket {
    /// Create a new packet with the given command ID and no parameters.
    pub fn new(command_id: [u8; 2]) -> Self {
        Self {
            command_id,
            parameters: BTreeMap::new(),
        }
    }

    /// Build a read request: command + empty-value parameters for the given param types.
    pub fn read_request(command_id: [u8; 2], param_types: &[u8]) -> Self {
        let mut pkt = Self::new(command_id);
        for &t in param_types {
            pkt.parameters.insert(t, Vec::new());
        }
        pkt
    }

    /// Build a write request with typed parameters.
    pub fn write_request(command_id: [u8; 2], params: &[(u8, Vec<u8>)]) -> Self {
        let mut pkt = Self::new(command_id);
        for (t, v) in params {
            pkt.parameters.insert(*t, v.clone());
        }
        pkt
    }

    /// Convenience: get a parameter value, returning empty slice if not present.
    pub fn find_param(&self, param_type: u8) -> &[u8] {
        self.parameters
            .get(&param_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Serialize this packet to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        // Build body: command_id + TLV parameters
        let mut body = Vec::new();
        body.extend_from_slice(&self.command_id);
        for (&p_type, p_value) in &self.parameters {
            body.push(p_type);
            body.push(p_value.len() as u8);
            body.extend_from_slice(p_value);
        }

        // Build packet: magic + length(2 BE) + 0x00 + body + CRC16
        let length = (body.len() + 1) as u16; // +1 for the 0x00 byte
        let mut result = Vec::with_capacity(4 + body.len() + 2);
        result.push(MAGIC);
        result.extend_from_slice(&length.to_be_bytes());
        result.push(0x00);
        result.extend_from_slice(&body);

        let crc = crc16_xmodem(&result);
        result.extend_from_slice(&crc);
        result
    }

    /// Parse a packet from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        ensure!(data.len() >= 8, "Packet too short: {} bytes", data.len());
        ensure!(data[0] == MAGIC, "Invalid magic byte: 0x{:02X}", data[0]);
        ensure!(data[3] == 0x00, "Invalid reserved byte: 0x{:02X}", data[3]);

        let length = u16::from_be_bytes([data[1], data[2]]) as usize;

        let command_id = [data[4], data[5]];
        let mut pkt = Self::new(command_id);

        let mut pos = 6;
        let end = length + 3; // length covers from byte[3] onward, so end = 3 + length
        while pos < end && pos + 1 < data.len() {
            let p_type = data[pos];
            let p_len = data[pos + 1] as usize;
            let p_end = pos + 2 + p_len;
            if p_end > data.len() {
                bail!(
                    "Parameter overflows packet: type={}, len={}, pos={}",
                    p_type,
                    p_len,
                    pos
                );
            }
            pkt.parameters.insert(p_type, data[pos + 2..p_end].to_vec());
            pos = p_end;
        }

        Ok(pkt)
    }

    /// Parse from bytes with CRC validation.
    pub fn from_bytes_checked(data: &[u8]) -> Result<Self> {
        ensure!(data.len() >= 8, "Packet too short for CRC check");
        let crc_data = &data[..data.len() - 2];
        let crc_value = &data[data.len() - 2..];
        let computed = crc16_xmodem(crc_data);
        ensure!(
            computed == [crc_value[0], crc_value[1]],
            "CRC mismatch: computed {:02X}{:02X}, expected {:02X}{:02X}",
            computed[0],
            computed[1],
            crc_value[0],
            crc_value[1]
        );
        Self::from_bytes(data)
    }
}

impl std::fmt::Display for HuaweiSppPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cmd={:02X}{:02X}",
            self.command_id[0], self.command_id[1]
        )?;
        for (&t, v) in &self.parameters {
            write!(f, " p{}={}", t, hex::encode(v))?;
        }
        Ok(())
    }
}

/// Helper module for hex encoding in Display (avoid extra dependency)
mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let mut pkt = HuaweiSppPacket::new([0x01, 0x08]);
        pkt.parameters.insert(1, vec![0x64]); // battery 100%
        pkt.parameters.insert(2, vec![0x50, 0x4E, 0x00]); // L:80 R:78 Case:0

        let bytes = pkt.to_bytes();
        let parsed = HuaweiSppPacket::from_bytes_checked(&bytes).unwrap();

        assert_eq!(parsed.command_id, [0x01, 0x08]);
        assert_eq!(parsed.find_param(1), &[0x64]);
        assert_eq!(parsed.find_param(2), &[0x50, 0x4E, 0x00]);
    }

    #[test]
    fn test_read_request() {
        let pkt = HuaweiSppPacket::read_request([0x01, 0x08], &[1, 2, 3]);
        assert_eq!(pkt.parameters.len(), 3);
        assert!(pkt.find_param(1).is_empty());
        assert!(pkt.find_param(2).is_empty());
        assert!(pkt.find_param(3).is_empty());
    }

    #[test]
    fn test_write_request() {
        let pkt = HuaweiSppPacket::write_request(
            [0x2B, 0x04],
            &[(1, vec![0x01, 0xFF])],
        );
        assert_eq!(pkt.command_id, [0x2B, 0x04]);
        assert_eq!(pkt.find_param(1), &[0x01, 0xFF]);
    }

    #[test]
    fn test_packet_format() {
        // Build a simple packet and verify wire format
        let pkt = HuaweiSppPacket::read_request([0x01, 0x08], &[1, 2, 3]);
        let bytes = pkt.to_bytes();

        assert_eq!(bytes[0], 0x5A); // magic
        assert_eq!(bytes[3], 0x00); // reserved
        assert_eq!(bytes[4], 0x01); // cmd hi
        assert_eq!(bytes[5], 0x08); // cmd lo

        // Verify we can parse it back
        let parsed = HuaweiSppPacket::from_bytes_checked(&bytes).unwrap();
        assert_eq!(parsed.command_id, [0x01, 0x08]);
    }

    #[test]
    fn test_empty_param_not_found() {
        let pkt = HuaweiSppPacket::new([0x01, 0x08]);
        assert!(pkt.find_param(99).is_empty());
    }

    #[test]
    fn test_crc_validation_fails_on_corrupt() {
        let pkt = HuaweiSppPacket::new([0x01, 0x08]);
        let mut bytes = pkt.to_bytes();
        // Corrupt the last byte (CRC)
        let len = bytes.len();
        bytes[len - 1] ^= 0xFF;
        assert!(HuaweiSppPacket::from_bytes_checked(&bytes).is_err());
    }
}
