#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod block;
mod error;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use error::{Error, Result};
use std::fs;
use std::io;

fn main() -> Result<()> {
    println!("Hello World!");

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("file").required(true))
        .get_matches();

    let filename = matches.value_of("file").unwrap();
    let mut file = io::BufReader::new(fs::File::open(filename).map_err(Error::io)?);

    for _ in 0..5 {
        let block = block::BlockHead::read(&mut file)?;
        println!("{:?}", block);
        println!("{:?}, {:?}", block.block_type(), block.block_size());
    }

    Ok(())
}
