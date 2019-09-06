use structopt::StructOpt;
use failure::Error;
use crate::commands::{grm_root, ExecutableCommand};

#[derive(StructOpt, Debug)]
pub struct Root {
    /// prints all known grm roots <not implemented yet>
    #[structopt(long = "all", short = "a")]
    all: bool,
}

impl ExecutableCommand for Root {
    fn execute(self) -> Result<(), Error> {
        let grm_root = grm_root()?;

        println!("{}", grm_root.as_path().display());

        Ok(())
    }
}
