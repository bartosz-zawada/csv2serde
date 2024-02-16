pub fn add_blank_lines(code: &str, blank_lines: usize) -> String {
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

#[cfg(test)]
mod tests {
    use super::add_blank_lines;
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
            add_blank_lines(code, 0),
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
            add_blank_lines(code, 1),
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
            add_blank_lines(code, 2),
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
