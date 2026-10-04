use std::sync::OnceLock;

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
    if let Some(profile) = editor_hybrid_gi_profile_override() {
        settings.profile = profile;
        if profile != RenderHybridGiProfile::Custom {
            settings.trace_budget = 0;
            settings.card_budget = 0;
            settings.voxel_budget = 0;
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

fn editor_hybrid_gi_profile_override() -> Option<RenderHybridGiProfile> {
    static PROFILE: OnceLock<Option<RenderHybridGiProfile>> = OnceLock::new();
    PROFILE
        .get_or_init(|| {
            std::env::var(EDITOR_HYBRID_GI_PROFILE_ENV)
                .ok()
                .and_then(|value| parse_editor_hybrid_gi_profile(&value))
        })
        .clone()
}

fn parse_editor_hybrid_gi_profile(value: &str) -> Option<RenderHybridGiProfile> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("fully-dynamic") || value.eq_ignore_ascii_case("fully_dynamic") {
        Some(RenderHybridGiProfile::FullyDynamic)
    } else if value.eq_ignore_ascii_case("indoor-static")
        || value.eq_ignore_ascii_case("indoor_static")
    {
        Some(RenderHybridGiProfile::IndoorStatic)
    } else if value.eq_ignore_ascii_case("open-world") || value.eq_ignore_ascii_case("open_world") {
        Some(RenderHybridGiProfile::OpenWorld)
    } else if value.eq_ignore_ascii_case("cinematic") {
        Some(RenderHybridGiProfile::Cinematic)
    } else if value.eq_ignore_ascii_case("custom") {
        Some(RenderHybridGiProfile::Custom)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "editor_viewport_render_defaults/tests/borrowed_parse_tests.rs"]
mod borrowed_parse_tests;

#[cfg(test)]
#[path = "tests/editor_viewport_render_defaults.rs"]
mod tests;
