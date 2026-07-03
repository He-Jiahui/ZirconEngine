use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_gltf_labeled_material_subassets_are_child_owner() {
    let parent = read_runtime_src("asset/importer/ingest/gltf_labeled_subassets.rs");
    let material = read_runtime_src("asset/importer/ingest/gltf_labeled_subassets/material.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let importer_doc = read_repo("docs/zircon_runtime/asset/importer.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
    );

    assert_contains_all(
        "glTF labeled parent keeps texture/mesh/scene orchestration and re-exports material child",
        &parent,
        &[
            "mod material;",
            "pub(crate) use self::material::add_gltf_material_subassets;",
            "pub(crate) fn add_gltf_texture_subassets",
            "pub(crate) fn add_gltf_mesh_subassets",
            "pub(crate) fn add_gltf_scene_subassets",
            "fn with_root_dependency_and_entry(",
            "pub(crate) fn gltf_label_reference(",
            "pub(crate) fn gltf_label_uri(",
        ],
    );
    for moved_owner in [
        "fn material_asset_from_gltf_material(",
        "struct GltfTextureSlotMetadata",
        "fn texture_info_metadata(",
        "fn normal_texture_metadata(",
        "fn occlusion_texture_metadata(",
        "fn texture_transform_extension_metadata(",
        "fn default_material_asset(",
        "fn insert_texture_slot(",
        "fn gltf_alpha_mode(",
        "fn default_pbr_shader_reference(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "asset/importer/ingest/gltf_labeled_subassets.rs should delegate {moved_owner} to gltf_labeled_subassets/material.rs"
        );
    }
    assert_contains_all(
        "glTF labeled material child owns PBR material asset and texture-slot metadata projection",
        &material,
        &[
            "pub(crate) fn add_gltf_material_subassets(",
            "fn material_asset_from_gltf_material(",
            "struct GltfTextureSlotMetadata",
            "fn texture_transform_extension_metadata(",
            "RenderMaterialTextureTransform",
            "MaterialTextureSlotValue::new",
            "fn default_material_asset(",
            "fn gltf_alpha_mode(",
            "fn default_pbr_shader_reference(",
            "with_root_dependency_and_entry",
        ],
    );

    for (path, source) in [
        (
            "asset/importer/ingest/gltf_labeled_subassets.rs",
            parent.as_str(),
        ),
        (
            "asset/importer/ingest/gltf_labeled_subassets/material.rs",
            material.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("importer doc", importer_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 glTF labeled material subasset owner split",
                "runtime_15_gltf_labeled_material_subasset_owner_split_static_passed_cargo_deferred",
                "asset/importer/ingest/gltf_labeled_subassets.rs",
                "asset/importer/ingest/gltf_labeled_subassets/material.rs",
                "runtime_15_gltf_labeled_material_subassets_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status map",
        &status_map,
        &[
            "Runtime 15 M4 glTF labeled material subasset owner split",
            "runtime_15_gltf_labeled_material_subasset_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "status-output date map",
        &date_map,
        &[
            "Runtime 15 M4 glTF labeled material subasset owner split",
            "2026-06-24",
        ],
    );
}
