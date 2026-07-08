use super::*;

pub(super) fn priority_plan_doc_status_map_source() -> String {
    priority_plan_doc_expected_slice_map_source("status")
}

pub(super) fn priority_plan_doc_date_map_source() -> String {
    priority_plan_doc_expected_slice_map_source("date")
}

pub(super) fn priority_plan_doc_owner_row_source() -> String {
    [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/layout_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/inventory_rows.rs",
    ]
    .into_iter()
    .map(read_runtime_src)
    .collect::<Vec<_>>()
    .join("\n")
}

fn priority_plan_doc_expected_slice_map_source(kind: &str) -> String {
    let root = format!(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/{kind}/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps"
    );
    [
        format!("{root}.rs"),
        format!("{root}/expected_slice_map_rows.rs"),
        format!("{root}/guard_child_owner_maps.rs"),
        format!("{root}/integrity_guard_maps.rs"),
        format!("{root}/inventory_sync_maps.rs"),
        format!("{root}/row_data_guard_maps.rs"),
        format!("{root}/status_mirror_maps.rs"),
    ]
    .into_iter()
    .map(|path| read_runtime_src(&path))
    .collect::<Vec<_>>()
    .join("\n")
}
