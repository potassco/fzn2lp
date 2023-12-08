use anyhow::Result;
use clap::Parser;
use fzn2lp::{write_fz_stmt, FlatZincError};
use log::error;
use std::{
    fs,
    io::{self, BufRead},
    path::PathBuf,
};

/// Convert FlatZinc to ASP facts
#[derive(Parser, Debug)]
#[clap(name = "fzn2lp")]
#[command(version, author)]
struct Opt {
    /// Input file in flatzinc format
    #[arg(name = "FILE")]
    file: Option<PathBuf>,
}

fn main() {
    env_logger::builder().format_timestamp(None).init();
    if let Err(err) = run() {
        error!("{}", err);
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let mut stdin_lock = io::stdin().lock();
    let mut file_reader;
    let input: &mut dyn BufRead = match Opt::parse().file {
        Some(path) => {
            let file = fs::File::open(path)?;
            file_reader = io::BufReader::new(file);
            &mut file_reader
        }
        None => &mut stdin_lock,
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
