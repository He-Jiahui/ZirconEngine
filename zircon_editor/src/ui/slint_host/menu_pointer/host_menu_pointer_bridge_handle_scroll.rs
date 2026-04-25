use zircon_runtime::ui::{dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind};

use super::constants::{POPUP_NODE_ID, WINDOW_MENU_INDEX};
use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_dispatch::HostMenuPointerDispatch;
use super::host_menu_pointer_target::HostMenuPointerTarget;
use super::route_conversion::to_public_route;

impl HostMenuPointerBridge {
    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<HostMenuPointerDispatch, String> {
        let route = self.dispatch_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
        )?;

        if self.state.open_menu_index == Some(WINDOW_MENU_INDEX) {
            if let Some(popup) = self.surface.tree.node(POPUP_NODE_ID) {
                let offset = popup.scroll_state.unwrap_or_default().offset;
                if (self.state.popup_scroll_offset - offset).abs() > f32::EPSILON {
                    self.state.popup_scroll_offset = offset;
                    self.rebuild_surface();
                }
            }
        }

        match route.as_ref() {
            Some(HostMenuPointerTarget::MenuItem {
                menu_index,
                item_index,
                ..
            }) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = Some(*item_index);
            }
            Some(HostMenuPointerTarget::PopupSurface(menu_index)) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = None;
            }
            Some(HostMenuPointerTarget::MenuButton(index)) => {
                self.state.hovered_menu_index = Some(*index);
                self.state.hovered_item_index = None;
            }
            Some(HostMenuPointerTarget::DismissOverlay) | None => {
                self.state.hovered_item_index = None;
            }
        }

        Ok(HostMenuPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
            action_id: None,
        })
    }
}
