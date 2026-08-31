use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::super::{VIEWPORT_POINTER_BUTTON_NONE, VIEWPORT_POINTER_MOVE};

pub(super) fn dispatch_viewport_pane_move(
    ui: &UiHostWindow,
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
) {
    ui.clear_hovered_template_node_for_pointer_move();
    match &pointer.target {
        PanePointerTarget::SceneViewport(_) => pane_host.invoke_scene_viewport_pointer_event(
            VIEWPORT_POINTER_MOVE,
            VIEWPORT_POINTER_BUTTON_NONE,
            pointer.local_x,
            pointer.local_y,
            0.0,
            false,
            false,
        ),
        PanePointerTarget::GameViewport(_) => pane_host.invoke_game_viewport_pointer_event(
            VIEWPORT_POINTER_MOVE,
            VIEWPORT_POINTER_BUTTON_NONE,
            pointer.local_x,
            pointer.local_y,
            0.0,
            false,
            false,
        ),
        _ => {}
    }
}
