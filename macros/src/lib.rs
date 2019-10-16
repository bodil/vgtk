#![cfg_attr(can_show_location_of_runtime_parse_error, feature(proc_macro_span))]
#![deny(rust_2018_idioms, unsafe_code)]

extern crate proc_macro;

use proc_macro_hack::proc_macro_hack;

#[allow(dead_code)]
mod combo;
mod context;
mod error;
mod gtk;
mod lexer;
mod parser;

use crate::combo::{Parser, Stream, Success};

#[proc_macro_hack]
pub fn gtk(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let orig_stream = input.clone();
    let stream = lexer::unroll_stream(input.into(), false);
    let result = match parser::element().parse(&stream.cursor()) {
        Ok(Success { value: gtk, .. }) => gtk::expand_gtk(&gtk),
        Err(err) => return error::parse_error(&err).into(),
    };
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
    result.into()
}
