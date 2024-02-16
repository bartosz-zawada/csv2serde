use clap::{builder::ArgPredicate, Parser};
use convert_case::{Case, Casing};
use csv::{self, Trim};
use csv2serde::Config;
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
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

enum ReaderSource {
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
    let args = Args::parse();

    let (reader, struct_name) = if let Some(path) = args.file {
        let file = File::open(&path).expect("Should be able to read the input file.");
        let struct_name = args.name.unwrap_or_else(|| get_name_from_path(path));

        (ReaderSource::File(file), struct_name)
    } else {
        let struct_name = args.name.expect("Name required when reading from STDIN.");

        (ReaderSource::Stdin, struct_name)
    };

    let struct_name = struct_name.to_case(Case::Pascal);

    let reader = csv::ReaderBuilder::new()
        .delimiter(args.delimiter as u8)
        .trim(Trim::All)
        .from_reader(reader);

    let config = Config {
        lines: args.lines.unwrap_or(usize::MAX),
        min_fields: args.min_fields,
        struct_name,
        blank_lines: args.blank_lines,
    };

    let code = csv2serde::run(reader, &config).unwrap();

    if let Some(path) = args.output {
        File::options()
            .read(false)
            .write(true)
            .create_new(!args.force)
            .open(path)
            .unwrap()
            .write_all(code.as_bytes())
            .unwrap();
    } else {
        println!("{}", code);
    };
}
