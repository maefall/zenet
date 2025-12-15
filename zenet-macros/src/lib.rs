mod define_fields;
use define_fields::{expand_define_fields, DefineFieldsInput};

mod define_message;
use define_message::{DefineMessageInput, expand_define_message};

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn define_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DefineFieldsInput);
    let expanded = expand_define_fields(input);

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn define_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DefineMessageInput);
    let expanded = expand_define_message(input);

    TokenStream::from(expanded)
}
