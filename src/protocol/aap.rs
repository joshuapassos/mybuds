/// Apple Audio Control Protocol (AACP) packet definitions.
///
/// Used by AirPods for communication over L2CAP PSM 0x1001.
/// Reference: LibrePods project (github.com/kavishdevar/librepods)

use crate::protocol::HuaweiSppPacket;

// --- Wire format ---

/// Standard AACP packet header.
pub const AAP_HEADER: [u8; 4] = [0x04, 0x00, 0x04, 0x00];

// --- Opcodes ---

pub const OP_BATTERY_INFO: u8 = 0x04;
pub const OP_EAR_DETECTION: u8 = 0x06;
pub const OP_CONTROL_COMMAND: u8 = 0x09;
pub const OP_REQUEST_NOTIFICATIONS: u8 = 0x0F;
pub const OP_DEVICE_INFO: u8 = 0x1D;
pub const OP_CONVERSATION_AWARENESS: u8 = 0x4B;
pub const OP_SET_FEATURE_FLAGS: u8 = 0x4D;

// --- Control Command identifiers (subtypes of opcode 0x09) ---

pub const CC_LISTENING_MODE: u8 = 0x0D;
pub const CC_LISTENING_MODE_CONFIGS: u8 = 0x1A;
pub const CC_ONE_BUD_ANC: u8 = 0x1B;
pub const CC_ADAPTIVE_VOLUME: u8 = 0x26;
pub const CC_CONVERSATION_DETECT: u8 = 0x28;
pub const CC_AUTO_ANC_STRENGTH: u8 = 0x2E;
pub const CC_EAR_DETECTION_CONFIG: u8 = 0x0A;

// --- Listening mode values ---

pub const LM_OFF: u8 = 0x01;
pub const LM_ANC: u8 = 0x02;
pub const LM_TRANSPARENCY: u8 = 0x03;
pub const LM_ADAPTIVE: u8 = 0x04;

// --- Battery component types ---

pub const BATT_RIGHT: u8 = 0x02;
pub const BATT_LEFT: u8 = 0x04;
pub const BATT_CASE: u8 = 0x08;

// --- Battery status ---

pub const BATT_CHARGING: u8 = 0x01;
pub const BATT_DISCONNECTED: u8 = 0x04;

// --- Ear detection values ---

pub const EAR_IN: u8 = 0x00;
pub const EAR_OUT: u8 = 0x01;
pub const EAR_IN_CASE: u8 = 0x02;

// --- Command ID prefixes for mapping AAP → HuaweiSppPacket ---
// These fake command IDs let AirPods handlers coexist with Huawei handlers.

/// Prefix for general AAP opcodes: command_id = [0xAA, opcode]
pub const CMD_PREFIX: u8 = 0xAA;

/// Prefix for control command subtypes: command_id = [0xA9, identifier]
pub const CMD_CC_PREFIX: u8 = 0xA9;

// --- Packet type ---

/// An AACP packet.
#[derive(Debug, Clone)]
pub struct AapPacket {
    pub opcode: u8,
    pub payload: Vec<u8>,
}

impl AapPacket {
    pub fn new(opcode: u8, payload: Vec<u8>) -> Self {
        Self { opcode, payload }
    }

    /// Serialize to wire bytes: [header][opcode][0x00][payload]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(6 + self.payload.len());
        bytes.extend_from_slice(&AAP_HEADER);
        bytes.push(self.opcode);
        bytes.push(0x00);
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Parse from raw L2CAP data (minimum 6 bytes).
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 5 {
            return None;
        }
        let opcode = data[4];
        // Payload starts after [header(4)][opcode(1)][0x00(1)]
        let payload = if data.len() > 6 {
            data[6..].to_vec()
        } else {
            Vec::new()
        };
        Some(Self { opcode, payload })
    }

    // --- Protocol init packets ---

    /// Handshake packet (different header: 00 00 04 00).
    pub fn handshake() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
    }

    /// Feature flags packet (enables conversational awareness etc.).
    pub fn feature_flags() -> Vec<u8> {
        let mut pkt = Vec::with_capacity(14);
        pkt.extend_from_slice(&AAP_HEADER);
        pkt.push(OP_SET_FEATURE_FLAGS);
        pkt.push(0x00);
        pkt.extend_from_slice(&[0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        pkt
    }

    /// Request all notifications from the device.
    pub fn request_notifications() -> Vec<u8> {
        let mut pkt = Vec::with_capacity(10);
        pkt.extend_from_slice(&AAP_HEADER);
        pkt.push(OP_REQUEST_NOTIFICATIONS);
        pkt.push(0x00);
        pkt.extend_from_slice(&[0xFF, 0xFF, 0xFE, 0xFF]);
        pkt
    }

    // --- Builder helpers ---

    /// Build a control command packet.
    #[cfg(test)]
    pub fn control_command(identifier: u8, value: u8) -> Self {
        Self {
            opcode: OP_CONTROL_COMMAND,
            payload: vec![identifier, value, 0x00, 0x00, 0x00],
        }
    }

    // --- Conversion to/from HuaweiSppPacket for handler compatibility ---

    /// Convert this AAP packet to a HuaweiSppPacket that handlers can process.
    ///
    /// - General opcodes → command_id = [0xAA, opcode], param 0 = payload
    /// - Control commands → command_id = [0xA9, identifier], param 0 = remaining data
    pub fn to_handler_packet(&self) -> HuaweiSppPacket {
        if self.opcode == OP_CONTROL_COMMAND && !self.payload.is_empty() {
            let identifier = self.payload[0];
            let remaining = if self.payload.len() > 1 {
                self.payload[1..].to_vec()
            } else {
                Vec::new()
            };
            let mut pkt = HuaweiSppPacket::new([CMD_CC_PREFIX, identifier]);
            pkt.parameters.insert(0, remaining);
            pkt
        } else {
            let mut pkt = HuaweiSppPacket::new([CMD_PREFIX, self.opcode]);
            pkt.parameters.insert(0, self.payload.clone());
            pkt
        }
    }

    /// Convert a HuaweiSppPacket (from a handler) back to raw AAP bytes for transmission.
    /// Returns None if the packet isn't an AAP packet.
    pub fn from_handler_packet(pkt: &HuaweiSppPacket) -> Option<Vec<u8>> {
        let [prefix, id] = pkt.command_id;
        let data = pkt.find_param(0);

        if prefix == CMD_CC_PREFIX {
            // Control command: opcode = 0x09, identifier = id
            let mut payload = vec![id];
            payload.extend_from_slice(data);
            let aap = AapPacket::new(OP_CONTROL_COMMAND, payload);
            Some(aap.to_bytes())
        } else if prefix == CMD_PREFIX {
            let aap = AapPacket::new(id, data.to_vec());
            Some(aap.to_bytes())
        } else {
            None
        }
    }
}

impl std::fmt::Display for AapPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AAP op=0x{:02X}", self.opcode)?;
        if !self.payload.is_empty() {
            let hex: String = self.payload.iter().map(|b| format!("{:02x}", b)).collect();
            write!(f, " payload={}", hex)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_command_roundtrip() {
        let aap = AapPacket::control_command(CC_LISTENING_MODE, LM_ANC);
        let handler_pkt = aap.to_handler_packet();

        assert_eq!(handler_pkt.command_id, [CMD_CC_PREFIX, CC_LISTENING_MODE]);
        let data = handler_pkt.find_param(0);
        assert_eq!(data[0], LM_ANC);

        // Convert back to wire bytes
        let bytes = AapPacket::from_handler_packet(&handler_pkt).unwrap();
        let parsed = AapPacket::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.opcode, OP_CONTROL_COMMAND);
        assert_eq!(parsed.payload[0], CC_LISTENING_MODE);
        assert_eq!(parsed.payload[1], LM_ANC);
    }

    #[test]
    fn test_general_opcode_roundtrip() {
        let aap = AapPacket::new(OP_BATTERY_INFO, vec![0x03, 0x02, 0x01, 0x64]);
        let handler_pkt = aap.to_handler_packet();

        assert_eq!(handler_pkt.command_id, [CMD_PREFIX, OP_BATTERY_INFO]);
        assert_eq!(handler_pkt.find_param(0), &[0x03, 0x02, 0x01, 0x64]);

        let bytes = AapPacket::from_handler_packet(&handler_pkt).unwrap();
        let parsed = AapPacket::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.opcode, OP_BATTERY_INFO);
    }

    #[test]
    fn test_init_packets() {
        let hs = AapPacket::handshake();
        assert_eq!(hs[0], 0x00); // Different header
        assert_eq!(hs.len(), 16);

        let ff = AapPacket::feature_flags();
        assert_eq!(ff[4], OP_SET_FEATURE_FLAGS);

        let rn = AapPacket::request_notifications();
        assert_eq!(rn[4], OP_REQUEST_NOTIFICATIONS);
    }
}
