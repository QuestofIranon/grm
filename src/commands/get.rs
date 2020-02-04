use crate::{
    commands::{grm_root, ExecutableCommand},
    git::{
        clone::GitClone,
        pull::{GitPull, MergeOption},
    },
};
use anyhow::{anyhow, Result};
use std::fs;
use structopt::StructOpt;
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

impl ExecutableCommand for Get {
    fn execute(self) -> Result<()> {
        command_get(self.update, self.replace, self.ssh, self.remote)
    }
}

fn command_get(update: bool, replace: bool, ssh: bool, remote: Option<String>) -> Result<()> {
    let remote = remote.ok_or(anyhow!("No remote repository provided."))?;
    let parsed_remote = if remote.starts_with("git@") {
        Url::parse(&format!("ssh://{}", remote.replace(".com:", ".com/")))?
    } else {
        Url::parse(&remote)?
    };

    // todo: the following code could potentially be improved?
    //       I've tried a few different approaches already...
    let mut path = grm_root()?;
    path.push(
        parsed_remote
            .host_str()
            .ok_or(anyhow!("Invalid remote url: {}", remote))?,
    );

    // strip out the leading "/" and ".git" if they exist
    let mut project = parsed_remote.path();
    project = project.strip_prefix("/").unwrap_or(project);
    let project = project.strip_suffix(".git").unwrap_or(project);

    let path = path.join(project);

    if !path.exists() {
        return GitClone::new(path, ssh, remote).run();
    }

    if replace {
        fs::remove_dir_all(&path)?;

        return GitClone::new(path, ssh, remote).run();
    }

    if update {
        return GitPull::new(path, MergeOption::FastForwardOnly, ssh).run();
    }

    Ok(())
}
