extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitInt, Token, Type,
};

struct DefineFieldsInput {
    fields: Vec<(Ident, Type, usize)>,
}

impl Parse for DefineFieldsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut fields = Vec::new();

        while !input.is_empty() {
            let content;
            syn::parenthesized!(content in input);

            let name: Ident = content.parse()?;
            content.parse::<Token![,]>()?;

            let prim_ty: Type = content.parse()?;
            content.parse::<Token![,]>()?;

            let offset_lit: LitInt = content.parse()?;
            let offset: usize = offset_lit.base10_parse()?;

            fields.push((name, prim_ty, offset));

            let _ = input.parse::<Token![,]>();
        }
        Ok(DefineFieldsInput { fields })
    }
}

#[proc_macro]
pub fn define_fields(input: TokenStream) -> TokenStream {
    let DefineFieldsInput { fields } = parse_macro_input!(input as DefineFieldsInput);

    let type_definitions = fields.iter().map(|(name, primitive_type, _)| {
        let length_prefix_ident = Ident::new(&format!("{}LengthPrefix", name), name.span());

        quote! {
            pub type #length_prefix_ident = #primitive_type;
        }
    });

    let offset_consts = fields.iter().filter_map(|(name, _, offset)| {
        if *offset > 0 {
            let upper_ident = Ident::new(
                &format!("{}_FIELD_OFFSET", name.to_string().to_uppercase()),
                name.span(),
            );
            Some(quote! {
                pub const #upper_ident: usize = #offset;
            })
        } else {
            None
        }
    });

    let header_length_terms = fields.iter().map(|(name, _, _)| {
        let length_prefix_ident = Ident::new(&(name.to_string() + "LengthPrefix"), name.span());

        quote! { <#length_prefix_ident>::WIDTH }
    });

    let expanded = quote! {
        #(#type_definitions)*

        #(#offset_consts)*

        pub const HEADER_LENGTH: usize = 0 #( + #header_length_terms )* ;
    };

    TokenStream::from(expanded)
}
