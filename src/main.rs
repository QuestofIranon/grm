#![feature(try_trait)]

#[macro_use]
extern crate failure;
extern crate once_cell;
extern crate enum_dispatch;

mod commands;
mod git;

use structopt::StructOpt;
use crate::commands::{ExecutableCommand, Grm};

fn main() {
    let command: Grm = commands::Grm::from_args();

    if let Err(e) = command.execute() {
        eprint!("An error occurred -> {}", e);
    }
}
