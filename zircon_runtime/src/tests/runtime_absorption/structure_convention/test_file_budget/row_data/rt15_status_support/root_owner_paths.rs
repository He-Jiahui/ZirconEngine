use super::*;

#[path = "owner_paths/expected_slice_maps.rs"]
mod expected_slice_maps;
#[path = "owner_paths/expected_slice_maps_folder_backed.rs"]
mod expected_slice_maps_folder_backed;
#[path = "owner_paths/folder_backed.rs"]
mod folder_backed;
#[path = "owner_paths/priority_plan_docs.rs"]
mod priority_plan_docs;
#[path = "owner_paths/root_rows.rs"]
mod root_rows;
#[path = "owner_paths/row_data_and_budget.rs"]
mod row_data_and_budget;
#[path = "owner_paths/runtime_index_anchors.rs"]
mod runtime_index_anchors;

pub(super) fn status_support_row_owner_path_groups(
) -> impl Iterator<Item = &'static [(&'static str, &'static str, usize)]> {
    [
        root_rows::STATUS_SUPPORT_ROOT_ROW_OWNER_PATHS,
        row_data_and_budget::STATUS_SUPPORT_ROW_DATA_AND_BUDGET_OWNER_PATHS,
        runtime_index_anchors::STATUS_SUPPORT_RUNTIME_INDEX_ANCHOR_OWNER_PATHS,
        priority_plan_docs::STATUS_SUPPORT_PRIORITY_PLAN_DOC_OWNER_PATHS,
    ]
    .into_iter()
    .chain(
        expected_slice_maps::STATUS_SUPPORT_EXPECTED_SLICE_MAP_OWNER_PATH_GROUPS
            .iter()
            .copied(),
    )
}
