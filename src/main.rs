#![feature(try_trait)]
#![feature(str_strip)]

mod commands;
mod git;

use crate::commands::ExecutableCommand;
use structopt::StructOpt;

fn main() {
    commands::Grm::from_args()
        .execute()
        .unwrap_or_else(|e| eprintln!("An error occurred: {}", e))
}
