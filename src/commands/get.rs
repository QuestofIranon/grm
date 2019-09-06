use crate::git::{
    clone::GitClone,
    pull::{GitPull, MergeOption},
};
use std::fs;
use structopt::StructOpt;
use crate::commands::{grm_root, ExecutableCommand};
use failure::Error;

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
    fn execute(self) -> Result<(), Error> {
        command_get(self.update, self.replace, self.ssh, self.remote)
    }
}

fn command_get(update: bool, replace: bool, ssh: bool, remote: Option<String>) -> Result<(), Error> {
    let grm_root = grm_root()?;

    if let Some(remote) = remote {
        let sub_path = remote
            .trim_start_matches(&"https://")
            .trim_end_matches(&".git");

        let path = grm_root.as_path().join(sub_path);

        if !path.exists() {
            let mut clone = GitClone::new(path, ssh, remote);
            clone.run();

            return Ok(());
        }

        if replace {
            // fixme: better error handling
            fs::remove_dir_all(&path).unwrap();

            let mut clone = GitClone::new(path, ssh, remote);
            clone.run();
            return Ok(());
        }

        if update {
            let mut pull = GitPull::new(path, MergeOption::FastForwardOnly, ssh);

            match pull.run() {
                Ok(_) => return Ok(()),
                Err(error) => println!("{}", error),
            };

            return Ok(());
        }

    };

    Ok(())
}
