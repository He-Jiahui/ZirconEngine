use super::*;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogSnapshotRecord, EditorAssetFolderRecord,
};

fn catalog_with_folder(folder_id: &str) -> EditorAssetCatalogGeneration {
    EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            folders: vec![EditorAssetFolderRecord {
                folder_id: folder_id.to_owned(),
                parent_folder_id: Some("res://".to_owned()),
                locator_prefix: folder_id.to_owned(),
                display_name: "Folder".to_owned(),
                child_folder_ids: Vec::new(),
                direct_asset_uuids: Vec::new(),
                recursive_asset_count: 0,
            }],
            ..EditorAssetCatalogSnapshotRecord::default()
        },
        7,
    )
}

#[test]
fn u09_folder_create_prompt_keeps_the_context_parent_and_accepts_unicode_names() {
    let request = AssetNamePromptRequest::create_folder("res://scenes").unwrap();
    assert_eq!(request.initial_query(), "");
    assert_eq!(
        request.target("关卡 One").unwrap().to_string(),
        "res://scenes/关卡 One"
    );
    let root = AssetNamePromptRequest::create_folder("res://").unwrap();
    assert_eq!(root.source(), "res://");
    assert!(root.source_is_current(&catalog_with_folder("res://")));
    assert_eq!(root.target("Levels").unwrap().to_string(), "res://Levels");
    assert!(AssetNamePromptRequest::create_folder("res://#label").is_err());
}

#[test]
fn u09_folder_rename_prompt_renames_only_the_terminal_segment() {
    let request = AssetNamePromptRequest::rename_folder("res://world/old").unwrap();
    assert_eq!(request.initial_query(), "old");
    assert_eq!(
        request.target("new").unwrap().to_string(),
        "res://world/new"
    );
    assert_eq!(request.target("old"), Err("asset.name_prompt.no_change"));
    assert!(AssetNamePromptRequest::rename_folder("res://").is_err());
}

#[test]
fn u09_asset_rename_prompt_preserves_uuid_parent_and_complete_filename() {
    let uuid = "00112233-4455-6677-8899-aabbccddeeff";
    let request =
        AssetNamePromptRequest::rename_asset(uuid, "res://scenes/main.scene.toml").unwrap();
    assert_eq!(request.initial_query(), "main.scene.toml");
    assert_eq!(
        request.target("level.scene.toml").unwrap().to_string(),
        "res://scenes/level.scene.toml"
    );
    assert!(
        matches!(request, AssetNamePromptRequest::RenameAsset { ref asset_uuid, .. } if asset_uuid == uuid)
    );
    assert!(AssetNamePromptRequest::rename_asset(uuid, "res://models/ship.glb#mesh=1").is_err());
    assert!(AssetNamePromptRequest::rename_asset(uuid, "package://builtin/ship.glb").is_err());
}

#[test]
fn u09_asset_rename_prompt_preserves_importer_extension() {
    let request = AssetNamePromptRequest::rename_asset("asset", "res://source.json").unwrap();
    assert_eq!(
        request.target("other.obj"),
        Err("asset.name_prompt.invalid_name")
    );
    assert_eq!(
        request.target("other"),
        Err("asset.name_prompt.invalid_name")
    );
    assert_eq!(
        request.target("other.json").unwrap().to_string(),
        "res://other.json"
    );
    assert!(request.target("other.JSON").is_ok());
    for (source, changed, valid) in [
        (
            "res://level.scene.toml",
            "level.prefab.toml",
            "other.scene.toml",
        ),
        (
            "res://walk.clip.zranim",
            "walk.graph.zranim",
            "run.clip.zranim",
        ),
        (
            "res://grid.albedo.png",
            "grid.albedo.obj",
            "grid.normal.png",
        ),
    ] {
        let request = AssetNamePromptRequest::rename_asset("asset", source).unwrap();
        assert_eq!(
            request.target(changed),
            Err("asset.name_prompt.invalid_name")
        );
        assert!(request.target(valid).is_ok());
    }
}

#[test]
fn u09_folder_prompt_rejects_unsafe_names_before_showing_a_candidate() {
    let request = AssetNamePromptRequest::create_folder("res://safe").unwrap();
    for name in [
        "",
        ".",
        "..",
        "../escape",
        "a/b",
        "a\\b",
        "res://escape",
        "x#label",
        " x",
        "x ",
        "x.",
        "CON",
        "nul.txt",
        "COM1",
        "LPT9",
        "a\0b",
        "a\nb",
    ] {
        assert_eq!(
            request.target(name),
            Err("asset.name_prompt.invalid_name"),
            "{name:?}"
        );
    }
    assert!(request.target("Content").is_ok());
}

#[test]
fn u09_folder_prompt_refuses_an_indexed_destination_and_stale_source() {
    let catalog = catalog_with_folder("res://levels");
    let create = AssetNamePromptRequest::create_folder("res://").unwrap();
    assert_eq!(
        available_target(&create, &catalog, "levels"),
        Err("asset.name_prompt.occupied")
    );
    let rename = AssetNamePromptRequest::rename_folder("res://levels").unwrap();
    assert!(rename.source_is_current(&catalog));
    assert!(!rename.source_is_current(&EditorAssetCatalogGeneration::default()));
}

#[test]
fn u09_asset_name_prompt_palette_is_bounded_to_zero_or_one_candidate() {
    let request = AssetNamePromptRequest::create_folder("res://").unwrap();
    let catalog = catalog_with_folder("res://existing");
    let i18n = EditorI18nService::default();
    for query in ["", "../bad", "existing"] {
        let state = prompt_state(&request, &catalog, query, &i18n);
        assert_eq!(state.total_match_count, 0);
        assert_eq!(state.commands, UiValue::Array(Vec::new()));
        assert_eq!(state.focused_index, -1);
    }
    let long_name = "Long localized 文件夹 ".repeat(8).trim_end().to_owned();
    let state = prompt_state(&request, &catalog, &long_name, &i18n);
    assert_eq!(state.total_match_count, 1);
    assert_eq!(state.catalog_generation, 7);
    assert_eq!(state.window_offset, 0);
    assert_eq!(state.selected_command_id, PROMPT_COMMIT_ID);
    let UiValue::Array(commands) = state.commands else {
        panic!("one command array is required")
    };
    let UiValue::Map(command) = &commands[0] else {
        panic!("a command must retain semantic fields")
    };
    assert_eq!(
        command.get("label"),
        Some(&UiValue::String(format!("Create res://{long_name}")))
    );
    assert_eq!(
        command.get("source"),
        Some(&UiValue::String(PROMPT_SOURCE.to_owned()))
    );
}
