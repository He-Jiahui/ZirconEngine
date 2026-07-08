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

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(date) = expected_slice_support_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = runtime_index_anchor_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = priority_plan_doc_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = status_row_data_support_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = render_shader_support_maps::expected_date_for_slice(slice) {
        return Some(date);
    }

    None
}
