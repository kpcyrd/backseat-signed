use backseat_signed::apt;
use backseat_signed::chksums;
use backseat_signed::errors::*;
use backseat_signed::pgp;
use std::io::Read;

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
