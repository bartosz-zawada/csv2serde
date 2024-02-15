use std::fs::File;

use convert_case::{self, Case, Casing};
pub use error::Error;
use quote::{format_ident, quote};

mod error;
mod keywords;

#[derive(Clone, Debug, PartialEq)]
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

    fn get_type_name(&self) -> &str {
        match &self {
            TypeParser::U8 => "u8",
            TypeParser::U16 => "u16",
            TypeParser::U32 => "u32",
            TypeParser::U64 => "u64",
            TypeParser::U128 => "u128",
            TypeParser::I8 => "i8",
            TypeParser::I16 => "i16",
            TypeParser::I32 => "i32",
            TypeParser::I64 => "i64",
            TypeParser::I128 => "i128",
            TypeParser::F32 => "f32",
            TypeParser::F64 => "f64",
            TypeParser::String => "String",
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

    fn get_type_name(&self) -> String {
        if self.is_empty {
            return "()".to_owned();
        }

        let all_parsers = TypeParser::get_all();

        let type_name = all_parsers
            .iter()
            .find(|p| self.valid_parsers.contains(p))
            .map(|p| p.get_type_name())
            .unwrap_or("String");

        if self.optional {
            format!("Option<{}>", type_name)
        } else {
            type_name.to_owned()
        }
    }
}

fn generate_code(struct_name: &str, headers: Vec<Header>) -> Result<String, Error> {
    let struct_name = format_ident!("{}", struct_name);

    let headers = headers.iter().map(|h| {
        let header_name = format_ident!("{}", &h.name);
        let type_name = syn::Type::Verbatim(h.get_type_name().parse().unwrap());

        let maybe_rename = if h.name != h.raw_name {
            let raw_name = &h.raw_name;
            quote! {#[serde(rename = #raw_name)]}
        } else {
            quote! {}
        };

        quote! {
            #maybe_rename
            #header_name: #type_name,
        }
    });

    let full = quote! {
        #[derive(Debug, Deserialize)]
        struct #struct_name {
            #(#headers)*
        }
    };

    let syntax_tree = syn::parse2(full).map_err(Error::CantGenerateCode)?;
    Ok(prettyplease::unparse(&syntax_tree))
}

pub fn run(
    mut reader: csv::Reader<File>,
    lines: usize,
    struct_name: &str,
) -> Result<String, Error> {
    let mut headers: Vec<Header> = reader
        .headers()
        .map_err(Error::CantParseHeaders)?
        .iter()
        .map(Header::from)
        .collect();

    for record in reader.records().take(lines) {
        let record = record.map_err(Error::CantParseRecord)?;

        for (i, field) in record.iter().enumerate() {
            headers.get_mut(i).unwrap().update_for(field);
        }
    }

    generate_code(struct_name, headers)
}
