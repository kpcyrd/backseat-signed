use crate::errors::*;
use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::io;
use std::path::PathBuf;

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
    Create(Create),
    Verify(Verify),
    #[command(subcommand)]
    Plumbing(Plumbing),
    Completions(Completions),
}

/// Bundle indicators that create a cryptographic link to the source input
#[derive(Debug, Parser)]
pub struct Create {}

/// Check collected indicators for integrity
#[derive(Debug, Parser)]
pub struct Verify {}

/// Low-level commands for debugging
#[derive(Debug, Subcommand)]
pub enum Plumbing {
    ArchlinuxPkgFromSig(ArchlinuxPkgFromSig),
    ArchlinuxPkgbuildFromPkg(ArchlinuxPkgbuildFromPkg),
    ArchlinuxFileFromPkgbuild(ArchlinuxFileFromPkgbuild),
    // ArchlinuxGitFromPkgbuild(ArchlinuxGitFromPkgbuild),
    // GitFromTarball(GitFromTarball),
    PgpVerify(PgpVerify),
    DebianSourcesFromRelease(DebianSourcesFromRelease),
    DebianTarballFromSources(DebianTarballFromSources),
}

/// Authenticate an Arch Linux package by signature and keyring
#[derive(Debug, Parser)]
pub struct ArchlinuxPkgFromSig {
    #[arg(long)]
    pub keyring: PathBuf,
    #[arg(long)]
    pub sig: PathBuf,
    pub file: PathBuf,
}

/// Authenticate a PKGBUILD belongs to an Arch Linux package
#[derive(Debug, Parser)]
pub struct ArchlinuxPkgbuildFromPkg {
    #[arg(long)]
    pub pkg: PathBuf,
    pub pkgbuild: PathBuf,
}

/// Authenticate a file referenced by hash from a PKGBUILD
#[derive(Debug, Parser)]
pub struct ArchlinuxFileFromPkgbuild {
    #[arg(long)]
    pub pkgbuild: PathBuf,
    pub file: PathBuf,
}

/// Authenticate a git tree by hash from a PKGBUILD
#[derive(Debug, Parser)]
pub struct ArchlinuxGitFromPkgbuild {}

/// Authenticate a git tree from a source tarball
#[derive(Debug, Parser)]
pub struct GitFromTarball {}

/// Authenticate a pgp signed message
#[derive(Debug, Parser)]
pub struct PgpVerify {
    #[arg(long)]
    pub keyring: PathBuf,
    #[arg(long)]
    pub sig: PathBuf,
    pub file: PathBuf,
}

/// Authenticate a Debian source index from a signed Debian release file
#[derive(Debug, Parser)]
pub struct DebianSourcesFromRelease {}

/// Authenticate a source tarball from a Debian source index
#[derive(Debug, Parser)]
pub struct DebianTarballFromSources {}

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
