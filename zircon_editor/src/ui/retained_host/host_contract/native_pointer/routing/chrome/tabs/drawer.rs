use crate::ui::retained_host::host_contract::data::{FrameRect, HostChromeTabData};
use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::{
    ChromePointerRoute,
    geometry::{contains, translated},
};

pub(in crate::ui::retained_host::host_contract::native_pointer::routing::chrome) fn route_drawer_header(
    surface_key: &str,
    region: &FrameRect,
    header: &FrameRect,
    tabs: &ModelRc<HostChromeTabData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let header_origin = translated(header, region.x, region.y);
    for row in 0..tabs.row_count() {
        let tab = tabs.row_data(row)?;
        let tab_frame = translated(&tab.frame, header_origin.x, header_origin.y);
        if contains(&tab_frame, x, y) {
            return Some(ChromePointerRoute::DrawerHeaderTab {
                surface_key: surface_key.into(),
                index: row,
                tab_x: tab.frame.x,
                tab_width: tab.frame.width,
                local_x: x - header_origin.x,
                local_y: y - header_origin.y,
            });
        }
    }
    None
}
