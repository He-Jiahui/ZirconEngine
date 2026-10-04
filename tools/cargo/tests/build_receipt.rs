use cargo_zircon::build::receipt::{
    ArtifactKind, BuildAction, ProducerIdentity, ProductReceipt, ProductReceiptDraft,
    ProductReceiptSigner, ProductReceiptVerifier, ReceiptArtifact, TargetProfile, ToolchainSet,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

struct FixtureSigner;

impl ProductReceiptSigner for FixtureSigner {
    fn signer_id(&self) -> &str {
        "fixture-worker-key"
    }

    fn algorithm(&self) -> &str {
        "fixture-signature-v1"
    }

    fn sign(&self, attestation_payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        assert!(!attestation_payload.is_empty());
        Ok(vec![0xA5, 0x5A])
    }
}

struct FixtureVerifier;

impl ProductReceiptVerifier for FixtureVerifier {
    fn verify(
        &self,
        signer_id: &str,
        algorithm: &str,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if signer_id != "fixture-worker-key" || algorithm != "fixture-signature-v1" {
            Err("fixture signer metadata did not match".into())
        } else if attestation_payload.is_empty() {
            Err("fixture signing payload was empty".into())
        } else if signature == [0xA5, 0x5A] {
            Ok(())
        } else {
            Err("fixture signature did not match".into())
        }
    }
}

struct PayloadDigestAuthority;

impl ProductReceiptSigner for PayloadDigestAuthority {
    fn signer_id(&self) -> &str {
        "payload-digest-key"
    }

    fn algorithm(&self) -> &str {
        "sha256-fixture"
    }

    fn sign(&self, attestation_payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(Sha256::digest(attestation_payload).to_vec())
    }
}

impl ProductReceiptVerifier for PayloadDigestAuthority {
    fn verify(
        &self,
        _signer_id: &str,
        _algorithm: &str,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let expected = Sha256::digest(attestation_payload);
        if &expected[..] == signature {
            Ok(())
        } else {
            Err("payload digest signature did not match".into())
        }
    }
}

fn sha256(letter: char) -> String {
    std::iter::repeat(letter).take(64).collect()
}

fn artifact(
    logical_name: &str,
    relative_path: &str,
    kind: ArtifactKind,
    byte_length: u64,
) -> ReceiptArtifact {
    ReceiptArtifact {
        logical_name: logical_name.to_string(),
        relative_path: relative_path.to_string(),
        kind,
        sha256: sha256('C'),
        byte_length,
    }
}

fn toolchain(cargo_sha256: String) -> ToolchainSet {
    ToolchainSet::new(
        cargo_sha256,
        sha256('E'),
        Some(sha256('F')),
        sha256('1'),
        sha256('2'),
    )
    .unwrap()
}

fn toolchain_without_linker(cargo_sha256: String) -> ToolchainSet {
    ToolchainSet::new(cargo_sha256, sha256('E'), None, sha256('1'), sha256('2')).unwrap()
}

fn valid_draft() -> ProductReceiptDraft {
    ProductReceiptDraft {
        build_set_id: sha256('A'),
        toolchain: toolchain(sha256('D')),
        target_profile: TargetProfile {
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            cargo_profile: "release".to_string(),
            codegen_flags_digest: sha256('3'),
            cargo_graph_digest: sha256('4'),
        },
        action: BuildAction {
            package: "zircon_app".to_string(),
            bin: Some("zircon_runtime".to_string()),
            features: vec!["target-client".to_string(), "platform-winit".to_string()],
        },
        producer: ProducerIdentity {
            tool: "cargo-zircon".to_string(),
            tool_version: "0.1.0".to_string(),
            worker_id: "windows-worker-01".to_string(),
            operation_id: "build-0001".to_string(),
        },
        build_products: vec![artifact(
            "runtime-executable",
            "bin/zircon_runtime.exe",
            ArtifactKind::Executable,
            42,
        )],
        runtime_dependencies: vec![artifact(
            "render-backend",
            "bin/wgpu.dll",
            ArtifactKind::DynamicLibrary,
            23,
        )],
        symbols: vec![artifact(
            "runtime-symbols",
            "symbols/zircon_runtime.pdb",
            ArtifactKind::SymbolFile,
            17,
        )],
        sbom: Some(artifact(
            "sbom",
            "metadata/sbom.spdx.json",
            ArtifactKind::Sbom,
            71,
        )),
    }
}

fn temporary_receipt_directory() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "cargo-zircon-build-receipt-test-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
fn issues_a_signed_receipt_bound_to_the_complete_build_closure() {
    let receipt = ProductReceipt::issue(
        valid_draft(),
        "2026-08-25T07:30:00.0000000Z",
        &FixtureSigner,
    )
    .unwrap();

    receipt.verify_integrity().unwrap();
    receipt.verify_attestation(&FixtureVerifier).unwrap();
    assert_eq!(receipt.schema_version, 1);
    assert_eq!(receipt.receipt_kind, "zircon_product_receipt");
    assert_eq!(receipt.attestation.signer_id, "fixture-worker-key");
    assert_eq!(receipt.attestation.algorithm, "fixture-signature-v1");
    assert_eq!(receipt.attestation.signature_hex, "A55A");
    assert_eq!(receipt.build_products.len(), 1);
    assert_eq!(receipt.runtime_dependencies.len(), 1);
    assert_eq!(receipt.symbols.len(), 1);
    assert_eq!(receipt.sbom.as_ref().unwrap().logical_name, "sbom");
}

#[test]
fn receipt_identity_changes_when_the_declared_toolchain_changes() {
    let first = ProductReceipt::issue(
        valid_draft(),
        "2026-08-25T07:30:00.0000000Z",
        &FixtureSigner,
    )
    .unwrap();
    let mut changed = valid_draft();
    changed.toolchain = toolchain(sha256('9'));
    let second =
        ProductReceipt::issue(changed, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap();

    assert_ne!(first.receipt_id, second.receipt_id);
}

#[test]
fn rejects_a_toolchain_set_id_that_does_not_match_its_components() {
    let mut draft = valid_draft();
    draft.toolchain.toolchain_set_id = sha256('B');

    let error =
        ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap_err();

    assert!(error.to_string().contains("ToolchainSet identity"));
}

#[test]
fn receipt_identity_changes_when_the_declared_target_profile_changes() {
    let first = ProductReceipt::issue(
        valid_draft(),
        "2026-08-25T07:30:00.0000000Z",
        &FixtureSigner,
    )
    .unwrap();
    let mut changed = valid_draft();
    changed.target_profile.codegen_flags_digest = sha256('8');
    let second =
        ProductReceipt::issue(changed, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap();

    assert_ne!(first.receipt_id, second.receipt_id);
}

#[test]
fn receipt_identity_binds_every_declared_build_closure_partition() {
    let baseline = ProductReceipt::issue(
        valid_draft(),
        "2026-08-25T07:30:00.0000000Z",
        &FixtureSigner,
    )
    .unwrap();
    let mut changed_build_set = valid_draft();
    changed_build_set.build_set_id = sha256('7');
    let mut changed_action = valid_draft();
    changed_action.action.package = "zircon_editor".to_string();
    let mut changed_producer = valid_draft();
    changed_producer.producer.operation_id = "build-0002".to_string();
    let mut changed_product = valid_draft();
    changed_product.build_products[0].sha256 = sha256('8');
    let mut changed_runtime_dependency = valid_draft();
    changed_runtime_dependency.runtime_dependencies[0].sha256 = sha256('9');
    let mut changed_symbols = valid_draft();
    changed_symbols.symbols[0].sha256 = sha256('0');
    let mut changed_sbom = valid_draft();
    changed_sbom.sbom = None;

    for draft in [
        changed_build_set,
        changed_action,
        changed_producer,
        changed_product,
        changed_runtime_dependency,
        changed_symbols,
        changed_sbom,
    ] {
        let receipt =
            ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap();

        assert_ne!(receipt.receipt_id, baseline.receipt_id);
    }
}

#[test]
fn rejects_duplicate_artifact_names_across_the_product_closure() {
    let mut draft = valid_draft();
    draft.runtime_dependencies.push(artifact(
        "runtime-executable",
        "bin/duplicate.exe",
        ArtifactKind::DynamicLibrary,
        11,
    ));

    let error =
        ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap_err();

    assert!(error
        .to_string()
        .contains("duplicate artifact logical name"));
}

#[test]
fn rejects_duplicate_artifact_paths_across_the_product_closure() {
    let mut draft = valid_draft();
    draft.runtime_dependencies[0].relative_path = draft.build_products[0].relative_path.clone();

    let error =
        ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap_err();

    assert!(error
        .to_string()
        .contains("duplicate artifact relative path"));
}

#[test]
fn rejects_artifact_kinds_outside_their_receipt_partition() {
    let mut invalid_drafts = Vec::new();
    let mut build_product = valid_draft();
    build_product.build_products[0].kind = ArtifactKind::Resource;
    invalid_drafts.push(build_product);
    let mut runtime_dependency = valid_draft();
    runtime_dependency.runtime_dependencies[0].kind = ArtifactKind::Executable;
    invalid_drafts.push(runtime_dependency);
    let mut symbol = valid_draft();
    symbol.symbols[0].kind = ArtifactKind::DynamicLibrary;
    invalid_drafts.push(symbol);
    let mut sbom = valid_draft();
    sbom.sbom.as_mut().unwrap().kind = ArtifactKind::Resource;
    invalid_drafts.push(sbom);

    for draft in invalid_drafts {
        let error = ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner)
            .unwrap_err();
        assert!(error.to_string().contains("artifact kind"));
    }
}

#[test]
fn rejects_an_artifact_path_that_escapes_the_declared_closure() {
    let mut draft = valid_draft();
    draft.build_products[0].relative_path = "../outside/zircon_runtime.exe".to_string();

    let error =
        ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap_err();

    assert!(error.to_string().contains("artifact relative path"));
}

#[test]
fn rejects_a_noncanonical_artifact_path() {
    let mut draft = valid_draft();
    draft.build_products[0].relative_path = "./bin/zircon_runtime.exe".to_string();

    let error =
        ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap_err();

    assert!(error.to_string().contains("artifact relative path"));
}

#[test]
fn rejects_artifact_paths_with_equivalent_separator_forms() {
    for relative_path in ["bin//zircon_runtime.exe", "bin/zircon_runtime.exe/"] {
        let mut draft = valid_draft();
        draft.build_products[0].relative_path = relative_path.to_string();

        let error = ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner)
            .unwrap_err();

        assert!(error.to_string().contains("artifact relative path"));
    }
}

#[test]
fn rejects_a_receipt_with_an_invalid_utc_timestamp() {
    let error =
        ProductReceipt::issue(valid_draft(), "2026-13-32T24:61:61Z", &FixtureSigner).unwrap_err();

    assert!(error.to_string().contains("ISO-8601 UTC timestamp"));
}

#[test]
fn rejects_a_windows_receipt_without_a_linker_fingerprint() {
    let mut draft = valid_draft();
    draft.toolchain = toolchain_without_linker(sha256('D'));

    let error =
        ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap_err();

    assert!(error.to_string().contains("linker fingerprint"));
}

#[test]
fn permits_a_non_windows_receipt_without_a_linker_fingerprint() {
    let mut draft = valid_draft();
    draft.target_profile.target_triple = "wasm32-unknown-unknown".to_string();
    draft.toolchain = toolchain_without_linker(sha256('D'));

    let receipt =
        ProductReceipt::issue(draft, "2026-08-25T07:30:00.0000000Z", &FixtureSigner).unwrap();

    receipt.verify_attestation(&FixtureVerifier).unwrap();
}

#[test]
fn captures_artifact_provenance_from_an_open_file_handle() {
    let directory = temporary_receipt_directory();
    fs::create_dir(&directory).unwrap();
    let artifact_path = directory.join("zircon_runtime.exe");
    fs::write(&artifact_path, [1_u8, 2, 3]).unwrap();

    let captured = ReceiptArtifact::capture_from_file(
        "runtime-executable",
        "bin/zircon_runtime.exe",
        ArtifactKind::Executable,
        fs::File::open(&artifact_path).unwrap(),
    )
    .unwrap();

    let expected = Sha256::digest([1_u8, 2, 3])
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    assert_eq!(captured.logical_name, "runtime-executable");
    assert_eq!(captured.relative_path, "bin/zircon_runtime.exe");
    assert_eq!(captured.kind, ArtifactKind::Executable);
    assert_eq!(captured.byte_length, 3);
    assert_eq!(captured.sha256, expected);

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn captures_toolchain_binary_hashes_from_open_file_handles() {
    let directory = temporary_receipt_directory();
    fs::create_dir(&directory).unwrap();
    let cargo_path = directory.join("cargo.exe");
    let rustc_path = directory.join("rustc.exe");
    let linker_path = directory.join("link.exe");
    fs::write(&cargo_path, [4_u8, 5, 6]).unwrap();
    fs::write(&rustc_path, [7_u8, 8, 9]).unwrap();
    fs::write(&linker_path, [10_u8, 11, 12]).unwrap();

    let captured = ToolchainSet::capture_from_files(
        fs::File::open(&cargo_path).unwrap(),
        fs::File::open(&rustc_path).unwrap(),
        Some(fs::File::open(&linker_path).unwrap()),
        sha256('1'),
        sha256('2'),
    )
    .unwrap();

    let cargo_digest = Sha256::digest([4_u8, 5, 6])
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    let rustc_digest = Sha256::digest([7_u8, 8, 9])
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    let linker_digest = Sha256::digest([10_u8, 11, 12])
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    assert_eq!(captured.cargo_sha256, cargo_digest);
    assert_eq!(captured.rustc_sha256, rustc_digest);
    assert_eq!(
        captured.linker_sha256.as_deref(),
        Some(linker_digest.as_str())
    );

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_an_attestation_that_the_configured_verifier_cannot_verify() {
    let mut receipt = ProductReceipt::issue(
        valid_draft(),
        "2026-08-25T07:30:00.0000000Z",
        &FixtureSigner,
    )
    .unwrap();
    receipt.attestation.signature_hex = "FFFF".to_string();

    let error = receipt.verify_attestation(&FixtureVerifier).unwrap_err();

    assert!(error
        .to_string()
        .contains("attestation verification failed"));
}

#[test]
fn attestation_signature_binds_signer_identity_and_algorithm() {
    let receipt = ProductReceipt::issue(
        valid_draft(),
        "2026-08-25T07:30:00.0000000Z",
        &PayloadDigestAuthority,
    )
    .unwrap();
    receipt.verify_attestation(&PayloadDigestAuthority).unwrap();

    let mut changed_signer = receipt.clone();
    changed_signer.attestation.signer_id = "replacement-key".to_string();
    let signer_error = changed_signer
        .verify_attestation(&PayloadDigestAuthority)
        .unwrap_err();
    assert!(signer_error
        .to_string()
        .contains("attestation verification failed"));

    let mut changed_algorithm = receipt;
    changed_algorithm.attestation.algorithm = "replacement-algorithm".to_string();
    let algorithm_error = changed_algorithm
        .verify_attestation(&PayloadDigestAuthority)
        .unwrap_err();
    assert!(algorithm_error
        .to_string()
        .contains("attestation verification failed"));
}

#[test]
fn writes_a_durable_receipt_without_overwriting_existing_evidence() {
    let directory = temporary_receipt_directory();
    fs::create_dir(&directory).unwrap();
    let output_path = directory.join("product-receipt.json");
    let receipt = ProductReceipt::issue(
        valid_draft(),
        "2026-08-25T07:30:00.0000000Z",
        &FixtureSigner,
    )
    .unwrap();

    receipt
        .write_new_verified(&output_path, &FixtureVerifier)
        .unwrap();
    let written = fs::read_to_string(&output_path).unwrap();
    assert!(written.contains(&receipt.receipt_id));
    let reloaded: ProductReceipt = serde_json::from_str(&written).unwrap();
    reloaded.verify_attestation(&FixtureVerifier).unwrap();
    assert_eq!(reloaded.receipt_id, receipt.receipt_id);

    let error = receipt
        .write_new_verified(&output_path, &FixtureVerifier)
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("could not create product receipt"));

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn refuses_to_write_evidence_with_an_unverified_attestation() {
    let directory = temporary_receipt_directory();
    fs::create_dir(&directory).unwrap();
    let output_path = directory.join("product-receipt.json");
    let mut receipt = ProductReceipt::issue(
        valid_draft(),
        "2026-08-25T07:30:00.0000000Z",
        &FixtureSigner,
    )
    .unwrap();
    receipt.attestation.signature_hex = "FFFF".to_string();

    let error = receipt
        .write_new_verified(&output_path, &FixtureVerifier)
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("attestation verification failed"));
    assert!(!output_path.exists());
    fs::remove_dir_all(directory).unwrap();
}
