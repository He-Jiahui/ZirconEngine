use crate::types::HybridGiPrepareFrame;

pub(super) fn collect_hybrid_gi_evictable_probe_ids(
    prepare: Option<&HybridGiPrepareFrame>,
) -> Vec<u32> {
    prepare
        .map(|prepare| prepare.evictable_probe_ids.clone())
        .unwrap_or_default()
}
