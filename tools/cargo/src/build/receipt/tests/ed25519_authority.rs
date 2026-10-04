use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};

use super::{
    bytes_to_hex, Ed25519ProductReceiptSigner, ProductReceiptTrustRegistry, ProductReceiptVerifier,
    TrustedIssuers, ED25519_PUBLIC_KEY_LENGTH,
};

#[test]
fn public_key_hex_is_cached_only_when_requested() {
    let private_key = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
    let signer =
        Ed25519ProductReceiptSigner::from_pkcs8("build-worker-01", private_key.as_ref()).unwrap();

    let first = signer.public_key_hex();
    let first_pointer = first.as_ptr();

    assert_eq!(first.len(), ED25519_PUBLIC_KEY_LENGTH * 2);
    assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(signer.public_key_hex().as_ptr(), first_pointer);
}

#[test]
fn borrowed_registry_text_retains_escaped_json_support() {
    let registry = format!(
        r#"{{"schema_version":1,"trust_registry_kind":"zircon_product_receipt_trust_reg\u0069stry","issuers":[{{"signer_id":"build-worker-01","algorithm":"ed25519-\u00761","public_key_hex":"{}","disabled":false}}]}}"#,
        "00".repeat(ED25519_PUBLIC_KEY_LENGTH)
    );

    ProductReceiptTrustRegistry::from_json(registry.as_bytes()).unwrap();
}

#[test]
fn single_issuer_registry_uses_direct_storage() {
    let registry = serde_json::json!({
        "schema_version": 1,
        "trust_registry_kind": "zircon_product_receipt_trust_registry",
        "issuers": [{
            "signer_id": "build-worker-01",
            "algorithm": "ed25519-v1",
            "public_key_hex": "00".repeat(ED25519_PUBLIC_KEY_LENGTH),
            "disabled": false
        }]
    });

    let registry =
        ProductReceiptTrustRegistry::from_json(&serde_json::to_vec(&registry).unwrap()).unwrap();

    assert!(matches!(registry.issuers, TrustedIssuers::Single { .. }));
}

#[test]
fn single_issuer_direct_storage_verifies_only_its_signer() {
    let private_key = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
    let key_pair = Ed25519KeyPair::from_pkcs8(private_key.as_ref()).unwrap();
    let registry = serde_json::json!({
        "schema_version": 1,
        "trust_registry_kind": "zircon_product_receipt_trust_registry",
        "issuers": [{
            "signer_id": "build-worker-01",
            "algorithm": "ed25519-v1",
            "public_key_hex": bytes_to_hex(key_pair.public_key().as_ref()),
            "disabled": false
        }]
    });
    let registry =
        ProductReceiptTrustRegistry::from_json(&serde_json::to_vec(&registry).unwrap()).unwrap();
    let payload = b"single issuer direct verification";
    let signature = key_pair.sign(payload);

    ProductReceiptVerifier::verify(
        &registry,
        "build-worker-01",
        "ed25519-v1",
        payload,
        signature.as_ref(),
    )
    .unwrap();
    assert!(ProductReceiptVerifier::verify(
        &registry,
        "build-worker-02",
        "ed25519-v1",
        payload,
        signature.as_ref(),
    )
    .is_err());
}

#[test]
fn multiple_issuer_registry_retains_hashed_lookup() {
    let registry = serde_json::json!({
        "schema_version": 1,
        "trust_registry_kind": "zircon_product_receipt_trust_registry",
        "issuers": [
            {
                "signer_id": "build-worker-01",
                "algorithm": "ed25519-v1",
                "public_key_hex": "00".repeat(ED25519_PUBLIC_KEY_LENGTH),
                "disabled": false
            },
            {
                "signer_id": "build-worker-02",
                "algorithm": "ed25519-v1",
                "public_key_hex": "11".repeat(ED25519_PUBLIC_KEY_LENGTH),
                "disabled": false
            }
        ]
    });

    let registry =
        ProductReceiptTrustRegistry::from_json(&serde_json::to_vec(&registry).unwrap()).unwrap();

    let TrustedIssuers::Multiple(issuers) = registry.issuers else {
        panic!("multiple issuers must retain hashed lookup");
    };
    assert_eq!(issuers.len(), 2);
}

#[test]
fn duplicate_issuer_is_rejected_before_decoding_its_unused_key() {
    let registry = serde_json::json!({
        "schema_version": 1,
        "trust_registry_kind": "zircon_product_receipt_trust_registry",
        "issuers": [
            {
                "signer_id": "build-worker-01",
                "algorithm": "ed25519-v1",
                "public_key_hex": "00".repeat(32),
                "disabled": false
            },
            {
                "signer_id": "build-worker-01",
                "algorithm": "ed25519-v1",
                "public_key_hex": "unused-invalid-key",
                "disabled": false
            }
        ]
    });

    let error = ProductReceiptTrustRegistry::from_json(&serde_json::to_vec(&registry).unwrap())
        .err()
        .unwrap();

    assert!(error.to_string().contains("duplicate signer"));
}
