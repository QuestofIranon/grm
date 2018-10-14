#[macro_use]
extern crate structopt;
extern crate git2;
extern crate walkdir;

use std::{env, path::Path};
use structopt::StructOpt;
use git2::Config;
use walkdir::{DirEntry, WalkDir};

/*
'ghq' get [-u] [-p] (<repository URL> | <user>/<project> | <project>)
'ghq' list [-p] [-e] [<query>]
'ghq' look (<project> | <path/to/project>)
'ghq' import [-u] [-p] < FILE
'ghq' import <subcommand> [<args>…​]
'ghq' root [--all]
*/


#[derive(StructOpt, Debug)]
#[structopt(name="grm", about = "Git remote repository manager")]
enum Grm {
    #[structopt(name = "get")]
    /// NOT IMPLEMENTED
    Get {
        #[structopt(long = "update", short = "u")]
        update: bool,
        #[structopt(short = "p")]
        ssh: bool,
        remote: Option<String>
    },
    #[structopt(name = "list")]
    /// NOT IMPLEMENTED
    List {
        #[structopt(long = "full-path", short = "p")]
        full_path: bool,
        #[structopt(long = "exact", short = "e")]
        exact: bool,
    },
    #[structopt(name = "look")]
    /// NOT IMPLEMENTED
    Look {
        repository: String
    },
    //todo: import
    #[structopt(name = "root")]
    Root { //todo: handle multiple roots
        #[structopt(long = "all", short = "a")]
        all: bool
    }
}

fn command_list(git_config: &Config, full_path: bool) {
    let grm_root = match git_config.get_path("grm.root") {
        Ok(root) => root,
        Err(error) => match git_config.get_path("ghq.root") {
            Ok(root) => root,
            Err(error) => {
                println!("grm.root not specified in git config");
                return
            }
        }
    };

    for entry in WalkDir::new(grm_root).min_depth(3).max_depth(3).into_iter().filter_entry(|e| e.path().join(".git").exists()).filter_map(|e| e.ok()) {
            println!("{}", entry.path().display());
    }
}

fn main() {
    let args = Grm::from_args();

    // fixme: better messages?
    let git_config = Config::open_default()
        .expect("No git config found, do you have git installed?");

    command_list(&git_config, true)
    
    

}
