use crate::builtin::RuntimeTargetMode;
use crate::plugin::{
    CapabilityStatus, CapabilityStatusManifest, CapabilityView, PluginFeatureBundleManifest,
    PluginModuleManifest, PluginPackageManifest, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

#[test]
fn capability_view_projects_only_concrete_registration_reports() {
    let physics_registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("physics", "Physics")
            .with_capability("runtime.capability.physics.raycast")
            .with_capability_status(CapabilityStatusManifest::new(
                "runtime.capability.physics.raycast",
                CapabilityStatus::Complete,
            ))
            .with_runtime_module(
                PluginModuleManifest::runtime("physics.runtime", "zircon_plugin_physics_runtime")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["runtime.capability.physics.collider_world"]),
            ),
    );
    let sound_feature_registration =
        RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
            PluginFeatureBundleManifest::new("sound.occlusion", "Sound Occlusion", "sound")
                .with_capability("runtime.capability.sound.occlusion")
                .with_runtime_module(
                    PluginModuleManifest::runtime(
                        "sound.occlusion.runtime",
                        "zircon_plugin_sound_occlusion_runtime",
                    )
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["runtime.capability.sound.occlusion.debug"]),
                ),
            Some("sound".to_string()),
        );

    let capability_view = CapabilityView::from_registration_reports(
        [&physics_registration],
        [&sound_feature_registration],
    );

    assert!(capability_view.has("runtime.capability.physics.raycast"));
    assert!(capability_view.has("runtime.capability.physics.collider_world"));
    assert!(capability_view.has("runtime.capability.sound.occlusion"));
    assert!(capability_view.has("runtime.capability.sound.occlusion.debug"));
    assert_eq!(
        capability_view.status("runtime.capability.physics.raycast"),
        Some(CapabilityStatus::Complete)
    );
    assert_eq!(
        capability_view.status("runtime.capability.sound.occlusion"),
        None
    );
}
