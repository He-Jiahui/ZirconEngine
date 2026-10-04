use zircon_runtime_interface::{
    RuntimeDiagnosticSeriesSnapshot, RuntimeDiagnosticsSnapshot, RuntimeInputDiagnosticsSnapshot,
    RuntimeRenderDeviceDiagnosticsSnapshot, ZrRuntimeViewportSizeV1,
};

use super::{
    product_frame_diagnostic, runtime_diagnostics_response_received,
    runtime_diagnostics_unavailable_error, validate_first_frame_product_snapshot,
    validate_first_frame_product_snapshot_with_input_probe, validate_mvp_input_probe_evidence,
};

#[test]
fn product_frame_diagnostic_reports_the_mvp_rendering_snapshot() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        frame_index: 17,
        project_identity: Some("ZirconProject".to_string()),
        scene_uri: Some("res://scenes/main.scene.toml".to_string()),
        selected_model_resource_id: Some("cube-model".to_string()),
        selected_material_resource_id: Some("cube-material".to_string()),
        render_backend_name: Some("wgpu(dx12)".to_string()),
        render_device: Some(RuntimeRenderDeviceDiagnosticsSnapshot {
            adapter_name: "Zircon Test Adapter".to_string(),
            adapter_device_type: "discrete_gpu".to_string(),
            max_bind_groups: 5,
            max_texture_dimension_2d: 16_384,
            max_texture_array_layers: 256,
            max_sampled_textures_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_buffer_binding_size: 134_217_728,
            max_binding_array_elements_per_shader_stage: 4_096,
            max_binding_array_sampler_elements_per_shader_stage: 1_024,
        }),
        diagnostic_series: vec![
            numeric_series("render.graph.executed_pass_count", 4.0),
            numeric_series("render.mesh.queue.draw_count", 2.0),
            numeric_series("render.light.directional.count", 1.0),
            numeric_series("render.material.fallback_count", 0.0),
            numeric_series("render.material.validation_error_count", 0.0),
        ],
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

    assert_eq!(
        product_frame_diagnostic(&snapshot, ZrRuntimeViewportSizeV1::new(1280, 720)),
        "runtime_product_frame_diagnostics frame_index=17 viewport=1280x720 project_identity=ZirconProject scene_uri=res://scenes/main.scene.toml selected_model_resource_id=cube-model selected_material_resource_id=cube-material render_backend=wgpu(dx12) render_adapter=Zircon Test Adapter render_adapter_type=discrete_gpu device_max_bind_groups=5 device_max_texture_dimension_2d=16384 device_max_texture_array_layers=256 device_max_sampled_textures_per_shader_stage=16 device_max_storage_buffers_per_shader_stage=8 device_max_storage_buffer_binding_size=134217728 graph_executed_pass_count=4 mesh_draw_count=2 directional_light_count=1 material_fallback_count=0 material_validation_error_count=0 input_viewport_resize_count=6 input_pointer_move_count=1 input_mouse_button_press_count=2 input_mouse_button_release_count=3 input_keyboard_press_count=4 input_keyboard_release_count=5"
    );
}

#[test]
fn product_frame_diagnostic_preserves_missing_metric_evidence() {
    assert_eq!(
        product_frame_diagnostic(
            &RuntimeDiagnosticsSnapshot::default(),
            ZrRuntimeViewportSizeV1::new(1, 1),
        ),
        "runtime_product_frame_diagnostics frame_index=0 viewport=1x1 project_identity=unavailable scene_uri=unavailable selected_model_resource_id=unavailable selected_material_resource_id=unavailable render_backend=unavailable render_adapter=unavailable render_adapter_type=unavailable device_max_bind_groups=unavailable device_max_texture_dimension_2d=unavailable device_max_texture_array_layers=unavailable device_max_sampled_textures_per_shader_stage=unavailable device_max_storage_buffers_per_shader_stage=unavailable device_max_storage_buffer_binding_size=unavailable graph_executed_pass_count=unavailable mesh_draw_count=unavailable directional_light_count=unavailable material_fallback_count=unavailable material_validation_error_count=unavailable input_viewport_resize_count=0 input_pointer_move_count=0 input_mouse_button_press_count=0 input_mouse_button_release_count=0 input_keyboard_press_count=0 input_keyboard_release_count=0"
    );
}

#[test]
fn missing_runtime_diagnostics_are_explicitly_actionable() {
    assert_eq!(
        runtime_diagnostics_unavailable_error("degraded", "snapshot is unavailable"),
        "runtime_product_frame_diagnostics_unavailable status=degraded message=snapshot is unavailable"
    );
}

#[test]
fn runtime_diagnostics_reject_non_ok_profile_control_responses() {
    assert_eq!(
        runtime_diagnostics_response_received("error", "runtime session unavailable")
            .unwrap_err(),
        "runtime_product_frame_diagnostics_unavailable status=error message=runtime session unavailable"
    );
    assert!(runtime_diagnostics_response_received("ok", "snapshot captured").is_ok());
}

#[test]
fn first_frame_product_snapshot_requires_a_visible_lit_mesh() {
    let missing_frame = RuntimeDiagnosticsSnapshot::default();
    assert_eq!(
        validate_first_frame_product_snapshot(&missing_frame).unwrap_err(),
        "runtime_product_frame_diagnostics_incomplete metric=render.graph.executed_pass_count expected=greater_than_zero observed=unavailable"
    );

    let material_failure = RuntimeDiagnosticsSnapshot {
        diagnostic_series: vec![
            numeric_series("render.graph.executed_pass_count", 1.0),
            numeric_series("render.mesh.queue.draw_count", 1.0),
            numeric_series("render.light.directional.count", 1.0),
            numeric_series("render.material.fallback_count", 0.0),
            numeric_series("render.material.validation_error_count", 1.0),
        ],
        ..RuntimeDiagnosticsSnapshot::default()
    };
    assert_eq!(
        validate_first_frame_product_snapshot(&material_failure).unwrap_err(),
        "runtime_product_frame_diagnostics_incomplete metric=render.material.validation_error_count expected=zero observed=1"
    );
}

#[test]
fn first_frame_product_snapshot_rejects_material_fallbacks() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        diagnostic_series: vec![
            numeric_series("render.graph.executed_pass_count", 1.0),
            numeric_series("render.mesh.queue.draw_count", 1.0),
            numeric_series("render.light.directional.count", 1.0),
            numeric_series("render.material.fallback_count", 1.0),
            numeric_series("render.material.validation_error_count", 0.0),
        ],
        ..RuntimeDiagnosticsSnapshot::default()
    };

    assert_eq!(
        validate_first_frame_product_snapshot(&snapshot).unwrap_err(),
        "runtime_product_frame_diagnostics_incomplete metric=render.material.fallback_count expected=zero observed=1"
    );
}

#[test]
fn first_frame_product_snapshot_accepts_a_visible_lit_mesh_without_material_errors() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        project_identity: Some("F2BasicScene".to_owned()),
        scene_uri: Some("res://scenes/main.scene.toml".to_owned()),
        selected_model_resource_id: Some("cube-model".to_owned()),
        selected_material_resource_id: Some("cube-material".to_owned()),
        render_backend_name: Some("wgpu(dx12)".to_owned()),
        render_device: Some(RuntimeRenderDeviceDiagnosticsSnapshot {
            adapter_name: "Zircon Test Adapter".to_owned(),
            adapter_device_type: "discrete_gpu".to_owned(),
            max_bind_groups: 5,
            max_texture_dimension_2d: 16_384,
            max_texture_array_layers: 256,
            max_sampled_textures_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 8,
            max_storage_buffer_binding_size: 134_217_728,
            max_binding_array_elements_per_shader_stage: 4_096,
            max_binding_array_sampler_elements_per_shader_stage: 1_024,
        }),
        diagnostic_series: vec![
            numeric_series("render.graph.executed_pass_count", 1.0),
            numeric_series("render.mesh.queue.draw_count", 1.0),
            numeric_series("render.light.directional.count", 1.0),
            numeric_series("render.material.fallback_count", 0.0),
            numeric_series("render.material.validation_error_count", 0.0),
        ],
        ..RuntimeDiagnosticsSnapshot::default()
    };

    assert!(validate_first_frame_product_snapshot_with_input_probe(&snapshot, false).is_ok());
}

#[test]
fn first_frame_product_snapshot_requires_project_scene_and_backend_identity() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        diagnostic_series: vec![
            numeric_series("render.graph.executed_pass_count", 1.0),
            numeric_series("render.mesh.queue.draw_count", 1.0),
            numeric_series("render.light.directional.count", 1.0),
            numeric_series("render.material.fallback_count", 0.0),
            numeric_series("render.material.validation_error_count", 0.0),
        ],
        ..RuntimeDiagnosticsSnapshot::default()
    };

    assert_eq!(
        validate_first_frame_product_snapshot(&snapshot).unwrap_err(),
        "runtime_product_frame_diagnostics_incomplete field=project_identity expected=nonempty observed=unavailable"
    );
}

#[test]
fn mvp_input_probe_evidence_requires_every_requested_input_class() {
    let snapshot = RuntimeDiagnosticsSnapshot {
        project_identity: Some("F2BasicScene".to_owned()),
        scene_uri: Some("res://scenes/main.scene.toml".to_owned()),
        render_backend_name: Some("wgpu(dx12)".to_owned()),
        diagnostic_series: vec![
            numeric_series("render.graph.executed_pass_count", 1.0),
            numeric_series("render.mesh.queue.draw_count", 1.0),
            numeric_series("render.light.directional.count", 1.0),
            numeric_series("render.material.validation_error_count", 0.0),
        ],
        ..RuntimeDiagnosticsSnapshot::default()
    };

    assert_eq!(
        validate_mvp_input_probe_evidence(&snapshot, true).unwrap_err(),
        "runtime_product_frame_diagnostics_incomplete metric=input.viewport_resize_count expected=greater_than_zero observed=0"
    );
}

fn numeric_series(path: &str, current: f64) -> RuntimeDiagnosticSeriesSnapshot {
    RuntimeDiagnosticSeriesSnapshot {
        path: path.to_string(),
        current: Some(current),
        ..RuntimeDiagnosticSeriesSnapshot::default()
    }
}
