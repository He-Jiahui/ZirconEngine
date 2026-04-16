use super::{
    workbench_document_tab_pointer_route::WorkbenchDocumentTabPointerRoute,
    workbench_document_tab_pointer_target::WorkbenchDocumentTabPointerTarget,
};

pub(in crate::host::slint_host::document_tab_pointer) fn to_public_route(
    target: WorkbenchDocumentTabPointerTarget,
) -> WorkbenchDocumentTabPointerRoute {
    match target {
        WorkbenchDocumentTabPointerTarget::ActivateTab {
            surface_key,
            item_index,
            instance_id,
        } => WorkbenchDocumentTabPointerRoute::ActivateTab {
            surface_key,
            item_index,
            instance_id,
        },
        WorkbenchDocumentTabPointerTarget::CloseTab {
            surface_key,
            item_index,
            instance_id,
        } => WorkbenchDocumentTabPointerRoute::CloseTab {
            surface_key,
            item_index,
            instance_id,
        },
    }
}
