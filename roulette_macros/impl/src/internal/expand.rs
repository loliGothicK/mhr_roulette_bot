use thiserror::Error;
use crate::internal::ast::{Enum, Field, Input, Struct};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Data, Attribute, DeriveInput, Member, PathArguments, Result, Type, Visibility};

#[derive(Error, Debug)]
pub(crate) enum MacroError {
    #[error("Got unexpected identifier {}", .ident)]
    UnexpectedIdent {
        ident: String,
    }
}

pub fn derive(node: &DeriveInput) -> anyhow::Result<TokenStream> {
    let input = Input::from_syn(node)?;
    input.validate()?;
    Ok(match input {
        Input::Struct(input) => todo!(),
        Input::Enum(input) => todo!(),
    })
}

pub(crate) fn derive_objective(input: DeriveInput) -> proc_macro2::TokenStream {
    let ident = &input.ident;

    let expanded= quote! {
        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", "Dummy")
            }
        }
    };

    // Hand the output tokens back to the compiler
    expanded
}
