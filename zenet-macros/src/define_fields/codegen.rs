use super::{
    ast::{DefineFieldsInput, FieldKind},
    types::is_u8_array_type,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;

pub fn expand_define_fields(input: DefineFieldsInput) -> TokenStream2 {
    let fields = input.fields;

    // fixed prefix sizes
    let fixed_length_terms = fields.iter().map(|field| {
        if let Some(n) = is_u8_array_type(&field.ty) {
            quote! { #n }
        } else {
            let ty = &field.ty;
            quote! { <#ty as crate::__zwire_macros_support::WiredInt>::SIZE }
        }
    });

    // sum of MAX_LENGTH for variable-length fields
    let max_length_terms = fields.iter().filter_map(|field| match field.kind {
        FieldKind::LengthPrefix | FieldKind::LengthPrefixString { .. } => {
            let module_ident =
                Ident::new(&field.name.to_string().to_lowercase(), field.name.span());
            Some(quote! { #module_ident::MAX_LENGTH })
        }
        FieldKind::Fixed => None,
    });

    let fields_modules = fields.iter().map(|field| {
        let module_ident = Ident::new(&field.name.to_string().to_lowercase(), field.name.span());
        let ty = &field.ty;
        let offset_value = field.offset;
        let name_str = field.name.to_string().to_lowercase();

        let is_lp = matches!(field.kind, FieldKind::LengthPrefix);
        let policy_variant_opt = match &field.kind {
            FieldKind::LengthPrefixString { policy_variant } => Some(policy_variant),
            _ => None,
        };

        let wired_field_impl_item = quote! {
            impl crate::__zwire_macros_support::WiredField for Wired {
                const FIELD_NAME: &'static str = #name_str;
                const OFFSET: usize = #offset_value;
            }
        };

        let (max_length_item_pub, max_length_item) = match field.kind {
            FieldKind::LengthPrefix | FieldKind::LengthPrefixString { .. } => {
                let max_length = field.max_length.expect("parser guarantees max_length");
                (
                    Some(quote! { pub const MAX_LENGTH: usize = #max_length; }),
                    Some(quote! { const MAX_LENGTH: usize = #max_length; }),
                )
            }
            FieldKind::Fixed => (None, None),
        };

        if let Some(length) = is_u8_array_type(ty) {
            return quote! {
                pub mod #module_ident {
                    pub struct Wired;

                    #wired_field_impl_item

                    impl crate::__zwire_macros_support::WiredFixedBytes for Wired {
                        const LENGTH: usize = #length;
                        type Output = crate::__zwire_macros_support::Bytes;

                        #[inline]
                        fn from_bytes(bytes: crate::__zwire_macros_support::Bytes) -> Self::Output {
                            bytes
                        }
                    }

                    #max_length_item_pub
                }
            };
        }

        let length_field_impl = if is_lp || policy_variant_opt.is_some() {
            Some(quote! {
                impl crate::__zwire_macros_support::WiredLengthPrefixed for Wired {
                    type LengthPrefix = #ty;
                    #max_length_item
                }
            })
        } else {
            None
        };

        let wired_string_impl = policy_variant_opt.map(|policy_variant| quote! {
            impl crate::__zwire_macros_support::WiredString for Wired {
                type Inner = Wired;

                const POLICY: crate::__zwire_macros_support::WiredStringPolicyKind =
                    crate::__zwire_macros_support::WiredStringPolicyKind::#policy_variant;
            }
        });

        quote! {
            pub mod #module_ident {
                pub struct Wired(pub #ty);

                #wired_field_impl_item

                use crate::__zwire_macros_support::{WiredInt, WireError};

                impl WiredInt for Wired {
                    type Int = #ty;
                    type ByteArray = <#ty as WiredInt>::ByteArray;

                    const MAX: usize = <#ty as WiredInt>::MAX;
                    const SIZE: usize = <#ty as WiredInt>::SIZE;

                    fn read_raw_unchecked(source: &[u8]) -> Self::Int {
                        <#ty as WiredInt>::read_raw_unchecked(source)
                    }

                    fn read_unchecked(source: &[u8], field_name: &'static str) -> Result<usize, WireError> {
                        <#ty as WiredInt>::read_unchecked(source, field_name)
                    }

                    fn read(source: &[u8], field_name: &'static str) -> Result<Option<usize>, WireError> {
                        <#ty as WiredInt>::read(source, field_name)
                    }

                    fn to_bytes_from_usize(value: usize) -> Self::ByteArray {
                        <#ty as WiredInt>::to_bytes_from_usize(value)
                    }

                    fn to_bytes(value: Self::Int) -> Self::ByteArray {
                        <#ty as WiredInt>::to_bytes(value)
                    }
                }

                #max_length_item_pub
                #length_field_impl
                #wired_string_impl
            }
        }
    });

    quote! {
        pub mod fields {
            pub const FIXED_PART_LENGTH: usize = 0 #( + #fixed_length_terms )* ;
            pub const MAX_LENGTH: usize = FIXED_PART_LENGTH #( + #max_length_terms )* ;
            #(#fields_modules)*
        }
    }
}
