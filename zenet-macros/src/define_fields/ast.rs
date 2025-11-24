use super::types::known_type_size;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitInt, Token, Type,
};

#[derive(Debug, Clone, Copy)]
pub enum FieldKind {
    LengthPrefix,
    Fixed,
}

pub struct FieldDef {
    pub name: Ident,
    pub ty: Type,
    pub offset: usize,
    pub kind: FieldKind,
    pub max_length: Option<usize>,
}

pub struct DefineFieldsInput {
    pub fields: Vec<FieldDef>,
}

// Parse either a type or an integer treated as [u8; N]
fn parse_type_or_len_as_type(input: ParseStream) -> syn::Result<Type> {
    if input.peek(LitInt) {
        let lit: LitInt = input.parse()?;
        let ty: Type = syn::parse_quote! { [u8; #lit] };

        Ok(ty)
    } else {
        input.parse()
    }
}

impl Parse for DefineFieldsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parsed_fields = Vec::new();

        while !input.is_empty() {
            let content;
            syn::parenthesized!(content in input);

            let name: Ident = content.parse()?;
            content.parse::<Token![,]>()?;

            // Type or integer (N -> [u8; N])
            let ty: Type = parse_type_or_len_as_type(&content)?;
            content.parse::<Token![,]>()?;

            // (Name, Type, <offset>, kind)
            // (Name, Type, kind)
            //
            // For length_prefix:
            //   (Name, Type, length_prefix, <max_length>)
            //   (Name, Type, <offset>, length_prefix, <max_length>)
            let lookahead = content.lookahead1();

            let mut offset_opt: Option<usize> = None;
            let kind_ident: Ident;

            if lookahead.peek(LitInt) {
                let offset_lit: LitInt = content.parse()?;
                offset_opt = Some(offset_lit.base10_parse()?);
                content.parse::<Token![,]>()?;
                kind_ident = content.parse()?;
            } else {
                kind_ident = content.parse()?;
            }

            let kind = match kind_ident.to_string().as_str() {
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

            // max_length:
            // - REQUIRED for length_prefix
            // - FORBIDDEN for fixed
            let mut max_length: Option<usize> = None;

            match kind {
                FieldKind::LengthPrefix => {
                    if !content.peek(Token![,]) {
                        return Err(syn::Error::new(
                            kind_ident.span(),
                            "length_prefix fields require a max length: \
                             (Name, Type, length_prefix, <max_length>) or \
                             (Name, Type, <offset>, length_prefix, <max_length>)",
                        ));
                    }
                    content.parse::<Token![,]>()?;
                    let lit: LitInt = content.parse()?;
                    max_length = Some(lit.base10_parse()?);
                }
                FieldKind::Fixed => {
                    if content.peek(Token![,]) {
                        return Err(syn::Error::new(
                            content.span(),
                            "unexpected extra argument after `fixed` \
                             (max length is only valid for `length_prefix` fields)",
                        ));
                    }
                }
            }

            parsed_fields.push((name, ty, offset_opt, kind, max_length));

            let _ = input.parse::<Token![,]>();
        }

        // Resolve offsets
        let mut current_offset: usize = 0;
        let mut fields = Vec::with_capacity(parsed_fields.len());

        for (name, ty, offset_opt, kind, max_length) in parsed_fields {
            let size = known_type_size(&ty).ok_or_else(|| {
                syn::Error::new(
                    name.span(),
                    "automatic offsets only support u8/u16/u32/u64/u128 and [u8; N]; \
                     for other types, please supply an explicit offset",
                )
            })?;

            let offset = match offset_opt {
                Some(explicit) => {
                    current_offset = explicit + size;
                    explicit
                }
                None => {
                    let auto = current_offset;
                    current_offset += size;
                    auto
                }
            };

            fields.push(FieldDef {
                name,
                ty,
                offset,
                kind,
                max_length,
            });
        }

        Ok(DefineFieldsInput { fields })
    }
}
