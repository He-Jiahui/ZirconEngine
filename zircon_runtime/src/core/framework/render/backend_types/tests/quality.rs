use super::{RenderFeatureQualitySettings, RenderQualityProfile};

#[test]
fn ssao_is_fail_closed_until_the_quality_profile_explicitly_enables_it() {
    assert!(!RenderFeatureQualitySettings::default().screen_space_ambient_occlusion);
    assert!(
        !RenderQualityProfile::new("default")
            .features
            .screen_space_ambient_occlusion
    );
    assert!(
        RenderQualityProfile::new("explicit-ssao")
            .with_screen_space_ambient_occlusion(true)
            .features
            .screen_space_ambient_occlusion
    );
}
