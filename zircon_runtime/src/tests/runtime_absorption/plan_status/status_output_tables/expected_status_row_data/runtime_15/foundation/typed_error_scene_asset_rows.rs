use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 F5 fixed world mutation typed errors",
        &[
            "runtime_15_fixed_world_mutation_typed_errors_static_passed_cargo_deferred",
            "scene/world/component_access.rs",
            "scene/world/hierarchy.rs",
            "review_f5_fixed_world_mutation_uses_scene_error_variants",
        ],
    ),
    (
        "Runtime 15 F5 asset authoring typed errors",
        &[
            "runtime_15_asset_authoring_typed_errors_static_passed_cargo_deferred",
            "asset/assets/authoring.rs",
            "AssetAuthoringError",
            "review_f5_asset_authoring_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 navigation asset typed errors",
        &[
            "runtime_15_navigation_asset_typed_errors_static_passed_cargo_deferred",
            "asset/assets/navigation.rs",
            "NavigationAssetError",
            "review_f5_navigation_asset_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 font asset typed errors",
        &[
            "runtime_15_font_asset_typed_errors_static_passed_cargo_deferred",
            "asset/assets/font.rs",
            "FontAssetError::Parse",
            "review_f5_font_asset_uses_typed_error_source",
        ],
    ),
    (
        "Runtime 15 F5 sound asset typed errors",
        &[
            "runtime_15_sound_asset_typed_errors_static_passed_cargo_deferred",
            "asset/assets/sound.rs",
            "SoundAssetError::UnsupportedSpeakerMaskBits",
            "review_f5_sound_asset_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 sound asset panic-free read helpers",
        &[
            "runtime_15_sound_asset_panic_free_read_helpers_static_passed_cargo_deferred",
            "read_fixed_bytes",
            ".unwrap()",
            "review_f5_sound_asset_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F7 artifact cache JSON number typed errors",
        &[
            "runtime_15_artifact_cache_json_number_typed_errors_static_passed_cargo_deferred",
            "asset/artifact/cache_payload/json_value.rs",
            "CachedJsonNonFiniteNumber",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    ),
    (
        "Runtime 15 F5 zshader v2 user definition migration",
        &[
            "runtime_15_zshader_v2_user_definition_migration_static_passed_cargo_deferred",
            "asset/assets/shader/zshader.rs",
            "ZShaderV2Error::ForbiddenField",
            "review_f5_zshader_v2_replaces_user_shader_definitions",
        ],
    ),
    (
        "Runtime 15 F5 asset meta typed errors",
        &[
            "runtime_15_asset_meta_typed_errors_static_passed_cargo_deferred",
            "asset/project/meta.rs",
            "AssetMetaError::UnsupportedFormatVersion",
            "review_f5_asset_meta_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 texture loader typed errors",
        &[
            "runtime_15_texture_loader_typed_errors_static_passed_cargo_deferred",
            "asset/load/texture.rs",
            "TextureLoadError::OpenImage",
            "review_f5_texture_loader_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 mesh loader typed errors",
        &[
            "runtime_15_mesh_loader_typed_errors_static_passed_cargo_deferred",
            "asset/load/mesh.rs",
            "MeshLoadError::UnsupportedFormat",
            "review_f5_mesh_loader_and_obj_decoder_use_typed_errors",
        ],
    ),
];
