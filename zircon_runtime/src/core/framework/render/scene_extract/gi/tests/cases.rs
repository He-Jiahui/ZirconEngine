use super::*;

#[test]
fn hybrid_gi_pre_m4_settings_default_to_dynamic_custom_profile() {
    let extract: RenderHybridGiExtract = serde_json::from_str(
        r#"{
            "enabled": true,
            "quality": "high",
            "trace_budget": 32,
            "card_budget": 64,
            "voxel_budget": 16,
            "debug_view": "surface_cache"
        }"#,
    )
    .expect("pre-M4 Hybrid GI settings should keep defaults for new M4 fields");

    assert_eq!(extract.mode, RenderHybridGiMode::DynamicOnly);
    assert_eq!(extract.profile, RenderHybridGiProfile::Custom);
}

#[test]
fn hybrid_gi_baked_mode_and_profile_serde_roundtrip() {
    let extract = RenderHybridGiExtract {
        enabled: true,
        mode: RenderHybridGiMode::BakedStaticDynamic,
        profile: RenderHybridGiProfile::IndoorStatic,
        quality: RenderHybridGiQuality::High,
        trace_budget: 32,
        card_budget: 64,
        voxel_budget: 16,
        debug_view: RenderHybridGiDebugView::InputSet,
    };

    let encoded = serde_json::to_string(&extract).expect("Hybrid GI settings should encode");
    let decoded: RenderHybridGiExtract =
        serde_json::from_str(&encoded).expect("Hybrid GI settings should decode");

    assert_eq!(decoded, extract);
}
