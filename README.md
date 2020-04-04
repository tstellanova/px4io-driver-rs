# px4io-driver-rs

Rust no_std library that allows communication with 
the px4io communications co-processor present on most
PX4-based autopilots since Pixhawk version 2. 

The px4io coprocessor handles a number of critical tasks
for many drones/UAVs/MAVs, including:

- PWM output on multiple channels 
- PWM input on multiple channels
- Handling of multiple RC input types such as S.Bus and 
- Failsafe behaviors in case the PX4 FMU processor halts or reboots

## Status

This is work-in-progress

- [ ] Support UART serial interface (blocking read/write)
- [ ] support basic register read/write
- [ ] library builds ok 
- [ ] release library builds ok


## Examples

Run example for eg Durandal stm32h743 board target:
```
cargo run --example pwm_sweep --target thumbv7em-none-eabihf 
```

The `memory.x` ,`.cargo/config`, and `dronecode.gdb` files included with this crate are
configured to run this example by connecting to the Durandal via a dronecode
probe (or similar, such as a Black Magic Probe)


## License

BSD-3-Clause, see `LICENSE` file.
 
