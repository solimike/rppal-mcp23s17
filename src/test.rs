use super::*;

#[test]
fn set_bits() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    mcp23s17.set_mock_data(RegisterAddress::GPIOA, 0x5a);

    mcp23s17
        .set_bits(RegisterAddress::GPIOA, 0xaa)
        .expect("Bad set bits");

    let gpioa = mcp23s17.get_mock_data(RegisterAddress::GPIOA);

    assert_eq!(gpioa, (0xfa, 1, 1), "Unexpected data");
}

#[test]
fn clear_bits() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    mcp23s17.set_mock_data(RegisterAddress::GPIOA, 0x5a);

    mcp23s17
        .clear_bits(RegisterAddress::GPIOA, 0xaa)
        .expect("Bad clear bits");

    let gpioa = mcp23s17.get_mock_data(RegisterAddress::GPIOA);

    assert_eq!(gpioa, (0x50, 1, 1), "Unexpected data");
}

#[test]
fn set_bit() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    mcp23s17.set_mock_data(RegisterAddress::GPIOA, 0x00);

    mcp23s17
        .set_bit(RegisterAddress::GPIOA, 5)
        .expect("Bad set bit");

    let gpioa = mcp23s17.get_mock_data(RegisterAddress::GPIOA);

    assert_eq!(gpioa, (0b0010_0000, 1, 1), "Unexpected data");
}

#[test]
fn clear_bit() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    mcp23s17.set_mock_data(RegisterAddress::GPIOA, 0xff);

    mcp23s17
        .clear_bit(RegisterAddress::GPIOA, 5)
        .expect("Bad clear bit");

    let gpioa = mcp23s17.get_mock_data(RegisterAddress::GPIOA);

    assert_eq!(gpioa, (0b1101_1111, 1, 1), "Unexpected data");
}

#[test]
fn get_bit() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    mcp23s17.set_mock_data(RegisterAddress::GPIOB, 0x55);

    assert_eq!(
        mcp23s17
            .get_bit(RegisterAddress::GPIOB, 0)
            .expect("Bad clear bit"),
        Level::High
    );
    assert_eq!(
        mcp23s17
            .get_bit(RegisterAddress::GPIOB, 1)
            .expect("Bad clear bit"),
        Level::Low
    );
}

#[test]
fn input_pin_disable_interrupts_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA, GPPUA, GPINTENA, INTCONA and DEFVALA that let us
        // observe the operation of the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPINTENA, 0b1111_1111);
    }

    let mut pin = mcp23s17
        .get(Port::GpioA, 7)
        .expect("Failed to get pin")
        .into_input_pin()
        .expect("Failed to convert to OutputPin");
    pin.set_interrupt_mode(pin::InterruptMode::None)
        .expect("Bad mode set");

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b1000_0000, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b0111_1111, 1, 1),
        "Bad GPPUA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPINTENA),
        (0b0111_1111, 1, 1),
        "Bad GPINTENA"
    );
}

#[test]
fn input_pin_enable_interrupts_rising_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA, GPPUA, GPINTENA, INTCONA and DEFVALA that let us
        // observe the operation of the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPINTENA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::INTCONA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::DEFVALA, 0b1111_1111);
    }

    let mut pin = mcp23s17
        .get(Port::GpioA, 7)
        .expect("Failed to get pin")
        .into_input_pin()
        .expect("Failed to convert to OutputPin");
    pin.set_interrupt_mode(pin::InterruptMode::ActiveHigh)
        .expect("Bad mode set");

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b1000_0000, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b0111_1111, 1, 1),
        "Bad GPPUA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPINTENA),
        (0b1000_0000, 1, 1),
        "Bad GPINTENA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::INTCONA),
        (0b1000_0000, 1, 1),
        "Bad INTCONA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::DEFVALA),
        (0b0111_1111, 1, 1),
        "Bad DEFVALA"
    );
}

#[test]
fn input_pin_enable_interrupts_falling_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA, GPPUA, GPINTENA, INTCONA and DEFVALA that let us
        // observe the operation of the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPINTENA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::INTCONA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::DEFVALA, 0b0000_0000);
    }

    let mut pin = mcp23s17
        .get(Port::GpioA, 7)
        .expect("Failed to get pin")
        .into_input_pin()
        .expect("Failed to convert to OutputPin");
    pin.set_interrupt_mode(pin::InterruptMode::ActiveLow)
        .expect("Bad mode set");

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b1000_0000, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b0111_1111, 1, 1),
        "Bad GPPUA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPINTENA),
        (0b1000_0000, 1, 1),
        "Bad GPINTENA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::INTCONA),
        (0b1000_0000, 1, 1),
        "Bad INTCONA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::DEFVALA),
        (0b1000_0000, 1, 1),
        "Bad DEFVALA"
    );
}

#[test]
fn input_pin_enable_interrupts_both_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA, GPPUA, GPINTENA, INTCONA and DEFVALA that let us
        // observe the operation of the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPINTENA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::INTCONA, 0b1111_1111);
    }

    let mut pin = mcp23s17
        .get(Port::GpioA, 7)
        .expect("Failed to get pin")
        .into_input_pin()
        .expect("Failed to convert to OutputPin");
    pin.set_interrupt_mode(pin::InterruptMode::BothEdges)
        .expect("Bad mode set");

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b1000_0000, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b0111_1111, 1, 1),
        "Bad GPPUA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPINTENA),
        (0b1000_0000, 1, 1),
        "Bad GPINTENA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::INTCONA),
        (0b0111_1111, 1, 1),
        "Bad INTCONA"
    );
}

#[test]
fn input_pin_enable_interrupts_rising_gpiob() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRB, GPPUB, GPINTENB, INTCONB and DEFVALB that let us
        // observe the operation of the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRB, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPPUB, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPINTENB, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::INTCONB, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::DEFVALB, 0b1111_1111);
    }

    let mut pin = mcp23s17
        .get(Port::GpioB, 7)
        .expect("Failed to get pin")
        .into_input_pin()
        .expect("Failed to convert to OutputPin");
    pin.set_interrupt_mode(pin::InterruptMode::ActiveHigh)
        .expect("Bad mode set");

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRB),
        (0b1000_0000, 1, 1),
        "Bad IODIRB"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUB),
        (0b0111_1111, 1, 1),
        "Bad GPPUB"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPINTENB),
        (0b1000_0000, 1, 1),
        "Bad GPINTENB"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::INTCONB),
        (0b1000_0000, 1, 1),
        "Bad INTCONB"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::DEFVALB),
        (0b0111_1111, 1, 1),
        "Bad DEFVALB"
    );
}

#[test]
fn write_output_pin_low_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0001_0000);
    }

    let pin = mcp23s17
        .get(Port::GpioA, 4)
        .expect("Failed to get pin")
        .into_output_pin()
        .expect("Failed to convert to OutputPin");
    pin.write(Level::Low).expect("Bad pin write");

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b1110_1111, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b1110_1111, 1, 1),
        "Bad GPPUA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPIOA),
        (0b0000_0000, 1, 1),
        "Bad GPIOA"
    );
}

#[test]
fn write_output_pin_high_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0000_0000);
    }

    let pin = mcp23s17
        .get(Port::GpioA, 4)
        .expect("Failed to get pin")
        .into_output_pin()
        .expect("Failed to convert to OutputPin");
    pin.write(Level::High).expect("Bad pin write");

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b1110_1111, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b1110_1111, 1, 1),
        "Bad GPPUA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPIOA),
        (0b0001_0000, 1, 1),
        "Bad GPIOA"
    );
}

#[test]
fn output_pin_low_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0001_0000);
    }

    let _pin = mcp23s17
        .get(Port::GpioA, 4)
        .expect("Failed to get pin")
        .into_output_pin_low()
        .expect("Failed to convert to OutputPinLow");

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b1110_1111, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b1110_1111, 1, 1),
        "Bad GPPUA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPIOA),
        (0b0000_0000, 1, 1),
        "Bad GPIOA"
    );
}

#[test]
fn output_pin_high_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0000_0000);
    }

    let _pin = mcp23s17
        .get(Port::GpioA, 4)
        .expect("Failed to get pin")
        .into_output_pin_high()
        .expect("Failed to convert to OutputPinHigh");

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b1110_1111, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b1110_1111, 1, 1),
        "Bad GPPUA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPIOA),
        (0b0001_0000, 1, 1),
        "Bad GPIOA"
    );
}

#[test]
fn write_output_pin_high_gpiob() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRB and GPPUB and that lets us observe the operation of
        // the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRB, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPPUB, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPIOB, 0b0000_0000);
    }

    let pin = mcp23s17
        .get(Port::GpioB, 4)
        .expect("Failed to get pin")
        .into_output_pin()
        .expect("Failed to convert to OutputPin");
    pin.write(Level::High).expect("Bad pin write");

    // Check we got the expected values written into IODIRB and GPPUB
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRB),
        (0b1110_1111, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUB),
        (0b1110_1111, 1, 1),
        "Bad GPPUA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPIOB),
        (0b0001_0000, 1, 1),
        "Bad GPIOB"
    );
}

#[test]
fn read_output_pin_low_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0000_0000);
    }

    let pin = mcp23s17
        .get(Port::GpioA, 4)
        .expect("Failed to get pin")
        .into_output_pin()
        .expect("Failed to convert to OutputPin");
    let pin_level = pin.read().expect("Bad pin read");
    assert_eq!(pin_level, Level::Low);

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b1110_1111, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b1110_1111, 1, 1),
        "Bad GPPUA"
    );
}

#[test]
fn read_output_pin_high_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the Pin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0001_0000);
    }

    let pin = mcp23s17
        .get(Port::GpioA, 4)
        .expect("Failed to get pin")
        .into_output_pin()
        .expect("Failed to convert to OutputPin");
    let pin_level = pin.read().expect("Bad pin read");
    assert_eq!(pin_level, Level::High)
}

#[test]
fn read_input_pin_low_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the InputPin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0000_0000);
    }

    let pin = mcp23s17
        .get(Port::GpioA, 0)
        .expect("Failed to get pin")
        .into_input_pin()
        .expect("Failed to convert to InputPin");
    let pin_level = pin.read().expect("Bad pin read");
    assert_eq!(pin_level, Level::Low);

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b0000_0001, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b1111_1110, 1, 1),
        "Bad GPPUA"
    );
}

#[test]
fn read_input_pin_high_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the InputPin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b1111_1111);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0000_0001);
    }

    let pin = mcp23s17
        .get(Port::GpioA, 0)
        .expect("Failed to get pin")
        .into_input_pin()
        .expect("Failed to convert to InputPin");
    let pin_level = pin.read().expect("Bad pin read");
    assert_eq!(pin_level, Level::High)
}

#[test]
fn read_pullup_input_pin_low_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the InputPin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0000_0000);
    }

    let pin = mcp23s17
        .get(Port::GpioA, 0)
        .expect("Failed to get pin")
        .into_pullup_input_pin()
        .expect("Failed to convert to InputPin");
    let pin_level = pin.read().expect("Bad pin read");
    assert_eq!(pin_level, Level::Low);

    // Check we got the expected values written into IODIRA and GPPUA
    let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::IODIRA),
        (0b0000_0001, 1, 1),
        "Bad IODIRA"
    );
    assert_eq!(
        mock_spi.get_mock_data(RegisterAddress::GPPUA),
        (0b0000_0001, 1, 1),
        "Bad GPPUA"
    );
}

#[test]
fn read_pullup_input_pin_high_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    {
        // Put data into IODIRA and GPPUA and that lets us observe the operation of
        // the InputPin.
        let mock_spi = &mcp23s17.mcp23s17_state.borrow().spi;
        mock_spi.set_mock_data(RegisterAddress::IODIRA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPPUA, 0b0000_0000);
        mock_spi.set_mock_data(RegisterAddress::GPIOA, 0b0000_0001);
    }
    let pin = mcp23s17
        .get(Port::GpioA, 0)
        .expect("Failed to get pin")
        .into_pullup_input_pin()
        .expect("Failed to convert to InputPin");
    let pin_level = pin.read().expect("Bad pin read");
    assert_eq!(pin_level, Level::High)
}

#[test]
fn read_pin_low_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    mcp23s17.set_mock_data(RegisterAddress::GPIOA, 0b0000_0000);
    let pin = mcp23s17.get(Port::GpioA, 0).expect("Failed to get pin");
    let pin_level = pin.read().expect("Bad pin read");
    assert_eq!(pin_level, Level::Low)
}

#[test]
fn read_pin_high_gpioa() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    mcp23s17.set_mock_data(RegisterAddress::GPIOA, 0b0000_0001);
    let pin = mcp23s17.get(Port::GpioA, 0).expect("Failed to get pin");
    let pin_level = pin.read().expect("Bad pin read");
    assert_eq!(pin_level, Level::High)
}

#[test]
fn read_pin_high_gpiob() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    mcp23s17.set_mock_data(RegisterAddress::GPIOB, 0b0000_1000);
    let pin = mcp23s17.get(Port::GpioB, 3).expect("Failed to get pin");
    let pin_level = pin.read().expect("Bad pin read");
    assert_eq!(pin_level, Level::High)
}

#[test]
fn create_unique_pin() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    let _pin = mcp23s17.get(Port::GpioA, 0).expect("Failed to get pin");
}

#[test]
fn create_out_of_range_pin() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    let pin = mcp23s17.get(Port::GpioA, 9);

    match pin {
        Err(Mcp23s17Error::PinNotAvailable(9)) => (),
        _ => panic!("Unexpected return result: {pin:?}"),
    }
}

#[test]
fn create_duplicate_pin() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    let _pin1 = mcp23s17
        .get(Port::GpioA, 0)
        .expect("Failed to get first (unique) pin");

    let duplicate_pin = mcp23s17.get(Port::GpioA, 0);
    match duplicate_pin {
        Err(Mcp23s17Error::PinNotAvailable(0)) => (),
        _ => {
            panic!("Unexpected return result - duplicate should be unavailable: {duplicate_pin:?}")
        }
    }
}

#[test]
fn create_duplicate_pins_separate_ports() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    let _pin1 = mcp23s17
        .get(Port::GpioA, 0)
        .expect("Failed to get first (unique) pin");
    let _pin2 = mcp23s17
        .get(Port::GpioB, 0)
        .expect("Failed to get second (unique) pin");
}

#[test]
fn read_iocon() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    mcp23s17.set_mock_data(RegisterAddress::IOCON, 0x64);

    let iocon = IOCON::from_bits(mcp23s17.read(RegisterAddress::IOCON).expect("Bad read"))
        .expect("Invalid flags");

    assert_eq!(
        iocon,
        IOCON::MIRROR | IOCON::SEQOP | IOCON::ODR,
        "Unexpected flags"
    );
}

#[test]
fn write_iocon() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");

    mcp23s17
        .write(RegisterAddress::IOCON, 0x64)
        .expect("Bad write");

    let (iocon, reads, writes) = mcp23s17.get_mock_data(RegisterAddress::IOCON);

    assert_eq!(
        IOCON::from_bits(iocon).expect("Invalid flags"),
        IOCON::MIRROR | IOCON::SEQOP | IOCON::ODR,
        "Unexpected flags"
    );
    assert_eq!(reads, 0);
    assert_eq!(writes, 1);
}

#[test]
fn iocon() {
    let iocon_mode = IOCON::BANK_OFF
        | IOCON::MIRROR_OFF
        | IOCON::SEQOP_OFF
        | IOCON::DISSLW_SLEW_RATE_CONTROLLED
        | IOCON::HAEN_ON
        | IOCON::ODR_OFF
        | IOCON::INTPOL_LOW;
    let iocon = iocon_mode.bits();
    assert_eq!(iocon, 0b00101000, "Unexpected flags 0b{iocon:08b}");
}

#[test]
fn spi_control_read() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");

    let spi_ctrl = mcp23s17
        .mcp23s17_state
        .borrow()
        .spi_control_byte(SpiCommand::Read);
    assert_eq!(0x41, spi_ctrl, "Unexpected control byte: 0x{spi_ctrl:02x}");
}

#[test]
fn spi_control_write() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(0).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    let spi_ctrl = mcp23s17
        .mcp23s17_state
        .borrow()
        .spi_control_byte(SpiCommand::Write);
    assert_eq!(0x40, spi_ctrl, "Unexpected control byte: 0x{spi_ctrl:02x}");
}

#[test]
fn spi_control_address_read() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(1).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    let spi_ctrl = mcp23s17
        .mcp23s17_state
        .borrow()
        .spi_control_byte(SpiCommand::Read);
    assert_eq!(0x43, spi_ctrl, "Unexpected control byte: 0x{spi_ctrl:02x}");
}

#[test]
fn spi_control_address_write() {
    let mcp23s17 = Mcp23s17::new(
        HardwareAddress::new(1).unwrap(),
        SpiBus::Spi0,
        ChipSelect::Cs0,
        100_000,
        SpiMode::Mode0,
    )
    .expect("Create MCP23S17");
    let spi_ctrl = mcp23s17
        .mcp23s17_state
        .borrow()
        .spi_control_byte(SpiCommand::Write);
    assert_eq!(0x42, spi_ctrl, "Unexpected control byte: 0x{spi_ctrl:02x}");
}

#[test]
fn good_hardware_address() {
    let addr = HardwareAddress::new(7).expect("Bad address");
    assert_eq!(7u8, addr.into(), "Unexpected address value");
}

#[test]
fn bad_hardware_address() {
    let addr = HardwareAddress::new(8);
    match addr {
        Err(Mcp23s17Error::HardwareAddressBoundsError(8)) => (),
        _ => panic!("Unexpected return value: {addr:?}"),
    }
}

#[test]
fn try_into_good_hardware_address() {
    let addr: HardwareAddress = 7u8.try_into().expect("Bad address");
    assert_eq!(7u8, addr.into(), "Unexpected address value");
}

#[test]
fn try_into_bad_hardware_address() {
    let addr: Result<HardwareAddress> = 8u8.try_into();
    match addr {
        Err(Mcp23s17Error::HardwareAddressBoundsError(8)) => (),
        _ => panic!("Unexpected return value: {addr:?}"),
    }
}

#[test]
fn good_register_address() {
    let addr: u8 = RegisterAddress::IOCON.into();
    assert_eq!(10, addr, "Unexpected address value");
}

#[test]
fn try_into_good_register_address() {
    let addr: RegisterAddress = 7.try_into().expect("Bad address");
    assert_eq!(RegisterAddress::DEFVALB, addr, "Unexpected address value");
}

#[test]
fn try_into_bad_register_address() {
    let addr: Result<RegisterAddress> = 28.try_into();
    match addr {
        Err(Mcp23s17Error::RegisterAddressBoundsError) => (),
        _ => panic!("Unexpected return value: {addr:?}"),
    }
}

#[test]
fn from_chip_select() {
    let ss: SlaveSelect = ChipSelect::Cs14.into();
    assert_eq!(ss, SlaveSelect::Ss14);
}

#[test]
fn from_slave_select() {
    let cs: ChipSelect = SlaveSelect::Ss8.into();
    assert_eq!(cs, ChipSelect::Cs8);
}

#[test]
fn display_spi_bus() {
    let s = format!("{}", SpiBus::Spi3);
    assert_eq!(s, "Spi3");
}
