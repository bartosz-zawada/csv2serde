use clap::{builder::ArgPredicate, Parser};
use convert_case::{self, Case, Casing};
use csv::{self, Trim};
use csv2serde::Config;
use std::{fs::File, io::Write, path::PathBuf};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Args {
    /// File for which types will be generated.
    /// If not provided, output will be sent to stdout.
    file: PathBuf,

    /// Name of the type, defaults to filename.
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
}

fn main() {
    let args = Args::parse();

    let path = args.file;

    let reader = csv::ReaderBuilder::new()
        .delimiter(args.delimiter as u8)
        .trim(Trim::All)
        .from_path(&path)
        .expect("Could not open reader.");

    let struct_name = args
        .name
        .unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().to_string())
        .to_case(Case::Pascal);

    let config = Config {
        lines: args.lines.unwrap_or(usize::MAX),
        min_fields: args.min_fields,
        struct_name,
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
