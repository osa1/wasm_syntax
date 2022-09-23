mod ast;
mod codegen;

use ast::Grammar;
use codegen::codegen;

use proc_macro::TokenStream;

#[proc_macro]
pub fn grammar(input: TokenStream) -> TokenStream {
    let grammar = syn::parse_macro_input!(input as Grammar);
    codegen(&grammar).into()
}
