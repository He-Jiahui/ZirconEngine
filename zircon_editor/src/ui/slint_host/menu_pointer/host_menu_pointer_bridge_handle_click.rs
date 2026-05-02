use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_dispatch::HostMenuPointerDispatch;
use super::host_menu_pointer_target::HostMenuPointerTarget;
use super::route_conversion::to_public_route;

impl HostMenuPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        point: UiPoint,
    ) -> Result<HostMenuPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        let action_id = match route.as_ref() {
            Some(HostMenuPointerTarget::MenuButton(index)) => {
                if self.state.open_menu_index == Some(*index) {
                    self.close_popup();
                } else {
                    self.open_popup(*index);
                }
                None
            }
            Some(HostMenuPointerTarget::MenuItem {
                action_id,
                menu_index,
                ..
            }) => {
                self.close_popup();
                Some((action_id.clone(), *menu_index))
            }
            Some(HostMenuPointerTarget::DismissOverlay) => {
                self.close_popup();
                None
            }
            Some(HostMenuPointerTarget::PopupSurface(menu_index)) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = None;
                None
            }
            None => {
                self.close_popup();
                None
            }
        };

        Ok(HostMenuPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
            action_id: action_id.map(|(action_id, _)| action_id),
        })
    }
}
