use std::collections::BTreeMap;

use crate::ui::template_runtime::RetainedUiHostBindingProjection;

mod fields;
mod items;
mod model;
mod pagination;
mod virtualization;

pub(super) use self::model::ProjectedCollection;

pub(super) fn projected_collection(
    component: &str,
    attributes: &BTreeMap<String, toml::Value>,
    bindings: &[RetainedUiHostBindingProjection],
) -> ProjectedCollection {
    let virtualization = virtualization::projected_virtualization(component, attributes);
    let pagination = pagination::projected_pagination(attributes);

    ProjectedCollection {
        items: items::projected_collection_items(attributes, &virtualization),
        rows: items::projected_collection_rows(attributes, &virtualization),
        fields: fields::projected_collection_fields(component, attributes, bindings),
        virtualization_enabled: virtualization.enabled,
        virtualization_item_extent: virtualization.item_extent,
        virtualization_overscan: virtualization.overscan,
        virtualization_total_count: virtualization.total_count,
        virtualization_visible_start: virtualization.visible_start,
        virtualization_visible_count: virtualization.visible_count,
        pagination_page_index: pagination.page_index,
        pagination_page_size: pagination.page_size,
        pagination_page_count: pagination.page_count,
        pagination_total_count: pagination.total_count,
    }
}
