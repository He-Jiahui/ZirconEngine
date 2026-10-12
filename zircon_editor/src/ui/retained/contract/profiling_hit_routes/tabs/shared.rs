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
        if tab.control_id.as_str() != id {
            continue;
        }
        let hit = match origin {
            Some(origin) => contains(&translated(&tab.frame, origin.x, origin.y), x, y),
            None => contains(&tab.frame, x, y),
        };
        if hit {
            return true;
        }
    }
    false
}
