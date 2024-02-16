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

impl From<CLI> for WriteDestination {
    fn from(cli: CLI) -> Self {
        let output = cli.output.as_ref();
        match output.as_ref() {
            None => WriteDestination::Stdout,

            Some(path) => {
                let f = File::options()
                    .read(false)
                    .write(true)
                    .create_new(!cli.force)
                    .truncate(true)
                    .open(path)
                    .expect("Should be able to write file");

                WriteDestination::File(f)
            }
        }
    }
}
