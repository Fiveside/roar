// extern crate num;

// #[macro_use]
// extern crate num_derive;

mod block;
mod error;
mod io;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use error::Result;
use futures::executor::block_on;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("file").required(true))
        .get_matches();
    let filename = matches.value_of("file").unwrap();
    if let Err(e) = block_on(run(filename)) {
        eprintln!("An error ocurred: {}", e);
        //        if let Some(bt) = e.backtrace() {
        //            eprintln!("Backtrace: ");
        //            eprintln!("{}", bt);
        //        }
    }
}

async fn run(filename: &str) -> Result<()> {
    let f = ::async_std::fs::File::open(filename).await.unwrap();
    let bf = ::async_std::io::BufReader::new(f);
    let mut fr = io::AsyncFileReader::new(bf);

    for _ in 0..3 {
        let block = block::read_block(&mut fr).await?;
        println!("OOOooooo {:?}", block);
    }
    Ok(())
}

// async fn run(filename: &str) -> Result<()> {
//     println!("Attempting to open file {}", filename);
//     let mut file = BufReader::new(fs::File::open(filename).await?);

//     match block::read_block(&mut file).await? {
//         block::Block::Marker => println!("Found marker block!"),
//         block::Block::Archive(ref x) => println!("Found archive header: {:?}", x),
//         x => println!("unimplemented: {:?}", x),
//     }
//     Ok(())
// }

//#[macro_use]
//extern crate num_derive;
//extern crate num_traits;
//
//#[macro_use]
//extern crate tokio;
//
//mod block;
//mod error;
//
//use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, ArgMatches};
//use crc::Hasher16;
//use error::{Error, Result};
//use std::fs;
//use std::io;
//use std::io::Read;
//use tokio::prelude::*;

//fn main() -> Result<()> {
//    let matches = App::new(crate_name!())
//        .version(crate_version!())
//        .author(crate_authors!())
//        .about(crate_description!())
//        .arg(Arg::with_name("file").required(true))
//        .get_matches();
//
//    // run_sync(&matches)?;
//
//    // dude why
//    let fuckoff = matches.value_of("file").unwrap().to_owned();
//    ::tokio::run_async(
//        async move {
//            await!(run_async(fuckoff)).unwrap();
//        },
//    );
//
//    Ok(())
//}
//
//async fn run_async(filename: String) -> Result<()> {
//    let mut file = await!(tokio::fs::File::open(filename)).map_err(Error::io)?;
//
//    for _ in 0usize..3usize {
//        let blk = await!(block::read_block(&mut file))?;
//        match blk {
//            block::Block::Marker => println!("Magic Marker block"),
//            block::Block::Archive(header) => println!("Archive Header block"),
//            x => println!("Unimplemented? {:?}", x),
//        }
//    }
//
//    // let mut buf = Vec::<u8>::new();
//    // buf.resize(7, 0);
//
//    // await!(tokio::io::read_exact(&mut file, &mut buf)).map_err(Error::io)?;
//    // let (block, rest) = block::BlockPrefix::from_buf(&buf)?;
//    // println!("Block type: {:?}", block.block_type());
//    // println!("Rest: {:?}", rest);
//
//    Ok(())
//}

// fn run_sync(matches: &ArgMatches) -> Result<()> {
//     let filename = matches.value_of("file").unwrap();
//     let mut file = io::BufReader::new(fs::File::open(filename).map_err(Error::io)?);

//     let mut restbuf = Vec::<u8>::new();

//     for _ in 0..3 {
//         let mut buf = restbuf.clone();
//         buf.resize(128, 0);
//         file.read_exact(&mut buf[restbuf.len()..])
//             .map_err(Error::io)?;

//         let blockres = block::BlockPrefix::from_buf(&buf);
//         println!("{:?}", blockres);
//         if blockres.is_err() {
//             continue;
//         }

//         let (block, rest) = blockres.unwrap();
//         println!("Block type: {:?}", block.block_type());
//         println!("Block flags: {:?}", block.flags());
//         println!("Block size: {:?}", block.size());
//         println!("{:?}", rest);

//         match block.block_type() {
//             Some(block::HeadType::MarkerBlock) => parse_marker(&mut restbuf, &mut buf, &mut file)?,
//             Some(block::HeadType::ArchiveHeader) => {
//                 parse_archive(&mut restbuf, &mut buf, &mut file)?
//             }
//             Some(block::HeadType::FileHeader) => {
//                 parse_file_header(&mut restbuf, &mut buf, &mut file)?
//             }
//             _ => println!("Don't have this yet lol: {:?}", block.block_type()),
//         }

//         // println!("{:?}", block.crc());
//         // println!("{:?}", block.block_type());
//         // println!("{:?}", block.flags());
//         // println!("{:?}", block.size());

//         // restbuf.truncate(0);
//         // restbuf.extend(rest);
//     }

//     Ok(())
// }

// fn parse_marker(restbuf: &mut Vec<u8>, buf: &mut Vec<u8>, file: &mut impl Read) -> Result<()> {
//     let (blk, rest) = block::BlockPrefix::from_buf(buf)?;
//     println!("Parsing used {:?} bytes", buf.len() - rest.len());
//     println!("{:?}", blk.crc());
//     println!("{:?}", blk.block_type());
//     println!("{:?}", blk.flags());
//     println!("{:?}", blk.size());

//     restbuf.truncate(0);
//     restbuf.extend(rest);
//     Ok(())
// }

// fn parse_archive(restbuf: &mut Vec<u8>, buf: &mut Vec<u8>, file: &mut impl Read) -> Result<()> {
//     let (arc, rest) = block::ArchiveHeader::from_buf(buf)?;
//     println!("Parsing used {:?} bytes", buf.len() - rest.len());
//     println!("Archive: {:?}", arc);
//     println!("Reserved 1: {:?}", arc.reserved1());
//     println!("Reserved 2: {:?}", arc.reserved2());
//     println!("CRC Expect: {:?}", arc.prefix().crc());
//     // println!("CRC Actual: {:?}", arc.crc_digest().sum16());

//     for i in 0..::std::u16::MAX {
//         let digest = arc.crc_digest(i).sum16();
//         if digest == arc.prefix().crc() {
//             println!("FOUND IT: {:X}", i);
//         }
//     }

//     restbuf.truncate(0);
//     restbuf.extend(rest);
//     Ok(())
// }

// fn parse_file_header(restbuf: &mut Vec<u8>, buf: &mut Vec<u8>, file: &mut impl Read) -> Result<()> {
//     let (arcfile, rest) = block::FileHeader::from_buf(buf)?;
//     println!("Parsing used {:?} bytes", buf.len() - rest.len());
//     println!("Got a file header: {:?}", arcfile);
//     // let (arcfile, rest) = block::FileHeader::from(buf)?;

//     Ok(())
// }
