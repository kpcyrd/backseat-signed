use crate::errors::*;
use std::str;
use yash_syntax::syntax::{self, TextUnit, Value, WordUnit};

#[derive(Debug, Default, PartialEq)]
pub struct Pkgbuild {
    pub sha256sums: Vec<String>,
    pub sha512sums: Vec<String>,
    pub b2sums: Vec<String>,
}

impl Pkgbuild {
    pub fn has_match_for_checksums(
        &self,
        sha256: Option<&str>,
        sha512: Option<&str>,
        blake2b: Option<&str>,
    ) -> bool {
        let Some(max) = [
            self.sha256sums.len(),
            self.sha512sums.len(),
            self.b2sums.len(),
        ]
        .into_iter()
        .max() else {
            return false;
        };

        for idx in 0..max {
            let cmps = [
                Self::compare_chksum(&self.sha256sums, idx, sha256),
                Self::compare_chksum(&self.sha512sums, idx, sha512),
                Self::compare_chksum(&self.b2sums, idx, blake2b),
            ];

            if cmps.iter().any(|c| *c == Compare::Mismatch) {
                continue;
            }

            if cmps.iter().any(|c| *c == Compare::StrongMatch) {
                info!("PKGBUILD has source= offset at #{idx:?} matching all checksums of artifact");
                return true;
            }
        }

        debug!("Could not find any matches in combined checksum arrays");
        false
    }

    pub fn compare_chksum(list: &[String], idx: usize, expected: Option<&str>) -> Compare {
        let Some(expected) = expected else {
            return Compare::WeakMatch;
        };
        match list.get(idx) {
            Some(value) if value == expected => Compare::StrongMatch,
            Some(value) if value == "SKIP" => Compare::WeakMatch,
            _ => Compare::Mismatch,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Compare {
    StrongMatch,
    WeakMatch,
    Mismatch,
}

pub fn parse(bytes: &[u8]) -> Result<Pkgbuild> {
    let script = str::from_utf8(bytes)?;
    let parsed: syntax::List = script
        .parse()
        .map_err(|err| anyhow!("Failed to parse input as shell script: {:#?}", err))?;

    let mut pkgbuild = Pkgbuild::default();

    for item in &parsed.0 {
        for cmd in &item.and_or.first.commands {
            let syntax::Command::Simple(cmd) = cmd.as_ref() else {
                continue;
            };
            for assign in &cmd.assigns {
                let name = assign.name.as_str();

                // handle bash-style `+=` assignments
                let name = name.strip_suffix('+').unwrap_or(name);
                debug!("Found assignment to {name:?}");

                let target = match name {
                    "sha256sums" => &mut pkgbuild.sha256sums,
                    "sha512sums" => &mut pkgbuild.sha512sums,
                    "b2sums" => &mut pkgbuild.b2sums,
                    _ => continue,
                };

                let Value::Array(values) = &assign.value else {
                    continue;
                };

                for value in values {
                    for unit in &value.units {
                        trace!("Found word unit: {unit:?}");

                        match unit {
                            WordUnit::SingleQuote(text) => target.push(text.to_string()),
                            WordUnit::DoubleQuote(text) => {
                                let mut s = String::new();
                                for unit in &text.0 {
                                    if let TextUnit::Literal(chr) = unit {
                                        s.push(*chr);
                                    }
                                }
                                target.push(s);
                            }
                            other => bail!("Unsupported word unit: {other:?}"),
                        }
                    }
                }
            }
        }
    }

    Ok(pkgbuild)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_parse() {
        init();

        let script = b"sha256sums=('7a1258a5dfc48c54cea1092adddb6bcfb1fcf19c7272c0a6a9e1d2d7daee6e12')
sha256sums+=(\"f9a4925f7d7bb7de54e17cd9ad7c584dfae88ad182d943b79cf403425000f128\")
b2sums=('cd594be73fcf632544195d09518901b1055ae86dcf463a5d446a83beba66073c70a9dfb75efd9d826c2ecf7215ab6cd76128a20104d5ef4ea57470061d2e29bf'
        'f4f89b720bcbe23c5413c6cbc2d0793d8e379fc53861a6fbd83f506e56a86132bb92236498b4357310b09e51fd05aa5ccc941649a4f205fb4e53cb6bc32cdd64')
";
        let pkgbuild = parse(script).unwrap();
        assert_eq!(
            pkgbuild,
            Pkgbuild {
                sha256sums: vec![
                    "7a1258a5dfc48c54cea1092adddb6bcfb1fcf19c7272c0a6a9e1d2d7daee6e12".to_string(),
                    "f9a4925f7d7bb7de54e17cd9ad7c584dfae88ad182d943b79cf403425000f128".to_string(),
                ],
                sha512sums: vec![],
                b2sums: vec![
                    "cd594be73fcf632544195d09518901b1055ae86dcf463a5d446a83beba66073c70a9dfb75efd9d826c2ecf7215ab6cd76128a20104d5ef4ea57470061d2e29bf".to_string(),
                    "f4f89b720bcbe23c5413c6cbc2d0793d8e379fc53861a6fbd83f506e56a86132bb92236498b4357310b09e51fd05aa5ccc941649a4f205fb4e53cb6bc32cdd64".to_string(),
                ],
            }
        );
    }
}
