use std::{fs::File, io};

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
