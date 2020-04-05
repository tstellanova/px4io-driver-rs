use super::DeviceInterface;
use crate::{Error, IoPacket};
use embedded_hal as hal;

use nb::block;

use crate::interface::{any_as_mut_u8_slice, any_as_u8_slice, IO_PACKET_HEADER_LEN};

#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;

/// This encapsulates the Serial UART peripheral
pub struct SerialInterface<SER> {
    /// the serial port to use when communicating
    serial: SER,
}

impl<SER, CommE> SerialInterface<SER>
where
    SER: hal::serial::Read<u8, Error = CommE>
        + hal::serial::Write<u8>,
    CommE: core::fmt::Debug,
{
    pub fn new(serial_port: SER) -> Self {
        Self {
            serial: serial_port,
        }
    }

    /// Read up to buffer size bytes
    fn read_many(&mut self, buffer: &mut [u8]) -> Result<usize, Error<CommE>> {
        let mut fail_count = 0;
        let mut read_count: usize = 0;
        let mut block_count = 0;
        while read_count < buffer.len() {
            let read_result =  self.serial.read();
            match read_result {
                Ok(read_byte) => {
                    fail_count = 0;
                    block_count = 0;
                    buffer[read_count] = read_byte;
                    read_count += 1;
                }
                Err(nb::Error::WouldBlock) => {
                    block_count += 1;
                    if block_count > 40 {
                        hprintln!("blocked!").unwrap();
                        break;
                    }
                }
                _ => {
                    fail_count += 1;
                    if fail_count > 20 {
                        hprintln!("read err {:?}", read_result).unwrap();
                        break;
                    }
                }
            }
        }
        if read_count != buffer.len() {
            let _ = hprintln!("nread {} != {} {:x?}", read_count, buffer.len(), buffer[..read_count].as_ref());
        }
        Ok(read_count)
    }

    /// Write up to buffer size bytes
    fn write_many(&mut self, buffer: &[u8]) -> Result<(), Error<CommE>> {
        for word in buffer {
            let rc = block!(self.serial.write(*word));
            if rc.is_err() {
                hprintln!("write err").unwrap();
            }
        }

        Ok(())
    }
}

impl<SER, CommE> DeviceInterface for SerialInterface<SER>
where
    SER: hal::serial::Read<u8, Error = CommE>
        + hal::serial::Write<u8>,
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

        let out_reg_count = send.valid_register_count();
        let packet_len = IO_PACKET_HEADER_LEN + (out_reg_count * 2) as usize;
        hprintln!("ex len: {}", packet_len).unwrap();

        // send a packet first, then receive one
        let write_slice = unsafe { any_as_u8_slice(send) };
        let trc = self.write_many(&write_slice[..packet_len]);
        if trc.is_err() {
            hprintln!("trc: {:?}", trc).unwrap();
        }

        // protocol version 4 says send packet is same size as receive packet
        let read_slice = unsafe { any_as_mut_u8_slice(recv) };
        let read_count = self.read_many(&mut read_slice[..packet_len])?;

        Ok(read_count)
    }

    /// Clear any remaining bytes in the pipe
    fn discard_input(&mut self) {
        let mut discard_count = 0;
        let mut fail_count = 0;
        loop {
            let read_result = self.serial.read();
            match read_result {
                Ok(_) => {
                    fail_count = 0;
                    discard_count += 1;
                }
                Err(nb::Error::WouldBlock) => {
                    break;
                }
                _ => {
                    //hprintln!("read err {:?}", read_result).unwrap();
                    fail_count += 1;
                    if fail_count > 10 {
                        break;
                    }
                }
            }
        }

        if discard_count > 0 {
            hprintln!("discard {}", discard_count).unwrap();
        }
    }
}
