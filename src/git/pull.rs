use super::command_state::CommandState;
use failure::{Error, ResultExt};
use git2::{FetchOptions, MergeAnalysis, RemoteCallbacks, Repository};
use std::{
    cell::RefCell,
    path::PathBuf,
};

pub enum MergeOption {
    FF_ONLY, // currently this is the only merge option used
}

pub struct Pull {
    pub(crate) state: RefCell<CommandState>,
    repository: Repository,
    merge_option: MergeOption,
    ssh: bool,
}

impl Pull {
    pub fn new(path: PathBuf, merge_option: MergeOption, ssh: bool) -> Pull {
        let repository = match Repository::open(path.clone()) {
            Ok(repo) => repo,
            // fixme: better error handling here
            Err(_) => panic!("failed to open repo at: {}", path.as_path().display()),
        };
        
        
        let state = RefCell::new(CommandState {
            path,
            new_line: true,
            total: 0,
            current: 0,
        });

        Self {
            state,
            repository,
            merge_option,
            ssh,
        }
    }

    pub fn run(&self) -> Result<(), Error> {
        let mut remote = self
            .repository
            .find_remote("origin")
            .context("Could not find origin")?;

        let mut remote_callbacks = RemoteCallbacks::new();
        remote_callbacks.transfer_progress(|progress| {
            let mut state = self.state.borrow_mut();

            if !state.new_line {
                println!();
                state.new_line = true;
            }

            print!("total objects: {} \r", progress.total_objects());

            true
        });

        let mut options = FetchOptions::new();
        options.remote_callbacks(remote_callbacks);

        remote
            .fetch(&[], Some(&mut options), None)
            .context("Count not fetch from origin")?;

        let head = self.repository.head().context("Could not get the head")?;

        if !head.is_branch() {
            println!("Head is not currently pointing to a branch, cannot perform update");
            return Ok(());
        };

        let branch_name = match head.shorthand() {
            Some(branch_name) => branch_name,
            None => panic!("no name"),
        };

        let origin_oid = self
            .repository
            .refname_to_id(&format!("refs/remotes/origin/{}", branch_name))
            .context("Could not find oid from refname")?;

        let remote_commit = self
            .repository
            .find_annotated_commit(origin_oid)
            .context("No remote annotated commit")?;

        // Note that the underlying library function uses an unsafe block
        let merge_analysis = match self.repository.merge_analysis(&[&remote_commit]) {
            Ok((analysis, _)) => analysis,
            Err(err) => return Err(format_err!("Could not perform analysis {}", err)),
        };

        match &self.merge_option {
            MergeOption::FF_ONLY => {
                // perform a fastforward only
                if !merge_analysis.contains(MergeAnalysis::ANALYSIS_FASTFORWARD) {
                    println!("Fastforward cannot be be performed, please perform merge manually");
                    return Ok(());
                };

                let tree_to_checkout = self
                    .repository
                    .find_object(origin_oid, None)
                    .context("Could not find tree")?;

                self.repository
                    .checkout_tree(&tree_to_checkout, None)
                    .context("Failed to checkout tree")?;

                let mut head = self.repository.head().context("Could not get the head")?;
                head.set_target(origin_oid, "fast_forward")
                    .context("Could not fastforward")?;
            }
        }

        self.repository
            .cleanup_state()
            .context("Failed to cleanup")?;

        Ok(())
    }
}
