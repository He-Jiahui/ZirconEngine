use zircon_ui::UiPoint;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::{
    event_bridge::SlintDispatchEffects,
    viewport_toolbar_pointer::{ViewportToolbarPointerBridge, ViewportToolbarPointerDispatch},
};

use super::super::{dispatch_viewport_toolbar_pointer_route, BuiltinViewportToolbarTemplateBridge};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedViewportToolbarPointerClickDispatch {
    pub pointer: ViewportToolbarPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_shared_viewport_toolbar_pointer_click(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinViewportToolbarTemplateBridge,
    pointer_bridge: &mut ViewportToolbarPointerBridge,
    surface_key: &str,
    control_id: &str,
    control_x: f32,
    control_y: f32,
    control_width: f32,
    control_height: f32,
    point: UiPoint,
) -> Result<SharedViewportToolbarPointerClickDispatch, String> {
    let (control_x, control_y, control_width, control_height) =
        if control_width > 0.0 && control_height > 0.0 {
            (control_x, control_y, control_width, control_height)
        } else {
            let frame = bridge.control_frame_for_action(control_id).ok_or_else(|| {
                format!("viewport toolbar projection is missing frame for control {control_id}")
            })?;
            (frame.x, frame.y, frame.width, frame.height)
        };
    let pointer = pointer_bridge.handle_click(
        surface_key,
        control_id,
        control_x,
        control_y,
        control_width,
        control_height,
        point,
    )?;
    let effects = pointer
        .route
        .as_ref()
        .map(|route| dispatch_viewport_toolbar_pointer_route(runtime, bridge, route))
        .transpose()?;
    Ok(SharedViewportToolbarPointerClickDispatch { pointer, effects })
}
