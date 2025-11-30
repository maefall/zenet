use syn::{Expr, ExprLit, ExprParen, Lit, PathArguments, Type, TypeArray, TypePath};

// size for u8/u16/u32/u64/u128 & [u8; N]
pub fn known_type_size(ty: &Type) -> Option<usize> {
    #[allow(clippy::collapsible_if)]
    if let Type::Path(TypePath { path, qself: None }) = ty {
        if path.segments.len() == 1 {
            let segment = &path.segments[0];

            if matches!(segment.arguments, PathArguments::None) {
                return match segment.ident.to_string().as_str() {
                    "u8" => Some(1),
                    "u16" => Some(2),
                    "u32" => Some(4),
                    "u64" => Some(8),
                    "u128" => Some(16),
                    _ => None,
                };
            }
        }
    }

    // Arrays: [u8; N]
    is_u8_array_type(ty)
}

// Detect if the type is exactly [u8; N] and return N.
pub fn is_u8_array_type(ty: &Type) -> Option<usize> {

    #[allow(clippy::collapsible_if)]
    if let Type::Array(TypeArray { elem, len, .. }) = ty {
        if let Type::Path(TypePath { path, qself: None }) = &**elem {
            if path.segments.len() == 1 && path.segments[0].ident == "u8" {
                let n = match len {
                    Expr::Lit(ExprLit {
                        lit: Lit::Int(lit_int),
                        ..
                    }) => lit_int.base10_parse::<usize>().ok(),
                    Expr::Paren(ExprParen { expr, .. }) => {
                        if let Expr::Lit(ExprLit {
                            lit: Lit::Int(lit_int),
                            ..
                        }) = &**expr
                        {
                            lit_int.base10_parse::<usize>().ok()
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                return n;
            }
        }
    }

    None
}
