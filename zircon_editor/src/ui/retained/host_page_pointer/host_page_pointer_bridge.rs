use super::host_page_pointer_layout::HostPagePointerLayout;
use super::host_page_pointer_route::HostPagePointerRoute;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Default)]
pub(crate) struct HostPagePointerBridge {
    pub(super) layout: HostPagePointerLayout,
}

impl HostPagePointerBridge {
    pub(crate) fn activation_target_for_route(
        &self,
        route: HostPagePointerRoute,
    ) -> Option<&MainPageId> {
        let HostPagePointerRoute::Activate { item_index } = route else {
            return None;
        };
        self.layout.items.get(item_index).map(|item| &item.page_id)
    }

    pub(crate) fn close_target_for_route(
        &self,
        route: HostPagePointerRoute,
    ) -> Option<&ViewInstanceId> {
        let HostPagePointerRoute::Close { item_index } = route else {
            return None;
        };
        self.layout
            .items
            .get(item_index)?
            .close_instance_id
            .as_ref()
    }
}
