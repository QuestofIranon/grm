use crate::commands::{grm_root, ExecutableCommand};
use anyhow::{Context, Result};
use pathdiff::diff_paths;
use regex::Regex;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt, Debug)]
pub struct List {
    /// print the full path instead <will likely become default behavior>
    #[structopt(long = "full-path", short = "p")]
    full_path: bool,
    /// forces the match to be exact (only if query is provided)
    #[structopt(long = "exact", short = "e")]
    exact: bool,
    /// Search Query
    query: Option<String>,
}

impl ExecutableCommand for List {
    fn execute(self) -> Result<()> {
        command_list(self.full_path, self.exact, self.query)
    }
}

fn command_list(full_path: bool, exact_match: bool, query: Option<String>) -> Result<()> {
    let grm_root = grm_root()?;

    let dirs = WalkDir::new(&grm_root)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .min_depth(0)
        .max_depth(4)
        .into_iter()
        .filter_map(Result::ok);

    let results: Vec<PathBuf> = match query {
        Some(query) => {
            // if this errors out then let the panic occur
            let regex = Regex::new(
                &query
                    .to_lowercase()
                    .replace("\\", "/")
                    .replace("/", r"\/")
                    .to_string(),
            )
            .context("malformed input was provided causing an internal error")?;

            dirs.filter(|p| {
                // todo: handle unwrap better?
                let path_string: String = p.path().to_str().unwrap().to_string();

                let normalized_path = path_string.to_lowercase().replace("\\", "/");

                if !exact_match {
                    return regex.is_match(&normalized_path);
                }

                let path_parts = normalized_path.rsplit('/').collect::<Vec<&str>>();

                if path_parts.len() <= 2 {
                    return false;
                }

                regex.is_match(&String::from(path_parts[path_parts.len() - 1]))
            })
            .map(|p| p.path().to_path_buf())
            .collect()
        }
        None => dirs.map(|p| p.path().to_path_buf()).collect(),
    };

    for entry in results {
        if entry.as_path().join(".git").exists() {
            if full_path {
                println!("{}", entry.as_path().display());
                continue;
            }

            let relative_path = match diff_paths(&entry.as_path(), &grm_root) {
                Some(path) => path,
                None => continue,
            };

            println!("{}", relative_path.as_path().display());
        }
    }

    Ok(())
}
