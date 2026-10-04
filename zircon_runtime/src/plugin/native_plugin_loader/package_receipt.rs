use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::native_artifact_trust::{
    NativePluginArtifactDependency, NativePluginArtifactDigest, NativePluginArtifactExpectation,
    NativePluginArtifactTarget, NativePluginArtifactTrust,
};
use crate::plugin::{PluginModuleKind, PluginPackageManifest};

#[cfg(test)]
#[path = "package_receipt/tests/cases.rs"]
mod tests;
mod trust;
mod wire;

pub use trust::{NativePackageKeyPolicy, NativePackageReceiptTrust};
use wire::{ProductReceipt, ReceiptArtifact};

/// Host-owned mapping from a manifest module to its signed artifact identity.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePackageModuleArtifact {
    pub module_kind: PluginModuleKind,
    pub logical_name: String,
    pub relative_path: String,
    pub dependencies: Vec<NativePackageDependencyArtifact>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePackageDependencyArtifact {
    pub logical_name: String,
    pub relative_path: String,
}

#[derive(Clone, Debug)]
pub struct NativePackageReceiptPolicy {
    pub plugin_id: String,
    pub package_id: String,
    pub package_version: String,
    pub sdk_api_version: String,
    pub build_set_id: String,
    pub target: NativePluginArtifactTarget,
    pub target_triple: String,
    pub manifest_logical_name: String,
    pub manifest_relative_path: String,
    pub modules: Vec<NativePackageModuleArtifact>,
    pub required_capabilities: Vec<String>,
    pub allowed_capabilities: Vec<String>,
    pub now: DateTime<Utc>,
    pub max_receipt_age_seconds: u64,
    pub max_receipt_bytes: usize,
    pub max_receipt_count: usize,
    pub max_manifest_bytes: usize,
}

/// Constructible only after cryptographic and package-policy verification.
#[derive(Debug)]
pub struct VerifiedNativePackageProof {
    expectations: Vec<NativePluginArtifactExpectation>,
    valid_until: DateTime<Utc>,
}

impl VerifiedNativePackageProof {
    /// Authenticated digests only; this does not grant native execution.
    pub fn artifact_expectations(&self) -> &[NativePluginArtifactExpectation] {
        &self.expectations
    }

    pub fn valid_until(&self) -> DateTime<Utc> {
        self.valid_until
    }

    pub(super) fn into_parts(self) -> (Vec<NativePluginArtifactExpectation>, DateTime<Utc>) {
        (self.expectations, self.valid_until)
    }
}

#[derive(Debug, Error)]
pub enum NativePackageReceiptError {
    #[error("invalid native package receipt: {0}")]
    Invalid(String),
    #[error("native package receipt signature is invalid")]
    Signature,
    #[error("native package receipt issuer is unknown, disabled, revoked or outside its validity window")]
    UntrustedIssuer,
    #[error("native package receipt is expired or issued in the future")]
    ReceiptTime,
    #[error(
        "native package receipt does not match required identity, target or capability policy: {0}"
    )]
    Policy(String),
    #[error("native package receipt artifact is missing, ambiguous or has different bytes: {0}")]
    Artifact(String),
}

type Result<T> = std::result::Result<T, NativePackageReceiptError>;

/// Verifies signed metadata; the loader still hashes its opened DLL before native load.
/// No path from a candidate is opened or trusted by this pure verifier.
pub fn verify_native_package_receipts(
    receipt_documents: &[&[u8]],
    manifest_bytes: &[u8],
    trust: &NativePackageReceiptTrust,
    policy: &NativePackageReceiptPolicy,
) -> Result<VerifiedNativePackageProof> {
    if policy.modules.is_empty()
        || receipt_documents.is_empty()
        || receipt_documents.len() > policy.max_receipt_count
        || policy.max_receipt_bytes == 0
        || policy.max_manifest_bytes == 0
        || manifest_bytes.len() > policy.max_manifest_bytes
    {
        return Err(NativePackageReceiptError::Policy(
            "empty or oversized package input".into(),
        ));
    }
    wire::validate_path(&policy.manifest_relative_path)?;
    let manifest_digest = digest(manifest_bytes);
    let max_age = i64::try_from(policy.max_receipt_age_seconds)
        .ok()
        .and_then(chrono::Duration::try_seconds)
        .filter(|age| *age > chrono::Duration::zero())
        .ok_or_else(|| {
            NativePackageReceiptError::Policy(
                "receipt age budget must be positive and representable".into(),
            )
        })?;
    let mut valid_until = trust.valid_until();
    let mut receipts = Vec::with_capacity(receipt_documents.len());
    let mut receipt_ids = HashSet::new();
    for bytes in receipt_documents {
        if bytes.len() > policy.max_receipt_bytes {
            return Err(NativePackageReceiptError::Invalid(
                "receipt exceeds host byte limit".into(),
            ));
        }
        let mut receipt: ProductReceipt = serde_json::from_slice(bytes)
            .map_err(|error| NativePackageReceiptError::Invalid(error.to_string()))?;
        receipt.verify_integrity()?;
        let key_valid_until = trust.verify(&receipt, policy.now, &policy.package_id)?;
        let created = receipt.created_at()?;
        if created > policy.now || policy.now.signed_duration_since(created) >= max_age {
            return Err(NativePackageReceiptError::ReceiptTime);
        }
        let receipt_deadline = created
            .checked_add_signed(max_age)
            .ok_or_else(|| NativePackageReceiptError::Policy("receipt expiry overflow".into()))?;
        valid_until = valid_until.min(key_valid_until).min(receipt_deadline);
        if receipt.target_profile.target_triple != policy.target_triple
            || !receipt
                .build_set_id
                .eq_ignore_ascii_case(&policy.build_set_id)
        {
            return Err(NativePackageReceiptError::Policy("target triple".into()));
        }
        if !receipt_ids.insert(receipt.receipt_id.clone()) {
            return Err(NativePackageReceiptError::Invalid(
                "duplicate receipt".into(),
            ));
        }
        let manifest_artifact = find_artifact(
            &receipt,
            &policy.manifest_logical_name,
            &policy.manifest_relative_path,
            "resource",
        )?;
        if !matches_digest(manifest_artifact, &manifest_digest) {
            return Err(NativePackageReceiptError::Artifact(
                "manifest digest".into(),
            ));
        }
        receipts.push(receipt);
    }
    let manifest: PluginPackageManifest = toml::from_str(
        std::str::from_utf8(manifest_bytes)
            .map_err(|error| NativePackageReceiptError::Invalid(error.to_string()))?,
    )
    .map_err(|error| NativePackageReceiptError::Invalid(error.to_string()))?;
    if policy.plugin_id.is_empty()
        || policy.package_id.is_empty()
        || manifest.id != policy.plugin_id
        || manifest.package_id() != policy.package_id
        || manifest.version != policy.package_version
        || manifest.sdk_api_version != policy.sdk_api_version
    {
        return Err(NativePackageReceiptError::Policy(
            "plugin/package identity".into(),
        ));
    }
    if (!manifest.supported_targets.is_empty()
        && !manifest
            .supported_targets
            .contains(&policy.target.runtime_mode))
        || (!manifest.supported_platforms.is_empty()
            && !manifest
                .supported_platforms
                .contains(&policy.target.platform))
    {
        return Err(NativePackageReceiptError::Policy("manifest target".into()));
    }
    let mut seen_kinds = Vec::new();
    let mut seen_artifacts = HashSet::new();
    let mut all_capabilities = HashSet::new();
    let mut expectations = Vec::with_capacity(policy.modules.len());
    for binding in &policy.modules {
        wire::validate_path(&binding.relative_path)?;
        if binding.logical_name.is_empty()
            || seen_kinds.contains(&binding.module_kind)
            || !seen_artifacts.insert((&binding.logical_name, &binding.relative_path))
            || !matches!(
                binding.module_kind,
                PluginModuleKind::Runtime | PluginModuleKind::Editor | PluginModuleKind::Native
            )
        {
            return Err(NativePackageReceiptError::Policy(
                "duplicate or unsupported native module".into(),
            ));
        }
        seen_kinds.push(binding.module_kind);
        let modules: Vec<_> = manifest
            .modules
            .iter()
            .filter(|module| module.kind == binding.module_kind)
            .collect();
        if modules.len() != 1
            || modules[0].crate_name.trim().is_empty()
            || (!modules[0].target_modes.is_empty()
                && !modules[0]
                    .target_modes
                    .contains(&policy.target.runtime_mode))
        {
            return Err(NativePackageReceiptError::Policy(
                "missing, ambiguous or wrong-target manifest module".into(),
            ));
        }
        let mut capabilities = manifest.capabilities.clone();
        capabilities.extend(modules[0].capabilities.iter().cloned());
        capabilities.sort();
        capabilities.dedup();
        if capabilities.iter().any(|capability| {
            capability.trim().is_empty() || !policy.allowed_capabilities.contains(capability)
        }) {
            return Err(NativePackageReceiptError::Policy(
                "capability exceeds host grant".into(),
            ));
        }
        all_capabilities.extend(capabilities.iter().cloned());
        let matches: Vec<_> = receipts
            .iter()
            .flat_map(|receipt| {
                receipt
                    .runtime_dependencies
                    .iter()
                    .filter(move |artifact| {
                        artifact.logical_name == binding.logical_name
                            && artifact.relative_path == binding.relative_path
                            && artifact.kind == "dynamic_library"
                    })
                    .map(move |artifact| (receipt, artifact))
            })
            .collect();
        if matches.len() != 1 {
            return Err(NativePackageReceiptError::Artifact(
                binding.logical_name.clone(),
            ));
        }
        let (receipt, artifact) = matches[0];
        let mut dependency_names = HashSet::new();
        let mut dependencies = Vec::with_capacity(binding.dependencies.len());
        for dependency in &binding.dependencies {
            wire::validate_path(&dependency.relative_path)?;
            let file_name = dependency.relative_path.rsplit('/').next().unwrap();
            let module_name = binding.relative_path.rsplit('/').next().unwrap();
            if file_name.eq_ignore_ascii_case(module_name)
                || !file_name.to_ascii_lowercase().ends_with(".dll")
                || !dependency_names.insert(file_name.to_ascii_lowercase())
            {
                return Err(NativePackageReceiptError::Policy(
                    "ambiguous dependency DLL basename".into(),
                ));
            }
            let dependency_matches: Vec<_> = receipts
                .iter()
                .flat_map(|receipt| receipt.runtime_dependencies.iter())
                .filter(|item| {
                    item.logical_name == dependency.logical_name
                        && item.relative_path == dependency.relative_path
                        && item.kind == "dynamic_library"
                })
                .collect();
            if dependency_matches.len() != 1 {
                return Err(NativePackageReceiptError::Artifact(
                    dependency.logical_name.clone(),
                ));
            }
            dependencies.push(NativePluginArtifactDependency {
                file_name: file_name.into(),
                digest: NativePluginArtifactDigest {
                    sha256: dependency_matches[0].sha256.to_ascii_lowercase(),
                    byte_length: dependency_matches[0].byte_length,
                },
            });
        }
        expectations.push(NativePluginArtifactExpectation {
            plugin_id: manifest.id.clone(),
            package_id: manifest.package_id(),
            manifest_digest: manifest_digest.clone(),
            library_digest: NativePluginArtifactDigest {
                sha256: artifact.sha256.to_ascii_lowercase(),
                byte_length: artifact.byte_length,
            },
            trust: NativePluginArtifactTrust::SignedPackage {
                signer_id: receipt.attestation.signer_id.clone(),
                algorithm: receipt.attestation.algorithm.clone(),
            },
            target: policy.target.clone(),
            module_kinds: vec![binding.module_kind],
            capabilities,
            dependencies,
        });
    }
    if policy
        .required_capabilities
        .iter()
        .any(|capability| !all_capabilities.contains(capability))
    {
        return Err(NativePackageReceiptError::Policy(
            "required capability is absent".into(),
        ));
    }
    Ok(VerifiedNativePackageProof {
        expectations,
        valid_until,
    })
}

fn find_artifact<'a>(
    receipt: &'a ProductReceipt,
    name: &str,
    path: &str,
    kind: &str,
) -> Result<&'a ReceiptArtifact> {
    receipt
        .runtime_dependencies
        .iter()
        .find(|artifact| {
            artifact.logical_name == name && artifact.relative_path == path && artifact.kind == kind
        })
        .ok_or_else(|| NativePackageReceiptError::Artifact(name.into()))
}

fn digest(bytes: &[u8]) -> NativePluginArtifactDigest {
    use sha2::{Digest, Sha256};
    NativePluginArtifactDigest {
        sha256: format!("{:x}", Sha256::digest(bytes)),
        byte_length: bytes.len() as u64,
    }
}

fn matches_digest(artifact: &ReceiptArtifact, digest: &NativePluginArtifactDigest) -> bool {
    artifact.byte_length == digest.byte_length
        && artifact.sha256.eq_ignore_ascii_case(&digest.sha256)
}
