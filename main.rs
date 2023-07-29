use clap::{builder::ArgPredicate, Parser};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
enum C2SError {}

type C2SResult<T> = Result<T, C2SError>;

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
    #[arg(short = 'd', long, default_value = ",")]
    delimiter: String,

    /// Number of rows to analyze for field type prediction.
    /// A value of 0 means the entire file will be read and analyzed.
    #[arg(short = 'l', long, default_value = "0")]
    lines: Option<usize>,
}

fn main() -> C2SResult<()> {
    let _args = Args::parse();
    Ok(())
}
