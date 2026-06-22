use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::drag_resize::arm_native_tab_drag;
use super::super::redraw_result::{chrome_press_redraw, resize_pointer_redraw};
use super::super::routing::{route_top_level_chrome, ChromePointerRoute};
use super::chrome_press::dispatch_chrome_press;

pub(super) fn dispatch_top_level_chrome_primary_press(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    cleared_text_input_frame: Option<FrameRect>,
) -> Option<NativePointerDispatchResult> {
    let route = route_top_level_chrome(presentation, x, y)?;
    arm_native_tab_drag(ui, presentation, &route, x, y);
    let redraw = if matches!(&route, ChromePointerRoute::Resize) {
        resize_pointer_redraw(presentation, cleared_text_input_frame)
    } else {
        chrome_press_redraw(presentation, &route, cleared_text_input_frame)
    };
    dispatch_chrome_press(ui, route, x, y);
    Some(redraw)
}
