/*
Copyright (c) 2020 Todd Stellanova
LICENSE: BSD3 (see LICENSE file)
*/

#![no_std]

use embedded_hal as hal;



#[allow(unused)]
mod protocol;
#[allow(unused)]
pub mod registers;

mod interface;
use crate::protocol::{PACKET_CODE_CORRUPT, PACKET_CODE_ERROR};
use crate::Error::ErrorResponse;
use interface::{DeviceInterface, IoPacket, SerialInterface};


// use cortex_m_rt as rt;
// #[cfg(debug_assertions)]
// use cortex_m_semihosting::hprintln;

/// Errors in this crate
#[derive(Debug)]
pub enum Error<CommE> {
    /// Communication error
    Comm(CommE),

    /// Error response from px4io
    ErrorResponse,

    /// Device is not responding
    Unresponsive,
}

pub fn new_serial_driver<UART, CommE>(
    uart: UART,
) -> Option<IoMcuDriver<SerialInterface<UART>>>
where
    UART: hal::serial::Read<u8, Error = CommE>
        + hal::serial::Write<u8>,
    CommE: core::fmt::Debug,
{
    let iface = interface::SerialInterface::new(uart);
    let mut driver = IoMcuDriver::new_with_interface(iface);
    if driver.setup().is_ok() {
        return Some(driver);
    }
    None
}

pub struct IoMcuDriver<DI> {
    /// the device interface
    di: DI,
    recv_packet: IoPacket,
    send_packet: IoPacket,
}

pub type RegisterValue = u16;

impl<DI, CommE> IoMcuDriver<DI>
where
    DI: DeviceInterface<InterfaceError = Error<CommE>>,
    CommE: core::fmt::Debug,
{
    pub(crate) fn new_with_interface(device_interface: DI) -> Self {
        Self {
            di: device_interface,
            recv_packet: IoPacket::default(),
            send_packet: IoPacket::default(),
        }
    }

    pub(crate) fn setup(&mut self) -> Result<(), DI::InterfaceError> {
        self.di.setup()?;
        Ok(())
    }

    /// Set one or more virtual register values
    /// - `page` is the register page to write
    /// - `offset` is the offset ot begin writing at
    /// - `values` are the values to write
    pub fn set_registers(
        &mut self,
        page: u8,
        offset: u8,
        values: &[RegisterValue],
    ) -> Result<(), DI::InterfaceError> {
        self.send_packet.set_values(
            protocol::PACKET_CODE_WRITE,
            page,
            offset,
            values,
        );
        self.packet_exchange(3)?;
        Ok(())
    }

    /// Send write_packet to px4io, receive read_packet
    fn packet_exchange(
        &mut self,
        retries: u8,
    ) -> Result<(), DI::InterfaceError> {
        self.recv_packet.clear();
        for _ in 0..retries {
            let _recv_size = self
                .di
                .exchange_packets(&self.send_packet, &mut self.recv_packet)?;

            if self.recv_packet.is_crc_valid() {
                let opcode = self.recv_packet.packet_code();
                if opcode != PACKET_CODE_CORRUPT && opcode != PACKET_CODE_ERROR {
                    return Ok(());
                }
            }
            else {
                //hprintln!("packet_exchange invalid crc").unwrap();
            }
        }
        // hprintln!("packet_exchange failed").unwrap();
        Err(ErrorResponse)
    }

    /// Set the value of one virtual register
    pub fn set_one_register(
        &mut self,
        page: u8,
        offset: u8,
        value: RegisterValue,
    ) -> Result<(), DI::InterfaceError> {
        self.set_registers(page, offset, &[value])
    }

    /// Get one or more virtual register values
    /// - `page` is the register page to read
    /// - `offset` is the offset to begin reading values from
    /// - `values` is the destination to copy register values into
    pub fn get_registers(
        &mut self,
        page: u8,
        offset: u8,
        values: &mut [RegisterValue],
    ) -> Result<(), DI::InterfaceError> {
        self.send_packet.set_values(
            protocol::PACKET_CODE_READ,
            page,
            offset,
            &[],
        );
        self.packet_exchange(3)?;
        // if we get this far, then self.recv_packet contains read values
        values.copy_from_slice(
            self.recv_packet.registers[..values.len()].as_ref(),
        );
        Ok(())
    }

    pub fn get_one_register(
        &mut self,
        page: u8,
        offset: u8,
    ) -> Result<RegisterValue, DI::InterfaceError> {
        let mut read_buf: [RegisterValue; 1] = [0; 1];
        self.get_registers(page, offset, &mut read_buf)?;
        Ok(read_buf[0])
    }

    /// Modify a virtual register value
    /// - `page` is the register page to modify
    /// - `offset` is the register offset to modify
    /// - `clear_bits` are bits to clear in the register
    /// - `set_bits` are bits to set in the register
    pub fn modify_register(
        &mut self,
        page: u8,
        offset: u8,
        clear_bits: RegisterValue,
        set_bits: RegisterValue,
    ) -> Result<(), DI::InterfaceError> {
        let mut reg_val = self.get_one_register(page, offset)?;
        reg_val |= set_bits;
        reg_val &= !clear_bits;
        self.set_one_register(page, offset, reg_val)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
