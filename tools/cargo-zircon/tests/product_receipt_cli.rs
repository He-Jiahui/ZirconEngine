use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use cargo_zircon::build::product_build::ProductBuildDraftBatch;
use cargo_zircon::build::receipt::{
    ArtifactKind, BuildAction, Ed25519ProductReceiptSigner, ProducerIdentity, ProductReceipt,
    ProductReceiptBatch, ProductReceiptClosure, ProductReceiptDraft, ProductReceiptTrustRegistry,
    ReceiptArtifact, ReceiptArtifactSource, TargetProfile, ToolchainSet, ToolchainSource,
};
use ring::rand::SystemRandom;
use ring::signature::Ed25519KeyPair;
use sha2::{Digest, Sha256};

const CREATED_UTC: &str = "2026-08-28T08:30:00.0000000Z";

fn sha256(letter: char) -> String {
    std::iter::repeat_n(letter, 64).collect()
}

fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect()
}

fn valid_draft() -> ProductReceiptDraft {
    ProductReceiptDraft {
        build_set_id: sha256('A'),
        toolchain: ToolchainSet::new(
            sha256('B'),
            sha256('C'),
            Some(sha256('D')),
            sha256('E'),
            sha256('F'),
        )
        .unwrap(),
        target_profile: TargetProfile {
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            cargo_profile: "release".to_string(),
            codegen_flags_digest: sha256('1'),
            cargo_graph_digest: sha256('2'),
        },
        action: BuildAction {
            package: "zircon_app".to_string(),
            bin: Some("zircon_runtime".to_string()),
            features: vec!["target-client".to_string()],
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            worker_id: "windows-worker-01".to_string(),
            operation_id: "product-receipt-test".to_string(),
        },
        build_products: vec![ReceiptArtifact {
            logical_name: "runtime-executable".to_string(),
            relative_path: "bin/zircon_runtime.exe".to_string(),
            kind: ArtifactKind::Executable,
            sha256: sha256('3'),
            byte_length: 42,
        }],
        runtime_dependencies: Vec::new(),
        symbols: Vec::new(),
        sbom: None,
    }
}

#[test]
fn receipt_batches_reject_feature_order_equivalent_actions() {
    let mut first = valid_draft();
    first.action.features = vec!["feature-a".to_string(), "feature-b".to_string()];
    let mut second = first.clone();
    second.action.features.reverse();
    second.producer.operation_id = "product-receipt-test-second".to_string();
    second.build_products[0].logical_name = "runtime-executable-second".to_string();
    second.build_products[0].relative_path = "bin/zircon_runtime_second.exe".to_string();

    let draft_batch = ProductBuildDraftBatch {
        schema_version: 1,
        draft_batch_kind: "zircon_product_build_draft_batch".to_string(),
        build_set_id: first.build_set_id.clone(),
        drafts: vec![first.clone(), second.clone()],
    };
    assert!(draft_batch
        .handoff_sha256()
        .unwrap_err()
        .to_string()
        .contains("duplicate build action"));

    let key = generated_key();
    let signer = Ed25519ProductReceiptSigner::from_pkcs8("build-worker-01", key.as_ref()).unwrap();
    let receipts = vec![
        ProductReceipt::issue(first, CREATED_UTC, &signer).unwrap(),
        ProductReceipt::issue(second, CREATED_UTC, &signer).unwrap(),
    ];
    assert!(ProductReceiptBatch::issue(sha256('A'), receipts, &signer)
        .unwrap_err()
        .to_string()
        .contains("duplicate canonical build action"));
}

fn generated_key() -> ring::pkcs8::Document {
    Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap()
}

fn trust_registry(signer_id: &str, public_key_hex: &str, disabled: bool) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schema_version": 1,
        "trust_registry_kind": "zircon_product_receipt_trust_registry",
        "issuers": [{
            "signer_id": signer_id,
            "algorithm": "ed25519-v1",
            "public_key_hex": public_key_hex,
            "disabled": disabled
        }]
    }))
    .unwrap()
}

fn temporary_directory(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "cargo-zircon-{label}-{}-{nanos}",
        std::process::id()
    ))
}

fn valid_closure(directory: &std::path::Path) -> ProductReceiptClosure {
    let cargo_path = directory.join("cargo.exe");
    let rustc_path = directory.join("rustc.exe");
    let linker_path = directory.join("link.exe");
    let product_path = directory.join("zircon_runtime.exe");
    fs::write(&cargo_path, b"fixture cargo bytes").unwrap();
    fs::write(&rustc_path, b"fixture rustc bytes").unwrap();
    fs::write(&linker_path, b"fixture linker bytes").unwrap();
    fs::write(&product_path, b"fixture product bytes").unwrap();

    ProductReceiptClosure {
        build_set_id: sha256('A'),
        toolchain: ToolchainSource {
            cargo_path,
            rustc_path,
            linker_path: Some(linker_path),
            sdk_fingerprint: sha256('E'),
            environment_digest: sha256('F'),
        },
        target_profile: TargetProfile {
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            cargo_profile: "release".to_string(),
            codegen_flags_digest: sha256('1'),
            cargo_graph_digest: sha256('2'),
        },
        action: BuildAction {
            package: "zircon_app".to_string(),
            bin: Some("zircon_runtime".to_string()),
            features: vec!["target-client".to_string()],
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            worker_id: "windows-worker-01".to_string(),
            operation_id: "product-receipt-test".to_string(),
        },
        build_products: vec![ReceiptArtifactSource {
            logical_name: "runtime-executable".to_string(),
            relative_path: "bin/zircon_runtime.exe".to_string(),
            kind: ArtifactKind::Executable,
            source_path: product_path,
        }],
        runtime_dependencies: Vec::new(),
        symbols: Vec::new(),
        sbom: None,
    }
}

#[test]
fn ed25519_authority_verifies_only_a_trusted_enabled_issuer() {
    let key = generated_key();
    let signer = Ed25519ProductReceiptSigner::from_pkcs8("build-worker-01", key.as_ref()).unwrap();
    let receipt = ProductReceipt::issue(valid_draft(), CREATED_UTC, &signer).unwrap();
    let registry = ProductReceiptTrustRegistry::from_json(&trust_registry(
        "build-worker-01",
        signer.public_key_hex(),
        false,
    ))
    .unwrap();

    receipt.verify_attestation(&registry).unwrap();

    let unknown = ProductReceiptTrustRegistry::from_json(&trust_registry(
        "other-worker",
        signer.public_key_hex(),
        false,
    ))
    .unwrap();
    assert!(receipt
        .verify_attestation(&unknown)
        .unwrap_err()
        .to_string()
        .contains("not trusted"));

    let disabled = ProductReceiptTrustRegistry::from_json(&trust_registry(
        "build-worker-01",
        signer.public_key_hex(),
        true,
    ))
    .unwrap();
    assert!(receipt
        .verify_attestation(&disabled)
        .unwrap_err()
        .to_string()
        .contains("disabled"));
}

#[test]
fn ed25519_authority_rejects_noncanonical_signer_ids() {
    let key = generated_key();

    for signer_id in ["", "Build-Worker-01", "build worker", "-build-worker"] {
        let error = Ed25519ProductReceiptSigner::from_pkcs8(signer_id, key.as_ref())
            .err()
            .unwrap();
        assert!(error.to_string().contains("stable lowercase identifier"));
    }
}

#[test]
fn product_receipt_cli_issues_without_overwrite_and_verifies_against_trust_registry() {
    let directory = temporary_directory("product-receipt-cli");
    fs::create_dir(&directory).unwrap();
    let closure_path = directory.join("closure.json");
    let key_path = directory.join("worker.pk8");
    let registry_path = directory.join("trust.json");
    let receipt_path = directory.join("receipt.json");
    let artifact_root = directory.join("materialized-product");

    let key = generated_key();
    let signer = Ed25519ProductReceiptSigner::from_pkcs8("build-worker-01", key.as_ref()).unwrap();
    fs::write(
        &closure_path,
        serde_json::to_vec(&valid_closure(&directory)).unwrap(),
    )
    .unwrap();
    fs::write(&key_path, key.as_ref()).unwrap();
    fs::write(
        &registry_path,
        trust_registry("build-worker-01", signer.public_key_hex(), false),
    )
    .unwrap();
    fs::create_dir_all(artifact_root.join("bin")).unwrap();
    fs::write(
        artifact_root.join("bin/zircon_runtime.exe"),
        b"fixture product bytes",
    )
    .unwrap();

    let issue = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "issue",
            "--closure",
            closure_path.to_str().unwrap(),
            "--private-key",
            key_path.to_str().unwrap(),
            "--signer-id",
            "build-worker-01",
            "--created-utc",
            CREATED_UTC,
            "--output",
            receipt_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        issue.status.success(),
        "{}",
        String::from_utf8_lossy(&issue.stderr)
    );

    let receipt_bytes = fs::read(&receipt_path).unwrap();
    let receipt: ProductReceipt = serde_json::from_slice(&receipt_bytes).unwrap();
    assert!(String::from_utf8_lossy(&issue.stdout).contains(&receipt.receipt_id));
    assert_eq!(
        receipt.build_products[0].sha256,
        Sha256::digest(b"fixture product bytes")
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<String>()
    );
    assert_eq!(
        receipt.toolchain.cargo_sha256,
        Sha256::digest(b"fixture cargo bytes")
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<String>()
    );

    let second_issue = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "issue",
            "--closure",
            closure_path.to_str().unwrap(),
            "--private-key",
            key_path.to_str().unwrap(),
            "--signer-id",
            "build-worker-01",
            "--created-utc",
            CREATED_UTC,
            "--output",
            receipt_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!second_issue.status.success());
    assert_eq!(fs::read(&receipt_path).unwrap(), receipt_bytes);

    let verify = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "verify",
            "--receipt",
            receipt_path.to_str().unwrap(),
            "--trust-registry",
            registry_path.to_str().unwrap(),
            "--artifact-root",
            artifact_root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8_lossy(&verify.stdout).contains(&receipt.receipt_id));

    #[cfg(windows)]
    {
        let junction = directory.join("artifact-root-parent-junction");
        let create_junction = Command::new("cmd")
            .args([
                "/D",
                "/C",
                "mklink",
                "/J",
                junction.to_str().unwrap(),
                directory.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(
            create_junction.status.success(),
            "{}",
            String::from_utf8_lossy(&create_junction.stderr)
        );
        let junction_artifact_root = junction.join("materialized-product");
        let junction_verify = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
            .args([
                "product-receipt",
                "verify",
                "--receipt",
                receipt_path.to_str().unwrap(),
                "--trust-registry",
                registry_path.to_str().unwrap(),
                "--artifact-root",
                junction_artifact_root.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(!junction_verify.status.success());
        assert!(String::from_utf8_lossy(&junction_verify.stderr).contains("reparse point"));
        fs::remove_dir(junction).unwrap();
    }

    fs::write(
        artifact_root.join("bin/zircon_runtime.exe"),
        b"tampered product bytes",
    )
    .unwrap();
    let tampered_verify = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "verify",
            "--receipt",
            receipt_path.to_str().unwrap(),
            "--trust-registry",
            registry_path.to_str().unwrap(),
            "--artifact-root",
            artifact_root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!tampered_verify.status.success());
    assert!(String::from_utf8_lossy(&tampered_verify.stderr).contains("does not match"));

    fs::write(
        artifact_root.join("bin/zircon_runtime.exe"),
        b"fixture product bytes",
    )
    .unwrap();
    fs::write(artifact_root.join("bin/undeclared.dll"), b"not in receipt").unwrap();
    let undeclared_verify = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "verify",
            "--receipt",
            receipt_path.to_str().unwrap(),
            "--trust-registry",
            registry_path.to_str().unwrap(),
            "--artifact-root",
            artifact_root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!undeclared_verify.status.success());
    assert!(String::from_utf8_lossy(&undeclared_verify.stderr).contains("undeclared artifact"));
    fs::remove_file(artifact_root.join("bin/undeclared.dll")).unwrap();

    fs::create_dir(artifact_root.join("undeclared-empty-directory")).unwrap();
    let undeclared_directory_verify = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "verify",
            "--receipt",
            receipt_path.to_str().unwrap(),
            "--trust-registry",
            registry_path.to_str().unwrap(),
            "--artifact-root",
            artifact_root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!undeclared_directory_verify.status.success());
    assert!(String::from_utf8_lossy(&undeclared_directory_verify.stderr)
        .contains("undeclared directory"));
    fs::remove_dir(artifact_root.join("undeclared-empty-directory")).unwrap();

    let mut receipt_with_unknown_field: serde_json::Value =
        serde_json::from_slice(&receipt_bytes).unwrap();
    receipt_with_unknown_field
        .as_object_mut()
        .unwrap()
        .insert("untrusted_note".to_string(), serde_json::json!("ignored?"));
    let unknown_receipt_path = directory.join("receipt-with-unknown-field.json");
    fs::write(
        &unknown_receipt_path,
        serde_json::to_vec(&receipt_with_unknown_field).unwrap(),
    )
    .unwrap();
    let unknown_verify = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "verify",
            "--receipt",
            unknown_receipt_path.to_str().unwrap(),
            "--trust-registry",
            registry_path.to_str().unwrap(),
            "--artifact-root",
            artifact_root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!unknown_verify.status.success());
    assert!(String::from_utf8_lossy(&unknown_verify.stderr).contains("unknown field"));

    fs::remove_dir_all(directory).unwrap();
}

#[cfg(windows)]
#[test]
fn product_receipt_cli_issues_and_verifies_one_exact_four_artifact_batch() {
    let directory = temporary_directory("product-receipt-batch-cli");
    fs::create_dir(&directory).unwrap();
    let draft_batch_path = directory.join("draft-batch.json");
    let key_path = directory.join("worker.pk8");
    let registry_path = directory.join("trust.json");
    let receipt_batch_path = directory.join("receipt-batch.json");
    let artifact_root = directory.join("materialized-products");

    let runtime_executable = b"runtime executable bytes";
    let runtime_library = b"runtime library bytes";
    let editor_executable = b"editor executable bytes";
    let editor_library = b"editor library bytes";

    let mut runtime = valid_draft();
    runtime.producer.operation_id = "runtime-product-build".to_string();
    runtime.build_products = vec![ReceiptArtifact {
        logical_name: "runtime-executable".to_string(),
        relative_path: "runtime/zircon_runtime.exe".to_string(),
        kind: ArtifactKind::Executable,
        sha256: sha256_bytes(runtime_executable),
        byte_length: runtime_executable.len() as u64,
    }];
    runtime.runtime_dependencies = vec![ReceiptArtifact {
        logical_name: "runtime-library-runtime".to_string(),
        relative_path: "runtime/zircon_runtime.dll".to_string(),
        kind: ArtifactKind::DynamicLibrary,
        sha256: sha256_bytes(runtime_library),
        byte_length: runtime_library.len() as u64,
    }];

    let mut editor = runtime.clone();
    editor.action.bin = Some("zircon_editor".to_string());
    editor.action.features = vec!["target-editor-host".to_string()];
    editor.producer.operation_id = "editor-product-build".to_string();
    editor.build_products = vec![ReceiptArtifact {
        logical_name: "editor-executable".to_string(),
        relative_path: "editor/zircon_editor.exe".to_string(),
        kind: ArtifactKind::Executable,
        sha256: sha256_bytes(editor_executable),
        byte_length: editor_executable.len() as u64,
    }];
    editor.runtime_dependencies = vec![ReceiptArtifact {
        logical_name: "runtime-library-editor".to_string(),
        relative_path: "editor/zircon_runtime.dll".to_string(),
        kind: ArtifactKind::DynamicLibrary,
        sha256: sha256_bytes(editor_library),
        byte_length: editor_library.len() as u64,
    }];

    let build_set_id = runtime.build_set_id.clone();
    let draft_batch = ProductBuildDraftBatch {
        schema_version: 1,
        draft_batch_kind: "zircon_product_build_draft_batch".to_string(),
        build_set_id: build_set_id.clone(),
        drafts: vec![runtime, editor],
    };
    let draft_handoff_sha256 = draft_batch.handoff_sha256().unwrap();
    fs::write(&draft_batch_path, serde_json::to_vec(&draft_batch).unwrap()).unwrap();

    let key = generated_key();
    let signer = Ed25519ProductReceiptSigner::from_pkcs8("build-worker-01", key.as_ref()).unwrap();
    fs::write(&key_path, key.as_ref()).unwrap();
    fs::write(
        &registry_path,
        trust_registry("build-worker-01", signer.public_key_hex(), false),
    )
    .unwrap();

    let issue = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "issue-draft-batch",
            "--draft-batch",
            draft_batch_path.to_str().unwrap(),
            "--expected-draft-sha256",
            &draft_handoff_sha256,
            "--private-key",
            key_path.to_str().unwrap(),
            "--trust-registry",
            registry_path.to_str().unwrap(),
            "--signer-id",
            "build-worker-01",
            "--created-utc",
            CREATED_UTC,
            "--output",
            receipt_batch_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        issue.status.success(),
        "{}",
        String::from_utf8_lossy(&issue.stderr)
    );
    let batch: ProductReceiptBatch =
        serde_json::from_slice(&fs::read(&receipt_batch_path).unwrap()).unwrap();
    assert_eq!(batch.build_set_id, build_set_id);
    assert_eq!(batch.receipts.len(), 2);
    batch.verify_attestations(&signer).unwrap();
    let mut reordered_batch = batch.clone();
    reordered_batch.receipts.swap(0, 1);
    assert!(reordered_batch
        .verify_attestations(&signer)
        .unwrap_err()
        .to_string()
        .contains("declared receipt set"));

    fs::create_dir_all(artifact_root.join("runtime")).unwrap();
    fs::create_dir_all(artifact_root.join("editor")).unwrap();
    fs::write(
        artifact_root.join("runtime/zircon_runtime.exe"),
        runtime_executable,
    )
    .unwrap();
    fs::write(
        artifact_root.join("runtime/zircon_runtime.dll"),
        runtime_library,
    )
    .unwrap();
    fs::write(
        artifact_root.join("editor/zircon_editor.exe"),
        editor_executable,
    )
    .unwrap();
    fs::write(
        artifact_root.join("editor/zircon_runtime.dll"),
        editor_library,
    )
    .unwrap();

    let verify = Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "verify-batch",
            "--receipt-batch",
            receipt_batch_path.to_str().unwrap(),
            "--trust-registry",
            registry_path.to_str().unwrap(),
            "--artifact-root",
            artifact_root.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(String::from_utf8_lossy(&verify.stdout).contains(&batch.batch_id));

    fs::remove_dir_all(directory).unwrap();
}
