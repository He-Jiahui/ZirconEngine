use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::host_menu_pointer_bridge::HostMenuPointerBridge;
use super::host_menu_pointer_dispatch::HostMenuPointerDispatch;
use super::host_menu_pointer_route_intent::HostMenuPointerRouteIntent;
use super::menu_item_tree::parent_path;
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

        let mut refreshed_hover_route = None;
        if self.state.open_menu_index.is_none()
            && delta.is_finite()
            && menu_bar_contains_point(&self.layout, point)
        {
            let next_offset = clamped_menu_bar_scroll_offset(
                &self.layout,
                self.state.menu_bar_scroll_offset + delta,
            );
            if (self.state.menu_bar_scroll_offset - next_offset).abs() > f32::EPSILON {
                self.state.menu_bar_scroll_offset = next_offset;
                self.rebuild_surface();
                refreshed_hover_route = Some(
                    self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?,
                );
            }
        } else if self.state.open_menu_index.is_some() {
            if self.root_popup_accepts_scroll(point) && delta.is_finite() {
                self.state.popup_scroll_offset += delta;
                self.clamp_popup_scroll_offset();
            }
            refreshed_hover_route = Some(self.project_open_popup_route_at_point(point));
        }

        let mut rebuild_after_hover = false;
        let hover_route = refreshed_hover_route.as_ref().unwrap_or(&route);
        match hover_route.as_ref() {
            Some(HostMenuPointerRouteIntent::SubmenuBranch {
                menu_index,
                item_index,
                item_path,
            }) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = Some(*item_index);
                reuse_menu_path(&mut self.state.hovered_item_path, item_path);
                if self.state.open_submenu_path != *item_path {
                    reuse_menu_path(&mut self.state.open_submenu_path, item_path);
                    rebuild_after_hover = true;
                }
            }
            Some(HostMenuPointerRouteIntent::MenuItem {
                menu_index,
                item_index,
                item_path,
                ..
            }) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = Some(*item_index);
                reuse_menu_path(&mut self.state.hovered_item_path, item_path);
                let parent = parent_path(item_path);
                if self.state.open_submenu_path != parent {
                    self.state.open_submenu_path = parent;
                    rebuild_after_hover = true;
                }
            }
            Some(HostMenuPointerRouteIntent::PopupSurface(menu_index)) => {
                self.state.hovered_menu_index = Some(*menu_index);
                self.state.hovered_item_index = None;
                self.state.hovered_item_path.clear();
            }
            Some(HostMenuPointerRouteIntent::MenuButton(index)) => {
                self.state.hovered_menu_index = Some(*index);
                self.state.hovered_item_index = None;
                self.state.hovered_item_path.clear();
            }
            Some(HostMenuPointerRouteIntent::DismissOverlay) | None => {
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

fn reuse_menu_path(target: &mut Vec<usize>, source: &[usize]) {
    target.clear();
    target.extend_from_slice(source);
}

#[cfg(test)]
#[path = "host_menu_pointer_bridge_handle_scroll/reused_path_tests.rs"]
mod reused_path_tests;
