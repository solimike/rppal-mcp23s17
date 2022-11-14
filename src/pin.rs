//! Various flavours of "Pin" that the I/O Expander GPIO ports support.
//!
//! * [`InputPin`] - GPIO input that may either be high impedance or have an internal
//!                  pull-up resistor connected.
//! * [`OutputPin`] - GPIO output that can be initialised to high or low [`Level`].
//!
//! # Acknowledgements
//!
//! The design of this module is heavily influenced by the
//! [RPPAL GPIO design](https://github.com/golemparts/rppal/blob/master/src/gpio.rs)

use std::{cell::RefCell, fmt, ops::Not, rc::Rc};

use super::{Mcp23s17State, Port, RegisterAddress, Result};

// There is a lot of repetitious code in each of the flavours of [`Pin`] so use macros
// to reduce that complexity.

/// Create the functions impl block for all the flavours of [`Pin`] that support input.
///
/// Note that this macro should be called from within an existing impl block.
macro_rules! impl_input {
    () => {
        /// Reads the pin's logic level.
        #[inline]
        pub fn read(&self) -> Result<Level> {
            self.pin.read()
        }

        /// Reads the pin's logic level, and returns [`true`] if it is set to
        /// [`Level::Low`].
        #[inline]
        pub fn is_low(&self) -> Result<bool> {
            Ok(self.pin.read()? == Level::Low)
        }

        /// Reads the pin's logic level, and returns [`true`] if it is set to
        /// [`Level::High`].
        #[inline]
        pub fn is_high(&self) -> Result<bool> {
            Ok(self.pin.read()? == Level::High)
        }

        /// Gets the pin's bit number (0-7).
        #[inline]
        pub fn get_pin_number(&self) -> u8 {
            self.pin.pin
        }
    };
}

/// Pin logic levels.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum Level {
    /// Low logic-level.
    Low = 0,
    /// High logic-level.
    High = 1,
}

impl From<bool> for Level {
    fn from(e: bool) -> Level {
        match e {
            true => Level::High,
            false => Level::Low,
        }
    }
}

impl From<Level> for bool {
    fn from(level: Level) -> Self {
        level == Level::High
    }
}

impl From<rppal::gpio::Level> for Level {
    fn from(level: rppal::gpio::Level) -> Self {
        match level {
            rppal::gpio::Level::Low => Self::Low,
            rppal::gpio::Level::High => Self::High,
        }
    }
}

impl From<Level> for rppal::gpio::Level {
    fn from(level: Level) -> Self {
        match level {
            Level::Low => Self::Low,
            Level::High => Self::High,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Level::Low => write!(f, "Low"),
            Level::High => write!(f, "High"),
        }
    }
}

impl From<u8> for Level {
    fn from(value: u8) -> Self {
        if value == 0 {
            Level::Low
        } else {
            Level::High
        }
    }
}

impl Not for Level {
    type Output = Level;

    fn not(self) -> Level {
        match self {
            Level::Low => Level::High,
            Level::High => Level::Low,
        }
    }
}

/// InputPin modes.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InputPinMode {
    /// The input pin is high-impedance (e.g. driven from a logic gate.)
    HighImpedance = 0,
    /// The input pin has an internal pull-up resistor connected (e.g. driven by switch
    /// contacts).
    PullUp = 1,
}

impl fmt::Display for InputPinMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            InputPinMode::HighImpedance => write!(f, "High Impedance"),
            InputPinMode::PullUp => write!(f, "Pull Up"),
        }
    }
}

/// Interrupt input trigger modes that an InputPin supports.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum InterruptMode {
    /// Interrupts are disabled.
    None,
    /// Interrupts are raised when the input is [`Level::High`] and so will typically
    /// happen on the [`Level::Low`] to [`Level::High`] transition. If interrupts are
    /// re-enabled while the input remains `High`, a new interrupt will be raised
    /// without another transition being necessary.
    ActiveHigh,
    /// Interrupts are raised when the input is [`Level::Low`] and so will typically
    /// happen on the [`Level::High`] to [`Level::Low`] transition. If interrupts are
    /// re-enabled while the input remains `Low`, a new interrupt will be raised
    /// without another transition being necessary.
    ActiveLow,
    /// Interrupts are enabled on both the [`Level::High`] to [`Level::Low`] transition
    /// and the  [`Level::Low`] to [`Level::High`] transition. If interrupts are
    /// re-enabled while the input remains in the state that triggered the interrupt, a
    /// new interrupt will _not_ be raised until another transition to the opposite
    /// state occurs.
    BothEdges,
}

impl fmt::Display for InterruptMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            InterruptMode::None => write!(f, "Off"),
            InterruptMode::ActiveHigh => write!(f, "↑"),
            InterruptMode::ActiveLow => write!(f, "↓"),
            InterruptMode::BothEdges => write!(f, "⇅"),
        }
    }
}

/// An unconfigured GPIO pin that implements functionality shared between pin types.
///
/// An instance of a [`Pin`] can be converted into a configured pin type using one of the
/// `into_*` methods that consume the [`Pin`] and return the specific pin type.
#[derive(Debug)]
pub struct Pin {
    port: Port,
    pub(crate) pin: u8,
    mcp23s17_state: Rc<RefCell<Mcp23s17State>>,
}

/// A pin on a GPIO port configured for input.
///
/// Two flavours exist that: either a high-impedance input (_e.g._ driven by an external
/// logic gate) or an input with a pull-up for use with switch inputs. Use the
/// appropriate method on the [`Pin`]: [`Pin::into_input_pin`] or
/// [`Pin::into_pullup_input_pin`].
#[derive(Debug)]
pub struct InputPin {
    pin: Pin,
    /// Whether interrupts are enabled - controls `Drop` behaviour.
    interrupts_enabled: bool,
}

/// A pin on a GPIO port configured for output.
#[derive(Debug)]
pub struct OutputPin {
    pin: Pin,
}

impl Pin {
    /// Create a new pin that maintains a reference to the MCP23S17.
    ///
    /// Generally this will be converted into a specific kind of Pin (_e.g._ InputPin)
    /// through one of the various `into_xxx()` methods.
    pub(crate) fn new(port: Port, pin: u8, mcp23s17_state: Rc<RefCell<Mcp23s17State>>) -> Pin {
        Pin {
            port,
            pin,
            mcp23s17_state,
        }
    }

    /// Read the state of the pin.
    pub fn read(&self) -> Result<Level> {
        match self.port {
            Port::GpioA => Ok(Level::from(
                self.mcp23s17_state.borrow().read(RegisterAddress::GPIOA)? & (0x01 << self.pin),
            )),
            Port::GpioB => Ok(Level::from(
                self.mcp23s17_state.borrow().read(RegisterAddress::GPIOB)? & (0x01 << self.pin),
            )),
        }
    }

    /// Turn the unconfigured `Pin` into an `InputPin` consuming the `Pin` in the process.
    ///
    /// The InputPin is high-impedance (does not have internal pull-up resistor
    /// connected).
    pub fn into_input_pin(self) -> Result<InputPin> {
        InputPin::new(self, InputPinMode::HighImpedance)
    }

    /// Turn the unconfigured `Pin` into an `InputPin` consuming the `Pin` in the process.
    ///
    /// The InputPin has internal pull-up resistor connected.
    pub fn into_pullup_input_pin(self) -> Result<InputPin> {
        InputPin::new(self, InputPinMode::PullUp)
    }

    /// Turn the unconfigured `Pin` into an `OutputPin` consuming the `Pin` in the process.
    pub fn into_output_pin(self) -> Result<OutputPin> {
        OutputPin::new(self)
    }

    /// Turn the unconfigured `Pin` into an `OutputPin` consuming the `Pin` in the process.
    ///
    /// Initialise the pin to be high.
    pub fn into_output_pin_high(self) -> Result<OutputPin> {
        let pin = OutputPin::new(self)?;
        pin.set_high()?;
        Ok(pin)
    }

    /// Turn the unconfigured `Pin` into an `OutputPin` consuming the `Pin` in the process.
    ///
    /// Initialise the pin to be low.
    pub fn into_output_pin_low(self) -> Result<OutputPin> {
        let pin = OutputPin::new(self)?;
        pin.set_low()?;
        Ok(pin)
    }
}

impl Drop for Pin {
    fn drop(&mut self) {
        self.mcp23s17_state.borrow_mut().gpioa_pins_taken[self.pin as usize] = false;
    }
}

impl InputPin {
    /// Constructs an `InputPin` consuming the unconfigured `Pin` in the process.
    ///
    /// Sets the direction of the appropriate GPIO line and configuration of the Pull-up
    /// control register.
    fn new(pin: Pin, mode: InputPinMode) -> Result<Self> {
        // Set the direction of the GPIO port.
        // Need to scope to drop the reference to the MCP23S17 state before we move the
        // pin into the return value.
        {
            let mcp23s17_state = pin.mcp23s17_state.borrow();
            mcp23s17_state.set_bit(
                if pin.port == Port::GpioA {
                    RegisterAddress::IODIRA
                } else {
                    RegisterAddress::IODIRB
                },
                pin.pin,
            )?;

            // Set whether pull-up is used, or not.
            match mode {
                InputPinMode::HighImpedance => mcp23s17_state.clear_bit(
                    if pin.port == Port::GpioA {
                        RegisterAddress::GPPUA
                    } else {
                        RegisterAddress::GPPUB
                    },
                    pin.pin,
                )?,
                InputPinMode::PullUp => mcp23s17_state.set_bit(
                    if pin.port == Port::GpioA {
                        RegisterAddress::GPPUA
                    } else {
                        RegisterAddress::GPPUB
                    },
                    pin.pin,
                )?,
            }
        }
        Ok(InputPin {
            pin,
            interrupts_enabled: false,
        })
    }

    /// Set the [`InputPin`] to the requested `mode` (_i.e._ which edge(s) on the input
    /// trigger an interrupt.)
    ///
    /// Note that setting an `mode` of [`InterruptMode::None`] disables
    /// interrupts.
    ///
    /// The relevant register bits are set according to the following table:
    ///
    /// | Mode                           | `GPINTEN` | `INTCON` | `DEFVAL` |
    /// |--------------------------------|:---------:|:--------:|:--------:|
    /// | [`InterruptMode::None`]        |    `L`    |    `X`   |   `X`    |
    /// | [`InterruptMode::ActiveHigh`]  |    `H`    |    `H`   |   `L`    |
    /// | [`InterruptMode::ActiveLow`]   |    `H`    |    `H`   |   `H`    |
    /// | [`InterruptMode::BothEdges`]   |    `H`    |    `L`   |   `X`    |
    ///
    /// `X` = "Don't care" so register unchanged when setting this mode.
    ///
    /// Because the MCP23S17 is solely concerned with raising the interrupt and not with
    /// handling it, the `InputPin` API just allows control of the relevant registers
    /// that affect the device's interrupt behaviour with handlers expected to be in
    /// some type that contains the [`Mcp23s17`][super::Mcp23s17].
    pub fn set_interrupt_mode(&mut self, mode: InterruptMode) -> Result<()> {
        let (gpinten, intcon, defval) = match self.pin.port {
            Port::GpioA => (
                RegisterAddress::GPINTENA,
                RegisterAddress::INTCONA,
                RegisterAddress::DEFVALA,
            ),
            Port::GpioB => (
                RegisterAddress::GPINTENB,
                RegisterAddress::INTCONB,
                RegisterAddress::DEFVALB,
            ),
        };

        // Set up the registers. Note that GPINTEN is set last so that the correct
        // criteria are set before enabling interrupts to avoid an spurious initial
        // interrupts.
        let mcp23s17_state = self.pin.mcp23s17_state.borrow();
        match mode {
            InterruptMode::None => {
                self.interrupts_enabled = false;
                mcp23s17_state.clear_bit(gpinten, self.pin.pin)?;
            }
            InterruptMode::ActiveHigh => {
                self.interrupts_enabled = true;
                mcp23s17_state.set_bit(intcon, self.pin.pin)?;
                mcp23s17_state.clear_bit(defval, self.pin.pin)?;
                mcp23s17_state.set_bit(gpinten, self.pin.pin)?;
            }
            InterruptMode::ActiveLow => {
                self.interrupts_enabled = true;
                mcp23s17_state.set_bit(intcon, self.pin.pin)?;
                mcp23s17_state.set_bit(defval, self.pin.pin)?;
                mcp23s17_state.set_bit(gpinten, self.pin.pin)?;
            }
            InterruptMode::BothEdges => {
                self.interrupts_enabled = true;
                mcp23s17_state.clear_bit(intcon, self.pin.pin)?;
                mcp23s17_state.set_bit(gpinten, self.pin.pin)?;
            }
        }
        Ok(())
    }

    impl_input!();
}

impl Drop for InputPin {
    fn drop(&mut self) {
        if self.interrupts_enabled {
            let _ = self.set_interrupt_mode(InterruptMode::None);
        }
    }
}

impl OutputPin {
    /// Constructs an `OutputPin` consuming the unconfigured `Pin` in the process.
    ///
    /// Sets the direction of the appropriate GPIO line and configuration of the Pull-up
    /// control register.
    fn new(pin: Pin) -> Result<Self> {
        // Set the direction of the GPIO port.
        // Need to scope to drop the reference to the MCP23S17 state before we move the
        // pin into the return value.
        {
            let mcp23s17_state = pin.mcp23s17_state.borrow();
            mcp23s17_state.clear_bit(
                if pin.port == Port::GpioA {
                    RegisterAddress::IODIRA
                } else {
                    RegisterAddress::IODIRB
                },
                pin.pin,
            )?;

            // Turn-off the pull-up.
            mcp23s17_state.clear_bit(
                if pin.port == Port::GpioA {
                    RegisterAddress::GPPUA
                } else {
                    RegisterAddress::GPPUB
                },
                pin.pin,
            )?;
        }
        Ok(OutputPin { pin })
    }

    /// Set the state of the pin.
    pub fn write(&self, level: Level) -> Result<()> {
        let gpio = match self.pin.port {
            Port::GpioA => RegisterAddress::GPIOA,
            Port::GpioB => RegisterAddress::GPIOB,
        };
        let mcp23s17_state = self.pin.mcp23s17_state.borrow();
        match level {
            Level::Low => mcp23s17_state.clear_bit(gpio, self.pin.pin),
            Level::High => mcp23s17_state.set_bit(gpio, self.pin.pin),
        }
    }

    /// Set the output to `Level::High`.
    pub fn set_high(&self) -> Result<()> {
        self.write(Level::High)
    }

    /// Set the output to `Level::Low`.
    pub fn set_low(&self) -> Result<()> {
        self.write(Level::Low)
    }

    // Reading from an OutputPin is valid.
    impl_input!();
}
