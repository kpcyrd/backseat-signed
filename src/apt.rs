use crate::errors::*;
use apt_parser::release::ReleaseHash;

#[derive(Debug, Default)]
pub struct Source {
    pub package: String,
    pub version: Option<String>,
    pub checksums_sha256: Vec<ReleaseHash>,
}

pub fn parse_sources(mut bytes: &[u8]) -> Result<Vec<Source>> {
    let mut buf = Vec::new();
    lzma_rs::xz_decompress(&mut bytes, &mut buf).context("Failed to decompress sources index")?;
    let sources = String::from_utf8(buf)?;

    let mut out = Vec::new();

    let mut package = None;
    let mut in_checksums_sha256_section = false;
    for line in sources.lines() {
        if let Some(value) = line.strip_prefix("Package: ") {
            package = Some(Source {
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
                out.push(package);
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

    Ok(out)
}
