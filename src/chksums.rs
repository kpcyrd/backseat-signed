use digest::Digest;

pub fn sha256(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn sha512(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha512::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn blake2b(bytes: &[u8]) -> String {
    let mut hasher = blake2::Blake2b512::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_sha512() {
        let hash = sha512(b"hello world");
        assert_eq!(hash, "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f");
    }

    #[test]
    fn test_blake2() {
        let hash = blake2b(b"hello world");
        assert_eq!(hash, "021ced8799296ceca557832ab941a50b4a11f83478cf141f51f933f653ab9fbcc05a037cddbed06e309bf334942c4e58cdf1a46e237911ccd7fcf9787cbc7fd0");
    }
}
