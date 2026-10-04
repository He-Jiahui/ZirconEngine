//! 生产签发密钥和独立信任注册表。
//! CLI 草案签发先由 PKCS8 密钥签名，再用信任注册表复核；验收命令仅用注册表公钥，不把持有私钥等同于公开信任。

use std::borrow::Cow;
use std::collections::{hash_map::Entry, HashMap};
use std::sync::OnceLock;

use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use serde::Deserialize;

use super::canonical::bytes_to_hex;
use super::{ProductReceiptError, ProductReceiptSigner, ProductReceiptVerifier};

pub const ED25519_RECEIPT_ALGORITHM: &str = "ed25519-v1";
const TRUST_REGISTRY_SCHEMA_VERSION: u32 = 1;
const TRUST_REGISTRY_KIND: &str = "zircon_product_receipt_trust_registry";
const ED25519_PUBLIC_KEY_LENGTH: usize = 32;

/// 本地签发能力，私钥只用于签发与签发后自检；公开验收应交给信任注册表。
pub struct Ed25519ProductReceiptSigner {
    signer_id: String,
    key_pair: Ed25519KeyPair,
    public_key_hex: OnceLock<String>,
}

impl Ed25519ProductReceiptSigner {
    pub fn from_pkcs8(
        signer_id: impl Into<String>,
        private_key_pkcs8: &[u8],
    ) -> Result<Self, ProductReceiptError> {
        let signer_id = signer_id.into();
        validate_signer_id(&signer_id)?;
        let key_pair = Ed25519KeyPair::from_pkcs8(private_key_pkcs8).map_err(|error| {
            ProductReceiptError::new(format!(
                "could not load product receipt Ed25519 private key: {error}"
            ))
        })?;
        Ok(Self {
            signer_id,
            key_pair,
            public_key_hex: OnceLock::new(),
        })
    }

    pub fn public_key_hex(&self) -> &str {
        self.public_key_hex
            .get_or_init(|| bytes_to_hex(self.key_pair.public_key().as_ref()))
    }
}

impl ProductReceiptSigner for Ed25519ProductReceiptSigner {
    fn signer_id(&self) -> &str {
        &self.signer_id
    }

    fn algorithm(&self) -> &str {
        ED25519_RECEIPT_ALGORITHM
    }

    fn sign(&self, attestation_payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(self.key_pair.sign(attestation_payload).as_ref().to_vec())
    }
}

impl ProductReceiptVerifier for Ed25519ProductReceiptSigner {
    fn verify(
        &self,
        signer_id: &str,
        algorithm: &str,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if signer_id != self.signer_id {
            return Err(ProductReceiptError::new(format!(
                "product receipt signer `{signer_id}` does not match issuing key `{}`",
                self.signer_id
            ))
            .into());
        }
        if algorithm != ED25519_RECEIPT_ALGORITHM {
            return Err(ProductReceiptError::new(format!(
                "product receipt algorithm `{algorithm}` is not supported by the issuing key"
            ))
            .into());
        }
        UnparsedPublicKey::new(&ED25519, self.key_pair.public_key().as_ref())
            .verify(attestation_payload, signature)
            .map_err(|_| {
                ProductReceiptError::new("product receipt Ed25519 signature is invalid")
            })?;
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustRegistryDocument<'a> {
    schema_version: u32,
    #[serde(borrow)]
    trust_registry_kind: Cow<'a, str>,
    #[serde(borrow)]
    issuers: Vec<TrustedIssuerDocument<'a>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustedIssuerDocument<'a> {
    signer_id: String,
    #[serde(borrow)]
    algorithm: Cow<'a, str>,
    #[serde(borrow)]
    public_key_hex: Cow<'a, str>,
    disabled: bool,
}

struct TrustedIssuer {
    public_key: [u8; ED25519_PUBLIC_KEY_LENGTH],
    disabled: bool,
}

enum TrustedIssuers {
    Single {
        signer_id: String,
        issuer: TrustedIssuer,
    },
    Multiple(HashMap<String, TrustedIssuer>),
}

/// 独立验收方的签发者白名单；禁用项和未知签发者都不能通过证明核验。
pub struct ProductReceiptTrustRegistry {
    issuers: TrustedIssuers,
}

impl ProductReceiptTrustRegistry {
    pub fn from_json(bytes: &[u8]) -> Result<Self, ProductReceiptError> {
        let document: TrustRegistryDocument<'_> =
            serde_json::from_slice(bytes).map_err(|error| {
                ProductReceiptError::new(format!(
                    "could not parse product receipt trust registry: {error}"
                ))
            })?;
        if document.schema_version != TRUST_REGISTRY_SCHEMA_VERSION
            || document.trust_registry_kind != TRUST_REGISTRY_KIND
        {
            return Err(ProductReceiptError::new(
                "product receipt trust registry has an unsupported schema or kind",
            ));
        }
        if document.issuers.is_empty() {
            return Err(ProductReceiptError::new(
                "product receipt trust registry must contain at least one issuer",
            ));
        }

        let issuers = if document.issuers.len() == 1 {
            let issuer = document.issuers.into_iter().next().expect("issuer exists");
            let signer_id = issuer.signer_id;
            let issuer = decode_trusted_issuer(
                &signer_id,
                issuer.algorithm,
                issuer.public_key_hex,
                issuer.disabled,
            )?;
            TrustedIssuers::Single { signer_id, issuer }
        } else {
            let mut issuers = HashMap::with_capacity(document.issuers.len());
            for issuer in document.issuers {
                let signer_id = issuer.signer_id;
                match issuers.entry(signer_id) {
                    Entry::Occupied(entry) => {
                        return Err(ProductReceiptError::new(format!(
                            "product receipt trust registry contains duplicate signer `{}`",
                            entry.key()
                        )));
                    }
                    Entry::Vacant(entry) => {
                        let issuer = decode_trusted_issuer(
                            entry.key(),
                            issuer.algorithm,
                            issuer.public_key_hex,
                            issuer.disabled,
                        )?;
                        entry.insert(issuer);
                    }
                }
            }
            TrustedIssuers::Multiple(issuers)
        };
        Ok(Self { issuers })
    }
}

fn decode_trusted_issuer(
    signer_id: &str,
    algorithm: Cow<'_, str>,
    public_key_hex: Cow<'_, str>,
    disabled: bool,
) -> Result<TrustedIssuer, ProductReceiptError> {
    validate_signer_id(signer_id)?;
    if algorithm != ED25519_RECEIPT_ALGORITHM {
        return Err(ProductReceiptError::new(format!(
            "product receipt trust registry issuer `{signer_id}` uses unsupported algorithm `{algorithm}`"
        )));
    }
    Ok(TrustedIssuer {
        public_key: decode_public_key(&public_key_hex)?,
        disabled,
    })
}

fn validate_signer_id(value: &str) -> Result<(), ProductReceiptError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || bytes.len() > 128
        || !bytes[0].is_ascii_lowercase() && !bytes[0].is_ascii_digit()
        || !bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        return Err(ProductReceiptError::new(format!(
            "product receipt signer id `{value}` must be one stable lowercase identifier"
        )));
    }
    Ok(())
}

impl ProductReceiptVerifier for ProductReceiptTrustRegistry {
    fn verify(
        &self,
        signer_id: &str,
        algorithm: &str,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let issuer = match &self.issuers {
            TrustedIssuers::Single {
                signer_id: trusted_signer_id,
                issuer,
            } if trusted_signer_id == signer_id => Some(issuer),
            TrustedIssuers::Single { .. } => None,
            TrustedIssuers::Multiple(issuers) => issuers.get(signer_id),
        }
        .ok_or_else(|| {
            ProductReceiptError::new(format!(
                "product receipt signer `{signer_id}` is not trusted"
            ))
        })?;
        if issuer.disabled {
            return Err(ProductReceiptError::new(format!(
                "product receipt signer `{signer_id}` is disabled"
            ))
            .into());
        }
        if algorithm != ED25519_RECEIPT_ALGORITHM {
            return Err(ProductReceiptError::new(format!(
                "product receipt signer `{signer_id}` does not trust algorithm `{algorithm}`"
            ))
            .into());
        }
        UnparsedPublicKey::new(&ED25519, issuer.public_key)
            .verify(attestation_payload, signature)
            .map_err(|_| {
                ProductReceiptError::new("product receipt Ed25519 signature is invalid")
            })?;
        Ok(())
    }
}

fn decode_public_key(value: &str) -> Result<[u8; ED25519_PUBLIC_KEY_LENGTH], ProductReceiptError> {
    if value.len() != ED25519_PUBLIC_KEY_LENGTH * 2 {
        return Err(ProductReceiptError::new(
            "product receipt trusted Ed25519 public key must be 32 bytes of hexadecimal",
        ));
    }
    let mut decoded = [0_u8; ED25519_PUBLIC_KEY_LENGTH];
    for (target, pair) in decoded.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
        let high = decode_hex_nibble(pair[0]);
        let low = decode_hex_nibble(pair[1]);
        let (Some(high), Some(low)) = (high, low) else {
            return Err(ProductReceiptError::new(
                "product receipt trusted Ed25519 public key must be hexadecimal",
            ));
        };
        *target = (high << 4) | low;
    }
    Ok(decoded)
}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/ed25519_authority.rs"]
mod tests;

#[cfg(test)]
#[path = "ed25519_authority/tests/performance_tests.rs"]
mod performance_tests;
