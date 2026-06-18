mod button_dispatch;
mod chrome_damage;
mod close_prompt_damage;
mod drag_resize;
mod menu_geometry;
mod move_dispatch;
mod pane_button_damage;
mod redraw_result;
mod resize_damage;
mod routing;
mod scroll_dispatch;
mod tab_drag_damage;
mod template_hover_damage;
mod viewport_toolbar_damage;

pub(super) use button_dispatch::dispatch_native_pointer_button;
pub(super) use move_dispatch::dispatch_native_pointer_move;
pub(super) use scroll_dispatch::dispatch_native_pointer_scroll;

const HOST_POINTER_DOWN: i32 = 0;
const HOST_POINTER_MOVE: i32 = 1;
const HOST_POINTER_UP: i32 = 2;
const VIEWPORT_POINTER_DOWN: i32 = 0;
const VIEWPORT_POINTER_MOVE: i32 = 1;
const VIEWPORT_POINTER_UP: i32 = 2;
const VIEWPORT_POINTER_SCROLL: i32 = 3;
const VIEWPORT_POINTER_BUTTON_NONE: i32 = 0;
const VIEWPORT_POINTER_BUTTON_PRIMARY: i32 = 1;
const VIEWPORT_POINTER_BUTTON_SECONDARY: i32 = 2;
const VIEWPORT_POINTER_BUTTON_MIDDLE: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum NativePointerButtonState {
    Pressed,
    Released,
}
