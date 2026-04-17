use crate::types::VirtualGeometryPrepareFrame;

pub(super) fn resident_slots(prepare: &VirtualGeometryPrepareFrame) -> Vec<u32> {
    prepare
        .resident_pages
        .iter()
        .map(|page| page.slot)
        .collect()
}
