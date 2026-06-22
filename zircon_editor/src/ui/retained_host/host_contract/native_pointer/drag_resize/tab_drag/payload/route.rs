use crate::ui::retained_host::host_contract::data::{HostWindowPresentationData, TabData};
use crate::ui::retained_host::primitives::SharedString;

use super::super::super::super::routing::ChromePointerRoute;
use super::document::document_tab_drag_payload;
use super::drawer::drawer_tab_drag_payload;

pub(in super::super) fn tab_drag_payload_for_route(
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
) -> Option<(TabData, SharedString)> {
    match route {
        ChromePointerRoute::DocumentTab {
            surface_key,
            index,
            close,
            ..
        } => {
            if *close {
                return None;
            }
            document_tab_drag_payload(presentation, surface_key, *index)
        }
        ChromePointerRoute::DrawerHeaderTab {
            surface_key, index, ..
        } => drawer_tab_drag_payload(presentation, surface_key, *index),
        ChromePointerRoute::ActivityRail { .. }
        | ChromePointerRoute::HostPageTab { .. }
        | ChromePointerRoute::FloatingWindowHeader { .. }
        | ChromePointerRoute::Resize => None,
    }
}
