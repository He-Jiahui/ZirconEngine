use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::frame_geometry::union_optional_frames;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::template_activation_semantics::dispatch_template_node_primary_press;
use crate::ui::retained_host::host_contract::template_input_semantics::hit_is_text_input;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::NativePointerButtonState;
use super::super::text_focus::focus_template_node_text_input;

pub(in crate::ui::retained_host::host_contract) fn dispatch_template_node_button(
    ui: &UiHostWindow,
    pane_host: &PaneSurfaceHostContext<'_>,
    hit: TemplateNodePointerHit,
    state: NativePointerButtonState,
    button: UiPointerButton,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    if state != NativePointerButtonState::Pressed || button != UiPointerButton::Primary {
        return None;
    }
    if hit_is_text_input(&hit) {
        if focus_template_node_text_input(ui, &hit) {
            let damage = union_optional_frames(cleared_text_input_frame, Some(hit.frame.clone()))
                .unwrap_or_else(|| hit.frame.clone());
            return Some(NativePointerDispatchResult::region(damage));
        }
        return Some(NativePointerDispatchResult::idle());
    }
    dispatch_template_node_primary_press(pane_host, hit);
    None
}
