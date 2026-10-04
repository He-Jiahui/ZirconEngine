use super::{NativePackageReceiptError as Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProductReceipt {
    pub schema_version: u32,
    pub receipt_kind: String,
    pub receipt_id: String,
    pub created_utc: String,
    pub build_set_id: String,
    pub toolchain: ToolchainSet,
    pub target_profile: TargetProfile,
    pub action: BuildAction,
    pub producer: ProducerIdentity,
    pub build_products: Vec<ReceiptArtifact>,
    pub runtime_dependencies: Vec<ReceiptArtifact>,
    pub symbols: Vec<ReceiptArtifact>,
    pub sbom: Option<ReceiptArtifact>,
    pub attestation: ReceiptAttestation,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ToolchainSet {
    pub toolchain_set_id: String,
    pub cargo_sha256: String,
    pub rustc_sha256: String,
    pub linker_sha256: Option<String>,
    pub sdk_fingerprint: String,
    pub environment_digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TargetProfile {
    pub target_triple: String,
    pub cargo_profile: String,
    pub codegen_flags_digest: String,
    pub cargo_graph_digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BuildAction {
    pub package: String,
    pub bin: Option<String>,
    pub features: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProducerIdentity {
    pub tool: String,
    pub tool_version: String,
    pub worker_id: String,
    pub operation_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReceiptArtifact {
    pub logical_name: String,
    pub relative_path: String,
    pub kind: String,
    pub sha256: String,
    pub byte_length: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReceiptAttestation {
    pub signer_id: String,
    pub algorithm: String,
    pub signature_hex: String,
}

#[derive(Serialize)]
struct CanonicalReceipt<'a> {
    schema_version: u32,
    receipt_kind: &'a str,
    created_utc: &'a str,
    build_set_id: &'a str,
    toolchain: &'a ToolchainSet,
    target_profile: &'a TargetProfile,
    action: &'a BuildAction,
    producer: &'a ProducerIdentity,
    build_products: &'a [ReceiptArtifact],
    runtime_dependencies: &'a [ReceiptArtifact],
    symbols: &'a [ReceiptArtifact],
    sbom: Option<&'a ReceiptArtifact>,
}
#[derive(Serialize)]
struct CanonicalToolchain<'a> {
    schema_version: u32,
    toolchain_set_kind: &'a str,
    cargo_sha256: &'a str,
    rustc_sha256: &'a str,
    linker_sha256: Option<&'a str>,
    sdk_fingerprint: &'a str,
    environment_digest: &'a str,
}
#[derive(Serialize)]
struct CanonicalAttestation<'a> {
    schema_version: u32,
    attestation_kind: &'a str,
    receipt_id: &'a str,
    signer_id: &'a str,
    algorithm: &'a str,
}

impl ProductReceipt {
    pub(super) fn verify_integrity(&mut self) -> Result<()> {
        if self.schema_version != 1 || self.receipt_kind != "zircon_product_receipt" {
            return Err(Error::Invalid("receipt identity/schema".into()));
        }
        normalize_digest(&mut self.build_set_id)?;
        normalize_digest(&mut self.target_profile.codegen_flags_digest)?;
        normalize_digest(&mut self.target_profile.cargo_graph_digest)?;
        let toolchain = &mut self.toolchain;
        for digest in [
            &mut toolchain.toolchain_set_id,
            &mut toolchain.cargo_sha256,
            &mut toolchain.rustc_sha256,
            &mut toolchain.sdk_fingerprint,
            &mut toolchain.environment_digest,
        ] {
            normalize_digest(digest)?;
        }
        if let Some(digest) = &mut toolchain.linker_sha256 {
            normalize_digest(digest)?;
        }
        if toolchain.toolchain_set_id != toolchain_id(toolchain)? {
            return Err(Error::Invalid("toolchain identity".into()));
        }
        if toolchain.linker_sha256.is_none()
            && self
                .target_profile
                .target_triple
                .split('-')
                .any(|part| part.eq_ignore_ascii_case("windows"))
        {
            return Err(Error::Invalid(
                "Windows receipt lacks linker fingerprint".into(),
            ));
        }
        for value in [
            &self.action.package,
            &self.target_profile.target_triple,
            &self.target_profile.cargo_profile,
            &self.producer.tool,
            &self.producer.tool_version,
            &self.producer.worker_id,
            &self.producer.operation_id,
        ] {
            required_text(value)?;
        }
        if let Some(bin) = &self.action.bin {
            required_text(bin)?;
        }
        for feature in &self.action.features {
            required_text(feature)?;
        }
        self.action.features.sort();
        if self
            .action
            .features
            .windows(2)
            .any(|pair| pair[0] == pair[1])
        {
            return Err(Error::Invalid("duplicate build feature".into()));
        }
        self.created_at()?;
        if self.build_products.is_empty() {
            return Err(Error::Invalid("missing product".into()));
        }
        let mut names = HashSet::new();
        let mut paths = HashSet::new();
        for (artifacts, allowed) in [
            (&mut self.build_products, &["executable"][..]),
            (
                &mut self.runtime_dependencies,
                &["dynamic_library", "resource"][..],
            ),
            (&mut self.symbols, &["symbol_file"][..]),
        ] {
            for artifact in artifacts.iter_mut() {
                validate_artifact(artifact, allowed, &mut names, &mut paths)?;
            }
            artifacts.sort_by(|left, right| left.logical_name.cmp(&right.logical_name));
        }
        if let Some(artifact) = &mut self.sbom {
            validate_artifact(artifact, &["sbom"], &mut names, &mut paths)?;
        }
        if self.attestation.algorithm != "ed25519-v1" {
            return Err(Error::Invalid("attestation algorithm".into()));
        }
        validate_signer(&self.attestation.signer_id)?;
        decode_hex::<64>(&self.attestation.signature_hex)?;
        if self.receipt_id != self.canonical_id()? {
            return Err(Error::Invalid("receipt digest".into()));
        }
        Ok(())
    }
    pub(super) fn canonical_id(&self) -> Result<String> {
        let bytes = serde_json::to_vec(&CanonicalReceipt {
            schema_version: 1,
            receipt_kind: "zircon_product_receipt",
            created_utc: &self.created_utc,
            build_set_id: &self.build_set_id,
            toolchain: &self.toolchain,
            target_profile: &self.target_profile,
            action: &self.action,
            producer: &self.producer,
            build_products: &self.build_products,
            runtime_dependencies: &self.runtime_dependencies,
            symbols: &self.symbols,
            sbom: self.sbom.as_ref(),
        })
        .map_err(|e| Error::Invalid(e.to_string()))?;
        Ok(format!("{:X}", Sha256::digest(bytes)))
    }
    pub(super) fn attestation_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&CanonicalAttestation {
            schema_version: 1,
            attestation_kind: "zircon_product_receipt_attestation",
            receipt_id: &self.receipt_id,
            signer_id: &self.attestation.signer_id,
            algorithm: &self.attestation.algorithm,
        })
        .map_err(|e| Error::Invalid(e.to_string()))
    }
    pub(super) fn created_at(&self) -> Result<DateTime<Utc>> {
        if !self.created_utc.ends_with('Z') || self.created_utc.as_bytes().get(10) != Some(&b'T') {
            return Err(Error::Invalid("created_utc must be UTC".into()));
        }
        DateTime::parse_from_rfc3339(&self.created_utc)
            .map(|value| value.with_timezone(&Utc))
            .map_err(|error| Error::Invalid(format!("created_utc: {error}")))
    }
}
pub(super) fn toolchain_id(toolchain: &ToolchainSet) -> Result<String> {
    let bytes = serde_json::to_vec(&CanonicalToolchain {
        schema_version: 1,
        toolchain_set_kind: "zircon_toolchain_set",
        cargo_sha256: &toolchain.cargo_sha256,
        rustc_sha256: &toolchain.rustc_sha256,
        linker_sha256: toolchain.linker_sha256.as_deref(),
        sdk_fingerprint: &toolchain.sdk_fingerprint,
        environment_digest: &toolchain.environment_digest,
    })
    .map_err(|e| Error::Invalid(e.to_string()))?;
    Ok(format!("{:X}", Sha256::digest(bytes)))
}
fn validate_artifact(
    artifact: &mut ReceiptArtifact,
    allowed: &[&str],
    names: &mut HashSet<String>,
    paths: &mut HashSet<String>,
) -> Result<()> {
    required_text(&artifact.logical_name)?;
    validate_path(&artifact.relative_path)?;
    normalize_digest(&mut artifact.sha256)?;
    if !allowed.contains(&artifact.kind.as_str())
        || !names.insert(artifact.logical_name.clone())
        || !paths.insert(artifact.relative_path.to_ascii_lowercase())
    {
        return Err(Error::Invalid(
            "ambiguous artifact name/path or invalid artifact kind".into(),
        ));
    }
    Ok(())
}
pub(super) fn required_text(value: &str) -> Result<()> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(Error::Invalid("empty/control text".into()));
    }
    Ok(())
}
pub(super) fn validate_signer(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || bytes.len() > 128
        || !(bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        || !bytes.iter().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
    {
        return Err(Error::Invalid("invalid signer ID".into()));
    }
    Ok(())
}
pub(super) fn validate_path(value: &str) -> Result<()> {
    required_text(value)?;
    if value.contains(['\\', ':'])
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(Error::Invalid("unsafe artifact path".into()));
    }
    Ok(())
}
fn normalize_digest(value: &mut String) -> Result<()> {
    decode_hex::<32>(value)?;
    value.make_ascii_uppercase();
    Ok(())
}
pub(super) fn decode_hex<const N: usize>(value: &str) -> Result<[u8; N]> {
    if value.len() != N * 2 {
        return Err(Error::Invalid("hex length".into()));
    }
    let mut output = [0; N];
    for (byte, pair) in output.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
        let digit = |b| match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            b'A'..=b'F' => Ok(b - b'A' + 10),
            _ => Err(Error::Invalid("hex encoding".into())),
        };
        *byte = digit(pair[0])? << 4 | digit(pair[1])?;
    }
    Ok(output)
}
