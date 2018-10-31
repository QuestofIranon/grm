use super::command_state::CommandState;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    FetchOptions, RemoteCallbacks, Repository,
};
use std::{
    cell::RefCell,
    io,
    io::Write,
    path::{Path, PathBuf},
};

pub struct Clone {
    pub(crate) state: RefCell<CommandState>,
    ssh: bool,
    remote: String,
}

impl Clone {
    pub fn new(path: PathBuf, ssh: bool, remote: String) -> Clone {
        let state = RefCell::new(CommandState {
            path,
            new_line: false,
            total: 0,
            current: 0,
        });

        Self { state, ssh, remote }
    }

    pub fn run(&self) {
        let mut state = self.state.borrow_mut();
        println!("Cloning into '{}'", state.path.display());
        state.new_line = true;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.transfer_progress(|progress| {

            let mut state = self.state.borrow_mut();

            let network_percentage =
                (100 * progress.received_objects()) / progress.total_objects();
            let index_percentage = (100 * progress.indexed_objects()) / progress.total_objects();
            let transferred_kbytes = progress.received_bytes() / 1024;

            let co_percentage = if state.total > 0 {
                (100 * state.current) / state.total
            } else{ 0 };

            if progress.received_objects() == progress.total_objects() {
                if !state.new_line {
                    println!();
                    state.new_line = true;
                }

                print!(
                    "Resolving deltas: {}/{}\r",
                    progress.indexed_deltas(),
                    progress.total_deltas()
                );
            } else {
                println!(
                    "Receiving objects: {:3}% ({:4} kb, {:5}/{:5}) / idx {:3}% ({:5}/{:5}) / chk {:3}% ({:4}/{:4}) {}\r",
                    network_percentage,
                    transferred_kbytes,
                    progress.received_objects(),
                    progress.total_objects(),
                    index_percentage,
                    progress.indexed_objects(),
                    progress.total_objects(),
                    co_percentage,
                    state.current,
                    state.total,
                    state.path.display()
                );
            };
            io::stdout().flush().unwrap();

            true
        });

        let mut checkout = CheckoutBuilder::new();
        checkout.progress(|path, cur, total| {
            let mut state = self.state.borrow_mut();

            state.path = match path {
                Some(path) => path.to_path_buf(),
                None => Path::new("").to_path_buf(),
            };

            state.current = cur;
            state.total = total;
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        match RepoBuilder::new()
            .fetch_options(fetch_options)
            .with_checkout(checkout)
            .clone(&self.remote, state.path.as_path())
        {
            Ok(repo) => match repo.workdir() {
                Some(dir) => println!("{}", dir.display()),
                None => println!("{}", repo.path().display()),
            },
            Err(e) => panic!("failed to clone: {}", e),
        }
    }
}
