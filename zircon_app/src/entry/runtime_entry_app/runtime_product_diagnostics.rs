//! 持久场景产品验收的首个成功呈现帧诊断。
//! 以明确指标与资源身份确认验收条件；该 lit-mesh 门槛的适用产品由调用配置决定。

use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, RuntimeDiagnosticsSnapshot,
    ZrRuntimeViewportSizeV1,
};

use super::mvp_input_probe::mvp_input_probe_enabled;
use super::RuntimeEntryApp;

impl RuntimeEntryApp {
    /// 在成功呈现后读取并验证产品诊断；调用者负责一次性状态和失败退出。
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

// TODO: [CR-APP-ENTRY-0017] 确认通用持久项目是否都应满足 F2 的受光 mesh/resource 指标；当前入口按 project_root 启用本检查，需在其外来改动稳定后验证空场景、纯 UI 与无方向光项目的启动策略。
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
            "input.viewport_resize_count",
            snapshot.input.viewport_resize_count,
        ),
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

// 输出字段供产品验收工具读取；缺失证据显式保留，不能用默认零伪造通过。
fn product_frame_diagnostic(
    snapshot: &RuntimeDiagnosticsSnapshot,
    viewport: ZrRuntimeViewportSizeV1,
) -> String {
    let render_device = snapshot.render_device.as_ref();
    format!(
        "runtime_product_frame_diagnostics frame_index={} viewport={}x{} project_identity={} scene_uri={} selected_model_resource_id={} selected_material_resource_id={} render_backend={} render_adapter={} render_adapter_type={} device_max_bind_groups={} device_max_texture_dimension_2d={} device_max_texture_array_layers={} device_max_sampled_textures_per_shader_stage={} device_max_storage_buffers_per_shader_stage={} device_max_storage_buffer_binding_size={} graph_executed_pass_count={} mesh_draw_count={} directional_light_count={} material_fallback_count={} material_validation_error_count={} input_viewport_resize_count={} input_pointer_move_count={} input_mouse_button_press_count={} input_mouse_button_release_count={} input_keyboard_press_count={} input_keyboard_release_count={}",
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
        snapshot.input.viewport_resize_count,
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
#[path = "tests/runtime_product_diagnostics.rs"]
mod tests;
