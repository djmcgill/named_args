use proc_macro2::{self, Ident, Span};
use quote::quote;

use syn::*;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Colon, Paren};

pub fn named_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fn_call = syn::parse::<NamedExprCall>(input).unwrap();
    let func_ident = Ident::new(
        &format!("{}_named", fn_call.func.clone()),
        Span::call_site()
    );
    let arg_struct_ident = Ident::new(
        &format!("Args_{}", fn_call.func.clone()),
        Span::call_site()
    );
    let struct_fields = named_args_to_struct_args(fn_call.args);
    let output: proc_macro2::TokenStream = quote! {
        #func_ident ( #arg_struct_ident { #struct_fields })
    };
    output.into()
}

struct NamedExprCall {
    func: Ident, // FIXME: this should be Box<Expr> because not all functions are just an Ident
    _paren_token: Paren,
    args: Punctuated<NamedField, Comma>,
}
impl Parse for NamedExprCall {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(NamedExprCall {
            func: input.parse::<Ident>()?,
            _paren_token: parenthesized!(content in input),
            args: content.parse_terminated(NamedField::parse)?,
        })
    }
}

fn named_args_to_struct_args(named_args: Punctuated<NamedField, Comma>) -> Punctuated<FieldValue, Comma> {
    let mut arg_struct_expr_fields: Punctuated<FieldValue, Comma> = Punctuated::new();
    for named_arg in named_args {
        arg_struct_expr_fields.push(named_arg.to_struct_field())
    }
    arg_struct_expr_fields
}


struct NamedField {
    name: Ident,
    colon_token: Colon,
    expr: Expr,
}
impl Parse for NamedField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(NamedField {
            name: input.parse()?,
            colon_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}
impl NamedField {
    fn to_struct_field(self) -> FieldValue {
        FieldValue{
            attrs: vec![],
            member: Member::Named(self.name),
            colon_token: Some(self.colon_token),
            expr: self.expr,
        }
    }
}
