#[macro_use]
extern crate bitflags;
extern crate bytes;
#[macro_use]
extern crate derive_error_chain;
#[macro_use]
extern crate error_chain;
extern crate flate2;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use bytes::BufMut;
use structopt::StructOpt;
use std::io::{self, Read, Seek};
use std::fs::File;

mod multiboot1;
mod multiboot2;

#[derive(Debug, ErrorChain)]
pub enum ErrorKind {
    Msg(String),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bootinfo", about = "Display boot information found in a file")]
struct Opts {
    #[structopt(help = "Input file")]
    input: String,
}

fn create_buffer<R: Read>(rdr: R) -> Result<bytes::Bytes> {
    const BUFLEN: usize = (32 * 1024);
    let mut fp = rdr.take(BUFLEN as u64);

    let buffer = bytes::BytesMut::with_capacity(BUFLEN);
    let mut buffer = buffer.writer();

    io::copy(&mut fp, &mut buffer)
        .chain_err(|| "failed to fill buffer with contents of input file")?;
    Ok(buffer.into_inner().freeze())
}

quick_main!{|| -> Result<()> {
    let opts = Opts::from_args();
    let fp = File::open(&opts.input)
        .chain_err(|| format!("failed to open input file {}", &opts.input))?;

    let fp = flate2::read::GzDecoder::new(fp);
    let bytes = if fp.header().is_some() { create_buffer(fp) } else {
        let mut fp = fp.into_inner();
        fp.seek(io::SeekFrom::Start(0)).chain_err(|| "failed to seek back to beginning of file")?;
        create_buffer(fp)
    };

    let bytes = bytes?;

    let header = multiboot1::Header::parse(bytes.clone());
    if let Some(header) = header {
        println!("{}", header);
    }

    let header = multiboot2::Header::parse(bytes.clone());
    if let Some(header) = header {
        println!("{}", header);
    }

    Ok(())
}}
