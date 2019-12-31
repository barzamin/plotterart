use std::io;

pub mod hp7470a;

pub trait PlotterWriteable {
    fn write<W>(&self, sink: &mut W) -> io::Result<()>
    where
        W: io::Write;
}

#[derive(Debug)]
pub struct HpglProgram(Vec<HpglCommand>);
impl HpglProgram {
    pub fn new(commands: Vec<HpglCommand>) -> Self {
        Self(commands)
    }
}

impl PlotterWriteable for HpglProgram {
    fn write<W>(&self, sink: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.0
            .iter()
            .map(|command| command.write(sink))
            .collect::<_>()
    }
}

impl From<Vec<HpglCommand>> for HpglProgram {
    fn from(inner: Vec<HpglCommand>) -> Self {
        Self(inner)
    }
}

/// Raw coordinate (can represent either absolute or relative, non-/plotter).
///
/// When plotting in _plotter coordinates_, x ∈ [0, 10900], y ∈ [0, 7650]
#[derive(Clone, Copy, Debug)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}
impl Coordinate {
    pub const MAX_X: f32 = 10900.;
    pub const MAX_Y: f32 = 7650.;
}

#[derive(Debug)]
pub struct CoordinateChain(pub Vec<Coordinate>);

impl CoordinateChain {
    pub fn write<W>(&self, sink: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        let mut iter = self.0.iter().peekable();
        while let Some(coord) = iter.next() {
            write!(sink, "{},{}", coord.x, coord.y)?;
            if let Some(_) = iter.peek() {
                write!(sink, ",")?;
            }
        }

        Ok(())
    }
}

impl From<Vec<Coordinate>> for CoordinateChain {
    fn from(inner: Vec<Coordinate>) -> Self {
        Self(inner)
    }
}

#[derive(Debug)]
pub enum HpglCommand {
    DefaultSettings,
    InitializePlotter,
    SelectPen {
        pen: usize,
    },
    VelocitySelect {
        velocity: f32,
    },
    /// Raises the pen. _Note_: **Deliberately** does not support moving the pen as part of the same command.
    PenUp,
    /// Lowers the pen. _Note_: **Deliberately** does not support moving the pen as part of the same command.
    PenDown,
    PlotAbsolute(CoordinateChain),
}

impl PlotterWriteable for HpglCommand {
    fn write<W>(&self, sink: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        use HpglCommand::*;
        match self {
            DefaultSettings => {
                sink.write(b"DF;")?;
            }
            InitializePlotter => {
                sink.write(b"IN;")?;
            }
            SelectPen { pen } => {
                sink.write(b"IN")?;
                write!(sink, "{}", pen)?;
                sink.write(b";")?;
            }
            VelocitySelect { velocity } => {
                sink.write(b"VS")?;
                write!(sink, "{}", velocity)?;
                sink.write(b";")?;
            }
            PenUp => {
                sink.write(b"PU;")?;
            }
            PenDown => {
                sink.write(b"PD;")?;
            }
            PlotAbsolute(coord) => {
                sink.write(b"PA")?;
                coord.write(sink)?;
                sink.write(b";")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_point_in_chain() {
        let chain: CoordinateChain = vec![Coordinate { x: 69., y: 420. }].into();
        let mut buf: Vec<u8> = Vec::new();

        chain.write(&mut buf).unwrap();
        assert_eq!(buf, b"69,420");
    }

    #[test]
    fn multipoint_in_chain() {
        let chain: CoordinateChain = vec![
            Coordinate { x: 69., y: 420. },
            Coordinate { x: 666., y: 69. },
        ]
        .into();
        let mut buf: Vec<u8> = Vec::new();

        chain.write(&mut buf).unwrap();
        assert_eq!(buf, b"69,420,666,69");
    }
}
