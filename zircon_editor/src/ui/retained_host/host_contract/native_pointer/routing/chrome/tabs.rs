mod document;
mod drawer;
mod host_page;

pub(super) use self::document::route_document_tabs;
pub(super) use self::drawer::route_drawer_header;
pub(super) use self::host_page::route_host_page_tabs;

use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::{
    geometry::{contains, translated},
    ChromePointerRoute,
};

pub(super) fn route_dock_overflow(
    surface_key: &str,
    origin: &FrameRect,
    overflow: &FrameRect,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    contains(&translated(overflow, origin.x, origin.y), x, y).then(|| {
        ChromePointerRoute::DockOverflow {
            surface_key: surface_key.into(),
        }
    })
}

#[cfg(test)]
#[path = "tests/tabs.rs"]
mod tests;
