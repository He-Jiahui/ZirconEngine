use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_dispatch::HostMenuPointerDispatch;
use super::host_menu_pointer_route_intent::HostMenuPointerRouteIntent;
use super::menu_item_tree::parent_path;
use super::route_conversion::to_public_route;

impl HostMenuPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        point: UiPoint,
    ) -> Result<HostMenuPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        let action_id = match route.as_ref() {
            Some(HostMenuPointerRouteIntent::MenuButton(index)) => {
                if self.state.open_menu_index == Some(*index) {
                    self.close_popup();
                } else {
                    self.open_popup(*index);
                }
                None
            }
            Some(HostMenuPointerRouteIntent::SubmenuBranch {
                menu_index,
                item_index,
                item_path,
            }) => {
                self.state.open_menu_index = Some(*menu_index);
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = Some(*item_index);
                reuse_menu_path(&mut self.state.hovered_item_path, item_path);
                if self.state.open_submenu_path != *item_path {
                    reuse_menu_path(&mut self.state.open_submenu_path, item_path);
                    self.rebuild_surface();
                }
                None
            }
            Some(HostMenuPointerRouteIntent::MenuItem {
                action_id,
                menu_index,
                item_path,
                ..
            }) => {
                self.state.open_submenu_path = parent_path(item_path);
                self.close_popup();
                Some((action_id.clone(), *menu_index))
            }
            Some(HostMenuPointerRouteIntent::DismissOverlay) => {
                self.close_popup();
                None
            }
            Some(HostMenuPointerRouteIntent::PopupSurface(menu_index)) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = None;
                self.state.hovered_item_path.clear();
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

fn reuse_menu_path(target: &mut Vec<usize>, source: &[usize]) {
    target.clear();
    target.extend_from_slice(source);
}

#[cfg(test)]
#[path = "host_menu_pointer_bridge_handle_click/reused_path_tests.rs"]
mod reused_path_tests;
