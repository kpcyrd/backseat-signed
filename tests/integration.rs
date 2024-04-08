use backseat_signed::apt;
use backseat_signed::buildinfo;
use backseat_signed::chksums;
use backseat_signed::errors::*;
use backseat_signed::pgp;
use backseat_signed::pkgbuild;
use backseat_signed::rpm;
use std::io::Read;
use std::path::Path;

fn lz4_decompress(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut lz4 = lz4_flex::frame::FrameDecoder::new(bytes);

    let mut bytes = Vec::new();
    lz4.read_to_end(&mut bytes)?;

    Ok(bytes)
}

#[test]
fn parse_lookup_apt_release_xz() {
    let release = include_bytes!("data/vim/Release");
    let release = apt::Release::parse(release).unwrap();

    let sources = include_bytes!("data/vim/Sources.xz");
    let sha256 = chksums::sha256(sources);

    let _sources_entry = release.find_source_entry_by_sha256(&sha256).unwrap();
}

#[test]
fn parse_lookup_apt_release_gz() {
    let release = include_bytes!("data/vim/Release");
    let release = apt::Release::parse(release).unwrap();

    let sources = include_bytes!("data/vim/Sources.gz");
    let sha256 = chksums::sha256(sources);

    let _sources_entry = release.find_source_entry_by_sha256(&sha256).unwrap();
}

#[test]
fn parse_lookup_apt_release_plain() {
    let release = include_bytes!("data/vim/Release");
    let release = apt::Release::parse(release).unwrap();

    let sources = lz4_decompress(include_bytes!("data/vim/Sources.lz4")).unwrap();
    let sha256 = chksums::sha256(&sources);

    let _sources_entry = release.find_source_entry_by_sha256(&sha256).unwrap();
}

#[test]
fn parse_lookup_apt_sources_vim() {
    let sources = lz4_decompress(include_bytes!("data/vim/Sources.lz4")).unwrap();
    let sources = apt::SourcesIndex::parse(&sources).unwrap();

    let content = include_bytes!("data/vim/vim_9.1.0199.orig.tar.xz");
    let sha256 = chksums::sha256(content);

    let _source_pkg = sources.find_pkg_by_sha256(None, None, &sha256).unwrap();
}

#[test]
fn test_pgp_verify_vim() {
    let keyring = include_bytes!("data/debian-archive-bookworm-automatic.asc");
    let keyring = pgp::keyring(keyring).unwrap();

    let sig = include_bytes!("data/vim/Release.gpg");
    let sig = pgp::signature(sig).unwrap();

    let release = include_bytes!("data/vim/Release");
    pgp::verify(&keyring, &sig, release).unwrap();
}

#[test]
fn test_archlinux_verify_pkg_sig() {
    let keyring = include_bytes!("data/kpcyrd.asc");
    let keyring = pgp::keyring(keyring).unwrap();

    let sig = include_bytes!("data/cmatrix/cmatrix-2.0-3-x86_64.pkg.tar.zst.sig");
    let sig = pgp::signature(sig).unwrap();

    let pkg = include_bytes!("data/cmatrix/cmatrix-2.0-3-x86_64.pkg.tar.zst");
    pgp::verify(&keyring, &sig, pkg).unwrap();
}

#[test]
fn test_archlinux_pkgbuild_hash() {
    let pkg = include_bytes!("data/cmatrix/cmatrix-2.0-3-x86_64.pkg.tar.zst");

    let buildinfo = buildinfo::from_archlinux_pkg(pkg).unwrap();
    let pkgbuild_sha256sum = buildinfo.pkgbuild_sha256sum;

    assert_eq!(
        pkgbuild_sha256sum,
        "6ed7af29ac762cca746c533046099708f78b0fe07b7bf6bc3a5e86df38c87180"
    );
}

#[test]
fn test_archlinux_pkgbuild_artifact() {
    let pkgbuild = include_bytes!("data/cmatrix/PKGBUILD");
    let pkgbuild = pkgbuild::parse(pkgbuild).unwrap();

    let content = include_bytes!("data/cmatrix/cmatrix-2.0.tar.gz");
    pkgbuild.has_artifact_by_checksum(content).unwrap();
}

#[test]
fn test_rpm_tarball_from_src_rpm() {
    let pkg = include_bytes!("data/cmatrix/cmatrix-2.0-9.fc40.src.rpm");
    let pkg = rpm::Rpm::parse(pkg).unwrap();

    let content = include_bytes!("data/cmatrix/cmatrix-2.0.tar.gz");

    let sha256 = chksums::sha256(content);
    let path = pkg.has_artifact_by_sha256(&sha256).unwrap();
    assert_eq!(path, Path::new("cmatrix-2.0.tar.gz"));
}
