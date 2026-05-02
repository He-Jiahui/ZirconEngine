use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventKind};

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::{
    event_bridge::SlintDispatchEffects, viewport_toolbar_pointer::ViewportToolbarPointerRoute,
};

use super::super::BuiltinViewportToolbarTemplateBridge;
use super::dispatch_builtin_viewport_toolbar_control;
use super::snap_cycle::{
    next_display_mode_name, next_grid_mode_name, next_rotate_snap_degrees, next_scale_snap,
    next_translate_snap,
};

pub(crate) fn dispatch_viewport_toolbar_pointer_route(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinViewportToolbarTemplateBridge,
    route: &ViewportToolbarPointerRoute,
) -> Result<SlintDispatchEffects, String> {
    let settings = runtime.chrome_snapshot().scene_viewport_settings;
    let (control_id, event_kind, arguments) = match route {
        ViewportToolbarPointerRoute::SetTool { tool, .. } => (
            "SetTool",
            UiEventKind::Change,
            vec![UiBindingValue::string(tool)],
        ),
        ViewportToolbarPointerRoute::SetTransformSpace { space, .. } => (
            "SetTransformSpace",
            UiEventKind::Change,
            vec![UiBindingValue::string(space)],
        ),
        ViewportToolbarPointerRoute::SetProjectionMode { mode, .. } => (
            "SetProjectionMode",
            UiEventKind::Change,
            vec![UiBindingValue::string(mode)],
        ),
        ViewportToolbarPointerRoute::AlignView { orientation, .. } => (
            "AlignView",
            UiEventKind::Change,
            vec![UiBindingValue::string(orientation)],
        ),
        ViewportToolbarPointerRoute::CycleDisplayMode { .. } => (
            "SetDisplayMode",
            UiEventKind::Change,
            vec![UiBindingValue::string(next_display_mode_name(
                settings.display_mode,
            ))],
        ),
        ViewportToolbarPointerRoute::CycleGridMode { .. } => (
            "SetGridMode",
            UiEventKind::Change,
            vec![UiBindingValue::string(next_grid_mode_name(
                settings.grid_mode,
            ))],
        ),
        ViewportToolbarPointerRoute::CycleTranslateSnap { .. } => (
            "SetTranslateSnap",
            UiEventKind::Change,
            vec![UiBindingValue::Float(
                next_translate_snap(settings.translate_step) as f64,
            )],
        ),
        ViewportToolbarPointerRoute::CycleRotateSnapDegrees { .. } => (
            "SetRotateSnapDegrees",
            UiEventKind::Change,
            vec![UiBindingValue::Float(
                next_rotate_snap_degrees(settings.rotate_step_deg) as f64,
            )],
        ),
        ViewportToolbarPointerRoute::CycleScaleSnap { .. } => (
            "SetScaleSnap",
            UiEventKind::Change,
            vec![UiBindingValue::Float(
                next_scale_snap(settings.scale_step) as f64
            )],
        ),
        ViewportToolbarPointerRoute::TogglePreviewLighting { .. } => (
            "SetPreviewLighting",
            UiEventKind::Change,
            vec![UiBindingValue::Bool(!settings.preview_lighting)],
        ),
        ViewportToolbarPointerRoute::TogglePreviewSkybox { .. } => (
            "SetPreviewSkybox",
            UiEventKind::Change,
            vec![UiBindingValue::Bool(!settings.preview_skybox)],
        ),
        ViewportToolbarPointerRoute::ToggleGizmosEnabled { .. } => (
            "SetGizmosEnabled",
            UiEventKind::Change,
            vec![UiBindingValue::Bool(!settings.gizmos_enabled)],
        ),
        ViewportToolbarPointerRoute::FrameSelection { .. } => {
            ("FrameSelection", UiEventKind::Click, Vec::new())
        }
        ViewportToolbarPointerRoute::EnterPlayMode { .. } => {
            ("EnterPlayMode", UiEventKind::Click, Vec::new())
        }
        ViewportToolbarPointerRoute::ExitPlayMode { .. } => {
            ("ExitPlayMode", UiEventKind::Click, Vec::new())
        }
    };

    let Some(result) = dispatch_builtin_viewport_toolbar_control(
        runtime, bridge, control_id, event_kind, arguments,
    ) else {
        return Err(format!("Unknown viewport toolbar control {control_id}"));
    };
    result
}
