use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_graph_materialization_tests_are_child_owner_split() {
    let parent =
        read_runtime_src("graphics/scene/scene_renderer/graph_execution/materialization.rs");
    let tests =
        read_runtime_src("graphics/scene/scene_renderer/graph_execution/materialization/tests.rs");

    let plan_01 = read_repo(
        "docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let transient_materialization_docs = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/transient_materialization.md",
    );
    let execution_resources_docs = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.md",
    );

    assert_contains_all(
        "RenderGraph materialization parent keeps production owner and test mount",
        &parent,
        &[
            "pub(super) fn materialize_transient_resources(",
            "pub(super) fn create_wgpu_texture(",
            "pub(super) fn create_wgpu_buffer(",
            "fn wgpu_texture_usages(",
            "fn wgpu_buffer_usages(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_test in [
        "fn non_storage_texture_formats_do_not_request_storage_binding(",
        "fn storage_texture_formats_request_storage_binding(",
        "fn materialization_creates_dense_transients_and_skips_sparse_reservations(",
        "fn materialization_aliases_compatible_transient_texture_slots(",
        "fn materialization_receives_incompatible_texture_resources_in_separate_graph_slots(",
        "fn materialization_overrides_preimported_terminal_aa_input_with_owned_transient(",
        "fn materialization_preserves_imported_persistent_texture_without_pool_backing(",
        "fn materialization_aliases_transient_buffer_slots(",
        "fn materialization_exposes_owned_texture_mip_views(",
        "fn materialization_aliases_ssr_reflection_coarse_pyramid_to_parent_mip_view(",
        "fn materialization_allocates_ssr_reflection_coarse_resource_when_parent_has_no_coarse_mip(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "RenderGraph materialization parent should not own moved test `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "RenderGraph materialization test owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "RenderGraph materialization tests keep WGPU resource fixtures",
        &tests,
        &[
            "use super::*;",
            "RenderBackend::new_offscreen()",
            "storage_requested_usages_for(",
            "texture_alias_for(",
            "buffer_alias_for(",
        ],
    );

    for (path, source) in [
        ("graph_execution/materialization.rs", parent.as_str()),
        ("graph_execution/materialization/tests.rs", tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the materialization test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 01", &plan_01),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        (
            "transient materialization docs",
            &transient_materialization_docs,
        ),
        ("execution resources docs", &execution_resources_docs),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "RenderGraph materialization test owner split",
                "render_plan01_materialization_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/graph_execution/materialization.rs",
                "graphics/scene/scene_renderer/graph_execution/materialization/tests.rs",
                "runtime_15_render_graph_materialization_tests_are_child_owner_split",
            ],
        );
    }
}

#[test]
fn runtime_15_render_graph_materialization_requires_transient_pool() {
    let materialization =
        read_runtime_src("graphics/scene/scene_renderer/graph_execution/materialization.rs");
    let transient_materialization = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/transient_materialization.rs",
    );
    let transient_pool = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs",
    );
    let execution_resources = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs",
    );
    let resource_resolver = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs",
    );
    let resource_resolver_production = resource_resolver
        .split_once("#[cfg(test)]\nmod tests")
        .map(|(production, _)| production)
        .unwrap_or(resource_resolver.as_str());
    let compiled_graph = read_runtime_src("render_graph/graph.rs");
    let compiled_graph_builder = read_runtime_src("render_graph/builder/compile.rs");
    let transient_aliasing_tests =
        read_runtime_src("render_graph/tests/resources/transient_aliasing.rs");
    let full_chain_product =
        read_runtime_src("graphics/tests/render_product_post_process_full_chain.rs");
    let full_chain_export =
        read_runtime_src("graphics/tests/render_product_post_process_full_chain/visual_export.rs");
    let transient_materialization_docs = read_repo(
        "docs/zircon_runtime/graphics/scene/scene_renderer/graph_execution/transient_materialization.md",
    );

    for (label, source) in [
        ("materialization orchestration", materialization.as_str()),
        (
            "transient slot materialization",
            transient_materialization.as_str(),
        ),
    ] {
        assert!(
            !source.contains("Option<&mut TransientResourcePool>"),
            "{label} must require the production transient pool instead of retaining an unpooled allocation fallback"
        );
    }

    assert!(
        !execution_resources.contains("fn materialize_transient_resources(\n"),
        "RenderGraphExecutionResources must not expose a test-only unpooled materialization entry point"
    );
    assert_contains_all(
        "RenderGraph execution resources keep the mandatory pool entry point",
        &execution_resources,
        &[
            "fn materialize_transient_resources_with_pool(",
            "pool: &mut TransientResourcePool,",
            "super::materialization::materialize_transient_resources(self, device, graph, pool)",
        ],
    );
    assert!(
        !transient_materialization.contains("collect::<BTreeMap<_, _>>()"),
        "texture and buffer materialization must not rebuild duplicate per-frame lifetime indexes"
    );
    assert!(
        !materialization.contains("collect::<HashMap<_, _>>()")
            && !transient_materialization.contains("lifetime_by_name"),
        "materialization must use the compiled resource index instead of allocating a frame-local lifetime map"
    );
    assert!(
        !compiled_graph.contains(
            "pub fn transient_allocation_plan(&self) -> CompiledRenderGraphTransientAllocationPlan"
        ),
        "CompiledRenderGraph must not rerun transient interval coloring on every allocation-plan query"
    );
    assert_contains_all(
        "CompiledRenderGraph owns its transient allocation plan",
        &compiled_graph,
        &[
            "transient_allocation_plan: CompiledRenderGraphTransientAllocationPlan",
            "let transient_allocation_plan = build_transient_allocation_plan(&resource_lifetimes);",
            "pub fn transient_allocation_plan(&self) -> &CompiledRenderGraphTransientAllocationPlan",
            "&self.transient_allocation_plan",
            "resource_lifetime_indices: HashMap<RenderGraphResource, usize>",
            "resource_declaration_indices: HashMap<RenderGraphResource, usize>",
            "resource_declaration_indices_by_name: HashMap<String, usize>",
            "pass_indices: HashMap<RenderPassId, usize>",
            "pass_resource_access_indices:",
            "pub struct CompiledRenderGraphTransientAllocation {\n    pub resource: RenderGraphResource,",
            "self.resource_lifetime_indices",
            ".get(&resource)",
            "self.resource_declaration_indices",
            "self.resource_declaration_indices_by_name",
            "self.pass_indices",
            "self.pass_resource_access_indices",
            ".get(name)",
        ],
    );
    assert_contains_all(
        "persistent resources bypass pooling and readback lifetimes remain live through graph completion",
        &(compiled_graph.clone() + &compiled_graph_builder),
        &[
            "!lifetime.usage.persistent",
            "let last_pass = if usage.readback",
            "ordered.len().saturating_sub(1)",
        ],
    );
    assert_contains_all(
        "transient allocation tests guard persistent bypass and post-graph readback ownership",
        &transient_aliasing_tests,
        &[
            "fn graph_transient_allocation_plan_bypasses_persistent_textures()",
            "fn graph_readback_lifetimes_extend_to_graph_end_and_do_not_alias()",
            "plan.slot_for(\"history.current.scene-color\"), None",
            "plan.buffer_slot_count, 2",
        ],
    );
    assert!(
        !resource_resolver_production
            .contains(".passes()\n            .iter()\n            .find(")
            && !resource_resolver_production.contains("self.pass_resources().iter().find(")
            && !resource_resolver_production
                .contains("self.pass_resources()\n            .iter()\n            .any("),
        "pass-scoped resource resolution must use compiled pass/access indexes instead of repeated linear scans"
    );
    assert_contains_all(
        "pass-scoped resource resolution uses compiled graph indexes",
        &resource_resolver,
        &[
            "self.graph.pass(self.pass_id)",
            "self.graph\n            .pass_resource_access(self.pass_id, resource, access)",
        ],
    );
    assert_contains_all(
        "transient materialization resolves typed allocation resources through the compiled graph",
        &transient_materialization,
        &["graph.resource_lifetime(allocation.resource)"],
    );
    assert_contains_all(
        "RG-M2 full-chain product locks logical-to-physical texture reduction",
        &full_chain_product,
        &[
            "fn render_product_post_full_chain_all_effects_on()",
            "fn assert_transient_texture_pool_aliases_logical_resources(",
            "let logical_count = report.texture_logical_count();",
            "let physical_count = report.texture_backing_count();",
            "physical_count < logical_count",
            "assert_transient_texture_pool_aliases_logical_resources(&stats);",
        ],
    );
    assert_contains_all(
        "Render17 full-chain exporter keeps the same reduction contract and exact PNG output",
        &full_chain_export,
        &[
            "fn export_render17_pfm1_render_graph_cold_warm_wgpu_png()",
            "assert_transient_texture_pool_aliases_logical_resources(&second);",
            "plan17_pfm1_render_graph_cold_warm_wgpu_20260729.png",
        ],
    );
    assert_contains_all(
        "RG-M2 module docs keep pending PNG and DX12 RenderDoc evidence explicit",
        &transient_materialization_docs,
        &[
            "plan01_render_graph_transient_cache_full_chain_wgpu_20260718.png",
            "plan01_render_graph_transient_cache_full_chain_dx12_renderdoc_20260718_capture.rdc",
            "WGPU_BACKEND=dx12",
            "ZR_RENDERDOC_CAPTURE_NEXT=1",
        ],
    );
    assert!(
        !transient_pool.contains("while texture_pool_bytes(textures) > budget_bytes")
            && !transient_pool.contains("while buffer_pool_bytes(buffers) > budget_bytes"),
        "transient pool budget eviction must not rescan every retained entry for each eviction"
    );
    assert!(
        !transient_pool.contains("fn oldest_texture_entry(")
            && !transient_pool.contains("fn oldest_buffer_entry("),
        "transient pool budget eviction must not rescan the full pool to choose every next oldest entry"
    );
    assert_contains_all(
        "transient pool budget eviction orders candidates once and carries retained totals forward",
        &transient_pool,
        &[
            "fn evict_pool_to_budget<",
            "let mut candidates = pool",
            "candidates\n        .sort_unstable_by(",
            "let mut selected_indices = BTreeMap",
            "let (mut retained_count, mut retained_bytes)",
            "while retained_bytes > budget_bytes",
            "retained_count = retained_count.saturating_sub(1);",
            "selected_indices.entry(key).or_default().push(index);",
            "indices.sort_unstable_by(|left, right| right.cmp(left));",
            "(evicted, retained_count, retained_bytes)",
            "evict_pool_to_budget(textures, budget_bytes",
            "evict_pool_to_budget(buffers, budget_bytes",
            "let (budget_evicted_texture_count, texture_pool_entry_count, texture_pool_retained_bytes)",
            "let (budget_evicted_buffer_count, buffer_pool_entry_count, buffer_pool_retained_bytes)",
        ],
    );
    assert!(
        !transient_pool.contains("fn texture_entry_count(&self)")
            && !transient_pool.contains("fn buffer_entry_count(&self)"),
        "transient pool end-frame reports must carry retained counts from budget eviction instead of rescanning both pools"
    );
    for redundant_begin_frame_stat in [
        "texture_pool_entry_count: self.texture_entry_count(),",
        "buffer_pool_entry_count: self.buffer_entry_count(),",
        "texture_pool_retained_bytes: self.texture_pool_bytes(),",
        "buffer_pool_retained_bytes: self.buffer_pool_bytes(),",
    ] {
        assert!(
            !transient_pool.contains(redundant_begin_frame_stat),
            "TransientResourcePool::begin_frame must not scan a final statistic that end_frame overwrites: {redundant_begin_frame_stat}"
        );
    }
    assert_contains_all(
        "transient pool begin-frame reset stays constant-time",
        &transient_pool,
        &[
            "pub fn begin_frame(&mut self)",
            "frame_index: self.frame_index",
            "..Default::default()",
        ],
    );
}
