#![allow(unused_imports, non_snake_case)]
use hpgl::hp7470a::{DeviceControlInstruction, HandshakeConfig, HandshakeMode};
use hpgl::{Coordinate, HpglCommand, HpglProgram, PlotterWriteable};
use serialport::{self, DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use core::f64::consts::PI;
use ndarray::prelude::*;
use gnuplot::{Figure, PlotOption};

fn gen_program() -> HpglProgram {
    let mut program = vec![
        HpglCommand::InitializePlotter,
        HpglCommand::SelectPen { pen: 1 },
    ];

    let K: f64       = 20.;
    let t_max: f64   = 4. * 2.*PI;
    let t_grains     = 1000;
    let omega0: f64  = 1.;
    let delta_k: f64 = 2.; // bump per k
    let margin: f64  = 500.;
    
    let x_ptsize = Coordinate::MAX_X_US as f64 - 2.*margin;
    let y_ptsize = Coordinate::MAX_Y    as f64 - 2.*margin;

    let tt = Array::linspace(0f64, t_max, t_grains);
    let kk = Array::range(1f64, K, 1.);
    let mut sintab = Array::<f64, _>::zeros((tt.len(), kk.len()));

    for (i, t) in tt.iter().enumerate() {
        for (j, k) in kk.iter().enumerate() {
            sintab[[i, j]] = 4./PI * (omega0 * t * (2.*k - 1.)).sin()/(2.*k - 1.);
        }
    }

    let partials = sintab.gencolumns().into_iter().scan(Array::zeros(tt.len()), |running, sinus| {
        *running += &sinus;

        Some(running.to_owned())
    });

    
    // let mut fg = Figure::new();
    // let mut ax = fg.axes2d();
    // for (k, partial) in partials.enumerate() {
    //     ax.lines(
    //         &tt,
    //         &(partial + k as f64*2.),
    //         &[],
    //     );
    // }
    // fg.show().unwrap();


    // figure out y scaling
    // maximum point we hit is delta_k * K  + 1
    let max_unscaled_y = delta_k * K;
    let y_scaling = y_ptsize / max_unscaled_y;

    // figure out x scaling
    // fit max(tt) to MAX_X_US
    let x_scaling = x_ptsize / t_max;

    for (k, partial) in partials.enumerate() {
        let partial = &partial + k as f64 * delta_k;
        program.push(HpglCommand::PlotAbsolute(Coordinate {x: (margin + tt[0] * x_scaling) as f32, y: (margin + partial[0] * y_scaling) as f32 }.into()));
        program.push(HpglCommand::PenDown);
        program.push(HpglCommand::PlotAbsolute(tt.into_iter().zip(partial.into_iter()).map(|(t, x)| {
            Coordinate {
                x: (margin + t * x_scaling) as f32,
                y: (margin + x * y_scaling) as f32,
            }
        }).collect::<Vec<_>>().into()));
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
    println!("{:#?}", program);
    program.write(&mut sp)?;

    Ok(())
}
