use super::{
    ast::{DefineFieldsInput, FieldKind},
    types::is_u8_array_type,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

pub fn expand_define_fields(input: DefineFieldsInput) -> TokenStream2 {
    let fields = input.fields;

    // sum of sizes of all fields
    let fixed_length_terms = fields.iter().map(|field| {
        if let Some(n) = is_u8_array_type(&field.ty) {
            quote! { #n }
        } else {
            let ty = &field.ty;
            quote! { <#ty as crate::__zwire_macros_support::WiredInt>::SIZE }
        }
    });

    // Sum of MAX_LENGTH for all length_prefix field
    let max_length_terms = fields.iter().filter_map(|field| {
        if let FieldKind::LengthPrefix = field.kind {
            let module_name = field.name.to_string().to_lowercase();
            let module_ident = Ident::new(&module_name, field.name.span());
            Some(quote! { #module_ident::MAX_LENGTH })
        } else {
            None
        }
    });

    let fields_modules = fields.iter().map(|field| {
        let module_name = field.name.to_string().to_lowercase();
        let module_ident = Ident::new(&module_name, field.name.span());
        let ty = &field.ty;
        let offset_value = field.offset;
        let name_str = field.name.to_string().to_lowercase();

        let is_length_prefix = matches!(field.kind, FieldKind::LengthPrefix);

        let (max_length_item_pub, max_length_item) = if is_length_prefix {
            let max_length = field
                .max_length
                .expect("parser guarantees max_length for length_prefix");

            (
                Some(quote! {
                    pub const MAX_LENGTH: usize = #max_length;
                }),
                Some(quote! {
                    const MAX_LENGTH: usize = #max_length;
                }),
            )
        } else {
            (None, None)
        };

        // [u8; N] => marker + WiredFixedBytes
        if let Some(size) = is_u8_array_type(ty) {
            quote! {
                pub mod #module_ident {
                    pub struct Wired;

                    impl crate::__zwire_macros_support::WiredFixedBytes for Wired {
                        const SIZE: usize = #size;
                        const FIELD_NAME: &'static str = #name_str;

                        type Output = crate::__zwire_macros_support::Bytes;

                        #[inline]
                        fn from_bytes(bytes: crate::__zwire_macros_support::Bytes) -> Self::Output {
                            bytes
                        }
                    }

                    pub const OFFSET: usize = #offset_value;
                    #max_length_item_pub
                }
            }
        } else {
            // Non [u8; N] => marker + (optionally) WiredLengthPrefixed
            let length_field_impl = if is_length_prefix {
                Some(quote! {
                    impl crate::__zwire_macros_support::WiredLengthPrefixed for Wired {
                        type Int = #ty;

                        const FIELD_NAME: &'static str = #name_str;
                        #max_length_item
                    }
                })
            } else {
                None
            };

            quote! {
                pub mod #module_ident {
                    pub struct Wired(pub #ty);

                    impl crate::__zwire_macros_support::WiredIntField for Wired {
                        type Int = #ty;

                        const FIELD_NAME: &'static str = #name_str;
                    }

                    pub const OFFSET: usize = #offset_value;
                    #max_length_item_pub
                    #length_field_impl
                }
            }
        }
    });

    quote! {
        pub mod fields {
            pub const FIXED_PART_LENGTH: usize = 0 #( + #fixed_length_terms )* ;

            // Total MAX_LENGTH = fixed part + sum of all field MAX_LENGTH (only length_prefix fields have it)
            pub const MAX_LENGTH: usize = FIXED_PART_LENGTH #( + #max_length_terms )* ;

            #(#fields_modules)*
        }
    }
}
