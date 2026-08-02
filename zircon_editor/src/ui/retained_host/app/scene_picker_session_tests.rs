use zircon_runtime::asset::project::PreviewState;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
};

use super::scene_picker_session::{
    ScenePickerMode, scene_create_request_for_query, scene_entries_from_catalog,
    scene_entry_for_open_submission, scene_open_palette_state,
};

fn catalog_record(locator: &str, kind: ResourceKind) -> EditorAssetCatalogRecord {
    EditorAssetCatalogRecord {
        uuid: format!("asset-{locator}"),
        id: locator.to_string(),
        locator: locator.to_string(),
        kind,
        display_name: locator.to_string(),
        file_name: locator.to_string(),
        extension: "toml".to_string(),
        preview_state: PreviewState::Dirty,
        meta_path: String::new(),
        preview_artifact_path: String::new(),
        source_mtime_unix_ms: 0,
        source_hash: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        direct_reference_uuids: Vec::new(),
    }
}

#[test]
fn scene_picker_catalog_exposes_only_valid_project_scene_assets() {
    let catalog = EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            catalog_revision: 9,
            assets: vec![
                catalog_record("res://levels/main.scene.toml", ResourceKind::Scene),
                catalog_record(
                    "res://materials/ground.material.toml",
                    ResourceKind::Material,
                ),
                catalog_record("res://levels/readme.txt", ResourceKind::Scene),
            ],
            ..Default::default()
        },
        7,
    );

    let entries = scene_entries_from_catalog(&catalog);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].scene_uri(), "res://levels/main.scene.toml");
    assert_eq!(
        entries[0].command_source(),
        ScenePickerMode::Open.command_source()
    );
}

#[test]
fn scene_picker_catalog_orders_project_scenes_for_stable_selection() {
    let catalog = EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            assets: vec![
                catalog_record("res://levels/Zebra.scene.toml", ResourceKind::Scene),
                catalog_record("res://levels/alpha.scene.toml", ResourceKind::Scene),
                catalog_record("res://levels/Beta.scene.toml", ResourceKind::Scene),
            ],
            ..Default::default()
        },
        4,
    );

    let entries = scene_entries_from_catalog(&catalog);
    let ordered_uris = entries
        .iter()
        .map(|entry| entry.scene_uri())
        .collect::<Vec<_>>();

    assert_eq!(
        ordered_uris,
        vec![
            "res://levels/alpha.scene.toml",
            "res://levels/Beta.scene.toml",
            "res://levels/Zebra.scene.toml",
        ]
    );
    assert_eq!(
        scene_open_palette_state(&catalog, "zebra", 0, false).selected_command_id,
        "scene-picker-open-2",
        "command IDs must be assigned after the stable display ordering"
    );
}

#[test]
fn scene_picker_catalog_deduplicates_repeated_scene_locators() {
    let catalog = EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            assets: vec![
                catalog_record("res://levels/main.scene.toml", ResourceKind::Scene),
                catalog_record("res://levels/main.scene.toml", ResourceKind::Scene),
                catalog_record("res://levels/secondary.scene.toml", ResourceKind::Scene),
            ],
            ..Default::default()
        },
        6,
    );

    let entries = scene_entries_from_catalog(&catalog);

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].scene_uri(), "res://levels/main.scene.toml");
    assert_eq!(entries[1].scene_uri(), "res://levels/secondary.scene.toml");
    assert_eq!(
        scene_open_palette_state(&catalog, "main", 0, false).total_match_count,
        1,
        "a duplicated catalog record must not produce duplicate picker rows"
    );
}

#[test]
fn scene_picker_submission_rejects_a_command_hidden_by_the_current_query() {
    let catalog = EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            assets: vec![
                catalog_record("res://levels/alpha.scene.toml", ResourceKind::Scene),
                catalog_record("res://levels/beta.scene.toml", ResourceKind::Scene),
            ],
            ..Default::default()
        },
        5,
    );
    let entries = scene_entries_from_catalog(&catalog);

    let selected = scene_entry_for_open_submission(&entries, "scene-picker-open-1", "BETA", 0)
        .expect("a visible scene command should remain submit-able");
    assert_eq!(selected.scene_uri(), "res://levels/beta.scene.toml");
    assert!(
        scene_entry_for_open_submission(&entries, "scene-picker-open-0", "BETA", 0).is_err(),
        "a stale command from an earlier query must not open a hidden scene"
    );
}

#[test]
fn scene_picker_submission_rejects_a_command_hidden_by_the_current_result_window() {
    let catalog = EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            assets: (0..14)
                .map(|index| {
                    catalog_record(
                        &format!("res://levels/level-{index}.scene.toml"),
                        ResourceKind::Scene,
                    )
                })
                .collect(),
            ..Default::default()
        },
        8,
    );
    let entries = scene_entries_from_catalog(&catalog);

    assert!(
        scene_entry_for_open_submission(&entries, "scene-picker-open-13", "", 12).is_ok(),
        "the selected item on the active result window must remain submit-able"
    );
    assert!(
        scene_entry_for_open_submission(&entries, "scene-picker-open-0", "", 12).is_err(),
        "a command from an earlier result window must not bypass the visible list"
    );
}

#[test]
fn create_scene_picker_requires_an_explicit_project_scene_uri() {
    let request = scene_create_request_for_query("res://levels/new.scene.toml")
        .expect("project scene URI should be accepted");
    assert_eq!(
        request.scene_uri().to_string(),
        "res://levels/new.scene.toml"
    );

    for invalid in [
        "levels/new.scene.toml",
        "res://levels/new.toml",
        "file://outside.scene.toml",
    ] {
        assert!(
            scene_create_request_for_query(invalid).is_err(),
            "{invalid} must not become a project scene creation request"
        );
    }
}

#[test]
fn scene_picker_queries_a_catalog_snapshot_and_windows_its_results() {
    let catalog = EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            catalog_revision: 21,
            assets: (0..14)
                .map(|index| {
                    catalog_record(
                        &format!("res://levels/level-{index}.scene.toml"),
                        ResourceKind::Scene,
                    )
                })
                .chain(std::iter::once(catalog_record(
                    "res://materials/ground.material.toml",
                    ResourceKind::Material,
                )))
                .collect(),
            ..Default::default()
        },
        11,
    );

    let final_page = scene_open_palette_state(&catalog, "", 12, true);

    assert_eq!(final_page.catalog_generation, 11);
    assert_eq!(final_page.total_match_count, 14);
    assert_eq!(final_page.window_offset, 12);
    assert_eq!(final_page.focused_index, 1);
    assert_eq!(
        final_page.selected_command_id, "scene-picker-open-13",
        "the final page should focus its last visible scene"
    );
    assert!(matches!(
        final_page.commands,
        UiValue::Array(commands) if commands.len() == 2
    ));

    let queried = scene_open_palette_state(&catalog, "LEVEL-9", 0, false);
    assert_eq!(queried.total_match_count, 1);
    assert_eq!(queried.window_offset, 0);
    assert_eq!(queried.selected_command_id, "scene-picker-open-13");
}
