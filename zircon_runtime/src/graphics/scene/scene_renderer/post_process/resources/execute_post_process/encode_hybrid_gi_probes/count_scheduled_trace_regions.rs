use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;

pub(super) fn count_scheduled_trace_regions(frame: &ViewportRenderFrame) -> u32 {
    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return 0;
    };
    let Some(hybrid_gi_extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() else {
        return 0;
    };

    prepare
        .scheduled_trace_region_ids
        .iter()
        .take(MAX_HYBRID_GI_TRACE_REGIONS)
        .filter(|region_id| {
            hybrid_gi_extract
                .trace_regions
                .iter()
                .any(|candidate| candidate.region_id == **region_id)
        })
        .count() as u32
}
