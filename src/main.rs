#[macro_use]
extern crate structopt;
extern crate git2;
extern crate walkdir;

use git2::Config;
use std::{env, path::Path};
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

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
    //todo: import
    #[structopt(name = "root")]
    Root {
        //todo: handle multiple roots
        #[structopt(long = "all", short = "a")]
        all: bool,
    },
}

fn command_get(git_config: &Config, update: bool, ssh: bool, remote: Option<String>){
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

    }
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

    for entry in WalkDir::new(grm_root)
        .min_depth(3)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| e.path().join(".git").exists())
        .filter_map(|e| e.ok())
    {
        println!("{}", entry.path().display());
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

    println!("{:?}", subcommand);

    match subcommand {
        Grm::Get{update, ssh, remote}   => println!("Unimplemented!"),
        Grm::List{full_path, exact}     => command_list(&git_config, full_path),
        Grm::Look{repository}           => println!("Unimplemented!"),
        Grm::Root{all}                  => command_root(&git_config),
        _                               => println!("Invalid command, use grm -h for help."),
    }

}
