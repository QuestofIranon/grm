extern crate git2;
extern crate pathdiff;
extern crate structopt;
extern crate walkdir;

#[macro_use]
extern crate failure;

mod git;

use failure::{Error, ResultExt};
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    Config, FetchOptions, MergeAnalysis, RemoteCallbacks, Repository,
};
use pathdiff::diff_paths;
use structopt::StructOpt;
use walkdir::WalkDir;
use git::clone::Clone;


#[derive(StructOpt, Debug)]
#[structopt(name = "grm", about = "Git remote repository manager")]
enum Grm {
    /// Clone a remote repository under the grm or ghq root directory
    #[structopt(name = "get")]
    Get {
        /// Perform an update for an already cloned repository (roughyl equivalent to `git pull --ff-only`)
        #[structopt(long = "update", short = "u")]
        update: bool,
        /// Use ssh <not implemented yet>
        #[structopt(short = "p")]
        ssh: bool,
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

// performs similar action to git pull -ff-only
fn git_pull_fastforward_only(repository: &Repository) -> Result<(), Error> {
    let mut remote = repository
        .find_remote("origin")
        .context("Could not find origin")?;

    let mut remote_callbacks = RemoteCallbacks::new();
    remote_callbacks.transfer_progress(|progress| {
        let owned_progress = progress.to_owned();

        println!("total objects: {}", owned_progress.total_objects());

        true
    });

    let mut options = FetchOptions::new();
    options.remote_callbacks(remote_callbacks);

    remote
        .fetch(&[], Some(&mut options), None)
        .context("Count not fetch from origin")?;

    let head = repository.head().context("Could not get the head")?;

    if !head.is_branch() {
        println!("Head is not currently pointing to a branch, cannot perform update");
        return Ok(());
    };

    let branch_name = match head.shorthand() {
        Some(branch_name) => branch_name,
        None => panic!("no name"),
    };

    let origin_oid = repository
        .refname_to_id(&format!("refs/remotes/origin/{}", branch_name))
        .context("Could not find oid from refname")?;

    let remote_commit = repository
        .find_annotated_commit(origin_oid)
        .context("No remote annotated commit")?;

    // Note that the underlying library function uses an unsafe block
    let merge_analysis = match repository.merge_analysis(&[&remote_commit]) {
        Ok((analysis, _)) => analysis,
        Err(err) => return Err(format_err!("Could not perform analysis {}", err)),
    };

    if !merge_analysis.contains(MergeAnalysis::ANALYSIS_FASTFORWARD) {
        println!("Fastforward cannot be be performed, please perform merge manually");
        return Ok(());
    };

    let tree_to_checkout = repository
        .find_object(origin_oid, None)
        .context("Could not find tree")?;

    repository
        .checkout_tree(&tree_to_checkout, None)
        .context("Failed to checkout tree")?;

    let mut head = repository.head().context("Could not get the head")?;
    head.set_target(origin_oid, "fast_forward")
        .context("Could not fastforward")?;

    repository.cleanup_state().context("Failed to cleanup")?;

    Ok(())
}

fn command_get(git_config: &Config, update: bool, ssh: bool, remote: Option<String>) {
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
            .trim_left_matches(&"https://")
            .trim_right_matches(&".git");

        let path = grm_root.as_path().clone().join(sub_path);

        if !path.exists() {
            let clone = Clone::new(path, ssh, remote.clone());
            clone.run();
        } else if update {
            let _repo = match Repository::open(path) {
                Ok(repo) => {
                    match git_pull_fastforward_only(&repo) {
                        Ok(_) => return,
                        Err(error) => println!("{}", error),
                    };
                }
                // fixme: better message
                Err(e) => panic!("failed to clone: {}", e),
            };
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

    match query {
        Some(query) => {
            for entry in WalkDir::new(&grm_root)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
                .min_depth(0)
                .max_depth(4)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = match entry.path().as_os_str().to_str() {
                    Some(entry_path) => entry_path,
                    None => continue,
                };

                if exact_match {
                    //fixme: make this cleanse windows only?
                    let cleansed_query = query.replace("\\", "/");
                    let cleansed_entry = entry_path.replace("\\", "/");

                    let entry_parts: Vec<&str> = cleansed_entry.rsplit("/").collect();
                    if !(entry_parts.len() > 2) {
                        continue;
                    };

                    let query_parts: Vec<&str> = cleansed_query.split("/").collect();

                    if query_parts.len() == 2 {
                        if (entry_parts[0] != query_parts[1]) || (entry_parts[1] != query_parts[0])
                        {
                            continue;
                        }
                    } else if query_parts.len() == 1 {
                        if entry_parts[0] != query_parts[0] {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else {
                    if !(entry_path.contains(&query)) {
                        continue;
                    }
                }

                if entry.path().join(".git").exists() {
                    if full_path {
                        println!("{}", entry.path().display());
                    } else {
                        let relative_path = match diff_paths(&entry.path(), &grm_root) {
                            Some(path) => path,
                            None => continue,
                        };

                        println!("{}", relative_path.as_path().display());
                    }
                }
            }
        }
        None => {
            for entry in WalkDir::new(&grm_root)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
                .min_depth(0)
                .max_depth(4)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().join(".git").exists() {
                    if full_path {
                        println!("{}", entry.path().display());
                    } else {
                        let relative_path = match diff_paths(&entry.path(), &grm_root) {
                            Some(path) => path,
                            None => continue,
                        };

                        println!("{}", relative_path.as_path().display());
                    }
                }
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

#[allow(unreachable_patterns)]
fn main() {
    let sub_command = Grm::from_args();

    // fixme: better messages?
    let git_config =
        Config::open_default().expect("No git config found, do you have git installed?");

    match sub_command {
        Grm::Get {
            update,
            ssh,
            remote,
        } => command_get(&git_config, update, ssh, remote),
        Grm::List {
            full_path,
            exact,
            query,
        } => command_list(&git_config, full_path, exact, query),
        Grm::Look { repository: _ } => println!("Unimplemented!"),
        Grm::Root { all: _ } => command_root(&git_config),
        _ => println!("Invalid command, use grm -h for help."),
    }
}
