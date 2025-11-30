mod define_fields;
use define_fields::{expand_define_fields, DefineFieldsInput};

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn define_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DefineFieldsInput);
    let expanded = expand_define_fields(input);

    TokenStream::from(expanded)
}
