use super::{
    ProfileCaptureConfig, ProfileSnapshot, RuntimeDiagnosticsSnapshot,
    RuntimeInputDiagnosticsSnapshot, RuntimeRenderDeviceDiagnosticsSnapshot, UiScenarioHotspot,
    PROFILE_CAPTURE_MAX_COUNTERS, PROFILE_CAPTURE_MAX_FRAMES, PROFILE_CAPTURE_MAX_FRAME_BUDGET_MS,
    PROFILE_CAPTURE_MAX_SPANS, PROFILE_DEFAULT_FRAME_BUDGET_MS, PROFILE_DEFAULT_MAX_COUNTERS,
    PROFILE_DEFAULT_MAX_FRAMES, PROFILE_DEFAULT_MAX_SPANS,
};

#[test]
fn profile_capture_config_normalization_enforces_hard_resource_limits() {
    let normalized = ProfileCaptureConfig {
        max_frames: usize::MAX,
        max_spans: usize::MAX,
        max_counters: usize::MAX,
        frame_budget_ms: f64::MAX,
        ..ProfileCaptureConfig::default()
    }
    .normalized();

    assert_eq!(normalized.max_frames, PROFILE_CAPTURE_MAX_FRAMES);
    assert_eq!(normalized.max_spans, PROFILE_CAPTURE_MAX_SPANS);
    assert_eq!(normalized.max_counters, PROFILE_CAPTURE_MAX_COUNTERS);
    assert_eq!(
        normalized.frame_budget_ms,
        PROFILE_CAPTURE_MAX_FRAME_BUDGET_MS
    );
}

#[test]
fn profile_capture_config_normalization_recovers_invalid_defaults() {
    for invalid_budget in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let normalized = ProfileCaptureConfig {
            max_frames: 0,
            max_spans: 0,
            max_counters: 0,
            frame_budget_ms: invalid_budget,
            ..ProfileCaptureConfig::default()
        }
        .normalized();

        assert_eq!(normalized.max_frames, PROFILE_DEFAULT_MAX_FRAMES);
        assert_eq!(normalized.max_spans, PROFILE_DEFAULT_MAX_SPANS);
        assert_eq!(normalized.max_counters, PROFILE_DEFAULT_MAX_COUNTERS);
        assert_eq!(normalized.frame_budget_ms, PROFILE_DEFAULT_FRAME_BUDGET_MS);
    }
}

#[test]
fn profile_snapshot_deserializes_pre_retention_payload() {
    let mut json =
        serde_json::to_value(ProfileSnapshot::default()).expect("serialize profile snapshot");
    json.as_object_mut()
        .expect("profile snapshot object")
        .remove("recorder_retention");

    let decoded: ProfileSnapshot =
        serde_json::from_value(json).expect("deserialize pre-retention snapshot");

    assert!(decoded.recorder_retention.is_empty());
}

#[test]
fn ui_scenario_hotspot_deserializes_pre_domain_counter_payload() {
    let mut json =
        serde_json::to_value(UiScenarioHotspot::empty("idle_hover")).expect("serialize hotspot");
    let hotspot = json.as_object_mut().expect("hotspot object");
    for field in [
        "host_invalidation_transaction_count",
        "host_invalidation_scope_count",
        "host_invalidation_legacy_dirty_transaction_count",
        "host_invalidation_full_target_count",
        "host_invalidation_shell_content_target_count",
        "host_invalidation_workbench_projection_target_count",
        "host_invalidation_view_presentation_target_count",
        "host_invalidation_window_metrics_target_count",
        "host_invalidation_paint_only_target_count",
        "presented_surface_pixels",
        "asset_editor_pane_presentation_build_count",
        "asset_editor_pane_reflection_build_count",
        "asset_editor_pane_preview_build_count",
        "asset_editor_pane_source_build_count",
        "asset_editor_pane_inspector_build_count",
        "asset_editor_pane_style_build_count",
        "asset_editor_pane_theme_build_count",
        "asset_editor_pane_command_availability_build_count",
        "workbench_hit_index_build_count",
        "workbench_hit_index_query_count",
        "pane_popup_index_query_count",
        "pane_popup_index_candidate_count",
        "visual_asset_targeted_invalidation_count",
        "svg_tree_targeted_invalidation_count",
        "visual_asset_reconcile_source_visit_count",
        "visual_asset_reconciled_invalidation_count",
        "svg_tree_reconcile_source_visit_count",
        "svg_tree_reconciled_invalidation_count",
        "visual_asset_full_invalidation_count",
        "visual_asset_cache_hit_count",
        "visual_asset_cache_miss_count",
        "visual_asset_cache_candidate_build_count",
        "svg_tree_cache_memory_hit_count",
        "svg_tree_cache_miss_count",
        "gpu_image_upload_write_count",
        "gpu_image_shared_resolve_count",
        "gpu_image_shared_upload_write_count",
        "gpu_image_shared_upload_bytes",
        "gpu_image_shared_resident_bytes",
        "gpu_image_cache_key_allocation_count",
        "gpu_image_cache_prune_visit_count",
        "gpu_image_cache_admission_reject_count",
        "gpu_image_invalid_payload_count",
        "gpu_image_cache_resident_bytes",
        "gpu_image_prepare_command_visit_count",
        "gpu_image_prepare_cache_hit_count",
        "gpu_timestamp_supported_present_count",
        "gpu_time_sample_count",
        "gpu_time_p50_us",
        "gpu_time_p95_us",
        "gpu_time_max_us",
        "gpu_profile_latency_max_frames",
        "gpu_compiled_draw_items",
        "gpu_batch_plan_build_count",
        "gpu_batch_plan_cache_hit_count",
        "gpu_vertex_buffer_create_count",
        "gpu_vertex_upload_bytes",
        "gpu_retained_cache_copy_bytes",
    ] {
        hotspot.remove(field);
    }

    let decoded: UiScenarioHotspot =
        serde_json::from_value(json).expect("deserialize pre-domain-counter hotspot");

    assert_eq!(decoded.scenario, "idle_hover");
    assert_eq!(decoded.host_invalidation_transaction_count, 0);
    assert_eq!(decoded.host_invalidation_scope_count, 0);
    assert_eq!(decoded.host_invalidation_legacy_dirty_transaction_count, 0);
    assert_eq!(decoded.host_invalidation_full_target_count, 0);
    assert_eq!(decoded.host_invalidation_shell_content_target_count, 0);
    assert_eq!(
        decoded.host_invalidation_workbench_projection_target_count,
        0
    );
    assert_eq!(decoded.host_invalidation_view_presentation_target_count, 0);
    assert_eq!(decoded.host_invalidation_window_metrics_target_count, 0);
    assert_eq!(decoded.host_invalidation_paint_only_target_count, 0);
    assert_eq!(decoded.presented_surface_pixels, 0);
    assert_eq!(decoded.asset_editor_pane_presentation_build_count, 0);
    assert_eq!(decoded.asset_editor_pane_reflection_build_count, 0);
    assert_eq!(decoded.asset_editor_pane_preview_build_count, 0);
    assert_eq!(decoded.asset_editor_pane_source_build_count, 0);
    assert_eq!(decoded.asset_editor_pane_inspector_build_count, 0);
    assert_eq!(decoded.asset_editor_pane_style_build_count, 0);
    assert_eq!(decoded.asset_editor_pane_theme_build_count, 0);
    assert_eq!(
        decoded.asset_editor_pane_command_availability_build_count,
        0
    );
    assert_eq!(decoded.workbench_hit_index_build_count, 0);
    assert_eq!(decoded.workbench_hit_index_query_count, 0);
    assert_eq!(decoded.pane_popup_index_query_count, 0);
    assert_eq!(decoded.pane_popup_index_candidate_count, 0);
    assert_eq!(decoded.visual_asset_targeted_invalidation_count, 0);
    assert_eq!(decoded.svg_tree_targeted_invalidation_count, 0);
    assert_eq!(decoded.visual_asset_reconcile_source_visit_count, 0);
    assert_eq!(decoded.visual_asset_reconciled_invalidation_count, 0);
    assert_eq!(decoded.svg_tree_reconcile_source_visit_count, 0);
    assert_eq!(decoded.svg_tree_reconciled_invalidation_count, 0);
    assert_eq!(decoded.visual_asset_full_invalidation_count, 0);
    assert_eq!(decoded.visual_asset_cache_hit_count, 0);
    assert_eq!(decoded.visual_asset_cache_miss_count, 0);
    assert_eq!(decoded.visual_asset_cache_candidate_build_count, 0);
    assert_eq!(decoded.svg_tree_cache_memory_hit_count, 0);
    assert_eq!(decoded.svg_tree_cache_miss_count, 0);
    assert_eq!(decoded.gpu_image_upload_write_count, 0);
    assert_eq!(decoded.gpu_image_shared_resolve_count, 0);
    assert_eq!(decoded.gpu_image_shared_upload_write_count, 0);
    assert_eq!(decoded.gpu_image_shared_upload_bytes, 0);
    assert_eq!(decoded.gpu_image_shared_resident_bytes, 0);
    assert_eq!(decoded.gpu_image_cache_key_allocation_count, 0);
    assert_eq!(decoded.gpu_image_cache_prune_visit_count, 0);
    assert_eq!(decoded.gpu_image_cache_admission_reject_count, 0);
    assert_eq!(decoded.gpu_image_invalid_payload_count, 0);
    assert_eq!(decoded.gpu_image_cache_resident_bytes, 0);
    assert_eq!(decoded.gpu_image_prepare_command_visit_count, 0);
    assert_eq!(decoded.gpu_image_prepare_cache_hit_count, 0);
    assert_eq!(decoded.gpu_timestamp_supported_present_count, 0);
    assert_eq!(decoded.gpu_time_sample_count, 0);
    assert_eq!(decoded.gpu_time_p50_us, 0);
    assert_eq!(decoded.gpu_time_p95_us, 0);
    assert_eq!(decoded.gpu_time_max_us, 0);
    assert_eq!(decoded.gpu_profile_latency_max_frames, 0);
    assert_eq!(decoded.gpu_compiled_draw_items, 0);
    assert_eq!(decoded.gpu_batch_plan_build_count, 0);
    assert_eq!(decoded.gpu_batch_plan_cache_hit_count, 0);
    assert_eq!(decoded.gpu_vertex_buffer_create_count, 0);
    assert_eq!(decoded.gpu_vertex_upload_bytes, 0);
    assert_eq!(decoded.gpu_retained_cache_copy_bytes, 0);
}

#[test]
fn runtime_diagnostics_snapshot_roundtrips_optional_product_identifiers() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        project_identity: Some("ZirconProject".to_string()),
        scene_uri: Some("res://scenes/main.scene.toml".to_string()),
        render_backend_name: Some("wgpu(dx12)".to_string()),
        ..RuntimeDiagnosticsSnapshot::default()
    };

    let json = serde_json::to_value(&snapshot).expect("serialize runtime diagnostics");
    let decoded: RuntimeDiagnosticsSnapshot =
        serde_json::from_value(json).expect("deserialize runtime diagnostics");

    assert_eq!(decoded.project_identity.as_deref(), Some("ZirconProject"));
    assert_eq!(
        decoded.scene_uri.as_deref(),
        Some("res://scenes/main.scene.toml")
    );
    assert_eq!(decoded.render_backend_name.as_deref(), Some("wgpu(dx12)"));
}

#[test]
fn runtime_diagnostics_snapshot_omits_missing_render_backend_name() {
    let json = serde_json::to_value(RuntimeDiagnosticsSnapshot::default())
        .expect("serialize runtime diagnostics");

    assert!(json.get("render_backend_name").is_none());
    assert!(json.get("project_identity").is_none());
    assert!(json.get("scene_uri").is_none());
    assert_eq!(json["input"]["viewport_resize_count"].as_u64(), Some(0));
    assert_eq!(json["input"]["pointer_move_count"].as_u64(), Some(0));
}

#[test]
fn runtime_diagnostics_snapshot_roundtrips_actual_render_device_evidence() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        render_device: Some(RuntimeRenderDeviceDiagnosticsSnapshot {
            adapter_name: "Zircon Test Adapter".to_owned(),
            adapter_device_type: "discrete_gpu".to_owned(),
            max_bind_groups: 5,
            max_texture_dimension_2d: 16_384,
            max_texture_array_layers: 256,
            max_sampled_textures_per_shader_stage: 16,
            max_binding_array_elements_per_shader_stage: 256,
            max_binding_array_sampler_elements_per_shader_stage: 128,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_buffer_binding_size: 134_217_728,
        }),
        ..RuntimeDiagnosticsSnapshot::default()
    };

    let json = serde_json::to_value(&snapshot).expect("serialize runtime diagnostics");
    let decoded: RuntimeDiagnosticsSnapshot =
        serde_json::from_value(json).expect("deserialize runtime diagnostics");
    let render_device = decoded.render_device.expect("render device evidence");

    assert_eq!(render_device.adapter_name, "Zircon Test Adapter");
    assert_eq!(render_device.adapter_device_type, "discrete_gpu");
    assert_eq!(render_device.max_bind_groups, 5);
    assert_eq!(render_device.max_texture_dimension_2d, 16_384);
    assert_eq!(render_device.max_texture_array_layers, 256);
    assert_eq!(render_device.max_sampled_textures_per_shader_stage, 16);
    assert_eq!(
        render_device.max_binding_array_elements_per_shader_stage,
        256
    );
    assert_eq!(
        render_device.max_binding_array_sampler_elements_per_shader_stage,
        128
    );
    assert_eq!(render_device.max_storage_buffers_per_shader_stage, 8);
    assert_eq!(render_device.max_storage_buffer_binding_size, 134_217_728);
}

#[test]
fn runtime_diagnostics_snapshot_deserializes_legacy_payload_without_render_device() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        render_device: Some(RuntimeRenderDeviceDiagnosticsSnapshot {
            adapter_name: "Zircon Test Adapter".to_owned(),
            adapter_device_type: "discrete_gpu".to_owned(),
            max_bind_groups: 5,
            max_texture_dimension_2d: 16_384,
            max_texture_array_layers: 256,
            max_sampled_textures_per_shader_stage: 16,
            max_binding_array_elements_per_shader_stage: 256,
            max_binding_array_sampler_elements_per_shader_stage: 128,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_buffer_binding_size: 134_217_728,
        }),
        ..RuntimeDiagnosticsSnapshot::default()
    };
    let mut json = serde_json::to_value(snapshot).expect("serialize runtime diagnostics");
    json.as_object_mut()
        .expect("runtime diagnostics object")
        .remove("render_device");

    let decoded: RuntimeDiagnosticsSnapshot =
        serde_json::from_value(json).expect("deserialize legacy runtime diagnostics");

    assert!(decoded.render_device.is_none());
}

#[test]
fn runtime_diagnostics_snapshot_deserializes_literal_pre_input_device_payload() {
    let legacy = r#"{
            "frame_index": 7,
            "diagnostic_series": [],
            "profile": {
                "session_id": "legacy",
                "output_root": "target/legacy",
                "active": false,
                "feature_enabled": false,
                "frame_budget_ms": 16.67,
                "frames": [],
                "spans": [],
                "counters": []
            }
        }"#;

    let decoded: RuntimeDiagnosticsSnapshot =
        serde_json::from_str(legacy).expect("deserialize literal legacy diagnostics");

    assert_eq!(decoded.frame_index, 7);
    assert!(decoded.project_identity.is_none());
    assert!(decoded.scene_uri.is_none());
    assert!(decoded.selected_model_resource_id.is_none());
    assert!(decoded.selected_material_resource_id.is_none());
    assert!(decoded.render_backend_name.is_none());
    assert_eq!(decoded.input, RuntimeInputDiagnosticsSnapshot::default());
    assert!(decoded.render_device.is_none());
}

#[test]
fn runtime_diagnostics_snapshot_deserializes_input_evidence_without_viewport_resize_count() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        input: RuntimeInputDiagnosticsSnapshot {
            viewport_resize_count: 9,
            pointer_move_count: 1,
            mouse_button_press_count: 2,
            mouse_button_release_count: 3,
            keyboard_press_count: 4,
            keyboard_release_count: 5,
        },
        ..RuntimeDiagnosticsSnapshot::default()
    };
    let mut json = serde_json::to_value(snapshot).expect("serialize runtime diagnostics");
    json["input"]
        .as_object_mut()
        .expect("runtime input diagnostics object")
        .remove("viewport_resize_count");

    let decoded: RuntimeDiagnosticsSnapshot =
        serde_json::from_value(json).expect("deserialize legacy input diagnostics");

    assert_eq!(decoded.input.viewport_resize_count, 0);
    assert_eq!(decoded.input.pointer_move_count, 1);
    assert_eq!(decoded.input.keyboard_release_count, 5);
}

#[test]
fn runtime_diagnostics_snapshot_roundtrips_product_input_evidence() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        input: RuntimeInputDiagnosticsSnapshot {
            viewport_resize_count: 6,
            pointer_move_count: 1,
            mouse_button_press_count: 2,
            mouse_button_release_count: 3,
            keyboard_press_count: 4,
            keyboard_release_count: 5,
        },
        ..RuntimeDiagnosticsSnapshot::default()
    };

    let json = serde_json::to_value(&snapshot).expect("serialize runtime diagnostics");
    let decoded: RuntimeDiagnosticsSnapshot =
        serde_json::from_value(json).expect("deserialize runtime diagnostics");

    assert_eq!(decoded.input, snapshot.input);
}
