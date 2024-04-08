use crate::errors::*;
use rpm::{DigestAlgorithm, Package};
use std::path::PathBuf;

pub struct Rpm {
    pkg: rpm::Package,
}

impl Rpm {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let pkg = Package::parse(&mut &bytes[..])?;
        Ok(Rpm { pkg })
    }

    pub fn has_artifact_by_sha256(&self, sha256: &str) -> Result<PathBuf> {
        for entry in self.pkg.metadata.get_file_entries()? {
            let path = entry.path;
            let Some(digest) = entry.digest else {
                debug!("Found entry {path:?}: no digest");
                continue;
            };
            debug!("Found entry {path:?}: digest={digest:?}");

            if digest.algorithm() != DigestAlgorithm::Sha2_256 {
                continue;
            }

            if digest.as_hex() == sha256 {
                debug!("Found matching entry in source rpm");
                return Ok(path);
            }
        }

        bail!("Could not find source tarball with matching hash in source rpm")
    }
}
