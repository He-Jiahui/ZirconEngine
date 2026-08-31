use crate::ui::binding::{
    AssetCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, SelectionCommand,
};

#[test]
fn selection_command_binding_roundtrips_for_scene_node_selection() {
    let binding = EditorUiBinding::new(
        "HierarchyView",
        "SceneNodeSelect",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode { node_id: 3 }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"HierarchyView/SceneNodeSelect:onClick(SelectionCommand.SelectSceneNode(3))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn asset_command_binding_roundtrips_for_asset_open() {
    let binding = EditorUiBinding::new(
        "ProjectView",
        "OpenAsset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::asset_command(AssetCommand::OpenAsset {
            asset_locator: "crate://prefabs/player.prefab".to_string(),
        }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"ProjectView/OpenAsset:onClick(AssetCommand.OpenAsset("crate://prefabs/player.prefab"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn asset_command_binding_roundtrips_for_import_model() {
    let binding = EditorUiBinding::new(
        "AssetsView",
        "ImportModel",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::asset_command(AssetCommand::ImportModel),
    );

    assert_eq!(
        binding.native_binding(),
        r#"AssetsView/ImportModel:onClick(AssetCommand.ImportModel())"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn asset_command_binding_roundtrips_for_relocation_drop() {
    let binding = EditorUiBinding::new(
        "AssetTree",
        "RelocateAsset",
        EditorUiEventKind::Drop,
        EditorUiBindingPayload::asset_command(AssetCommand::RelocateAsset {
            asset_uuid: "00112233-4455-6677-8899-aabbccddeeff".to_string(),
            target_locator: "res://environment/cube.zmodel".to_string(),
        }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"AssetTree/RelocateAsset:onDrop(AssetCommand.RelocateAsset("00112233-4455-6677-8899-aabbccddeeff","res://environment/cube.zmodel"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn asset_command_binding_roundtrips_for_deletion() {
    let binding = EditorUiBinding::new(
        "AssetContextMenu",
        "DeleteAsset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::asset_command(AssetCommand::DeleteAsset {
            asset_uuid: "00112233-4455-6677-8899-aabbccddeeff".to_string(),
        }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"AssetContextMenu/DeleteAsset:onClick(AssetCommand.DeleteAsset("00112233-4455-6677-8899-aabbccddeeff"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}
