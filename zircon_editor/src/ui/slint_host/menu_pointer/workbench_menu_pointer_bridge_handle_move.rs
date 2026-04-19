use zircon_runtime::ui::{dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind};

use super::route_conversion::to_public_route;
use super::workbench_menu_pointer_bridge::WorkbenchMenuPointerBridge;
use super::workbench_menu_pointer_dispatch::WorkbenchMenuPointerDispatch;
use super::workbench_menu_pointer_target::WorkbenchMenuPointerTarget;

impl WorkbenchMenuPointerBridge {
    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<WorkbenchMenuPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        let mut rebuild = false;
        match route.as_ref() {
            Some(WorkbenchMenuPointerTarget::MenuButton(index)) => {
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
            Some(WorkbenchMenuPointerTarget::MenuItem {
                menu_index,
                item_index,
                ..
            }) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = Some(*item_index);
            }
            Some(WorkbenchMenuPointerTarget::PopupSurface(menu_index)) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = None;
            }
            Some(WorkbenchMenuPointerTarget::DismissOverlay) | None => {
                self.state.hovered_menu_index = None;
                self.state.hovered_item_index = None;
            }
        }

        if rebuild {
            self.rebuild_surface();
        }

        Ok(WorkbenchMenuPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
            action_id: None,
        })
    }
}
