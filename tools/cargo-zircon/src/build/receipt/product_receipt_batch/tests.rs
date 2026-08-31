use super::ProductReceiptBatch;
use crate::build::receipt::{
    canonical::{
        canonical_receipt_batch_sha256, canonical_receipt_batch_sha256_with_collected_ids,
    },
    ArtifactKind, BuildAction, ProducerIdentity, ProductReceipt, ProductReceiptDraft,
    ProductReceiptSigner, ProductReceiptVerifier, ReceiptArtifact, TargetProfile, ToolchainSet,
};
use std::cell::Cell;

struct TestSigner;

#[derive(Default)]
struct MetadataCountingSigner {
    signer_id_calls: Cell<usize>,
    algorithm_calls: Cell<usize>,
    sign_calls: Cell<usize>,
}

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

impl ProductReceiptSigner for MetadataCountingSigner {
    fn signer_id(&self) -> &str {
        self.signer_id_calls.set(self.signer_id_calls.get() + 1);
        "test-worker"
    }

    fn algorithm(&self) -> &str {
        self.algorithm_calls.set(self.algorithm_calls.get() + 1);
        "test-signature-v1"
    }

    fn sign(&self, _attestation_payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.sign_calls.set(self.sign_calls.get() + 1);
        Ok(vec![0xA5; 64])
    }
}

struct CountingVerifier {
    calls: Cell<usize>,
    reject_on: Option<usize>,
}

struct PayloadSequenceVerifier {
    calls: Cell<usize>,
    expected_payloads: Vec<Vec<u8>>,
}

impl CountingVerifier {
    fn accepting() -> Self {
        Self {
            calls: Cell::new(0),
            reject_on: None,
        }
    }

    fn rejecting(reject_on: usize) -> Self {
        Self {
            calls: Cell::new(0),
            reject_on: Some(reject_on),
        }
    }
}

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
        let call = self.calls.get() + 1;
        self.calls.set(call);
        if self.reject_on == Some(call) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "fixture rejected signature",
            )
            .into());
        }
        Ok(())
    }
}

impl ProductReceiptVerifier for PayloadSequenceVerifier {
    fn verify(
        &self,
        signer_id: &str,
        algorithm: &str,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let call = self.calls.get();
        assert_eq!(signer_id, "test-worker");
        assert_eq!(algorithm, "test-signature-v1");
        assert_eq!(attestation_payload, self.expected_payloads[call].as_slice());
        assert_eq!(signature, &[0xA5; 64]);
        self.calls.set(call + 1);
        Ok(())
    }
}

#[test]
fn public_batch_issue_rejects_a_tampered_child_receipt() {
    let build_set_id = digest('A');
    let mut first = test_receipt(0, &build_set_id);
    first.receipt_id = digest('0');
    let second = test_receipt(1, &build_set_id);

    let error =
        ProductReceiptBatch::issue(build_set_id, vec![first, second], &TestSigner).unwrap_err();

    assert!(error
        .to_string()
        .contains("identity does not match its declared build closure"));
}

#[test]
fn public_batch_issue_rejects_a_duplicate_artifact_identity() {
    let build_set_id = digest('A');
    let first = test_receipt(0, &build_set_id);
    let mut second = test_receipt(1, &build_set_id);
    second.build_products[0].logical_name = first.build_products[0].logical_name.clone();

    let error =
        ProductReceiptBatch::issue(build_set_id, vec![first, second], &TestSigner).unwrap_err();

    assert!(error
        .to_string()
        .contains("duplicate artifact logical name"));
}

#[test]
fn streaming_batch_identity_matches_the_legacy_collected_payload() {
    let build_set_id = digest('A');
    let receipts = vec![
        test_receipt(0, &build_set_id),
        test_receipt(1, &build_set_id),
    ];

    assert_eq!(
        canonical_receipt_batch_sha256(&build_set_id, &receipts).unwrap(),
        canonical_receipt_batch_sha256_with_collected_ids(&build_set_id, &receipts).unwrap()
    );
}

#[test]
fn batch_issue_reads_signer_metadata_once() {
    let build_set_id = digest('A');
    let signer = MetadataCountingSigner::default();

    ProductReceiptBatch::issue_fresh_from_batch_shape_drafts(
        build_set_id.clone(),
        vec![test_draft(0, &build_set_id), test_draft(1, &build_set_id)],
        "2026-08-29T00:00:00Z".to_string(),
        &signer,
    )
    .unwrap();

    assert_eq!(signer.signer_id_calls.get(), 1);
    assert_eq!(signer.algorithm_calls.get(), 1);
    assert_eq!(signer.sign_calls.get(), 3);
}

#[test]
fn fresh_publication_proof_checks_batch_and_every_child_attestation_once() {
    let build_set_id = digest('A');
    let batch = ProductReceiptBatch::issue_fresh_after_validated_closure(
        build_set_id.clone(),
        vec![
            test_receipt(0, &build_set_id),
            test_receipt(1, &build_set_id),
        ],
        &TestSigner,
    )
    .unwrap();
    let verifier = CountingVerifier::accepting();

    let publication = batch.verify_attestations(&verifier).unwrap();

    assert_eq!(publication.batch_id(), batch_id_for(&build_set_id));
    assert_eq!(verifier.calls.get(), 3);
}

#[test]
fn fresh_publication_proof_rejects_any_untrusted_child_attestation() {
    let build_set_id = digest('A');
    let batch = ProductReceiptBatch::issue_fresh_after_validated_closure(
        build_set_id.clone(),
        vec![
            test_receipt(0, &build_set_id),
            test_receipt(1, &build_set_id),
        ],
        &TestSigner,
    )
    .unwrap();
    let verifier = CountingVerifier::rejecting(3);

    let error = batch.verify_attestations(&verifier).unwrap_err();

    assert!(error
        .to_string()
        .contains("product receipt attestation verification failed"));
    assert_eq!(verifier.calls.get(), 3);
}

#[test]
fn fresh_batch_publication_verifies_retained_raw_signatures() {
    let build_set_id = digest('A');
    let mut batch = ProductReceiptBatch::issue_fresh_after_validated_receipts(
        build_set_id.clone(),
        vec![
            ProductReceipt::issue_fresh(
                test_draft(0, &build_set_id),
                "2026-08-29T00:00:00Z",
                &TestSigner,
            )
            .unwrap(),
            ProductReceipt::issue_fresh(
                test_draft(1, &build_set_id),
                "2026-08-29T00:00:00Z",
                &TestSigner,
            )
            .unwrap(),
        ],
        &TestSigner,
    )
    .unwrap();
    batch.batch.attestation.signature_hex = "not-hex".to_string();
    for receipt in &mut batch.batch.receipts {
        receipt.attestation.signature_hex = "not-hex".to_string();
    }
    let verifier = CountingVerifier::accepting();

    batch.verify_attestations(&verifier).unwrap();

    assert_eq!(verifier.calls.get(), 3);
}

#[test]
fn fresh_batch_publication_reuses_signed_payloads() {
    let build_set_id = digest('A');
    let mut batch = ProductReceiptBatch::issue_fresh_after_validated_receipts(
        build_set_id.clone(),
        vec![
            ProductReceipt::issue_fresh(
                test_draft(0, &build_set_id),
                "2026-08-29T00:00:00Z",
                &TestSigner,
            )
            .unwrap(),
            ProductReceipt::issue_fresh(
                test_draft(1, &build_set_id),
                "2026-08-29T00:00:00Z",
                &TestSigner,
            )
            .unwrap(),
        ],
        &TestSigner,
    )
    .unwrap();
    let expected_payloads = std::iter::once(batch.batch_attestation.payload().to_vec())
        .chain(
            batch
                .receipt_attestations
                .as_ref()
                .unwrap()
                .iter()
                .map(|attestation| attestation.payload().to_vec()),
        )
        .collect();
    batch.batch.batch_id = digest('0');
    for (index, receipt) in batch.batch.receipts.iter_mut().enumerate() {
        receipt.receipt_id = digest(char::from_digit(index as u32 + 1, 10).unwrap());
    }
    let verifier = PayloadSequenceVerifier {
        calls: Cell::new(0),
        expected_payloads,
    };

    batch.verify_attestations(&verifier).unwrap();

    assert_eq!(verifier.calls.get(), 3);
}

fn batch_id_for(build_set_id: &str) -> String {
    ProductReceiptBatch::issue_after_validated_closure(
        build_set_id.to_string(),
        vec![test_receipt(0, build_set_id), test_receipt(1, build_set_id)],
        &TestSigner,
    )
    .unwrap()
    .batch_id
}

fn test_receipt(index: usize, build_set_id: &str) -> ProductReceipt {
    ProductReceipt::issue(
        test_draft(index, build_set_id),
        "2026-08-29T00:00:00Z",
        &TestSigner,
    )
    .unwrap()
}

fn test_draft(index: usize, build_set_id: &str) -> ProductReceiptDraft {
    ProductReceiptDraft {
        build_set_id: build_set_id.to_string(),
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
            package: format!("zircon-product-{index}"),
            bin: Some(format!("zircon_product_{index}")),
            features: vec![format!("product-feature-{index}")],
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: "0.1.0".to_string(),
            worker_id: "test-worker".to_string(),
            operation_id: format!("test-operation-{index}"),
        },
        build_products: vec![ReceiptArtifact {
            logical_name: format!("product-artifact-{index}"),
            relative_path: format!("product-{index}/zircon_product.exe"),
            kind: ArtifactKind::Executable,
            sha256: digest('3'),
            byte_length: 4_096,
        }],
        runtime_dependencies: Vec::new(),
        symbols: Vec::new(),
        sbom: None,
    }
}

fn digest(character: char) -> String {
    std::iter::repeat(character).take(64).collect()
}
