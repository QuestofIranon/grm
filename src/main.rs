#[macro_use]
extern crate failure;

#[macro_use]
extern crate once_cell;

mod commands;
mod git;

use structopt::StructOpt;

fn main() {
    let _ = commands::Grm::from_args();
}
