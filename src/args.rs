use crate::errors::*;
use crate::plumbing;
use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::io;

/// Authenticate cryptographic links from a signed derivate to its source input
#[derive(Debug, Parser)]
#[command(version)]
pub struct Args {
    /// Increase logging output (can be used multiple times)
    #[arg(short, long, global = true, action(ArgAction::Count))]
    pub verbose: u8,
    /// Reduce logging output (can be used multiple times)
    #[arg(short, long, global = true, action(ArgAction::Count))]
    pub quiet: u8,
    #[command(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    /*
    Create(Create),
    Verify(Verify),
    */
    #[command(subcommand)]
    Plumbing(plumbing::Plumbing),
    Completions(Completions),
}

/*
/// Bundle indicators that create a cryptographic link to the source input
#[derive(Debug, Parser)]
pub struct Create {}

/// Check collected indicators for integrity
#[derive(Debug, Parser)]
pub struct Verify {}
*/

/// Generate shell completions
#[derive(Debug, Parser)]
pub struct Completions {
    pub shell: Shell,
}

impl Completions {
    pub fn generate<W: io::Write>(&self, mut w: W) -> Result<()> {
        clap_complete::generate(self.shell, &mut Args::command(), "backseat-signed", &mut w);
        Ok(())
    }
}
