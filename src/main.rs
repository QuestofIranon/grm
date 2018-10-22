#[macro_use]
extern crate structopt;
extern crate git2;
extern crate walkdir;
extern crate pathdiff;

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{
    Config, Direction, FetchOptions, MergeAnalysis, MergeOptions, Progress, Refspec,
    RemoteCallbacks, Repository, StatusOptions,
};
use std::{boxed::Box, env, io::Result, path::Path, thread, time};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};
use pathdiff::diff_paths;

#[derive(StructOpt, Debug)]
#[structopt(name = "grm", about = "Git remote repository manager")]
enum Grm {
    #[structopt(name = "get")]
    /// NOT IMPLEMENTED
    Get {
        #[structopt(long = "update", short = "u")]
        update: bool,
        #[structopt(short = "p")]
        ssh: bool,
        remote: Option<String>,
    },
    #[structopt(name = "list")]
    List {
        #[structopt(long = "full-path", short = "p")]
        full_path: bool,
        #[structopt(long = "exact", short = "e")]
        exact: bool,
    },
    #[structopt(name = "look")]
    /// NOT IMPLEMENTED
    Look { repository: String },
    #[structopt(name = "root")]
    Root {
        //todo: handle multiple roots
        #[structopt(long = "all", short = "a")]
        all: bool,
    },
}

// performs similar action to git pull -ff-only
fn git_pull_fastforward_only(repository: &Repository) -> Result<()> {
    let mut remote = match repository.find_remote("origin") {
        Ok(remote) => remote,
        Err(err) => panic!("Could not retrieve origin to pull {}", err), // todo: silently die?
    };

    let mut remote_callbacks = RemoteCallbacks::new();

    remote_callbacks.transfer_progress(|progress| {
        let owned_progress = progress.to_owned();

        println!("total objects: {}", owned_progress.total_objects());

        true
    });

    let mut options = FetchOptions::new();
    options.remote_callbacks(remote_callbacks);

    let fetch_results = match remote.fetch(&[], Some(&mut options), None) {
        Ok(fetch_results) => fetch_results,
        Err(err) => panic!("Could not fetch => {}", err),
    };

    let head = match repository.head() {
        Ok(head) => head,
        Err(err) => panic!("Could not get the head => {}", err),
    };

    if !head.is_branch() {
        println!("Cannot perform update");
        return Ok(());
    };

    let branch_name = match head.shorthand() {
        Some(branch_name) => branch_name,
        None => panic!("no name"),
    };

    println!("branch name => {}", branch_name);

    let local_oid = match head.target() {
        Some(oid) => oid,
        None => panic!("no local oid"),
    };

    let origin_oid = match repository.refname_to_id(&format!("refs/remotes/origin/{}", branch_name))
    {
        Ok(oid) => oid,
        Err(err) => panic!("NO remote OID"),
    };

    let local_commit = match repository.find_annotated_commit(local_oid) {
        Ok(commit) => commit,
        Err(err) => panic!("no local annotated commit"),
    };

    let remote_commit = match repository.find_annotated_commit(origin_oid) {
        Ok(commit) => commit,
        Err(_) => panic!("no remote annotated commit"),
    };

    // Note that the underlying library function uses an unsafe block
    let merge_analysis = match repository.merge_analysis(&[&remote_commit]) {
        Ok((analysis, _)) => analysis,
        Err(err) => panic!("Could not perform analysis => {}", err),
    };

    println!("merge analysis => {:?}", merge_analysis);

    if !merge_analysis.contains(MergeAnalysis::ANALYSIS_FASTFORWARD) {
        println!("Fastforward cannot be be performed, please perform merge manually");
        return Ok(());
    };

    let tree_to_checkout = match repository.find_object(origin_oid, None) {
        Ok(tree) => tree,
        Err(err) => panic!("MORE NOPE"),
    };

    match repository.checkout_tree(&tree_to_checkout, None) {
        Ok(()) => println!("maybe success"),
        Err(err) => panic!("NOEP"),
    };

    let mut head = match repository.head() {
        Ok(head) => head,
        Err(err) => panic!("Could not get the head => {}", err),
    };

    match head.set_target(origin_oid, "fast_forward") {
        Ok(_) => println!("success?"),
        Err(err) => panic!("failed to fast forward"),
    }

    match repository.cleanup_state() {
        Ok(()) => return Ok(()),
        Err(err) => panic!("Failed to run cleanup state => {}", err),
    };
}

fn command_get(git_config: &Config, update: bool, ssh: bool, remote: Option<String>) {
    let grm_root = match git_config.get_path("grm.root") {
        Ok(root) => root,
        Err(error) => match git_config.get_path("ghq.root") {
            Ok(root) => root,
            Err(error) => {
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
            let repo = match Repository::clone(&remote, path) {
                Ok(repo) => match repo.workdir() {
                    Some(dir) => println!("{}", dir.display()),
                    None => println!("{}", repo.path().display()),
                },
                Err(e) => panic!("failed to clone: {}", e),
            };
        } else if (update) {
            let repo = match Repository::open(path) {
                Ok(repo) => {
                    println!("not yet implemented");
                    //todo: find and update repo (git pull -ff)
                    git_pull_fastforward_only(&repo);
                }
                // fixme: better message
                Err(e) => panic!("failed to clone: {}", e),
            };
        }
    };
}

fn command_list(git_config: &Config, full_path: bool) {
    let grm_root = match git_config.get_path("grm.root") {
        Ok(root) => root,
        Err(error) => match git_config.get_path("ghq.root") {
            Ok(root) => root,
            Err(error) => {
                println!("grm.root not specified in git config");
                return;
            }
        },
    };

    for entry in WalkDir::new(&grm_root)
        .sort_by(|a,b| a.file_name().cmp(b.file_name()))
        .min_depth(0)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().join(".git").exists() {
            if full_path {
                println!("{}", entry.path().display());
            } else {
                let relative_path = match diff_paths(&entry.path(), &grm_root){
                    Some(path) => path,
                    None => return
                };

                println!("{}", relative_path.as_path().display());
            }
        }
    }
}

fn command_root(git_config: &Config) {
    let grm_root = match git_config.get_path("grm.root") {
        Ok(root) => root,
        Err(error) => match git_config.get_path("ghq.root") {
            Ok(root) => root,
            Err(error) => {
                println!("grm.root not specified in git config");
                return;
            }
        },
    };

    println!("{}", grm_root.as_path().display());
}

fn main() {
    let subcommand = Grm::from_args();

    // fixme: better messages?
    let git_config =
        Config::open_default().expect("No git config found, do you have git installed?");

    match subcommand {
        Grm::Get {
            update,
            ssh,
            remote,
        } => command_get(&git_config, update, ssh, remote),
        Grm::List { full_path, exact } => command_list(&git_config, full_path),
        Grm::Look { repository } => println!("Unimplemented!"),
        Grm::Root { all } => command_root(&git_config),
        _ => println!("Invalid command, use grm -h for help."),
    }
}
