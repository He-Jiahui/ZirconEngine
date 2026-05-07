use zircon_runtime_interface::ui::layout::UiPoint;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::{
    event_bridge::UiHostEventEffects,
    viewport_toolbar_pointer::{ViewportToolbarPointerBridge, ViewportToolbarPointerDispatch},
};

use super::super::{
    dispatch_builtin_viewport_toolbar_control, dispatch_viewport_toolbar_pointer_route,
    BuiltinViewportToolbarTemplateBridge,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedViewportToolbarPointerClickDispatch {
    pub pointer: ViewportToolbarPointerDispatch,
    pub effects: Option<UiHostEventEffects>,
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
        } else if let Some(frame) = bridge.control_frame_for_control(control_id) {
            (frame.x, frame.y, frame.width, frame.height)
        } else {
            (point.x, point.y, 1.0, 1.0)
        };
    let pointer = match pointer_bridge.handle_click(
        surface_key,
        control_id,
        control_x,
        control_y,
        control_width,
        control_height,
        point,
    ) {
        Ok(pointer) => pointer,
        Err(error) if error.contains("Unknown viewport toolbar control") => {
            let effects =
                dispatch_projection_control(runtime, bridge, control_id).ok_or(error)??;
            return Ok(SharedViewportToolbarPointerClickDispatch {
                pointer: ViewportToolbarPointerDispatch { route: None },
                effects: Some(effects),
            });
        }
        Err(error) => return Err(error),
    };
    let effects = pointer
        .route
        .as_ref()
        .map(|route| dispatch_viewport_toolbar_pointer_route(runtime, bridge, route))
        .transpose()?;
    Ok(SharedViewportToolbarPointerClickDispatch { pointer, effects })
}

fn dispatch_projection_control(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinViewportToolbarTemplateBridge,
    control_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    for event_kind in [
        zircon_runtime_interface::ui::binding::UiEventKind::Click,
        zircon_runtime_interface::ui::binding::UiEventKind::Change,
    ] {
        if bridge.binding_for_control(control_id, event_kind).is_some() {
            return dispatch_builtin_viewport_toolbar_control(
                runtime,
                bridge,
                control_id,
                event_kind,
                Vec::new(),
            );
        }
    }
    None
}
