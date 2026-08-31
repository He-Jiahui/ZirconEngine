use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use cargo_zircon::build::product_build::{
    build_product_receipt_draft, build_product_receipt_draft_batch, select_cargo_product_artifact,
    CargoRuntimeDependencyDeclaration, ProductArtifactDeclaration, ProductBuildBatchRequest,
    ProductBuildProducer, ProductBuildRequest, ProductBuildSdkSource, ProductBuildTarget,
    ProductBuildToolchain,
};
use cargo_zircon::build::receipt::{BuildAction, Ed25519ProductReceiptSigner, ProductReceipt};
use ring::rand::SystemRandom;
use ring::signature::Ed25519KeyPair;
use sha2::{Digest, Sha256};

#[path = "product_build_owner/batch.rs"]
mod batch;

fn cargo_message(
    package_id: &str,
    target_name: &str,
    executable: Option<&str>,
    filenames: &[&str],
) -> String {
    serde_json::json!({
        "reason": "compiler-artifact",
        "package_id": package_id,
        "target": {
            "kind": ["bin"],
            "crate_types": ["bin"],
            "name": target_name,
            "src_path": "snapshot/src/main.rs",
            "edition": "2024",
            "doc": true,
            "doctest": false,
            "test": true
        },
        "profile": {
            "opt_level": "0",
            "debuginfo": 2,
            "debug_assertions": true,
            "overflow_checks": true,
            "test": false
        },
        "features": [],
        "filenames": filenames,
        "executable": executable,
        "fresh": false
    })
    .to_string()
}

fn sha256_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect()
}

fn sha256_letter(letter: char) -> String {
    std::iter::repeat_n(letter, 64).collect()
}

fn write_length_framed(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as i64).to_le_bytes());
    hasher.update(value.as_bytes());
}

fn write_build_set_manifest(build_set_root: &std::path::Path) -> PathBuf {
    let snapshot = build_set_root.join("source");
    fs::create_dir_all(&snapshot).unwrap();
    let cargo_toml = b"[workspace]\n";
    fs::write(snapshot.join("Cargo.toml"), cargo_toml).unwrap();
    let git_revision = "a".repeat(40);
    let dirty_overlay_sha256 = sha256_letter('B');
    let file_sha256 = sha256_bytes(cargo_toml);
    let byte_length = cargo_toml.len() as u64;
    let mut identity = Sha256::new();
    for value in [
        "zircon-mvp-build-set-v1",
        &git_revision,
        &dirty_overlay_sha256,
        "Cargo.toml",
        &file_sha256,
        &byte_length.to_string(),
    ] {
        write_length_framed(&mut identity, value);
    }
    let build_set_id = identity
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    let manifest_path = build_set_root.join("build-set.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "build_set_kind": "zircon_mvp_product_build_set",
            "status": "completed",
            "build_set_id": build_set_id,
            "created_utc": "2026-08-28T12:00:00.0000000Z",
            "snapshot_relative_path": "source",
            "source_policy": "tracked_head_plus_tracked_dirty_overlay",
            "git_revision": git_revision,
            "dirty_overlay_sha256": dirty_overlay_sha256,
            "files": [{
                "relative_path": "Cargo.toml",
                "sha256": file_sha256,
                "byte_length": byte_length
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    manifest_path
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

fn batch_contract_request(build_set_manifest_path: PathBuf, feature: &str) -> ProductBuildRequest {
    ProductBuildRequest {
        schema_version: 1,
        build_set_manifest_path,
        manifest_path: "Cargo.toml".to_string(),
        target_directory: PathBuf::from(format!("D:/ZirconBuilds/{feature}-target")),
        toolchain: ProductBuildToolchain {
            cargo_path: PathBuf::from("C:/toolchain/cargo.exe"),
            rustc_path: PathBuf::from("C:/toolchain/rustc.exe"),
            linker_path: Some(PathBuf::from("C:/toolchain/link.exe")),
            sdk_files: vec![ProductBuildSdkSource {
                logical_name: "windows-kernel32-lib".to_string(),
                source_path: PathBuf::from("C:/sdk/kernel32.lib"),
            }],
        },
        target: ProductBuildTarget {
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            cargo_profile: "dev".to_string(),
            rustflags: Vec::new(),
        },
        action: BuildAction {
            package: "zircon_app".to_string(),
            bin: Some("zircon_runtime".to_string()),
            features: vec![feature.to_string()],
        },
        producer: ProductBuildProducer {
            worker_id: "windows-worker-01".to_string(),
            operation_id: format!("batch-{feature}"),
        },
        product: ProductArtifactDeclaration {
            logical_name: format!("{feature}-executable"),
            relative_path: format!("{feature}/zircon_runtime.exe"),
            symbol_relative_directory: format!("{feature}/symbols"),
        },
        environment_policy: "windows-msvc-v1".to_string(),
        runtime_dependencies: vec![CargoRuntimeDependencyDeclaration {
            logical_name: format!("{feature}-runtime-library"),
            relative_path: format!("{feature}/zircon_runtime.dll"),
            package: "zircon_runtime".to_string(),
            target: "zircon_runtime".to_string(),
            artifact_file_name: "zircon_runtime.dll".to_string(),
        }],
        sbom: None,
    }
}

#[test]
fn product_build_batch_rejects_repeated_actions_and_mixed_build_sets_before_cargo() {
    let first_manifest = PathBuf::from("D:/ZirconBuilds/build-set-a/build-set.json");
    let first = batch_contract_request(first_manifest.clone(), "target-client");
    let repeated = ProductBuildBatchRequest {
        schema_version: 1,
        builds: vec![first.clone(), first],
    };
    let repeated_error = build_product_receipt_draft_batch(repeated).unwrap_err();
    assert!(repeated_error
        .to_string()
        .contains("duplicate build action"));

    let mixed = ProductBuildBatchRequest {
        schema_version: 1,
        builds: vec![
            batch_contract_request(first_manifest, "target-client"),
            batch_contract_request(
                PathBuf::from("D:/ZirconBuilds/build-set-b/build-set.json"),
                "target-editor-host",
            ),
        ],
    };
    let mixed_error = build_product_receipt_draft_batch(mixed).unwrap_err();
    assert!(mixed_error.to_string().contains("one BuildSet manifest"));
}

fn write_fake_cargo(
    directory: &std::path::Path,
    metadata: &std::path::Path,
    messages: &std::path::Path,
    artifacts: &[(&std::path::Path, &std::path::Path)],
) -> PathBuf {
    #[cfg(windows)]
    {
        let path = directory.join("fake-cargo.cmd");
        let artifact_commands = artifacts
            .iter()
            .map(|(source, destination)| {
                format!(
                    "  if not exist \"{}\" mkdir \"{}\"\r\n  copy /b /y \"{}\" \"{}\" >nul\r\n",
                    destination.parent().unwrap().display(),
                    destination.parent().unwrap().display(),
                    source.display(),
                    destination.display()
                )
            })
            .collect::<String>();
        fs::write(
            &path,
            format!(
                "@echo off\r\nif \"%1\"==\"metadata\" (\r\n  type \"{}\"\r\n  exit /b 0\r\n)\r\nif \"%1\"==\"build\" (\r\n{}  type \"{}\"\r\n  exit /b 0\r\n)\r\nexit /b 9\r\n",
                metadata.display(),
                artifact_commands,
                messages.display()
            ),
        )
        .unwrap();
        path
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = directory.join("fake-cargo");
        let artifact_commands = artifacts
            .iter()
            .map(|(source, destination)| {
                format!(
                    "  mkdir -p '{}'\n  cp '{}' '{}'\n",
                    destination.parent().unwrap().display(),
                    source.display(),
                    destination.display()
                )
            })
            .collect::<String>();
        fs::write(
            &path,
            format!(
                "#!/bin/sh\nif [ \"$1\" = metadata ]; then\n  /bin/cat '{}'\n  exit 0\nfi\nif [ \"$1\" = build ]; then\n{}  /bin/cat '{}'\n  exit 0\nfi\nexit 9\n",
                metadata.display(),
                artifact_commands,
                messages.display()
            ),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }
}

#[test]
fn selects_only_the_exact_cargo_product_and_discovers_its_symbol_file() {
    let dependency = cargo_message(
        "path+file:///snapshot#dependency@0.1.0",
        "zircon_runtime",
        Some("D:/target/debug/dependency.exe"),
        &["D:/target/debug/dependency.exe"],
    );
    let product = cargo_message(
        "path+file:///snapshot#zircon_app@0.1.0",
        "zircon_runtime",
        Some("D:/target/debug/zircon_runtime.exe"),
        &[
            "D:/target/debug/zircon_runtime.exe",
            "D:/target/debug/zircon_runtime.pdb",
        ],
    );
    let finished = serde_json::json!({"reason": "build-finished", "success": true}).to_string();
    let stream = format!("{dependency}\n{product}\n{finished}\n");

    let selected = select_cargo_product_artifact(
        Cursor::new(stream),
        "path+file:///snapshot#zircon_app@0.1.0",
        "zircon_runtime",
    )
    .unwrap();

    assert_eq!(
        selected.executable,
        PathBuf::from("D:/target/debug/zircon_runtime.exe")
    );
    assert_eq!(
        selected.symbol_files,
        vec![PathBuf::from("D:/target/debug/zircon_runtime.pdb")]
    );
}

#[test]
fn rejects_missing_or_ambiguous_cargo_product_artifacts() {
    let product = cargo_message(
        "path+file:///snapshot#zircon_app@0.1.0",
        "zircon_runtime",
        Some("D:/target/debug/zircon_runtime.exe"),
        &["D:/target/debug/zircon_runtime.exe"],
    );

    let missing = select_cargo_product_artifact(
        Cursor::new("{\"reason\":\"build-finished\",\"success\":true}\n"),
        "path+file:///snapshot#zircon_app@0.1.0",
        "zircon_runtime",
    )
    .unwrap_err();
    assert!(missing.to_string().contains("did not emit"));

    let ambiguous = select_cargo_product_artifact(
        Cursor::new(format!("{product}\n{product}\n")),
        "path+file:///snapshot#zircon_app@0.1.0",
        "zircon_runtime",
    )
    .unwrap_err();
    assert!(ambiguous.to_string().contains("more than one"));
}

#[test]
fn rejects_non_json_and_oversized_cargo_message_lines() {
    let malformed = select_cargo_product_artifact(
        Cursor::new("not-json\n"),
        "path+file:///snapshot#zircon_app@0.1.0",
        "zircon_runtime",
    )
    .unwrap_err();
    assert!(malformed.to_string().contains("Cargo JSON message"));

    let oversized = format!("{{\"reason\":\"{}\"}}\n", "x".repeat(4 * 1024 * 1024));
    let oversized_error = select_cargo_product_artifact(
        Cursor::new(oversized),
        "path+file:///snapshot#zircon_app@0.1.0",
        "zircon_runtime",
    )
    .unwrap_err();
    assert!(oversized_error.to_string().contains("line limit"));
}

#[test]
fn build_owner_runs_cargo_from_the_snapshot_and_captures_actual_build_outputs() {
    let directory = temporary_directory("product-build-owner");
    let build_set_root = directory.join("build-set");
    let build_set_manifest_path = write_build_set_manifest(&build_set_root);
    let snapshot = build_set_root.join("source");
    let target = directory.join("target");

    let rustc_path = directory.join("rustc.exe");
    let linker_path = directory.join("link.exe");
    let sdk_library_path = directory.join("kernel32.lib");
    let executable = target.join("debug/zircon_runtime.exe");
    let symbols = target.join("debug/zircon_runtime.pdb");
    let runtime_library = target.join("debug/zircon_runtime.dll");
    let executable_source = directory.join("actual-product.bin");
    let symbols_source = directory.join("actual-symbols.bin");
    let runtime_library_source = directory.join("actual-runtime-library.bin");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(&rustc_path, b"actual rustc bytes").unwrap();
    fs::write(&linker_path, b"actual linker bytes").unwrap();
    fs::write(&sdk_library_path, b"actual sdk library bytes").unwrap();
    fs::write(&executable, b"stale product bytes").unwrap();
    fs::write(&symbols, b"stale symbol bytes").unwrap();
    fs::write(&runtime_library, b"stale runtime library bytes").unwrap();
    fs::write(&executable_source, b"actual product bytes").unwrap();
    fs::write(&symbols_source, b"actual symbol bytes").unwrap();
    fs::write(&runtime_library_source, b"actual runtime library bytes").unwrap();

    let package_id = "path+file:///snapshot#zircon_app@0.1.0";
    let runtime_package_id = "path+file:///snapshot#zircon_runtime@0.1.0";
    let metadata = serde_json::to_vec(&serde_json::json!({
        "packages": [
            {
                "name": "zircon_app",
                "id": package_id,
                "targets": [{"name": "zircon_runtime", "kind": ["bin"]}]
            },
            {
                "name": "zircon_runtime",
                "id": runtime_package_id,
                "targets": [{"name": "zircon_runtime", "kind": ["lib"]}]
            }
        ],
        "workspace_members": [package_id, runtime_package_id],
        "resolve": {"nodes": [
            {"id": package_id, "dependencies": [runtime_package_id], "deps": [], "features": []},
            {"id": runtime_package_id, "dependencies": [], "deps": [], "features": []}
        ]}
    }))
    .unwrap();
    let metadata_path = directory.join("metadata.json");
    fs::write(&metadata_path, &metadata).unwrap();
    let build_messages_path = directory.join("build.jsonl");
    let product_message = cargo_message(
        package_id,
        "zircon_runtime",
        executable.to_str(),
        &[executable.to_str().unwrap(), symbols.to_str().unwrap()],
    );
    let runtime_message = cargo_message(
        runtime_package_id,
        "zircon_runtime",
        None,
        &[runtime_library.to_str().unwrap()],
    );
    fs::write(
        &build_messages_path,
        format!(
            "{runtime_message}\n{product_message}\n{}\n",
            serde_json::json!({"reason": "build-finished", "success": true})
        ),
    )
    .unwrap();
    let cargo_path = write_fake_cargo(
        &directory,
        &metadata_path,
        &build_messages_path,
        &[
            (&executable_source, &executable),
            (&symbols_source, &symbols),
            (&runtime_library_source, &runtime_library),
        ],
    );

    let request = ProductBuildRequest {
        schema_version: 1,
        build_set_manifest_path: build_set_manifest_path.clone(),
        manifest_path: "Cargo.toml".to_string(),
        target_directory: target.clone(),
        toolchain: ProductBuildToolchain {
            cargo_path: cargo_path.clone(),
            rustc_path: rustc_path.clone(),
            linker_path: Some(linker_path.clone()),
            sdk_files: vec![ProductBuildSdkSource {
                logical_name: "windows-kernel32-lib".to_string(),
                source_path: sdk_library_path,
            }],
        },
        target: ProductBuildTarget {
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            cargo_profile: "dev".to_string(),
            rustflags: vec!["-C".to_string(), "target-cpu=x86-64-v2".to_string()],
        },
        action: BuildAction {
            package: "zircon_app".to_string(),
            bin: Some("zircon_runtime".to_string()),
            features: vec!["target-client".to_string()],
        },
        producer: ProductBuildProducer {
            worker_id: "windows-worker-01".to_string(),
            operation_id: "product-build-owner-test".to_string(),
        },
        product: ProductArtifactDeclaration {
            logical_name: "runtime-executable".to_string(),
            relative_path: "bin/zircon_runtime.exe".to_string(),
            symbol_relative_directory: "symbols".to_string(),
        },
        environment_policy: "windows-msvc-v1".to_string(),
        runtime_dependencies: vec![CargoRuntimeDependencyDeclaration {
            logical_name: "runtime-library".to_string(),
            relative_path: "bin/zircon_runtime.dll".to_string(),
            package: "zircon_runtime".to_string(),
            target: "zircon_runtime".to_string(),
            artifact_file_name: "zircon_runtime.dll".to_string(),
        }],
        sbom: None,
    };

    let stale_target_error = build_product_receipt_draft(request.clone())
        .err()
        .expect("an existing Cargo target directory must be rejected");
    assert!(stale_target_error
        .to_string()
        .contains("must not already exist"));
    fs::remove_dir_all(&target).unwrap();

    let cli_request = request.clone();
    let draft = build_product_receipt_draft(request).unwrap();

    assert_eq!(draft.build_products.len(), 1);
    assert_eq!(
        draft.build_products[0].sha256,
        sha256_bytes(b"actual product bytes")
    );
    assert_eq!(draft.symbols.len(), 1);
    assert_eq!(draft.symbols[0].relative_path, "symbols/zircon_runtime.pdb");
    assert_eq!(
        draft.symbols[0].sha256,
        sha256_bytes(b"actual symbol bytes")
    );
    assert_eq!(draft.runtime_dependencies.len(), 1);
    assert_eq!(
        draft.runtime_dependencies[0].sha256,
        sha256_bytes(b"actual runtime library bytes")
    );
    assert_eq!(
        draft.toolchain.cargo_sha256,
        sha256_bytes(&fs::read(cargo_path).unwrap())
    );
    assert_eq!(
        draft.toolchain.rustc_sha256,
        sha256_bytes(b"actual rustc bytes")
    );
    assert_eq!(
        draft.toolchain.linker_sha256,
        Some(sha256_bytes(b"actual linker bytes"))
    );
    assert_ne!(draft.toolchain.sdk_fingerprint, sha256_letter('E'));
    assert_eq!(draft.toolchain.sdk_fingerprint.len(), 64);
    assert_eq!(draft.target_profile.cargo_graph_digest.len(), 64);
    assert!(draft
        .target_profile
        .cargo_graph_digest
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte)));
    assert_ne!(
        draft.target_profile.cargo_graph_digest,
        sha256_bytes(&metadata)
    );
    assert_eq!(draft.action.package, "zircon_app");
    assert_eq!(draft.action.bin.as_deref(), Some("zircon_runtime"));
    assert_eq!(draft.producer.tool, "cargo-zircon");
    assert_eq!(draft.producer.tool_version, env!("CARGO_PKG_VERSION"));

    let mut tampered_request = cli_request.clone();
    tampered_request.target_directory = directory.join("tampered-target");
    fs::create_dir(&tampered_request.target_directory).unwrap();
    fs::write(snapshot.join("Cargo.toml"), b"tampered workspace bytes").unwrap();
    let tampered_error = build_product_receipt_draft(tampered_request).err().unwrap();
    assert!(tampered_error
        .to_string()
        .contains("BuildSet snapshot file content differs"));
    fs::write(snapshot.join("Cargo.toml"), b"[workspace]\n").unwrap();

    let request_path = directory.join("product-build-request.json");
    let draft_path = directory.join("product-receipt-draft.json");
    let key_path = directory.join("build-worker.pk8");
    let trust_registry_path = directory.join("product-receipt-trust.json");
    let receipt_path = directory.join("product-receipt.json");
    let key = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
    let signer = Ed25519ProductReceiptSigner::from_pkcs8("build-worker-01", key.as_ref()).unwrap();
    fs::write(&request_path, serde_json::to_vec(&cli_request).unwrap()).unwrap();
    fs::write(&key_path, key.as_ref()).unwrap();
    fs::write(
        &trust_registry_path,
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "trust_registry_kind": "zircon_product_receipt_trust_registry",
            "issuers": [{
                "signer_id": "build-worker-01",
                "algorithm": "ed25519-v1",
                "public_key_hex": signer.public_key_hex(),
                "disabled": false
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::remove_dir_all(&target).unwrap();
    let build = std::process::Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "build",
            "--request",
            request_path.to_str().unwrap(),
            "--output",
            draft_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let built_draft: cargo_zircon::build::receipt::ProductReceiptDraft =
        serde_json::from_slice(&fs::read(&draft_path).unwrap()).unwrap();
    let draft_handoff_sha256 = built_draft.handoff_sha256().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&build.stdout).trim(),
        draft_handoff_sha256
    );
    assert_eq!(
        built_draft.build_products[0].sha256,
        sha256_bytes(b"actual product bytes")
    );
    let mut tampered_handoff = built_draft.clone();
    tampered_handoff.producer.operation_id = "tampered-after-build".to_string();
    assert!(tampered_handoff
        .verify_handoff_sha256(&draft_handoff_sha256)
        .unwrap_err()
        .to_string()
        .contains("build-owner handoff digest"));

    let issue = std::process::Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "issue-draft",
            "--draft",
            draft_path.to_str().unwrap(),
            "--expected-draft-sha256",
            &draft_handoff_sha256,
            "--private-key",
            key_path.to_str().unwrap(),
            "--trust-registry",
            trust_registry_path.to_str().unwrap(),
            "--signer-id",
            "build-worker-01",
            "--created-utc",
            "2026-08-28T12:00:00.0000000Z",
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
    let receipt: ProductReceipt =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    assert_eq!(
        receipt.build_products[0].sha256,
        sha256_bytes(b"actual product bytes")
    );
    assert_eq!(
        receipt.symbols[0].sha256,
        sha256_bytes(b"actual symbol bytes")
    );
    assert!(String::from_utf8_lossy(&issue.stdout).contains(&receipt.receipt_id));

    let untrusted_registry_path = directory.join("untrusted-product-receipt-trust.json");
    let rejected_receipt_path = directory.join("rejected-product-receipt.json");
    fs::write(
        &untrusted_registry_path,
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "trust_registry_kind": "zircon_product_receipt_trust_registry",
            "issuers": [{
                "signer_id": "other-build-worker",
                "algorithm": "ed25519-v1",
                "public_key_hex": signer.public_key_hex(),
                "disabled": false
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let rejected = std::process::Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "issue-draft",
            "--draft",
            draft_path.to_str().unwrap(),
            "--expected-draft-sha256",
            &draft_handoff_sha256,
            "--private-key",
            key_path.to_str().unwrap(),
            "--trust-registry",
            untrusted_registry_path.to_str().unwrap(),
            "--signer-id",
            "build-worker-01",
            "--created-utc",
            "2026-08-28T12:01:00.0000000Z",
            "--output",
            rejected_receipt_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(!rejected_receipt_path.exists());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("not trusted"));

    fs::remove_dir_all(directory).unwrap();
}
