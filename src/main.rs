extern crate toyjq;

use toyjq::*;
use toyjq::jsonparser::*;

use std::io;
use std::io::*;

fn main() {
    interact(|s| {
        let json = parse_json().parse(s).unwrap();
        print_json(&json, 80)
    }).unwrap_or_else(|e| {
        println!("ERROR");
        println!("{:?}", e);
    })
}

fn interact<F>(f: F) -> io::Result<()>
    where F: FnOnce(&str) -> String
{
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    println!("{}", f(input.as_ref()));

    Ok(())
}
