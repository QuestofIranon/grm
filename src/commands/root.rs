use structopt::StructOpt;
use crate::commands::grm_root;

#[derive(StructOpt, Debug)]
pub struct Root {
    /// prints all known grm roots <not implemented yet>
    #[structopt(long = "all", short = "a")]
    all: bool,
}

impl Drop for Root {
    fn drop(&mut self) {
        command_root()
    }
}

fn command_root() {
    // todo: propagate errors upwards
    let grm_root = grm_root().unwrap();

    println!("{}", grm_root.as_path().display());
}
