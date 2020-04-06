#![no_main]
#![no_std]

extern crate panic_semihosting;

use p_hal::{pac, prelude::*};
use stm32h7xx_hal as p_hal;

use cortex_m_rt::entry;
use p_hal::interrupt;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use core::fmt::Write;
use cortex_m_semihosting::hprintln;

use px4io_driver::{new_serial_driver, registers, RegisterValue};
use core::ops::DerefMut;


type Usart8PortType = p_hal::serial::Serial<
    stm32h7::stm32h743::UART8,
    (p_hal::gpio::gpioe::PE1<p_hal::gpio::Alternate<p_hal::gpio::AF8>>,
     p_hal::gpio::gpioe::PE0<p_hal::gpio::Alternate<p_hal::gpio::AF8>>)
>;
// static UART8_PORT:  Mutex<RefCell<Option< Usart8PortType >>> = Mutex::new(RefCell::new(None));

// type Px4ioDriverType = IoMcuDriver<SerialInterface<Usart8PortType>>;
// static PX4IO_DRIVER:  Mutex<RefCell<Option< Px4ioDriverType >>> = Mutex::new(RefCell::new(None));


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
    let uart8_port:Usart8PortType = {
        let config = p_hal::serial::config::Config::default()
            .baudrate(1_500_000_u32.bps());
        let rx = gpioe.pe0.into_alternate_af8();
        let tx = gpioe.pe1.into_alternate_af8();
        let mut port = dp.UART8.usart((tx, rx), config, &mut ccdr).unwrap();
        //port.listen(p_hal::serial::Event::GenError); //CR3_EIE
        //port.listen(p_hal::serial::Event::Idle); //CR1_IDLEIE
        port
    };



    // //setup DMA
    // let dma1_channels: () =  dp.DMA1.split(); //dma1::Channels =
    // let _dma1_channel2: () = dma1_channels.2; //dma1::Channel2 =
    //
    if let Some(mut iomcu_driver) = new_serial_driver(uart8_port) {
        loop {
            delay_source.delay_ms(250u8);

            let mut values: [RegisterValue; 5] = [0; 5];
            let mut offset = registers::REG_CONFIG_PROTOCOL_VERSION;
            let _ = writeln!(console_tx, "---\r").unwrap();
            if iomcu_driver
                .get_registers(registers::PAGE_CONFIG, offset, &mut values)
                .is_ok()
            {
                writeln!(console_tx, "{}: {:x?} \r", offset, values).unwrap();
            }


            delay_source.delay_ms(250u8);
            let mut values: [RegisterValue; 4] = [0; 4];
            offset = registers::REG_CONFIG_N_RC_INPUTS;
            if iomcu_driver
                .get_registers(registers::PAGE_CONFIG, offset, &mut values)
                .is_ok()
            {
                writeln!(console_tx, "{}: {:x?} \r", offset, values).unwrap();
            }

        }
    }

    loop {
        delay_source.delay_ms(250u8);
    }
}

// // Interrupt handler for the UART8 interrupt
// #[interrupt]
// fn UART8() {
//     if let Some(ref mut driver) = PX4IO_DRIVER.borrow(cs).borrow_mut().deref_mut() {
//
//     }
//
//     //clear_interrupt_flags();
//
//     // TODO Clear reason for the generated interrupt request
//     //rICR = sr & rISR_ERR_FLAGS_MASK;  /* clear flags */
//
//     // ..
//     // Clear reason for the generated interrupt request
// }

// fn setup_uart_dma<T>(port: p_hal::serial::Serial) -> p_hal::serial::Serial
// {
// // #define PX4IO_SERIAL_VECTOR            STM32_IRQ_UART8
// // #define PX4IO_SERIAL_TX_DMAMAP         DMAMAP_UART8_TX
// // #define PX4IO_SERIAL_RX_DMAMAP         DMAMAP_UART8_RX
//
//
// }