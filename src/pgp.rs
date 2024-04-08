use crate::errors::*;
use sequoia_openpgp::cert::prelude::*;
use sequoia_openpgp::packet::Signature;
use sequoia_openpgp::parse::{PacketParser, PacketParserResult, Parse};
use sequoia_openpgp::types::SignatureType;
use sequoia_openpgp::Packet;
use sequoia_openpgp::{Cert, Fingerprint};

pub fn keyring(bytes: &[u8]) -> Result<Vec<SigningKey>> {
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

pub fn signature(bytes: &[u8]) -> Result<Vec<Signature>> {
    let mut sigs = Vec::new();

    let mut ppr = PacketParser::from_bytes(&bytes)?;
    while let PacketParserResult::Some(pp) = ppr {
        let (packet, next_ppr) = pp.recurse()?;
        ppr = next_ppr;
        debug!("Found packet in signature block: {packet:?}");
        if let Packet::Signature(sig) = packet {
            sigs.push(sig);
        }
    }

    if sigs.is_empty() {
        bail!("Failed to decode any pgp signatures")
    }

    Ok(sigs)
}

pub struct SigningKey {
    cert: Cert,
}

pub fn verify(keyring: &[SigningKey], sigs: &[Signature], msg: &[u8]) -> Result<Fingerprint> {
    for sig in sigs {
        let body = match sig.typ() {
            SignatureType::Binary => msg,
            other => bail!("Signature type is currently not supported: {other:?}"),
        };

        for pubkey in keyring {
            for key in pubkey.cert.keys() {
                let key = key.key();

                let key_fp = key.fingerprint();
                debug!("Attempting verification with {:X}", key_fp);

                match sig.clone().verify_message(key, body) {
                    Ok(_) => {
                        debug!("Successfully verified signature");
                        return Ok(key_fp);
                    }
                    Err(err) => {
                        debug!("Signature verification failed: {err:#}");
                    }
                }
            }
        }
    }

    bail!("Failed to verify message")
}
