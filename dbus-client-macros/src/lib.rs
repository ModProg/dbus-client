use manyhow::{error_message, manyhow, Result, bail};
use proc_macro2::TokenStream;
use quote_use::{format_ident, quote_use as quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Paren};
use syn::{braced, parenthesized, Ident, LitStr, Path, Token};

mod dbus_object;

// TODO partial parse
#[manyhow(proc_macro)]
pub use dbus_object::dbus_object;

mod derive;
#[manyhow(proc_macro_derive(Append, attributes(value_signature)))]
pub use derive::append;
#[manyhow(proc_macro_derive(Arg))]
pub use derive::arg;
#[manyhow(proc_macro_derive(Get))]
pub use derive::get;
