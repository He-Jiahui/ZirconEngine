use super::helpers::assert_feature_registration_module;

#[test]
fn linked_feature_registration_reports_contribute_runtime_modules() {
    assert_feature_registration_module(
        zircon_plugin_sound_timeline_animation_runtime::plugin_feature_registration(),
        "SoundTimelineAnimationFeatureModule",
        "Sound timeline animation trigger track feature",
    );
    assert_feature_registration_module(
        zircon_plugin_sound_ray_traced_convolution_runtime::plugin_feature_registration(),
        "SoundRayTracedConvolutionFeatureModule",
        "Sound ray-traced convolution reverb feature",
    );
}
