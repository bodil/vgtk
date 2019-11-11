use crate::lexer::{to_stream, Token, Tokens};
use lalrpop_util::ParseError::*;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

pub type ParseError = lalrpop_util::ParseError<usize, Token, RsxParseError>;

#[derive(Debug)]
pub enum RsxParseError {
    TagMismatch { open: Tokens, close: Tokens },
    UnexpectedConstructor { name: Tokens, args: Token },
}

fn pprint_token(token: &str) -> &str {
    match token {
        "BraceGroupToken" => "code block",
        "ParenGroupToken" => "parenthesised block",
        "BracketGroupToken" => "array",
        "LiteralToken" => "literal",
        "IdentToken" => "identifier",
        a => a,
    }
}

fn pprint_tokens(tokens: &[String]) -> String {
    let tokens: Vec<&str> = tokens.iter().map(|s| pprint_token(&s)).collect();
    if tokens.len() > 1 {
        let start = tokens[..tokens.len() - 1].join(", ");
        let end = &tokens[tokens.len() - 1];
        format!("{} or {}", start, end)
    } else {
        tokens[0].to_string()
    }
}

pub fn parse_error(input: &[Token], error: &ParseError) -> TokenStream {
    match error {
        InvalidToken { location } => {
            let span = input[*location].span();
            quote_spanned! {span=>
                compile_error! { "invalid token" }
            }
        }
        UnrecognizedEOF { expected, .. } => {
            let msg = format!(
                "unexpected end of gtk! macro; missing {}",
                pprint_tokens(&expected)
            );
            quote! {
                compile_error! { #msg }
            }
        }
        UnrecognizedToken {
            token: (_, token, _),
            expected,
        } => {
            let span = token.span();
            let error_msg = format!("expected {}", pprint_tokens(&expected));
            let error = quote_spanned! { span => compile_error! { #error_msg } };
            quote! {{ #error }}
        }
        ExtraToken {
            token: (_, token, _),
        } => {
            let span = token.span();
            quote_spanned! { span => compile_error! { "superfluous token" }}
        }
        User {
            error: RsxParseError::TagMismatch { open, close },
        } => {
            let close_span = close[0].span();
            let close_msg = format!(
                "expected closing tag `</{}>`, found `</{}>`",
                to_stream(open),
                to_stream(close)
            );
            let close_error = quote_spanned! {close_span=>
                compile_error! { #close_msg }
            };
            let open_span = open[0].span();
            let open_error = quote_spanned! {open_span=>
                compile_error! { "unclosed tag" }
            };
            quote! {{
                #close_error
                #open_error
            }}
        }
        User {
            error: RsxParseError::UnexpectedConstructor { name, args },
        } => {
            let args_span = args.span();
            let error_msg = format!(
                "type is not a function - did you mean to call `{}::new()`?",
                name
            );
            quote_spanned! { args_span =>
                compile_error! { #error_msg }
            }
        }
    }
}
