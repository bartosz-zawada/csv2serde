use std::{fs::File, io};

use crate::CLI;

pub enum WriteDestination {
    File(File),
    Stdout,
}

impl io::Write for WriteDestination {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            WriteDestination::File(f) => f.write(buf),
            WriteDestination::Stdout => io::stdout().write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            WriteDestination::File(f) => f.flush(),
            WriteDestination::Stdout => io::stdout().flush(),
        }
    }
}

impl TryFrom<&CLI> for WriteDestination {
    type Error = io::Error;

    fn try_from(cli: &CLI) -> Result<Self, Self::Error> {
        let output = cli.output.as_ref();
        match output.as_ref() {
            None => Ok(WriteDestination::Stdout),

            Some(path) => {
                let f = File::options()
                    .read(false)
                    .write(true)
                    .create_new(!cli.force)
                    .truncate(true)
                    .open(path)?;

                Ok(WriteDestination::File(f))
            }
        }
    }
}
