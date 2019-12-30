use std::time::Duration;
use std::io::{self, Write};
use serialport::{self, SerialPortSettings, Parity, DataBits, StopBits, FlowControl};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sp = serialport::open_with_settings("/dev/ttyUSB0",
            &SerialPortSettings {
                baud_rate: 9600,
                data_bits: DataBits::Eight,
                stop_bits: StopBits::One,
                parity: Parity::Odd,
                flow_control: FlowControl::Software,
                timeout: Duration::from_secs(1),
            })
        .expect("can't open serial port");

    Ok(())
}