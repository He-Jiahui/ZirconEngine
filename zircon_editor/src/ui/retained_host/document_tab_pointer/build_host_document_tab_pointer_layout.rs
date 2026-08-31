use crate::ui::workbench::model::WorkbenchViewModel;

use super::host_document_tab_pointer_item::HostDocumentTabPointerItem;
use super::host_document_tab_pointer_layout::HostDocumentTabPointerLayout;
use super::host_document_tab_pointer_surface::HostDocumentTabPointerSurface;

pub(crate) fn build_host_document_tab_pointer_layout(
    model: &WorkbenchViewModel,
) -> HostDocumentTabPointerLayout {
    let root_surface_count = if model.document_tabs.is_empty() { 0 } else { 1 };
    let mut surfaces = Vec::with_capacity(root_surface_count + model.floating_windows.len());
    if !model.document_tabs.is_empty() {
        surfaces.push(HostDocumentTabPointerSurface {
            key: "document".to_string(),
            items: model
                .document_tabs
                .iter()
                .map(|tab| HostDocumentTabPointerItem {
                    instance_id: tab.instance_id.clone(),
                    closeable: tab.closeable,
                })
                .collect(),
        });
    }

    surfaces.extend(model.floating_windows.iter().map(|window| {
        HostDocumentTabPointerSurface {
            key: window.window_id.0.clone(),
            items: window
                .tabs
                .iter()
                .map(|tab| HostDocumentTabPointerItem {
                    instance_id: tab.instance_id.clone(),
                    closeable: tab.closeable,
                })
                .collect(),
        }
    }));

    zircon_runtime::profile_counter!("editor", "ui.document_tab.receipt_projection_count", 1);
    zircon_runtime::profile_counter!(
        "editor",
        "ui.document_tab.receipt_projection_surface_count",
        surfaces.len()
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.document_tab.receipt_projection_tab_count",
        surfaces
            .iter()
            .map(|surface| surface.items.len())
            .sum::<usize>()
    );
    HostDocumentTabPointerLayout { surfaces }
}
