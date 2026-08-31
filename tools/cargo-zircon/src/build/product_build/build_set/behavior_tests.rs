use std::ffi::OsStr;

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

use sha2::{Digest, Sha256};

#[test]
fn snapshot_relative_path_appends_one_entry_to_the_carried_directory() {
    assert_eq!(
        super::snapshot_relative_path("", OsStr::new("Cargo.toml")).unwrap(),
        "Cargo.toml"
    );
    assert_eq!(
        super::snapshot_relative_path("crates/runtime", OsStr::new("lib.rs")).unwrap(),
        "crates/runtime/lib.rs"
    );
}

#[test]
fn direct_relative_path_construction_preserves_components() {
    let path = super::relative_path("crates/runtime/src/lib.rs").unwrap();
    let components = path
        .components()
        .map(|component| component.as_os_str().to_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(components, ["crates", "runtime", "src", "lib.rs"]);
}

#[test]
fn bounded_manifest_read_retains_the_locked_handle() {
    let path = std::env::temp_dir().join(format!(
        "cargo-zircon-retained-manifest-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let expected = b"{\"schema_version\":1}";
    std::fs::write(&path, expected).unwrap();
    let mut file = std::fs::File::open(&path).unwrap();

    let actual = super::read_bounded_file(&mut file, 4_096, "test manifest").unwrap();

    assert_eq!(actual, expected);
    assert_eq!(file.metadata().unwrap().len(), expected.len() as u64);
    drop(file);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn shared_hash_buffer_verifies_multiple_snapshot_files() {
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-shared-hash-buffer-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let mut buffer = [0xA5_u8; super::BUILD_SET_HASH_BUFFER_BYTES];

    for (file_name, contents) in [
        ("first.rs", b"pub fn first() {}\n".as_slice()),
        (
            "second.rs",
            b"pub fn second() { println!(\"second\"); }\n".as_slice(),
        ),
    ] {
        let path = directory.join(file_name);
        std::fs::write(&path, contents).unwrap();
        let mut file = std::fs::File::open(&path).unwrap();
        let metadata = file.metadata().unwrap();
        let expected = super::BuildSetFile {
            relative_path: file_name.to_string(),
            sha256: super::hex_digest(&Sha256::digest(contents)),
            byte_length: contents.len() as u64,
        };

        super::verify_file_content(&mut file, &metadata, &expected, &mut buffer).unwrap();
    }

    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(windows)]
#[test]
fn snapshot_relative_path_rejects_non_unicode_entry_names() {
    let invalid_name = OsString::from_wide(&[0xD800]);

    let error = super::snapshot_relative_path("crates", &invalid_name)
        .err()
        .unwrap();

    assert!(error.to_string().contains("path is not Unicode"));
}

#[cfg(windows)]
#[test]
fn locks_the_snapshot_namespace_against_absent_input_and_a_b_a_mutation() {
    let directory = std::env::temp_dir().join(format!(
        "cargo-zircon-build-set-namespace-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let source = directory.join("source");
    std::fs::create_dir_all(&source).unwrap();
    let cargo_toml = b"[workspace]\n";
    let cargo_toml_path = source.join("Cargo.toml");
    std::fs::write(&cargo_toml_path, cargo_toml).unwrap();
    let mut manifest = super::BuildSetManifest {
        schema_version: super::BUILD_SET_SCHEMA_VERSION,
        build_set_kind: super::BUILD_SET_KIND.to_string(),
        status: super::BUILD_SET_STATUS.to_string(),
        build_set_id: String::new(),
        created_utc: "2026-08-28T00:00:00Z".to_string(),
        snapshot_relative_path: super::BUILD_SET_SNAPSHOT_RELATIVE_PATH.to_string(),
        source_policy: super::BUILD_SET_SOURCE_POLICY.to_string(),
        git_revision: "0".repeat(40),
        dirty_overlay_sha256: "0".repeat(64),
        files: vec![super::BuildSetFile {
            relative_path: "Cargo.toml".to_string(),
            sha256: super::hex_digest(&Sha256::digest(cargo_toml)),
            byte_length: cargo_toml.len() as u64,
        }],
    };
    manifest.build_set_id = super::derive_build_set_id(&manifest);
    std::fs::write(
        directory.join("build-set.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": manifest.schema_version,
            "build_set_kind": manifest.build_set_kind,
            "status": manifest.status,
            "build_set_id": manifest.build_set_id,
            "created_utc": manifest.created_utc,
            "snapshot_relative_path": manifest.snapshot_relative_path,
            "source_policy": manifest.source_policy,
            "git_revision": manifest.git_revision,
            "dirty_overlay_sha256": manifest.dirty_overlay_sha256,
            "files": [{
                "relative_path": manifest.files[0].relative_path,
                "sha256": manifest.files[0].sha256,
                "byte_length": manifest.files[0].byte_length,
            }]
        }))
        .unwrap(),
    )
    .unwrap();

    let build_set = super::ValidatedBuildSet::open(&directory.join("build-set.json")).unwrap();
    assert!(std::fs::write(source.join("build.rs"), b"fn main() {}\n").is_err());
    assert!(std::fs::write(&cargo_toml_path, b"[workspace]\n# state B\n").is_err());
    assert!(std::fs::write(&cargo_toml_path, cargo_toml).is_err());
    assert_eq!(
        std::fs::read(&cargo_toml_path).unwrap().as_slice(),
        cargo_toml
    );
    drop(build_set);
    std::fs::write(&cargo_toml_path, b"[workspace]\n# state B\n").unwrap();
    std::fs::write(&cargo_toml_path, cargo_toml).unwrap();
    std::fs::write(source.join("build.rs"), b"fn main() {}\n").unwrap();
    std::fs::remove_dir_all(directory).unwrap();
}
