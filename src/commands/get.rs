use crate::git::{
    clone::GitClone,
    pull::{GitPull, MergeOption},
};
use std::fs;
use structopt::StructOpt;
use crate::commands::grm_root;

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

impl Drop for Get {
    fn drop(&mut self) {
        command_get(self.update, self.replace, self.ssh, self.remote.take())
    }
}

fn command_get(update: bool, replace: bool, ssh: bool, remote: Option<String>) {
    // todo: propagate errors upwards
    let grm_root = grm_root().unwrap();

    if let Some(remote) = remote {
        let sub_path = remote
            .trim_start_matches(&"https://")
            .trim_end_matches(&".git");

        let path = grm_root.as_path().join(sub_path);

        if !path.exists() {
            let mut clone = GitClone::new(path, ssh, remote);
            clone.run();

            return;
        }

        if replace {
            // fixme: better error handling
            fs::remove_dir_all(&path).unwrap();

            let mut clone = GitClone::new(path, ssh, remote);
            clone.run();
            return;
        }

        if update {
            let mut pull = GitPull::new(path, MergeOption::FastForwardOnly, ssh);

            match pull.run() {
                Ok(_) => return,
                Err(error) => println!("{}", error),
            };

            return;
        }

    };
}
