use crate::errors::*;
use bstr::ByteSlice;
use ruzstd::decoding::StreamingDecoder;
use std::io::Read;
use std::str;

#[derive(Debug, PartialEq)]
pub struct ArchLinuxBuildinfo {
    pub pkgbuild_sha256sum: String,
}

pub fn parse_archlinux(bytes: &[u8]) -> Result<ArchLinuxBuildinfo> {
    for line in bytes.lines() {
        trace!("Line in .BUILDINFO: {:?}", bstr::BStr::new(line));
        let line = str::from_utf8(line)?;

        if let Some(hash) = line.strip_prefix("pkgbuild_sha256sum = ") {
            return Ok(ArchLinuxBuildinfo {
                pkgbuild_sha256sum: hash.to_string(),
            });
        }
    }
    bail!("Failed to find pkgbuild_sha256sum in .BUILDINFO")
}

pub fn from_archlinux_pkg(bytes: &[u8]) -> Result<ArchLinuxBuildinfo> {
    let decoder = StreamingDecoder::new(bytes)?;
    let mut tar = tar::Archive::new(decoder);

    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.to_str() != Some(".BUILDINFO") {
            debug!("Skipping file in package: {path:?}");
            continue;
        }
        debug!("Found .BUILDINFO file in package");
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        return parse_archlinux(&buf);
    }

    bail!("Failed to locate .BUILDINFO in package")
}
