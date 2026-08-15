use crate::ui::retained_host::host_contract::data::{FrameRect, HostChromeControlFrameData};
use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::geometry::{contains, translated};

pub(super) fn activity_rail_button_hit(
    rail: &FrameRect,
    buttons: &ModelRc<HostChromeControlFrameData>,
    x: f32,
    y: f32,
) -> Option<crate::ui::retained_host::primitives::SharedString> {
    for row in 0..buttons.row_count() {
        let button = buttons.row_data(row)?;
        let button_frame = translated(&button.frame, rail.x, rail.y);
        if contains(&button_frame, x, y) {
            return Some(button.control_id);
        }
    }
    None
}
