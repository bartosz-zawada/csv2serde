use clap::{builder::ArgPredicate, Parser};
use csv;
use std::{
    char::{ParseCharError, TryFromCharError},
    path::PathBuf,
};
use thiserror;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Could not create Reader for path {1}")]
    CantOpenReader(#[source] csv::Error, String),

    #[error("Could not parse delimiter")]
    CantParseDelimiter(#[source] ParseCharError),

    #[error("Could not cast delimiter to u8: {0}")]
    CantCastDelimiter(#[source] TryFromCharError),

    #[error("Could not parse headers: {0}")]
    CantParseHeaders(#[source] csv::Error),

    #[error("Could not parse record: {0}")]
    CantParseRecord(#[source] csv::Error),
}
use Error::*;

type Result<T> = std::result::Result<T, Error>;

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
    #[arg(short = 'd', long, default_value_t = b',', value_parser = parse_delimiter)]
    delimiter: u8,

    /// Number of rows to analyze for field type prediction.
    /// A value of 0 means the entire file will be read and analyzed.
    #[arg(short = 'l', long, default_value = "0")]
    lines: Option<usize>,
}

fn parse_delimiter(arg: &str) -> Result<u8> {
    let c: char = arg.parse().map_err(CantParseDelimiter)?;
    Ok(TryInto::<u8>::try_into(c).map_err(CantCastDelimiter)?)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let path = args.file;
    let path_str = path.to_string_lossy().to_string();

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(args.delimiter)
        .from_path(path)
        .map_err(|e| CantOpenReader(e, path_str))?;

    let headers = reader.headers().map_err(CantParseHeaders)?;
    println!("Headers: {:#?}", headers);

        let record = record.map_err(CantParseRecord)?;
        println!("{:#?}", record);
    }

    Ok(())
}
