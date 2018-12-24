extern crate proc_macro;
extern crate proc_macro_hack;

mod call_syntax;
mod declare_syntax;

use proc_macro2;
use proc_macro_hack::proc_macro_hack;

#[proc_macro_hack]
pub fn named(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    call_syntax::named_impl(input)
}

#[proc_macro_attribute]
pub fn named_args(_: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    declare_syntax::named_args(item)
}
