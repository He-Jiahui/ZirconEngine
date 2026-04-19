use zircon_runtime::ui::{dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind};

use super::route_conversion::to_public_route;
use super::workbench_menu_pointer_bridge::WorkbenchMenuPointerBridge;
use super::workbench_menu_pointer_dispatch::WorkbenchMenuPointerDispatch;
use super::workbench_menu_pointer_target::WorkbenchMenuPointerTarget;

impl WorkbenchMenuPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        point: UiPoint,
    ) -> Result<WorkbenchMenuPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        let action_id = match route.as_ref() {
            Some(WorkbenchMenuPointerTarget::MenuButton(index)) => {
                if self.state.open_menu_index == Some(*index) {
                    self.close_popup();
                } else {
                    self.open_popup(*index);
                }
                None
            }
            Some(WorkbenchMenuPointerTarget::MenuItem {
                action_id,
                menu_index,
                ..
            }) => {
                self.close_popup();
                Some((action_id.clone(), *menu_index))
            }
            Some(WorkbenchMenuPointerTarget::DismissOverlay) => {
                self.close_popup();
                None
            }
            Some(WorkbenchMenuPointerTarget::PopupSurface(menu_index)) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = None;
                None
            }
            None => {
                self.close_popup();
                None
            }
        };

        Ok(WorkbenchMenuPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
            action_id: action_id.map(|(action_id, _)| action_id),
        })
    }
}
