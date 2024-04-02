use crate::errors::*;
use sequoia_openpgp::cert::prelude::*;
use sequoia_openpgp::packet::Signature;
use sequoia_openpgp::parse::{PacketParser, PacketParserResult, Parse};
use sequoia_openpgp::types::SignatureType;
use sequoia_openpgp::Packet;
use sequoia_openpgp::{Cert, Fingerprint};

pub fn pubkey(bytes: &[u8]) -> Result<Vec<SigningKey>> {
    let mut keys = Vec::new();

    let ppr = PacketParser::from_bytes(bytes)?;
    for certo in CertParser::from(ppr) {
        let cert = certo.context("Error reading pgp key")?;
        keys.push(SigningKey { cert });
    }

    if keys.is_empty() {
        bail!("Failed to find any pgp public keys")
    }

    Ok(keys)
}

pub fn signature(bytes: &[u8]) -> Result<Signature> {
    let mut ppr = PacketParser::from_bytes(&bytes)?;
    while let PacketParserResult::Some(pp) = ppr {
        let (packet, next_ppr) = pp.recurse()?;
        ppr = next_ppr;
        debug!("Found packet in signature block: {packet:?}");
        if let Packet::Signature(sig) = packet {
            return Ok(sig);
        }
    }
    bail!("Failed to decode pgp signature")
}

pub struct SigningKey {
    cert: Cert,
}

pub fn verify(keyring: &[SigningKey], sig: &Signature, msg: &[u8]) -> Result<Fingerprint> {
    let body = match sig.typ() {
        SignatureType::Binary => msg,
        other => bail!("Signature type is currently not supported: {other:?}"),
    };

    for pubkey in keyring {
        for key in pubkey.cert.keys() {
            let key = key.key();

            let key_fp = key.fingerprint();
            debug!("Attempting verification with {:X}", key_fp);

            if sig.clone().verify_message(key, body).is_ok() {
                debug!("Successfully verified signature");
                return Ok(key_fp);
            }
        }
    }

    bail!("Failed to verify message")
}
