use super::host_document_tab_pointer_layout::HostDocumentTabPointerLayout;
use super::host_document_tab_pointer_route::HostDocumentTabPointerRoute;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Default)]
pub(crate) struct HostDocumentTabPointerBridge {
    pub(in crate::ui::retained_host::document_tab_pointer) layout: HostDocumentTabPointerLayout,
}

impl HostDocumentTabPointerBridge {
    pub(in crate::ui::retained_host::document_tab_pointer) fn route_for_receipt(
        &self,
        surface_key: &str,
        item_index: usize,
        close: bool,
    ) -> Result<HostDocumentTabPointerRoute, String> {
        let (surface_index, surface) = self
            .layout
            .surfaces
            .iter()
            .enumerate()
            .find(|(_, surface)| surface.key == surface_key)
            .ok_or_else(|| format!("Unknown document tab surface {surface_key}"))?;
        let item = surface.items.get(item_index).ok_or_else(|| {
            format!("Document tab index {item_index} is outside surface {surface_key}")
        })?;
        if close && !item.closeable {
            return Err(format!(
                "Document tab index {item_index} on surface {surface_key} is not closeable"
            ));
        }
        Ok(if close {
            HostDocumentTabPointerRoute::CloseTab {
                surface_index,
                item_index,
            }
        } else {
            HostDocumentTabPointerRoute::ActivateTab {
                surface_index,
                item_index,
            }
        })
    }

    pub(crate) fn target_for_route(
        &self,
        route: HostDocumentTabPointerRoute,
    ) -> Option<&ViewInstanceId> {
        let (surface_index, item_index) = match route {
            HostDocumentTabPointerRoute::ActivateTab {
                surface_index,
                item_index,
            }
            | HostDocumentTabPointerRoute::CloseTab {
                surface_index,
                item_index,
            } => (surface_index, item_index),
        };
        self.layout
            .surfaces
            .get(surface_index)?
            .items
            .get(item_index)
            .map(|item| &item.instance_id)
    }
}
