use crate::plugin::{
    CapabilityStatus, CapabilityStatusManifest, PluginMaturity, PluginPackageManifest,
    RuntimePluginAvailabilityEntry, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
    RuntimeProfileDescriptor, RuntimeProfileId,
};
use crate::{RuntimePluginId, RuntimeTargetMode};

#[test]
fn plugin_manifest_roundtrips_maturity_and_capability_statuses() {
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_maturity(PluginMaturity::Beta)
        .with_capability("runtime.plugin.weather")
        .with_capability_status(
            CapabilityStatusManifest::new("runtime.plugin.weather", CapabilityStatus::Partial)
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_bevy_reference("dev/bevy/crates/bevy_app/src/plugin.rs"),
        );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");

    assert_eq!(decoded.maturity, PluginMaturity::Beta);
    assert_eq!(decoded.capability_statuses.len(), 1);
    assert_eq!(
        decoded.capability_statuses[0].capability,
        "runtime.plugin.weather"
    );
    assert_eq!(
        decoded.capability_statuses[0].status,
        CapabilityStatus::Partial
    );
    assert_eq!(decoded, manifest);
}

#[test]
fn runtime_plugin_descriptor_projects_maturity_and_statuses_to_manifest() {
    let descriptor = RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_maturity(PluginMaturity::Beta)
    .with_capability("runtime.plugin.sound")
    .with_capability_status(
        CapabilityStatusManifest::new("runtime.plugin.sound", CapabilityStatus::Partial)
            .with_bevy_reference("dev/bevy/crates/bevy_audio/src/lib.rs"),
    );

    let manifest = descriptor.package_manifest();

    assert_eq!(manifest.maturity, PluginMaturity::Beta);
    assert_eq!(manifest.capability_statuses, descriptor.capability_statuses);
    assert_eq!(
        manifest.capability_statuses[0].status,
        CapabilityStatus::Partial
    );
}

#[test]
fn builtin_catalog_classifies_bevy_parity_runtime_plugins() {
    let catalog = RuntimePluginDescriptor::builtin_catalog();
    let sound = descriptor(&catalog, RuntimePluginId::Sound);
    let animation = descriptor(&catalog, RuntimePluginId::Animation);
    let particles = descriptor(&catalog, RuntimePluginId::Particles);
    let rendering = descriptor(&catalog, RuntimePluginId::Rendering);

    assert_eq!(sound.maturity, PluginMaturity::Beta);
    assert!(sound.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.sound" && status.status == CapabilityStatus::Partial
    }));
    assert_eq!(animation.maturity, PluginMaturity::Beta);
    assert_eq!(particles.maturity, PluginMaturity::Experimental);
    assert_eq!(rendering.maturity, PluginMaturity::Stable);
}

#[test]
fn runtime_profiles_cover_expected_bevy_grade_profile_ids() {
    let profiles = RuntimeProfileDescriptor::builtin_profiles();
    let ids = profiles
        .iter()
        .map(|profile| profile.id)
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        vec![
            RuntimeProfileId::Minimal,
            RuntimeProfileId::Client2d,
            RuntimeProfileId::Client3d,
            RuntimeProfileId::Editor,
            RuntimeProfileId::Dev,
            RuntimeProfileId::Server,
        ]
    );

    let client_2d = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client2d);
    assert_eq!(client_2d.target_mode, RuntimeTargetMode::ClientRuntime);
    assert!(client_2d
        .default_plugins
        .iter()
        .any(|plugin| plugin.id == RuntimePluginId::Sound && plugin.required));
    assert!(client_2d
        .default_plugins
        .iter()
        .any(|plugin| plugin.id == RuntimePluginId::Rendering && plugin.required));
    assert!(client_2d
        .optional_plugins
        .contains(&RuntimePluginId::Particles));

    let server = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Server);
    assert_eq!(server.target_mode, RuntimeTargetMode::ServerRuntime);
    assert!(server
        .default_plugins
        .iter()
        .all(|plugin| plugin.id != RuntimePluginId::Sound && plugin.id != RuntimePluginId::Ui));
}

#[test]
fn profile_project_manifest_is_deterministic_and_target_scoped() {
    let profile = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client3d);
    let manifest = profile.project_manifest();
    let ids = manifest
        .selections
        .iter()
        .map(|selection| selection.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["ui", "sound", "rendering", "texture"]);
    assert!(manifest.selections.iter().all(|selection| {
        selection.target_modes == vec![RuntimeTargetMode::ClientRuntime] && selection.enabled
    }));
}

#[test]
fn profile_availability_report_blocks_required_externalized_or_stub_plugins() {
    let profile = RuntimeProfileDescriptor::new(
        RuntimeProfileId::Client2d,
        "test.client_2d",
        RuntimeTargetMode::ClientRuntime,
    )
    .with_minimum_maturity(PluginMaturity::Beta)
    .with_default_plugin(RuntimePluginId::Sound, true)
    .with_default_plugin(RuntimePluginId::Particles, true)
    .with_default_plugin(RuntimePluginId::Terrain, true);
    let descriptors = [
        RuntimePluginDescriptor::new(
            "sound",
            "Sound",
            RuntimePluginId::Sound,
            "zircon_plugin_sound_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_maturity(PluginMaturity::Externalized),
        RuntimePluginDescriptor::new(
            "particles",
            "Particles",
            RuntimePluginId::Particles,
            "zircon_plugin_particles_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_maturity(PluginMaturity::Stub),
        RuntimePluginDescriptor::new(
            "terrain",
            "Terrain",
            RuntimePluginId::Terrain,
            "zircon_plugin_terrain_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_maturity(PluginMaturity::Experimental),
    ];

    let report = profile.availability_report(descriptors.iter(), std::iter::empty::<&str>());

    assert!(contains_entry(&report.externalized_missing, "sound"));
    assert!(contains_entry(&report.stub, "particles"));
    assert!(contains_entry(&report.blocked_by_maturity, "terrain"));
    assert!(contains_entry(&report.missing_required, "sound"));
    assert!(contains_entry(&report.missing_required, "particles"));
    assert!(contains_entry(&report.missing_required, "terrain"));
}

#[test]
fn profile_availability_report_warns_for_optional_unavailable_plugins_without_missing_required() {
    let profile = RuntimeProfileDescriptor::new(
        RuntimeProfileId::Client3d,
        "test.client_3d",
        RuntimeTargetMode::ClientRuntime,
    )
    .with_minimum_maturity(PluginMaturity::Beta)
    .with_default_plugin(RuntimePluginId::Rendering, true)
    .with_optional_plugin(RuntimePluginId::Particles)
    .with_optional_plugin(RuntimePluginId::Navigation);
    let descriptors = [
        RuntimePluginDescriptor::new(
            "rendering",
            "Rendering",
            RuntimePluginId::Rendering,
            "zircon_plugin_rendering_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_maturity(PluginMaturity::Stable),
        RuntimePluginDescriptor::new(
            "particles",
            "Particles",
            RuntimePluginId::Particles,
            "zircon_plugin_particles_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_maturity(PluginMaturity::Experimental),
        RuntimePluginDescriptor::new(
            "navigation",
            "Navigation",
            RuntimePluginId::Navigation,
            "zircon_plugin_navigation_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ServerRuntime])
        .with_maturity(PluginMaturity::Beta),
    ];

    let report = profile.availability_report(descriptors.iter(), std::iter::empty::<&str>());

    assert!(contains_entry(&report.available, "rendering"));
    assert!(contains_entry(&report.blocked_by_maturity, "particles"));
    assert!(contains_entry(&report.blocked_by_target, "navigation"));
    assert!(report.missing_required.is_empty());
}

#[test]
fn stable_profile_defaults_do_not_require_externalized_or_stub_plugins() {
    let descriptors = RuntimePluginDescriptor::builtin_catalog();
    for profile_id in [RuntimeProfileId::Client2d, RuntimeProfileId::Client3d] {
        let profile = RuntimeProfileDescriptor::for_id(profile_id);
        let report = profile.availability_report(descriptors.iter(), std::iter::empty::<&str>());

        assert!(
            report.missing_required.is_empty(),
            "profile {:?} has unavailable required plugins: {:?}",
            profile_id,
            report.missing_required
        );
    }
}

#[test]
fn required_profile_plugins_need_linked_or_native_provider_reports() {
    let descriptors = RuntimePluginDescriptor::builtin_catalog();
    let profile = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client2d);

    let report = profile.availability_report_with_providers(
        descriptors.iter(),
        std::iter::empty::<&str>(),
        std::iter::empty::<&str>(),
    );

    assert!(contains_entry(&report.available, "ui"));
    assert!(contains_entry(&report.externalized_missing, "sound"));
    assert!(contains_entry(&report.externalized_missing, "rendering"));
    assert!(contains_entry(&report.missing_required, "sound"));
    assert!(contains_entry(&report.missing_required, "rendering"));
}

#[test]
fn linked_and_native_provider_reports_satisfy_profile_requirements() {
    let descriptors = RuntimePluginDescriptor::builtin_catalog();
    let profile = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client2d);

    let report =
        profile.availability_report_with_providers(descriptors.iter(), ["sound"], ["rendering"]);

    assert!(contains_entry(&report.linked, "sound"));
    assert!(contains_entry(&report.native_dynamic, "rendering"));
    assert!(!contains_entry(&report.missing_required, "sound"));
    assert!(!contains_entry(&report.missing_required, "rendering"));
}

#[test]
fn registration_reports_can_drive_profile_provider_availability() {
    let profile = RuntimeProfileDescriptor::new(
        RuntimeProfileId::Dev,
        "test.dev",
        RuntimeTargetMode::ClientRuntime,
    )
    .with_minimum_maturity(PluginMaturity::Beta)
    .with_default_plugin(RuntimePluginId::Sound, true)
    .with_default_plugin(RuntimePluginId::Rendering, true);
    let descriptors = RuntimePluginDescriptor::builtin_catalog();
    let linked = RuntimePluginRegistrationReport::from_plugin(
        &RuntimePluginDescriptor::new(
            "sound",
            "Sound",
            RuntimePluginId::Sound,
            "zircon_plugin_sound_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_maturity(PluginMaturity::Beta),
    );
    let mut native_manifest = descriptor(&descriptors, RuntimePluginId::Rendering)
        .package_manifest()
        .with_maturity(PluginMaturity::Stable);
    native_manifest.default_packaging = vec![crate::plugin::ExportPackagingStrategy::NativeDynamic];
    let native = RuntimePluginRegistrationReport::from_native_package_manifest(native_manifest);

    let report = profile
        .availability_report_for_registration_reports(descriptors.iter(), [&linked, &native]);

    assert!(contains_entry(&report.linked, "sound"));
    assert!(contains_entry(&report.native_dynamic, "rendering"));
    assert!(report.missing_required.is_empty());
}

#[test]
fn runtime_profile_module_loading_uses_linked_required_provider_reports() {
    let descriptors = RuntimePluginDescriptor::builtin_catalog();
    let sound = RuntimePluginRegistrationReport::from_plugin(descriptor(
        &descriptors,
        RuntimePluginId::Sound,
    ));
    let rendering = RuntimePluginRegistrationReport::from_plugin(descriptor(
        &descriptors,
        RuntimePluginId::Rendering,
    ));

    let report = crate::runtime_modules_for_runtime_profile_with_plugin_registration_reports(
        RuntimeProfileId::Client2d,
        [&sound, &rendering],
    );

    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert!(!report
        .required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::Sound));
    assert!(!report
        .required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::Rendering));
}

#[test]
fn minimal_runtime_profile_module_loading_does_not_inherit_legacy_target_defaults() {
    let report = crate::runtime_modules_for_runtime_profile(RuntimeProfileId::Minimal);
    let module_names = report
        .modules
        .iter()
        .map(|module| module.module_name())
        .collect::<Vec<_>>();

    assert!(!module_names.contains(&"UiModule"));
}

#[test]
fn runtime_profile_module_loading_keeps_missing_required_providers_fatal() {
    let report = crate::runtime_modules_for_runtime_profile(RuntimeProfileId::Client2d);

    assert!(report
        .required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::Sound));
    assert!(report
        .required_missing()
        .iter()
        .any(|missing| missing.id == RuntimePluginId::Rendering));
}

#[test]
fn provider_reports_do_not_bypass_maturity_gates() {
    let profile = RuntimeProfileDescriptor::new(
        RuntimeProfileId::Client2d,
        "test.client_2d",
        RuntimeTargetMode::ClientRuntime,
    )
    .with_minimum_maturity(PluginMaturity::Beta)
    .with_default_plugin(RuntimePluginId::Sound, true)
    .with_default_plugin(RuntimePluginId::Particles, true);
    let descriptors = [
        RuntimePluginDescriptor::new(
            "sound",
            "Sound",
            RuntimePluginId::Sound,
            "zircon_plugin_sound_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_maturity(PluginMaturity::Stub),
        RuntimePluginDescriptor::new(
            "particles",
            "Particles",
            RuntimePluginId::Particles,
            "zircon_plugin_particles_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_maturity(PluginMaturity::Experimental),
    ];

    let report = profile.availability_report_with_providers(
        descriptors.iter(),
        ["sound", "particles"],
        std::iter::empty::<&str>(),
    );

    assert!(contains_entry(&report.stub, "sound"));
    assert!(contains_entry(&report.blocked_by_maturity, "particles"));
    assert!(contains_entry(&report.missing_required, "sound"));
    assert!(contains_entry(&report.missing_required, "particles"));
    assert!(!contains_entry(&report.linked, "sound"));
    assert!(!contains_entry(&report.linked, "particles"));
}

#[test]
fn builtin_catalog_statuses_match_importer_and_physics_capability_metadata() {
    let catalog = RuntimePluginDescriptor::builtin_catalog();
    let physics = descriptor(&catalog, RuntimePluginId::Physics);

    for (id, capability) in [
        (
            RuntimePluginId::GltfImporter,
            "runtime.asset.importer.model.gltf",
        ),
        (
            RuntimePluginId::ObjImporter,
            "runtime.asset.importer.model.obj",
        ),
        (
            RuntimePluginId::TextureImporter,
            "runtime.asset.importer.texture.image",
        ),
        (
            RuntimePluginId::AudioImporter,
            "runtime.asset.importer.audio.wav",
        ),
        (
            RuntimePluginId::ShaderWgslImporter,
            "runtime.asset.importer.shader.wgsl",
        ),
        (
            RuntimePluginId::UiDocumentImporter,
            "runtime.asset.importer.ui_document",
        ),
    ] {
        let importer = descriptor(&catalog, id);
        assert!(
            importer.capability_statuses.iter().any(|status| {
                status.capability == capability && status.status == CapabilityStatus::Partial
            }),
            "missing importer capability status {capability}"
        );
    }
    assert!(physics.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.physics" && status.status == CapabilityStatus::Partial
    }));
    assert!(physics.capability_statuses.iter().any(|status| {
        status.capability == "runtime.capability.physics.raycast"
            && status.status == CapabilityStatus::Partial
    }));
}

fn descriptor(
    descriptors: &[RuntimePluginDescriptor],
    id: RuntimePluginId,
) -> &RuntimePluginDescriptor {
    descriptors
        .iter()
        .find(|descriptor| descriptor.runtime_id == id)
        .expect("descriptor")
}

fn contains_entry(entries: &[RuntimePluginAvailabilityEntry], id: &str) -> bool {
    entries.iter().any(|entry| entry.id == id)
}
