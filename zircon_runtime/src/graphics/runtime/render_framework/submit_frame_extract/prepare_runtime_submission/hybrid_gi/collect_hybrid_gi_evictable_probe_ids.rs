use crate::graphics::types::HybridGiPrepareFrame;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract::prepare_runtime_submission) fn collect_hybrid_gi_evictable_probe_ids(
    prepare: Option<&HybridGiPrepareFrame>,
) -> Vec<u32> {
    prepare
        .map(|prepare| prepare.evictable_probe_ids.clone())
        .unwrap_or_default()
}
