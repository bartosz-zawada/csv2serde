use std::fs::File;

use convert_case::{self, Case, Casing};
pub use error::Error;
use quote::{format_ident, quote};
use type_parser::TypeParser;

mod error;
mod keywords;
mod type_parser;

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
    use super::fancy_styling;
    use indoc::indoc;

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
