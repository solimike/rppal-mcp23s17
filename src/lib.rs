#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use std::{cell::RefCell, fmt, rc::Rc, result};

use bitflags::bitflags;
use log::{debug, error};
use rppal::spi::SlaveSelect;

/// Re-exports of [rppal::spi] module APIs used on this crate's APIs. Renamed to make
/// sure that the intended usage is clear.
pub use rppal::spi::{Bus as SpiBus, Mode as SpiMode};

// Run with mock hardware in testing.
#[cfg(any(test, feature = "mockspi"))]
use mock_spi::MockSpi;
#[cfg(not(any(test, feature = "mockspi")))]
use rppal::spi::Spi;

use thiserror::Error;

pub mod pin;
pub use self::pin::{InputPin, InterruptMode, Level, OutputPin, Pin};

//--------------------------------------------------------------------------------------
/// The hardware address of the device - three bits.
///
/// MCP23S17 is a client SPI device. The client address contains four fixed bits and
/// three user-defined hardware address bits (pins `A2`, `A1` and `A0`), if enabled via
/// [`IOCON::HAEN`] with the read/write bit filling out the control byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HardwareAddress(u8);

impl HardwareAddress {
    /// Hardware address space is three bits wide so 0-7 are valid.
    pub const MAX_HARDWARE_ADDRESS: u8 = 7;

    /// Create a HardwareAddress bounds-checking that it is valid.
    pub fn new(address: u8) -> Result<Self> {
        if address <= Self::MAX_HARDWARE_ADDRESS {
            Ok(Self(address))
        } else {
            Err(Mcp23s17Error::HardwareAddressBoundsError(address))
        }
    }
}

impl fmt::Display for HardwareAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&format!("{}", self.0), f)
    }
}

impl TryFrom<u8> for HardwareAddress {
    type Error = Mcp23s17Error;

    fn try_from(value: u8) -> Result<Self> {
        HardwareAddress::new(value)
    }
}

impl From<HardwareAddress> for u8 {
    fn from(addr: HardwareAddress) -> Self {
        addr.0
    }
}

//--------------------------------------------------------------------------------------
/// The direction of the operation on the SPI bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum SpiCommand {
    Write = 0,
    Read = 1,
}

impl fmt::Display for SpiCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SpiCommand::Read => fmt::Display::fmt("Read", f),
            SpiCommand::Write => fmt::Display::fmt("Write", f),
        }
    }
}

//--------------------------------------------------------------------------------------
/// The register address within the device.
///
/// Note that this follows the "interleaved" format for the register addresses so that
/// the [`IOCON::BANK`] bit of [`IOCON`][`RegisterAddress::IOCON`] register must be set
/// to 0 ([`IOCON::BANK_OFF`]).
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum RegisterAddress {
    /// I/O direction A
    IODIRA = 0x0,
    /// I/O direction B
    IODIRB = 0x1,
    /// I/O polarity A
    IPOLA = 0x2,
    /// I/O polarity B
    IPOLB = 0x3,
    /// interrupt enable A
    GPINTENA = 0x4,
    /// interrupt enable B
    GPINTENB = 0x5,
    /// register default value A (interrupts)
    DEFVALA = 0x6,
    /// register default value B (interrupts)
    DEFVALB = 0x7,
    /// interrupt control A
    INTCONA = 0x8,
    /// interrupt control B
    INTCONB = 0x9,
    /// I/O config (also at 0xB)
    IOCON = 0xA,
    /// I/O config (duplicate)
    IOCON2 = 0xB,
    /// port A pull-ups
    GPPUA = 0xC,
    /// port B pull-ups
    GPPUB = 0xD,
    /// interrupt flag A (where the interrupt came from)
    INTFA = 0xE,
    /// interrupt flag B
    INTFB = 0xF,
    /// interrupt capture A (value at interrupt is saved here)
    INTCAPA = 0x10,
    /// interrupt capture B
    INTCAPB = 0x11,
    /// port A
    GPIOA = 0x12,
    /// port B
    GPIOB = 0x13,
    /// output latch A
    OLATA = 0x14,
    /// output latch B
    OLATB = 0x15,
}

/// Total size of the MCP27S17 register space.
impl RegisterAddress {
    /// Total number of registers defined within the MCP23S17.
    pub const LENGTH: usize = 0x16;
}

impl From<RegisterAddress> for u8 {
    fn from(address: RegisterAddress) -> Self {
        address as u8
    }
}

impl TryFrom<usize> for RegisterAddress {
    type Error = Mcp23s17Error;

    fn try_from(value: usize) -> Result<Self> {
        match value {
            x if x == RegisterAddress::IODIRA as usize => Ok(RegisterAddress::IODIRA),
            x if x == RegisterAddress::IODIRB as usize => Ok(RegisterAddress::IODIRB),
            x if x == RegisterAddress::IPOLA as usize => Ok(RegisterAddress::IPOLA),
            x if x == RegisterAddress::IPOLB as usize => Ok(RegisterAddress::IPOLB),
            x if x == RegisterAddress::GPINTENA as usize => Ok(RegisterAddress::GPINTENA),
            x if x == RegisterAddress::GPINTENB as usize => Ok(RegisterAddress::GPINTENB),
            x if x == RegisterAddress::DEFVALA as usize => Ok(RegisterAddress::DEFVALA),
            x if x == RegisterAddress::DEFVALB as usize => Ok(RegisterAddress::DEFVALB),
            x if x == RegisterAddress::INTCONA as usize => Ok(RegisterAddress::INTCONA),
            x if x == RegisterAddress::INTCONB as usize => Ok(RegisterAddress::INTCONB),
            x if x == RegisterAddress::IOCON as usize => Ok(RegisterAddress::IOCON),
            x if x == RegisterAddress::IOCON2 as usize => Ok(RegisterAddress::IOCON2),
            x if x == RegisterAddress::GPPUA as usize => Ok(RegisterAddress::GPPUA),
            x if x == RegisterAddress::GPPUB as usize => Ok(RegisterAddress::GPPUB),
            x if x == RegisterAddress::INTFA as usize => Ok(RegisterAddress::INTFA),
            x if x == RegisterAddress::INTFB as usize => Ok(RegisterAddress::INTFB),
            x if x == RegisterAddress::INTCAPA as usize => Ok(RegisterAddress::INTCAPA),
            x if x == RegisterAddress::INTCAPB as usize => Ok(RegisterAddress::INTCAPB),
            x if x == RegisterAddress::GPIOA as usize => Ok(RegisterAddress::GPIOA),
            x if x == RegisterAddress::GPIOB as usize => Ok(RegisterAddress::GPIOB),
            x if x == RegisterAddress::OLATA as usize => Ok(RegisterAddress::OLATA),
            x if x == RegisterAddress::OLATB as usize => Ok(RegisterAddress::OLATB),
            _ => Err(Mcp23s17Error::RegisterAddressBoundsError),
        }
    }
}

impl fmt::Display for RegisterAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RegisterAddress::IODIRA => fmt::Display::fmt("IODIRA", f),
            RegisterAddress::IODIRB => fmt::Display::fmt("IODIRB", f),
            RegisterAddress::IPOLA => fmt::Display::fmt("IPOLA", f),
            RegisterAddress::IPOLB => fmt::Display::fmt("IPOLB", f),
            RegisterAddress::GPINTENA => fmt::Display::fmt("GPINTENA", f),
            RegisterAddress::GPINTENB => fmt::Display::fmt("GPINTENB", f),
            RegisterAddress::DEFVALA => fmt::Display::fmt("DEFVALA", f),
            RegisterAddress::DEFVALB => fmt::Display::fmt("DEFVALB", f),
            RegisterAddress::INTCONA => fmt::Display::fmt("INTCONA", f),
            RegisterAddress::INTCONB => fmt::Display::fmt("INTCONB", f),
            RegisterAddress::IOCON => fmt::Display::fmt("IOCON", f),
            RegisterAddress::IOCON2 => fmt::Display::fmt("IOCON (2)", f),
            RegisterAddress::GPPUA => fmt::Display::fmt("GPPUA", f),
            RegisterAddress::GPPUB => fmt::Display::fmt("GPPUB", f),
            RegisterAddress::INTFA => fmt::Display::fmt("INTFA", f),
            RegisterAddress::INTFB => fmt::Display::fmt("INTFB", f),
            RegisterAddress::INTCAPA => fmt::Display::fmt("INTCAPA", f),
            RegisterAddress::INTCAPB => fmt::Display::fmt("INTCAPB", f),
            RegisterAddress::GPIOA => fmt::Display::fmt("GPIOA", f),
            RegisterAddress::GPIOB => fmt::Display::fmt("GPIOB", f),
            RegisterAddress::OLATA => fmt::Display::fmt("OLATA", f),
            RegisterAddress::OLATB => fmt::Display::fmt("OLATB", f),
        }
    }
}

//--------------------------------------------------------------------------------------

bitflags! {
    /// I/O Expander Configuration Register (`IOCON`) bit definitions.
    pub struct IOCON: u8 {
        /// Controls how the registers are addressed:
        ///
        ///   1 = The registers associated with each port are separated into different
        ///       banks. (*Not currently supported in this library.*)
        ///
        ///   0 = The registers are in the same bank (addresses are sequential).
        const BANK = 0b1000_0000;

        /// `INT` Pins Mirror bit:
        ///
        ///   1 = The `INT` pins are internally connected.
        ///
        ///   0 = The `INT` pins are not connected. `INTA` is associated with `PORTA`
        ///       and `INTB` is associated with `PORTB`.
        const MIRROR = 0b0100_0000;

        /// Sequential Operation mode bit:
        ///
        ///   1 = Sequential operation disabled, address pointer does not increment.
        ///
        ///   0 = Sequential operation enabled, address pointer increments.
        const SEQOP = 0b0010_0000;

        /// Slew Rate control bit for SDA output:
        ///
        ///   1 = Slew rate control disabled.
        ///
        ///   0 = Slew rate control enabled.
        const DISSLW = 0b0001_0000;

        /// Hardware Address Enable bit:
        ///
        ///   1 = Enables the MCP23S17 address pins.
        ///
        ///   0 = Disables the MCP23S17 address pins.
        const HAEN = 0b0000_1000;

        /// Configures the `INT` pin as an open-drain output:
        ///
        ///   1 = Open-drain output (overrides the `INTPOL` bit.)
        ///
        ///   0 = Active driver output (`INTPOL` bit sets the polarity.)
        const ODR = 0b0000_0100;

        /// Sets the polarity of the `INT` output pin:
        ///
        ///   1 = Active-high.
        ///
        ///   0 = Active-low.
        const INTPOL = 0b0000_0010;

        /// Unimplemented: Read as 0.
        const _NA = 0b0000_0001;
    }
}

impl IOCON {
    /// The registers associated with each port are separated into different
    /// banks. (*Not currently supported in this library.*)
    pub const BANK_ON: IOCON = IOCON::BANK;
    /// The registers are in the same bank (addresses are interleaved sequentially).
    pub const BANK_OFF: IOCON = IOCON { bits: 0 };
    /// The `INT` pins are internally connected.
    pub const MIRROR_ON: IOCON = IOCON::MIRROR;
    /// The `INT` pins are not connected. `INTA` is associated with `PORTA` and `INTB`
    /// is associated with `PORTB`.
    pub const MIRROR_OFF: IOCON = IOCON { bits: 0 };
    /// Sequential operation enabled, address pointer increments.
    pub const SEQOP_ON: IOCON = IOCON { bits: 0 };
    /// Sequential operation disabled, address pointer does not increment.
    pub const SEQOP_OFF: IOCON = IOCON::SEQOP;
    /// Slew rate control enabled.
    pub const DISSLW_SLEW_RATE_CONTROLLED: IOCON = IOCON { bits: 0 };
    /// Slew rate control disabled.
    pub const DISSLW_SLEW_RATE_MAX: IOCON = IOCON::DISSLW;
    /// Enables the MCP23S17 address pins.
    pub const HAEN_ON: IOCON = IOCON::HAEN;
    /// Disables the MCP23S17 address pins.
    pub const HAEN_OFF: IOCON = IOCON { bits: 0 };
    /// Open-drain output (overrides the `INTPOL` bit.)
    pub const ODR_ON: IOCON = IOCON::ODR;
    /// Active driver output (`INTPOL` bit sets the polarity.)
    pub const ODR_OFF: IOCON = IOCON { bits: 0 };
    /// Active-high.
    pub const INTPOL_HIGH: IOCON = IOCON::INTPOL;
    /// Active-low.
    pub const INTPOL_LOW: IOCON = IOCON { bits: 0 };
}

/// The MCP23S17 has two GPIO ports, GPIOA and GPIOB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Port {
    /// GPIO A
    GpioA,
    /// GPIO B
    GpioB,
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Port::GpioA => fmt::Display::fmt("GPIO A", f),
            Port::GpioB => fmt::Display::fmt("GPIO B", f),
        }
    }
}

//--------------------------------------------------------------------------------------

/// Which `Chip Select` line to use on the SPI bus.
///
/// This is a purely cosmetic facade in front of [`SlaveSelect`] so that we use less
/// contentious language in our public API. Whilst both `CS` and `SS` terms are used
/// across existing documentation, the "Chip Select" term seems the one favoured by the
/// [Linux documentation for spidev](https://www.kernel.org/doc/html/latest/spi/spidev.html).
///
/// Since the definitions are homologous, hopefully the compiler will make this a zero-cost
/// abstraction and, if not, it will at least be extremely cheap.
///
/// Which Chip Select lines are used for the different busses on the Raspberry Pi is
/// documented in detail as part of the [`rppal::spi`] module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum ChipSelect {
    Cs0 = 0,
    Cs1 = 1,
    Cs2 = 2,
    Cs3 = 3,
    Cs4 = 4,
    Cs5 = 5,
    Cs6 = 6,
    Cs7 = 7,
    Cs8 = 8,
    Cs9 = 9,
    Cs10 = 10,
    Cs11 = 11,
    Cs12 = 12,
    Cs13 = 13,
    Cs14 = 14,
    Cs15 = 15,
}

impl From<SlaveSelect> for ChipSelect {
    fn from(ss: SlaveSelect) -> Self {
        match ss {
            SlaveSelect::Ss0 => ChipSelect::Cs0,
            SlaveSelect::Ss1 => ChipSelect::Cs1,
            SlaveSelect::Ss2 => ChipSelect::Cs2,
            SlaveSelect::Ss3 => ChipSelect::Cs3,
            SlaveSelect::Ss4 => ChipSelect::Cs4,
            SlaveSelect::Ss5 => ChipSelect::Cs5,
            SlaveSelect::Ss6 => ChipSelect::Cs6,
            SlaveSelect::Ss7 => ChipSelect::Cs7,
            SlaveSelect::Ss8 => ChipSelect::Cs8,
            SlaveSelect::Ss9 => ChipSelect::Cs9,
            SlaveSelect::Ss10 => ChipSelect::Cs10,
            SlaveSelect::Ss11 => ChipSelect::Cs11,
            SlaveSelect::Ss12 => ChipSelect::Cs12,
            SlaveSelect::Ss13 => ChipSelect::Cs13,
            SlaveSelect::Ss14 => ChipSelect::Cs14,
            SlaveSelect::Ss15 => ChipSelect::Cs15,
        }
    }
}

impl From<ChipSelect> for SlaveSelect {
    fn from(cs: ChipSelect) -> Self {
        match cs {
            ChipSelect::Cs0 => SlaveSelect::Ss0,
            ChipSelect::Cs1 => SlaveSelect::Ss1,
            ChipSelect::Cs2 => SlaveSelect::Ss2,
            ChipSelect::Cs3 => SlaveSelect::Ss3,
            ChipSelect::Cs4 => SlaveSelect::Ss4,
            ChipSelect::Cs5 => SlaveSelect::Ss5,
            ChipSelect::Cs6 => SlaveSelect::Ss6,
            ChipSelect::Cs7 => SlaveSelect::Ss7,
            ChipSelect::Cs8 => SlaveSelect::Ss8,
            ChipSelect::Cs9 => SlaveSelect::Ss9,
            ChipSelect::Cs10 => SlaveSelect::Ss10,
            ChipSelect::Cs11 => SlaveSelect::Ss11,
            ChipSelect::Cs12 => SlaveSelect::Ss12,
            ChipSelect::Cs13 => SlaveSelect::Ss13,
            ChipSelect::Cs14 => SlaveSelect::Ss14,
            ChipSelect::Cs15 => SlaveSelect::Ss15,
        }
    }
}

//--------------------------------------------------------------------------------------

/// Errors that operation of the MCP23S17 can raise.
#[derive(Error, Debug)]
pub enum Mcp23s17Error {
    /// Errors from the [SPI][rppal::spi::Spi].
    #[error("SPI error")]
    SpiError {
        /// Underlying error source.
        #[from]
        source: rppal::spi::Error,
    },

    /// Attempt to access an MCP23S17 beyond the hardware address range
    /// (0 - [`HardwareAddress::MAX_HARDWARE_ADDRESS`]).
    #[error("Hardware address out of range")]
    HardwareAddressBoundsError(u8),

    /// Attempt to access an MCP23S17 register beyond the valid set defined in
    /// [`RegisterAddress`].
    #[error("Register address out of range")]
    RegisterAddressBoundsError,

    /// The [SPI][rppal::spi::Spi] reported a number of bytes transferred that did not
    /// match expected length.
    #[error("Unexpected number of bytes read")]
    UnexpectedReadLength(usize),

    /// Either a [`Pin`] was requested beyond the width of the byte-wide GPIO port or
    /// the [`Pin`] has already been taken.
    #[error("Pin out of range or already in use")]
    PinNotAvailable(u8),

    /// A bit operation was attempted on an MCP23S17 register on an invalid (greater
    /// than 7) bit number.
    #[error("Specified bit is out of range 0-7")]
    RegisterBitBoundsError(u8),
}

/// Convenient wrapper for Result types can have [`Mcp23s17Error`]s.
pub type Result<T> = result::Result<T, Mcp23s17Error>;

/// Struct to represent the state of an MCP23S17 I/O Expander.
///
/// This is separated from the `Mcp23s17` itself so that the state can be shared between
/// various `Pin` objects etc.
///
/// In testing environments this uses mocked hardware.
#[derive(Debug)]
struct Mcp23s17State {
    #[cfg(not(any(test, feature = "mockspi")))]
    spi: Spi,

    #[cfg(any(test, feature = "mockspi"))]
    spi: MockSpi,

    /// The SPI bus the device is connected to.
    spi_bus: SpiBus,

    /// The hardware address on the bus.
    address: HardwareAddress,

    /// Keep track of which pins are in use on `GPIOA`.
    gpioa_pins_taken: [bool; 8],

    /// Keep track of which pins are in use on `GPIOB`.
    gpiob_pins_taken: [bool; 8],
}

/// A structure that represents an instance of the MCP23S17 I/O expander chip.
///
/// This is the key entrypoint into the driver. The user instantiates an `Mcp23s17` and
/// then uses [`Mcp23s17::get()`] to acquire an unconfigured GPIO [`Pin`]. The [`Pin`] is
/// then configured by turning it into an [`InputPin`] or [`OutputPin`] through one of the
/// `Pin::into_*()` methods.
///
/// ```no_run
/// use rppal_mcp23s17::{ChipSelect, HardwareAddress, Level, Mcp23s17, SpiBus, SpiMode, Port, RegisterAddress};
///
/// // Create an instance of the driver for the device with the hardware address
/// // (A2, A1, A0) of 0b000.
/// let mcp23s17 = Mcp23s17::new(
///     HardwareAddress::new(0).expect("Invalid hardware address"),
///     SpiBus::Spi0,
///     ChipSelect::Cs0,
///     100_000,
///     SpiMode::Mode0,
/// )
/// .expect("Failed to create MCP23S17");
///
/// // Take ownership of the pin on bit 4 of GPIOA and then convert it into an
/// // OutputPin. Initialisation of the OutputPin ensures that the MCP23S17
/// // registers (e.g. IODIRA) are set accordingly.
/// let pin = mcp23s17.get(Port::GpioA, 4).expect("Failed to get Pin");
/// ```
#[derive(Debug)]
pub struct Mcp23s17 {
    mcp23s17_state: Rc<RefCell<Mcp23s17State>>,
}

impl Mcp23s17 {
    /// Create an MCP23S17 instance with either real or mock hardware.
    ///
    /// For now testing always uses mock hardware, which precludes running unit tests on
    /// real hardware. In practice, that's not much of a practical limitation when running
    /// tests in local or CI cross-compilation environments. Testing on real hardware
    /// focuses on integration testing with the full build.
    pub fn new(
        address: HardwareAddress,
        spi_bus: SpiBus,
        chip_select: ChipSelect,
        spi_clock: u32,
        spi_mode: SpiMode,
    ) -> Result<Self> {
        let mcp23s17_state = Mcp23s17State {
            #[cfg(not(any(test, feature = "mockspi")))]
            spi: Spi::new(spi_bus, chip_select.into(), spi_clock, spi_mode)?,
            #[cfg(any(test, feature = "mockspi"))]
            spi: MockSpi::new(spi_bus, chip_select, spi_clock, spi_mode),
            spi_bus,
            address,
            gpioa_pins_taken: [false; 8],
            gpiob_pins_taken: [false; 8],
        };
        Ok(Mcp23s17 {
            mcp23s17_state: Rc::new(RefCell::new(mcp23s17_state)),
        })
    }

    /// Read a byte from the MCP23S17 register at the address `register`.
    pub fn read(&self, register: RegisterAddress) -> Result<u8> {
        self.mcp23s17_state.borrow().read(register)
    }

    /// Write the byte `data` to the MCP23S17 register at address `register`.
    pub fn write(&self, register: RegisterAddress, data: u8) -> Result<()> {
        self.mcp23s17_state.borrow().write(register, data)
    }

    /// Set the specified bits in the register.
    ///
    /// Sets the bits by first reading the MCP23S17 register at `register` and then ORing
    /// it with `data` before writing it back to `register`. Note the race-hazard if
    /// there are multiple [`Mcp23s17`]s that can be writing to the same device.
    pub fn set_bits(&self, register: RegisterAddress, data: u8) -> Result<()> {
        self.mcp23s17_state.borrow().set_bits(register, data)
    }

    /// Clear the specified bits in the register.
    ///
    /// Clears the bits by first reading the MCP23S17 register at `register` and then ANDing
    /// it with `!data` before writing it back to `register`. Note the race-hazard if
    /// there are multiple [`Mcp23s17`]s that can be writing to the same device.
    pub fn clear_bits(&self, register: RegisterAddress, data: u8) -> Result<()> {
        self.mcp23s17_state.borrow().clear_bits(register, data)
    }

    /// Set the specified bit in the register.
    ///
    /// Sets the bit at position `bit` (0-7) by first reading the MCP23S17 register at
    /// `register` and then ORing with a mask with the appropriate bit set before
    /// writing it back to `register`. Note the race-hazard if there are multiple
    /// [`Mcp23s17`]s that can be writing to the same device.
    pub fn set_bit(&self, register: RegisterAddress, bit: u8) -> Result<()> {
        self.mcp23s17_state.borrow().set_bit(register, bit)
    }

    /// Clear the specified bit in the register.
    ///
    /// Clears the bit at position `bit` (0-7) by first reading the MCP23S17 register at
    /// `register` and then ANDing with a mask with the appropriate bit cleared before
    /// writing it back to `register`. Note the race-hazard if there are multiple
    /// [`Mcp23s17`]s that can be writing to the same device.
    pub fn clear_bit(&self, register: RegisterAddress, bit: u8) -> Result<()> {
        self.mcp23s17_state.borrow().clear_bit(register, bit)
    }

    /// Get the specified bit in the register.
    ///
    /// Gets the bit at position `bit` (0-7) by first reading the MCP23S17 register at
    /// `register` and then ANDing with a mask with the appropriate bit set before
    /// converting to a [`Level`].
    pub fn get_bit(&self, register: RegisterAddress, bit: u8) -> Result<Level> {
        self.mcp23s17_state.borrow().get_bit(register, bit)
    }

    /// Returns a [`Pin`] for the specified GPIO port and pin number.
    ///
    /// Retrieving a GPIO pin grants access to the pin through an owned [`Pin`] instance.
    /// If the pin is already in use, or the pin number `pin` is greater than 7 then
    /// `Mcp23s17::get()` returns `Err(`[`Mcp23s17Error::PinNotAvailable`]`)`.
    ///
    /// After a [`Pin`] (or a derived [`InputPin`] or [`OutputPin`]) goes out of scope,
    /// it can be retrieved again through another `get()` call.
    pub fn get(&self, port: Port, pin: u8) -> Result<Pin> {
        if pin > 7 {
            return Err(Mcp23s17Error::PinNotAvailable(pin));
        }

        // Returns an error if the pin is already taken, otherwise sets it to true here
        // Since we are guaranteed to be single-threaded this doesn't need to worry
        // about synchronisation or races.
        match port {
            Port::GpioA => {
                if self.mcp23s17_state.borrow().gpioa_pins_taken[pin as usize] {
                    return Err(Mcp23s17Error::PinNotAvailable(pin));
                }
                {
                    self.mcp23s17_state.borrow_mut().gpioa_pins_taken[pin as usize] = true;
                }
                Ok(Pin::new(port, pin, self.mcp23s17_state.clone()))
            }
            Port::GpioB => {
                if self.mcp23s17_state.borrow().gpiob_pins_taken[pin as usize] {
                    return Err(Mcp23s17Error::PinNotAvailable(pin));
                }
                {
                    self.mcp23s17_state.borrow_mut().gpiob_pins_taken[pin as usize] = true;
                }
                Ok(Pin::new(port, pin, self.mcp23s17_state.clone()))
            }
        }
    }

    /// Get the SPI bus that the MCP23S17 is accessed over.
    pub fn get_spi_bus(&self) -> SpiBus {
        self.mcp23s17_state.borrow().spi_bus
    }

    /// Get the hardware address of the MCP23S17.
    pub fn get_hardware_address(&self) -> HardwareAddress {
        self.mcp23s17_state.borrow().address
    }

    /// In testing environments provide an API to read the MockSpi registers.
    #[cfg(any(feature = "mockspi", test))]
    pub fn get_mock_data(&self, register: RegisterAddress) -> (u8, usize, usize) {
        self.mcp23s17_state.borrow().spi.get_mock_data(register)
    }

    /// In testing environments provide an API to write the MockSpi registers.
    #[cfg(any(feature = "mockspi", test))]
    pub fn set_mock_data(&self, register: RegisterAddress, data: u8) {
        self.mcp23s17_state
            .borrow_mut()
            .spi
            .set_mock_data(register, data);
    }
}

impl Mcp23s17State {
    /// Read an MCP23S17 register.
    fn read(&self, register: RegisterAddress) -> Result<u8> {
        debug!("Read {register:?}");

        let mut read_buffer = [0u8; 3];
        let mut write_buffer = [0u8; 3];
        write_buffer[0] = self.spi_control_byte(SpiCommand::Read);
        write_buffer[1] = register as u8;

        let read_length = self.spi.transfer(&mut read_buffer, &write_buffer)?;
        if read_length != 3 {
            error!("Unexpected number of bytes read ({read_length})");
            return Err(Mcp23s17Error::UnexpectedReadLength(read_length));
        }
        debug!("Read value = 0x{:02x}", read_buffer[2]);
        Ok(read_buffer[2])
    }

    /// Write an MCP23S17 register.
    fn write(&self, register: RegisterAddress, data: u8) -> Result<()> {
        debug!("Write 0x{data:02x} to {register:?}");

        let mut read_buffer = [0u8; 3];
        let mut write_buffer = [0u8; 3];
        write_buffer[0] = self.spi_control_byte(SpiCommand::Write);
        write_buffer[1] = register as u8;
        write_buffer[2] = data;

        let read_length = self.spi.transfer(&mut read_buffer, &write_buffer)?;
        if read_length != 3 {
            error!("Unexpected number of bytes read ({read_length})");
            return Err(Mcp23s17Error::UnexpectedReadLength(read_length));
        }
        Ok(())
    }

    /// Set the specified bits in the register.
    fn set_bits(&self, register: RegisterAddress, data: u8) -> Result<()> {
        debug!("Set bits {data:08b} in {register:?}");
        self.write(register, self.read(register)? | data)
    }

    /// Clear the specified bits in the register.
    fn clear_bits(&self, register: RegisterAddress, data: u8) -> Result<()> {
        debug!("Clear bits {data:08b} in {register:?}");
        self.write(register, self.read(register)? & !data)
    }

    /// Set the specified bit (0-7) in the register.
    fn set_bit(&self, register: RegisterAddress, bit: u8) -> Result<()> {
        debug!("Set bit {bit} in {register:?}");
        if bit > 7 {
            error!("Set bit {bit} is out of range (0-7)!");
            return Err(Mcp23s17Error::RegisterBitBoundsError(bit));
        }
        self.set_bits(register, 0x01 << bit)
    }

    /// Clear the specified bit (0-7) in the register.
    fn clear_bit(&self, register: RegisterAddress, bit: u8) -> Result<()> {
        debug!("Clear bit {bit} in {register:?}");
        if bit > 7 {
            error!("Clear bit {bit} is out of range (0-7)!");
            return Err(Mcp23s17Error::RegisterBitBoundsError(bit));
        }
        self.clear_bits(register, 0x01 << bit)
    }

    /// Read the level of the specified bit (0-7).
    fn get_bit(&self, register: RegisterAddress, bit: u8) -> Result<Level> {
        debug!("Get bit {bit} in {register:?}");
        if bit > 7 {
            error!("Get bit {bit} is out of range (0-7)!");
            return Err(Mcp23s17Error::RegisterBitBoundsError(bit));
        }
        Ok((self.read(register)? & (0x01 << bit)).into())
    }

    /// Calculate the control byte to use in a message.
    ///
    /// The client address contains four fixed bits and three user-defined hardware
    /// address bits (if enabled via `IOCON::HAEN`) (pins A2, A1 and A0) with the
    /// read/write bit filling out the control byte.
    fn spi_control_byte(&self, command: SpiCommand) -> u8 {
        let control_byte = 0x40 | self.address.0 << 1 | command as u8;
        debug!(
            "ControlByte: 0x{:02x} (Command='{:?}' address={:?})",
            control_byte,
            command,
            u8::from(self.address)
        );
        control_byte
    }
}

#[cfg(any(test, feature = "mockspi"))]
pub mod mock_spi;

#[cfg(test)]
mod test;
