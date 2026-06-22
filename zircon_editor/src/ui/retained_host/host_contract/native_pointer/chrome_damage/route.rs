mod dock;
mod document;
mod floating;
mod host_page;

use super::super::routing::ChromePointerRoute;
use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use self::dock::{route_activity_rail_damage_frame, route_drawer_header_damage_frame};
use self::document::route_document_tab_damage_frame;
use self::floating::route_floating_window_header_damage_frame;
use self::host_page::route_host_page_tab_damage_frame;

pub(super) fn route_chrome_press_damage_frame(
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
) -> Option<FrameRect> {
    match route {
        ChromePointerRoute::ActivityRail { .. } => route_activity_rail_damage_frame(presentation),
        ChromePointerRoute::DocumentTab { surface_key, .. } => {
            route_document_tab_damage_frame(presentation, surface_key.as_str())
        }
        ChromePointerRoute::DrawerHeaderTab { surface_key, .. } => {
            route_drawer_header_damage_frame(presentation, surface_key.as_str())
        }
        ChromePointerRoute::HostPageTab { .. } => route_host_page_tab_damage_frame(presentation),
        ChromePointerRoute::Resize => None,
        ChromePointerRoute::FloatingWindowHeader { window_id } => {
            route_floating_window_header_damage_frame(presentation, window_id.as_str())
        }
    }
}
