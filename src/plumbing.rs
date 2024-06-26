use crate::apt;
use crate::buildinfo;
use crate::chksums;
use crate::compression;
use crate::errors::*;
use crate::pgp;
use crate::pkgbuild;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio::fs;

pub async fn run(plumbing: Plumbing) -> Result<()> {
    match plumbing {
        Plumbing::ArchlinuxPkgFromSig(args) => args.run().await,
        Plumbing::ArchlinuxPkgbuildFromPkg(args) => args.run().await,
        Plumbing::ArchlinuxFileFromPkgbuild(args) => args.run().await,
        // Plumbing::ArchlinuxGitFromPkgbuild(args) => args.run(),
        // Plumbing::GitFromTarball(args) => args.run(),
        Plumbing::PgpVerify(args) => args.run().await,
        Plumbing::DebianSourcesFromRelease(args) => args.run().await,
        Plumbing::DebianTarballFromSources(args) => args.run().await,
    }
}

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

impl ArchlinuxPkgFromSig {
    async fn run(&self) -> Result<()> {
        info!("Loading keyring from {:?}", self.keyring);
        let keyring = fs::read(&self.keyring).await?;
        let keyring = pgp::keyring(&keyring)?;
        info!("Loaded {} public keys", keyring.len());

        info!("Loading signature from {:?}", self.sig);
        let sig = fs::read(&self.sig).await?;
        let sig = pgp::signature(&sig)?;

        info!("Loading package from {:?}", self.file);
        let msg = fs::read(&self.file).await?;

        pgp::verify(&keyring, &sig, &msg)?;
        info!("Package verified successfully");

        Ok(())
    }
}

/// Authenticate a PKGBUILD belongs to an Arch Linux package
#[derive(Debug, Parser)]
pub struct ArchlinuxPkgbuildFromPkg {
    #[arg(long)]
    pub pkg: PathBuf,
    pub pkgbuild: PathBuf,
}

impl ArchlinuxPkgbuildFromPkg {
    async fn run(&self) -> Result<()> {
        info!("Loading PKGBUILD from {:?}", self.pkgbuild);
        let pkgbuild = fs::read(&self.pkgbuild).await?;

        info!("Loading package from {:?}", self.pkg);
        let pkg = fs::read(&self.pkg).await?;

        info!("Parsing .BUILDINFO from package");
        let buildinfo = buildinfo::from_archlinux_pkg(&pkg)?;
        let pkgbuild_sha256sum = buildinfo.pkgbuild_sha256sum;

        debug!("Hashing PKGBUILD");
        let sha256 = chksums::sha256(&pkgbuild);

        if sha256 == pkgbuild_sha256sum {
            info!("PKGBUILD verified successfully");
            Ok(())
        } else {
            bail!("PKGBUILD sha256={sha256:?} does not match .BUILDINFO pkgbuild_sha256sum={pkgbuild_sha256sum:?}");
        }
    }
}

/// Authenticate a file referenced by hash from a PKGBUILD
#[derive(Debug, Parser)]
pub struct ArchlinuxFileFromPkgbuild {
    #[arg(long)]
    pub pkgbuild: PathBuf,
    pub file: PathBuf,
}

impl ArchlinuxFileFromPkgbuild {
    async fn run(&self) -> Result<()> {
        info!("Loading PKGBUILD from {:?}", self.pkgbuild);
        let pkgbuild = fs::read(&self.pkgbuild).await?;
        let pkgbuild = pkgbuild::parse(&pkgbuild)?;

        info!("Loading file from {:?}", self.file);
        let content = fs::read(&self.file).await?;

        info!("Checking hashes");
        pkgbuild.has_artifact_by_checksum(&content)?;

        info!("File verified successfully");
        Ok(())
    }
}

/*
/// Authenticate a git tree by hash from a PKGBUILD
#[derive(Debug, Parser)]
pub struct ArchlinuxGitFromPkgbuild {}

impl ArchlinuxGitFromPkgbuild {
    fn run(&self) -> Result<()> {
        todo!()
    }
}

/// Authenticate a git tree from a source tarball
#[derive(Debug, Parser)]
pub struct GitFromTarball {}

impl GitFromTarball {
    fn run(&self) -> Result<()> {
        todo!()
    }
}
*/

/// Authenticate a pgp signed message
#[derive(Debug, Parser)]
pub struct PgpVerify {
    #[arg(long)]
    pub keyring: PathBuf,
    #[arg(long)]
    pub sig: PathBuf,
    pub file: PathBuf,
}

impl PgpVerify {
    async fn run(&self) -> Result<()> {
        info!("Loading keyring from {:?}", self.keyring);
        let keyring = fs::read(&self.keyring).await?;
        let keyring = pgp::keyring(&keyring)?;
        info!("Loaded {} public keys", keyring.len());

        info!("Loading signature from {:?}", self.sig);
        let sig = fs::read(&self.sig).await?;
        let sig = pgp::signature(&sig)?;

        info!("Loading message from {:?}", self.file);
        let msg = fs::read(&self.file).await?;

        pgp::verify(&keyring, &sig, &msg)?;
        info!("Message verified successfully");

        Ok(())
    }
}

/// Authenticate a Debian source index from a signed Debian release file
#[derive(Debug, Parser)]
pub struct DebianSourcesFromRelease {
    #[arg(long)]
    pub keyring: PathBuf,
    #[arg(long)]
    pub sig: PathBuf,
    #[arg(long)]
    pub release: PathBuf,
    pub sources: PathBuf,
}

impl DebianSourcesFromRelease {
    async fn run(&self) -> Result<()> {
        info!("Loading keyring from {:?}", self.keyring);
        let keyring = fs::read(&self.keyring)
            .await
            .with_context(|| anyhow!("Failed to load keyring from {:?}", self.keyring))?;
        let keyring = pgp::keyring(&keyring)?;
        info!("Loaded {} public keys", keyring.len());

        info!("Loading signature from {:?}", self.sig);
        let sig = fs::read(&self.sig)
            .await
            .with_context(|| anyhow!("Failed to load signatures from {:?}", self.sig))?;
        let sig = pgp::signature(&sig)?;

        info!("Loading release file from {:?}", self.release);
        let release = fs::read(&self.release)
            .await
            .with_context(|| anyhow!("Failed to load release file from {:?}", self.release))?;

        info!("Loading sources index from {:?}", self.sources);
        let sources = fs::read(&self.sources)
            .await
            .with_context(|| anyhow!("Failed to load sources index from {:?}", self.sources))?;

        // Verify release file signature
        pgp::verify(&keyring, &sig, &release)?;

        // Parse release, match with sources
        let release = apt::Release::parse(&release)?;

        debug!("Checking hash...");
        let sha256 = chksums::sha256(&sources);
        let _sources_entry = release.find_source_entry_by_sha256(&sha256)?;

        info!("Sources index verified successfully");
        Ok(())
    }
}

/// Authenticate a source tarball from a Debian source index
#[derive(Debug, Parser)]
pub struct DebianTarballFromSources {
    #[arg(long)]
    pub sources: PathBuf,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub orig: Option<PathBuf>,
    pub file: PathBuf,
}

impl DebianTarballFromSources {
    async fn run(&self) -> Result<()> {
        info!("Loading sources index from {:?}", self.sources);
        let sources = fs::read(&self.sources).await?;
        let sources = apt::SourcesIndex::parse(&sources)?;

        info!("Loading file from {:?}", self.file);
        let content = fs::read(&self.file).await?;

        let sha256 = if let Some(orig) = &self.orig {
            debug!("Decompressing file...");
            let content = compression::decompress(&content)?;

            info!("Loading Debian .orig.tar from {orig:?}");
            let orig = fs::read(orig).await?;
            let sha256 = chksums::sha256(&orig);
            let orig = compression::decompress(&orig)?;

            if orig != content {
                bail!("Decompressed file does match match decompressed Debian .orig.tar");
            }

            sha256
        } else {
            chksums::sha256(&content)
        };

        info!("Searching in index...");
        let _source_pkg =
            sources.find_pkg_by_sha256(self.name.as_deref(), self.version.as_deref(), &sha256)?;

        info!("File verified successfully");
        Ok(())
    }
}
