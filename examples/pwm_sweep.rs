#![no_main]
#![no_std]

extern crate panic_semihosting;

use p_hal::{pac, prelude::*};
use stm32h7xx_hal as p_hal;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use px4io_driver::{new_serial_driver, registers, RegisterValue, protocol};
use nb::block;
use embedded_hal::blocking::serial::Write;
use px4io_driver::interface::{MAX_PACKET_REGISTERS, PACKET_HEADER_LEN, PACKET_MAX_LEN};
use px4io_driver::registers::{PAGE_TEST_DEBUG, PAGE_CONFIG};
use px4io_driver::protocol::PACKET_CODE_READ;

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



    // const EXPECTED_PX4IO_PROTOCOL_VERSION: u8 = 4;
    //
    // const TEST_REG_COUNT:usize = 6;
    // const QUERY_RESPONSE_LEN:usize = PACKET_HEADER_LEN + 2*TEST_REG_COUNT;
    // const QUERY_TYPE: u8 = px4io_driver::protocol::PACKET_CODE_READ; //PACKET_CODE_WRITE
    // const QUERY_PAGE: u8 = PAGE_CONFIG;
    //
    // let mut query_block = [0u8; QUERY_RESPONSE_LEN];
    // query_block[0] = (TEST_REG_COUNT as u8) | QUERY_TYPE;
    // query_block[2] = QUERY_PAGE;
    // let crc =  px4io_driver::interface::IoPacket::crc8_anon(&query_block, TEST_REG_COUNT);
    // query_block[1] = crc;

    // 'outer: loop {
    //     // discard all input
    //     loop {
    //         let rc = uart8_port.read();
    //         match rc {
    //             Err(nb::Error::WouldBlock) => {
    //                 //hprintln!("b").unwrap();
    //                 break;
    //             }
    //             _ => {
    //                 //hprintln!("?").unwrap();
    //             }
    //         }
    //     }
    //
    //     if let Ok(tx_result) =  uart8_port.bwrite_all(&query_block) {
    //         hprintln!("--").unwrap();
    //         // if let Ok(flush_result)  = uart8_port.bflush() {
    //         //     hprintln!("--").unwrap();
    //         // }
    //     }
    //
    //     let mut recv_count = 0;
    //     let mut blocking_count = 0;
    //     let mut packet_error = false;
    //     let mut packet_reg_count = 0;
    //     loop {
    //         let rc = uart8_port.read();
    //         match rc {
    //             Err(nb::Error::WouldBlock) => {
    //                 //hprintln!("b").unwrap();
    //                 blocking_count += 1;
    //                 if blocking_count > 1 {
    //                     continue 'outer;
    //                 }
    //             }
    //             Ok(word) => {
    //                 blocking_count = 0;
    //                 match recv_count {
    //                     0 => {
    //                         if (word & 0x40) == 0x40 || (word & 0x80) == 0x80 {
    //                             hprintln!("pkt err: 0x{:x}",word).unwrap();
    //                             packet_error = true;
    //                         }
    //                         else {
    //                             let _ = hprintln!("cc: 0x{:x}", word);
    //                             packet_reg_count = word;
    //                             if packet_reg_count != (TEST_REG_COUNT as u8) &&
    //                                 QUERY_TYPE == PACKET_CODE_READ {
    //                                 continue 'outer;
    //                             }
    //                         }
    //                     }
    //                     1 => { hprintln!("crc: 0x{:x}", word).unwrap();}
    //                     2 => { hprintln!("p: {}", word).unwrap();}
    //                     3 => {
    //                         hprintln!("o: {}", word).unwrap();
    //                         if packet_error || (0 == packet_reg_count) {
    //                             //no more packet data will come after this
    //                             continue 'outer;
    //                         }
    //                     }
    //                     4 => {
    //                         hprintln!("[{}] {:x}",recv_count - PACKET_HEADER_LEN, word).unwrap();
    //
    //                         if QUERY_PAGE == PAGE_CONFIG  && word != EXPECTED_PX4IO_PROTOCOL_VERSION {
    //                             hprintln!("proto version: {} expected: {} ", word, EXPECTED_PX4IO_PROTOCOL_VERSION);
    //                             continue 'outer;
    //                         }
    //                         // else {
    //                         //     hprintln!("proto match!!");
    //                         //     continue 'outer;
    //                         // }
    //                     }
    //                     _ => {
    //                         hprintln!("[{}] {:x}",recv_count - PACKET_HEADER_LEN, word).unwrap();
    //                     }
    //                 }
    //                 recv_count += 1;
    //                 if recv_count == QUERY_RESPONSE_LEN {
    //                     continue 'outer;
    //                 }
    //             }
    //             Err(any) => {
    //                 hprintln!("{:?}",any).unwrap();
    //                 continue 'outer;
    //                 //hprintln!(".").unwrap();
    //                 //continue 'outer;
    //             }
    //         }
    //     }
    // }



    if let Some(mut driver) = new_serial_driver(uart8_port) {
        loop {
            let _ = hprintln!("---");
            // 6 == REG_CONFIG_N_ACTUATORS+1
            let mut fetch_regs: [RegisterValue; 6] = [0; 6];
            let _ = driver.get_registers(
                registers::PAGE_CONFIG,
                registers::REG_CONFIG_PROTOCOL_VERSION,
                &mut fetch_regs,
            );

            //hprintln!("fetch_regs: {:x?}", fetch_regs).unwrap();
        }
    }

    loop {
        delay_source.delay_ms(250u8);
    }
}
