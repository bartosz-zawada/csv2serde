use convert_case::{Case, Casing};

use crate::{keywords, type_parser::TypeParser};

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub raw_name: String,
    valid_parsers: Vec<TypeParser>,
    optional: bool,
    is_empty: bool,
}

impl Field {
    pub fn update_for(&mut self, field: &str) {
        if field.is_empty() {
            self.optional = true;
        } else {
            self.valid_parsers.retain(|parser| parser.can_parse(field));
            self.is_empty = false;
        }
    }

    pub fn type_name(&self) -> &'static str {
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
