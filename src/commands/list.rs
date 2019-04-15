use once_cell::sync::Lazy;
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

impl Drop for List {
    fn drop(&mut self) {
        command_list(self.full_path, self.exact, self.query.take())
    }
}
fn command_list(full_path: bool, exact_match: bool, query: Option<String>) {
    let grm_root = grm_root!();

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
