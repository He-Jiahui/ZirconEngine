use super::host_drawer_header_pointer_layout::HostDrawerHeaderPointerLayout;
use super::host_drawer_header_pointer_route::HostDrawerHeaderPointerRoute;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Default)]
pub(crate) struct HostDrawerHeaderPointerBridge {
    pub(in crate::ui::retained_host::drawer_header_pointer) layout: HostDrawerHeaderPointerLayout,
}

impl HostDrawerHeaderPointerBridge {
    pub(in crate::ui::retained_host::drawer_header_pointer) fn route_for_receipt(
        &self,
        surface_key: &str,
        item_index: usize,
    ) -> Result<HostDrawerHeaderPointerRoute, String> {
        let (surface_index, surface) = self
            .layout
            .surfaces
            .iter()
            .enumerate()
            .find(|(_, surface)| surface.key == surface_key)
            .ok_or_else(|| format!("Unknown drawer header surface {surface_key}"))?;
        if surface.items.get(item_index).is_none() {
            return Err(format!(
                "Drawer header index {item_index} is outside surface {surface_key}"
            ));
        }
        Ok(HostDrawerHeaderPointerRoute::Tab {
            surface_index,
            item_index,
        })
    }

    pub(crate) fn target_for_route(
        &self,
        route: HostDrawerHeaderPointerRoute,
    ) -> Option<(ActivityDrawerSlot, &ViewInstanceId)> {
        let HostDrawerHeaderPointerRoute::Tab {
            surface_index,
            item_index,
        } = route;
        let item = self
            .layout
            .surfaces
            .get(surface_index)?
            .items
            .get(item_index)?;
        Some((item.slot, &item.instance_id))
    }
}
