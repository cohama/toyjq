extern crate toyjq;

use toyjq::parser::*;

fn main() {
    println!("{:?}", string("foo").parse("fooo").unwrap());
}
