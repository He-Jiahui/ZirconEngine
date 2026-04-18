use crate::types::VirtualGeometryPrepareFrame;

pub(in crate::runtime::render_framework::submit_frame_extract::prepare_runtime_submission) fn collect_virtual_geometry_evictable_page_ids(
    prepare: Option<&VirtualGeometryPrepareFrame>,
) -> Vec<u32> {
    prepare
        .map(|prepare| {
            prepare
                .evictable_pages
                .iter()
                .map(|page| page.page_id)
                .collect()
        })
        .unwrap_or_default()
}
