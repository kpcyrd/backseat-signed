use crate::errors::*;
use libflate::gzip::Decoder;
use std::borrow::Cow;
use std::io::Read;

pub const GZIP_MAGIC: &[u8] = &[0x1f, 0x8b];
pub const LZMA_MAGIC: &[u8] = &[0xfd, 0x37, 0x7a, 0x58];

pub fn decompress(mut bytes: &[u8]) -> Result<Cow<[u8]>> {
    if bytes.starts_with(GZIP_MAGIC) {
        debug!("Detected gzip compression");
        let mut decoder = Decoder::new(bytes)?;
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf)?;
        Ok(Cow::Owned(buf))
    } else if bytes.starts_with(LZMA_MAGIC) {
        debug!("Detected lzma compression");
        let mut buf = Vec::new();
        lzma_rs::xz_decompress(&mut bytes, &mut buf)?;
        Ok(Cow::Owned(buf))
    } else {
        debug!("Did not detect any compression, using as tar directly");
        Ok(Cow::Borrowed(bytes))
    }
}
