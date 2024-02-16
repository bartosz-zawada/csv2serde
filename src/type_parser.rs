#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TypeParser {
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

    pub fn all() -> Vec<Self> {
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

    pub fn type_name(&self, optional: bool) -> &'static str {
        if optional {
            TypeParser::OPTIONAL_TYPE_NAMES[self.index()]
        } else {
            TypeParser::TYPE_NAMES[self.index()]
        }
    }

    pub fn can_parse(&self, field: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::TypeParser;

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
}
