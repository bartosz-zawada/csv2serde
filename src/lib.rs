use std::fs::File;

pub use error::Error;
use field::Field;
use quote::{format_ident, quote};

mod error;
mod field;
mod keywords;
mod styling;
mod type_parser;

fn generate_code(struct_name: &str, fields: Vec<Field>, config: &Config) -> Result<String, Error> {
    let struct_name = format_ident!("{}", struct_name);

    let fields = fields.iter().map(|f| {
        let field_name = format_ident!("{}", &f.name);
        let type_name = syn::Type::Verbatim(f.type_name().parse().unwrap());

        let maybe_rename = if f.name != f.raw_name {
            let raw_name = &f.raw_name;
            quote! {#[serde(rename = #raw_name)]}
        } else {
            quote! {}
        };

        quote! {
            #maybe_rename
            pub #field_name: #type_name,
        }
    });

    let full = quote! {
        #[derive(Debug, Deserialize)]
        pub struct #struct_name {
            #(#fields)*
        }
    };

    let syntax_tree = syn::parse2(full).map_err(Error::CantGenerateCode)?;
    let result = prettyplease::unparse(&syntax_tree);

    match config.blank_lines {
        Some(n) if n > 0 => Ok(styling::add_blank_lines(&result, n)),
        _ => Ok(result),
    }
}

pub struct Config {
    pub lines: usize,
    pub min_fields: Option<usize>,
    pub struct_name: String,
    pub blank_lines: Option<usize>,
}

pub fn run(mut reader: csv::Reader<File>, config: &Config) -> Result<String, Error> {
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

    generate_code(config.struct_name.as_str(), fields, config)
}
