use std::path::Path;

use super::super::non_empty_string_value;

pub(super) fn assert_known_asset_kind(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    let asset_kind = non_empty_string_value(table, relative_path, context, field_name);
    assert_known_asset_kind_value(asset_kind, relative_path, context, field_name);
}

pub(super) fn assert_known_asset_kind_value(
    asset_kind: &str,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    assert!(
        matches!(
            asset_kind,
            "Data"
                | "Model"
                | "Mesh"
                | "Material"
                | "MaterialGraph"
                | "Texture"
                | "Shader"
                | "Scene"
                | "Sound"
                | "Font"
                | "PhysicsMaterial"
                | "NavMesh"
                | "NavigationSettings"
                | "Terrain"
                | "TerrainLayerStack"
                | "TileSet"
                | "TileMap"
                | "Prefab"
                | "AnimationSkeleton"
                | "AnimationClip"
                | "AnimationSequence"
                | "AnimationGraph"
                | "AnimationStateMachine"
                | "UiLayout"
                | "UiWidget"
                | "UiStyle"
        ),
        "plugin manifest {relative_path:?} {context} `{field_name}` asset kind `{asset_kind}` should be a known ResourceKind"
    );
}
