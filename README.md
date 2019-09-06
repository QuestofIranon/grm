# grm

## About
grm is a git repository manager inspired by [ghq](https://github.com/motemen/ghq) and written in Rust. 
It is built using [git2](https://github.com/rust-lang/git2-rs) and does not rely on making shell calls to the git cli.

_Note that it is still a WIP and may not be ready for usage_

## Setup
Currently there is not a release version. See [Building](#building-from-source) below.

To set the root for `grm` to clone your repositories in, run the command:

`git config --global grm.root <your repository>`

If not set it will fallback to `~/grm`

_I haven't tested this on windows_

## Usage
```
Git remote repository manager

USAGE:
    grm <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    get     Clone a remote repository under the grm or ghq root directory
    help    Prints this message or the help of the given subcommand(s)
    list    Print a list of repositories relative to their root
    root    prints the grm.root of the current repository if you are inside one, otherwise prints the main root <not fully implemented>
```

## Building from Source
After cloning, ensure you have a 2018 version of Rust and run `cargo build` from the repository's root directory.

## License
grm is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE and LICENSE-MIT for details.

## Author
QuestofIranon<QuestofIranon@gmail.com>
