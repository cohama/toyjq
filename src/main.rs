extern crate toyjq;

use toyjq::Json;

use std::io;
use std::io::{Read};

fn main() {
    interact(|s| {
        let json = Json::from_str(s).map_err(ToyjqError::ParseError)?;
        Ok(json.pretty_print(80))
    }).unwrap_or_else(|e| {
        println!("ERROR");
        println!("{:?}", e);
    })
}

#[derive(Debug)]
enum ToyjqError {
    IoError(io::Error),
    ParseError(toyjq::parsercombinator::ParseError)
}

type ToyjqResult<T> = std::result::Result<T, ToyjqError>;

fn interact<F>(f: F) -> ToyjqResult<()>
    where F: FnOnce(&str) -> ToyjqResult<String>
{
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).map_err(ToyjqError::IoError)?;
    let s = f(input.as_ref())?;
    println!("{}", s);

    Ok(())
}
