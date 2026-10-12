#[path = "pointer/buttons.rs"]
mod button_dispatch;
#[path = "pointer/chrome_damage.rs"]
mod chrome_damage;
#[path = "pointer/close_prompt_damage.rs"]
mod close_prompt_damage;
#[path = "pointer/constants.rs"]
mod constants;
#[path = "pointer/drag_resize.rs"]
mod drag_resize;
#[path = "pointer/menu_geometry.rs"]
mod menu_geometry;
#[path = "pointer/motion.rs"]
mod move_dispatch;
#[path = "pointer/pane_button_damage.rs"]
mod pane_button_damage;
#[path = "pointer/redraw_result.rs"]
mod redraw_result;
#[path = "pointer/resize_damage.rs"]
mod resize_damage;
#[path = "pointer/routing.rs"]
pub(super) mod routing;
#[path = "pointer/scroll.rs"]
mod scroll_dispatch;
#[path = "pointer/state.rs"]
mod state;
#[path = "pointer/tab_drag_damage.rs"]
mod tab_drag_damage;
#[path = "pointer/hover_damage.rs"]
mod template_hover_damage;
#[path = "pointer/tooltip_target.rs"]
mod tooltip_target;
#[path = "pointer/viewport_toolbar_damage.rs"]
mod viewport_toolbar_damage;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use button_dispatch::asset_deletion_blocker_action_at;
pub(in crate::ui::retained_host::host_contract) use button_dispatch::dispatch_native_pointer_button;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use button_dispatch::dispatch_pane_button;
pub(in crate::ui::retained_host::host_contract) use constants::{
    HOST_POINTER_CANCEL, HOST_POINTER_DOWN, HOST_POINTER_MOVE, HOST_POINTER_UP,
    VIEWPORT_POINTER_BUTTON_MIDDLE, VIEWPORT_POINTER_BUTTON_NONE, VIEWPORT_POINTER_BUTTON_PRIMARY,
    VIEWPORT_POINTER_BUTTON_SECONDARY, VIEWPORT_POINTER_DOWN, VIEWPORT_POINTER_MOVE,
    VIEWPORT_POINTER_SCROLL, VIEWPORT_POINTER_UP,
};
pub(in crate::ui::retained_host::host_contract) use move_dispatch::dispatch_native_pointer_move;
pub(in crate::ui::retained_host::host_contract) use routing::{
    route_pointer_move_to_pane, PanePointerTarget,
};
pub(in crate::ui::retained_host::host_contract) use scroll_dispatch::dispatch_native_pointer_scroll;
pub(in crate::ui::retained_host::host_contract) use state::NativePointerButtonState;
pub(in crate::ui::retained_host::host_contract) use tooltip_target::tooltip_target_for_chrome_route;
pub(crate) use tooltip_target::{HostChromeTooltipTarget, WorkbenchTooltipPointerTarget};
pub(super) use viewport_toolbar_damage::{
    native_viewport_chrome_damage_frame, viewport_chrome_damage_frame,
};
