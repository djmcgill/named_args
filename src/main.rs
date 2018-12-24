extern crate my_macro;
use my_macro::named_args;

#[named_args]
fn foo(a: i32, b: u32, c: String) {
    println!("a: {}", a);
    println!("b: {}", b);
    println!("c: {}", c);
}

pub fn main() {
    foo_named(Args_foo{a: -3, b: 4, c: "s".to_string()});
}
