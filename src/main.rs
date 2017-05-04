extern crate toyjq;

use toyjq::*;

fn main() {
    println!("{:?}", string("foo").parse("fooo").unwrap());
}
