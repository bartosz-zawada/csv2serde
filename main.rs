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

#[derive(Clone, Debug)]
enum TypeParser {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    String,
}

impl TypeParser {
    fn get_all() -> Vec<Self> {
        vec![
            TypeParser::I8,
            TypeParser::I16,
            TypeParser::I32,
            TypeParser::I64,
            TypeParser::I128,
            TypeParser::U8,
            TypeParser::U16,
            TypeParser::U32,
            TypeParser::U64,
            TypeParser::U128,
            TypeParser::F32,
            TypeParser::F64,
            TypeParser::String,
        ]
    }

    fn can_parse(&self, field: &str) -> bool {
        match self {
            TypeParser::String => true,
            TypeParser::U8 => field.parse::<u8>().is_ok(),
            TypeParser::U16 => field.parse::<u16>().is_ok(),
            TypeParser::U32 => field.parse::<u32>().is_ok(),
            TypeParser::U64 => field.parse::<u64>().is_ok(),
            TypeParser::U128 => field.parse::<u128>().is_ok(),
            TypeParser::I8 => field.parse::<i8>().is_ok(),
            TypeParser::I16 => field.parse::<i16>().is_ok(),
            TypeParser::I32 => field.parse::<i32>().is_ok(),
            TypeParser::I64 => field.parse::<i64>().is_ok(),
            TypeParser::I128 => field.parse::<i128>().is_ok(),
            TypeParser::F32 => field.parse::<f32>().is_ok(),
            TypeParser::F64 => field.parse::<f64>().is_ok(),
        }
    }
}

#[derive(Clone, Debug)]
struct Header {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    raw_name: String,
    valid_parsers: Vec<TypeParser>,
    optional: bool,
    is_empty: bool,
}

impl From<&str> for Header {
    fn from(name: &str) -> Self {
        Header {
            name: name.to_string(),
            raw_name: name.to_string(),
            valid_parsers: TypeParser::get_all(),
            optional: false,
            is_empty: true,
        }
    }
}

impl Header {
    fn update_for(&mut self, field: &str) {
        if field.is_empty() {
            self.optional = true;
        } else {
            self.valid_parsers.retain(|parser| parser.can_parse(field));
            self.is_empty = false;
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let path = args.file;
    let path_str = path.to_string_lossy().to_string();

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(args.delimiter)
        .from_path(path)
        .map_err(|e| CantOpenReader(e, path_str))?;

    let mut headers: Vec<Header> = reader
        .headers()
        .map_err(CantParseHeaders)?
        .iter()
        .map(Header::from)
        .collect();

    let lines = args.lines.unwrap_or(usize::MAX);

    for record in reader.records().take(lines) {
        let record = record.map_err(CantParseRecord)?;

        for (i, field) in record.iter().enumerate() {
            headers.get_mut(i).unwrap().update_for(field);
        }
    }

    println!("Processed Headers:");
    for h in headers {
        println!("{:#?}", h);
    }

    Ok(())
}
