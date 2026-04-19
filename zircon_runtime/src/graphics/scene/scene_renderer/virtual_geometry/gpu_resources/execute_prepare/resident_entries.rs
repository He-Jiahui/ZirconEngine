use crate::graphics::types::VirtualGeometryPrepareFrame;

pub(super) fn resident_entries(prepare: &VirtualGeometryPrepareFrame) -> Vec<[u32; 2]> {
    prepare
        .resident_pages
        .iter()
        .map(|page| [page.page_id, page.slot])
        .collect()
}
