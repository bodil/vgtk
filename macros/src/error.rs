use crate::combo::Error;
use crate::lexer::Token;
// use ansi_term::Style;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

pub fn parse_error<'a>(error: &Error<'a, Token>) -> TokenStream {
    let expected = error
        .expected
        .iter()
        .cloned()
        .collect::<Vec<String>>()
        .join(", ");
    let msg = format!("expected {}", expected);
    match error.input.get() {
        Some(token) => {
            let span = token.span();
            let msg = format!("{}, but found \"{}\"", msg, token);
            quote_spanned! {span=>compile_error!{#msg}}
        }
        None => {
            quote! {compile_error!{#msg}}
        }
    }
}
