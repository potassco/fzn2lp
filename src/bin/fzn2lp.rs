use anyhow::Result;
use clap::Parser;
use fzn2lp::write_fz_stmt;
use fzn2lp::FlatZincError;
use log::error;
use std::fs;
use std::{
    io::{self, prelude::*},
    path::PathBuf,
};

/// Convert FlatZinc to ASP facts
#[derive(Parser, Debug)]
#[clap(name = "fzn2lp")]
struct Opt {
    /// Input file in flatzinc format
    #[clap(name = "FILE", parse(from_os_str))]
    file: Option<PathBuf>,
}

pub enum Reader<'a> {
    File(io::BufReader<fs::File>),
    Stdin(io::StdinLock<'a>),
}

impl<'a> io::Read for Reader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::File(reader) => reader.read(buf),
            Self::Stdin(guard) => guard.read(buf),
        }
    }
}

impl<'a> io::BufRead for Reader<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self {
            Self::File(reader) => reader.fill_buf(),
            Self::Stdin(guard) => guard.fill_buf(),
        }
    }
    fn consume(&mut self, amt: usize) {
        match self {
            Self::File(reader) => reader.consume(amt),
            Self::Stdin(guard) => guard.consume(amt),
        }
    }
}
fn main() {
    stderrlog::new()
        .module(module_path!())
        .verbosity(2)
        .init()
        .unwrap();
    if let Err(err) = run() {
        error!("{:?}", err);
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let opt = Opt::parse();

    let stdin = io::stdin();
    let input = match opt.file {
        Some(path) => {
            let file = fs::File::open(path)?;
            Reader::File(io::BufReader::new(file))
        }
        None => {
            let guard = stdin.lock();
            Reader::Stdin(guard)
        }
    };

    let mut out = std::io::stdout();
    let mut level = 1;
    let mut constraint_counter = 0;
    for line in input.lines() {
        write_fz_stmt(&mut out, &line?, &mut constraint_counter, &mut level)?;
    }
    if level < 5 {
        return Err(FlatZincError::NoSolveItem.into());
    }
    Ok(())
}
