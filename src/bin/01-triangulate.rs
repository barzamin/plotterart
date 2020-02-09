use hpgl::hp7470a::{DeviceControlInstruction, HandshakeConfig, HandshakeMode};
use hpgl::{Coordinate, HpglCommand, HpglProgram, PlotterWriteable};
use serialport::{self, DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use delaunator::{triangulate, Point};
use rand::prelude::*;

fn gen_points(n: usize) -> Vec<Point> {
    let mut rng = rand::thread_rng();

    let margin = 500.;

    (0..n)
        .map(|_| {
            let x = rng.gen_range(0. + margin, Coordinate::MAX_X_US as f64 - margin);
            let y = rng.gen_range(0. + margin, Coordinate::MAX_Y as f64 - margin);
            Point { x, y }
        })
        .collect()
}

fn gen_program() -> HpglProgram {
    let mut program = vec![
        HpglCommand::InitializePlotter,
        HpglCommand::SelectPen { pen: 1 },
    ];

    let points = gen_points(100);
    let triangulation = triangulate(&points).expect("no triangulation found");

    // naive asf
    for triangle in triangulation.triangles.chunks(3) {
        program.push(HpglCommand::PlotAbsolute(
            Coordinate {
                x: points[triangle[0]].x as f32,
                y: points[triangle[0]].y as f32,
            }
            .into(),
        ));
        program.push(HpglCommand::PenDown);
        program.push(HpglCommand::PlotAbsolute(
            vec![
                Coordinate {
                    x: points[triangle[1]].x as f32,
                    y: points[triangle[1]].y as f32,
                },
                Coordinate {
                    x: points[triangle[2]].x as f32,
                    y: points[triangle[2]].y as f32,
                },
                Coordinate {
                    x: points[triangle[0]].x as f32,
                    y: points[triangle[0]].y as f32,
                },
            ]
            .into(),
        ));
        program.push(HpglCommand::PenUp);
    }

    program.into()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sp = serialport::open_with_settings(
        "/dev/ttyUSB0",
        &SerialPortSettings {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::Software,
            timeout: Duration::from_secs(20),
        },
    )
    .expect("can't open serial port");

    // let mut sp = io::stdout();

    // -- configure handshaking
    DeviceControlInstruction::SetPlotterConfig(Default::default()).write(&mut sp)?;
    DeviceControlInstruction::SetHandshakeMode(
        HandshakeMode::Mode2,
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

    let program = gen_program();
    // let program: HpglProgram = vec![
    //     HpglCommand::InitializePlotter,
    //     HpglCommand::PlotAbsolute(
    //         vec![Coordinate {
    //             x: Coordinate::MAX_X / 2.,
    //             y: Coordinate::MAX_Y / 2.,
    //         }]
    //         .into(),
    //     ),
    // ]
    // .into();
    println!("{:#?}", program);
    program.write(&mut sp)?;

    Ok(())
}
