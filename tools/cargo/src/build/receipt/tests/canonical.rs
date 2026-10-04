use std::borrow::Cow;

use serde::Serialize;

use super::{
    attestation_bytes, batch_attestation_bytes, bytes_to_hex, canonical_build_action_key,
    canonical_build_action_sha256, decode_hex, decode_hex_into, serialized_sha256_matches,
    sha256_hex, sha256_serialized, BuildAction, CanonicalAttestation, CanonicalBatchAttestation,
    PRODUCT_RECEIPT_ATTESTATION_KIND, PRODUCT_RECEIPT_BATCH_ATTESTATION_KIND,
    PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION, PRODUCT_RECEIPT_SCHEMA_VERSION,
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
