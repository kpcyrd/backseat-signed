use crate::apt;
use crate::buildinfo;
use crate::chksums;
use crate::compression;
use crate::errors::*;
use crate::pgp;
use crate::pkgbuild;
use apt_parser::Release;
use clap::{Parser, Subcommand};
use std::ops::Not;
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
        let keyring = pgp::pubkey(&keyring)?;
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
        let sha256 = pkgbuild
            .sha256sums
            .is_empty()
            .not()
            .then(|| chksums::sha256(&content));
        let sha512 = pkgbuild
            .sha512sums
            .is_empty()
            .not()
            .then(|| chksums::sha512(&content));
        let blake2b = pkgbuild
            .b2sums
            .is_empty()
            .not()
            .then(|| chksums::blake2b(&content));

        if pkgbuild.has_match_for_checksums(
            sha256.as_deref(),
            sha512.as_deref(),
            blake2b.as_deref(),
        ) {
            info!("File verified successfully");
            Ok(())
        } else {
            bail!("PKGBUILD does not seem to have any matching sources, sha256={sha256:?}, sha512={sha512:?}, blake2b={blake2b:?}")
        }
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
        let keyring = pgp::pubkey(&keyring)?;
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
        let keyring = pgp::pubkey(&keyring)?;
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
        let release = String::from_utf8(release)?;
        let release = Release::from(&release)?;

        debug!("Checking hash...");
        let sha256 = chksums::sha256(&sources);

        let sha256sums = release
            .sha256sum
            .context("Release file has no sha256sum section")?;
        let _sources_entry = sha256sums
            .into_iter()
            .filter(|entry| entry.filename.contains("/source/Sources"))
            .find(|entry| {
                debug!("Found sha256sum entry for sources index: {entry:?}");
                entry.hash == sha256
            })
            .with_context(|| {
                anyhow!(
                    "Failed to find matching source entry in release file with sha256={sha256:?}"
                )
            })?;

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
        let sources = apt::parse_sources(&sources)?;

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
        for pkg in sources {
            trace!("Found package in sources index: {pkg:?}");

            if let Some(name) = &self.name {
                if pkg.package != *name {
                    trace!("Skipping due to package name mismatch");
                    continue;
                }
            }

            if let Some(version) = &self.version {
                if pkg.version.as_ref() != Some(version) {
                    trace!("Skipping due to package version mismatch");
                    continue;
                }
            }

            for chksum in pkg.checksums_sha256 {
                if !chksum.filename.ends_with(".orig.tar.gz")
                    && !chksum.filename.ends_with(".orig.tar.xz")
                {
                    continue;
                }

                if chksum.hash == sha256 {
                    info!("File verified successfully");
                    return Ok(());
                }
            }
        }

        bail!("Could not find source tarball with matching hash in source index")
    }
}
