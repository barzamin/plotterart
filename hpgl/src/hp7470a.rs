use bitflags::bitflags;
use std::io;

use crate::PlotterWriteable;

bitflags! {
    #[derive(Default)]
    pub struct PlotterConfig: u8 {
        const HW_HANDSHAKE = 1 << 0;
        /// monitor mode 1 if set, monitor mode 0 if cleared
        const MON_MODE_CTL = 1 << 2;
        const MON_MODE_ENA = 1 << 3;
    }
}

pub enum HandshakeMode {
    Mode1,
    Mode2,
}
pub enum HandshakeConfig {
    EnqAck {
        block_size: u8,
        enq_char: u8,
        ack_string: Vec<u8>,
    },
    XonXoff {
        xoff_threshold: u8,
        xon_trigger_chars: Vec<u8>,
    },
}

pub enum DeviceControlInstruction {
    SetPlotterConfig(PlotterConfig),
    SetHandshakeMode(HandshakeMode, HandshakeConfig),
    SetExtHandshakeOptions {
        interchar_delay: Option<u16>,
        xoff_trigger_chars: Vec<u8>,
    },
}

impl PlotterWriteable for DeviceControlInstruction {
    fn write<W>(&self, w: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        use DeviceControlInstruction::*;
        match self {
            SetPlotterConfig(config) => {
                write!(w, "\x1b.@;{}:", config.bits)?;
            }
            SetHandshakeMode(mode, config) => {
                w.write(b"\x1b.")?;
                match mode {
                    HandshakeMode::Mode1 => {
                        w.write(b"H")?;
                    }
                    HandshakeMode::Mode2 => {
                        w.write(b"I")?;
                    }
                }

                match config {
                    HandshakeConfig::EnqAck {
                        block_size,
                        enq_char,
                        ack_string,
                    } => {
                        write!(w, "{};{};", block_size, enq_char)?;
                        w.write(
                            ack_string
                                .iter()
                                .map(|c| c.to_string())
                                .collect::<Vec<String>>()
                                .join(";")
                                .as_bytes(),
                        )?;
                    }
                    HandshakeConfig::XonXoff {
                        xoff_threshold,
                        xon_trigger_chars,
                    } => {
                        write!(w, "{};;", xoff_threshold)?;
                        w.write(
                            xon_trigger_chars
                                .iter()
                                .map(|c| c.to_string())
                                .collect::<Vec<String>>()
                                .join(";")
                                .as_bytes(),
                        )?;
                    }
                }

                w.write(b":")?;
            }
            SetExtHandshakeOptions {
                interchar_delay,
                xoff_trigger_chars,
            } => {
                write!(w, "\x1b.N")?;
                if let Some(interchar_delay) = interchar_delay {
                    write!(w, "{}", interchar_delay)?;
                }
                w.write(b";")?;
                w.write(
                    xoff_trigger_chars
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<String>>()
                        .join(";")
                        .as_bytes(),
                )?;

                w.write(b":")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn config_hardshaking_monmode1() {
        let mut buf: Vec<u8> = Vec::new();
        let instruction = DeviceControlInstruction::SetPlotterConfig(
            PlotterConfig::HW_HANDSHAKE | PlotterConfig::MON_MODE_CTL | PlotterConfig::MON_MODE_ENA,
        );
        instruction.write(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b.@;13:");
    }

    #[test]
    fn config_softshaking_nomon() {
        let mut buf: Vec<u8> = Vec::new();
        let instruction = DeviceControlInstruction::SetPlotterConfig(Default::default());
        instruction.write(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b.@;0:");
    }

    #[test]
    fn mode2_xon_handshaking() {
        let mut buf: Vec<u8> = Vec::new();
        let instruction = DeviceControlInstruction::SetHandshakeMode(
            HandshakeMode::Mode2,
            HandshakeConfig::XonXoff {
                xoff_threshold: 81,
                xon_trigger_chars: b"\x11".to_vec(), // DC1
            },
        );
        instruction.write(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b.I81;;17:");
    }

    #[test]
    fn mode1_enqack_handshaking() {
        let mut buf: Vec<u8> = Vec::new();
        let instruction = DeviceControlInstruction::SetHandshakeMode(
            HandshakeMode::Mode1,
            HandshakeConfig::EnqAck {
                block_size: 132,
                enq_char: 0x13,
                ack_string: [0x14, 0x07].to_vec(),
            },
        );
        instruction.write(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b.H132;19;20;7:");
    }

    #[test]
    fn xoff_trigger_char() {
        let mut buf: Vec<u8> = Vec::new();
        let instruction = DeviceControlInstruction::SetExtHandshakeOptions {
            interchar_delay: None,
            xoff_trigger_chars: b"\x13".to_vec(),
        };
        instruction.write(&mut buf).unwrap();

        assert_eq!(buf, b"\x1b.N;19:");
    }
}
