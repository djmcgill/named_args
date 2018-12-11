extern crate my_macro;
use my_macro::named_args;

#[named_args]
fn foo(a: i32, b: u32) {
    println!("a: {}", a);
    println!("b: {}", b);
}

pub fn main() {
    foo(Args{a: -3, b: 4});
}
