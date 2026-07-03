use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_scene_world_project_io_mesh_is_child_owner() {
    let parent = read_runtime_src("scene/world/project_io.rs");
    let mesh = read_runtime_src("scene/world/project_io/mesh.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert_contains_all(
        "project I/O parent keeps scene document flow and delegates mesh projection",
        &parent,
        &[
            "mod mesh;",
            "use mesh::{mesh_from_asset, mesh_to_asset};",
            "mesh_from_asset(project, entity.mesh.as_ref())",
            "mesh_to_asset(project, record.mesh)?",
            "pub fn from_scene_asset",
            "pub fn to_scene_asset",
            "fn normalize_loaded_state",
        ],
    );
    for moved_owner in [
        "SceneMeshInstanceAsset",
        "SceneMeshLodLevelAsset",
        "SceneMeshPrimitiveBindingAsset",
        "MeshRendererPrimitiveBinding",
        "MeshRendererLodLevel",
        "model_handle_for_reference",
        "material_handle_for_reference",
        "reference_for_model_handle",
        "reference_for_mesh_handle",
        "reference_for_material_handle",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "scene/world/project_io.rs should delegate {moved_owner} to project_io/mesh.rs"
        );
    }
    assert_contains_all(
        "mesh child owns scene mesh asset and renderer projection",
        &mesh,
        &[
            "pub(super) fn mesh_from_asset",
            "pub(super) fn mesh_to_asset",
            "SceneMeshInstanceAsset",
            "SceneMeshLodLevelAsset",
            "SceneMeshPrimitiveBindingAsset",
            "MeshRendererPrimitiveBinding",
            "MeshRendererLodLevel",
            "model_handle_for_reference",
            "material_handle_for_reference",
            "reference_for_model_handle",
            "reference_for_mesh_handle",
            "reference_for_material_handle",
        ],
    );

    for (path, source) in [
        ("scene/world/project_io.rs", parent.as_str()),
        ("scene/world/project_io/mesh.rs", mesh.as_str()),
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
        ("scene ECS doc", ecs_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 scene world project I/O mesh owner split",
                "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
                "scene/world/project_io.rs",
                "scene/world/project_io/mesh.rs",
                "runtime_15_scene_world_project_io_mesh_is_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 scene world project I/O mesh owner split",
            "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
            "scene/world/project_io.rs",
            "scene/world/project_io/mesh.rs",
            "runtime_15_scene_world_project_io_mesh_is_child_owner",
        ],
    );
}
