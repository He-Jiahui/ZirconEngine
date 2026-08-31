use crate::ui::workbench::model::WorkbenchViewModel;

use super::host_page_pointer_item::HostPagePointerItem;
use super::host_page_pointer_layout::HostPagePointerLayout;

pub(crate) fn build_host_page_pointer_layout(model: &WorkbenchViewModel) -> HostPagePointerLayout {
    let items = model
        .host_strip
        .pages
        .iter()
        .map(|page| HostPagePointerItem {
            page_id: page.id.clone(),
            close_instance_id: page.close_instance_id.clone(),
        })
        .collect::<Vec<_>>();

    zircon_runtime::profile_counter!("editor", "ui.host_page.receipt_projection_count", 1);
    zircon_runtime::profile_counter!(
        "editor",
        "ui.host_page.receipt_projection_item_count",
        items.len()
    );
    HostPagePointerLayout { items }
}
