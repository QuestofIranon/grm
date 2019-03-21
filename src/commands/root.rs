
use structopt::StructOpt;

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
    let grm_root = grm_root!();

    println!("{}", grm_root.as_path().display());
}