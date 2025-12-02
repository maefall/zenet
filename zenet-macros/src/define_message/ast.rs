use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, LitInt, Token,
};

pub struct Variant {
    pub name: Ident,
    pub value: LitInt,
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;

        input.parse::<Token![=]>()?;

        let value: LitInt = input.parse()?;

        Ok(Variant { name, value })
    }
}

pub struct DefineMessageInput {
    pub enum_name: Ident,
    pub variants: Vec<Variant>,
}

impl Parse for DefineMessageInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let enum_name: Ident = input.parse()?;

        if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
        }

        let content;
        braced!(content in input);

        let mut variants = Vec::new();

        while !content.is_empty() {
            let variant: Variant = content.parse()?;
            let _ = content.parse::<Token![,]>();

            variants.push(variant);
        }

        Ok(DefineMessageInput {
            enum_name,
            variants,
        })
    }
}
