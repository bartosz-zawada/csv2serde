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

impl From<&CLI> for ReaderSource {
    fn from(cli: &CLI) -> Self {
        if let Some(ref path) = cli.file {
            let file = File::open(path).expect("Should be able to read the input file.");
            ReaderSource::File(file)
        } else {
            ReaderSource::Stdin
        }
    }
}
