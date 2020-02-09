#![allow(unused_imports)]
use gnuplot::{Figure, PlotOption};
use hpgl::hp7470a::{DeviceControlInstruction, HandshakeConfig, HandshakeMode};
use hpgl::{Coordinate, HpglCommand, HpglProgram, PlotterWriteable};
use serialport::{self, DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
use std::time::Duration;
use lsystem::ParametricLSystem;
use maplit::hashmap;
use std::collections::HashMap;

const PI: f64 = 3.14159;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Symbol {
    A,
    B,
    Left,
    Right,
}

fn gen_line(lsystem: &ParametricLSystem<Symbol>) -> Vec<(f64, f64)> {
    let mut line: Vec<(f64, f64)> = vec![(0., 0.)];
    let mut direction: f64 = 0.; // radians
    let dt = 1.;
    for symbol in &lsystem.state {
        match symbol {
            Symbol::A | Symbol::B => {
                let mut pt = line.last().unwrap().clone();
                pt.0 += dt * direction.cos();
                pt.1 += dt * direction.sin();
                line.push(pt);
            }
            Symbol::Left => {
                direction -= PI / 3.;
            }
            Symbol::Right => {
                direction += PI / 3.;
            }
        }
    }

    let scale = 1. / (line.last().unwrap().0);
    line.iter().map(|(x, y)| (x * scale, y * scale)).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut s = {
        use Symbol::*;
        ParametricLSystem::new(
            vec![A],
            hashmap! {
                A => vec![B, Left, A, Left, B],
                B => vec![A, Right, B, Right, A],
            },
        )
    };

    let mut program = vec![
        HpglCommand::InitializePlotter,
        HpglCommand::SelectPen { pen: 1 },
    ];

    let mut lines_by_iter: HashMap<i32, Vec<(f64, f64)>> = HashMap::new();
    lines_by_iter.insert(0, gen_line(&s));

    const ITERS: i32 = 9;
    for i in 1..ITERS {
        s.state = s.evolve();

        // lines_by_iter.insert(i, gen_line(&s));
    }

    const DOTS_PER_UNIT: f64 = 7650.; //2000.0;
    // let points = lines_by_iter.get(&6).unwrap();
    let points = gen_line(&s);
    program.push(HpglCommand::PlotAbsolute(
        Coordinate {
            x: (1000. + points[0].0 * DOTS_PER_UNIT) as f32,
            y: (1000. + points[0].1 * DOTS_PER_UNIT) as f32,
        }
        .into(),
    ));
    program.push(HpglCommand::PenDown);
    // todo: coordinatechain
    for (x, y) in &points {
        program.push(HpglCommand::PlotAbsolute(
            Coordinate {
                x: (1000. + x * DOTS_PER_UNIT) as f32,
                y: (1000. + y * DOTS_PER_UNIT) as f32,
            }
            .into(),
        ));
    }
    program.push(HpglCommand::PenUp);

    let mut fg = Figure::new();
    let mut ax = fg.axes2d();
    ax.lines(
        points.iter().map(|pt| pt.0),
        points.iter().map(|pt| pt.1),
        &[],
    );
    fg.show().unwrap();

    println!("{:#?}", program);
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
    let program: HpglProgram = program.into();
    program.write(&mut sp)?;
    Ok(())
}
