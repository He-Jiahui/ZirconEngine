use super::{assert_contains_all, read_repo, read_runtime_src};

const DIRECT_CPU_MORPHED_DRAW_SOURCE_STATUS: &str =
    "render_plan08_direct_cpu_morphed_draw_source_metadata_check_passed_wgpu_deferred";

#[test]
fn runtime_15_extend_pending_draws_tests_are_child_owner() {
    let root = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
    );
    let material_inputs = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/material_inputs.rs",
    );
    let pending_mesh_draw = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs",
    );
    let geometry_source_selection = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/geometry_source_selection.rs",
    );
    let cache_extract = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs",
    );
    let cache_plan = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs",
    );
    let tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/tests.rs",
    );

    let plan_02 = read_repo("docs/plans/zircon_runtime/render/02/2026-07-09-mesh-draw-command-pipeline-output-records.md");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let mesh_pipeline_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let current_session_doc =
        read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "extend_pending_draws parent keeps production build helpers and child test mount",
        &root,
        &[
            "pub(super) fn extend_pending_draws_for_mesh_instance(",
            "fn dynamic_direct_mesh_primitive(",
            "cpu_morphed: bool",
            "PendingMeshGeometry::CpuMorphed",
            "fn push_dynamic_mesh_draws(",
            "fn push_prepared_mesh_draws(",
            "mod material_inputs;",
            "use self::material_inputs::{",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );

    assert_contains_all(
        "direct CPU-morphed pending geometry metadata reaches build and cache planning",
        &pending_mesh_draw,
        &["PendingMeshGeometry", "CpuMorphed(ModelPrimitiveAsset)"],
    );
    assert_contains_all(
        "draw build resolves direct CPU-morphed metadata conservatively",
        &geometry_source_selection,
        &[
            "pending_mesh_geometry_source(",
            "direct_cpu_morphed_geometry_stays_static_shader_fallback",
            "MeshDrawGeometrySource::DynamicCpuMorphedSource",
        ],
    );
    for (label, source) in [
        ("pending command cache extract", cache_extract.as_str()),
        ("pending command cache plan", cache_plan.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "pending_mesh_draw_queue_profile(",
                "pending_draw_has_enabled_skinned_gpu_source(",
            ],
        );
    }

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
    for moved_material_anchor in [
        "pub(super) fn material_tinted(",
        "pub(super) fn material_receive_shadows(",
        "pub(super) fn material_cast_shadows(",
        "pub(super) fn material_taa_reactive_mask_strength(",
        "pub(super) fn material_disabled_passes(",
        "pub(super) fn material_texture_set(",
        "fn material_output_target_texture_binding(",
    ] {
        assert!(
            !root.contains(moved_material_anchor),
            "extend_pending_draws parent should delegate material-input anchor `{moved_material_anchor}` to material_inputs.rs"
        );
        assert!(
            material_inputs.contains(moved_material_anchor),
            "material_inputs.rs should own material-input anchor `{moved_material_anchor}`"
        );
    }
    assert_contains_all(
        "extend_pending_draws material-input child owns material flags and texture binding assembly",
        &material_inputs,
        &[
            "MaterialTextureSet::new(",
            "MaterialTextureBinding::output_target(output_target)",
            "RenderImageUsage::Sampled",
            "streamer.normal_texture(texture_id)",
            "taa_reactive_mask_strength",
        ],
    );

    for (path, source) in [
        ("extend_pending_draws_for_mesh_instance.rs", root.as_str()),
        (
            "extend_pending_draws_for_mesh_instance/material_inputs.rs",
            material_inputs.as_str(),
        ),
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
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/material_inputs.rs",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance/tests.rs",
                "runtime_15_extend_pending_draws_tests_are_child_owner",
            ],
        );
    }

    for (label, doc) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("mesh pipeline cache doc", mesh_pipeline_cache_doc.as_str()),
        ("current render session doc", current_session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "Direct CPU-morphed draw-source metadata",
                DIRECT_CPU_MORPHED_DRAW_SOURCE_STATUS,
                "PendingMeshGeometry::CpuMorphed",
                "DynamicCpuMorphedSource",
                "GEOMETRY_SOURCE_ID_STATIC_MESH",
            ],
        );
    }
}
