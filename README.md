# MCP23S17 driver

![Crates.io](https://img.shields.io/crates/v/rppal-mcp23s17)
![Crates.io](https://img.shields.io/crates/d/rppal-mcp23s17)
![Crates.io](https://img.shields.io/crates/l/rppal-mcp23s17)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/solimike/rppal-mcp23s17/ci.yml?branch=main)

A driver for the MCP23S17 I/O expander which is accessed over an SPI bus. Note that
this driver depends on [RPPAL](https://docs.golemparts.com/rppal) and is therefore
specific to the [Raspberry Pi](https://www.raspberrypi.org/).

## Example usage

``` rust no_run
use rppal_mcp23s17::{ChipSelect, HardwareAddress, Level, Mcp23s17, Port, RegisterAddress, SpiBus, SpiMode};

// Create an instance of the driver for the device with the hardware address
// (A2, A1, A0) of 0b000.
let mcp23s17 = Mcp23s17::new(
    HardwareAddress::new(0).expect("Invalid hardware address"),
    SpiBus::Spi0,
    ChipSelect::Cs0,
    100_000,
    SpiMode::Mode0,
)
.expect("Failed to create MCP23S17");

// Take ownership of the pin on bit 4 of GPIOA and then convert it into an
// OutputPin. Initialisation of the OutputPin ensures that the MCP23S17
// registers (e.g. IODIRA) are set accordingly.
let pin = mcp23s17
    .get(Port::GpioA, 4)
    .expect("Failed to get Pin")
    .into_output_pin()
    .expect("Failed to convert to OutputPin");

// Set the pin to logic-level low.
pin.write(Level::Low).expect("Bad pin write");
```

## Concurrency Warning

Note that the [`rppal::spi::Spi`] contained in the [`Mcp23s17`] is
[`!Send`](std::marker::Send) so that the device can only be used within the
context of a single thread. However, there is nothing to stop separate instances on
separate threads accessing the same MCP23S17. We could possibly do something smarter
to enforce uniqueness of [`Pin`]s between threads but currently it is down to the user
to ensure if multiple instances are in use they don't tread on each other's toes!

Indeed, there is nothing to stop separate processes accessing the MCP23S17 over the
SPI bus at the same time and given that many bit-flipping operations are implemented
as a read-modify-write on the relevant registers there are huge windows for race
hazards between processes/threads. Clearly much more reliable for everyone if a
single process "owns" the MCP23S17 device and a single thread within that process
instantiates a singleton [`Mcp23s17`] object.

## Acknowledgements

Many of the documentation comments in this library are taken direct from the
[MCP23S17 datasheet](https://www.microchip.com/en-us/product/MCP23S17) and are
© 2005-2022 Microchip Technology Inc. and its subsidiaries.

This library not only uses, but has also taken a lot of inspiration from, the
[RPPAL crate](https://crates.io/crates/rppal).
