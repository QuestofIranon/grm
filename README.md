# grm

## About
grm is a git repository manager inspired by [ghq](https://github.com/motemen/ghq) and written in Rust. 
It is built using git2 and does not rely on making shell calls to the git cli.

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

## License
grm is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See LICENSE-APACHE and LICENSE-MIT for details.

## Author
QuestofIranon<QuestofIranon@gmail.com>
