use std::path::PathBuf;

use super::super::{ProjectManifestSummary, ProjectManifestSummaryError};

#[test]
fn legacy_v1_manifest_summary_migrates_to_v2_and_reports_source_version() {
    let document = fixture("v1");
    let loaded =
        ProjectManifestSummary::parse_toml_bytes(&std::fs::read(document).unwrap()).unwrap();

    assert_eq!(loaded.migrated_from, Some(1));
    assert_eq!(loaded.value.name, "Shared Legacy Project");
    assert_eq!(loaded.value.engine_version_req, None);
    assert_eq!(loaded.value.default_scene, "res://scenes/main.scene.toml");
    assert_eq!(loaded.value.format_version, 2);
}

#[test]
fn summary_parser_rejects_future_manifest_versions() {
    let error =
        ProjectManifestSummary::parse_toml_bytes(&std::fs::read(fixture("future")).unwrap())
            .unwrap_err();

    assert!(matches!(
        error,
        ProjectManifestSummaryError::FutureVersion {
            found: 3,
            supported: 2
        }
    ));
}

#[test]
fn summary_parser_accepts_shared_v2_semver_and_rejects_shared_invalid_requirement() {
    let current =
        ProjectManifestSummary::parse_toml_bytes(&std::fs::read(fixture("v2")).unwrap()).unwrap();
    assert_eq!(current.migrated_from, None);
    assert_eq!(
        current.value.engine_version_req.as_deref(),
        Some(">=0.1.0, <0.2.0")
    );

    let error =
        ProjectManifestSummary::parse_toml_bytes(&std::fs::read(fixture("invalid")).unwrap())
            .unwrap_err();
    assert!(matches!(
        error,
        ProjectManifestSummaryError::InvalidEngineVersionReq { value, .. }
            if value == "not a semver requirement"
    ));
}

#[test]
fn summary_parser_rejects_bad_field_shapes() {
    let error = ProjectManifestSummary::parse_toml_bytes(
        br#"
name = ["not", "text"]
format_version = 2
default_scene = "res://scenes/main.scene.toml"
library_version = 1
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        ProjectManifestSummaryError::InvalidShape { .. }
    ));
}

fn fixture(version: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("serialization")
        .join("project-manifest")
        .join(version)
        .join("zircon-project.toml")
}
