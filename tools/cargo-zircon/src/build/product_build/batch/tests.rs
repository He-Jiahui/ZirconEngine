use std::collections::HashSet;
use std::path::PathBuf;

use super::{validate_build_batch_request, ProductBuildBatchRequest, ProductBuildDraftBatch};
use crate::build::product_build::{
    CargoRuntimeDependencyDeclaration, ProductArtifactDeclaration, ProductBuildProducer,
    ProductBuildRequest, ProductBuildSdkSource, ProductBuildTarget, ProductBuildToolchain,
};
use crate::build::receipt::{
    canonical::bytes_to_hex, ArtifactKind, BuildAction, ProducerIdentity, ProductReceiptDraft,
    ProductReceiptSigner, ReceiptArtifact, TargetProfile, ToolchainSet,
};
use sha2::{Digest, Sha256};

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

#[test]
fn four_product_batch_accepts_unique_actions_on_one_build_set() {
    let mut request = ProductBuildBatchRequest {
        schema_version: 1,
        builds: vec![
            test_build("runtime"),
            test_build("editor"),
            test_build("hub"),
            test_build("workbench"),
        ],
    };

    validate_build_batch_request(&mut request).unwrap();

    assert_eq!(request.builds.len(), 4);
    assert!(request
        .builds
        .iter()
        .all(|build| build.build_set_manifest_path == PathBuf::from("build-set.json")));
    let artifact_names: HashSet<_> = request
        .builds
        .iter()
        .flat_map(|build| {
            std::iter::once(build.product.logical_name.as_str()).chain(
                build
                    .runtime_dependencies
                    .iter()
                    .map(|dependency| dependency.logical_name.as_str()),
            )
        })
        .collect();
    assert_eq!(artifact_names.len(), 8);
}

#[test]
fn batch_rejects_a_second_build_set_before_running_actions() {
    let mut editor = test_build("editor");
    editor.build_set_manifest_path = PathBuf::from("other-build-set.json");
    let mut request = ProductBuildBatchRequest {
        schema_version: 1,
        builds: vec![test_build("runtime"), editor],
    };

    let error = validate_build_batch_request(&mut request).unwrap_err();

    assert!(error
        .to_string()
        .contains("product build batch must use one BuildSet manifest"));
}

#[test]
fn batch_rejects_a_duplicate_build_action_before_running_cargo() {
    let runtime = test_build("runtime");
    let mut duplicate = test_build("editor");
    duplicate.action = runtime.action.clone();
    let mut request = ProductBuildBatchRequest {
        schema_version: 1,
        builds: vec![runtime, duplicate],
    };

    let error = validate_build_batch_request(&mut request).unwrap_err();

    assert!(error
        .to_string()
        .contains("product build batch contains a duplicate build action"));
}

#[test]
fn product_build_batch_issue_reuses_validated_artifact_uniqueness() {
    let batch = test_draft_batch();

    let receipt_batch = batch.issue("2026-08-29T00:00:00Z", &TestSigner).unwrap();

    assert_eq!(receipt_batch.receipts.len(), 2);
    assert!(receipt_batch
        .receipts
        .iter()
        .all(|receipt| receipt.created_utc == "2026-08-29T00:00:00Z"));
}

#[test]
fn product_build_batch_issue_rejects_duplicate_paths_before_reusing_the_proof() {
    let mut batch = test_draft_batch();
    batch.drafts[1].build_products[0].relative_path =
        batch.drafts[0].build_products[0].relative_path.clone();

    let error = batch
        .issue("2026-08-29T00:00:00Z", &TestSigner)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("product build batch contains duplicate artifact relative path"));
}

#[test]
fn product_build_batch_issue_still_validates_artifact_fields() {
    let mut batch = test_draft_batch();
    batch.drafts[0].build_products[0].sha256 = "not-a-digest".to_string();

    let error = batch
        .issue("2026-08-29T00:00:00Z", &TestSigner)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("artifact SHA-256 must be a SHA-256 hex digest"));
}

#[test]
fn product_build_batch_issue_rejects_invalid_shared_created_utc() {
    let error = test_draft_batch()
        .issue("2026-02-30T00:00:00Z", &TestSigner)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("created_utc must be an ISO-8601 UTC timestamp"));
}

#[test]
fn draft_batch_write_returns_the_validated_handoff_digest() {
    let batch = test_draft_batch();
    let expected_handoff = batch.handoff_sha256().unwrap();
    let output = std::env::temp_dir().join(format!(
        "cargo-zircon-draft-batch-write-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let plain_output = output.with_extension("plain.json");

    let handoff = batch.write_new_with_handoff_sha256(&output).unwrap();
    let written_bytes = std::fs::read(&output).unwrap();
    let written: ProductBuildDraftBatch = serde_json::from_slice(&written_bytes).unwrap();

    assert_eq!(handoff, expected_handoff);
    assert_eq!(handoff, bytes_to_hex(&Sha256::digest(&written_bytes)));
    assert_eq!(written, batch);
    ProductBuildDraftBatch::parse_and_verify_handoff_sha256(&written_bytes, &handoff).unwrap();
    ProductBuildDraftBatch::parse_and_verify_handoff_sha256(
        &serde_json::to_vec_pretty(&batch).unwrap(),
        &handoff,
    )
    .unwrap();
    assert!(batch.write_new_with_handoff_sha256(&output).is_err());
    batch.write_new(&plain_output).unwrap();
    assert_eq!(
        expected_handoff,
        bytes_to_hex(&Sha256::digest(std::fs::read(&plain_output).unwrap()))
    );
    std::fs::remove_file(output).unwrap();
    std::fs::remove_file(plain_output).unwrap();
}

fn test_draft_batch() -> ProductBuildDraftBatch {
    let build_set_id = digest('A');
    ProductBuildDraftBatch {
        schema_version: 1,
        draft_batch_kind: "zircon_product_build_draft_batch".to_string(),
        build_set_id: build_set_id.clone(),
        drafts: vec![
            test_receipt_draft(0, &build_set_id),
            test_receipt_draft(1, &build_set_id),
        ],
    }
}

fn test_receipt_draft(index: usize, build_set_id: &str) -> ProductReceiptDraft {
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
            features: vec![format!("product-{index}")],
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: "0.1.0".to_string(),
            worker_id: "test-worker".to_string(),
            operation_id: format!("test-operation-{index}"),
        },
        build_products: vec![ReceiptArtifact {
            logical_name: format!("product-{index}"),
            relative_path: format!("product-{index}/zircon_product_{index}.exe"),
            kind: ArtifactKind::Executable,
            sha256: digest(char::from_digit(index as u32 + 3, 16).unwrap()),
            byte_length: 4_096,
        }],
        runtime_dependencies: Vec::new(),
        symbols: Vec::new(),
        sbom: None,
    }
}

fn digest(character: char) -> String {
    std::iter::repeat_n(character, 64).collect()
}

fn test_build(product: &str) -> ProductBuildRequest {
    ProductBuildRequest {
        schema_version: 1,
        build_set_manifest_path: PathBuf::from("build-set.json"),
        manifest_path: "Cargo.toml".to_string(),
        target_directory: PathBuf::from(format!("target-{product}")),
        toolchain: ProductBuildToolchain {
            cargo_path: PathBuf::from("cargo.exe"),
            rustc_path: PathBuf::from("rustc.exe"),
            linker_path: Some(PathBuf::from("link.exe")),
            sdk_files: vec![ProductBuildSdkSource {
                logical_name: "windows-sdk".to_string(),
                source_path: PathBuf::from("kernel32.lib"),
            }],
        },
        target: ProductBuildTarget {
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            cargo_profile: "release".to_string(),
            rustflags: Vec::new(),
        },
        action: BuildAction {
            package: "zircon-app".to_string(),
            bin: Some(format!("zircon_{product}")),
            features: vec![product.to_string()],
        },
        producer: ProductBuildProducer {
            worker_id: "test-worker".to_string(),
            operation_id: format!("test-{product}-operation"),
        },
        product: ProductArtifactDeclaration {
            logical_name: format!("{product}-executable"),
            relative_path: format!("{product}/zircon_{product}.exe"),
            symbol_relative_directory: format!("{product}/symbols"),
        },
        environment_policy: "windows-msvc-v1".to_string(),
        runtime_dependencies: vec![CargoRuntimeDependencyDeclaration {
            logical_name: format!("runtime-library-{product}"),
            relative_path: format!("{product}/zircon_runtime.dll"),
            package: "zircon-runtime".to_string(),
            target: "zircon_runtime".to_string(),
            artifact_file_name: "zircon_runtime.dll".to_string(),
        }],
        sbom: None,
    }
}
