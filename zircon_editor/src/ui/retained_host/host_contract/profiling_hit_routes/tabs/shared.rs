use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{FrameRect, HostChromeTabData};
use super::super::geometry::{contains, translated};

pub(super) fn tab_route_hit(
    tabs: &ModelRc<HostChromeTabData>,
    id: &str,
    x: f32,
    y: f32,
    origin: Option<&FrameRect>,
) -> bool {
    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let frame = match origin {
            Some(origin) => translated(&tab.frame, origin.x, origin.y),
            None => tab.frame.clone(),
        };
        if tab.control_id.as_str() == id && contains(&frame, x, y) {
            return true;
        }
    }
    false
}
