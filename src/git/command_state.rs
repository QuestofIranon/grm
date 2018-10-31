use git2::Progress;
use std::path::Path;
use std::path::PathBuf;

// CommandState is the common state across all (used) git commands
pub(crate) struct CommandState {
    //    progress: Option<Progress<'static>>,
    pub(crate) path: PathBuf,
    pub(crate) new_line: bool,
    pub(crate) total: usize,
    pub(crate) current: usize,
}
