use std::io::Read;

pub use error::Error;
use field::Field;

mod codegen;
mod error;
mod field;
mod keywords;
mod type_parser;

pub struct Config {
    pub lines: usize,
    pub min_fields: Option<usize>,
    pub struct_name: String,
    pub blank_lines: Option<usize>,
}

pub fn run<T: Read>(mut reader: csv::Reader<T>, config: &Config) -> Result<String, Error> {
    let mut fields: Vec<Field> = reader
        .headers()
        .map_err(Error::CantParseFieldHeaders)?
        .iter()
        .map(Field::from)
        .collect();

    for record in reader.records().take(config.lines) {
        let record = record.map_err(Error::CantParseRecord)?;

        if let Some(min_fields) = config.min_fields {
            let len = record.iter().filter(|s| !s.is_empty()).count();
            if len <= min_fields {
                continue;
            }
        }

        for (i, field) in record.iter().enumerate() {
            fields.get_mut(i).unwrap().update_for(field);
        }
    }

    codegen::generate(config, fields)
}
