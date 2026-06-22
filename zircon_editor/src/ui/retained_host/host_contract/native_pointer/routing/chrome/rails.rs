mod frame;
mod hit;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostChromeControlFrameData};
use crate::ui::retained_host::primitives::ModelRc;

use self::frame::activity_rail_frame_for_pointer;
use self::hit::activity_rail_button_hit;
use super::super::ChromePointerRoute;

pub(super) fn route_activity_rail(
    region: &FrameRect,
    rail_before_panel: bool,
    rail_width: f32,
    buttons: &ModelRc<HostChromeControlFrameData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let rail = activity_rail_frame_for_pointer(region, rail_before_panel, rail_width, x, y)?;
    activity_rail_button_hit(&rail, buttons, x, y)?;
    Some(ChromePointerRoute::ActivityRail {
        side: if rail_before_panel { "left" } else { "right" }.into(),
        local_x: x - rail.x,
        local_y: y - rail.y,
    })
}
