use zircon_runtime::ui::{dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind};

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_dispatch::HostMenuPointerDispatch;
use super::host_menu_pointer_target::HostMenuPointerTarget;
use super::route_conversion::to_public_route;

impl HostMenuPointerBridge {
    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<HostMenuPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        let mut rebuild = false;
        match route.as_ref() {
            Some(HostMenuPointerTarget::MenuButton(index)) => {
                if self.state.open_menu_index.is_some()
                    && self.state.open_menu_index != Some(*index)
                {
                    self.state.open_menu_index = Some(*index);
                    self.state.popup_scroll_offset = 0.0;
                    rebuild = true;
                }
                self.state.hovered_menu_index = Some(*index);
                self.state.hovered_item_index = None;
            }
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
            Some(HostMenuPointerTarget::DismissOverlay) | None => {
                self.state.hovered_menu_index = None;
                self.state.hovered_item_index = None;
            }
        }

        if rebuild {
            self.rebuild_surface();
        }

        Ok(HostMenuPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
            action_id: None,
        })
    }
}
