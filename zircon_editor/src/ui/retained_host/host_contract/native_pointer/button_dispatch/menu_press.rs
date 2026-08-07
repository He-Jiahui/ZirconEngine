use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::menu_geometry::{
    menu_damage_frame_with_state, menu_handles_point_with_state,
    menu_popup_handles_point_with_state,
};

pub(super) fn dispatch_menu_primary_press(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    menu_state: &HostMenuStateData,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    if !menu_handles_point_with_state(presentation, menu_state, x, y)
        && !menu_popup_handles_point_with_state(presentation, menu_state, x, y)
    {
        return None;
    }
    let before_damage = menu_damage_frame_with_state(presentation, menu_state);
    ui.global::<UiHostContext>()
        .invoke_menu_pointer_clicked(x, y);
    let after_state = ui.get_menu_state();
    let after_damage = menu_damage_frame_with_state(presentation, &after_state);
    let mut damage = union_frame(&before_damage, &after_damage);
    if let Some(frame) = cleared_text_input_frame {
        damage = union_frame(&damage, &frame);
    }
    Some(NativePointerDispatchResult::region(damage))
}
