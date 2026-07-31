use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, RuntimeDiagnosticsSnapshot,
    ZrRuntimeViewportSizeV1,
};

use super::mvp_input_probe::mvp_input_probe_enabled;
use super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn emit_first_frame_product_diagnostics(
        &self,
    ) -> Result<(), String> {
        let request = ProfileControlRequest {
            command: ProfileControlCommand::RuntimeDiagnosticsSnapshot,
            config: None,
        };
        match self.session.profile_control(&request) {
            Ok(Some(response)) => {
                runtime_diagnostics_response_received(&response.status, &response.message)?;
                let Some(snapshot) = response.runtime_diagnostics else {
                    return Err(runtime_diagnostics_unavailable_error(
                        &response.status,
                        &response.message,
                    ));
                };
                validate_first_frame_product_snapshot(&snapshot)?;
                write_log(
                    "runtime_surface_present",
                    product_frame_diagnostic(&snapshot, self.viewport_size),
                );
                Ok(())
            }
            Ok(None) => Err(
                "runtime_product_frame_diagnostics_unavailable reason=profile_control_not_supported"
                    .to_owned(),
            ),
            Err(error) => Err(format!("runtime_product_frame_diagnostics_failed error={error}")),
        }
    }
}

fn runtime_diagnostics_unavailable_error(status: &str, message: &str) -> String {
    format!("runtime_product_frame_diagnostics_unavailable status={status} message={message}")
}

fn runtime_diagnostics_response_received(status: &str, message: &str) -> Result<(), String> {
    if status == "ok" {
        Ok(())
    } else {
        Err(runtime_diagnostics_unavailable_error(status, message))
    }
}

fn validate_first_frame_product_snapshot(
    snapshot: &RuntimeDiagnosticsSnapshot,
) -> Result<(), String> {
    validate_first_frame_product_snapshot_with_input_probe(snapshot, mvp_input_probe_enabled())
}

fn validate_first_frame_product_snapshot_with_input_probe(
    snapshot: &RuntimeDiagnosticsSnapshot,
    input_probe_enabled: bool,
) -> Result<(), String> {
    require_positive_metric(snapshot, "render.graph.executed_pass_count")?;
    require_positive_metric(snapshot, "render.mesh.queue.draw_count")?;
    require_positive_metric(snapshot, "render.light.directional.count")?;
    require_zero_metric(snapshot, "render.material.fallback_count")?;
    require_zero_metric(snapshot, "render.material.validation_error_count")?;
    require_nonempty_field("project_identity", snapshot.project_identity.as_deref())?;
    require_nonempty_field("scene_uri", snapshot.scene_uri.as_deref())?;
    require_nonempty_field(
        "selected_model_resource_id",
        snapshot.selected_model_resource_id.as_deref(),
    )?;
    require_nonempty_field(
        "selected_material_resource_id",
        snapshot.selected_material_resource_id.as_deref(),
    )?;
    require_nonempty_field("render_backend", snapshot.render_backend_name.as_deref())?;
    validate_render_device_diagnostics(snapshot)?;
    validate_mvp_input_probe_evidence(snapshot, input_probe_enabled)
}

fn validate_render_device_diagnostics(snapshot: &RuntimeDiagnosticsSnapshot) -> Result<(), String> {
    let Some(device) = snapshot.render_device.as_ref() else {
        return Err(incomplete_field_error("render_device", "unavailable"));
    };
    require_nonempty_field("render_adapter", Some(&device.adapter_name))?;
    require_nonempty_field("render_adapter_type", Some(&device.adapter_device_type))?;
    for (field, value) in [
        ("device.max_bind_groups", u64::from(device.max_bind_groups)),
        (
            "device.max_texture_dimension_2d",
            u64::from(device.max_texture_dimension_2d),
        ),
        (
            "device.max_texture_array_layers",
            u64::from(device.max_texture_array_layers),
        ),
        (
            "device.max_sampled_textures_per_shader_stage",
            u64::from(device.max_sampled_textures_per_shader_stage),
        ),
        (
            "device.max_storage_buffers_per_shader_stage",
            u64::from(device.max_storage_buffers_per_shader_stage),
        ),
        (
            "device.max_storage_buffer_binding_size",
            device.max_storage_buffer_binding_size,
        ),
    ] {
        if value == 0 {
            return Err(format!(
                "runtime_product_frame_diagnostics_incomplete field={field} expected=greater_than_zero observed=0"
            ));
        }
    }
    Ok(())
}

fn validate_mvp_input_probe_evidence(
    snapshot: &RuntimeDiagnosticsSnapshot,
    input_probe_enabled: bool,
) -> Result<(), String> {
    if !input_probe_enabled {
        return Ok(());
    }

    for (field, value) in [
        (
            "input.pointer_move_count",
            snapshot.input.pointer_move_count,
        ),
        (
            "input.mouse_button_press_count",
            snapshot.input.mouse_button_press_count,
        ),
        (
            "input.mouse_button_release_count",
            snapshot.input.mouse_button_release_count,
        ),
        (
            "input.keyboard_press_count",
            snapshot.input.keyboard_press_count,
        ),
        (
            "input.keyboard_release_count",
            snapshot.input.keyboard_release_count,
        ),
    ] {
        if value == 0 {
            return Err(format!(
                "runtime_product_frame_diagnostics_incomplete metric={field} expected=greater_than_zero observed=0"
            ));
        }
    }
    Ok(())
}

fn require_positive_metric(
    snapshot: &RuntimeDiagnosticsSnapshot,
    path: &str,
) -> Result<(), String> {
    match metric_value(snapshot, path) {
        Some(value) if value > 0.0 => Ok(()),
        value => Err(incomplete_metric_error(path, "greater_than_zero", value)),
    }
}

fn require_zero_metric(snapshot: &RuntimeDiagnosticsSnapshot, path: &str) -> Result<(), String> {
    match metric_value(snapshot, path) {
        Some(0.0) => Ok(()),
        value => Err(incomplete_metric_error(path, "zero", value)),
    }
}

fn incomplete_metric_error(path: &str, expected: &str, value: Option<f64>) -> String {
    format!(
        "runtime_product_frame_diagnostics_incomplete metric={path} expected={expected} observed={}",
        value
            .map(|value| format!("{value:.0}"))
            .unwrap_or_else(|| "unavailable".to_owned())
    )
}

fn require_nonempty_field(field: &str, value: Option<&str>) -> Result<(), String> {
    if value.is_some_and(|value| !value.trim().is_empty()) {
        Ok(())
    } else {
        Err(incomplete_field_error(field, "unavailable"))
    }
}

fn incomplete_field_error(field: &str, observed: &str) -> String {
    format!(
        "runtime_product_frame_diagnostics_incomplete field={field} expected=nonempty observed={observed}"
    )
}

fn product_frame_diagnostic(
    snapshot: &RuntimeDiagnosticsSnapshot,
    viewport: ZrRuntimeViewportSizeV1,
) -> String {
    let render_device = snapshot.render_device.as_ref();
    format!(
        "runtime_product_frame_diagnostics frame_index={} viewport={}x{} project_identity={} scene_uri={} selected_model_resource_id={} selected_material_resource_id={} render_backend={} render_adapter={} render_adapter_type={} device_max_bind_groups={} device_max_texture_dimension_2d={} device_max_texture_array_layers={} device_max_sampled_textures_per_shader_stage={} device_max_storage_buffers_per_shader_stage={} device_max_storage_buffer_binding_size={} graph_executed_pass_count={} mesh_draw_count={} directional_light_count={} material_fallback_count={} material_validation_error_count={} input_pointer_move_count={} input_mouse_button_press_count={} input_mouse_button_release_count={} input_keyboard_press_count={} input_keyboard_release_count={}",
        snapshot.frame_index,
        viewport.width,
        viewport.height,
        product_value(snapshot.project_identity.as_deref()),
        product_value(snapshot.scene_uri.as_deref()),
        product_value(snapshot.selected_model_resource_id.as_deref()),
        product_value(snapshot.selected_material_resource_id.as_deref()),
        render_backend_name(snapshot),
        product_value(render_device.map(|device| device.adapter_name.as_str())),
        product_value(render_device.map(|device| device.adapter_device_type.as_str())),
        device_limit_value(render_device.map(|device| u64::from(device.max_bind_groups))),
        device_limit_value(render_device.map(|device| u64::from(device.max_texture_dimension_2d))),
        device_limit_value(render_device.map(|device| u64::from(device.max_texture_array_layers))),
        device_limit_value(
            render_device.map(|device| u64::from(device.max_sampled_textures_per_shader_stage))
        ),
        device_limit_value(
            render_device.map(|device| u64::from(device.max_storage_buffers_per_shader_stage))
        ),
        device_limit_value(render_device.map(|device| device.max_storage_buffer_binding_size)),
        metric_count(snapshot, "render.graph.executed_pass_count"),
        metric_count(snapshot, "render.mesh.queue.draw_count"),
        metric_count(snapshot, "render.light.directional.count"),
        metric_count(snapshot, "render.material.fallback_count"),
        metric_count(snapshot, "render.material.validation_error_count"),
        snapshot.input.pointer_move_count,
        snapshot.input.mouse_button_press_count,
        snapshot.input.mouse_button_release_count,
        snapshot.input.keyboard_press_count,
        snapshot.input.keyboard_release_count,
    )
}

fn render_backend_name(snapshot: &RuntimeDiagnosticsSnapshot) -> &str {
    product_value(snapshot.render_backend_name.as_deref())
}

fn product_value(value: Option<&str>) -> &str {
    value
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("unavailable")
}

fn device_limit_value(value: Option<u64>) -> String {
    value
        .filter(|value| *value > 0)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn metric_count(snapshot: &RuntimeDiagnosticsSnapshot, path: &str) -> String {
    metric_value(snapshot, path)
        .map(|value| format!("{value:.0}"))
        .unwrap_or_else(|| "unavailable".to_string())
}

fn metric_value(snapshot: &RuntimeDiagnosticsSnapshot, path: &str) -> Option<f64> {
    snapshot
        .series(path)
        .and_then(|series| series.current)
        .filter(|value| value.is_finite())
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{
        RuntimeDiagnosticSeriesSnapshot, RuntimeDiagnosticsSnapshot,
        RuntimeInputDiagnosticsSnapshot, RuntimeRenderDeviceDiagnosticsSnapshot,
        ZrRuntimeViewportSizeV1,
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
            }),
            diagnostic_series: vec![
                numeric_series("render.graph.executed_pass_count", 4.0),
                numeric_series("render.mesh.queue.draw_count", 2.0),
                numeric_series("render.light.directional.count", 1.0),
                numeric_series("render.material.fallback_count", 0.0),
                numeric_series("render.material.validation_error_count", 0.0),
            ],
            input: RuntimeInputDiagnosticsSnapshot {
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
            "runtime_product_frame_diagnostics frame_index=17 viewport=1280x720 project_identity=ZirconProject scene_uri=res://scenes/main.scene.toml selected_model_resource_id=cube-model selected_material_resource_id=cube-material render_backend=wgpu(dx12) render_adapter=Zircon Test Adapter render_adapter_type=discrete_gpu device_max_bind_groups=5 device_max_texture_dimension_2d=16384 device_max_texture_array_layers=256 device_max_sampled_textures_per_shader_stage=16 device_max_storage_buffers_per_shader_stage=8 device_max_storage_buffer_binding_size=134217728 graph_executed_pass_count=4 mesh_draw_count=2 directional_light_count=1 material_fallback_count=0 material_validation_error_count=0 input_pointer_move_count=1 input_mouse_button_press_count=2 input_mouse_button_release_count=3 input_keyboard_press_count=4 input_keyboard_release_count=5"
        );
    }

    #[test]
    fn product_frame_diagnostic_preserves_missing_metric_evidence() {
        assert_eq!(
            product_frame_diagnostic(
                &RuntimeDiagnosticsSnapshot::default(),
                ZrRuntimeViewportSizeV1::new(1, 1),
            ),
            "runtime_product_frame_diagnostics frame_index=0 viewport=1x1 project_identity=unavailable scene_uri=unavailable selected_model_resource_id=unavailable selected_material_resource_id=unavailable render_backend=unavailable render_adapter=unavailable render_adapter_type=unavailable device_max_bind_groups=unavailable device_max_texture_dimension_2d=unavailable device_max_texture_array_layers=unavailable device_max_sampled_textures_per_shader_stage=unavailable device_max_storage_buffers_per_shader_stage=unavailable device_max_storage_buffer_binding_size=unavailable graph_executed_pass_count=unavailable mesh_draw_count=unavailable directional_light_count=unavailable material_fallback_count=unavailable material_validation_error_count=unavailable input_pointer_move_count=0 input_mouse_button_press_count=0 input_mouse_button_release_count=0 input_keyboard_press_count=0 input_keyboard_release_count=0"
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
            "runtime_product_frame_diagnostics_incomplete metric=input.pointer_move_count expected=greater_than_zero observed=0"
        );
    }

    fn numeric_series(path: &str, current: f64) -> RuntimeDiagnosticSeriesSnapshot {
        RuntimeDiagnosticSeriesSnapshot {
            path: path.to_string(),
            current: Some(current),
            ..RuntimeDiagnosticSeriesSnapshot::default()
        }
    }
}
