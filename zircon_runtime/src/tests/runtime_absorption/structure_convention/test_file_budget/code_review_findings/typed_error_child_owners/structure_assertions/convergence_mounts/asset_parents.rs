use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_asset_parents_are_folder_backed(
    sources: &TypedErrorConvergenceMountSources,
) {
    assert_contains_all(
        "asset loaders typed-error parent mounts focused child owners",
        &sources.asset_loaders_parent,
        &[
            "#[path = \"asset_loaders/animation_binary.rs\"]",
            "mod animation_binary;",
            "#[path = \"asset_loaders/artifact_importer.rs\"]",
            "mod artifact_importer;",
            "#[path = \"asset_loaders/mesh_obj.rs\"]",
            "mod mesh_obj;",
            "#[path = \"asset_loaders/texture.rs\"]",
            "mod texture;",
        ],
    );
    assert_eq!(
        sources.asset_loaders_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/asset_loaders.rs should only mount child test owners"
    );
    assert_contains_all(
        "asset records typed-error parent mounts focused child owners",
        &sources.asset_records_parent,
        &[
            "#[path = \"asset_records/authoring.rs\"]",
            "mod authoring;",
            "#[path = \"asset_records/font.rs\"]",
            "mod font;",
            "#[path = \"asset_records/meta.rs\"]",
            "mod meta;",
            "#[path = \"asset_records/navigation.rs\"]",
            "mod navigation;",
            "#[path = \"asset_records/sound.rs\"]",
            "mod sound;",
            "#[path = \"asset_records/zshader.rs\"]",
            "mod zshader;",
        ],
    );
    assert_eq!(
        sources.asset_records_parent.matches("#[test]").count(),
        0,
        "typed_error_convergence/asset_records.rs should only mount child test owners"
    );
}
