use crate::{
    commands::{grm_root, ExecutableCommand},
    git::{
        clone::GitClone,
        pull::{GitPull, MergeOption},
    },
};
use anyhow::Result;
use std::fs;
use structopt::StructOpt;
use thiserror::Error;
use url::Url;

#[derive(StructOpt, Debug)]
pub struct Get {
    /// Perform an update for an already cloned repository (roughly equivalent to `git pull --ff-only`)
    #[structopt(long = "update", short = "u")]
    update: bool,
    /// Use ssh <not implemented yet>
    #[structopt(short = "p")]
    ssh: bool,
    /// Replace the local repository
    #[structopt(long = "replace", short = "r")]
    replace: bool,
    /// Remote url
    remote: Option<String>,
}

#[derive(Debug, Error)]
pub enum GetError {
    #[error("Malformed remote url: {}", remote)]
    ErrMalformedRemote { remote: String },
    #[error("No remote repository provided.")]
    ErrNoRemote,
}

impl ExecutableCommand for Get {
    fn execute(self) -> Result<()> {
        command_get(self.update, self.replace, self.ssh, self.remote)
    }
}

fn command_get(update: bool, replace: bool, ssh: bool, remote: Option<String>) -> Result<()> {
    let grm_root = grm_root()?;
    let remote = remote.ok_or(GetError::ErrNoRemote)?;

    let parsed_remote = Url::parse(&remote)?;

    println!("root: {}", grm_root.as_os_str().to_string_lossy());

    // todo: remove the clone to the error
    let path = grm_root
        .as_path()
        .join(
            parsed_remote
                .host_str()
                .ok_or(GetError::ErrMalformedRemote {
                    remote: remote.clone(),
                })?,
        )
        .join(parsed_remote.path());

    if !path.exists() {
        let mut clone = GitClone::new(path, ssh, remote);

        // todo: clone should return a Result in the future
        clone.run();

        return Ok(());
    }

    if replace {
        // fixme: better error handling
        fs::remove_dir_all(&path)?;

        GitClone::new(path, ssh, remote).run();

        return Ok(());
    }

    if update {
        let mut pull = GitPull::new(path, MergeOption::FastForwardOnly, ssh);

        return pull.run();
    }

    Ok(())
}
