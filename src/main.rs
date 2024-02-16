mod main {
    pub mod reader_source;
}

use clap::{builder::ArgPredicate, Parser};
use convert_case::{Case, Casing};
use csv::{self, Trim};
use csv2serde::Config;
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use main::reader_source::ReaderSource;

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct CLI {
    /// File for which types will be generated.
    /// If not provided, output will be sent to stdout.
    file: Option<PathBuf>,

    /// Name of the type, defaults to filename.
    #[arg(short = 'n', long)]
    name: Option<String>,

    /// File into which the types will be written.
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// If writing into a file, overwrite content rather than error out.
    #[arg(short = 'f', long, requires_if(ArgPredicate::IsPresent, "output"))]
    force: bool,

    /// Character or string used as delimiter.
    #[arg(short = 'd', long, default_value_t = ',')]
    delimiter: char,

    /// Number of rows to analyze for field type prediction.
    #[arg(short = 'l', long)]
    lines: Option<usize>,

    /// Skips lines with a number of fields less or equal to this number.
    /// Useful when you want to omit subsection headers.
    #[arg(short = 's', long)]
    min_fields: Option<usize>,

    /// Add blank lines between struct fields.
    #[arg(short = 'b', long, default_value = "1")]
    blank_lines: Option<usize>,
}

enum WriteDestination {
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

fn get_name_from_path<P: AsRef<Path>>(path: P) -> String {
    let stem = path.as_ref().file_stem();
    let stem = stem.unwrap_or_else(|| {
        panic!(
            "Could not parse name from path '{}'",
            path.as_ref().display(),
        );
    });

    stem.to_string_lossy().to_string()
}

fn main() {
    let cli = CLI::parse();

    let struct_name = match (&cli.name, &cli.file) {
        (Some(name), _) => name.to_case(Case::Pascal),
        (None, Some(path)) => get_name_from_path(path).to_case(Case::Pascal),
        _ => unreachable!("Name should be required when no path provided."),
    };

    let reader = if let Some(ref path) = cli.file {
        let file = File::open(path).expect("Should be able to read the input file.");
        ReaderSource::File(file)
    } else {
        ReaderSource::Stdin
    };

    let reader = csv::ReaderBuilder::new()
        .delimiter(cli.delimiter as u8)
        .trim(Trim::All)
        .from_reader(reader);

    let config = Config {
        lines: cli.lines.unwrap_or(usize::MAX),
        min_fields: cli.min_fields,
        struct_name,
        blank_lines: cli.blank_lines,
    };

    let code = csv2serde::run(reader, &config).unwrap();

    let mut output = WriteDestination::from(cli);
    output.write_all(code.as_bytes()).unwrap();
    output.flush().unwrap();
}
