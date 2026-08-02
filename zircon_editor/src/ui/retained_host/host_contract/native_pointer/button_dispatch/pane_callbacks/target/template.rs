use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::NativePointerButtonState;
use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::super::template_nodes::dispatch_template_node_button;

pub(super) fn dispatch_template_pane_target_button(
    ui: &UiHostWindow,
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    state: NativePointerButtonState,
    button: UiPointerButton,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    let PanePointerTarget::TemplateNode(hit) = &pointer.target else {
        return None;
    };
    dispatch_template_node_button(
        ui,
        pane_host,
        hit.clone(),
        state,
        button,
        cleared_text_input_frame,
    )
}
