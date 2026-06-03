mod support;

use crate::core::CoreRuntime;

use support::{
    assert_light_family_series, assert_render_bool_series, assert_render_count_series,
    assert_series_current, fake_render_module, DIAGNOSTICS_TEST_MODULE,
};

#[test]
fn runtime_diagnostics_reports_missing_runtime_contracts_without_panicking() {
    let runtime = CoreRuntime::new();

    let snapshot = crate::core::diagnostics::collect_runtime_diagnostics(&runtime.handle());

    assert!(!snapshot.render.available);
    assert!(snapshot.render.stats.is_none());
    assert!(snapshot.render.error.is_some());
    assert!(!snapshot.physics.available);
    assert!(snapshot.physics.backend_status.is_none());
    assert!(snapshot.physics.error.is_some());
    assert!(!snapshot.animation.available);
    assert!(snapshot.animation.playback_settings.is_none());
    assert!(snapshot.animation.error.is_some());
    assert!(snapshot.store.is_empty());
}

#[test]
fn runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins() {
    let runtime = CoreRuntime::new();
    runtime.register_module(fake_render_module()).unwrap();
    runtime.activate_module(DIAGNOSTICS_TEST_MODULE).unwrap();

    let snapshot = crate::core::diagnostics::collect_runtime_diagnostics(&runtime.handle());

    assert!(snapshot.render.available);
    let render_stats = snapshot.render.stats.expect("render stats");
    assert_eq!(render_stats.active_viewports, 2);
    assert_eq!(render_stats.submitted_frames, 7);
    assert_eq!(
        render_stats.capabilities.backend_name,
        "diagnostics-test-renderer"
    );
    assert!(!snapshot.render.virtual_geometry_debug_available);
    assert!(snapshot.render.error.is_none());

    assert!(!snapshot.physics.available);
    assert!(snapshot.physics.backend_status.is_none());
    assert!(snapshot.physics.error.is_some());

    assert!(!snapshot.animation.available);
    assert!(snapshot.animation.playback_settings.is_none());
    assert!(snapshot.animation.error.is_some());

    assert!(snapshot
        .store
        .series
        .iter()
        .any(|series| series.path.as_str() == "render.submitted_frames"
            && series.current == Some(7.0)));
    assert_render_count_series(
        &snapshot.store,
        "render.capability.queue_class_count",
        3.0,
        &["capability", "queue"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.surface_supported",
        true,
        &["capability", "surface"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.async_copy_supported",
        false,
        &["capability", "async_copy"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.storage_buffer_supported",
        true,
        &["capability", "storage_buffer"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.buffer_readback_supported",
        false,
        &["capability", "readback"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.inline_ray_query_supported",
        true,
        &["capability", "raytracing"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.texture_binding_array_supported",
        true,
        &["capability", "binding_array"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.partially_bound_binding_array_supported",
        false,
        &["capability", "binding_array"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.fxaa_supported",
        true,
        &["capability", "anti_alias"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.dlss_supported",
        false,
        &["capability", "anti_alias"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.neural_compute_supported",
        true,
        &["capability", "neural_compute"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.sparse_texture_supported",
        false,
        &["capability", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.capability.max_msaa_samples",
        8.0,
        &["capability", "anti_alias"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.virtual_geometry_supported",
        true,
        &["capability", "virtual_geometry"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capability.hybrid_gi_supported",
        true,
        &["capability", "hybrid_gi"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.current_handle_present",
        true,
        &["history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.previous_handle_present",
        true,
        &["history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.previous_available",
        false,
        &["history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated",
        true,
        &["history", "invalidation"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.target_width",
        1280.0,
        &["history", "target_size"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.target_height",
        720.0,
        &["history", "target_size"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.render_width",
        960.0,
        &["history", "render_size"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.history.render_height",
        540.0,
        &["history", "render_size"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated.no_previous_frame",
        false,
        &["history", "invalidation", "no_previous_frame"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated.frame_inputs_changed",
        false,
        &["history", "invalidation", "frame_inputs_changed"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated.render_size_changed",
        true,
        &["history", "invalidation", "render_size_changed"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.last_graph_executed_pass_count",
        14.0,
        &["graph"],
    );
    assert_render_count_series(&snapshot.store, "render.graph.pass_count", 18.0, &["graph"]);
    assert_render_count_series(
        &snapshot.store,
        "render.graph.culled_pass_count",
        4.0,
        &["graph", "culling"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.queue_fallback_pass_count",
        2.0,
        &["graph", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.resource_lifetime_count",
        6.0,
        &["graph", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.sparse_texture_lifetime_count",
        1.0,
        &["graph", "resource", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.planned_resource_access_count",
        22.0,
        &["graph", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.planned_dependency_count",
        9.0,
        &["graph", "dependency"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.transient_texture_slot_count",
        3.0,
        &["graph", "transient", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.sparse_texture_slot_count",
        1.0,
        &["graph", "transient", "texture", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.transient_buffer_slot_count",
        2.0,
        &["graph", "transient", "buffer"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_pass_count",
        14.0,
        &["graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_resource_access_count",
        19.0,
        &["graph", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_dependency_count",
        8.0,
        &["graph", "dependency"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_dispatch_count",
        2.0,
        &["graph", "compute", "dispatch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_dispatch_group_count",
        1234.0,
        &["graph", "compute", "dispatch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_storage_write_resource_count",
        2.0,
        &["graph", "compute", "storage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_planned_workload_count",
        2.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_matched_workload_count",
        1.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_missing_dispatch_count",
        1.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_workload_mismatch_count",
        0.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_unexpected_dispatch_count",
        0.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.debug_marker_count",
        14.0,
        &["graph", "debug_marker"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_anti_alias_pass_count",
        1.0,
        &["graph", "anti_alias"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_virtual_geometry_pass_count",
        2.0,
        &["graph", "virtual_geometry"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_hybrid_gi_pass_count",
        3.0,
        &["graph", "hybrid_gi"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_particle_pass_count",
        1.0,
        &["graph", "particle"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_transparent_pass_count",
        4.0,
        &["graph", "transparent"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_async_compute_pass_count",
        2.0,
        &["graph", "async_compute"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.requested_requires_history",
        true,
        &["anti_alias", "requested", "history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.effective_post_process",
        true,
        &["anti_alias", "effective"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.fallback.active",
        true,
        &["anti_alias", "fallback"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.fallback.missing_history",
        true,
        &["anti_alias", "fallback", "history"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.particle.gpu.alive_count",
        31.0,
        &["particle", "gpu"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.particle.gpu.indirect_instance_count",
        29.0,
        &["particle", "gpu", "indirect"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.graph.node_count",
        5.0,
        &["post_process", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.graph.skipped_node_count",
        1.0,
        &["post_process", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.graph.executed_node_count",
        3.0,
        &["post_process", "graph"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.graph.final_composite_present",
        true,
        &["post_process", "graph", "final_composite"],
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.enabled",
        1.0,
        "bool",
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.active_family_count",
        3.0,
        "count",
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.approximated_family_count",
        2.0,
        "count",
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.missing_resource_count",
        1.0,
        "count",
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.request_count",
        1.0,
        &["post_process", "lut"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.ready_count",
        0.0,
        &["post_process", "lut", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.fallback_count",
        1.0,
        &["post_process", "lut", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.texture_2d_strip_ready_count",
        0.0,
        &["post_process", "lut", "texture_2d_strip", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.texture_3d_request_count",
        1.0,
        &["post_process", "lut", "texture_3d"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.unsupported_shape_count",
        0.0,
        &["post_process", "lut", "unsupported_shape"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.count",
        13.0,
        &["material"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.ready_count",
        10.0,
        &["material"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.fallback_count",
        2.0,
        &["material", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.validation_error_count",
        1.0,
        &["material", "validation"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.diagnostic_count",
        4.0,
        &["material", "diagnostic"],
    );
    assert_light_family_series(&snapshot.store, "directional", 3.0, 1.0, 2.0);
    assert_light_family_series(&snapshot.store, "point", 4.0, 0.0, 4.0);
    assert_light_family_series(&snapshot.store, "spot", 5.0, 0.0, 5.0);
    assert_light_family_series(&snapshot.store, "ambient", 2.0, 2.0, 0.0);
    assert_light_family_series(&snapshot.store, "rect", 1.0, 0.0, 1.0);
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.draw_count",
        12.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.opaque_draw_count",
        6.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.alpha_mask_draw_count",
        2.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.transparent_draw_count",
        4.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.early_z_draw_count",
        8.0,
        &["mesh", "queue", "early_z"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.prepared_geometry_draw_count",
        5.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.dynamic_geometry_draw_count",
        7.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_draw_count",
        3.0,
        &["mesh", "queue", "indirect"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.static_batch_candidate_group_count",
        2.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.static_batch_candidate_draw_count",
        5.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.dynamic_batch_candidate_group_count",
        3.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.dynamic_batch_candidate_draw_count",
        6.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.gpu_instancing_candidate_group_count",
        4.0,
        &["mesh", "queue", "instancing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.gpu_instancing_candidate_draw_count",
        9.0,
        &["mesh", "queue", "instancing"],
    );
    assert_render_count_series(&snapshot.store, "render.sprite.count", 11.0, &["sprite"]);
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.ready_count",
        9.0,
        &["sprite"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.texture_fallback_count",
        2.0,
        &["sprite", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.graph_executed_pass_count",
        3.0,
        &["sprite", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.draw_batch_count",
        4.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.batched_sprite_count",
        10.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.vertex_count",
        60.0,
        &["sprite", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.opaque_draw_batch_count",
        1.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.alpha_mask_draw_batch_count",
        1.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.sprite.queue.transparent_draw_batch_count",
        2.0,
        &["sprite", "queue", "batch"],
    );
    assert_render_count_series(&snapshot.store, "render.ui.command_count", 17.0, &["ui"]);
    assert_render_count_series(&snapshot.store, "render.ui.quad_count", 8.0, &["ui"]);
    assert_render_count_series(
        &snapshot.store,
        "render.ui.text_payload_count",
        5.0,
        &["ui", "text"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.ui.image_payload_count",
        2.0,
        &["ui", "image"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.ui.clipped_command_count",
        3.0,
        &["ui", "clip"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.ui.graph_executed_pass_count",
        1.0,
        &["ui", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.virtual_geometry.cluster_budget",
        128.0,
        &["virtual_geometry", "budget"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.virtual_geometry.payload.source.authored",
        true,
        &["virtual_geometry", "payload", "source"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.virtual_geometry.debug.freeze_cull",
        true,
        &["virtual_geometry", "debug"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.virtual_geometry.resident_page_count",
        20.0,
        &["virtual_geometry", "page", "resident"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.virtual_geometry.execution_missing_segment_count",
        2.0,
        &["virtual_geometry", "execution", "missing"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.virtual_geometry.cluster_selection.input_source.prepare_on_demand",
        true,
        &["virtual_geometry", "cluster_selection", "source"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.virtual_geometry.node_and_cluster_cull.dispatch_group_z",
        5.0,
        &["virtual_geometry", "cull", "dispatch"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.virtual_geometry.visbuffer64.source.render_path_execution_selections",
        true,
        &["virtual_geometry", "visbuffer64", "source"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hybrid_gi.active_probe_count",
        5.0,
        &["hybrid_gi", "probe"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hybrid_gi.surface_cache.invalidated_page_count",
        15.0,
        &["hybrid_gi", "surface_cache", "invalidation"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hybrid_gi.voxel.invalidated_clipmap_count",
        18.0,
        &["hybrid_gi", "voxel", "invalidation"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hybrid_gi.payload.source.authored",
        true,
        &["hybrid_gi", "payload", "source"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.advanced_provider.availability.virtual_geometry_provider_present",
        true,
        &["advanced_provider", "availability", "virtual_geometry"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.advanced_provider.availability.hybrid_gi_provider_present",
        false,
        &["advanced_provider", "availability", "hybrid_gi"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.advanced_provider.report_count",
        2.0,
        &["advanced_provider"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.advanced_provider.enabled_count",
        1.0,
        &["advanced_provider", "enabled"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.advanced_provider.virtual_geometry.ready",
        true,
        &["advanced_provider", "virtual_geometry", "ready"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.advanced_provider.hybrid_gi.degraded",
        true,
        &["advanced_provider", "hybrid_gi", "degraded"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.advanced_provider.hybrid_gi.missing_provider_degradation_count",
        1.0,
        &["advanced_provider", "hybrid_gi", "degradation", "provider"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.solari.requested",
        true,
        &["solari", "requested"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.solari.enabled",
        false,
        &["solari", "enabled"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.solari.status.experimental_disabled",
        true,
        &["solari", "status", "experimental"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.solari.experimental_disabled_degradation_count",
        1.0,
        &["solari", "degradation", "experimental"],
    );

    let devtools = crate::core::diagnostics::collect_runtime_devtools_snapshot(&runtime.handle());
    assert!(devtools
        .modules
        .iter()
        .any(|module| module.name == DIAGNOSTICS_TEST_MODULE));
    assert!(devtools
        .services
        .iter()
        .any(|service| service.name == crate::core::manager::RENDER_FRAMEWORK_NAME));
    assert!(devtools
        .plugin_catalog
        .iter()
        .any(|plugin| plugin.package_id == "physics"));
    assert!(devtools
        .diagnostics_summary
        .tagged_subsystems
        .contains(&"render".to_string()));
}
