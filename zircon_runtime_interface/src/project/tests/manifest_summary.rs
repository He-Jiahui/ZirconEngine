use std::path::PathBuf;

use super::super::{
    ProjectManifestSummary, ProjectManifestSummaryError, ProjectNameError, MAX_PROJECT_ASSET_ROOTS,
    MAX_PROJECT_MANIFEST_ARRAY_ITEMS, MAX_PROJECT_MANIFEST_BYTES,
    MAX_PROJECT_MANIFEST_NESTING_DEPTH, MAX_PROJECT_MANIFEST_TABLE_ENTRIES,
};

#[test]
fn legacy_v1_manifest_summary_migrates_to_v3_and_reports_source_version() {
    let document = fixture("v1");
    let loaded =
        ProjectManifestSummary::parse_toml_bytes(&std::fs::read(document).unwrap()).unwrap();

    assert_eq!(loaded.migrated_from, Some(1));
    assert_eq!(loaded.value.name, "Shared Legacy Project");
    assert_eq!(loaded.value.engine_version_req, None);
    assert_eq!(loaded.value.default_scene, "res://scenes/main.scene.toml");
    assert_eq!(loaded.value.format_version, 3);
    assert_eq!(loaded.value.project_guid, None);
}

#[test]
fn summary_parser_rejects_future_manifest_versions() {
    let error =
        ProjectManifestSummary::parse_toml_bytes(&std::fs::read(fixture("future")).unwrap())
            .unwrap_err();

    assert!(matches!(
        error,
        ProjectManifestSummaryError::FutureVersion {
            found: 4,
            supported: 3
        }
    ));
}

#[test]
fn summary_parser_marks_shared_v2_for_migration_and_accepts_shared_v3() {
    let legacy =
        ProjectManifestSummary::parse_toml_bytes(&std::fs::read(fixture("v2")).unwrap()).unwrap();
    assert_eq!(legacy.migrated_from, Some(2));
    assert_eq!(legacy.value.project_guid, None);

    let current =
        ProjectManifestSummary::parse_toml_bytes(&std::fs::read(fixture("v3")).unwrap()).unwrap();
    assert_eq!(current.migrated_from, None);
    assert!(current.value.project_guid.is_some());
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
format_version = 3
project_guid = "20664b9f-4ab6-4e68-bd83-64e64f6ea5b4"
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

#[test]
fn summary_parser_reuses_portable_project_name_admission() {
    for (name, expected) in [
        (
            "CON",
            ProjectNameError::WindowsReserved {
                value: "CON".to_string(),
            },
        ),
        (
            "Game.",
            ProjectNameError::WindowsTrailingAlias {
                value: "Game.".to_string(),
            },
        ),
        (
            "folder/Game",
            ProjectNameError::NotSingleComponent {
                value: "folder/Game".to_string(),
            },
        ),
    ] {
        let error = ProjectManifestSummary::parse_toml_str(&manifest_with_name(name)).unwrap_err();
        assert!(matches!(
            error,
            ProjectManifestSummaryError::InvalidProjectName { source } if source == expected
        ));
    }
}

#[test]
fn summary_parser_rejects_duplicate_and_component_nested_asset_roots() {
    let duplicate =
        ProjectManifestSummary::parse_toml_str(&manifest_with_roots(&["assets", "assets"]))
            .unwrap_err();
    assert!(matches!(
        duplicate,
        ProjectManifestSummaryError::DuplicateAssetRoot { root } if root == "assets"
    ));

    let overlap =
        ProjectManifestSummary::parse_toml_str(&manifest_with_roots(&["a", "a-b", "a/child"]))
            .unwrap_err();
    assert!(matches!(
        overlap,
        ProjectManifestSummaryError::OverlappingAssetRoots {
            ancestor,
            descendant,
        } if ancestor == "a" && descendant == "a/child"
    ));
}

#[test]
fn summary_parser_rejects_manifest_and_asset_root_count_over_budget() {
    let oversized = vec![b' '; MAX_PROJECT_MANIFEST_BYTES + 1];
    assert!(matches!(
        ProjectManifestSummary::parse_toml_bytes(&oversized),
        Err(ProjectManifestSummaryError::DocumentTooLarge { max, found })
            if max == MAX_PROJECT_MANIFEST_BYTES && found == MAX_PROJECT_MANIFEST_BYTES + 1
    ));

    let roots = (0..=MAX_PROJECT_ASSET_ROOTS)
        .map(|index| format!("root-{index}"))
        .collect::<Vec<_>>();
    let root_refs = roots.iter().map(String::as_str).collect::<Vec<_>>();
    assert!(matches!(
        ProjectManifestSummary::parse_toml_str(&manifest_with_roots(&root_refs)),
        Err(ProjectManifestSummaryError::TooManyAssetRoots { max, found })
            if max == MAX_PROJECT_ASSET_ROOTS && found == MAX_PROJECT_ASSET_ROOTS + 1
    ));
}

#[test]
fn summary_parser_rejects_toml_container_complexity_over_budget() {
    let nested_array = format!(
        "probe = {}0{}\n",
        "[".repeat(MAX_PROJECT_MANIFEST_NESTING_DEPTH + 1),
        "]".repeat(MAX_PROJECT_MANIFEST_NESTING_DEPTH + 1),
    );
    assert!(matches!(
        ProjectManifestSummary::parse_toml_str(&nested_array),
        Err(ProjectManifestSummaryError::TomlNestingTooDeep { max, found })
            if max == MAX_PROJECT_MANIFEST_NESTING_DEPTH && found == max + 1
    ));

    let mut table_entries = String::new();
    for index in 0..=MAX_PROJECT_MANIFEST_TABLE_ENTRIES {
        table_entries.push_str(&format!("entry_{index} = 0\n"));
    }
    assert!(matches!(
        ProjectManifestSummary::parse_toml_str(&table_entries),
        Err(ProjectManifestSummaryError::TooManyTomlTableEntries { max, found })
            if max == MAX_PROJECT_MANIFEST_TABLE_ENTRIES && found == max + 1
    ));

    let array_items = std::iter::repeat("0")
        .take(MAX_PROJECT_MANIFEST_ARRAY_ITEMS + 1)
        .collect::<Vec<_>>()
        .join(",");
    let array_items = format!("probe = [{array_items}]\n");
    assert!(matches!(
        ProjectManifestSummary::parse_toml_str(&array_items),
        Err(ProjectManifestSummaryError::TooManyTomlArrayItems { max, found })
            if max == MAX_PROJECT_MANIFEST_ARRAY_ITEMS && found == max + 1
    ));
}

fn manifest_with_roots(roots: &[&str]) -> String {
    let roots = roots
        .iter()
        .map(|root| format!("{root:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "name = \"Root Validation\"\nformat_version = 3\nproject_guid = \"20664b9f-4ab6-4e68-bd83-64e64f6ea5b4\"\ndefault_scene = \"res://scenes/main.scene.toml\"\nasset_roots = [{roots}]\nlibrary_version = 1\n"
    )
}

fn manifest_with_name(name: &str) -> String {
    format!(
        "name = {name:?}\nformat_version = 3\nproject_guid = \"20664b9f-4ab6-4e68-bd83-64e64f6ea5b4\"\ndefault_scene = \"res://scenes/main.scene.toml\"\nasset_roots = [\"assets\"]\nlibrary_version = 1\n"
    )
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
