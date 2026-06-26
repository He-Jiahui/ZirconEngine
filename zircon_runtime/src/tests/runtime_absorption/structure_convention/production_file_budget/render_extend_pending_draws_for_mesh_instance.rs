use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_extend_pending_draws_tests_are_child_owner() {
    let root = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
    );
    let tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/tests.rs",
    );

    let plan_02 = read_repo("docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");

    assert_contains_all(
        "extend_pending_draws parent keeps production build helpers and child test mount",
        &root,
        &[
            "pub(super) fn extend_pending_draws_for_mesh_instance(",
            "fn dynamic_direct_mesh_primitive(",
            "fn push_dynamic_mesh_draws(",
            "fn push_prepared_mesh_draws(",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );

    for moved_test_anchor in [
        "fn morphed_mesh_asset_primitive_ignores_zero_weights_for_static_direct_mesh_fallback(",
        "fn morphed_mesh_asset_primitive_applies_nonzero_weights_for_dynamic_direct_mesh(",
        "fn morph_shape_signature_tracks_mesh_and_weights(",
        "fn skinned_gpu_source_candidate_requires_palette(",
        "fn morph_test_mesh(",
        "MeshMorphTargetAsset",
    ] {
        assert!(
            !root.contains(moved_test_anchor),
            "extend_pending_draws parent should delegate test anchor `{moved_test_anchor}` to tests.rs"
        );
    }

    assert_contains_all(
        "extend_pending_draws tests child owns morph and skinned GPU-source coverage",
        &tests,
        &[
            "fn morphed_mesh_asset_primitive_ignores_zero_weights_for_static_direct_mesh_fallback(",
            "fn morphed_mesh_asset_primitive_applies_nonzero_weights_for_dynamic_direct_mesh(",
            "fn morph_shape_signature_tracks_mesh_and_weights(",
            "fn skinned_gpu_source_candidate_requires_palette(",
            "fn morph_test_mesh(",
        ],
    );

    for (path, source) in [
        ("extend_pending_draws_for_mesh_instance.rs", root.as_str()),
        (
            "extend_pending_draws_for_mesh_instance/tests.rs",
            tests.as_str(),
        ),
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
                "extend_pending_draws tests owner split",
                "render_plan02_extend_pending_draws_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/tests.rs",
                "runtime_15_extend_pending_draws_tests_are_child_owner",
            ],
        );
    }
}
