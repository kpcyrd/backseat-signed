use backseat_signed::args::{Args, SubCommand};
use backseat_signed::errors::*;
use backseat_signed::plumbing;
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
        /*
        SubCommand::Create(_create) => bail!("This feature doesn't exist yet, refer to the README for how to use the plumbing commands"),
        SubCommand::Verify(_verify) => bail!("This feature doesn't exist yet, refer to the README for how to use the plumbing commands"),
        */
        SubCommand::Plumbing(plumbing) => plumbing::run(plumbing).await,
        SubCommand::Completions(completions) => completions.generate(io::stdout()),
    }
}
