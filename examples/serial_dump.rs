#![no_main]
#![no_std]

extern crate panic_semihosting;

use p_hal::{pac, prelude::*};
use stm32h7xx_hal as p_hal;

use cortex_m_rt::entry;

use px4io_driver::interface::PACKET_HEADER_LEN;
use px4io_driver::protocol::PACKET_CODE_READ;
use px4io_driver::registers::PAGE_CONFIG;

use core::fmt::Write;

/// This isn't strictly an example, but rather a tool for debugging the
/// serial protocol between PX4FMU and PX4IO (IOMCU)
///
/// `cargo run --example serial_dump --target thumbv7em-none-eabihf`
///
///
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
    let mut uart8_port = {
        let config = p_hal::serial::config::Config::default()
            //.hw_flow_enabled(true)
            .baudrate(1_500_000_u32.bps());
        let rx = gpioe.pe0.into_alternate_af8();
        let tx = gpioe.pe1.into_alternate_af8();
        dp.UART8.usart((tx, rx), config, &mut ccdr).unwrap()
    };

    const EXPECTED_PX4IO_PROTOCOL_VERSION: u8 = 4;
    const TEST_REG_COUNT: usize = 4;
    const QUERY_RESPONSE_LEN: usize = PACKET_HEADER_LEN + 2 * TEST_REG_COUNT;
    const QUERY_TYPE: u8 = px4io_driver::protocol::PACKET_CODE_READ; //PACKET_CODE_WRITE
    const QUERY_PAGE: u8 = PAGE_CONFIG;

    let mut query_block = [0u8; QUERY_RESPONSE_LEN];
    query_block[0] = (TEST_REG_COUNT as u8) | QUERY_TYPE;
    query_block[2] = QUERY_PAGE;
    let crc = px4io_driver::interface::IoPacket::crc8_anon(
        &query_block,
        TEST_REG_COUNT,
    );
    query_block[1] = crc;

    'outer: loop {
        // discard all input
        loop {
            let rc = uart8_port.read();
            match rc {
                Err(nb::Error::WouldBlock) => {
                    break;
                }
                _ => {}
            }
        }

        delay_source.delay_ms(250u8);
        if uart8_port.bwrite_all(&query_block).is_ok()
            && uart8_port.bflush().is_ok()
        {
            writeln!(console_tx, "--\r").unwrap();
        } else {
            continue;
        }

        let mut recv_count = 0;
        let mut blocking_count = 0;
        let mut packet_error = false;
        let mut packet_reg_count = 0;
        loop {
            let rc = uart8_port.read();
            match rc {
                Err(nb::Error::WouldBlock) => {
                    //writeln!(console_tx,"b \r").unwrap();
                    blocking_count += 1;
                    if blocking_count > 1 {
                        continue 'outer;
                    }
                }
                Ok(word) => {
                    blocking_count = 0;
                    match recv_count {
                        0 => {
                            if (word & 0x40) == 0x40 || (word & 0x80) == 0x80 {
                                writeln!(console_tx, "pkt err: 0x{:x}\r", word)
                                    .unwrap();
                                packet_error = true;
                            } else {
                                let _ =
                                    writeln!(console_tx, "cc: 0x{:x} \r", word);
                                packet_reg_count = word;
                                if packet_reg_count != (TEST_REG_COUNT as u8)
                                    && QUERY_TYPE == PACKET_CODE_READ
                                {
                                    continue 'outer;
                                }
                            }
                        }
                        1 => {
                            writeln!(console_tx, "crc: 0x{:x}\r", word)
                                .unwrap();
                        }
                        2 => {
                            writeln!(console_tx, "p: {} \r", word).unwrap();
                        }
                        3 => {
                            writeln!(console_tx, "o: {} \r", word).unwrap();
                            if packet_error || (0 == packet_reg_count) {
                                //no more packet data will come after this
                                continue 'outer;
                            }
                        }
                        4 => {
                            writeln!(
                                console_tx,
                                "[{}] {:x} \r",
                                recv_count - PACKET_HEADER_LEN,
                                word
                            )
                            .unwrap();
                            if QUERY_PAGE == PAGE_CONFIG
                                && word != EXPECTED_PX4IO_PROTOCOL_VERSION
                            {
                                writeln!(
                                    console_tx,
                                    "proto version: {} expected: {} \r",
                                    word, EXPECTED_PX4IO_PROTOCOL_VERSION
                                )
                                .unwrap();
                                continue 'outer;
                            }
                        }
                        _ => {
                            writeln!(
                                console_tx,
                                "[{}] {:x} \r",
                                recv_count - PACKET_HEADER_LEN,
                                word
                            )
                            .unwrap();
                        }
                    }
                    recv_count += 1;
                    if recv_count == QUERY_RESPONSE_LEN {
                        continue 'outer;
                    }
                }
                Err(any) => {
                    writeln!(console_tx, "{:?} \r", any).unwrap();
                    continue 'outer;
                    //writeln!(console_tx,".").unwrap();
                }
            }
        }
    }
}
