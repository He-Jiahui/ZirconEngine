use crate::ui::retained_host::callback_dispatch;
use crate::ui::retained_host::UiHostWindow;

mod docked;
mod floating;
mod pane_frame;

pub(in crate::ui::retained_host::app) fn attach_viewport_toolbar_surface_frames_to_ui(
    ui: &UiHostWindow,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    document_viewport_toolbar_width: Option<f32>,
) {
    ui.update_host_presentation(|presentation| {
        docked::attach_docked_viewport_toolbar_surface_frames(
            presentation,
            viewport_toolbar_bridge,
            document_viewport_toolbar_width,
        );
        floating::attach_floating_viewport_toolbar_surface_frames(
            presentation,
            viewport_toolbar_bridge,
        );
    });
}
