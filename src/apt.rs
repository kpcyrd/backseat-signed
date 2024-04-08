use crate::compression;
use crate::errors::*;
use apt_parser::release::ReleaseHash;
use std::str;

pub struct SourcesIndex {
    pkgs: Vec<SourcePkg>,
}

impl SourcesIndex {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let buf = compression::decompress(bytes).context("Failed to decompress sources index")?;
        let sources = str::from_utf8(&buf)?;

        let mut pkgs = Vec::new();

        let mut package = None;
        let mut in_checksums_sha256_section = false;
        for line in sources.lines() {
            if let Some(value) = line.strip_prefix("Package: ") {
                package = Some(SourcePkg {
                    package: value.to_string(),
                    ..Default::default()
                });
            } else if let Some(value) = line.strip_prefix("Version: ") {
                let Some(package) = package.as_mut() else {
                    continue;
                };
                package.version = Some(value.to_string());
            } else if line.is_empty() {
                if let Some(package) = package.take() {
                    pkgs.push(package);
                }
            } else if line == "Checksums-Sha256:" {
                in_checksums_sha256_section = true;
            } else if let Some(line) = line.strip_prefix(' ') {
                if !in_checksums_sha256_section {
                    continue;
                }

                let Some(package) = package.as_mut() else {
                    continue;
                };

                let (hash, line) = line
                    .split_once(' ')
                    .with_context(|| anyhow!("Failed to remove hash from line: {line:?}"))?;

                let (size, filename) = line
                    .split_once(' ')
                    .with_context(|| anyhow!("Failed to remove hash from line: {line:?}"))?;
                let size = size
                    .parse()
                    .with_context(|| anyhow!("Failed to parse size as number: {size:?}"))?;

                package.checksums_sha256.push(ReleaseHash {
                    hash: hash.to_string(),
                    size,
                    filename: filename.to_string(),
                });
            } else {
                in_checksums_sha256_section = false;
            }
        }

        Ok(SourcesIndex { pkgs })
    }

    pub fn find_pkg_by_sha256(
        &self,
        filter_name: Option<&str>,
        filter_version: Option<&str>,
        sha256: &str,
    ) -> Result<&SourcePkg> {
        for pkg in &self.pkgs {
            trace!("Found package in sources index: {pkg:?}");

            if let Some(name) = filter_name {
                if pkg.package != *name {
                    trace!("Skipping due to package name mismatch");
                    continue;
                }
            }

            if let Some(version) = filter_version {
                if pkg.version.as_deref() != Some(version) {
                    trace!("Skipping due to package version mismatch");
                    continue;
                }
            }

            for chksum in &pkg.checksums_sha256 {
                if !chksum.filename.ends_with(".orig.tar.gz")
                    && !chksum.filename.ends_with(".orig.tar.xz")
                {
                    continue;
                }

                if chksum.hash == sha256 {
                    info!("File verified successfully");
                    return Ok(pkg);
                }
            }
        }

        bail!("Could not find source tarball with matching hash in source index")
    }
}

#[derive(Debug, Default)]
pub struct SourcePkg {
    pub package: String,
    pub version: Option<String>,
    pub checksums_sha256: Vec<ReleaseHash>,
}

pub struct Release {
    release: apt_parser::Release,
}

impl Release {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let release = str::from_utf8(bytes)?;
        let release = apt_parser::Release::from(release)?;
        Ok(Release { release })
    }

    pub fn find_source_entry_by_sha256(&self, sha256: &str) -> Result<&ReleaseHash> {
        let sha256sums = self
            .release
            .sha256sum
            .as_ref()
            .context("Release file has no sha256sum section")?;

        let sources_entry = sha256sums
            .iter()
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

        Ok(sources_entry)
    }
}
