use backseat_signed::apt;
use backseat_signed::buildinfo;
use backseat_signed::chksums;
use backseat_signed::errors::*;
use backseat_signed::pgp;
use backseat_signed::pkgbuild;
use std::io::Read;
use std::process::Command;

fn lz4_decompress(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut lz4 = lz4_flex::frame::FrameDecoder::new(bytes);

    let mut bytes = Vec::new();
    lz4.read_to_end(&mut bytes)?;

    Ok(bytes)
}

fn git_integration_data(path: &str, sha256: &str) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("show")
        .arg(format!("origin/integration-data:tests/{path}"))
        .output()?;
    if !output.status.success() {
        bail!("Failed to get test-data file from git: {path:?}");
    }
    let data = output.stdout;

    let hashed = chksums::sha256(&data);
    if hashed != sha256 {
        bail!("Mismatching hash for {path:?}, expected={sha256:?}, got={hashed:?}");
    }

    Ok(data)
}

#[test]
fn parse_lookup_apt_release_xz() {
    let release = git_integration_data(
        "data/vim/Release",
        "0bba2751e8ab74cf19c628db12e921d8753be857c77a54652fb0a25767bef92a",
    )
    .unwrap();
    let release = apt::Release::parse(&release).unwrap();

    let sources = git_integration_data(
        "data/vim/Sources.xz",
        "ba14ca35563ace9dc1e81446f6d72979cdc5aa7ea5c558cb0fe5071736c602b2",
    )
    .unwrap();
    let sha256 = chksums::sha256(&sources);

    let _sources_entry = release.find_source_entry_by_sha256(&sha256).unwrap();
}

#[test]
fn parse_lookup_apt_release_gz() {
    let release = git_integration_data(
        "data/vim/Release",
        "0bba2751e8ab74cf19c628db12e921d8753be857c77a54652fb0a25767bef92a",
    )
    .unwrap();
    let release = apt::Release::parse(&release).unwrap();

    let sources = git_integration_data(
        "data/vim/Sources.gz",
        "4fff7a4e41c2f22240d3905de0eb85640082b74f544d66cfafef277d6c5a8b14",
    )
    .unwrap();
    let sha256 = chksums::sha256(&sources);

    let _sources_entry = release.find_source_entry_by_sha256(&sha256).unwrap();
}

#[test]
fn parse_lookup_apt_release_plain() {
    let release = git_integration_data(
        "data/vim/Release",
        "0bba2751e8ab74cf19c628db12e921d8753be857c77a54652fb0a25767bef92a",
    )
    .unwrap();
    let release = apt::Release::parse(&release).unwrap();

    let sources = lz4_decompress(
        &git_integration_data(
            "data/vim/Sources.lz4",
            "0d4a5448832d6f75f25be884cda2b791ff36390d3c413f0753f5baf78afc538f",
        )
        .unwrap(),
    )
    .unwrap();
    let sha256 = chksums::sha256(&sources);

    let _sources_entry = release.find_source_entry_by_sha256(&sha256).unwrap();
}

#[test]
fn parse_lookup_apt_sources_vim() {
    let sources = lz4_decompress(
        &git_integration_data(
            "data/vim/Sources.lz4",
            "0d4a5448832d6f75f25be884cda2b791ff36390d3c413f0753f5baf78afc538f",
        )
        .unwrap(),
    )
    .unwrap();
    let sources = apt::SourcesIndex::parse(&sources).unwrap();

    let content = git_integration_data(
        "data/vim/vim_9.1.0199.orig.tar.xz",
        "a3284e44b55a7877f3b0bbb1b0a349748e3b48f9d1e1c9d0f93856f7be417dda",
    )
    .unwrap();
    let sha256 = chksums::sha256(&content);

    let _source_pkg = sources.find_pkg_by_sha256(None, None, &sha256).unwrap();
}

#[test]
fn test_pgp_verify_vim() {
    let keyring = git_integration_data(
        "data/debian-archive-bookworm-automatic.asc",
        "c2a9a16fde95e037bafd0fa6b7e31f41b4ff1e85851de5558f19a2a2f0e955e2",
    )
    .unwrap();
    let keyring = pgp::keyring(&keyring).unwrap();

    let sig = git_integration_data(
        "data/vim/Release.gpg",
        "9699b97cfcdcadd083c099683ec7dd009a9f5315cd62c429c136ed7a1d96ed0b",
    )
    .unwrap();
    let sig = pgp::signature(&sig).unwrap();

    let release = git_integration_data(
        "data/vim/Release",
        "0bba2751e8ab74cf19c628db12e921d8753be857c77a54652fb0a25767bef92a",
    )
    .unwrap();
    pgp::verify(&keyring, &sig, &release).unwrap();
}

#[test]
fn test_archlinux_verify_pkg_sig() {
    let keyring = git_integration_data(
        "data/kpcyrd.asc",
        "4cc0ddd01c958b6a9fc0eb689b581f088d9c5a74b7ac5ba72594c1c85c09ce32",
    )
    .unwrap();
    let keyring = pgp::keyring(&keyring).unwrap();

    let sig = git_integration_data(
        "data/cmatrix/cmatrix-2.0-3-x86_64.pkg.tar.zst.sig",
        "ce8fe71e99503512f09beddcfd4ae8961037f6ada0bed2f731080c02806dc8ed",
    )
    .unwrap();
    let sig = pgp::signature(&sig).unwrap();

    let pkg = git_integration_data(
        "data/cmatrix/cmatrix-2.0-3-x86_64.pkg.tar.zst",
        "03a7237192794b7789cb40640b151fffa77d832ab0d33bed8778a6d569f0f8ca",
    )
    .unwrap();
    pgp::verify(&keyring, &sig, &pkg).unwrap();
}

#[test]
fn test_archlinux_pkgbuild_hash() {
    let pkg = git_integration_data(
        "data/cmatrix/cmatrix-2.0-3-x86_64.pkg.tar.zst",
        "03a7237192794b7789cb40640b151fffa77d832ab0d33bed8778a6d569f0f8ca",
    )
    .unwrap();

    let buildinfo = buildinfo::from_archlinux_pkg(&pkg).unwrap();
    let pkgbuild_sha256sum = buildinfo.pkgbuild_sha256sum;

    assert_eq!(
        pkgbuild_sha256sum,
        "6ed7af29ac762cca746c533046099708f78b0fe07b7bf6bc3a5e86df38c87180"
    );
}

#[test]
fn test_archlinux_pkgbuild_artifact() {
    let pkgbuild = git_integration_data(
        "data/cmatrix/PKGBUILD",
        "6ed7af29ac762cca746c533046099708f78b0fe07b7bf6bc3a5e86df38c87180",
    )
    .unwrap();
    let pkgbuild = pkgbuild::parse(&pkgbuild).unwrap();

    let pkg = git_integration_data(
        "data/cmatrix/cmatrix-2.0.tar.gz",
        "ad93ba39acd383696ab6a9ebbed1259ecf2d3cf9f49d6b97038c66f80749e99a",
    )
    .unwrap();
    pkgbuild.has_artifact_by_checksum(&pkg).unwrap();
}
