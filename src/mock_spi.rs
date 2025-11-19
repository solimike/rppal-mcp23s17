//! A very simple mock version of the SPI interface from `rppal` that allows the
//! MCP23S17 registers to be set and read by the test harness and then accessed over the
//! "SPI" `transfer()` API.
//!
use std::cell::RefCell;

use crate::{ChipSelect, RegisterAddress};

/// A mock for the SPI hardware to use during testing.
///
/// Note that the real SPI manages to use immutable references on its transfer methods
/// so we need to be able to do the same despite actually mutating some internal state
/// hence the use of [`RefCell<_>`].
#[derive(Debug, Default)]
pub struct MockSpi {
    register_values: RefCell<[u8; RegisterAddress::LENGTH]>,
    read_access_count: RefCell<[usize; RegisterAddress::LENGTH]>,
    write_access_count: RefCell<[usize; RegisterAddress::LENGTH]>,
    hardware_present: bool,
}

impl MockSpi {
    /// Crude emulation of the SPI transfer method specific to MCP23S17 use.
    ///
    /// Assumes normally going to be reading single bytes of register data so that the
    /// read and write buffers will both be of length 3.
    ///
    /// Assumes that the second byte of the write buffer is the register address.
    ///
    /// ## Special Case
    ///
    /// Any device created on Bus::Spi6 is simulated to "not exist": reads and writes
    /// both succeed, but reads always return zero. (The mock registers still get updated
    /// and the access counts are maintained as normal.)
    pub(crate) fn transfer(
        &self,
        read_buffer: &mut [u8],
        write_buffer: &[u8],
    ) -> rppal::spi::Result<usize> {
        assert_eq!(read_buffer.len(), 3);
        assert_eq!(write_buffer.len(), 3);

        println!("MockSpi::transfer write={write_buffer:?}");
        let register = write_buffer[1] as usize;
        if (write_buffer[0] & 0b0000_0001) != 0 {
            // Reading from register.
            self.read_access_count.borrow_mut()[register] += 1;
            if self.hardware_present {
                read_buffer[read_buffer.len() - 1] = self.register_values.borrow()[register];
                println!("MockSpi::transfer (hardware present) read={read_buffer:?}");
            } else {
                read_buffer[read_buffer.len() - 1] = 0;
                println!("MockSpi::transfer (NO HARDWARE!) read={read_buffer:?}");
            }
        } else {
            // Writing to register.
            self.write_access_count.borrow_mut()[register] += 1;
            self.register_values.borrow_mut()[register] = write_buffer[2];
        }

        Ok(read_buffer.len())
    }

    /// Store of mock data to a register
    pub(crate) fn set_mock_data(&self, register: RegisterAddress, data: u8) {
        println!("Store mock data (0x{data:02x}) to read from {register:?}");
        self.register_values.borrow_mut()[register as usize] = data;
    }

    /// Get mock data from a register
    ///
    /// Returns a 3-tuple of (`u8`, `usize`, `usize`) containing:
    ///
    /// - The mock register data itself
    /// - How many times the register has been read
    /// - How many times the register has been written
    pub(crate) fn get_mock_data(&self, register: RegisterAddress) -> (u8, usize, usize) {
        let data = self.register_values.borrow()[register as usize];
        let reads = self.read_access_count.borrow()[register as usize];
        let writes = self.write_access_count.borrow()[register as usize];
        println!(
            "Retrieve mock data (0x{data:02x}) written to {register:?} (r={reads} w={writes})"
        );
        (data, reads, writes)
    }

    /// Create a MockSpi setting registers to match real hardware after power-on-reset.
    pub(crate) fn new(
        bus: rppal::spi::Bus,
        chip_select: ChipSelect,
        frequency: u32,
        mode: rppal::spi::Mode,
    ) -> MockSpi {
        println!(
            "Mock SPI created:\n  SPI bus:{bus}\n  CS: {chip_select:?}\n  f: {frequency}\n  mode: {mode}"
        );
        let mut mock_spi = MockSpi::default();

        // Set registers that don't have 0x00 at POR, which is actually only the IODIR
        // registers.
        {
            let mut registers = mock_spi.register_values.borrow_mut();
            registers[RegisterAddress::IODIRA as usize] = 0xff;
            registers[RegisterAddress::IODIRB as usize] = 0xff;
        }

        // Use the value SpiBus::Spi6 as a special case that the hardware is not present.
        mock_spi.hardware_present = !(bus == rppal::spi::Bus::Spi6);
        mock_spi
    }
}
