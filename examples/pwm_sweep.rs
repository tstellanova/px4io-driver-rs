#![no_main]
#![no_std]

extern crate panic_semihosting;

use p_hal::{pac, prelude::*};
use stm32h7xx_hal as p_hal;

use cortex_m_rt::entry;
use p_hal::interrupt;

use core::fmt::Write;
use cortex_m_semihosting::hprintln;

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
        let mut port = dp.UART8.usart((tx, rx), config, &mut ccdr).unwrap();
        //	rCR1 = USART_CR1_RE | USART_CR1_TE | USART_CR1_UE | USART_CR1_IDLEIE;
        port.listen(p_hal::serial::Event::GenError);
        port.listen(p_hal::serial::Event::Idle);
        port
    };

    if let Some(mut driver) = new_serial_driver(uart8_port) {
        loop {
            delay_source.delay_ms(250u8);

            let mut values: [RegisterValue; 5] = [0; 5];
            let mut offset = registers::REG_CONFIG_PROTOCOL_VERSION;
            let _ = writeln!(console_tx, "---\r").unwrap();
            if driver
                .get_registers(registers::PAGE_CONFIG, offset, &mut values)
                .is_ok()
            {
                writeln!(console_tx, "{}: {:x?} \r", offset, values).unwrap();
            }

            delay_source.delay_ms(250u8);
            let mut values: [RegisterValue; 4] = [0; 4];
            offset = registers::REG_CONFIG_N_RC_INPUTS;
            if driver
                .get_registers(registers::PAGE_CONFIG, offset, &mut values)
                .is_ok()
            {
                writeln!(console_tx, "{} : {:x?} \r", offset, values).unwrap();
            }
        }
    }

    loop {
        delay_source.delay_ms(250u8);
    }
}

// Interrupt handler for the UART8 interrupt
#[interrupt]
fn UART8() {
    static mut COUNT: i32 = 0;

    // `COUNT` is safe to access and has type `&mut i32`
    *COUNT += 1;
    hprintln!("uart8: {}", COUNT).unwrap();

    // ..
    // Clear reason for the generated interrupt request
}

// fn setup_uart_dma<T>(port: p_hal::serial::Serial) -> p_hal::serial::Serial
// {
// // #define PX4IO_SERIAL_VECTOR            STM32_IRQ_UART8
// // #define PX4IO_SERIAL_TX_DMAMAP         DMAMAP_UART8_TX
// // #define PX4IO_SERIAL_RX_DMAMAP         DMAMAP_UART8_RX
//
//
// }