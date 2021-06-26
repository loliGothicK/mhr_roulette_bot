#![feature(format_args_capture)]
mod internal;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Attribute};

#[proc_macro_derive(Objective, attributes(order))]
pub fn derive_objective(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(internal::derive_objective(input))
}
