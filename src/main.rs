use hpgl::hp7470a::{DeviceControlInstruction, HandshakeConfig, HandshakeMode};
use hpgl::{Coordinate, HpglCommand, HpglProgram, PlotterWriteable};
use serialport::{self, DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
use std::io::{self, Write};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sp = serialport::open_with_settings(
        "/dev/ttyUSB0",
        &SerialPortSettings {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::Odd,
            flow_control: FlowControl::Software,
            timeout: Duration::from_secs(1),
        },
    )
    .expect("can't open serial port");

    // let mut sp = io::stdout();

    // -- configure handshaking
    DeviceControlInstruction::SetPlotterConfig(Default::default()).write(&mut sp)?;
    DeviceControlInstruction::SetHandshakeMode(
        HandshakeMode::Mode1,
        HandshakeConfig::XonXoff {
            xoff_threshold: 80,
            xon_trigger_chars: b"\x11".to_vec(),
        },
    )
    .write(&mut sp)?;
    DeviceControlInstruction::SetExtHandshakeOptions {
        interchar_delay: None,
        xoff_trigger_chars: b"\x13".to_vec(),
    }
    .write(&mut sp)?;

    let program: HpglProgram = vec![
        HpglCommand::InitializePlotter,
        HpglCommand::PlotAbsolute(
            vec![Coordinate {
                x: Coordinate::MAX_X / 2.,
                y: Coordinate::MAX_Y / 2.,
            }]
            .into(),
        ),
    ]
    .into();
    program.write(&mut sp)?;

    println!("bytes to read: {:?}", sp.bytes_to_read());

    Ok(())
}
