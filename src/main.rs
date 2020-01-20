#![feature(try_trait)]

mod commands;
mod git;

use crate::commands::ExecutableCommand;
use structopt::StructOpt;

fn main() {
    commands::Grm::from_args()
        .execute()
        .unwrap_or_else(|e| eprint!("An error occurred: {}", e))
}
