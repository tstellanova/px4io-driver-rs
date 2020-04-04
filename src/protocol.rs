/// Packet exchange codes

/// Read from the px4io
pub const PACKET_CODE_READ: u8 = 0x00;
/// Write to the px4io
pub const PACKET_CODE_WRITE: u8 = 0x40;
/// Success reply from px4io
pub const PACKET_CODE_SUCCESS: u8 = 0x00;
/// Corrupt packet reply from px4io
pub const PACKET_CODE_CORRUPT: u8 = 0x40;
/// Error reply from px4io
pub const PACKET_CODE_ERROR: u8 = 0x80;
