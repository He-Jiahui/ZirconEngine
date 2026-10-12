use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::super::routing::ChromePointerRoute;
use super::route::route_chrome_press_damage_frame;

pub(in crate::ui::retained_host::host_contract) fn chrome_press_damage_frame(
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
) -> Option<FrameRect> {
    route_chrome_press_damage_frame(presentation, route)
}
