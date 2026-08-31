use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build() {
    let build_mod =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/mod.rs");
    let plan_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs",
    );
    let extract_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs",
    );
    let extract_command_slots_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/command_slots.rs",
    );
    let extract_item_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs",
    );
    let extract_rebuild_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs",
    );
    let extract_rebuild_batch_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/rebuild_batch.rs",
    );
    let extract_remainder_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/remainder.rs",
    );
    let extract_residual_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs",
    );
    let extract_second_frame_tests_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/second_frame_tests.rs",
    );
    let extract_lazy_tests_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/lazy_rebuild_tests.rs",
    );
    let extract_fallback_tests_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/fallback_tests.rs",
    );
    let extract_visibility_tests_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/visibility_tests.rs",
    );
    let extract_tests_owner = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/tests.rs",
    );
    let build_owner =
        read_runtime_src("graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs");
    let compiled_scene_draws = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs",
    );
    let render_owner = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs",
    );
    let prepared_queue = read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue.rs");
    let prepared_queue_stats_bridge =
        read_runtime_src("graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs");
    let render_stats = read_runtime_src("core/framework/render/backend_types.rs");
    let render_product_mesh_cache = read_runtime_src("graphics/tests/render_product_mesh_cache.rs");
    let mesh_queue_diagnostics =
        read_runtime_src("core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs");
    let product_diagnostics =
        read_runtime_src("core/runtime/diagnostics/render_stats_store/product.rs");
    let plan_02 = read_repo(
        "docs/plans/zircon_runtime/render/02/2026-07-09-mesh-draw-command-pipeline-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");
    let module_convention = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "build module mounts pending command cache plan owner",
        &build_mod,
        &[
            "mod pending_command_cache_plan;",
            "mod pending_command_cache_extract;",
            "pub(crate) use pending_command_cache_plan::PendingMeshCommandCachePlanStats;",
            "PendingMeshCommandCacheExtractionContext",
            "PendingMeshCommandCacheExtractionStats",
        ],
    );
    assert_contains_all(
        "pending command cache extract mounts bounded command slots owner",
        &extract_owner,
        &[
            "mod command_slots;",
            "use command_slots::PendingMeshCommandSlots;",
        ],
    );
    assert_contains_all(
        "pending command slots owner keeps the cacheable phase bound explicit",
        &extract_command_slots_owner,
        &[
            "const MAX_CACHEABLE_PHASE_COUNT: usize = 3;",
            "pub(super) struct PendingMeshCommandSlots<T>",
            "impl<T> IntoIterator for PendingMeshCommandSlots<T>",
        ],
    );
    assert_contains_all(
        "pending command cache plan owner keeps pre-MeshDraw static cache census",
        &plan_owner,
        &[
            "pub(crate) struct PendingMeshCommandCachePlanStats",
            "fn summarize_pending_mesh_command_cache_plan(",
            "fn summarize_pending_mesh_command_cache_plan_items(",
            "pending_mesh_draw_queue_profile(",
            "fn pending_command_cache_plan_counts_static_opaque_phase_candidates",
            "fn pending_command_cache_plan_keeps_identity_candidate_when_visibility_prunes_phases",
        ],
    );
    assert_contains_all(
        "pending command cache extraction owner skips full-hit static draws before MeshDraw build",
        &extract_owner,
        &[
            "pub(crate) struct PendingMeshCommandCacheExtractionStats",
            "pub(crate) struct PendingMeshCommandCacheExtractionContext",
            "fn extract_pending_static_mesh_command_cache_hits(",
            "fn cached_commands_for_extract_item(",
            "fn commands_for_extract_item(",
            "fn commands_for_extract_item_with_stats(",
            "residual_fallback::rebuild_non_material_command_or_record_residual",
            "rebuild_batch::pending_mesh_command_cache_rebuild_batch_for_phase(",
            "mod extract_item;",
            "mod fallback_tests;",
            "mod lazy_rebuild_tests;",
            "mod non_material_rebuild;",
            "mod rebuild_batch;",
            "mod remainder;",
            "mod residual_fallback;",
            "mod second_frame_tests;",
            "mod visibility_tests;",
            "visibility_pruned_mesh_draw_count",
            "residual_material_phase_draw_count",
        ],
    );
    assert!(
        !extract_owner.contains("Vec<Option<PendingMeshDraw>>"),
        "pending command cache extraction must move-partition residual draws instead of allocating an Option wrapper row per draw"
    );
    assert_contains_all(
        "pending command cache extraction lazily preallocates and move-partitions residual draws",
        &extract_owner,
        &[
            "let residual_capacity = pending_draws.len();",
            "let mut residual_pending_draws = None;",
            "for (source_draw_index, draw) in pending_draws.into_iter().enumerate()",
            "get_or_insert_with(|| Vec::with_capacity(residual_capacity))",
            ".push((source_draw_index, draw));",
            "PendingMeshDrawRemainder::Residual(residual_pending_draws.unwrap_or_default())",
        ],
    );
    assert!(
        !extract_owner.contains("let mut residual_pending_draws = Vec::new();"),
        "residual extraction must avoid allocating a residual container on full-hit frames while preallocating once on the first residual draw"
    );
    assert_contains_all(
        "pending command cache remainder owner avoids indexed repack allocation",
        &extract_remainder_owner,
        &[
            "enum PendingMeshDrawRemainder",
            "All(Vec<PendingMeshDraw>)",
            "Residual(Vec<(usize, PendingMeshDraw)>)",
            "draws.into_iter().enumerate()",
            "PendingMeshDrawRemainderIntoIter::Residual(draws.into_iter())",
        ],
    );
    assert_contains_all(
        "pending command cache rebuild batch owner keeps lazy MeshBatchRef construction",
        &extract_rebuild_batch_owner,
        &[
            "pub(super) fn pending_mesh_command_cache_rebuild_batch_for_phase(",
            "fn pending_mesh_command_cache_rebuild_batch(",
            "PendingMeshGeometry::Prepared(mesh)",
            "non_material_rebuild::can_rebuild_non_material_command_phase(phase)",
            "with_gpu_scene_instance_span(first_instance_index, instance_count)",
        ],
    );
    assert_contains_all(
        "pending command cache extract item owner keeps eligibility and phase selection",
        &extract_item_owner,
        &[
            "pub(super) struct PendingMeshCommandCacheExtractItem",
            "pub(super) fn pending_mesh_command_cache_extract_item(",
            "pub(super) fn can_skip_pending_mesh_draw_for_cached_commands(",
            "pub(super) fn cacheable_phases_for_extract_item(",
            "fn pending_mesh_draw_queue_profile(",
        ],
    );
    assert_contains_all(
        "pending command cache non-material rebuild owner only rebuilds opaque shadow",
        &(extract_rebuild_owner.clone() + &extract_tests_owner),
        &[
            "fn rebuild_non_material_command<R>(",
            "phase == RenderPhase::Shadow",
            "MeshDrawQueuePhase::Opaque => MeshPassPipelineKind::ShadowDepth",
            "MeshDrawQueuePhase::AlphaMask | MeshDrawQueuePhase::Transparent => return None",
            "fn rebuilds_opaque_shadow_command_without_material_handles",
            "fn alpha_mask_shadow_is_not_pre_mesh_draw_rebuildable",
            "fn depth_and_material_phases_are_not_pre_mesh_draw_rebuildable",
            "fn pending_command_cache_extracts_full_hit_without_rebuild_input",
            "fn pending_command_cache_extract_rebuilds_shadow_only_miss_before_mesh_draw",
        ],
    );
    assert_contains_all(
        "pending command cache residual fallback owner keeps fallback reason accounting",
        &extract_residual_owner,
        &[
            "enum PendingMeshCommandCacheResidualReason",
            "fn rebuild_non_material_command_or_record_residual<R>(",
            "fn rebuild_non_material_command<R>(",
            "fn record_residual_reason(",
            "PendingMeshCommandCacheResidualReason::MaterialPhase",
            "residual_material_phase_draw_count",
            "residual_rebuild_input_missing_draw_count",
            "residual_rebuild_rejected_draw_count",
        ],
    );
    assert_contains_all(
        "pending command cache second-frame tests keep extraction reuse and material invalidation guarded",
        &extract_second_frame_tests_owner,
        &[
            "fn pending_command_cache_extract_second_frame_full_hit_reports_zero_rebuilds",
            "fn pending_command_cache_extract_rebuilds_shadow_material_invalidation_before_mesh_draw",
            "cached_command_hit_count",
            "command_rebuild_count",
            "cache_invalidated_material_count",
        ],
    );
    assert_contains_all(
        "MD-M2 product coverage compares a static-heavy cold frame with a zero-rebuild warm frame",
        &render_product_mesh_cache,
        &[
            "const STATIC_CACHE_HEAVY_INSTANCE_COUNT: usize = 64;",
            "fn render_product_static_mesh_second_submit_reports_pre_mesh_command_cache_reuse()",
            "first.last_mesh_command_rebuild_count >= STATIC_CACHE_HEAVY_INSTANCE_COUNT",
            "second.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count",
            "second.last_mesh_command_rebuild_count, 0",
        ],
    );
    assert_contains_all(
        "pending command cache visibility tests keep zero-command skip diagnostics separate",
        &extract_visibility_tests_owner,
        &[
            "fn pending_command_cache_extract_marks_visibility_pruned_static_draw",
            "extracted.visibility_pruned",
            "rebuild_batch_requested = true",
        ],
    );
    assert_contains_all(
        "pending command cache fallback tests keep residual reasons observable",
        &extract_fallback_tests_owner,
        &[
            "fn pending_command_cache_extract_records_material_phase_residual_fallback",
            "fn pending_command_cache_extract_records_missing_rebuild_input_fallback",
            "fn pending_command_cache_extract_records_rebuild_rejected_fallback",
            "residual_material_phase_draw_count",
            "residual_rebuild_input_missing_draw_count",
            "residual_rebuild_rejected_draw_count",
        ],
    );
    assert_contains_all(
        "pending command cache lazy rebuild tests keep full-hit path from materializing batches",
        &extract_lazy_tests_owner,
        &[
            "fn pending_command_cache_extract_defers_rebuild_batch_on_full_hit",
            "fn pending_command_cache_extract_does_not_materialize_batch_for_material_phase_miss",
            "rebuild_batch_requested = true",
        ],
    );
    assert_contains_all(
        "build_mesh_draws records plan before MeshDraw construction",
        &build_owner,
        &[
            "summarize_pending_mesh_command_cache_plan(&pending_draws",
            "prepared_mesh_queue_stats_for_pending_draws(&pending_draws",
            "extract_pending_static_mesh_command_cache_hits(",
            "PendingMeshCommandCacheVisibility::from",
            "pending_command_cache_plan_stats",
            "pending_command_cache_extraction_stats",
            "pub(crate) fn pending_command_cache_plan_stats(&self)",
        ],
    );
    assert!(
        !build_owner.contains("pending_draws.into_iter().map(Some)"),
        "build_mesh_draws must pass pending draws directly into cache extraction without allocating an Option wrapper vector"
    );
    assert!(
        !build_owner.contains("pending_draws.into_iter().enumerate().filter_map"),
        "build_mesh_draws must not perform a second Option-filtering residual repack"
    );
    assert!(
        !build_owner.contains("pending_draws.into_iter().enumerate().collect"),
        "build_mesh_draws must not allocate an indexed tuple vector when command-cache extraction is disabled"
    );
    assert!(
        !build_owner.contains("let ordered_pending_draws"),
        "build_mesh_draws must construct final MeshDraw values directly instead of allocating an intermediate ordered tuple vector"
    );
    assert_contains_all(
        "build_mesh_draws consumes indexed residual draws directly",
        &build_owner,
        &[
            "PendingMeshDrawRemainder::all(pending_draws)",
            "let indexed_pending_draws = pending_draws",
            "draws: indexed_pending_draws",
            "let indirect_args_offset = indirect_args_offsets",
            "let submission_detail = pending_draw_submission_details",
        ],
    );
    assert!(
        build_owner.matches(".get(original_index)").count() >= 5,
        "residual draw offsets, buffers, submission details, tokens, and draw-ref indices must all use the preserved original pending index"
    );
    assert!(
        !build_owner.contains("self.prebuilt_mesh_pass_command_buffers.clone()"),
        "BuiltMeshDraws must move prebuilt command buffers into the compiled-scene owner instead of deep-cloning command vectors"
    );
    assert!(
        !compiled_scene_draws.contains("self.prebuilt_mesh_pass_command_buffers.clone()"),
        "CompiledSceneDraws must move prebuilt command buffers into frame execution instead of deep-cloning command vectors"
    );
    assert_contains_all(
        "prebuilt mesh command buffers transfer ownership across frame boundaries",
        &(build_owner.clone() + &compiled_scene_draws),
        &[
            "fn from_built_mesh_draws(mut built_mesh_draws: BuiltMeshDraws)",
            "pub(crate) fn prebuilt_mesh_pass_command_buffers(&mut self)",
            "pub(super) fn prebuilt_mesh_pass_command_buffers(&mut self)",
            "std::mem::take(&mut self.prebuilt_mesh_pass_command_buffers)",
        ],
    );
    for field in [
        "indirect_args_buffer",
        "indirect_submission_buffer",
        "indirect_authority_buffer",
        "indirect_draw_ref_buffer",
        "indirect_segment_buffer",
    ] {
        assert!(
            !build_owner.contains(&format!("self.{field}.clone()")),
            "BuiltMeshDraws must move `{field}` into the compiled-scene owner instead of incrementing the Arc count"
        );
        assert!(
            build_owner.contains(&format!("self.{field}.take()")),
            "BuiltMeshDraws must transfer `{field}` with Option::take"
        );
        assert!(
            compiled_scene_draws.contains(&format!("self.{field}.is_some()")),
            "CompiledSceneDraws virtual-geometry stats must inspect `{field}` without cloning its Arc"
        );
    }
    assert_contains_all(
        "compiled scene and prepared queue carry pending plan stats",
        &(compiled_scene_draws + &render_owner + &prepared_queue + &prepared_queue_stats_bridge),
        &[
            "PendingMeshCommandCachePlanStats",
            "PendingMeshCommandCacheExtractionStats",
            "pending_command_cache_plan_stats(&self)",
            "pending_command_cache_extraction_stats(",
            "prebuilt_mesh_pass_command_buffers",
            "with_pending_command_cache_extraction_stats",
            "with_pending_command_cache_plan_stats",
            "pending_static_command_cache_phase_candidate_count",
        ],
    );
    assert_contains_all(
        "render stats and product diagnostics expose pending static cache candidates",
        &(render_stats + &mesh_queue_diagnostics + &product_diagnostics),
        &[
            "last_mesh_pending_static_command_cache_draw_candidate_count",
            "last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count",
            "last_mesh_pre_mesh_draw_static_command_cache_visibility_pruned_draw_count",
            "last_mesh_pre_mesh_draw_static_command_cache_residual_material_phase_draw_count",
            "render.mesh.queue.pending_static_command_cache.phase_candidate_count",
            "render.mesh.queue.pending_static_command_cache.alpha_mask_candidate_count",
            "render.mesh.queue.pre_mesh_draw_static_command_cache.skipped_draw_count",
            "render.mesh.queue.pre_mesh_draw_static_command_cache.visibility_pruned_draw_count",
            "render.mesh.queue.pre_mesh_draw_static_command_cache.residual_material_phase_draw_count",
        ],
    );

    for (path, source, budget) in [
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs",
            plan_owner.as_str(),
            360,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs",
            extract_owner.as_str(),
            320,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/command_slots.rs",
            extract_command_slots_owner.as_str(),
            100,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs",
            extract_item_owner.as_str(),
            180,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs",
            extract_rebuild_owner.as_str(),
            180,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/rebuild_batch.rs",
            extract_rebuild_batch_owner.as_str(),
            120,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/remainder.rs",
            extract_remainder_owner.as_str(),
            100,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs",
            extract_residual_owner.as_str(),
            120,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/second_frame_tests.rs",
            extract_second_frame_tests_owner.as_str(),
            220,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/lazy_rebuild_tests.rs",
            extract_lazy_tests_owner.as_str(),
            160,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/fallback_tests.rs",
            extract_fallback_tests_owner.as_str(),
            180,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/visibility_tests.rs",
            extract_visibility_tests_owner.as_str(),
            120,
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/tests.rs",
            extract_tests_owner.as_str(),
            260,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue.rs",
            prepared_queue.as_str(),
            280,
        ),
        (
            "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs",
            prepared_queue_stats_bridge.as_str(),
            120,
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
                "MD-M2 pending command cache plan diagnostics",
                "MD-M2 pre-MeshDraw command cache extraction",
                "MD-M2 pre-MeshDraw opaque shadow cache rebuild",
                "MD-M2 lazy pre-MeshDraw rebuild input",
                "MD-M2 visibility-pruned pre-MeshDraw empty extraction",
                "MD-M2 visibility-pruned pre-MeshDraw diagnostics split",
                "MD-M2 pre-MeshDraw residual fallback diagnostics",
                "MD-M2 residual fallback owner split",
                "MD-M2 pre-MeshDraw second-frame extraction guards",
                "Plan 02 prepared queue stats bridge owner split",
                "MD-M2 pending command cache extract-item owner split",
                "render_plan02_pending_command_cache_plan_static_passed",
                "render_plan02_pre_mesh_draw_command_cache_extraction_static_passed",
                "render_plan02_pre_mesh_draw_shadow_cache_rebuild_static_passed",
                "render_plan02_lazy_pre_mesh_draw_rebuild_input_static_passed_cargo_lock_blocked",
                "render_plan02_visibility_pruned_pre_mesh_draw_empty_extract_static_passed_cargo_lock_blocked",
                "render_plan02_visibility_pruned_pre_mesh_draw_diagnostics_static_passed_cargo_timeout_no_result",
                "render_plan02_pre_mesh_draw_residual_fallback_diagnostics_static_passed",
                "render_plan02_residual_fallback_owner_split_static_passed_cargo_lock_blocked",
                "render_plan02_pre_mesh_draw_second_frame_extract_guards_static_passed_cargo_timeout_no_result",
                "render_plan02_prepared_queue_stats_bridge_owner_split_static_passed",
                "render_plan02_pending_command_cache_extract_item_owner_split_static_passed_cargo_lock_blocked",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_plan.rs",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/extract_item.rs",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/second_frame_tests.rs",
                "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/visibility_tests.rs",
                "graphics/scene/scene_renderer/mesh/prepared_queue/stats_bridge.rs",
                "runtime_15_pending_command_cache_plan_is_observable_before_mesh_draw_build",
            ],
        );
    }

    assert_contains_all(
        "pending command cache focused child-owner documentation",
        &format!(
            "{plan_02}\n{render_index}\n{review_findings}\n{structure_convention}\n{mesh_pass_doc}\n{module_convention}"
        ),
        &[
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/rebuild_batch.rs",
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/residual_fallback.rs",
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/lazy_rebuild_tests.rs",
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/fallback_tests.rs",
        ],
    );
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
