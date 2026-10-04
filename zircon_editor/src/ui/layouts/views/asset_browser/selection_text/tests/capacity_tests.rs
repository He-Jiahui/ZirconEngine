use super::*;
use zircon_runtime_interface::resource::ResourceKind;

use crate::ui::workbench::snapshot::{AssetSubassetSnapshot, AssetTypeProjectionSnapshot};

const EDITOR848_SELECTION_METADATA_SUMMARY_CAPACITY_BENCH_V1: &str =
    "EDITOR848_SELECTION_METADATA_SUMMARY_CAPACITY_BENCH_V1";

fn subasset(index: usize) -> AssetSubassetSnapshot {
    AssetSubassetSnapshot {
        uuid: format!("uuid-{index}"),
        locator: format!("res://subasset-{index}"),
        kind: ResourceKind::Texture,
        asset_type: AssetTypeProjectionSnapshot::default(),
        artifact_locator: None,
        dependency_locators: Vec::new(),
    }
}

#[test]
fn selection_metadata_summary_capacity_covers_all_optional_fields() {
    let selection = AssetSelectionSnapshot {
        toolkit_view_id: "inspector".to_string(),
        asset_unit: "meters".to_string(),
        package_id: Some("package".to_string()),
        included_files: vec!["a".to_string()],
        subassets: vec![subasset(0)],
        ..AssetSelectionSnapshot::default()
    };

    assert_eq!(selection_metadata_summary_capacity(&selection), 5);
}

#[test]
fn selection_metadata_body_capacity_matches_emitted_sections() {
    let selection = AssetSelectionSnapshot {
        diagnostics: vec!["warning".to_string()],
        included_files: vec!["a".to_string(), "b".to_string()],
        subassets: vec![subasset(0), subasset(1), subasset(2)],
        ..AssetSelectionSnapshot::default()
    };

    assert_eq!(
        selection_metadata_body_capacity(&selection),
        1 + 1 + 2 + 1 + 3
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor848_selection_metadata_capacity_release_benchmark_marker() {
    std::hint::black_box(EDITOR848_SELECTION_METADATA_SUMMARY_CAPACITY_BENCH_V1);
}
