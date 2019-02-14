#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod block;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use std::fs;
use std::io;

fn main() -> Result<(), io::Error> {
    println!("Hello World!");

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("file").required(true))
        .get_matches();

    let filename = matches.value_of("file").unwrap();
    let mut file = io::BufReader::new(fs::File::open(filename)?);

    Ok(())
}
