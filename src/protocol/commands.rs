/// Command IDs for the Huawei SPP protocol.
/// Format: [service_id, command_id] as 2-byte big-endian.

// Device info
pub const CMD_DEVICE_INFO: [u8; 2] = [0x01, 0x07];

// Battery
pub const CMD_BATTERY_READ: [u8; 2] = [0x01, 0x08];
pub const CMD_BATTERY_NOTIFY: [u8; 2] = [0x01, 0x27];

// ANC
pub const CMD_ANC_READ: [u8; 2] = [0x2B, 0x2A];
pub const CMD_ANC_WRITE: [u8; 2] = [0x2B, 0x04];
pub const CMD_ANC_LEGACY_NOTIFY: [u8; 2] = [0x2B, 0x03];

// Auto-pause
pub const CMD_AUTO_PAUSE_READ: [u8; 2] = [0x2B, 0x11];
pub const CMD_AUTO_PAUSE_WRITE: [u8; 2] = [0x2B, 0x10];

// Gestures - Double tap
pub const CMD_DUAL_TAP_READ: [u8; 2] = [0x01, 0x20];
pub const CMD_DUAL_TAP_WRITE: [u8; 2] = [0x01, 0x1F];

// Gestures - Triple tap
pub const CMD_TRIPLE_TAP_READ: [u8; 2] = [0x01, 0x26];
pub const CMD_TRIPLE_TAP_WRITE: [u8; 2] = [0x01, 0x25];

// Gestures - Long tap (split left/right)
pub const CMD_LONG_TAP_SPLIT_READ_BASE: [u8; 2] = [0x2B, 0x17];
pub const CMD_LONG_TAP_SPLIT_READ_ANC: [u8; 2] = [0x2B, 0x19];
pub const CMD_LONG_TAP_SPLIT_WRITE_BASE: [u8; 2] = [0x2B, 0x16];
pub const CMD_LONG_TAP_SPLIT_WRITE_ANC: [u8; 2] = [0x2B, 0x18];

// Gestures - Swipe
pub const CMD_SWIPE_READ: [u8; 2] = [0x2B, 0x1F];
pub const CMD_SWIPE_WRITE: [u8; 2] = [0x2B, 0x1E];

// Low latency
pub const CMD_LOW_LATENCY: [u8; 2] = [0x2B, 0x6C];

// Dual connect
pub const CMD_DUAL_CONNECT_ENABLED_READ: [u8; 2] = [0x2B, 0x2F];
pub const CMD_DUAL_CONNECT_ENABLED_WRITE: [u8; 2] = [0x2B, 0x2E];
pub const CMD_DUAL_CONNECT_ENUMERATE: [u8; 2] = [0x2B, 0x31];
pub const CMD_DUAL_CONNECT_PREFERRED_WRITE: [u8; 2] = [0x2B, 0x32];
pub const CMD_DUAL_CONNECT_EXECUTE: [u8; 2] = [0x2B, 0x33];
pub const CMD_DUAL_CONNECT_CHANGE_EVENT: [u8; 2] = [0x2B, 0x36];

// Equalizer
pub const CMD_EQUALIZER_READ: [u8; 2] = [0x2B, 0x4A];
pub const CMD_EQUALIZER_WRITE: [u8; 2] = [0x2B, 0x49];

// Sound quality preference
pub const CMD_SOUND_QUALITY_READ: [u8; 2] = [0x2B, 0xA3];
pub const CMD_SOUND_QUALITY_WRITE: [u8; 2] = [0x2B, 0xA2];

// Voice language
pub const CMD_VOICE_LANGUAGE_READ: [u8; 2] = [0x0C, 0x02];
pub const CMD_VOICE_LANGUAGE_WRITE: [u8; 2] = [0x0C, 0x01];

// In-ear state
pub const CMD_IN_EAR_STATE: [u8; 2] = [0x01, 0x0B];

/// Helper type for command IDs
pub type CommandId = [u8; 2];
