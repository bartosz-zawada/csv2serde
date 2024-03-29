mod styling;

use crate::{field::Field, Config, Error};
use quote::{format_ident, quote};

pub fn generate(config: &Config, fields: Vec<Field>) -> Result<String, Error> {
    let struct_name = format_ident!("{}", config.struct_name);

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

    Ok(styling::add_blank_lines(result, config.blank_lines))
}
