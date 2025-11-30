use super::types::known_type_size;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitInt, Token, Type,
};

#[derive(Debug, Clone)]
pub enum FieldKind {
    LengthPrefix,
    Fixed,
    LengthPrefixString { policy_variant: Ident },
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

            let ty: Type = parse_type_or_len_as_type(&content)?;
            content.parse::<Token![,]>()?;

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

            let kind_string = kind_ident.to_string();

            let kind: FieldKind = match kind_string.as_str() {
                "fixed" => FieldKind::Fixed,
                "length_prefix" => FieldKind::LengthPrefix,
                "length_prefix_string" => {
                    if !content.peek(Token![,]) {
                        return Err(syn::Error::new(
                            kind_ident.span(),
                            "length_prefix_string requires: <max_length>, <policy_variant>",
                        ));
                    }

                    content.parse::<Token![,]>()?;
                    let max_len_lit: LitInt = content.parse()?;
                    let max_length_val = max_len_lit.base10_parse::<usize>()?;

                    content.parse::<Token![,]>()?;
                    let policy_variant: Ident = content.parse()?;

                    parsed_fields.push((
                        name,
                        ty,
                        offset_opt,
                        FieldKind::LengthPrefixString { policy_variant },
                        Some(max_length_val),
                    ));

                    let _ = input.parse::<Token![,]>();

                    continue;
                }
                other => {
                    return Err(syn::Error::new(
                        kind_ident.span(),
                        format!(
                            "unknown field kind `{}` (expected `fixed`, `length_prefix`, or `length_prefix_string`)",
                            other
                        ),
                    ));
                }
            };

            let mut max_length: Option<usize> = None;

            match kind {
                FieldKind::LengthPrefix => {
                    if !content.peek(Token![,]) {
                        return Err(syn::Error::new(
                            kind_ident.span(),
                            "length_prefix requires: <max_length>",
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
                            "unexpected extra argument after `fixed`",
                        ));
                    }
                }
                FieldKind::LengthPrefixString { .. } => {
                    unreachable!("handled earlier");
                }
            }

            parsed_fields.push((name, ty, offset_opt, kind, max_length));
            let _ = input.parse::<Token![,]>();
        }

        let mut current_offset: usize = 0;
        let mut fields = Vec::with_capacity(parsed_fields.len());

        for (name, ty, offset_opt, kind, max_length) in parsed_fields {
            let size = known_type_size(&ty).ok_or_else(|| {
                syn::Error::new(
                    name.span(),
                    "automatic offsets only support u8/u16/u32/u64/u128 and [u8; N]; provide explicit offset",
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

            match kind {
                FieldKind::LengthPrefix | FieldKind::LengthPrefixString { .. } => {
                    if max_length.is_none() {
                        return Err(syn::Error::new(
                            name.span(),
                            "missing max_length for length-prefix variant",
                        ));
                    }
                }
                FieldKind::Fixed => {
                    if max_length.is_some() {
                        return Err(syn::Error::new(
                            name.span(),
                            "fixed field should not have max_length",
                        ));
                    }
                }
            }

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
