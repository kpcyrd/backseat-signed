mod args;
mod buildinfo;
mod chksums;
mod errors;
mod pgp;
mod pkgbuild;
mod plumbing;

use crate::args::{Args, SubCommand};
use crate::errors::*;
use clap::Parser;
use env_logger::Env;
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = match (args.quiet, args.verbose) {
        (0, 0) => "warn,backseat_signed=info",
        (1, 0) => "warn",
        (_, 0) => "error",
        (_, 1) => "info,backseat_signed=debug",
        (_, 2) => "debug",
        (_, 3) => "debug,backseat_signed=trace",
        _ => "trace",
    };
    env_logger::init_from_env(Env::default().default_filter_or(log_level));

    match args.subcommand {
        SubCommand::Create(_create) => todo!("backseat-signed create"),
        SubCommand::Verify(_verify) => todo!("backseat-signed verify"),
        SubCommand::Plumbing(plumbing) => plumbing::run(plumbing).await,
        SubCommand::Completions(completions) => completions.generate(io::stdout()),
    }
}
