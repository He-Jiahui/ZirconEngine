use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventKind};

use crate::scene::viewport::PivotMode;

use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::{
    event_bridge::UiHostEventEffects, viewport_toolbar_pointer::ViewportToolbarPointerRoute,
};

use super::super::BuiltinViewportToolbarTemplateBridge;
use super::dispatch_builtin_viewport_toolbar_control;
use super::snap_cycle::{
    next_display_mode_name, next_grid_mode_name, next_rotate_snap_degrees, next_scale_snap,
    next_translate_snap,
};

pub(crate) fn dispatch_viewport_toolbar_pointer_route(
    runtime: &EditorHostEventController,
    bridge: &BuiltinViewportToolbarTemplateBridge,
    route: &ViewportToolbarPointerRoute,
) -> Result<UiHostEventEffects, String> {
    let (control_id, event_kind, arguments) = match route {
        ViewportToolbarPointerRoute::ActivateSceneMode { mode, .. } => (
            "ActivateSceneMode",
            UiEventKind::Change,
            vec![UiBindingValue::string(mode)],
        ),
        ViewportToolbarPointerRoute::SetTransformSpace { space, .. } => (
            "SetTransformSpace",
            UiEventKind::Change,
            vec![UiBindingValue::string(space)],
        ),
        ViewportToolbarPointerRoute::CyclePivotMode { .. } => {
            let settings = runtime.scene_viewport_settings();
            let next = match settings.pivot_mode {
                PivotMode::Primary => "Centroid",
                PivotMode::Centroid => "Primary",
            };
            (
                "SetPivotMode",
                UiEventKind::Change,
                vec![UiBindingValue::string(next)],
            )
        }
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
        ViewportToolbarPointerRoute::CycleDisplayMode { .. } => {
            let settings = runtime.scene_viewport_settings();
            (
                "SetDisplayMode",
                UiEventKind::Change,
                vec![UiBindingValue::string(next_display_mode_name(
                    settings.display_mode,
                ))],
            )
        }
        ViewportToolbarPointerRoute::CycleGridMode { .. } => {
            let settings = runtime.scene_viewport_settings();
            (
                "SetGridMode",
                UiEventKind::Change,
                vec![UiBindingValue::string(next_grid_mode_name(
                    settings.grid_mode,
                ))],
            )
        }
        ViewportToolbarPointerRoute::CycleTranslateSnap { .. } => {
            let settings = runtime.scene_viewport_settings();
            (
                "SetTranslateSnap",
                UiEventKind::Change,
                vec![UiBindingValue::Float(
                    next_translate_snap(settings.translate_step) as f64,
                )],
            )
        }
        ViewportToolbarPointerRoute::CycleRotateSnapDegrees { .. } => {
            let settings = runtime.scene_viewport_settings();
            (
                "SetRotateSnapDegrees",
                UiEventKind::Change,
                vec![UiBindingValue::Float(
                    next_rotate_snap_degrees(settings.rotate_step_deg) as f64,
                )],
            )
        }
        ViewportToolbarPointerRoute::CycleScaleSnap { .. } => {
            let settings = runtime.scene_viewport_settings();
            (
                "SetScaleSnap",
                UiEventKind::Change,
                vec![UiBindingValue::Float(
                    next_scale_snap(settings.scale_step) as f64
                )],
            )
        }
        ViewportToolbarPointerRoute::TogglePreviewLighting { .. } => {
            let settings = runtime.scene_viewport_settings();
            (
                "SetPreviewLighting",
                UiEventKind::Change,
                vec![UiBindingValue::Bool(!settings.preview_lighting)],
            )
        }
        ViewportToolbarPointerRoute::TogglePreviewSkybox { .. } => {
            let settings = runtime.scene_viewport_settings();
            (
                "SetPreviewSkybox",
                UiEventKind::Change,
                vec![UiBindingValue::Bool(!settings.preview_skybox)],
            )
        }
        ViewportToolbarPointerRoute::ToggleGizmosEnabled { .. } => {
            let settings = runtime.scene_viewport_settings();
            (
                "SetGizmosEnabled",
                UiEventKind::Change,
                vec![UiBindingValue::Bool(!settings.gizmos_enabled)],
            )
        }
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
