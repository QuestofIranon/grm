pub mod get;
pub mod list;
pub mod root;

use structopt::StructOpt;

use anyhow::{anyhow, Context, Result};
use dirs::home_dir;
use enum_dispatch::enum_dispatch;
use git2::Config;
use std::{option::NoneError, path::PathBuf};

#[enum_dispatch]
#[derive(StructOpt, Debug)]
#[structopt(name = "grm", about = "Git remote repository manager")]
pub enum Grm {
    /// Clone a remote repository under the grm or ghq root directory
    #[structopt(name = "get")]
    Get(get::Get),

    /// Print a list of repositories relative to their root
    #[structopt(name = "list")]
    List(list::List),

    /// prints the grm.root of the current repository if you are inside one, otherwise prints the main root <not fully implemented>
    #[structopt(name = "root")]
    Root(root::Root),
}

#[inline]
pub fn grm_root() -> Result<PathBuf> {
    let config =
        Config::open_default().context("No git config found, do you have git installed?")?;

    config
        .get_path("grm.root")
        .or_else(|_| config.get_path("ghq.root"))
        .or_else(|_| Ok(home_dir()?.join("grm")))
        .map_err(|_: NoneError| anyhow!("No home directory found"))
}

#[enum_dispatch(Grm)]
pub trait ExecutableCommand {
    fn execute(self) -> Result<()>;
}
