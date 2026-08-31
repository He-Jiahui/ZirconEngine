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
#[path = "editor_viewport_render_defaults/borrowed_parse_tests.rs"]
mod borrowed_parse_tests;

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

    #[test]
    fn editor_hybrid_gi_environment_override_uses_a_process_cache() {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/ui/retained_host/viewport/editor_viewport_render_defaults.rs"),
        )
        .expect("render defaults source should read");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("render defaults production source should exist");

        assert!(production.contains("static PROFILE: OnceLock<Option<RenderHybridGiProfile>>"));
        assert_eq!(production.matches("std::env::var(").count(), 1);
    }
}
