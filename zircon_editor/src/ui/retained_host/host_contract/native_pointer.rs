mod button_dispatch;
mod chrome_damage;
mod close_prompt_damage;
mod constants;
mod drag_resize;
mod menu_geometry;
mod move_dispatch;
mod pane_button_damage;
mod redraw_result;
mod resize_damage;
mod routing;
mod scroll_dispatch;
mod state;
mod tab_drag_damage;
mod template_hover_damage;
mod tooltip_target;
mod viewport_toolbar_damage;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use button_dispatch::asset_deletion_blocker_action_at;
pub(in crate::ui::retained_host::host_contract) use button_dispatch::dispatch_native_pointer_button;
pub(in crate::ui::retained_host::host_contract) use constants::{
    HOST_POINTER_DOWN, HOST_POINTER_MOVE, HOST_POINTER_UP, VIEWPORT_POINTER_BUTTON_MIDDLE,
    VIEWPORT_POINTER_BUTTON_NONE, VIEWPORT_POINTER_BUTTON_PRIMARY,
    VIEWPORT_POINTER_BUTTON_SECONDARY, VIEWPORT_POINTER_DOWN, VIEWPORT_POINTER_MOVE,
    VIEWPORT_POINTER_SCROLL, VIEWPORT_POINTER_UP,
};
pub(in crate::ui::retained_host::host_contract) use move_dispatch::dispatch_native_pointer_move;
pub(in crate::ui::retained_host::host_contract) use scroll_dispatch::dispatch_native_pointer_scroll;
pub(in crate::ui::retained_host::host_contract) use state::NativePointerButtonState;
pub(in crate::ui::retained_host::host_contract) use tooltip_target::tooltip_target_for_chrome_route;
pub(crate) use tooltip_target::{HostChromeTooltipTarget, WorkbenchTooltipPointerTarget};
pub(super) use viewport_toolbar_damage::{
    native_viewport_chrome_damage_frame, viewport_chrome_damage_frame,
};
