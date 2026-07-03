use super::*;

const STATUS: &str = "render_plan08_morph_geometry_source_selection_static_passed_wgpu_deferred";

#[test]
fn runtime_15_morph_geometry_source_selection_is_wired() {
    let geometry_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_draw/geometry_source.rs");
    let queue_profile =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs");
    let build_mod =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/mod.rs");
    let selection = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/geometry_source_selection.rs",
    );
    let pending = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs",
    );
    let build =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs");
    let extend = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
    );
    let mesh_pass =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let mesh_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "mesh draw geometry source exposes GPU morph variants",
        &format!("{geometry_source}{queue_profile}{mesh_pass}"),
        &[
            "DynamicGpuMorphedSource",
            "DynamicGpuSkinnedMorphedSource",
            "GEOMETRY_SOURCE_ID_MORPHED_MESH",
            "GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH",
            "uses_gpu_morph_payload_source",
            "mesh_pass_build_context_resolves_gpu_skinned_morphed_as_skinned_morphed_variant",
        ],
    );
    assert_contains_all(
        "pending draw selection keeps GPU and CPU morph sources distinct",
        &format!("{build_mod}{selection}{pending}{build}{extend}"),
        &[
            "mod geometry_source_selection;",
            "PendingMeshGeometry::GpuMorphed",
            "pending_mesh_source_selection(",
            "pending_mesh_draw_queue_profile(",
            "pending_draw_has_enabled_skinned_gpu_source(",
            "skinned_gpu_source_geometry_source(",
            "morph_payload_slot.is_some()",
            "PendingSkinnedGpuSource::Prepared(_) if has_morph_payload_slot",
            "MeshDrawGeometrySource::DynamicGpuMorphedSource",
            "MeshDrawGeometrySource::DynamicGpuSkinnedMorphedSource",
            "MeshDrawGeometrySource::DynamicCpuMorphedSource",
            "MeshDrawGeometrySource::DynamicCpuMorphedGpuSkinningSource",
            "morph_payload_available",
        ],
    );

    for (path, source) in [
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs",
            build.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/geometry_source_selection.rs",
            selection.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/morph_geometry_source_selection.rs",
            include_str!("morph_geometry_source_selection.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 900,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("mesh cache doc", mesh_cache_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render Plan 08 session", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Morph geometry-source selection",
                STATUS,
                "runtime_15_morph_geometry_source_selection_is_wired",
            ],
        );
    }
}
