use super::*;

#[test]
fn runtime_15_asset_gltf_importer_tests_are_folder_backed() {
    let parent = read_runtime_src("asset/tests/assets/gltf_importer.rs");
    let basic_import = read_runtime_src("asset/tests/assets/gltf_importer/basic_import.rs");
    let labeled_subassets =
        read_runtime_src("asset/tests/assets/gltf_importer/labeled_subassets.rs");
    let multi_primitive = read_runtime_src("asset/tests/assets/gltf_importer/multi_primitive.rs");
    let external_inputs = read_runtime_src("asset/tests/assets/gltf_importer/external_inputs.rs");
    let vertex_channels = read_runtime_src("asset/tests/assets/gltf_importer/vertex_channels.rs");
    let material_transforms =
        read_runtime_src("asset/tests/assets/gltf_importer/material_transforms.rs");
    let multi_scene = read_runtime_src("asset/tests/assets/gltf_importer/multi_scene.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "asset glTF importer parent test module mounts",
        &parent,
        &[
            "mod basic_import;",
            "mod labeled_subassets;",
            "mod multi_primitive;",
            "mod external_inputs;",
            "mod vertex_channels;",
            "mod material_transforms;",
            "mod multi_scene;",
            "fn entry_for_label",
            "fn assert_cooked_virtual_geometry",
            "fn label_uri",
        ],
    );
    for moved_test in [
        "fn importer_decodes_triangle_gltf_into_model_asset",
        "fn importer_emits_bevy_style_gltf_labeled_subassets",
        "fn importer_emits_gltf_multi_primitive_material_labels",
        "fn importer_reports_missing_gltf_external_buffer",
        "fn importer_preserves_gltf_skinning_channels_on_model_vertices",
        "fn importer_preserves_gltf_texture_transform_on_standard_material_slots",
        "fn importer_emits_gltf_multi_scene_labels",
    ] {
        assert!(
            !parent.contains(moved_test),
            "asset/tests/assets/gltf_importer.rs should mount child test owners instead of defining {moved_test}"
        );
    }
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "asset/tests/assets/gltf_importer.rs should not keep executable tests in the parent module"
    );
    let child_sources = [
        basic_import.as_str(),
        labeled_subassets.as_str(),
        multi_primitive.as_str(),
        external_inputs.as_str(),
        vertex_channels.as_str(),
        material_transforms.as_str(),
        multi_scene.as_str(),
    ];
    assert_eq!(
        child_sources
            .iter()
            .map(|source| source.matches("#[test]").count())
            .sum::<usize>(),
        13,
        "asset glTF importer child modules should preserve the original 13 tests"
    );

    assert_contains_all(
        "asset glTF basic child owns root import contracts",
        &basic_import,
        &[
            "use super::*;",
            "fn importer_decodes_triangle_gltf_into_model_asset",
            "fn default_importer_decodes_gltf_without_first_wave_plugin_fixture",
        ],
    );
    assert_contains_all(
        "asset glTF labeled child owns subasset and skeleton contracts",
        &labeled_subassets,
        &[
            "use super::*;",
            "fn importer_emits_bevy_style_gltf_labeled_subassets",
            "fn importer_emits_synthetic_skeleton_for_node_animation_without_skin",
            "Skin0/InverseBindMatrices",
        ],
    );
    assert_contains_all(
        "asset glTF multi-primitive child owns material label contracts",
        &multi_primitive,
        &[
            "use super::*;",
            "fn importer_emits_gltf_multi_primitive_material_labels",
            "Mesh0/Primitive1",
        ],
    );
    assert_contains_all(
        "asset glTF external input child owns file/error contracts",
        &external_inputs,
        &[
            "use super::*;",
            "fn importer_decodes_gltf_external_texture_image",
            "fn importer_reports_missing_gltf_external_buffer",
            "fn importer_rejects_unsupported_gltf_primitive_mode",
        ],
    );
    assert_contains_all(
        "asset glTF vertex child owns channel preservation contracts",
        &vertex_channels,
        &[
            "use super::*;",
            "fn importer_preserves_gltf_skinning_channels_on_model_vertices",
            "fn importer_preserves_gltf_tangent_and_color_channels_on_model_vertices",
            "fn importer_preserves_gltf_texcoord_1_on_model_vertices_and_mesh_subasset",
        ],
    );
    assert_contains_all(
        "asset glTF material child owns texture transform contracts",
        &material_transforms,
        &[
            "use super::*;",
            "fn importer_preserves_gltf_texture_transform_on_standard_material_slots",
            "assert_texture_slot_transform",
        ],
    );
    assert_contains_all(
        "asset glTF multi-scene child owns scene label contracts",
        &multi_scene,
        &[
            "use super::*;",
            "fn importer_emits_gltf_multi_scene_labels",
            "assert_scene_entity",
        ],
    );

    for source in [parent.as_str()].into_iter().chain(child_sources) {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "asset glTF importer parent and child test owners should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
