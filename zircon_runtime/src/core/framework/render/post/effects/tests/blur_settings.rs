use super::RenderBlurSettings;

#[test]
fn blur_settings_use_explicit_enable_predicate_and_clamp_upload_radius() {
    let disabled = RenderBlurSettings { radius: -1.0 };
    let enabled = RenderBlurSettings { radius: 2.5 };

    assert!(!disabled.is_enabled());
    assert_eq!(disabled.render_radius(), 0.0);
    assert!(enabled.is_enabled());
    assert_eq!(enabled.render_radius(), 2.5);
}
