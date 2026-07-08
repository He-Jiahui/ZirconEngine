#[path = "plan_doc_support_maps/expected_slice_support_maps.rs"]
mod expected_slice_support_maps;
#[path = "plan_doc_support_maps/priority_plan_doc_maps.rs"]
mod priority_plan_doc_maps;
#[path = "plan_doc_support_maps/render_shader_support_maps.rs"]
mod render_shader_support_maps;
#[path = "plan_doc_support_maps/runtime_index_anchor_maps.rs"]
mod runtime_index_anchor_maps;
#[path = "plan_doc_support_maps/status_row_data_support_maps.rs"]
mod status_row_data_support_maps;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = expected_slice_support_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = runtime_index_anchor_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = priority_plan_doc_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = status_row_data_support_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = render_shader_support_maps::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}
