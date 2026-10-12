use super::{
    wire::{self, ProductReceipt},
    NativePackageReceiptError as Error, Result,
};
use chrono::{DateTime, Utc};
use ring::signature::{UnparsedPublicKey, ED25519};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    schema_version: u32,
    trust_registry_kind: String,
    issuers: Vec<Issuer>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Issuer {
    signer_id: String,
    algorithm: String,
    public_key_hex: String,
    disabled: bool,
}

/// External host policy; validity and package grants are not taken from candidate metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePackageKeyPolicy {
    pub signer_id: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub revoked: bool,
    pub allowed_package_ids: Vec<String>,
}
#[derive(Clone, Debug)]
struct TrustedIssuer {
    key: [u8; 32],
    disabled: bool,
    policy: NativePackageKeyPolicy,
}

#[derive(Clone, Debug)]
pub struct NativePackageReceiptTrust {
    issuers: HashMap<String, TrustedIssuer>,
    valid_until: DateTime<Utc>,
}
impl NativePackageReceiptTrust {
    pub(super) fn valid_until(&self) -> DateTime<Utc> {
        self.valid_until
    }
    /// Reads the existing trust-registry wire; adds explicit fresh host key policy.
    pub fn from_registry_json(
        bytes: &[u8],
        policies: Vec<NativePackageKeyPolicy>,
        valid_until: DateTime<Utc>,
        max_registry_bytes: usize,
    ) -> Result<Self> {
        if bytes.len() > max_registry_bytes {
            return Err(Error::Invalid("trust registry byte limit".into()));
        }
        let registry: Registry =
            serde_json::from_slice(bytes).map_err(|e| Error::Invalid(e.to_string()))?;
        if registry.schema_version != 1
            || registry.trust_registry_kind != "zircon_product_receipt_trust_registry"
            || registry.issuers.is_empty()
        {
            return Err(Error::Invalid("trust registry schema".into()));
        }
        let mut policy_map = HashMap::new();
        for policy in policies {
            wire::validate_signer(&policy.signer_id)?;
            if policy.not_after <= policy.not_before
                || policy.allowed_package_ids.is_empty()
                || policy
                    .allowed_package_ids
                    .iter()
                    .any(|id| id.trim().is_empty())
                || policy_map
                    .insert(policy.signer_id.clone(), policy)
                    .is_some()
            {
                return Err(Error::Invalid("invalid/duplicate host key policy".into()));
            }
        }
        let mut issuers = HashMap::new();
        for issuer in registry.issuers {
            wire::validate_signer(&issuer.signer_id)?;
            if issuer.algorithm != "ed25519-v1" {
                return Err(Error::Invalid("trust algorithm".into()));
            }
            let key = wire::decode_hex::<32>(&issuer.public_key_hex)?;
            let policy = policy_map
                .remove(&issuer.signer_id)
                .ok_or(Error::UntrustedIssuer)?;
            if issuers
                .insert(
                    issuer.signer_id,
                    TrustedIssuer {
                        key,
                        disabled: issuer.disabled,
                        policy,
                    },
                )
                .is_some()
            {
                return Err(Error::Invalid("duplicate trust issuer".into()));
            }
        }
        if !policy_map.is_empty() {
            return Err(Error::Invalid("host policy names unknown signer".into()));
        }
        Ok(Self {
            issuers,
            valid_until,
        })
    }
    pub(super) fn verify(
        &self,
        receipt: &ProductReceipt,
        now: DateTime<Utc>,
        package_id: &str,
    ) -> Result<DateTime<Utc>> {
        let issuer = self
            .issuers
            .get(&receipt.attestation.signer_id)
            .ok_or(Error::UntrustedIssuer)?;
        let created = receipt.created_at()?;
        if now >= self.valid_until
            || issuer.disabled
            || issuer.policy.revoked
            || now < issuer.policy.not_before
            || now >= issuer.policy.not_after
            || created < issuer.policy.not_before
            || created >= issuer.policy.not_after
            || !issuer
                .policy
                .allowed_package_ids
                .iter()
                .any(|id| id == package_id)
        {
            return Err(Error::UntrustedIssuer);
        }
        let payload = receipt.attestation_bytes()?;
        let signature = wire::decode_hex::<64>(&receipt.attestation.signature_hex)?;
        UnparsedPublicKey::new(&ED25519, issuer.key)
            .verify(&payload, &signature)
            .map_err(|_| Error::Signature)?;
        Ok(self.valid_until.min(issuer.policy.not_after))
    }
}
