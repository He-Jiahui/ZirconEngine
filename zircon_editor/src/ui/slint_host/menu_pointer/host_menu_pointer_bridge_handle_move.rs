use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_dispatch::HostMenuPointerDispatch;
use super::host_menu_pointer_target::HostMenuPointerTarget;
use super::menu_item_tree::parent_path;
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
                    self.state.open_submenu_path.clear();
                    rebuild = true;
                }
                self.state.hovered_menu_index = Some(*index);
                self.state.hovered_item_index = None;
                self.state.hovered_item_path.clear();
            }
            Some(HostMenuPointerTarget::SubmenuBranch {
                menu_index,
                item_index,
                item_path,
            }) => {
                if self.state.open_submenu_path != *item_path {
                    self.state.open_submenu_path = item_path.clone();
                    rebuild = true;
                }
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = Some(*item_index);
                self.state.hovered_item_path = item_path.clone();
            }
            Some(HostMenuPointerTarget::MenuItem {
                menu_index,
                item_index,
                item_path,
                ..
            }) => {
                let parent = parent_path(item_path);
                if self.state.open_submenu_path != parent {
                    self.state.open_submenu_path = parent;
                    rebuild = true;
                }
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = Some(*item_index);
                self.state.hovered_item_path = item_path.clone();
            }
            Some(HostMenuPointerTarget::PopupSurface(menu_index)) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = None;
                self.state.hovered_item_path.clear();
            }
            Some(HostMenuPointerTarget::DismissOverlay) | None => {
                self.state.hovered_menu_index = None;
                self.state.hovered_item_index = None;
                self.state.hovered_item_path.clear();
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
