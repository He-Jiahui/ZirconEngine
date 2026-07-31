use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;

use super::super::collection_projection::ProjectedCollection;
use super::super::selection_options::ProjectedSelectionOptions;
use super::super::string_lists::to_host_contract_shared_string_list;

pub(super) fn assign_options_collection_fields(
    node: &mut host_contract::TemplatePaneNodeData,
    selection_options: ProjectedSelectionOptions,
    collection: ProjectedCollection,
) {
    node.selection_state = selection_options.selection_state.into();
    node.search_query = selection_options.search_query.into();
    node.selected = selection_options.selected;
    node.tree_depth = selection_options.tree_depth;
    node.tree_indent_px = selection_options.tree_indent_px;
    node.options_text = selection_options.options_text.into();
    node.options = to_host_contract_shared_string_list(selection_options.options);
    node.structured_options = model_rc(selection_options.structured_options);

    node.collection_items = to_host_contract_shared_string_list(collection.items);
    node.collection_rows = model_rc(collection.rows);
    node.collection_fields = model_rc(collection.fields);
    node.virtualization_enabled = collection.virtualization_enabled;
    node.virtualization_item_extent = collection.virtualization_item_extent;
    node.virtualization_overscan = collection.virtualization_overscan;
    node.virtualization_total_count = collection.virtualization_total_count;
    node.virtualization_visible_start = collection.virtualization_visible_start;
    node.virtualization_visible_count = collection.virtualization_visible_count;
    node.pagination_page_index = collection.pagination_page_index;
    node.pagination_page_size = collection.pagination_page_size;
    node.pagination_page_count = collection.pagination_page_count;
    node.pagination_total_count = collection.pagination_total_count;
}
