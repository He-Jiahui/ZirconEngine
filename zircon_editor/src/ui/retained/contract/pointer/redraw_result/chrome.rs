use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::frame_geometry::union_frame;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::chrome_damage::chrome_press_damage_frame;
use super::super::routing::ChromePointerRoute;

pub(in crate::ui::retained_host::host_contract) fn chrome_press_redraw(
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
    extra_damage: Option<FrameRect>,
) -> NativePointerDispatchResult {
    let Some(frame) = chrome_press_damage_frame(presentation, route) else {
        return NativePointerDispatchResult::full_frame();
    };
    let damage = match extra_damage {
        Some(extra) => union_frame(&frame, &extra),
        None => frame,
    };
    NativePointerDispatchResult::region_with_frame_update(damage)
}
