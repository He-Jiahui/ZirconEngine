use crate::ui::retained_host::callback_dispatch;
use crate::ui::retained_host::host_contract::HostPanePresentationPatch;
use crate::ui::retained_host::UiHostWindow;

mod docked;
mod floating;
mod pane_frame;

pub(in crate::ui::retained_host::app) fn attach_viewport_toolbar_surface_frames_to_ui(
    ui: &UiHostWindow,
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    document_viewport_toolbar_width: Option<f32>,
) {
    viewport_toolbar_bridge.admit_layout_context(
        ui.window().scale_factor(),
        ui.get_host_presentation_generation().theme_generation(),
    );
    let _ = ui.patch_host_presentation_panes(|presentation| {
        let mut patch = HostPanePresentationPatch::new();
        docked::append_docked_viewport_toolbar_surface_frame_patches(
            &mut patch,
            presentation,
            viewport_toolbar_bridge,
            document_viewport_toolbar_width,
        );
        floating::append_floating_viewport_toolbar_surface_frame_patches(
            &mut patch,
            presentation,
            viewport_toolbar_bridge,
        );
        (!patch.is_empty()).then_some((patch, ()))
    });
}

#[cfg(test)]
#[path = "tests/surface_frames.rs"]
mod tests;
