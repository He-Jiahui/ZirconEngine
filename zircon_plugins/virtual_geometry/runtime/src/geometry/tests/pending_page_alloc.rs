use std::collections::BTreeMap;

use crate::virtual_geometry::VirtualGeometryPageRequest;

use super::{pending_page_requests, VirtualGeometryRuntimeState};

#[test]
fn pending_page_parent_cycle_is_bounded() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.set_page_budget(4);
    state.replace_page_parent_pages(BTreeMap::from([(3, 1), (1, 2), (2, 1)]));
    state.push_pending_page_request(VirtualGeometryPageRequest::new(3, 128, 1));
    state.insert_pending_page(3);

    let requests = pending_page_requests(&state);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].page_id, 3);
}
