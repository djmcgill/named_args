extern crate proc_macro;
extern crate proc_macro_hack;

use proc_macro2::{self, Ident, Span};
use proc_macro_hack::proc_macro_hack;
use quote::quote;

use syn::*;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Comma, Colon, Colon2, Paren};

struct NamedExprCall {
    func: Ident, // FIXME: this should be Box<Expr> because not all functions are just an Ident
    paren_token: Paren,
    args: Punctuated<NamedField, Comma>,
}
impl Parse for NamedExprCall {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(NamedExprCall {
            func: input.parse::<Ident>()?,
            paren_token: parenthesized!(content in input),
            args: content.parse_terminated(NamedField::parse)?,
        })
    }
}
impl NamedExprCall {
    fn to_named_call(self) -> Expr {
        let NamedExprCall { func, paren_token, args} = self;
        let mut arg_struct_expr_fields: Punctuated<FieldValue, Comma> = Punctuated::new();
        for NamedField{ name, colon_token, expr } in args {
            arg_struct_expr_fields.push(FieldValue{
                attrs: vec![],
                member: Member::Named(name),
                colon_token: Some(colon_token),
                expr,
            })
        }

        let arg_struct_expr = Expr::Struct(ExprStruct{
            attrs: vec![],
            path: path_of_one(Ident::new(&format!("Args_{}", func), Span::call_site())),
            brace_token: Brace::default(),
            fields: arg_struct_expr_fields,
            dot2_token: None,
            rest: None
        });
        let mut expr_args: Punctuated<Expr, Comma> = Punctuated::new();
        expr_args.push(arg_struct_expr);

        Expr::Call(ExprCall {
            attrs: vec![],
            func: Box::new(Expr::Path(ExprPath{
                attrs: vec![],
                qself: None,
                path: path_of_one(Ident::new(&format!("{}_named", func), Span::call_site())),
            })),
            paren_token,
            args: expr_args,
        })
    }
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

#[proc_macro_hack]
pub fn named_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let fn_call = syn::parse::<NamedExprCall>(input.clone()).unwrap();
    let named_fn_call = fn_call.to_named_call();
    let output: proc_macro2::TokenStream = quote! {
        #named_fn_call
    };
    output.into()
}

#[proc_macro_attribute]
pub fn named_args(_: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: proc_macro2::TokenStream = item.into();
    let item_fn = syn::parse2::<syn::ItemFn>(input).unwrap();
    let args = item_fn.decl.inputs.clone();

    let arg_struct_name = Ident::new(&format!("Args_{}", item_fn.ident), Span::call_site());

    let mut named_item_fn = item_fn.clone();
    named_item_fn.ident = Ident::new(&format!("{}_named", item_fn.ident), Span::call_site());
    let mut named_arg: Punctuated<FnArg, Comma> = Punctuated::new();

    let mut named_item_fn_args: Punctuated<Expr, Comma> = Punctuated::new();
    for arg in args.iter() {
        match arg {
            FnArg::Captured(ArgCaptured{pat, ..}) => {
                match pat {
                    Pat::Ident(PatIdent{ident, ..}) => {
                        named_item_fn_args.push(
                            Expr::Path(ExprPath{
                                path: path_of_one(ident.clone()),
                                qself: None,
                                attrs: vec![],
                            })
                        );
                    },
                    _ => panic!("Pattern type other than Ident not supported"),
                }
            },
            _ => panic!("Arg type other than Captured not supported"),
        }
    }


    named_item_fn.block = Box::new(Block {
        brace_token: Brace::default(),
        stmts: vec!(Stmt::Expr(Expr::Call(ExprCall{
            attrs: vec![],
            paren_token: Paren::default(),
            func: Box::new(Expr::Path(ExprPath{
                path: path_of_one(item_fn.ident.clone()),
                qself: None,
                attrs: vec![],
            })),
            args: named_item_fn_args,
        }))),
    });

    let mut named_arg_fields: Punctuated<FieldPat, Comma> = Punctuated::new();
    for arg in args.iter() {
        match arg {
            FnArg::Captured(ArgCaptured{pat, ..}) => {
                match pat {
                    Pat::Ident(PatIdent{ident, ..}) => {
                        named_arg_fields.push(
                            FieldPat {
                                attrs: vec![],
                                member: Member::Named(ident.clone()),
                                colon_token: None,
                                pat: Box::new(Pat::Ident(PatIdent {
                                    by_ref: None,
                                    mutability: None,
                                    ident: ident.clone(),
                                    subpat: None,
                                }))
                            }
                        );
                    },
                    _ => panic!("Pattern type other than Ident not supported"),
                }
            },
            _ => panic!("Arg type other than Captured not supported"),
        }
    }

    named_arg.push(FnArg::Captured(ArgCaptured {
        pat: Pat::Struct(PatStruct {
            path: path_of_one(arg_struct_name.clone()),
            brace_token: Brace::default(),
            fields: named_arg_fields,
            dot2_token: None,
        }),
        colon_token: Colon::default(),
        ty: Type::Path(TypePath {
            qself: None,
            path: path_of_one(arg_struct_name.clone())
        }),
    }));
    named_item_fn.decl.inputs = named_arg;

    let output: proc_macro2::TokenStream = quote! {
        struct #arg_struct_name { #args }

        #item_fn
        #named_item_fn
    };
    output.into()
}

fn path_of_one(ident: Ident) -> Path {
    let mut path_ident: Punctuated<PathSegment, Colon2> = Punctuated::new();
    path_ident.push(PathSegment{
        ident,
        arguments: PathArguments::None,
    });
    Path {
        leading_colon: None,
        segments: path_ident,
    }
}
