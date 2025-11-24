mod types;

mod ast;
pub use ast::DefineFieldsInput;

mod codegen;
pub use codegen::expand_define_fields;
