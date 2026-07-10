use crate::scene::viewport::{RenderFrameExtract, RenderHybridGiExtract, RenderQualityProfile};

const EDITOR_VIEWPORT_QUALITY_PROFILE_NAME: &str = "editor-viewport-default";
const EDITOR_HYBRID_GI_TRACE_BUDGET: u32 = 32;
const EDITOR_HYBRID_GI_CARD_BUDGET: u32 = 64;
const EDITOR_HYBRID_GI_VOXEL_BUDGET: u32 = 16;

pub(super) fn editor_viewport_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new(EDITOR_VIEWPORT_QUALITY_PROFILE_NAME)
        .with_virtual_geometry(false)
        .with_hybrid_global_illumination(true)
}

pub(super) fn apply_editor_viewport_render_defaults(extract: &mut RenderFrameExtract) {
    let settings = extract
        .lighting
        .hybrid_global_illumination
        .get_or_insert_with(RenderHybridGiExtract::default);
    settings.enabled = true;
    if settings.trace_budget == 0 {
        settings.trace_budget = EDITOR_HYBRID_GI_TRACE_BUDGET;
    }
    if settings.card_budget == 0 {
        settings.card_budget = EDITOR_HYBRID_GI_CARD_BUDGET;
    }
    if settings.voxel_budget == 0 {
        settings.voxel_budget = EDITOR_HYBRID_GI_VOXEL_BUDGET;
    }
}
