use super::DeviceInterface;
use crate::{Error, IoPacket};
use embedded_hal as hal;

use crate::interface::{any_as_mut_u8_slice, any_as_u8_slice};

/// This encapsulates the Serial UART peripheral
pub struct SerialInterface<SER> {
    /// the serial port to use when communicating
    serial: SER,
}

impl<SER, CommE> SerialInterface<SER>
where
    SER: hal::serial::Read<u8, Error = CommE>
        + hal::blocking::serial::Write<u8, Error = CommE>,
    CommE: core::fmt::Debug,
{
    pub fn new(serial_port: SER) -> Self {
        Self {
            serial: serial_port,
        }
    }

    /// Read up to buffer size bytes
    fn read_many(&mut self, buffer: &mut [u8]) -> Result<usize, Error<CommE>> {
        let mut read_count: usize = 0;
        for word in buffer {
            if let Ok(read_byte) = self.serial.read() {
                read_count += 1;
                *word = read_byte;
            } else {
                break;
            }
        }

        Ok(read_count)
    }
}

impl<SER, CommE> DeviceInterface for SerialInterface<SER>
where
    SER: hal::serial::Read<u8, Error = CommE>
        + hal::blocking::serial::Write<u8, Error = CommE>,
    CommE: core::fmt::Debug,
{
    type InterfaceError = Error<CommE>;

    fn setup(&mut self) -> Result<(), Self::InterfaceError> {
        //TODO need to do any UART configuration? Ensure HW flow control?
        Ok(())
    }

    fn exchange_packets(
        &mut self,
        send: &IoPacket,
        recv: &mut IoPacket,
    ) -> Result<usize, Self::InterfaceError> {
        // send a packet first, then receive one
        let write_slice = unsafe { any_as_u8_slice(send) };
        self.serial.bwrite_all(write_slice).map_err(Error::Comm)?;

        let read_slice = unsafe { any_as_mut_u8_slice(recv) };
        let read_count = self.read_many(read_slice)?;

        Ok(read_count)
    }
}
