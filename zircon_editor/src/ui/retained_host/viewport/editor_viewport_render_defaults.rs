use crate::scene::viewport::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProfile, RenderQualityProfile,
};

const EDITOR_VIEWPORT_QUALITY_PROFILE_NAME: &str = "editor-viewport-default";
const EDITOR_HYBRID_GI_PROFILE_ENV: &str = "ZIRCON_EDITOR_HYBRID_GI_PROFILE";
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
    if let Ok(value) = std::env::var(EDITOR_HYBRID_GI_PROFILE_ENV) {
        if let Some(profile) = parse_editor_hybrid_gi_profile(&value) {
            settings.profile = profile;
            if profile != RenderHybridGiProfile::Custom {
                settings.trace_budget = 0;
                settings.card_budget = 0;
                settings.voxel_budget = 0;
            }
        }
    }
    if settings.profile != RenderHybridGiProfile::Custom {
        return;
    }
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

fn parse_editor_hybrid_gi_profile(value: &str) -> Option<RenderHybridGiProfile> {
    match value.trim().to_ascii_lowercase().as_str() {
        "fully-dynamic" | "fully_dynamic" => Some(RenderHybridGiProfile::FullyDynamic),
        "indoor-static" | "indoor_static" => Some(RenderHybridGiProfile::IndoorStatic),
        "open-world" | "open_world" => Some(RenderHybridGiProfile::OpenWorld),
        "cinematic" => Some(RenderHybridGiProfile::Cinematic),
        "custom" => Some(RenderHybridGiProfile::Custom),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_hybrid_gi_profile_parser_accepts_product_profile_labels() {
        assert_eq!(
            parse_editor_hybrid_gi_profile("fully-dynamic"),
            Some(RenderHybridGiProfile::FullyDynamic)
        );
        assert_eq!(
            parse_editor_hybrid_gi_profile("indoor-static"),
            Some(RenderHybridGiProfile::IndoorStatic)
        );
        assert_eq!(
            parse_editor_hybrid_gi_profile("open-world"),
            Some(RenderHybridGiProfile::OpenWorld)
        );
        assert_eq!(
            parse_editor_hybrid_gi_profile("cinematic"),
            Some(RenderHybridGiProfile::Cinematic)
        );
        assert_eq!(parse_editor_hybrid_gi_profile("unknown"), None);
    }
}
