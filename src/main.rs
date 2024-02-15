use clap::{builder::ArgPredicate, Parser};
use convert_case::{self, Case, Casing};
use csv::{self, Trim};
use csv2serde::Config;
use std::path::PathBuf;

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
        struct_name,
    };

    let code = csv2serde::run(reader, &config).unwrap();

    println!("{}", code);
}
