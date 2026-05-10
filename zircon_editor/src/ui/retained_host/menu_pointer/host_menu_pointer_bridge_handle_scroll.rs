use zircon_runtime::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_dispatch::HostMenuPointerDispatch;
use super::host_menu_pointer_target::HostMenuPointerTarget;
use super::menu_item_tree::parent_path;
use super::node_ids::popup_node_id;
use super::popup_layout::{clamped_menu_bar_scroll_offset, menu_bar_contains_point};
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

        let mut hover_route = route.clone();
        if self.state.open_menu_index.is_none() && menu_bar_contains_point(&self.layout, point) {
            let next_offset = clamped_menu_bar_scroll_offset(
                &self.layout,
                self.state.menu_bar_scroll_offset + delta,
            );
            if (self.state.menu_bar_scroll_offset - next_offset).abs() > f32::EPSILON {
                self.state.menu_bar_scroll_offset = next_offset;
                self.rebuild_surface();
                hover_route =
                    self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
            }
        } else if self.state.open_menu_index.is_some() {
            if let Some(popup) = self.surface.tree.node(popup_node_id(0)) {
                let offset = popup.scroll_state.unwrap_or_default().offset;
                if (self.state.popup_scroll_offset - offset).abs() > f32::EPSILON {
                    self.state.popup_scroll_offset = offset;
                    self.rebuild_surface();
                }
            }
            hover_route =
                self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        }

        let mut rebuild_after_hover = false;
        match hover_route.as_ref() {
            Some(HostMenuPointerTarget::SubmenuBranch {
                menu_index,
                item_index,
                item_path,
            }) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = Some(*item_index);
                self.state.hovered_item_path = item_path.clone();
                if self.state.open_submenu_path != *item_path {
                    self.state.open_submenu_path = item_path.clone();
                    rebuild_after_hover = true;
                }
            }
            Some(HostMenuPointerTarget::MenuItem {
                menu_index,
                item_index,
                item_path,
                ..
            }) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = Some(*item_index);
                self.state.hovered_item_path = item_path.clone();
                let parent = parent_path(item_path);
                if self.state.open_submenu_path != parent {
                    self.state.open_submenu_path = parent;
                    rebuild_after_hover = true;
                }
            }
            Some(HostMenuPointerTarget::PopupSurface(menu_index)) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = None;
                self.state.hovered_item_path.clear();
            }
            Some(HostMenuPointerTarget::MenuButton(index)) => {
                self.state.hovered_menu_index = Some(*index);
                self.state.hovered_item_index = None;
                self.state.hovered_item_path.clear();
            }
            Some(HostMenuPointerTarget::DismissOverlay) | None => {
                self.state.hovered_item_index = None;
                self.state.hovered_item_path.clear();
            }
        }
        if rebuild_after_hover {
            self.rebuild_surface();
        }

        Ok(HostMenuPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
            action_id: None,
        })
    }
}
