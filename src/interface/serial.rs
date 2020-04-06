use super::DeviceInterface;
use crate::{Error, IoPacket};
use embedded_hal as hal;

use nb::block;

use crate::interface::{
    any_as_mut_u8_slice, any_as_u8_slice, PACKET_CODE_MASK, PACKET_HEADER_LEN,
    PACKET_REG_COUNT_MASK,
};

use crate::Error::{Stalled, Unresponsive};
#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;

/// This encapsulates the Serial UART peripheral
pub struct SerialInterface<SER> {
    /// the serial port to use when communicating
    serial: SER,

    /// error counts
    restart_count_blocking: u32,
    restart_count_comm_err: u32,
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
            restart_count_blocking: 0,
            restart_count_comm_err: 0
        }
    }

    /// Read up to buffer size bytes
    fn read_many(&mut self, buffer: &mut [u8]) -> Result<usize, Error<CommE>> {
        let mut read_count: usize = 0;
        let mut block_count = 0;
        while read_count < buffer.len() {
            let read_result = self.serial.read();
            match read_result {
                Ok(read_byte) => {
                    block_count = 0;
                    buffer[read_count] = read_byte;
                    read_count += 1;
                }
                Err(nb::Error::WouldBlock) => {
                    block_count += 1;
                    if block_count > 1 {
                        self.restart_count_blocking += 1;
                        return Err(Stalled);
                    }
                }
                Err(nb::Error::Other(e)) => {
                    return Err(Error::Comm(e))
                }
            }
        }

        Ok(read_count)
    }

    /// Write up to buffer size bytes
    fn write_many(&mut self, buffer: &[u8]) -> Result<(), Error<CommE>> {
        for &byte in buffer {
            //TODO fix error handling on write
            let _ = block!(self.serial.write(byte));
        }
        let _ = block!(self.serial.flush());
        Ok(())

    }
}

impl<SER, CommE> DeviceInterface for SerialInterface<SER>
where
    SER: hal::serial::Read<u8, Error = CommE> + hal::serial::Write<u8>,
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
        num_retries: u8,
    ) -> Result<usize, Self::InterfaceError> {
        let out_reg_count = send.valid_register_count();
        let packet_len = PACKET_HEADER_LEN + (out_reg_count * 2) as usize;
        //hprintln!("ex len: {}", packet_len).unwrap();
        let write_slice = unsafe { any_as_u8_slice(send) };
        let read_slice = unsafe { any_as_mut_u8_slice(recv) };
        let mut header_buf = [0u8; PACKET_HEADER_LEN];

        for _ in 0..num_retries {
            self.discard_input();
            // send a packet first, then receive one
            let trc = self.write_many(&write_slice[..packet_len]);
            if trc.is_err() {
                hprintln!("trc: {:?}", trc).unwrap();
                return Err(trc.unwrap_err());
            }

            //read header first
            let nread_rc = self.read_many(&mut header_buf);
            if let Ok(header_count) = nread_rc {
                hprintln!("h {:x?}", header_buf).unwrap();
                if header_count != PACKET_HEADER_LEN {
                    continue;
                }

                let packet_err = header_buf[0] & PACKET_CODE_MASK;
                if 0 != packet_err {
                    continue;
                }

                let mut read_count = header_count;
                let reg_count = header_buf[0] & PACKET_REG_COUNT_MASK;
                if reg_count > 0 {
                    let total_len =
                        PACKET_HEADER_LEN + (2 * reg_count) as usize;
                    if let Ok(body_count) = self.read_many(
                        &mut read_slice[PACKET_HEADER_LEN..total_len],
                    ) {
                        read_count += body_count;
                        read_slice[0..PACKET_HEADER_LEN]
                            .copy_from_slice(header_buf.as_ref());
                        return Ok(read_count);
                    }
                } else {
                    return Ok(read_count);
                }
            }
        }

        hprintln!("<<< {} {}", self.restart_count_comm_err, self.restart_count_blocking).unwrap();
        //hprintln!("ex_fl").unwrap();
        Err(Unresponsive)
    }

    /// Clear any remaining bytes in the pipe
    fn discard_input(&mut self) {
        loop {
            let rc = self.serial.read();
            match rc {
                Err(nb::Error::WouldBlock) => {
                    break;
                }
                _ => {}
            }
        }
    }
}
