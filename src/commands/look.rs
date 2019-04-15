use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Look {
    /// Repository to look in
    repository: String,
}
