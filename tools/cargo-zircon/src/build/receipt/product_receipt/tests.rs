use super::ProductReceipt;
use crate::build::receipt::canonical::bytes_to_hex;
use crate::build::receipt::{
    ArtifactKind, BuildAction, ProducerIdentity, ProductReceiptDraft, ProductReceiptSigner,
    ProductReceiptVerifier, ReceiptArtifact, TargetProfile, ToolchainSet,
};
use std::cell::Cell;

struct TestSigner;

impl ProductReceiptSigner for TestSigner {
    fn signer_id(&self) -> &str {
        "test-worker"
    }

    fn algorithm(&self) -> &str {
        "test-signature-v1"
    }

    fn sign(&self, _attestation_payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(vec![0xA5; 64])
    }
}

struct CountingVerifier {
    calls: Cell<usize>,
    reject: bool,
}

struct ExpectedPayloadVerifier<'a> {
    expected_payload: &'a [u8],
}

struct LongSignatureVerifier;

impl ProductReceiptVerifier for CountingVerifier {
    fn verify(
        &self,
        signer_id: &str,
        algorithm: &str,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(signer_id, "test-worker");
        assert_eq!(algorithm, "test-signature-v1");
        assert!(!attestation_payload.is_empty());
        assert_eq!(signature, &[0xA5; 64]);
        self.calls.set(self.calls.get() + 1);
        if self.reject {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "fixture rejected signature",
            )
            .into());
        }
        Ok(())
    }
}

impl ProductReceiptVerifier for ExpectedPayloadVerifier<'_> {
    fn verify(
        &self,
        signer_id: &str,
        algorithm: &str,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(signer_id, "test-worker");
        assert_eq!(algorithm, "test-signature-v1");
        assert_eq!(attestation_payload, self.expected_payload);
        assert_eq!(signature, &[0xA5; 64]);
        Ok(())
    }
}

impl ProductReceiptVerifier for LongSignatureVerifier {
    fn verify(
        &self,
        _signer_id: &str,
        _algorithm: &str,
        _attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(signature, &[0xA5; 65]);
        Ok(())
    }
}

#[test]
fn noncanonical_receipt_keeps_legacy_normalizing_verification() {
    let mut receipt =
        ProductReceipt::issue(test_draft(), "2026-08-29T00:00:00Z", &TestSigner).unwrap();
    receipt.build_set_id.make_ascii_lowercase();
    receipt.action.features.reverse();
    receipt.build_products.reverse();

    receipt.verify_integrity().unwrap();
}

#[test]
fn fresh_receipt_issue_normalizes_external_unordered_draft() {
    let mut draft = test_draft();
    draft.build_set_id.make_ascii_lowercase();
    draft.action.features.sort_unstable();
    draft.action.features.reverse();
    draft
        .build_products
        .sort_by(|left, right| left.logical_name.cmp(&right.logical_name));
    draft.build_products.reverse();

    let receipt = ProductReceipt::issue(draft, "2026-08-29T00:00:00Z", &TestSigner).unwrap();

    assert!(receipt
        .build_set_id
        .bytes()
        .all(|byte| !byte.is_ascii_lowercase()));
    assert!(receipt
        .action
        .features
        .windows(2)
        .all(|pair| pair[0] <= pair[1]));
    assert!(receipt
        .build_products
        .windows(2)
        .all(|pair| pair[0].logical_name <= pair[1].logical_name));
}

#[test]
fn receipt_issue_rejects_duplicate_names_across_artifact_partitions() {
    let mut draft = test_draft();
    draft.runtime_dependencies.push(ReceiptArtifact {
        logical_name: "editor".to_string(),
        relative_path: "runtime/editor_support.dll".to_string(),
        kind: ArtifactKind::DynamicLibrary,
        sha256: digest('3'),
        byte_length: 12_288,
    });

    let error = ProductReceipt::issue(draft, "2026-08-29T00:00:00Z", &TestSigner).unwrap_err();

    assert!(error
        .to_string()
        .contains("duplicate artifact logical name `editor`"));
}

#[test]
fn fresh_receipt_publication_checks_attestation_once() {
    let draft = test_draft();
    let handoff_sha256 = draft.handoff_sha256().unwrap();
    let verified = draft.verify_handoff_sha256_owned(&handoff_sha256).unwrap();
    let verifier = CountingVerifier {
        calls: Cell::new(0),
        reject: false,
    };

    let publication = verified
        .issue_verified("2026-08-29T00:00:00Z", &TestSigner, &verifier)
        .unwrap();

    let expected =
        ProductReceipt::issue(test_draft(), "2026-08-29T00:00:00Z", &TestSigner).unwrap();
    assert_eq!(publication.receipt_id(), expected.receipt_id);
    assert_eq!(verifier.calls.get(), 1);
}

#[test]
fn fresh_receipt_publication_rejects_an_untrusted_attestation() {
    let draft = test_draft();
    let handoff_sha256 = draft.handoff_sha256().unwrap();
    let verified = draft.verify_handoff_sha256_owned(&handoff_sha256).unwrap();
    let verifier = CountingVerifier {
        calls: Cell::new(0),
        reject: true,
    };

    let error = verified
        .issue_verified("2026-08-29T00:00:00Z", &TestSigner, &verifier)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("product receipt attestation verification failed"));
    assert_eq!(verifier.calls.get(), 1);
}

#[test]
fn direct_fresh_receipt_publication_checks_attestation_once() {
    let verifier = CountingVerifier {
        calls: Cell::new(0),
        reject: false,
    };

    let publication = ProductReceipt::issue_verified(
        test_draft(),
        "2026-08-29T00:00:00Z",
        &TestSigner,
        &verifier,
    )
    .unwrap();

    assert!(!publication.receipt_id().is_empty());
    assert_eq!(verifier.calls.get(), 1);
}

#[test]
fn fresh_receipt_publication_verifies_the_retained_raw_signature() {
    let verifier = CountingVerifier {
        calls: Cell::new(0),
        reject: false,
    };
    let mut fresh =
        ProductReceipt::issue_fresh(test_draft(), "2026-08-29T00:00:00Z", &TestSigner).unwrap();
    fresh.receipt.attestation.signature_hex = "not-hex".to_string();

    fresh.verify_attestation(&verifier).unwrap();

    assert_eq!(verifier.calls.get(), 1);
}

#[test]
fn fresh_receipt_publication_reuses_the_signed_payload() {
    let mut fresh =
        ProductReceipt::issue_fresh(test_draft(), "2026-08-29T00:00:00Z", &TestSigner).unwrap();
    let signed_payload = fresh.attestation.payload().to_vec();
    fresh.receipt.receipt_id = digest('0');
    let verifier = ExpectedPayloadVerifier {
        expected_payload: &signed_payload,
    };

    fresh.verify_attestation(&verifier).unwrap();
}

#[test]
fn oversized_custom_signature_keeps_allocating_decode_fallback() {
    let mut receipt =
        ProductReceipt::issue(test_draft(), "2026-08-29T00:00:00Z", &TestSigner).unwrap();
    receipt.attestation.signature_hex = bytes_to_hex(&[0xA5; 65]);

    receipt.verify_attestation(&LongSignatureVerifier).unwrap();
}

#[test]
fn draft_handoff_parser_accepts_compact_and_legacy_pretty_bytes() {
    let draft = test_draft();
    let expected = draft.handoff_sha256().unwrap();
    let compact = serde_json::to_vec(&draft).unwrap();
    let pretty = serde_json::to_vec_pretty(&draft).unwrap();

    let compact_verified =
        ProductReceiptDraft::parse_and_verify_handoff_sha256(&compact, &expected)
            .unwrap()
            .issue("2026-08-29T00:00:00Z", &TestSigner)
            .unwrap();
    let pretty_verified = ProductReceiptDraft::parse_and_verify_handoff_sha256(&pretty, &expected)
        .unwrap()
        .issue("2026-08-29T00:00:00Z", &TestSigner)
        .unwrap();

    assert_eq!(compact_verified.receipt_id, pretty_verified.receipt_id);
}

fn test_draft() -> ProductReceiptDraft {
    ProductReceiptDraft {
        build_set_id: digest('A'),
        toolchain: ToolchainSet::new(
            digest('B'),
            digest('C'),
            Some(digest('D')),
            digest('E'),
            digest('F'),
        )
        .unwrap(),
        target_profile: TargetProfile {
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            cargo_profile: "release".to_string(),
            codegen_flags_digest: digest('1'),
            cargo_graph_digest: digest('2'),
        },
        action: BuildAction {
            package: "zircon-product".to_string(),
            bin: Some("zircon_product".to_string()),
            features: vec!["runtime".to_string(), "editor".to_string()],
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: "0.1.0".to_string(),
            worker_id: "test-worker".to_string(),
            operation_id: "test-operation".to_string(),
        },
        build_products: vec![
            artifact("runtime", "runtime/zircon_runtime.exe", 1),
            artifact("editor", "editor/zircon_editor.exe", 2),
        ],
        runtime_dependencies: Vec::new(),
        symbols: Vec::new(),
        sbom: None,
    }
}

fn artifact(logical_name: &str, relative_path: &str, seed: u64) -> ReceiptArtifact {
    ReceiptArtifact {
        logical_name: logical_name.to_string(),
        relative_path: relative_path.to_string(),
        kind: ArtifactKind::Executable,
        sha256: digest(char::from_digit(seed as u32, 16).unwrap()),
        byte_length: seed * 4_096,
    }
}

fn digest(character: char) -> String {
    std::iter::repeat(character).take(64).collect()
}
