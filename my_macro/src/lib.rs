extern crate proc_macro;

use proc_macro2::{self, Ident, Span};
use quote::quote;

use syn::*;
use syn::punctuated::Punctuated;
use syn::token::{Brace, Comma, Colon, Colon2, Paren};

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
