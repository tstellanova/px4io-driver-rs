use shufflebuf::ShuffleBuf;

use embedded_hal as hal;

/// Errors in this crate
#[derive(Debug)]
pub enum Error<CommE> {
    /// Communication error
    Comm(CommE),

    /// Device is not responding
    Unresponsive,
}

pub struct SerialInterface<SER> {
    /// the serial port to use when communicating
    serial: SER,
    shuffler: ShuffleBuf,
}

impl<SER, CommE> SerialInterface<SER>
    where
        SER: hal::serial::Read<u8, Error = CommE> +
            hal::serial::Write<u8, Error = CommE>
{

}

pub fn new_serial_driver<UART, CommE>(
    uart: UART,
) -> IoMcuDriver<SerialInterface<UART>>
    where
        UART: hal::serial::Read<u8, Error = CommE> +  hal::serial::Write<u8, Error = CommE>,
        CommE: core::fmt::Debug,
{
    let iface = interface::SerialInterface::new(uart);
    IoMcuDriver::new_with_interface(iface)
}

struct IoMcuDriver<DI> {
    /// the device interface
    di: DI,
}

//TODO not completely clear what the type of a valid register is...u16 or u32?
type RegisterValue = u16;

impl<DI, CommE> IoMcuDriver<DI>
    where
        DI: DeviceInterface<InterfaceError = Error<CommE>>,
        CommE: core::fmt::Debug,
{
    pub(crate) fn new_with_interface(device_interface: DI) -> Self {
        Self {
            di: device_interface,
        }
    }

    /// Set one or more virtual register values
    /// - `page` is the register page to write
    /// - `offset` is the offset ot begin writing at
    /// - `values` are the values to write
    pub fn set_registers(&mut self, page: u8, offset: u8, values: &[RegisterValue])
        -> Result<(), DI::InterfaceError> {
        unimplemented!()
    }

    pub fn set_one_register(&mut self, page: u8, offset: u8, value: RegisterValue)
        -> Result<(), DI::InterfaceError> {
        self.set_registers(page, offset, &[value])
    }

    /// Get one or more virtual register values
    /// - `page` is the register page to read
    /// - `offset` is the offset to begin reading values from
    /// - `values` is the destination to copy register values into
    pub fn get_registers(&mut self, page: u8, offset: u8, values: &mut [RegisterValue])
        -> Result<(), DI::InterfaceError> {
        unimplemented!()
    }

    pub fn get_one_register(&mut self, page: u8, offset: u8) -> Result<RegisterValue, DI::InterfaceError> {
        let mut read_buf: [RegisterValue;1] = [0; 1];
        self.get_registers(offset, offset,&mut read_buf)?;
        Ok(read_buf[0])
    }

    /// Modify a virtual register value
    /// - `page` is the register page to modify
    /// - `offset` is the register offset to modify
    /// - `clear_bits` are bits to clear in the register
    /// - `set_bits` are bits to set in the register
    pub fn modify_register(&mut self, page: u8, offset: u8, clear_bits: RegisterValue, set_bits: RegisterValue)
        -> Result<(), DI::InterfaceError> {
        let mut reg_val = self.get_one_register(page, offset)?;
        reg_val |= set_bits;
        reg_val ~= clear_bits;
        self.set_one_register(page, offset, reg_val)
    }


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
