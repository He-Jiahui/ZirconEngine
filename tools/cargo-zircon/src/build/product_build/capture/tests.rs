use std::fs::File;

use sha2::{Digest, Sha256};

use super::{
    capture_declared_artifacts, capture_sdk_fingerprint, OpenedDeclaredArtifact, OpenedSdkSource,
    PreparedProductBuildToolchain,
};
use crate::build::product_build::{ProductBuildSdkSource, ProductBuildToolchain};
use crate::build::receipt::canonical::bytes_to_hex;
use crate::build::receipt::{ArtifactKind, FileDigestBuffer, ReceiptArtifactSource};

#[test]
fn product_build_capture_helpers_reuse_one_digest_buffer() {
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-product-build-digest-buffer-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let sdk_bytes = b"sdk fingerprint source";
    let runtime_bytes = b"runtime dependency source with another size";
    let sdk_path = directory.join("sdk.lib");
    let runtime_path = directory.join("runtime.dll");
    std::fs::write(&sdk_path, sdk_bytes).unwrap();
    std::fs::write(&runtime_path, runtime_bytes).unwrap();
    let sdk = OpenedSdkSource {
        logical_name: "sdk-lib".to_string(),
        file: File::open(&sdk_path).unwrap(),
    };
    let runtime = OpenedDeclaredArtifact {
        source: ReceiptArtifactSource {
            logical_name: "runtime-dll".to_string(),
            relative_path: "runtime/runtime.dll".to_string(),
            kind: ArtifactKind::DynamicLibrary,
            source_path: runtime_path.clone(),
        },
        file: File::open(&runtime_path).unwrap(),
    };
    let mut buffer = FileDigestBuffer::new();
    let mut sdk_sources = vec![sdk];

    let sdk_fingerprint = capture_sdk_fingerprint(&mut sdk_sources, &mut buffer).unwrap();
    let artifacts = capture_declared_artifacts(vec![runtime], &mut buffer).unwrap();

    assert_eq!(sdk_fingerprint.len(), 64);
    assert_eq!(
        artifacts[0].sha256,
        bytes_to_hex(&Sha256::digest(runtime_bytes))
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn prepared_toolchain_reuses_components_and_retains_handles() {
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-prepared-toolchain-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    for (name, contents) in [
        ("cargo.exe", b"cargo bytes".as_slice()),
        ("rustc.exe", b"rustc bytes".as_slice()),
        ("link.exe", b"linker bytes".as_slice()),
        ("sdk.lib", b"sdk bytes".as_slice()),
    ] {
        std::fs::write(directory.join(name), contents).unwrap();
    }
    let mut source = ProductBuildToolchain {
        cargo_path: directory.join("cargo.exe"),
        rustc_path: directory.join("rustc.exe"),
        linker_path: Some(directory.join("link.exe")),
        sdk_files: vec![ProductBuildSdkSource {
            logical_name: "sdk-lib".to_string(),
            source_path: directory.join("sdk.lib"),
        }],
    };

    let prepared = PreparedProductBuildToolchain::open(&mut source).unwrap();
    let first = prepared.receipt_toolchain("A".repeat(64)).unwrap();
    let second = prepared.receipt_toolchain("B".repeat(64)).unwrap();

    assert_eq!(first.cargo_sha256, second.cargo_sha256);
    assert_eq!(first.rustc_sha256, second.rustc_sha256);
    assert_eq!(first.linker_sha256, second.linker_sha256);
    assert_eq!(first.sdk_fingerprint, second.sdk_fingerprint);
    assert_ne!(first.environment_digest, second.environment_digest);
    assert_ne!(first.toolchain_set_id, second.toolchain_set_id);
    assert!(prepared._cargo_file.metadata().unwrap().is_file());
    assert!(prepared._rustc_file.metadata().unwrap().is_file());
    assert!(prepared
        ._linker_file
        .as_ref()
        .unwrap()
        .metadata()
        .unwrap()
        .is_file());
    assert_eq!(prepared._sdk_files.len(), 1);

    drop(prepared);
    std::fs::remove_dir_all(directory).unwrap();
}
