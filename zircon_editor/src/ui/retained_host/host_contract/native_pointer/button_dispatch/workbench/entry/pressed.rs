use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::primary::dispatch_workbench_primary_button;
use super::super::secondary::dispatch_workbench_secondary_button;

pub(super) fn dispatch_pressed_workbench_button(
    ui: &UiHostWindow,
    hit: TemplateNodePointerHit,
    button: UiPointerButton,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    match button {
        UiPointerButton::Secondary => Some(dispatch_workbench_secondary_button(
            ui,
            hit,
            x,
            y,
            cleared_text_input_frame,
        )),
        UiPointerButton::Primary => Some(dispatch_workbench_primary_button(
            ui,
            hit,
            cleared_text_input_frame,
        )),
        _ => None,
    }
}
