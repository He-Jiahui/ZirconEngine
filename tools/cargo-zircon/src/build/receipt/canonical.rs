use std::borrow::Cow;
use std::io::{self, Write};

use serde::{ser::SerializeSeq, Serialize, Serializer};
use sha2::{Digest, Sha256};

use super::{
    BuildAction, ProducerIdentity, ProductReceipt, ProductReceiptDraft, ProductReceiptError,
    ReceiptArtifact, TargetProfile, ToolchainSet,
};

pub(crate) const PRODUCT_RECEIPT_SCHEMA_VERSION: u32 = 1;
pub(crate) const PRODUCT_RECEIPT_KIND: &str = "zircon_product_receipt";
const PRODUCT_RECEIPT_ATTESTATION_KIND: &str = "zircon_product_receipt_attestation";
pub(crate) const PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION: u32 = 1;
pub(crate) const PRODUCT_RECEIPT_BATCH_KIND: &str = "zircon_product_receipt_batch";
const PRODUCT_RECEIPT_BATCH_ATTESTATION_KIND: &str = "zircon_product_receipt_batch_attestation";
const UPPER_HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";
const RECEIPT_IDENTITY_SERIALIZATION_ERROR: &str = "could not serialize product receipt identity";
const RECEIPT_BATCH_IDENTITY_SERIALIZATION_ERROR: &str =
    "could not serialize product receipt batch identity";
const ATTESTATION_JSON_FIXED_CAPACITY: usize = 192;
pub(crate) const INLINE_SIGNATURE_CAPACITY: usize = 64;

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
struct CanonicalAttestation<'a> {
    schema_version: u32,
    attestation_kind: &'a str,
    receipt_id: &'a str,
    signer_id: &'a str,
    algorithm: &'a str,
}

#[derive(Serialize)]
struct CanonicalReceiptBatch<'a> {
    schema_version: u32,
    receipt_batch_kind: &'a str,
    build_set_id: &'a str,
    receipt_ids: ReceiptIds<'a>,
}

struct ReceiptIds<'a>(&'a [ProductReceipt]);

impl Serialize for ReceiptIds<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for receipt in self.0 {
            sequence.serialize_element(receipt.receipt_id.as_str())?;
        }
        sequence.end()
    }
}

#[derive(Serialize, PartialEq, Eq, Hash)]
pub(crate) struct CanonicalBuildActionKey<'a> {
    package: &'a str,
    bin: Option<&'a str>,
    features: Cow<'a, [String]>,
}

#[derive(Serialize)]
struct CanonicalBatchAttestation<'a> {
    schema_version: u32,
    attestation_kind: &'a str,
    batch_id: &'a str,
    signer_id: &'a str,
    algorithm: &'a str,
}

struct CanonicalDigestWriter {
    hasher: Sha256,
}

impl CanonicalDigestWriter {
    fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    fn finish_bytes(self) -> [u8; 32] {
        let digest = self.hasher.finalize();
        let mut bytes = [0_u8; 32];
        bytes.copy_from_slice(&digest);
        bytes
    }

    fn finish(self) -> String {
        bytes_to_hex(&self.finish_bytes())
    }
}

impl Write for CanonicalDigestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.hasher.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn canonical_receipt_sha256(
    draft: &ProductReceiptDraft,
    created_utc: &str,
) -> Result<String, ProductReceiptError> {
    let payload = canonical_receipt(draft, created_utc);
    sha256_serialized(&payload, RECEIPT_IDENTITY_SERIALIZATION_ERROR)
}

pub(crate) fn canonical_receipt_sha256_matches(
    draft: &ProductReceiptDraft,
    created_utc: &str,
    expected: &str,
) -> Result<bool, ProductReceiptError> {
    let payload = canonical_receipt(draft, created_utc);
    serialized_sha256_matches(&payload, expected, RECEIPT_IDENTITY_SERIALIZATION_ERROR)
}

fn canonical_receipt<'a>(
    draft: &'a ProductReceiptDraft,
    created_utc: &'a str,
) -> CanonicalReceipt<'a> {
    CanonicalReceipt {
        schema_version: PRODUCT_RECEIPT_SCHEMA_VERSION,
        receipt_kind: PRODUCT_RECEIPT_KIND,
        created_utc,
        build_set_id: &draft.build_set_id,
        toolchain: &draft.toolchain,
        target_profile: &draft.target_profile,
        action: &draft.action,
        producer: &draft.producer,
        build_products: &draft.build_products,
        runtime_dependencies: &draft.runtime_dependencies,
        symbols: &draft.symbols,
        sbom: draft.sbom.as_ref(),
    }
}

pub(crate) fn canonical_receipt_sha256_from_receipt(
    receipt: &ProductReceipt,
) -> Result<String, ProductReceiptError> {
    let payload = canonical_receipt_from_receipt(receipt);
    sha256_serialized(&payload, RECEIPT_IDENTITY_SERIALIZATION_ERROR)
}

pub(crate) fn canonical_receipt_sha256_from_receipt_matches(
    receipt: &ProductReceipt,
    expected: &str,
) -> Result<bool, ProductReceiptError> {
    let payload = canonical_receipt_from_receipt(receipt);
    serialized_sha256_matches(&payload, expected, RECEIPT_IDENTITY_SERIALIZATION_ERROR)
}

fn canonical_receipt_from_receipt(receipt: &ProductReceipt) -> CanonicalReceipt<'_> {
    CanonicalReceipt {
        schema_version: PRODUCT_RECEIPT_SCHEMA_VERSION,
        receipt_kind: PRODUCT_RECEIPT_KIND,
        created_utc: &receipt.created_utc,
        build_set_id: &receipt.build_set_id,
        toolchain: &receipt.toolchain,
        target_profile: &receipt.target_profile,
        action: &receipt.action,
        producer: &receipt.producer,
        build_products: &receipt.build_products,
        runtime_dependencies: &receipt.runtime_dependencies,
        symbols: &receipt.symbols,
        sbom: receipt.sbom.as_ref(),
    }
}

pub(crate) fn sha256_serialized<T: Serialize>(
    payload: &T,
    error_context: &str,
) -> Result<String, ProductReceiptError> {
    Ok(sha256_serialized_writer(payload, error_context)?.finish())
}

pub(crate) fn serialized_sha256_matches<T: Serialize>(
    payload: &T,
    expected: &str,
    error_context: &str,
) -> Result<bool, ProductReceiptError> {
    let digest = sha256_serialized_writer(payload, error_context)?.finish_bytes();
    Ok(upper_hex_matches(&digest, expected))
}

pub(crate) fn sha256_bytes_matches(bytes: &[u8], expected: &str) -> bool {
    upper_hex_matches(&Sha256::digest(bytes), expected)
}

fn sha256_serialized_writer<T: Serialize>(
    payload: &T,
    error_context: &str,
) -> Result<CanonicalDigestWriter, ProductReceiptError> {
    let mut writer = CanonicalDigestWriter::new();
    serde_json::to_writer(&mut writer, payload)
        .map_err(|error| ProductReceiptError::new(format!("{error_context}: {error}")))?;
    Ok(writer)
}

pub(crate) fn upper_hex_matches(bytes: &[u8], expected: &str) -> bool {
    if expected.len() != bytes.len().saturating_mul(2) {
        return false;
    }
    bytes
        .iter()
        .zip(expected.as_bytes().chunks_exact(2))
        .all(|(byte, pair)| {
            let byte = *byte;
            pair[0] == UPPER_HEX_DIGITS[usize::from(byte >> 4)]
                && pair[1] == UPPER_HEX_DIGITS[usize::from(byte & 0x0F)]
        })
}

pub(crate) fn attestation_bytes(
    receipt_id: &str,
    signer_id: &str,
    algorithm: &str,
) -> Result<Vec<u8>, ProductReceiptError> {
    // Bind signer selection to the closure identity so either cannot be substituted after issue.
    let payload = CanonicalAttestation {
        schema_version: PRODUCT_RECEIPT_SCHEMA_VERSION,
        attestation_kind: PRODUCT_RECEIPT_ATTESTATION_KIND,
        receipt_id,
        signer_id,
        algorithm,
    };
    serialize_attestation_with_capacity(
        &payload,
        receipt_id
            .len()
            .saturating_add(signer_id.len())
            .saturating_add(algorithm.len()),
        "could not serialize product receipt attestation",
    )
}

fn serialize_attestation_with_capacity(
    payload: &impl Serialize,
    dynamic_bytes: usize,
    error_context: &str,
) -> Result<Vec<u8>, ProductReceiptError> {
    let mut serialized =
        Vec::with_capacity(ATTESTATION_JSON_FIXED_CAPACITY.saturating_add(dynamic_bytes));
    serde_json::to_writer(&mut serialized, payload)
        .map_err(|error| ProductReceiptError::new(format!("{error_context}: {error}")))?;
    Ok(serialized)
}

pub(crate) fn canonical_receipt_batch_sha256(
    build_set_id: &str,
    receipts: &[ProductReceipt],
) -> Result<String, ProductReceiptError> {
    let payload = canonical_receipt_batch(build_set_id, receipts);
    sha256_serialized(&payload, RECEIPT_BATCH_IDENTITY_SERIALIZATION_ERROR)
}

pub(crate) fn canonical_receipt_batch_sha256_matches(
    build_set_id: &str,
    receipts: &[ProductReceipt],
    expected: &str,
) -> Result<bool, ProductReceiptError> {
    let payload = canonical_receipt_batch(build_set_id, receipts);
    serialized_sha256_matches(
        &payload,
        expected,
        RECEIPT_BATCH_IDENTITY_SERIALIZATION_ERROR,
    )
}

fn canonical_receipt_batch<'a>(
    build_set_id: &'a str,
    receipts: &'a [ProductReceipt],
) -> CanonicalReceiptBatch<'a> {
    CanonicalReceiptBatch {
        schema_version: PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION,
        receipt_batch_kind: PRODUCT_RECEIPT_BATCH_KIND,
        build_set_id,
        receipt_ids: ReceiptIds(receipts),
    }
}

pub(crate) fn canonical_build_action_key(action: &BuildAction) -> CanonicalBuildActionKey<'_> {
    let features = if action.features.windows(2).all(|pair| pair[0] <= pair[1]) {
        Cow::Borrowed(action.features.as_slice())
    } else {
        let mut features = action.features.clone();
        features.sort_unstable();
        Cow::Owned(features)
    };
    CanonicalBuildActionKey {
        package: &action.package,
        bin: action.bin.as_deref(),
        features,
    }
}

#[cfg(test)]
pub(crate) fn canonical_receipt_batch_sha256_with_collected_ids(
    build_set_id: &str,
    receipts: &[ProductReceipt],
) -> Result<String, ProductReceiptError> {
    #[derive(Serialize)]
    struct LegacyCanonicalReceiptBatch<'a> {
        schema_version: u32,
        receipt_batch_kind: &'a str,
        build_set_id: &'a str,
        receipt_ids: Vec<&'a str>,
    }

    let payload = LegacyCanonicalReceiptBatch {
        schema_version: PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION,
        receipt_batch_kind: PRODUCT_RECEIPT_BATCH_KIND,
        build_set_id,
        receipt_ids: receipts
            .iter()
            .map(|receipt| receipt.receipt_id.as_str())
            .collect(),
    };
    sha256_serialized(
        &payload,
        "could not serialize legacy product receipt batch identity",
    )
}

pub(crate) fn canonical_build_action_sha256(
    action: &BuildAction,
) -> Result<String, ProductReceiptError> {
    let payload = canonical_build_action_key(action);
    sha256_serialized(&payload, "could not serialize canonical build action")
}

pub(crate) fn batch_attestation_bytes(
    batch_id: &str,
    signer_id: &str,
    algorithm: &str,
) -> Result<Vec<u8>, ProductReceiptError> {
    let payload = CanonicalBatchAttestation {
        schema_version: PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION,
        attestation_kind: PRODUCT_RECEIPT_BATCH_ATTESTATION_KIND,
        batch_id,
        signer_id,
        algorithm,
    };
    serialize_attestation_with_capacity(
        &payload,
        batch_id
            .len()
            .saturating_add(signer_id.len())
            .saturating_add(algorithm.len()),
        "could not serialize product receipt batch attestation",
    )
}

#[cfg(test)]
fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    bytes_to_hex(&digest)
}

pub(crate) fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        let byte = *byte;
        encoded.push(UPPER_HEX_DIGITS[usize::from(byte >> 4)] as char);
        encoded.push(UPPER_HEX_DIGITS[usize::from(byte & 0x0F)] as char);
    }
    encoded
}

pub(crate) fn decode_hex(value: &str) -> Result<Vec<u8>, ProductReceiptError> {
    if value.len() % 2 != 0 {
        return Err(ProductReceiptError::new(
            "product receipt attestation signature must be hexadecimal",
        ));
    }
    let mut decoded = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let high = decode_hex_nibble(pair[0]).ok_or_else(|| {
            ProductReceiptError::new("product receipt attestation signature must be hexadecimal")
        })?;
        let low = decode_hex_nibble(pair[1]).ok_or_else(|| {
            ProductReceiptError::new("product receipt attestation signature must be hexadecimal")
        })?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

pub(crate) fn decode_hex_into(
    value: &str,
    output: &mut [u8],
) -> Result<Option<usize>, ProductReceiptError> {
    if value.len() % 2 != 0 {
        return Err(ProductReceiptError::new(
            "product receipt attestation signature must be hexadecimal",
        ));
    }
    let decoded_len = value.len() / 2;
    if decoded_len > output.len() {
        return Ok(None);
    }
    for (target, pair) in output
        .iter_mut()
        .take(decoded_len)
        .zip(value.as_bytes().chunks_exact(2))
    {
        let high = decode_hex_nibble(pair[0]).ok_or_else(|| {
            ProductReceiptError::new("product receipt attestation signature must be hexadecimal")
        })?;
        let low = decode_hex_nibble(pair[1]).ok_or_else(|| {
            ProductReceiptError::new("product receipt attestation signature must be hexadecimal")
        })?;
        *target = (high << 4) | low;
    }
    Ok(Some(decoded_len))
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
mod tests {
    use std::borrow::Cow;

    use serde::Serialize;

    use super::{
        attestation_bytes, batch_attestation_bytes, bytes_to_hex, canonical_build_action_key,
        canonical_build_action_sha256, decode_hex, decode_hex_into, serialized_sha256_matches,
        sha256_hex, sha256_serialized, BuildAction, CanonicalAttestation,
        CanonicalBatchAttestation, PRODUCT_RECEIPT_ATTESTATION_KIND,
        PRODUCT_RECEIPT_BATCH_ATTESTATION_KIND, PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION,
        PRODUCT_RECEIPT_SCHEMA_VERSION,
    };

    #[test]
    fn encodes_and_decodes_every_hex_nibble_boundary() {
        let bytes = [0x00, 0x0F, 0x10, 0x7F, 0x80, 0xF0, 0xFF];

        let encoded = bytes_to_hex(&bytes);

        assert_eq!(encoded, "000F107F80F0FF");
        assert_eq!(decode_hex(&encoded).unwrap(), bytes);
    }

    #[test]
    fn decodes_mixed_case_hex_and_rejects_invalid_input() {
        assert_eq!(decode_hex("0fA5c0").unwrap(), vec![0x0F, 0xA5, 0xC0]);
        assert!(decode_hex("F").is_err());
        assert!(decode_hex("GG").is_err());
    }

    #[test]
    fn inline_hex_decode_matches_allocating_decode() {
        let encoded = bytes_to_hex(&(0_u8..=63).collect::<Vec<_>>());
        let mut inline = [0_u8; 64];

        let inline_len = decode_hex_into(&encoded, &mut inline).unwrap().unwrap();

        assert_eq!(&inline[..inline_len], decode_hex(&encoded).unwrap());
        assert!(decode_hex_into("00".repeat(65).as_str(), &mut inline)
            .unwrap()
            .is_none());
        assert!(decode_hex_into("GG", &mut inline).is_err());
    }

    #[derive(Serialize)]
    struct StreamedDigestFixture<'a> {
        label: &'a str,
        values: &'a [u32],
    }

    #[test]
    fn streamed_digest_matches_the_canonical_json_bytes() {
        let payload = StreamedDigestFixture {
            label: "receipt identity",
            values: &[3, 1, 4, 1, 5, 9],
        };

        let expected = sha256_hex(&serde_json::to_vec(&payload).unwrap());

        assert_eq!(
            sha256_serialized(&payload, "fixture serialization").unwrap(),
            expected
        );
    }

    #[test]
    fn streamed_digest_match_preserves_canonical_uppercase() {
        let payload = StreamedDigestFixture {
            label: "receipt identity",
            values: &[2, 7, 1, 8, 2, 8],
        };
        let expected = sha256_serialized(&payload, "fixture serialization").unwrap();

        assert!(serialized_sha256_matches(&payload, &expected, "fixture serialization").unwrap());
        assert!(!serialized_sha256_matches(
            &payload,
            &expected.to_lowercase(),
            "fixture serialization"
        )
        .unwrap());
        assert!(!serialized_sha256_matches(
            &payload,
            &expected[..expected.len() - 1],
            "fixture serialization"
        )
        .unwrap());
    }

    #[test]
    fn preallocated_attestation_payloads_match_serde_for_escaped_fields() {
        let receipt_id = "receipt\\\"identity\\nwith-escape";
        let batch_id = "batch\\\\identity\\twith-escape";
        let signer_id = "signer\\\"id";
        let algorithm = "algorithm\\nversion";
        let receipt_payload = CanonicalAttestation {
            schema_version: PRODUCT_RECEIPT_SCHEMA_VERSION,
            attestation_kind: PRODUCT_RECEIPT_ATTESTATION_KIND,
            receipt_id,
            signer_id,
            algorithm,
        };
        let batch_payload = CanonicalBatchAttestation {
            schema_version: PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION,
            attestation_kind: PRODUCT_RECEIPT_BATCH_ATTESTATION_KIND,
            batch_id,
            signer_id,
            algorithm,
        };

        assert_eq!(
            attestation_bytes(receipt_id, signer_id, algorithm).unwrap(),
            serde_json::to_vec(&receipt_payload).unwrap()
        );
        assert_eq!(
            batch_attestation_bytes(batch_id, signer_id, algorithm).unwrap(),
            serde_json::to_vec(&batch_payload).unwrap()
        );
    }

    #[test]
    fn borrowed_build_action_digest_matches_the_legacy_sorted_payload() {
        let action = BuildAction {
            package: "zircon-editor".to_string(),
            bin: Some("zircon_editor".to_string()),
            features: vec![
                "runtime".to_string(),
                "editor".to_string(),
                "asset-pipeline".to_string(),
            ],
        };
        let mut legacy = action.clone();
        legacy.features.sort();
        let expected = sha256_serialized(&legacy, "legacy build action").unwrap();

        assert_eq!(canonical_build_action_sha256(&action).unwrap(), expected);
    }

    #[test]
    fn structural_build_action_key_ignores_feature_order() {
        let left = BuildAction {
            package: "zircon-editor".to_string(),
            bin: Some("zircon_editor".to_string()),
            features: vec!["runtime".to_string(), "editor".to_string()],
        };
        let right = BuildAction {
            features: vec!["editor".to_string(), "runtime".to_string()],
            ..left.clone()
        };

        assert!(canonical_build_action_key(&left) == canonical_build_action_key(&right));
    }

    #[test]
    fn canonical_build_action_key_borrows_normalized_features() {
        let action = BuildAction {
            package: "zircon-editor".to_string(),
            bin: Some("zircon_editor".to_string()),
            features: vec!["asset-pipeline".to_string(), "editor".to_string()],
        };

        assert!(matches!(
            canonical_build_action_key(&action).features,
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn canonical_build_action_key_normalizes_unordered_external_features() {
        let action = BuildAction {
            package: "zircon-editor".to_string(),
            bin: Some("zircon_editor".to_string()),
            features: vec!["runtime".to_string(), "asset-pipeline".to_string()],
        };
        let key = canonical_build_action_key(&action);

        assert!(matches!(&key.features, Cow::Owned(_)));
        assert_eq!(key.features.len(), 2);
        assert_eq!(key.features[0], "asset-pipeline");
        assert_eq!(key.features[1], "runtime");
    }
}

#[cfg(test)]
mod performance_tests;
