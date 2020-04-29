use nom::error::{convert_error, VerboseError};
use nom::Err;
use std::path::PathBuf;

use anyhow::Result;
use structopt::StructOpt;

/// Convert FlatZinc to AnsProlog facts
#[derive(StructOpt, Debug)]
#[structopt(name = "fzn2lp")]
struct Opt {
    /// Input in flatzinc format
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    file: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let buf = std::fs::read_to_string(opt.file)?;
    match flatzinc::model::<VerboseError<&str>>(&buf) {
        Ok((_, result)) => fzn2lp(&result),

        Err(Err::Error(e)) | Err(Err::Failure(e)) => {
            println!("Failed to parse flatzinc!\n{}", convert_error(&buf, e))
        }
        Err(e) => println!("Failed to parse flatzinc: {:?}", e),
    }
    Ok(())
}

fn fzn2lp(model: &flatzinc::Model) {
    // print!("{:?}",model);
    for c in &model.constraint_items {
        println!("constraint{:?}", c);
    }
}
