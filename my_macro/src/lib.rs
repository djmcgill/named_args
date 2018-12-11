extern crate proc_macro;
use proc_macro::*;
use std::iter::FromIterator;

#[proc_macro_attribute]
pub fn named_args(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_iter = item.into_iter();
    let name = item_iter.nth(1).unwrap(); // FIXME: should instead seek until a "fn" is found
    let args = item_iter.next().unwrap(); // FIXME: should work with generics
    let body = item_iter.next().unwrap(); // FIXME: should work with "where" type qualifiers

    // we need to group like this because of stuff like "ref a: &i32, b: u32"
    let grouped_args = match args {
        TokenTree::Group(group) => split_by_comma(group.stream()),
        other => panic!("Expected to find the function arguments here, instead found {:?}", other),
    };

    // Construct the struct's members
    let struct_arg_bodies_vec: Vec<TokenTree> = vec![
        Ident::new("a", Span::call_site()).into(), // FIXME: parse this properly from grouped_args
        Punct::new(':', Spacing::Alone).into(),
        Ident::new("i32", Span::call_site()).into(), // FIXME: consider refs and lifetimes :scream:
        Punct::new(',', Spacing::Alone).into(),
        Ident::new("b", Span::call_site()).into(),
        Punct::new(':', Spacing::Alone).into(),
        Ident::new("u32", Span::call_site()).into(),
    ];
    let struct_arg_bodies = TokenStream::from_iter(struct_arg_bodies_vec.into_iter());

    // Construct the _new_ argument to the function
    let param_arg_bodies_vec: Vec<TokenTree> = vec![
        Ident::new("args", Span::call_site()).into(),
        Punct::new(':', Spacing::Alone).into(),
        Ident::new("Args", Span::call_site()).into(),
    ];
    let param_arg_bodies = TokenStream::from_iter(param_arg_bodies_vec.into_iter());

    // Construct the returned syntax: a struct definition and then the transformed function
    let return_item_vec: Vec<TokenTree> = vec![
        Ident::new("struct", Span::call_site()).into(),
        Ident::new("Args", Span::call_site()).into(),
        Group::new(Delimiter::Brace, struct_arg_bodies).into(),
        Ident::new("fn", Span::call_site()).into(),
        name,
        Group::new(Delimiter::Parenthesis, param_arg_bodies).into(),
    ];
    let mut return_item = TokenStream::from_iter(return_item_vec.into_iter());

    // FIXME: inject "let Args { <stuff> } = args;" into function body
    let new_body: TokenStream = insert_arg_debinding(&grouped_args, body).into();

    return_item.extend(new_body);
    return_item.extend(TokenStream::from_iter(item_iter));
    return_item
}

fn insert_arg_debinding(args: &Vec<Vec<TokenTree>>, body: TokenTree) -> TokenTree {
    let body_stream = match body {
        TokenTree::Group(ref group) if group.delimiter() == Delimiter::Brace => group.stream(),
        other => panic!("Expected the fn body's group, instead found {:?}", other),
    };

    // let arg_names_vec: Vec<TokenTree> = args.as_slice().map(|arg_vec| arg_vec[0].clone());
    // FIXME: convert `args` to just get the names
    let arg_names_vec: Vec<TokenTree> = vec![
        Ident::new("a", Span::call_site()).into(),
        Punct::new(',', Spacing::Alone).into(),
        Ident::new("b", Span::call_site()).into(),
    ];
    let arg_names = TokenStream::from_iter(arg_names_vec.into_iter());

    let mut body_vec: Vec<TokenTree> = vec![
        Ident::new("let", Span::call_site()).into(),
        Ident::new("Args", Span::call_site()).into(),
        Group::new(Delimiter::Brace, arg_names).into(),
        Punct::new('=', Spacing::Alone).into(),
        Ident::new("args", Span::call_site()).into(),
        Punct::new(';', Spacing::Alone).into(),
    ];
    let mut body = TokenStream::from_iter(body_vec.into_iter());

    body.extend(body_stream);
    Group::new(Delimiter::Brace, body).into()
}

// Turns the stream "a b c , d e f , g h"
// into [["a", "b", "c"], ["d", "e", "f"], ["g", "h"]]
fn split_by_comma(stream: TokenStream) -> Vec<Vec<TokenTree>> {
    let mut iter = stream.into_iter();

    let mut groups = vec![];
    let mut current_group = vec![];
    loop {
        match iter.next() {
            Some(TokenTree::Punct(ref next_punct)) if next_punct.as_char() == ',' => {
                let mut temp_group = vec![];
                temp_group.append(&mut current_group);
                groups.insert(groups.len(), temp_group);
            },


            Some(next_non_punct) => {
                current_group.insert(current_group.len(), next_non_punct)
            }
            None => {
                if !current_group.is_empty() {
                    groups.insert(groups.len(), current_group);
                }
                break; // nll ftw
            },
        }
    }

    groups
}
