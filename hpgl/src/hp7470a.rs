use std::io;
use bitflags::bitflags;

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

pub enum DeviceControlInstruction {
    SetPlotterConfig(PlotterConfig),
}

impl PlotterWriteable for DeviceControlInstruction {
    fn write<W>(&self, w: &mut W) -> io::Result<()> where W: io::Write {
        use DeviceControlInstruction::*;
        match self {
            SetPlotterConfig(config) => {
                write!(w, "\x03.@;{}:", config.bits)?;
            },
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
              PlotterConfig::HW_HANDSHAKE
            | PlotterConfig::MON_MODE_CTL
            | PlotterConfig::MON_MODE_ENA );
        instruction.write(&mut buf).unwrap();

        assert_eq!(buf, b"\x03.@;13:");
    }

    #[test]
    fn config_softshaking_nomon() {
        let mut buf: Vec<u8> = Vec::new();
        let instruction = DeviceControlInstruction::SetPlotterConfig(Default::default());
        instruction.write(&mut buf).unwrap();

        assert_eq!(buf, b"\x03.@;0:");
    }
}