use std::collections::HashSet;

use super::*;
use cargo_zircon::build::product_build::ProductBuildDraftBatch;

struct FakeCargoAction {
    bin: &'static str,
    messages: PathBuf,
    artifacts: Vec<(PathBuf, PathBuf)>,
}

fn write_fake_batch_cargo(
    directory: &std::path::Path,
    metadata: &std::path::Path,
    invocation_log: &std::path::Path,
    actions: &[FakeCargoAction],
) -> PathBuf {
    #[cfg(windows)]
    {
        let action_commands = actions
            .iter()
            .map(|action| {
                let copy_commands = action
                    .artifacts
                    .iter()
                    .map(|(source, destination)| {
                        format!(
                            "    if not exist \"{}\" mkdir \"{}\"\r\n    copy /b /y \"{}\" \"{}\" >nul\r\n",
                            destination.parent().unwrap().display(),
                            destination.parent().unwrap().display(),
                            source.display(),
                            destination.display()
                        )
                    })
                    .collect::<String>();
                format!(
                    "  echo %*| findstr /C:\"--bin {}\" >nul\r\n  if not errorlevel 1 (\r\n{}    type \"{}\"\r\n    exit /b 0\r\n  )\r\n",
                    action.bin,
                    copy_commands,
                    action.messages.display(),
                )
            })
            .collect::<String>();
        let path = directory.join("fake-cargo.cmd");
        fs::write(
            &path,
            format!(
                "@echo off\r\nif \"%1\"==\"metadata\" (\r\n  type \"{}\"\r\n  exit /b 0\r\n)\r\nif \"%1\"==\"build\" (\r\n  echo %*>>\"{}\"\r\n{}  exit /b 8\r\n)\r\nexit /b 9\r\n",
                metadata.display(),
                invocation_log.display(),
                action_commands,
            ),
        )
        .unwrap();
        path
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;

        let action_commands = actions
            .iter()
            .map(|action| {
                let copy_commands = action
                    .artifacts
                    .iter()
                    .map(|(source, destination)| {
                        format!(
                            "      mkdir -p '{}'\n      cp '{}' '{}'\n",
                            destination.parent().unwrap().display(),
                            source.display(),
                            destination.display()
                        )
                    })
                    .collect::<String>();
                format!(
                    "    *\"--bin {}\"*)\n{}      /bin/cat '{}'\n      ;;\n",
                    action.bin,
                    copy_commands,
                    action.messages.display(),
                )
            })
            .collect::<String>();
        let path = directory.join("fake-cargo");
        fs::write(
            &path,
            format!(
                "#!/bin/sh\nif [ \"$1\" = metadata ]; then\n  /bin/cat '{}'\n  exit 0\nfi\nif [ \"$1\" = build ]; then\n  echo \"$*\" >> '{}'\n  case \"$*\" in\n{}    *) exit 8 ;;\n  esac\n  exit 0\nfi\nexit 9\n",
                metadata.display(),
                invocation_log.display(),
                action_commands,
            ),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }
}

#[test]
fn public_four_product_batch_builds_each_action_once() {
    let directory = temporary_directory("public-four-product-batch");
    fs::create_dir(&directory).unwrap();
    let build_set_manifest_path = write_build_set_manifest(&directory.join("build-set"));
    let rustc_path = directory.join("rustc.exe");
    let linker_path = directory.join("link.exe");
    let sdk_path = directory.join("kernel32.lib");
    fs::write(&rustc_path, b"shared rustc bytes").unwrap();
    fs::write(&linker_path, b"shared linker bytes").unwrap();
    fs::write(&sdk_path, b"shared sdk bytes").unwrap();

    let package_id = "path+file:///snapshot#zircon_app@0.1.0";
    let runtime_package_id = "path+file:///snapshot#zircon_runtime@0.1.0";
    let products = [
        ("runtime", "zircon_runtime", "target-client"),
        ("editor", "zircon_editor", "target-editor-host"),
        ("hub", "zircon_hub", "target-hub"),
        ("workbench", "zircon_workbench", "target-workbench"),
    ];
    let metadata_path = directory.join("metadata.json");
    let targets = products
        .iter()
        .map(|(_, bin, _)| serde_json::json!({"name": bin, "kind": ["bin"]}))
        .collect::<Vec<_>>();
    fs::write(
        &metadata_path,
        serde_json::to_vec(&serde_json::json!({
            "packages": [
                {"name": "zircon_app", "id": package_id, "targets": targets},
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
        .unwrap(),
    )
    .unwrap();

    let mut fake_actions = Vec::with_capacity(products.len());
    for (product, bin, _) in products {
        let target = directory.join(format!("{product}-target"));
        let executable = target.join(format!("debug/{bin}.exe"));
        let runtime_library = target.join("debug/zircon_runtime.dll");
        let executable_source = directory.join(format!("{product}-executable.bin"));
        let library_source = directory.join(format!("{product}-runtime-library.bin"));
        fs::write(
            &executable_source,
            format!("{product} executable bytes").as_bytes(),
        )
        .unwrap();
        fs::write(
            &library_source,
            format!("{product} runtime library bytes").as_bytes(),
        )
        .unwrap();
        let messages = directory.join(format!("{product}-build.jsonl"));
        fs::write(
            &messages,
            format!(
                "{}\n{}\n{}\n",
                cargo_message(
                    runtime_package_id,
                    "zircon_runtime",
                    None,
                    &[runtime_library.to_str().unwrap()],
                ),
                cargo_message(
                    package_id,
                    bin,
                    executable.to_str(),
                    &[executable.to_str().unwrap()],
                ),
                serde_json::json!({"reason": "build-finished", "success": true}),
            ),
        )
        .unwrap();
        fake_actions.push(FakeCargoAction {
            bin,
            messages,
            artifacts: vec![
                (executable_source, executable),
                (library_source, runtime_library),
            ],
        });
    }

    let invocation_log = directory.join("build-invocations.log");
    let cargo_path =
        write_fake_batch_cargo(&directory, &metadata_path, &invocation_log, &fake_actions);
    let builds = products
        .into_iter()
        .map(|(product, bin, feature)| ProductBuildRequest {
            schema_version: 1,
            build_set_manifest_path: build_set_manifest_path.clone(),
            manifest_path: "Cargo.toml".to_string(),
            target_directory: directory.join(format!("{product}-target")),
            toolchain: ProductBuildToolchain {
                cargo_path: cargo_path.clone(),
                rustc_path: rustc_path.clone(),
                linker_path: Some(linker_path.clone()),
                sdk_files: vec![ProductBuildSdkSource {
                    logical_name: "windows-kernel32-lib".to_string(),
                    source_path: sdk_path.clone(),
                }],
            },
            target: ProductBuildTarget {
                target_triple: "x86_64-pc-windows-msvc".to_string(),
                cargo_profile: "dev".to_string(),
                rustflags: Vec::new(),
            },
            action: BuildAction {
                package: "zircon_app".to_string(),
                bin: Some(bin.to_string()),
                features: vec![feature.to_string()],
            },
            producer: ProductBuildProducer {
                worker_id: "windows-worker-01".to_string(),
                operation_id: format!("batch-{product}-product"),
            },
            product: ProductArtifactDeclaration {
                logical_name: format!("{product}-executable"),
                relative_path: format!("{product}/{bin}.exe"),
                symbol_relative_directory: format!("{product}/symbols"),
            },
            environment_policy: "windows-msvc-v1".to_string(),
            runtime_dependencies: vec![CargoRuntimeDependencyDeclaration {
                logical_name: format!("{product}-runtime-library"),
                relative_path: format!("{product}/zircon_runtime.dll"),
                package: "zircon_runtime".to_string(),
                target: "zircon_runtime".to_string(),
                artifact_file_name: "zircon_runtime.dll".to_string(),
            }],
            sbom: None,
        })
        .collect();

    let request = ProductBuildBatchRequest {
        schema_version: 1,
        builds,
    };
    let request_path = directory.join("four-product-build-request.json");
    let output_path = directory.join("four-product-draft-batch.json");
    fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
    let build = std::process::Command::new(env!("CARGO_BIN_EXE_cargo-zircon"))
        .args([
            "product-receipt",
            "build-batch",
            "--request",
            request_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let output_bytes = fs::read(&output_path).unwrap();
    let batch: ProductBuildDraftBatch = serde_json::from_slice(&output_bytes).unwrap();
    let handoff_sha256 = batch.handoff_sha256().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&build.stdout).trim(),
        handoff_sha256
    );
    assert_eq!(sha256_bytes(&output_bytes), handoff_sha256);
    assert_eq!(batch.drafts.len(), 4);
    assert!(batch
        .drafts
        .iter()
        .all(|draft| draft.build_set_id == batch.build_set_id));
    assert_eq!(
        batch
            .drafts
            .iter()
            .map(|draft| draft.build_products.len() + draft.runtime_dependencies.len())
            .sum::<usize>(),
        8
    );
    let first_toolchain = &batch.drafts[0].toolchain;
    for draft in &batch.drafts[1..] {
        assert_eq!(draft.toolchain.cargo_sha256, first_toolchain.cargo_sha256);
        assert_eq!(draft.toolchain.rustc_sha256, first_toolchain.rustc_sha256);
        assert_eq!(draft.toolchain.linker_sha256, first_toolchain.linker_sha256);
        assert_eq!(
            draft.toolchain.sdk_fingerprint,
            first_toolchain.sdk_fingerprint
        );
        assert_ne!(
            draft.toolchain.environment_digest,
            first_toolchain.environment_digest
        );
    }
    assert_eq!(
        batch
            .drafts
            .iter()
            .map(|draft| draft.toolchain.toolchain_set_id.as_str())
            .collect::<HashSet<_>>()
            .len(),
        4
    );
    let invocations = fs::read_to_string(&invocation_log).unwrap();
    assert_eq!(invocations.lines().count(), 4);
    for (_, bin, _) in products {
        assert_eq!(
            invocations
                .lines()
                .filter(|line| line.contains(&format!("--bin {bin}")))
                .count(),
            1
        );
    }

    fs::remove_dir_all(directory).unwrap();
}
