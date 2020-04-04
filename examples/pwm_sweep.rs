#![no_main]
#![no_std]

extern crate panic_semihosting;

use p_hal::{pac, prelude::*};
use stm32h7xx_hal as p_hal;

use cortex_m_rt::entry;

use p_hal::serial::config::{Parity, StopBits, WordLength};
use px4io_driver::{new_serial_driver, registers::*};


#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let vos = pwr.freeze();

    // Constrain and Freeze clock
    let rcc = dp.RCC.constrain();
    let mut ccdr = rcc.sys_ck(100.mhz()).freeze(vos, &dp.SYSCFG);

    let clocks = ccdr.clocks;
    let mut delay_source = p_hal::delay::Delay::new(cp.SYST, clocks);

    // Grab the only GPIO we need for this example
    let gpiob = dp.GPIOB.split(&mut ccdr.ahb4);

    // UART8 is the serial connection to the px4io IO coprocessor
    let uart8_port = {
        let config = p_hal::serial::config::Config::default()
            .baudrate(1_500_000_u32.bps());
        let rx = gpioe.pe0.into_alternate_af8();
        let tx = gpioe.pe1.into_alternate_af8();
        dp.UART8.usart((tx, rx), config, &mut ccdr).unwrap()
    };

    //TODO

    if let Some(mut driver) = new_serial_driver(uart8_port) {
        // 0 == REG_CONFIG_N_RELAY_OUTPUTS+1
        let mut fun_regs: [RegisterValue; 9] = [0; 9];
        driver.get_registers(protocol::PAGE_CONFIG, protocol::REG_CONFIG_PROTOCOL_VERSION, &mut fun_regs);
    }

}
