mod document;
mod drawer;
mod host_page;

pub(super) use self::document::route_document_tabs;
pub(super) use self::drawer::route_drawer_header;
pub(super) use self::host_page::route_host_page_tabs;

use crate::ui::retained_host::host_contract::data::FrameRect;

use super::{
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
mod tests {
    use super::*;

    #[test]
    fn dock_overflow_route_uses_the_published_local_anchor_once() {
        let route = route_dock_overflow(
            "document",
            &FrameRect {
                x: 100.0,
                y: 40.0,
                ..FrameRect::default()
            },
            &FrameRect {
                x: 200.0,
                y: 2.0,
                width: 28.0,
                height: 28.0,
            },
            314.0,
            54.0,
        );

        assert!(matches!(
            route,
            Some(ChromePointerRoute::DockOverflow { surface_key })
                if surface_key == "document"
        ));
    }
}
