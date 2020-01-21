use anyhow::Result;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    Config, FetchOptions, RemoteCallbacks,
};
use git2_credentials::CredentialHandler;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

struct Inner {
    working_path: PathBuf,
    new_line: bool,
    total: usize,
    current: usize,
}

pub struct GitClone {
    inner: Arc<RwLock<Inner>>,
    into: PathBuf,
    // ssh: bool, fixme: re-enable
    remote: String,
}

impl GitClone {
    pub fn new(path: PathBuf, _ssh: bool, remote: String) -> GitClone {
        let inner = Arc::new(RwLock::new(Inner {
            working_path: path.clone(),
            new_line: true,
            total: 0,
            current: 0,
        }));

        Self {
            inner,
            into: path,
            // ssh, fixme
            remote,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut callbacks = RemoteCallbacks::new();

        callbacks.transfer_progress(|progress| {
            if let Ok(mut inner) = self.inner.write() {
                let network_percentage = (100 * progress.received_objects()) / progress.total_objects();
                let index_percentage = (100 * progress.indexed_objects()) / progress.total_objects();
                let transferred_kbytes = progress.received_bytes() / 1024;

                let co_percentage = if inner.total > 0 {
                    100 * (inner.current / inner.total)
                } else { 0 };

                if progress.received_objects() == progress.total_objects() {
                    if !inner.new_line {
                        println!("new line was toggled");
                        inner.new_line = true;
                    }

                    print!(
                        "\r\rResolving deltas: {}/{}",
                        progress.indexed_deltas(),
                        progress.total_deltas()
                    );
                } else {
                    print!(
                        "\r\rReceiving objects: {:3}% ({:4} kb, {:5}/{:5}) / idx {:3}% ({:5}/{:5}) / chk {:3}% ({:4}/{:4}) {}",
                        network_percentage,
                        transferred_kbytes,
                        progress.received_objects(),
                        progress.total_objects(),
                        index_percentage,
                        progress.indexed_objects(),
                        progress.total_objects(),
                        co_percentage,
                        inner.current,
                        inner.total,
                        inner.working_path.display()
                    );
                };
                io::stdout().flush().unwrap();

                true
            } else {
                false
            }
		});

        // todo: refactor this later
        let config =
            Config::open_default().expect("No git config found, do you have git installed?");

        let mut credential_handler = CredentialHandler::new(config);

        callbacks.credentials(move |url, username, allowed| {
            credential_handler.try_next_credential(url, username, allowed)
        });

        let mut checkout = CheckoutBuilder::new();
        checkout.progress(|path, cur, total| {
            match self.inner.write() {
                Ok(mut inner) => {
                    inner.working_path = match path {
                        Some(path) => path.to_path_buf(),
                        None => Path::new("").to_path_buf(),
                    };

                    inner.current = cur;
                    inner.total = total;
                    true
                }
                //fixme: Panic?
                Err(_) => false,
            };
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        println!("Cloning into '{}'", &self.into.as_path().display());

        match RepoBuilder::new()
            .fetch_options(fetch_options)
            .with_checkout(checkout)
            .clone(&self.remote, &self.into)
        {
            Ok(repo) => match repo.workdir() {
                Some(dir) => println!("{}", dir.display()),
                None => println!("{}", repo.path().display()),
            },
            Err(e) => panic!("failed to clone: {}", e),
        }

        return Ok(());
    }
}
