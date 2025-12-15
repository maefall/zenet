use crate::define_message::DefineMessageInput;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_define_message(input: DefineMessageInput) -> TokenStream {
    let enum_name = input.enum_name;
    let variants = input.variants;

    let variant_decls = variants.iter().map(|v| {
        let name = &v.name;
        let value = &v.value;

        quote! { #name = #value, }
    });

    let try_from_arms = variants.iter().map(|v| {
        let name = &v.name;
        let value = &v.value;

        quote! { #value => Ok(#enum_name::#name), }
    });

    let from_enum_arms = variants.iter().map(|v| {
        let name = &v.name;
        let value = &v.value;

        quote! { #enum_name::#name => #value as u8, }
    });

    quote! {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #enum_name {
            #(#variant_decls)*
        }

        impl std::convert::TryFrom<u8> for #enum_name {
            type Error = crate::__zwire_macros_support::WireError;

            fn try_from(code: u8) -> Result<Self, Self::Error> {
                match code {
                    #(#try_from_arms)*
                    other => Err(crate::__zwire_macros_support::WireError::InvalidMessageType(other)),
                }
            }
        }

        impl std::convert::TryFrom<&crate::__zwire_macros_support::Message> for #enum_name {
            type Error = crate::__zwire_macros_support::WireError;

            fn try_from(message: &crate::__zwire_macros_support::Message) -> Result<Self, Self::Error> {
                std::convert::TryFrom::<u8>::try_from(message.0)
            }
        }

        impl From<#enum_name> for u8 {
            fn from(message: #enum_name) -> Self {
                match message {
                    #(#from_enum_arms)*
                }
            }
        }

        impl From<#enum_name> for crate::__zwire_macros_support::Message {
            fn from(message: #enum_name) -> Self {
                crate::__zwire_macros_support::Message(message.into())
            }
        }
    }
}
