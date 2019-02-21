#[macro_use]
extern crate num_derive;
extern crate num_traits;

mod block;
mod error;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use crc::Hasher16;
use error::{Error, Result};
use std::fs;
use std::io;
use std::io::Read;

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

    let mut restbuf = Vec::<u8>::new();

    for _ in 0..2 {
        let mut buf = restbuf.clone();
        buf.resize(13, 0);
        file.read_exact(&mut buf[restbuf.len()..]).ok();

        let blockres = block::BlockPrefix::from(&buf);
        println!("{:?}", blockres);
        if blockres.is_err() {
            continue;
        }

        let (block, rest) = blockres.unwrap();
        println!("{:?}", rest);

        match block.block_type() {
            Some(block::HeadType::MarkerBlock) => parse_marker(&mut restbuf, &mut buf, &mut file)?,
            Some(block::HeadType::ArchiveHeader) => {
                parse_archive(&mut restbuf, &mut buf, &mut file)?
            }
            _ => println!("Don't have this yet lol: {:?}", block.block_type()),
        }

        // println!("{:?}", block.crc());
        // println!("{:?}", block.block_type());
        // println!("{:?}", block.flags());
        // println!("{:?}", block.size());

        // restbuf.truncate(0);
        // restbuf.extend(rest);
    }

    Ok(())
}

fn parse_marker(restbuf: &mut Vec<u8>, buf: &mut Vec<u8>, file: &mut impl Read) -> Result<()> {
    let (blk, rest) = block::BlockPrefix::from(buf)?;
    println!("{:?}", blk.crc());
    println!("{:?}", blk.block_type());
    println!("{:?}", blk.flags());
    println!("{:?}", blk.size());

    restbuf.truncate(0);
    restbuf.extend(rest);
    Ok(())
}

fn parse_archive(restbuf: &mut Vec<u8>, buf: &mut Vec<u8>, file: &mut impl Read) -> Result<()> {
    let (arc, rest) = block::ArchiveHeader::from(buf)?;
    println!("Archive: {:?}", arc);
    println!("CRC Expect: {:?}", arc.prefix().crc());
    // println!("CRC Actual: {:?}", arc.crc_digest().sum16());

    for i in 0..::std::u16::MAX {
        let digest = arc.crc_digest(i).sum16();
        if digest == arc.prefix().crc() {
            println!("FOUND IT: {:X}", i);
        }
    }

    restbuf.truncate(0);
    restbuf.extend(rest);
    Ok(())
}
