use std::fs;
use std::io::ErrorKind;

use zircon_runtime::core::framework::render::ShaderVariantPrewarmManifest;

use super::super::{merge_manifests, read_manifest};
use crate::error::ShaderPrewarmManifestError;

#[test]
fn shader_prewarm_read_manifest_reports_typed_read_error() {
    let missing_path = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_missing_manifest_{}_not_found.json",
        std::process::id()
    ));
    let _ = fs::remove_file(&missing_path);

    let error = read_manifest(&missing_path).unwrap_err();

    match error {
        ShaderPrewarmManifestError::Read { path, source } => {
            assert_eq!(path, missing_path);
            assert_eq!(source.kind(), ErrorKind::NotFound);
        }
        other => panic!("expected typed manifest read error, got {other:?}"),
    }
}

#[test]
fn shader_prewarm_read_manifest_reports_typed_parse_error() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_bad_manifest_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let manifest_path = root.join("bad_manifest.json");
    fs::write(&manifest_path, "{not valid json").unwrap();

    let error = read_manifest(&manifest_path).unwrap_err();

    match error {
        ShaderPrewarmManifestError::Parse { path, source } => {
            assert_eq!(path, manifest_path);
            assert!(source.is_syntax());
        }
        other => panic!("expected typed manifest parse error, got {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_merge_manifest_reports_typed_schema_error() {
    let mut stale_manifest = ShaderVariantPrewarmManifest::empty();
    stale_manifest.schema_version = ShaderVariantPrewarmManifest::SCHEMA_VERSION + 1;
    let valid_manifest = ShaderVariantPrewarmManifest::empty();

    let error = merge_manifests(stale_manifest, valid_manifest).unwrap_err();

    match error {
        ShaderPrewarmManifestError::UnsupportedSchema { actual, expected } => {
            assert_eq!(actual, ShaderVariantPrewarmManifest::SCHEMA_VERSION + 1);
            assert_eq!(expected, ShaderVariantPrewarmManifest::SCHEMA_VERSION);
        }
        other => panic!("expected typed manifest schema error, got {other:?}"),
    }
    assert_eq!(
        error.to_string(),
        "shader prewarm manifest schema 4 is not supported; expected 3"
    );
}

#[test]
fn shader_prewarm_merge_manifest_rejects_v2_source_identity_schema() {
    let mut stale_manifest = ShaderVariantPrewarmManifest::empty();
    stale_manifest.schema_version = 2;

    let error = merge_manifests(stale_manifest, ShaderVariantPrewarmManifest::empty())
        .expect_err("v2 source identities included provenance labels and must not be reused");

    assert!(matches!(
        error,
        ShaderPrewarmManifestError::UnsupportedSchema {
            actual: 2,
            expected: 3
        }
    ));
}
