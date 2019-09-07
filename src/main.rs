#![feature(try_trait)]

#[macro_use]
extern crate failure;

mod commands;
mod git;

use structopt::StructOpt;
use crate::commands::ExecutableCommand;

fn main() {
    commands::Grm::from_args().execute()
        .unwrap_or_else(|e| {eprint!("An error occurred: {}", e)})
}
