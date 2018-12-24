extern crate my_macro;
extern crate proc_macro_hack;

use proc_macro_hack::proc_macro_hack;
use my_macro::named_args;

#[proc_macro_hack]
pub use my_macro::named_impl as named;

#[named_args]
fn foo(a: i32, b: u32, c: String) {
    println!("a: {}", a);
    println!("b: {}", b);
    println!("c: {}", c);
}

pub fn main() {
    named!(foo(a: -4, b: 5, c: "n".to_string()));
}
