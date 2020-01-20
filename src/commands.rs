pub mod get;
pub mod list;
pub mod root;

use structopt::StructOpt;

use dirs::home_dir;
use enum_dispatch::enum_dispatch;
use thiserror::Error;
use git2::Config;
use std::path::PathBuf;
use std::option::NoneError;
use anyhow::Result;

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

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("No git config found, do you have git installed?")]
    ErrConfigNotFound,
    #[error("No home directory found")]
    ErrHomeNotFound,
}

#[inline]
pub fn grm_root() -> Result<PathBuf, ConfigError> {
    let config =
        Config::open_default().map_err(|_| -> ConfigError { ConfigError::ErrConfigNotFound })?;

    config
        .get_path("grm.root")
        .or_else(|_| config.get_path("ghq.root"))
        .or_else(|_| Ok(home_dir()?.join("grm")))
        .map_err(|_: NoneError| ConfigError::ErrHomeNotFound)
}

#[enum_dispatch(Grm)]
pub trait ExecutableCommand {
    fn execute(self) -> Result<()>;
}
