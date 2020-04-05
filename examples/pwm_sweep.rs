#![no_main]
#![no_std]

extern crate panic_semihosting;

use p_hal::{pac, prelude::*};
use stm32h7xx_hal as p_hal;

use cortex_m_rt::entry;

use core::fmt::Write;

use px4io_driver::{new_serial_driver, registers, RegisterValue};

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
    let gpioe = dp.GPIOE.split(&mut ccdr.ahb4);
    let gpiof = dp.GPIOF.split(&mut ccdr.ahb4);

    //UART7 is debug (dronecode debug port) on Durandal
    let uart7_port = {
        let config =
            p_hal::serial::config::Config::default().baudrate(57_600_u32.bps());
        let rx = gpiof.pf6.into_alternate_af7();
        let tx = gpioe.pe8.into_alternate_af7();
        dp.UART7.usart((tx, rx), config, &mut ccdr).unwrap()
    };
    let (mut console_tx, _) = uart7_port.split();

    // UART8 is the serial connection to the px4io IO coprocessor
    // 1.5 Mbps is the max rate for px4io
    let uart8_port = {
        let config = p_hal::serial::config::Config::default()
            .baudrate(1_500_000_u32.bps());
        let rx = gpioe.pe0.into_alternate_af8();
        let tx = gpioe.pe1.into_alternate_af8();
        dp.UART8.usart((tx, rx), config, &mut ccdr).unwrap()
    };

    if let Some(mut driver) = new_serial_driver(uart8_port) {
        loop {
            delay_source.delay_ms(100u8);

            let mut values: [RegisterValue; 5] = [0; 5];
            let mut offset = registers::REG_CONFIG_PROTOCOL_VERSION;
            let _ = writeln!(console_tx, "---").unwrap();
            let _ = driver.get_registers(
                registers::PAGE_CONFIG,
                offset,
                &mut values,
            );
            writeln!(console_tx, "{}: {:x?}", offset, values).unwrap();

            let mut values: [RegisterValue; 4] = [0; 4];
            offset = registers::REG_CONFIG_N_RC_INPUTS;
            let _ = driver.get_registers(
                registers::PAGE_CONFIG,
                offset,
                &mut values,
            );
            writeln!(console_tx, "{} : {:x?}", offset, values).unwrap();
        }
    }

    loop {
        delay_source.delay_ms(250u8);
    }
}
