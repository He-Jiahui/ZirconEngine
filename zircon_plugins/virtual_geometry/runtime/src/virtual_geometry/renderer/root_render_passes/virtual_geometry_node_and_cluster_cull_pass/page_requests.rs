use std::collections::BTreeSet;

#[cfg(test)]
#[path = "page_requests/allocation_tests.rs"]
mod allocation_tests;

pub(super) fn append_node_and_cluster_cull_page_requests(
    page_request_ids: &mut Vec<u32>,
    seen_page_request_ids: &mut BTreeSet<u32>,
    requested_page_ids: &[u32],
    page_budget: u32,
) {
    for page_id in requested_page_ids {
        if page_request_ids.len() >= page_budget as usize {
            break;
        }
        if !seen_page_request_ids.insert(*page_id) {
            continue;
        }

        page_request_ids.push(*page_id);
    }
}
