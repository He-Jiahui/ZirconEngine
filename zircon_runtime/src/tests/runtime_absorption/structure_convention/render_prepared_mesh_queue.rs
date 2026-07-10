use super::{assert_contains_all, repo_path, runtime_src_path};

const CPU_MORPHED_GPU_SKINNING_DRAW_SOURCE_STATUS: &str =
    "render_plan08_cpu_morphed_gpu_skinning_draw_source_metadata_static_passed_cargo_deferred_active_lanes";
const DIRECT_CPU_MORPHED_DRAW_SOURCE_STATUS: &str =
    "render_plan08_direct_cpu_morphed_draw_source_metadata_check_passed_wgpu_deferred";

#[test]
fn runtime_15_prepared_mesh_queue_is_folder_backed() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue.rs");
    let stats_bridge =
        read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs");
    let stats_bridge_gpu_scene = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge/gpu_scene_stats.rs",
    );
    let stats_bridge_tests =
        read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs");
    let stats_bridge_virtual_geometry_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests/virtual_geometry.rs",
    );
    let stats_owner =
        read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue/stats.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs");
    let gpu_source_tests =
        read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue/tests/gpu_sources.rs");
    let mesh_draw_geometry_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_draw/geometry_source.rs");
    let mesh_draw_queue_profile =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_draw/queue_profile.rs");
    let geometry_source_selection = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/geometry_source_selection.rs",
    );
    let mesh_pass_processor =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let plan_02 = format!(
        "{}\n{}",
        read_repo(
            "docs/plans/zircon_runtime/render/02/2026-07-09-mesh-draw-command-pipeline-output-records.md",
        ),
        render_index,
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let mesh_pipeline_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");
    let module_convention = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let current_session_doc =
        read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "prepared queue parent keeps production stats owner surface",
        &parent,
        &[
            "pub(crate) struct PreparedMeshQueue",
            "pub(crate) fn prepare_mesh_queue",
            "pub(crate) fn summarize_prepared_mesh_queue_items",
            "fn repeated_group_stats",
            "mod stats;",
            "mod stats_bridge;",
            "mod stats_bridge_tests;",
            "mod tests;",
        ],
    );
    for moved_owner in [
        "mod tests {",
        "fn prepared_queue_stats_allow_early_z_only_for_opaque_and_alpha_mask",
        "fn prepared_queue_stats_carry_mesh_pass_command_buffer_counts",
        "fn prepared_queue_stats_carry_gpu_scene_counts",
        "fn render_gpu_scene_upload_path",
        "fn with_mesh_pass_command_buffer_stats",
        "fn item<K>",
        "fn profile(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "prepared_queue.rs should delegate {moved_owner} to prepared_queue/tests.rs"
        );
    }

    assert_contains_all(
        "prepared queue stats child owns queue and virtual-geometry stat DTOs",
        &stats_owner,
        &[
            "pub(crate) struct PreparedMeshQueueStats",
            "pub(crate) struct PreparedMeshVirtualGeometryIndirectStats",
            "pub(crate) struct PreparedMeshVirtualGeometryExecutionStats",
            "pub(crate) fn from_execution_draws(",
            "struct VirtualGeometryExecutionSegmentKey",
        ],
    );

    assert_contains_all(
        "prepared queue stats bridge child owns cross-subsystem stat forwarding",
        &stats_bridge,
        &[
            "impl PreparedMeshQueueStats",
            "with_pending_command_cache_extraction_stats",
            "with_pending_command_cache_plan_stats",
            "with_mesh_pass_command_buffer_stats",
            "with_mesh_draw_replay_stats",
            "mod gpu_scene_stats;",
            "residual_material_phase_draw_count",
            "pre_mesh_draw_static_command_cache_residual_material_phase_draw_count",
        ],
    );
    assert_contains_all(
        "prepared queue GPUScene stats bridge child owns upload stat forwarding",
        &stats_bridge_gpu_scene,
        &[
            "with_gpu_scene_stats",
            "fn render_gpu_scene_upload_path",
            "upload_report.uploaded_bytes",
            "upload_report.primitive_upload_range_count",
            "upload_report.instance_upload_range_count",
        ],
    );

    assert_contains_all(
        "prepared queue tests child owns queue stats behavior coverage",
        &tests,
        &[
            "mod gpu_sources;",
            "fn prepared_queue_stats_allow_early_z_only_for_opaque_and_alpha_mask",
            "fn prepared_queue_stats_require_repeated_direct_prepared_keys_for_batching",
            "fn prepared_queue_stats_count_dynamic_velocity_history_readiness",
            "fn prepared_queue_stats_count_cpu_morphed_gpu_skinning_source_as_dynamic_geometry",
            "fn prepared_queue_stats_count_direct_cpu_morphed_source_as_dynamic_geometry",
        ],
    );
    assert_contains_all(
        "prepared queue GPU-source tests child owns GPU morph, LOD, and skinned batching coverage",
        &gpu_source_tests,
        &[
            "fn prepared_queue_stats_count_gpu_morphed_sources_as_dynamic_geometry",
            "fn prepared_queue_stats_count_conventional_mesh_lod_draws",
            "fn prepared_queue_stats_count_gpu_skinned_velocity_with_previous_palette",
            "fn prepared_queue_stats_exclude_gpu_skinned_draws_from_direct_batch_candidates",
        ],
    );
    assert_contains_all(
        "cpu-morphed gpu-skinning draw source metadata stays explicit",
        &mesh_draw_geometry_source,
        &[
            "DynamicCpuMorphedGpuSkinningSource",
            "DynamicCpuMorphedSource",
            "uses_cpu_morphed_source",
            "uses_cpu_morphed_gpu_skinning_source",
            "GEOMETRY_SOURCE_ID_SKINNED_MESH",
            "GEOMETRY_SOURCE_ID_STATIC_MESH",
        ],
    );
    assert_contains_all(
        "cpu-morphed gpu-skinning queue profile keeps conservative shader id",
        &mesh_draw_queue_profile,
        &[
            "DynamicCpuMorphedGpuSkinningSource",
            "DynamicCpuMorphedSource",
            "queue_profile_preserves_cpu_morphed_gpu_skinning_source_metadata",
            "queue_profile_preserves_direct_cpu_morphed_source_metadata",
            "GEOMETRY_SOURCE_ID_SKINNED_MESH",
            "GEOMETRY_SOURCE_ID_STATIC_MESH",
        ],
    );
    assert_contains_all(
        "pending cpu-morphed gpu source maps to explicit draw source",
        &geometry_source_selection,
        &[
            "PendingSkinnedGpuSource::CpuMorphed",
            "MeshDrawGeometrySource::DynamicCpuMorphedGpuSkinningSource",
        ],
    );
    assert_contains_all(
        "mesh pass resolver keeps cpu-morphed gpu source on skinned variant",
        &mesh_pass_processor,
        &[
            "mesh_pass_build_context_resolves_cpu_morphed_gpu_skinning_as_skinned_variant",
            "DynamicCpuMorphedGpuSkinningSource",
            "GEOMETRY_SOURCE_ID_SKINNED_MESH",
        ],
    );
    for moved_stat_bridge_test in [
        "fn prepared_queue_stats_carry_mesh_pass_command_buffer_counts",
        "fn prepared_queue_stats_carry_pending_command_cache_plan_counts",
        "fn prepared_queue_stats_carry_pending_command_cache_extraction_counts",
        "fn prepared_queue_stats_carry_mesh_draw_replay_counts",
        "fn prepared_queue_stats_carry_gpu_scene_counts",
    ] {
        assert!(
            !tests.contains(moved_stat_bridge_test),
            "prepared_queue/tests.rs should delegate {moved_stat_bridge_test} to stats_bridge_tests.rs"
        );
    }

    assert_contains_all(
        "prepared queue stats bridge tests child owns forwarding coverage",
        &stats_bridge_tests,
        &[
            "mod virtual_geometry;",
            "fn prepared_queue_stats_carry_mesh_pass_command_buffer_counts",
            "fn prepared_queue_stats_carry_pending_command_cache_plan_counts",
            "fn prepared_queue_stats_carry_pending_command_cache_extraction_counts",
            "fn prepared_queue_stats_carry_mesh_draw_replay_counts",
            "fn prepared_queue_stats_carry_gpu_scene_counts",
            "PendingMeshCommandCacheExtractionStats",
            "MeshPassCommandBufferStats",
            "GpuSceneUploadReport",
        ],
    );
    assert_contains_all(
        "prepared queue stats bridge virtual geometry tests child owns execution coverage",
        &stats_bridge_virtual_geometry_tests,
        &[
            "fn prepared_queue_stats_carry_virtual_geometry_indirect_counts",
            "fn prepared_queue_stats_carry_virtual_geometry_execution_counts",
            "PreparedMeshVirtualGeometryExecutionStats::from_execution_draws",
            "fn execution_draw(",
            "fn non_virtual_geometry_indirect_draw()",
        ],
    );

    for (path, source, budget) in [
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue.rs",
            parent.as_str(),
            280,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs",
            stats_bridge.as_str(),
            120,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge/gpu_scene_stats.rs",
            stats_bridge_gpu_scene.as_str(),
            80,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/stats.rs",
            stats_owner.as_str(),
            220,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs",
            tests.as_str(),
            620,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/tests/gpu_sources.rs",
            gpu_source_tests.as_str(),
            180,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs",
            stats_bridge_tests.as_str(),
            220,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests/virtual_geometry.rs",
            stats_bridge_virtual_geometry_tests.as_str(),
            160,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the owner budget {budget}; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 02", plan_02.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("mesh pass doc", mesh_pass_doc.as_str()),
        ("module convention doc", module_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 02 prepared queue tests owner split",
                "Plan 02 prepared queue stats bridge tests owner split",
                "Plan 02 prepared queue stats bridge owner split",
                "render_plan02_prepared_queue_tests_owner_split_static_passed",
                "render_plan02_prepared_queue_stats_bridge_tests_owner_split_static_passed_cargo_lock_blocked",
                "render_plan02_prepared_queue_stats_bridge_owner_split_static_passed",
                "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs",
                "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests.rs",
                "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge_tests/virtual_geometry.rs",
                "graphics/scene/scene_renderer/mesh/prepared_queue/tests.rs",
                "runtime_15_prepared_mesh_queue_is_folder_backed",
            ],
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("mesh pipeline cache doc", mesh_pipeline_cache_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("current render session doc", current_session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "CPU-morphed GPU-skinning draw-source metadata",
                CPU_MORPHED_GPU_SKINNING_DRAW_SOURCE_STATUS,
                "DynamicCpuMorphedGpuSkinningSource",
                "uses_cpu_morphed_gpu_skinning_source",
                "GEOMETRY_SOURCE_ID_SKINNED_MESH",
            ],
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("mesh pipeline cache doc", mesh_pipeline_cache_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("current render session doc", current_session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Direct CPU-morphed draw-source metadata",
                DIRECT_CPU_MORPHED_DRAW_SOURCE_STATUS,
                "DynamicCpuMorphedSource",
                "uses_cpu_morphed_source",
                "GEOMETRY_SOURCE_ID_STATIC_MESH",
            ],
        );
    }
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
