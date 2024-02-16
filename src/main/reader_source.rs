use std::{fs::File, io};

use crate::CLI;

pub enum ReaderSource {
    File(File),
    Stdin,
}

impl io::Read for ReaderSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // No need to buffer manually; csv::Reader buffers for us.
        match self {
            ReaderSource::Stdin => io::stdin().read(buf),
            ReaderSource::File(f) => f.read(buf),
        }
    }
}

impl TryFrom<&CLI> for ReaderSource {
    type Error = io::Error;

    fn try_from(cli: &CLI) -> Result<Self, Self::Error> {
        if let Some(ref path) = cli.file {
            let file = File::open(path)?;
            Ok(ReaderSource::File(file))
        } else {
            Ok(ReaderSource::Stdin)
        }
    }
}
