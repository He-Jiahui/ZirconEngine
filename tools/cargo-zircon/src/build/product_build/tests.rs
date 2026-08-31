use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::{
    build_arguments, metadata_arguments, validate_build_request, CargoRuntimeDependencyDeclaration,
    ProductArtifactDeclaration, ProductBuildProducer, ProductBuildRequest, ProductBuildSdkSource,
    ProductBuildTarget, ProductBuildToolchain,
};
use crate::build::receipt::BuildAction;

#[test]
fn request_validation_rejects_duplicate_sdk_names_after_sorting() {
    let mut request = test_request();
    let duplicate = request.toolchain.sdk_files[0].clone();
    request.toolchain.sdk_files.push(duplicate);

    let error = validate_build_request(&mut request).unwrap_err();

    assert!(error
        .to_string()
        .contains("duplicate SDK file `windows-sdk`"));
}

#[test]
fn request_validation_rejects_duplicate_runtime_dependency_names_after_sorting() {
    let mut request = test_request();
    let duplicate = request.runtime_dependencies[0].clone();
    request.runtime_dependencies.push(duplicate);

    let error = validate_build_request(&mut request).unwrap_err();

    assert!(error
        .to_string()
        .contains("duplicate runtime dependency `zircon-runtime`"));
}

#[test]
fn request_validation_rejects_duplicate_features_after_sorting() {
    let mut request = test_request();
    request.action.features.push("runtime".to_string());

    let error = validate_build_request(&mut request).unwrap_err();

    assert!(error.to_string().contains("duplicate feature `runtime`"));
}

#[test]
fn cargo_argument_builders_preserve_the_declared_cli_order() {
    let mut request = test_request();
    let manifest = Path::new("snapshot/Cargo.toml");
    let target = Path::new("output/target");

    assert_eq!(
        argument_text(&metadata_arguments(&request, manifest)),
        [
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            "snapshot/Cargo.toml",
            "--frozen",
            "--filter-platform",
            "x86_64-pc-windows-msvc",
            "--features",
            "runtime",
        ]
    );
    assert_eq!(
        argument_text(&build_arguments(
            &request,
            manifest,
            target,
            "zircon_runtime",
        )),
        [
            "build",
            "--manifest-path",
            "snapshot/Cargo.toml",
            "--package",
            "zircon-app",
            "--bin",
            "zircon_runtime",
            "--frozen",
            "--target",
            "x86_64-pc-windows-msvc",
            "--profile",
            "release",
            "--target-dir",
            "output/target",
            "--message-format=json-render-diagnostics",
            "--features",
            "runtime",
        ]
    );

    request.action.features.clear();
    assert_eq!(metadata_arguments(&request, manifest).len(), 8);
    assert_eq!(
        build_arguments(&request, manifest, target, "zircon_runtime").len(),
        15
    );
}

fn argument_text<T: AsRef<OsStr>>(arguments: &[T]) -> Vec<String> {
    arguments
        .iter()
        .map(|argument| argument.as_ref().to_string_lossy().into_owned())
        .collect()
}

pub(super) fn test_request() -> ProductBuildRequest {
    ProductBuildRequest {
        schema_version: 1,
        build_set_manifest_path: PathBuf::from("build-set.json"),
        manifest_path: "Cargo.toml".to_string(),
        target_directory: PathBuf::from("target"),
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
            bin: Some("zircon_runtime".to_string()),
            features: vec!["runtime".to_string()],
        },
        producer: ProductBuildProducer {
            worker_id: "test-worker".to_string(),
            operation_id: "test-operation".to_string(),
        },
        product: ProductArtifactDeclaration {
            logical_name: "zircon-runtime".to_string(),
            relative_path: "bin/zircon_runtime.exe".to_string(),
            symbol_relative_directory: "symbols/runtime".to_string(),
        },
        environment_policy: "windows-msvc-v1".to_string(),
        runtime_dependencies: vec![CargoRuntimeDependencyDeclaration {
            logical_name: "zircon-runtime".to_string(),
            relative_path: "bin/zircon_runtime.dll".to_string(),
            package: "zircon-runtime".to_string(),
            target: "zircon_runtime".to_string(),
            artifact_file_name: "zircon_runtime.dll".to_string(),
        }],
        sbom: None,
    }
}
