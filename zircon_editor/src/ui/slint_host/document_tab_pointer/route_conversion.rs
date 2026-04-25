use super::{
    host_document_tab_pointer_route::HostDocumentTabPointerRoute,
    host_document_tab_pointer_target::HostDocumentTabPointerTarget,
};

pub(in crate::ui::slint_host::document_tab_pointer) fn to_public_route(
    target: HostDocumentTabPointerTarget,
) -> HostDocumentTabPointerRoute {
    match target {
        HostDocumentTabPointerTarget::ActivateTab {
            surface_key,
            item_index,
            instance_id,
        } => HostDocumentTabPointerRoute::ActivateTab {
            surface_key,
            item_index,
            instance_id,
        },
        HostDocumentTabPointerTarget::CloseTab {
            surface_key,
            item_index,
            instance_id,
        } => HostDocumentTabPointerRoute::CloseTab {
            surface_key,
            item_index,
            instance_id,
        },
    }
}
