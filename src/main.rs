extern crate git2;
extern crate pathdiff;
extern crate structopt;
extern crate walkdir;

#[macro_use]
extern crate failure;

mod git;

use failure::{err_msg, Error};
use git::{
    clone::GitClone,
    pull::{GitPull, MergeOption},
};
use git2::Config;
use once_cell::unsync::Lazy;
use pathdiff::diff_paths;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt, Debug)]
#[structopt(name = "grm", about = "Git remote repository manager")]
enum Grm {
    /// Clone a remote repository under the grm or ghq root directory
    #[structopt(name = "get")]
    Get {
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
    },

    /// Print a list of repositories relative to their root
    #[structopt(name = "list")]
    List {
        /// print the full path instead <will likely become default behavior>
        #[structopt(long = "full-path", short = "p")]
        full_path: bool,
        /// forces the match to be exact (only if query is provided)
        #[structopt(long = "exact", short = "e")]
        exact: bool,
        /// Search Query
        query: Option<String>,
    },
    /// Change directories to the given repository
    #[structopt(name = "look")]
    Look {
        /// Repository to look in
        repository: String,
    },
    /// prints the grm.root of the current repository if you are inside one, otherwise prints the main root <not fully implemented>
    #[structopt(name = "root")]
    Root {
        /// prints all known grm roots <not implemented yet>
        #[structopt(long = "all", short = "a")]
        all: bool,
    },
}

impl Drop for Grm {
    #[allow(unreachable_patterns)]
    fn drop(&mut self) {
        // fixme: better messages?
        let git_config =
            Config::open_default().expect("No git config found, do you have git installed?");

        match self {
            Grm::Get {
                update,
                ssh,
                replace,
                remote,
            } => command_get(&git_config, *update, *replace, *ssh, remote.take()),
            Grm::List {
                full_path,
                exact,
                query,
            } => command_list(&git_config, *full_path, *exact, query.take()),
            Grm::Look { repository: _ } => println!("Unimplemented!"),
            Grm::Root { all: _ } => command_root(&git_config),
            _ => println!("Invalid command, use grm -h for help."),
        }
    }
}

fn command_get(
    git_config: &Config,
    update: bool,
    replace: bool,
    ssh: bool,
    remote: Option<String>,
) {
    let grm_root = match git_config.get_path("grm.root") {
        Ok(root) => root,
        Err(_error) => match git_config.get_path("ghq.root") {
            Ok(root) => root,
            Err(_error) => {
                println!("grm.root not specified in git config");
                return;
            }
        },
    };

    if let Some(remote) = remote {
        let sub_path = remote
            .trim_start_matches(&"https://")
            .trim_end_matches(&".git");

        let path = grm_root.as_path().join(sub_path);

        if !path.exists() {
            let mut clone = GitClone::new(path, ssh, remote);
            clone.run();
        } else {
            if replace {
                //fixme: better error handling
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
        }
    };
}

fn command_list(git_config: &Config, full_path: bool, exact_match: bool, query: Option<String>) {
    let grm_root = match git_config.get_path("grm.root") {
        Ok(root) => root,
        Err(_error) => match git_config.get_path("ghq.root") {
            Ok(root) => root,
            Err(_error) => {
                println!("grm.root not specified in git config");
                return;
            }
        },
    };

    let results: Vec<PathBuf> = match query {
        Some(query) => {
            WalkDir::new(&grm_root)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
                .min_depth(0)
                .max_depth(4)
                .into_iter()
                .filter_map(Result::ok)
                .filter_map(|p| {
                    p.path().as_os_str().to_os_string().to_str().map_or_else(
                        || None,
                        |e| {
                            let regex = Lazy::new(|| {
                                // if this errors out then let the panic occur
                                Regex::new(&format!(
                                    "{}",
                                    query.to_lowercase().replace("\\", "/").replace("/", r"\/")
                                ))
                                .unwrap()
                            });

                            let mut normalized_path = e.to_lowercase().replace("\\", "/");

                            if exact_match {
                                let path_parts = normalized_path.rsplit("/").collect::<Vec<&str>>();

                                if !(path_parts.len() > 2) {
                                    return None;
                                }

                                normalized_path = String::from(path_parts[path_parts.len() - 1])
                            }

                            if !regex.is_match(&normalized_path) {
                                return None;
                            }

                            Some(p.path().to_path_buf())
                        },
                    )
                })
                .collect()
        }
        None => WalkDir::new(&grm_root)
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
            .min_depth(0)
            .max_depth(4)
            .into_iter()
            .filter_map(Result::ok)
            .map(|p| p.path().to_path_buf())
            .collect(),
    };

    for entry in results {
        if entry.as_path().join(".git").exists() {
            if full_path {
                println!("{}", entry.as_path().display());
            } else {
                let relative_path = match diff_paths(&entry.as_path(), &grm_root) {
                    Some(path) => path,
                    None => continue,
                };

                println!("{}", relative_path.as_path().display());
            }
        }
    }
}

fn command_root(git_config: &Config) {
    let grm_root = match git_config.get_path("grm.root") {
        Ok(root) => root,
        Err(_error) => match git_config.get_path("ghq.root") {
            Ok(root) => root,
            Err(_error) => {
                println!("grm.root not specified in git config");
                return;
            }
        },
    };

    println!("{}", grm_root.as_path().display());
}

fn main() {
    let _ = Grm::from_args();
}
