use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBuilder, RenderGraphError, RenderGraphExternalResourceBinding,
    RenderGraphResource, RenderGraphResourceUsageFlags,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

fn color_desc(name: &str) -> TextureDesc {
    TextureDesc::new(
        name,
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    )
}

#[test]
fn render_graph_culls_passes_unreachable_from_present_root() {
    let mut builder = RenderGraphBuilder::new("present-root-culling");
    let unused = builder.create_texture(color_desc("unused"));
    let color = builder.create_texture(color_desc("scene-color"));
    let output = builder.import_present_external_resource("viewport-output");

    let unused_pass = builder.add_pass("unused-pass", QueueLane::Graphics);
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);
    builder.write_texture(unused_pass, unused).unwrap();
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();
    builder.write_external(final_blit, output).unwrap();

    let graph = builder.compile().unwrap();
    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| (pass.name.as_str(), pass.culled))
            .collect::<Vec<_>>(),
        vec![
            ("unused-pass", true),
            ("opaque", false),
            ("final-blit", false),
        ]
    );
    assert_eq!(graph.stats().culled_pass_count, 1);
}

#[test]
fn access_allocation_table_excludes_culled_pass_resource_accesses() {
    let mut builder = RenderGraphBuilder::new("live-access-allocation-table");
    let unused = builder.create_texture(color_desc("unused"));
    let color = builder.create_texture(color_desc("scene-color"));
    let output = builder.import_present_external_resource("viewport-output");

    let unused_pass = builder.add_pass("unused-pass", QueueLane::Graphics);
    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);
    builder.write_texture(unused_pass, unused).unwrap();
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();
    builder.write_external(final_blit, output).unwrap();

    let graph = builder
        .compile()
        .expect("compile graph with culled producer");
    let unused_access = graph
        .access_id_for(
            unused_pass,
            RenderGraphResource::TransientTexture(unused),
            crate::render_graph::RenderGraphResourceAccessKind::Write,
        )
        .expect("compiled access index retains diagnostics for the culled pass");
    let opaque_access = graph
        .access_id_for(
            opaque,
            RenderGraphResource::TransientTexture(color),
            crate::render_graph::RenderGraphResourceAccessKind::Write,
        )
        .expect("opaque write access id");
    let final_read_access = graph
        .access_id_for(
            final_blit,
            RenderGraphResource::TransientTexture(color),
            crate::render_graph::RenderGraphResourceAccessKind::Read,
        )
        .expect("final read access id");
    let final_write_access = graph
        .access_id_for(
            final_blit,
            RenderGraphResource::External(output),
            crate::render_graph::RenderGraphResourceAccessKind::Write,
        )
        .expect("present write access id");

    assert!(graph.passes().iter().any(|pass| pass.culled));
    assert_eq!(graph.access_allocation_bindings().len(), 3);
    assert_eq!(
        graph
            .access_allocation_bindings()
            .iter()
            .map(|binding| binding.key.access_id)
            .collect::<Vec<_>>(),
        vec![opaque_access, final_read_access, final_write_access]
    );
    assert_eq!(graph.access_allocation_binding(unused_access), None);
}

#[test]
fn render_graph_non_root_external_write_is_culled() {
    let mut builder = RenderGraphBuilder::new("explicit-external-root");
    let color = builder.create_texture(color_desc("scene-color"));
    let output = builder.import_present_external_resource("viewport-output");
    let debug_output = builder.import_external_resource_with_usage(
        "debug-output",
        RenderGraphResourceUsageFlags::default(),
    );

    let opaque = builder.add_pass("opaque", QueueLane::Graphics);
    let final_blit = builder.add_pass("final-blit", QueueLane::Graphics);
    let debug = builder.add_pass("debug-export", QueueLane::Graphics);
    builder.write_texture(opaque, color).unwrap();
    builder.read_texture(final_blit, color).unwrap();
    builder.write_external(final_blit, output).unwrap();
    builder.write_external(debug, debug_output).unwrap();

    let graph = builder.compile().unwrap();
    assert!(
        graph
            .passes()
            .iter()
            .find(|pass| pass.name == "debug-export")
            .unwrap()
            .culled
    );
}

#[test]
fn render_graph_default_external_import_requires_an_explicit_cull_root_role() {
    let mut builder = RenderGraphBuilder::new("default-external-is-not-present");
    let output = builder.import_external_resource("diagnostic-output");
    let pass = builder.add_pass("diagnostic-export", QueueLane::Graphics);
    builder.write_external(pass, output).unwrap();

    let error = builder.compile().unwrap_err();
    assert!(matches!(
        error,
        RenderGraphError::MissingCullRoot { graph_name }
            if graph_name == "default-external-is-not-present"
    ));
}

#[test]
fn render_graph_default_typed_external_import_requires_an_explicit_cull_root_role() {
    let mut builder = RenderGraphBuilder::new("default-typed-external-is-not-present");
    let output = builder.import_external_resource_with_binding(
        "diagnostic-buffer-output",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = builder.add_pass("diagnostic-buffer-export", QueueLane::AsyncCompute);
    builder.write_external(pass, output).unwrap();

    let error = builder.compile().unwrap_err();
    assert!(matches!(
        error,
        RenderGraphError::MissingCullRoot { graph_name }
            if graph_name == "default-typed-external-is-not-present"
    ));
}

#[test]
fn render_graph_readback_marked_buffer_keeps_producer_alive() {
    let mut builder = RenderGraphBuilder::new("readback-root");
    let buffer = builder.create_buffer(BufferDesc::new(
        "occlusion-results",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC,
    ));
    builder
        .mark_readback(RenderGraphResource::TransientBuffer(buffer))
        .unwrap();

    let write = builder.add_pass("write-readback", QueueLane::AsyncCompute);
    builder.write_buffer(write, buffer).unwrap();

    let graph = builder.compile().unwrap();
    assert!(!graph.passes()[0].culled);
    assert!(graph
        .resource_lifetime_by_name("occlusion-results")
        .is_some());
}

#[test]
fn render_graph_persistent_texture_keeps_producer_alive() {
    let mut builder = RenderGraphBuilder::new("persistent-root");
    let history = builder.create_texture(color_desc("history.current.scene-color"));
    builder.mark_persistent(history).unwrap();

    let write = builder.add_pass("taa-resolve", QueueLane::Graphics);
    builder.write_texture(write, history).unwrap();

    let graph = builder.compile().unwrap();
    let lifetime = graph
        .resource_lifetime_by_name("history.current.scene-color")
        .unwrap();
    assert!(!graph.passes()[0].culled);
    assert!(lifetime.usage.persistent);
}

#[test]
fn render_graph_side_effect_pass_survives_culling() {
    let mut builder = RenderGraphBuilder::new("side-effect-root");
    let upload = builder.add_pass("timestamp-query", QueueLane::Graphics);
    builder
        .set_pass_flags(
            upload,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    assert_eq!(graph.stats().executable_pass_count, 1);
    assert!(!graph.passes()[0].culled);
}

#[test]
fn render_graph_missing_cull_root_is_compile_error() {
    let mut builder = RenderGraphBuilder::new("missing-root");
    let scratch = builder.create_texture(color_desc("scratch"));
    let pass = builder.add_pass("scratch-only", QueueLane::Graphics);
    builder.write_texture(pass, scratch).unwrap();

    let error = builder.compile().unwrap_err();
    assert!(matches!(
        error,
        RenderGraphError::MissingCullRoot { graph_name } if graph_name == "missing-root"
    ));
}

#[test]
fn compiled_graph_stats_are_materialized_during_graph_construction() {
    let graph_source = include_str!("../graph.rs");
    let stats_accessor = graph_source
        .split("    pub fn stats(&self) -> CompiledRenderGraphStats {")
        .nth(1)
        .expect("compiled graph exposes stats");

    assert!(graph_source.contains("    stats: CompiledRenderGraphStats,"));
    assert!(stats_accessor.starts_with("\n        self.stats\n    }"));
}

#[test]
fn compile_resource_hazards_split_execution_and_provenance_adjacency() {
    let inference_source = include_str!("../builder/resource_dependency_inference.rs");
    let tracker_source = include_str!("../builder/access_scope_tracker.rs");

    assert!(inference_source.contains("struct DependencyAdjacency"));
    assert!(inference_source.contains("membership: Vec<HashSet<RenderPassId>>"));
    assert!(inference_source.contains("let mut execution_dependencies"));
    assert!(inference_source.contains("let mut culling_dependencies"));
    assert!(tracker_source.contains("struct ResourceAccessHistory"));
    assert!(tracker_source.contains("readers_since_last_write"));
    assert!(tracker_source.contains("struct BufferScopeHistory"));
    assert!(!inference_source.contains("struct ManualPassReachability"));
}

#[test]
fn compile_lifetime_analysis_reuses_resource_declaration_metadata() {
    let compile_source = include_str!("../builder/compile.rs");

    assert!(compile_source.contains("resource_declarations: &[RenderGraphResourceDeclaration],"));
    assert!(!compile_source.contains("let resource_descs = self"));
    assert!(!compile_source.contains("let resource_usages = self"));
    assert!(!compile_source.contains("let external_bindings = self"));
}

#[test]
fn compile_lifetime_analysis_returns_typed_error_instead_of_panicking() {
    let compile_source = include_str!("../builder/compile.rs");
    let error_source = include_str!("../error.rs");

    assert!(
        compile_source.contains(") -> Result<Vec<RenderGraphResourceLifetime>, RenderGraphError>")
    );
    assert!(compile_source.contains("RenderGraphError::ResourceDeclarationMissing"));
    assert!(!compile_source.contains(".expect(\"resource accesses are validated"));
    assert!(error_source.contains("ResourceDeclarationMissing"));
}

#[test]
fn compute_packet_lowering_fails_closed_without_production_expect() {
    let binding_source = include_str!("../graph/compute_binding_access_packet.rs");
    let dispatch_source = include_str!("../graph/compute_dispatch_access_packet.rs");
    let error_source = include_str!("../error.rs");

    assert!(error_source.contains("CompiledAccessIndexEntryMissing"));
    assert!(binding_source.contains("PassComputeAccessLookup::new(pass, access_index)?"));
    assert!(!binding_source.contains(".expect("));
    assert!(!dispatch_source.contains(".expect("));
    assert!(!binding_source.contains("compiled access index cardinality was validated"));
    assert!(!binding_source.contains("compiled access index owns every pass access ID"));
    assert!(!binding_source.contains("lookup values are never empty"));
    assert!(!dispatch_source.contains("dispatch access candidates retain their declaration"));
}

#[test]
fn compile_resource_hazard_inference_tracks_writers_and_readers() {
    let inference_source = include_str!("../builder/resource_dependency_inference.rs");
    let tracker_source = include_str!("../builder/access_scope_tracker.rs");

    assert!(
        inference_source.contains("execution_dependencies.add_dependency(writer.pass, pass.id);")
    );
    assert!(inference_source.contains("culling_dependencies.add_dependency(writer.pass, pass.id);"));
    assert!(
        inference_source.contains("for reader in history.readers_since_last_write.iter().copied()")
    );
    assert!(tracker_source.contains("fn split_at(&mut self, boundary: u64, identity: usize)"));
    assert!(tracker_source.contains("PreparedScopeKind::Texture"));
    assert!(!inference_source.contains("validate_write_dependencies"));
    assert!(!inference_source.contains("writer_ids.windows(2)"));
}

#[test]
fn compile_cull_root_writer_lookup_does_not_clone_reader_histories() {
    let tracker_source = include_str!("../builder/access_scope_tracker.rs");
    let start = tracker_source
        .find("    pub(super) fn cull_root_writers_for(")
        .expect("scope-aware cull-root writer query");
    let end = tracker_source[start..]
        .find("    fn ensure_history(")
        .map(|offset| start + offset)
        .expect("history initialization follows cull-root query");
    let cull_root_query = &tracker_source[start..end];

    assert!(cull_root_query.contains("self.latest_writers_for_scope(&scope)"));
    assert!(cull_root_query.contains("fn latest_writers_for_scope("));
    assert!(!cull_root_query.contains("histories_for(&scope)"));
}

#[test]
fn compile_culling_does_not_allocate_a_write_list_per_pass() {
    let compile_source = include_str!("../builder/compile.rs");
    let culling_start = compile_source
        .find("    fn cull_passes(")
        .expect("render graph pass culling implementation");
    let culling_end = compile_source[culling_start..]
        .find("    fn resource_lifetimes(")
        .map(|offset| culling_start + offset)
        .expect("render graph lifetime analysis follows culling");
    let culling = &compile_source[culling_start..culling_end];

    assert!(!culling.contains(".collect::<Vec<_>>()"));
    assert!(culling.contains("dependencies[pass.0].iter().copied()"));
    assert!(!culling.contains("pass.dependencies.iter().copied()"));
}
