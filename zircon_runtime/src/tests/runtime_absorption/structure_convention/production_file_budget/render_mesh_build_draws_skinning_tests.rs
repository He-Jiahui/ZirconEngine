use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_build_mesh_draws_skinning_tests_are_child_owner() {
    let root =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs");
    let tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning/tests.rs",
    );

    let plan_02 = read_repo(
        "docs/plans/zircon_runtime/render/02/2026-07-09-mesh-draw-command-pipeline-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");

    assert_contains_all(
        "skinning parent keeps production palette and CPU fallback helpers plus child test mount",
        &root,
        &[
            "pub(super) struct SkinnedMeshJointPalette",
            "pub(super) struct SkinnedMeshPreparedPrimitive",
            "pub(super) fn skin_mesh_asset_primitive(",
            "pub(super) fn prepare_skinned_mesh_asset_primitive(",
            "fn skin_vertex(",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );

    for moved_test_anchor in [
        "fn joint_palette_composes_pose_world_against_bind_world_matrices(",
        "fn joint_palette_reports_missing_parent_bone_reference(",
        "fn joint_palette_storage_packs_gpu_matrices_and_count(",
        "fn joint_palette_storage_rejects_current_storage_limit_overflow(",
        "fn prepared_skinned_model_primitive_keeps_cpu_skinning_when_palette_exceeds_storage_limit(",
        "fn skin_model_primitive_rotates_weighted_vertex_around_joint_bind_origin(",
        "fn skin_mesh_asset_primitive_converts_direct_mesh_attributes_before_skinning(",
        "fn skin_mesh_asset_primitive_applies_morph_weights_before_skinning(",
        "fn prepare_skinned_mesh_asset_primitive_keeps_morphed_shader_source_before_cpu_skinning(",
        "fn unit_test_skeleton(",
        "fn oversized_storage_skeleton(",
        "fn joint_quarter_turn_pose(",
    ] {
        assert!(
            !root.contains(moved_test_anchor),
            "skinning parent should delegate test anchor `{moved_test_anchor}` to tests.rs"
        );
    }

    assert_contains_all(
        "skinning tests child owns joint palette, CPU fallback, direct mesh, and morph coverage",
        &tests,
        &[
            "use super::{",
            "SKINNED_MESH_MAX_JOINT_MATRICES",
            "fn joint_palette_composes_pose_world_against_bind_world_matrices(",
            "fn joint_palette_reports_missing_parent_bone_reference(",
            "fn joint_palette_storage_packs_gpu_matrices_and_count(",
            "fn joint_palette_storage_rejects_current_storage_limit_overflow(",
            "fn prepared_skinned_model_primitive_keeps_cpu_skinning_when_palette_exceeds_storage_limit(",
            "fn skin_model_primitive_rotates_weighted_vertex_around_joint_bind_origin(",
            "fn skin_mesh_asset_primitive_converts_direct_mesh_attributes_before_skinning(",
            "fn skin_mesh_asset_primitive_applies_morph_weights_before_skinning(",
            "fn prepare_skinned_mesh_asset_primitive_keeps_morphed_shader_source_before_cpu_skinning(",
            "fn unit_test_skeleton(",
            "fn oversized_storage_skeleton(",
            "fn joint_quarter_turn_pose(",
        ],
    );

    for (path, source) in [
        ("skinning.rs", root.as_str()),
        ("skinning/tests.rs", tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 02", plan_02.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("mesh pass docs", mesh_pass_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "build_mesh_draws skinning tests owner split",
                "render_plan02_build_mesh_draws_skinning_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning/tests.rs",
                "runtime_15_build_mesh_draws_skinning_tests_are_child_owner",
            ],
        );
    }
}
