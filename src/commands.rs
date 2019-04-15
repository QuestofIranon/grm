#[macro_use]
mod macros;

pub mod get;
pub mod list;
pub mod look;
pub mod root;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "grm", about = "Git remote repository manager")]
pub enum Grm {
    /// Clone a remote repository under the grm or ghq root directory
    #[structopt(name = "get")]
    Get(get::Get),

    /// Print a list of repositories relative to their root
    #[structopt(name = "list")]
    List(list::List),

    /// Change directories to the given repository
    #[structopt(name = "look")]
    Look(look::Look),
    /// prints the grm.root of the current repository if you are inside one, otherwise prints the main root <not fully implemented>
    #[structopt(name = "root")]
    Root(root::Root),
}
