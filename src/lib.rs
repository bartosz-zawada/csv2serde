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
struct Field {
    name: String,
    raw_name: String,
    valid_parsers: Vec<TypeParser>,
    optional: bool,
    is_empty: bool,
}

impl From<&str> for Field {
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

        Field {
            name,
            raw_name: field.to_string(),
            valid_parsers: TypeParser::all(),
            optional: false,
            is_empty: true,
        }
    }
}

impl Field {
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
        Some(n) if n > 0 => Ok(fancy_styling(&result, n)),
        _ => Ok(result),
    }
}

pub fn fancy_styling(code: &str, blank_lines: usize) -> String {
    let replacement_separator = "\n".repeat(blank_lines);

    let mut parts = vec![];

    // Let's skip straight for the struct block.
    let (first, rest) = code
        .split_once('{')
        .expect("There must be struct block opening braces.");
    parts.push(first);
    parts.push("{");

    // Let's take care of the end as well.
    let (rest, last) = rest
        .rsplit_once('}')
        .expect("There must be struct block closing braces.");

    // Split the struct fields using the trailing comma.
    let mut iter = rest.split_inclusive(',').peekable();

    while let Some(s) = iter.next() {
        parts.push(s);

        if iter.peek().is_some_and(|s| s.ends_with(',')) {
            parts.push(&replacement_separator);
        }
    }

    parts.push("}");
    parts.push(last);

    parts.concat()
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

#[cfg(test)]
mod tests {
    use crate::{fancy_styling, TypeParser};
    use indoc::indoc;

    #[test]
    fn names() {
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

    #[test]
    fn blank_lines() {
        let code = indoc! {r#"
            #[derive(Debug, Deserialize, PartialEq)]
            pub struct SomeStruct {
                pub field_a: String,
                pub field_b: i32,
                #[serde(rename = "terriblyNamed_Field")]
                pub terribly_named_field: Option<f32>,
                pub field_d: Option<()>,
            }"#};

        assert_eq!(
            fancy_styling(code, 0),
            indoc! {r#"
            #[derive(Debug, Deserialize, PartialEq)]
            pub struct SomeStruct {
                pub field_a: String,
                pub field_b: i32,
                #[serde(rename = "terriblyNamed_Field")]
                pub terribly_named_field: Option<f32>,
                pub field_d: Option<()>,
            }"#}
        );

        assert_eq!(
            fancy_styling(code, 1),
            indoc! {r#"
        #[derive(Debug, Deserialize, PartialEq)]
        pub struct SomeStruct {
            pub field_a: String,

            pub field_b: i32,

            #[serde(rename = "terriblyNamed_Field")]
            pub terribly_named_field: Option<f32>,

            pub field_d: Option<()>,
        }"#}
        );

        assert_eq!(
            fancy_styling(code, 2),
            indoc! {r#"
        #[derive(Debug, Deserialize, PartialEq)]
        pub struct SomeStruct {
            pub field_a: String,


            pub field_b: i32,


            #[serde(rename = "terriblyNamed_Field")]
            pub terribly_named_field: Option<f32>,


            pub field_d: Option<()>,
        }"#}
        );
    }
}
