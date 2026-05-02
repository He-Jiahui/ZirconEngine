pub(super) fn append_node_and_cluster_cull_page_requests(
    page_request_ids: &mut Vec<u32>,
    requested_page_ids: &[u32],
    page_budget: u32,
) {
    for page_id in requested_page_ids {
        if page_request_ids.len() >= page_budget as usize {
            break;
        }
        if page_request_ids.contains(page_id) {
            continue;
        }

        page_request_ids.push(*page_id);
    }
}
