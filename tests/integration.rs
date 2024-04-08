use backseat_signed::apt;
use backseat_signed::chksums;
use backseat_signed::pgp;
use std::io::Read;

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

    let sources = include_bytes!("data/vim/Sources.lz4");
    let mut lz4 = lz4_flex::frame::FrameDecoder::new(&sources[..]);

    let mut sources = Vec::new();
    lz4.read_to_end(&mut sources).unwrap();
    let sha256 = chksums::sha256(&sources);

    let _sources_entry = release.find_source_entry_by_sha256(&sha256).unwrap();
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
