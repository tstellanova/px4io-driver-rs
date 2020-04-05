pub mod serial;
pub use serial::SerialInterface;

use crate::RegisterValue;

/// A method of communicating with the device
pub trait DeviceInterface {
    /// Interface associated error type
    type InterfaceError;

    /// Gives the interface a chance to initialize
    fn setup(&mut self) -> Result<(), Self::InterfaceError>;

    /// Write one packet and receive one packet
    fn exchange_packets(
        &mut self,
        send: &IoPacket,
        recv: &mut IoPacket,
    ) -> Result<usize, Self::InterfaceError>;

    /// Clear any remaining bytes in the pipe
    fn discard_input(&mut self);
}

/// Bytes for each packet header (excluding registers)
pub const PACKET_HEADER_LEN: usize = 4;
/// Maximum number of register values a packet can contain
pub const MAX_PACKET_REGISTERS: usize = 32;
/// Maximum size of a packet (bytes)
pub const PACKET_MAX_LEN: usize = PACKET_HEADER_LEN + MAX_PACKET_REGISTERS * 2;

pub(crate) unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        core::mem::size_of::<T>(),
    )
}

pub(crate) unsafe fn any_as_mut_u8_slice<T: Sized>(p: &mut T) -> &mut [u8] {
    core::slice::from_raw_parts_mut(
        (p as *mut T) as *mut u8,
        core::mem::size_of::<T>(),
    )
}

/// The packet format supported by the px4io mcu firmware
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct IoPacket {
    /// A mix of an operation code (such as "read") and a count of the number
    /// of valid registers in `registers`. In practice this limits the
    /// number of valid
    count_code: u8,
    crc: u8,
    pub page: u8,
    pub offset: u8,
    /// Fixed-length register value buffer
    /// px4io always sends (and expects) the same fixed length of registers
    /// even though the count_code may indicate fewer valid registers
    pub registers: [RegisterValue; MAX_PACKET_REGISTERS],
}

/// The length of the packet excluding register values
const IO_PACKET_HEADER_LEN: usize = 4;
// const IO_PACKET_MAX_LEN: usize = IO_PACKET_HEADER_LEN + MAX_PACKET_REGISTERS*2;

impl IoPacket {
    pub fn default() -> Self {
        Self {
            count_code: 0,
            crc: 0,
            page: 0,
            offset: 0,
            registers: [0; MAX_PACKET_REGISTERS],
        }
    }

    /// - mode is one of the `PACKET_CODE_xx` constants
    pub fn set_values(
        &mut self,
        mode: u8,
        page: u8,
        offset: u8,
        values: &[RegisterValue],
    ) -> &mut IoPacket {
        let count = values.len() as usize;
        self.count_code = (count as u8) | mode;
        self.crc = 0;
        self.page = page;
        self.offset = offset;
        if count > 0 {
            self.registers[..count].copy_from_slice(values);
        }

        self.crc = self.calc_crc();
        self
    }

    pub fn clear(&mut self) -> &mut IoPacket {
        self.count_code = 0;
        self.crc = 0;
        self.page = 0;
        self.offset = 0;
        for reg in self.registers.as_mut() {
            //TODO Do we need to use this magic value?
            *reg = 0x55aa;
        }
        self
    }

    /// Check whether the received CRC is valid
    pub fn is_crc_valid(&self) -> bool {
        let calc_crc = self.calc_crc();
        calc_crc == self.crc
    }

    /// How many register values have been filled in this packet?
    /// (Extracts the valid register count from our count_code)
    pub fn valid_register_count(&self) -> u8 {
        self.count_code & Self::PACKET_REG_COUNT_MASK
    }

    /// Extract the packet code from our count_code
    pub fn packet_code(&self) -> u8 {
        self.count_code & Self::PACKET_CODE_MASK
    }

    pub fn count_code(&self) -> u8 {
        self.count_code
    }

    /// Calculate the CRC for this packet
    /// This includes the header (skipping self.crc)
    /// as well as all the valid values.
    fn calc_crc(&self) -> u8 {
        let full_slice = unsafe { any_as_u8_slice(self) };
        //terminate crc calculation at the last register value from valid_register_count
        let reg_vals_len = (self.valid_register_count() * 2) as usize;

        Self::crc8_anon(full_slice, reg_vals_len)
    }

    pub fn crc8_anon(buf: &[u8], reg_vals_len: usize) -> u8 {
        let total_len = IO_PACKET_HEADER_LEN + reg_vals_len * 2;
        let mut crc = Self::CRC8_TABLE[buf[0] as usize];
        //skip buf[1], which is self.crc
        crc = Self::CRC8_TABLE[crc as usize];
        for i in 2..total_len {
            crc = Self::CRC8_TABLE[(crc ^ buf[i]) as usize];
        }

        crc
    }

    const CRC8_TABLE: [u8; 256] = [
        0x00, 0x07, 0x0E, 0x09, 0x1C, 0x1B, 0x12, 0x15, 0x38, 0x3F, 0x36, 0x31,
        0x24, 0x23, 0x2A, 0x2D, 0x70, 0x77, 0x7E, 0x79, 0x6C, 0x6B, 0x62, 0x65,
        0x48, 0x4F, 0x46, 0x41, 0x54, 0x53, 0x5A, 0x5D, 0xE0, 0xE7, 0xEE, 0xE9,
        0xFC, 0xFB, 0xF2, 0xF5, 0xD8, 0xDF, 0xD6, 0xD1, 0xC4, 0xC3, 0xCA, 0xCD,
        0x90, 0x97, 0x9E, 0x99, 0x8C, 0x8B, 0x82, 0x85, 0xA8, 0xAF, 0xA6, 0xA1,
        0xB4, 0xB3, 0xBA, 0xBD, 0xC7, 0xC0, 0xC9, 0xCE, 0xDB, 0xDC, 0xD5, 0xD2,
        0xFF, 0xF8, 0xF1, 0xF6, 0xE3, 0xE4, 0xED, 0xEA, 0xB7, 0xB0, 0xB9, 0xBE,
        0xAB, 0xAC, 0xA5, 0xA2, 0x8F, 0x88, 0x81, 0x86, 0x93, 0x94, 0x9D, 0x9A,
        0x27, 0x20, 0x29, 0x2E, 0x3B, 0x3C, 0x35, 0x32, 0x1F, 0x18, 0x11, 0x16,
        0x03, 0x04, 0x0D, 0x0A, 0x57, 0x50, 0x59, 0x5E, 0x4B, 0x4C, 0x45, 0x42,
        0x6F, 0x68, 0x61, 0x66, 0x73, 0x74, 0x7D, 0x7A, 0x89, 0x8E, 0x87, 0x80,
        0x95, 0x92, 0x9B, 0x9C, 0xB1, 0xB6, 0xBF, 0xB8, 0xAD, 0xAA, 0xA3, 0xA4,
        0xF9, 0xFE, 0xF7, 0xF0, 0xE5, 0xE2, 0xEB, 0xEC, 0xC1, 0xC6, 0xCF, 0xC8,
        0xDD, 0xDA, 0xD3, 0xD4, 0x69, 0x6E, 0x67, 0x60, 0x75, 0x72, 0x7B, 0x7C,
        0x51, 0x56, 0x5F, 0x58, 0x4D, 0x4A, 0x43, 0x44, 0x19, 0x1E, 0x17, 0x10,
        0x05, 0x02, 0x0B, 0x0C, 0x21, 0x26, 0x2F, 0x28, 0x3D, 0x3A, 0x33, 0x34,
        0x4E, 0x49, 0x40, 0x47, 0x52, 0x55, 0x5C, 0x5B, 0x76, 0x71, 0x78, 0x7F,
        0x6A, 0x6D, 0x64, 0x63, 0x3E, 0x39, 0x30, 0x37, 0x22, 0x25, 0x2C, 0x2B,
        0x06, 0x01, 0x08, 0x0F, 0x1A, 0x1D, 0x14, 0x13, 0xAE, 0xA9, 0xA0, 0xA7,
        0xB2, 0xB5, 0xBC, 0xBB, 0x96, 0x91, 0x98, 0x9F, 0x8A, 0x8D, 0x84, 0x83,
        0xDE, 0xD9, 0xD0, 0xD7, 0xC2, 0xC5, 0xCC, 0xCB, 0xE6, 0xE1, 0xE8, 0xEF,
        0xFA, 0xFD, 0xF4, 0xF3,
    ];

    /// Used to extract the packet code from a count_code
    const PACKET_CODE_MASK: u8 = 0xc0;
    /// Used to extract the number of register values from count_code
    const PACKET_REG_COUNT_MASK: u8 = 0x3f;
}
