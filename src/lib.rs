use std::fs::File;

use convert_case::{self, Case, Casing};
pub use error::Error;
use quote::{format_ident, quote};

mod error;
mod keywords;

#[derive(Copy, Clone, Debug, PartialEq)]
enum TypeParser {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    String,
}

impl TypeParser {
    const TYPE_NAMES: [&'static str; 13] = [
        "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
        "String",
    ];

    const OPTIONAL_TYPE_NAMES: [&'static str; 13] = [
        "Option<u8>",
        "Option<u16>",
        "Option<u32>",
        "Option<u64>",
        "Option<u128>",
        "Option<i8>",
        "Option<i16>",
        "Option<i32>",
        "Option<i64>",
        "Option<i128>",
        "Option<f32>",
        "Option<f64>",
        "Option<String>",
    ];

    fn all() -> Vec<Self> {
        vec![
            TypeParser::U8,
            TypeParser::U16,
            TypeParser::U32,
            TypeParser::U64,
            TypeParser::U128,
            TypeParser::I8,
            TypeParser::I16,
            TypeParser::I32,
            TypeParser::I64,
            TypeParser::I128,
            TypeParser::F32,
            TypeParser::F64,
            TypeParser::String,
        ]
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn type_name(&self, optional: bool) -> &'static str {
        if optional {
            TypeParser::OPTIONAL_TYPE_NAMES[self.index()]
        } else {
            TypeParser::TYPE_NAMES[self.index()]
        }
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
    name: String,
    raw_name: String,
    valid_parsers: Vec<TypeParser>,
    optional: bool,
    is_empty: bool,
}

impl From<&str> for Header {
    fn from(field: &str) -> Self {
        // Handle punctuation, and convert to snake_case.
        let mut name = field
            .replace(|c: char| c.is_ascii_punctuation(), "_")
            .trim_start_matches('_')
            .to_case(Case::Snake);

        // Check for reserved words.
        if keywords::check(&name) {
            name = format!("r#{}", name);
        }

        Header {
            name,
            raw_name: field.to_string(),
            valid_parsers: TypeParser::all(),
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

    fn type_name(&self) -> &'static str {
        if self.is_empty {
            return "Option<()>";
        }

        TypeParser::all()
            .into_iter()
            .find(|p| self.valid_parsers.contains(p))
            .unwrap_or(TypeParser::String)
            .type_name(self.optional)
    }
}

fn generate_code(struct_name: &str, headers: Vec<Header>) -> Result<String, Error> {
    let struct_name = format_ident!("{}", struct_name);

    let headers = headers.iter().map(|h| {
        let header_name = format_ident!("{}", &h.name);
        let type_name = syn::Type::Verbatim(h.type_name().parse().unwrap());

        let maybe_rename = if h.name != h.raw_name {
            let raw_name = &h.raw_name;
            quote! {#[serde(rename = #raw_name)]}
        } else {
            quote! {}
        };

        quote! {
            #maybe_rename
            pub #header_name: #type_name,
        }
    });

    let full = quote! {
        #[derive(Debug, Deserialize)]
        pub struct #struct_name {
            #(#headers)*
        }
    };

    let syntax_tree = syn::parse2(full).map_err(Error::CantGenerateCode)?;
    Ok(prettyplease::unparse(&syntax_tree))
}

pub struct Config {
    pub lines: usize,
    pub min_fields: Option<usize>,
    pub struct_name: String,
}

pub fn run(mut reader: csv::Reader<File>, config: &Config) -> Result<String, Error> {
    let mut headers: Vec<Header> = reader
        .headers()
        .map_err(Error::CantParseHeaders)?
        .iter()
        .map(Header::from)
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
            headers.get_mut(i).unwrap().update_for(field);
        }
    }

    generate_code(config.struct_name.as_str(), headers)
}

#[cfg(test)]
mod tests {
    use crate::TypeParser;

    #[test]
    fn test_names() {
        let parsers = TypeParser::all();
        let results: Vec<(&str, &str)> = parsers
            .iter()
            .map(|p| (p.type_name(false), p.type_name(true)))
            .collect();

        assert_eq!(
            results,
            vec![
                ("u8", "Option<u8>"),
                ("u16", "Option<u16>"),
                ("u32", "Option<u32>"),
                ("u64", "Option<u64>"),
                ("u128", "Option<u128>"),
                ("i8", "Option<i8>"),
                ("i16", "Option<i16>"),
                ("i32", "Option<i32>"),
                ("i64", "Option<i64>"),
                ("i128", "Option<i128>"),
                ("f32", "Option<f32>"),
                ("f64", "Option<f64>"),
                ("String", "Option<String>"),
            ]
        );
    }
}
