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
            asset_path: "crate://prefabs/player.prefab".to_string(),
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
