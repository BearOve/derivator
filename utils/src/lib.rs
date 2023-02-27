pub use crate::{attrs::Attrs, error::Result, impl_struct::StructGen};
pub use quote::{quote, quote_spanned, ToTokens};
pub use std::collections::{HashMap, HashSet};
pub use syn::{
    self,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    DeriveInput, Error, Ident, Meta, NestedMeta, Token,
    __private::{Span, TokenStream2},
};

#[macro_use]
mod error;
mod attrs;
mod impl_struct;
