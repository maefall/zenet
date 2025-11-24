extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, LitInt, Token, Type,
};

enum FieldKind {
    LengthPrefix,
    Fixed,
}

struct FieldDef {
    name: Ident,
    ty: Type,
    offset: usize,
    _kind: FieldKind,
}

struct DefineFieldsInput {
    fields: Vec<FieldDef>,
}

impl Parse for DefineFieldsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut fields = Vec::new();

        while !input.is_empty() {
            let content;
            syn::parenthesized!(content in input);

            let name: Ident = content.parse()?;
            content.parse::<Token![,]>()?;

            let ty: Type = content.parse()?;
            content.parse::<Token![,]>()?;

            let offset_lit: LitInt = content.parse()?;
            content.parse::<Token![,]>()?;

            let offset: usize = offset_lit.base10_parse()?;

            let kind_ident: Ident = content.parse()?;
            let _kind = match kind_ident.to_string().as_str() {
                "fixed" => FieldKind::Fixed,
                "length_prefix" => FieldKind::LengthPrefix,
                other => {
                    return Err(syn::Error::new(
                        kind_ident.span(),
                        format!(
                            "unknown field kind `{}` (expected `fixed` or `length_prefix`)",
                            other
                        ),
                    ));
                }
            };

            fields.push(FieldDef {
                name,
                ty,
                offset,
                _kind,
            });

            let _ = input.parse::<Token![,]>();
        }

        Ok(DefineFieldsInput { fields })
    }
}

#[proc_macro]
pub fn define_fields(input: TokenStream) -> TokenStream {
    let DefineFieldsInput { fields } = parse_macro_input!(input as DefineFieldsInput);

    let wired_type_defs = fields.iter().map(|field| {
        let name = &field.name;
        let ty = &field.ty;
        let length_prefix_ident = Ident::new(&format!("{}Wired", name), name.span());

        quote! {
            pub type #length_prefix_ident = #ty;
        }
    });

    let offset_consts = fields.iter().map(|field| {
        let upper_ident = Ident::new(
            &format!("{}_FIELD_OFFSET", field.name.to_string().to_uppercase()),
            field.name.span(),
        );

        let offset = field.offset;

        quote! {
            pub const #upper_ident: usize = #offset;
        }
    });

    let header_length_terms = fields.iter().map(|field| {
        let name = &field.name;
        let length_prefix_ident = Ident::new(&(name.to_string() + "Wired"), name.span());

        quote! { <#length_prefix_ident as crate::codec::WiredInt>::SIZE }
    });

    let expanded = quote! {
        #(#wired_type_defs)*

        #(#offset_consts)*

        pub const FIXED_PART_LENGTH: usize = 0 #( + #header_length_terms )* ;
    };

    TokenStream::from(expanded)
}
