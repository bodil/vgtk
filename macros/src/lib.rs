#![cfg_attr(can_show_location_of_runtime_parse_error, feature(proc_macro_span))]
#![deny(rust_2018_idioms, unsafe_code)]

use proc_macro;
use proc_macro_hack::proc_macro_hack;

mod context;
mod error;
mod gtk;
mod lexer;
mod parser;

#[proc_macro_hack]
pub fn gtk(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let orig_stream = input.clone();
    // let input: proc_macro2::TokenStream = input.into();
    // panic!("{:?}", input);
    let stream: lexer::Tokens = input.into();
    // panic!("{:?}", stream);

    let result = parser::grammar::GtkElementParser::new().parse(stream.lexer());
    match result {
        Err(err) => error::parse_error(&stream, &err),
        Ok(element) => gtk::expand_gtk(&element),
    }
    .into()

    // let mut f = std::fs::OpenOptions::new()
    //     .append(true)
    //     .open("macroexpand.log")
    //     .expect("unable to open macroexpand.log");
    // use std::io::Write;
    // f.write_fmt(format_args!(
    //     "Original stream:\n\n{}\n\nExpanded stream:\n\n{}\n\n------\n\n",
    //     orig_stream, result
    // ))
    // .expect("unable to write to macroexpand.log");
}
