use zircon_runtime::builtin::RuntimePluginId;
#[cfg(any(
    all(feature = "first-party-runtime-plugins", feature = "ui"),
    feature = "first-party-advanced-render-runtime-plugins",
    all(feature = "first-party-zr-vm-language-runtime-plugin", feature = "ui")
))]
use zircon_runtime::core::framework::project::RuntimeProfileId;
use zircon_runtime::core::framework::render::{RenderProductFeature, RenderProfileBundle};
#[cfg(any(
    all(feature = "first-party-runtime-plugins", feature = "ui"),
    feature = "first-party-advanced-render-runtime-plugins"
))]
use zircon_runtime::plugin::RuntimePluginRegistrationReport;
#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
use zircon_runtime::plugin::{CapabilityStatus, PluginMaturity};

#[cfg(any(
    all(feature = "first-party-runtime-plugins", feature = "ui"),
    feature = "first-party-advanced-render-runtime-plugins",
    all(feature = "first-party-zr-vm-language-runtime-plugin", feature = "ui")
))]
use crate::entry::{
    first_party_runtime_plugin_registrations_for_config, BuiltinEngineEntry, EntryRunner,
};
use crate::entry::{EntryConfig, EntryProfile};

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_bootstrap_uses_linked_first_party_provider_registrations() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d);
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);
    let ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(config.runtime_profile(), Some(RuntimeProfileId::Client2d));
    assert_eq!(
        ids,
        vec!["ui_document_importer", "sound", "rendering", "texture"]
    );
    assert!(registrations
        .iter()
        .all(RuntimePluginRegistrationReport::is_success));

    let entry = BuiltinEngineEntry::for_config(&config)
        .expect("plain client_2d bootstrap should use linked first-party registrations");
    let descriptors = entry.module_descriptors();

    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "sound.runtime"));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "rendering.runtime"));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "texture.runtime"));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "ui_document_importer.runtime"));
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_bootstrap_reports_missing_required_ui_document_importer() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d);
    let registrations = first_party_runtime_plugin_registrations_for_config(&config)
        .into_iter()
        .filter(|registration| {
            registration.package_manifest.id != RuntimePluginId::UiDocumentImporter.key()
        })
        .collect::<Vec<_>>();

    let error =
        BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(&config, registrations)
            .expect_err(
                "client_2d bootstrap must reject a missing required document importer provider",
            );

    assert!(error
        .to_string()
        .contains("required runtime plugin UiDocumentImporter is unavailable"));
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn first_party_sound_provider_preserves_manifest_maturity_and_capability_status() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d);
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);
    let sound = registrations
        .iter()
        .find(|registration| registration.package_manifest.id == "sound")
        .expect("client_2d linked providers should include sound");

    assert_eq!(sound.package_manifest.maturity, PluginMaturity::Beta);
    assert!(sound
        .package_manifest
        .capability_statuses
        .iter()
        .any(|status| {
            status.capability == "runtime.plugin.sound"
                && status.status == CapabilityStatus::Partial
        }));
    assert!(sound
        .extensions
        .modules()
        .iter()
        .any(|module| { module.name == "sound.runtime" }));
    assert!(sound
        .extensions
        .plugin_options()
        .iter()
        .any(|option| option.key == "sound.global_volume_gain"));
    assert!(sound
        .extensions
        .plugin_event_catalogs()
        .iter()
        .any(|catalog| catalog.namespace == "sound.dynamic_events"));
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_feature_bootstrap_uses_profile_level_provider_availability() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d);
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);

    let entry = BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
        &config,
        registrations,
        std::iter::empty::<zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport>(),
    )
    .expect("client_2d profile should use linked providers during feature-aware bootstrap");

    let descriptors = entry.module_descriptors();
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "sound.runtime"));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "rendering.runtime"));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "texture.runtime"));
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_bootstrap_can_link_optional_first_party_runtime_plugins() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d)
        .with_optional_runtime_plugins([
            RuntimePluginId::Animation,
            RuntimePluginId::Net,
            RuntimePluginId::Particles,
        ]);
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);
    let ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();

    for expected in [
        "sound",
        "rendering",
        "texture",
        "animation",
        "net",
        "particles",
    ] {
        assert!(
            ids.contains(&expected),
            "missing first-party registration {expected}"
        );
    }

    let entry =
        BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations(&config)
            .expect("optional first-party runtime plugin registrations should satisfy bootstrap");
    let descriptors = entry.module_descriptors();

    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "animation.runtime"));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "net.runtime"));
    assert!(descriptors
        .iter()
        .any(|descriptor| descriptor.name == "particles.runtime"));

    let feature_aware_entry =
        BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
            &config,
            registrations,
            std::iter::empty::<zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport>(),
        )
        .expect("feature-aware bootstrap should preserve profile manifest optional providers");
    let feature_aware_descriptors = feature_aware_entry.module_descriptors();
    let feature_aware_report = feature_aware_entry.module_selection_report();

    assert!(feature_aware_descriptors
        .iter()
        .any(|descriptor| descriptor.name == "animation.runtime"));
    assert!(feature_aware_descriptors
        .iter()
        .any(|descriptor| descriptor.name == "net.runtime"));
    assert!(feature_aware_descriptors
        .iter()
        .any(|descriptor| descriptor.name == "particles.runtime"));
    for expected in [
        RuntimePluginId::Animation,
        RuntimePluginId::Net,
        RuntimePluginId::Particles,
    ] {
        assert!(
            feature_aware_report
                .runtime_plugin_availability
                .linked
                .iter()
                .any(|entry| entry.runtime_id == expected),
            "feature-aware report should surface linked {expected:?}"
        );
    }
}

#[cfg(feature = "first-party-advanced-render-runtime-plugins")]
#[test]
fn render_profile_runtime_plugins_do_not_link_advanced_providers_for_default_render() {
    let config = EntryConfig::new(EntryProfile::Runtime);
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);

    assert!(registrations
        .iter()
        .all(|registration| registration.package_manifest.id != "virtual_geometry"));
    assert!(registrations
        .iter()
        .all(|registration| registration.package_manifest.id != "hybrid_gi"));
}

#[test]
fn entry_defaults_enable_hybrid_gi_only_for_editor_rendering() {
    let editor = EntryConfig::new(EntryProfile::Editor);
    let runtime = EntryConfig::new(EntryProfile::Runtime);

    assert!(
        editor
            .render_profile
            .has_feature(RenderProductFeature::HybridGlobalIllumination),
        "editor rendering should request Hybrid GI by default"
    );
    assert!(
        !editor
            .render_profile
            .has_feature(RenderProductFeature::VirtualGeometry),
        "editor Hybrid GI defaults should not implicitly opt into virtual geometry"
    );
    assert!(
        !runtime
            .render_profile
            .has_feature(RenderProductFeature::HybridGlobalIllumination),
        "client runtime rendering must keep Hybrid GI project opt-in"
    );
}

#[cfg(feature = "first-party-advanced-render-runtime-plugins")]
#[test]
fn editor_default_render_profile_links_hybrid_gi_without_virtual_geometry() {
    let registrations = first_party_runtime_plugin_registrations_for_config(&EntryConfig::new(
        EntryProfile::Editor,
    ));
    let ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();

    assert!(ids.contains(&"hybrid_gi"));
    assert!(!ids.contains(&"virtual_geometry"));
}

#[cfg(feature = "first-party-advanced-render-runtime-plugins")]
#[test]
fn render_profile_runtime_plugins_link_advanced_providers_when_advanced_render_is_selected() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_render_profile(RenderProfileBundle::advanced_render());
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);
    let ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();

    let advanced_ids = ids
        .iter()
        .copied()
        .filter(|id| matches!(*id, "virtual_geometry" | "hybrid_gi"))
        .collect::<Vec<_>>();
    assert_eq!(advanced_ids, vec!["virtual_geometry", "hybrid_gi"]);
    assert!(registrations
        .iter()
        .all(RuntimePluginRegistrationReport::is_success));
    assert_eq!(
        registrations
            .iter()
            .find(|registration| registration.package_manifest.id == "virtual_geometry")
            .expect("advanced render should link virtual geometry")
            .extensions
            .virtual_geometry_runtime_providers()
            .len(),
        1
    );
    assert_eq!(
        registrations
            .iter()
            .find(|registration| registration.package_manifest.id == "hybrid_gi")
            .expect("advanced render should link hybrid GI")
            .extensions
            .hybrid_gi_runtime_providers()
            .len(),
        1
    );

    let diagnostics =
        EntryRunner::module_selection_diagnostics_with_first_party_runtime_plugin_registrations(
            config,
        )
        .expect("advanced render provider registrations should satisfy diagnostics");

    assert!(diagnostics.contains("module=virtual_geometry.runtime"));
    assert!(diagnostics.contains("module=hybrid_gi.runtime"));
}

#[cfg(feature = "first-party-advanced-render-runtime-plugins")]
#[test]
fn render_profile_runtime_plugins_link_solari_provider_when_solari_experimental_is_selected() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_render_profile(RenderProfileBundle::solari_experimental());
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);
    let ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();

    let advanced_ids = ids
        .iter()
        .copied()
        .filter(|id| matches!(*id, "virtual_geometry" | "hybrid_gi" | "solari"))
        .collect::<Vec<_>>();
    assert_eq!(
        advanced_ids,
        vec!["virtual_geometry", "hybrid_gi", "solari"]
    );
    assert!(registrations
        .iter()
        .all(RuntimePluginRegistrationReport::is_success));
    assert_eq!(
        registrations
            .iter()
            .find(|registration| registration.package_manifest.id == "solari")
            .expect("Solari profile should link Solari")
            .extensions
            .solari_runtime_providers()
            .len(),
        1
    );

    let diagnostics =
        EntryRunner::module_selection_diagnostics_with_first_party_runtime_plugin_registrations(
            config,
        )
        .expect("solari experimental provider registrations should satisfy diagnostics");

    assert!(diagnostics.contains("module=virtual_geometry.runtime"));
    assert!(diagnostics.contains("module=hybrid_gi.runtime"));
    assert!(diagnostics.contains("module=solari.runtime"));
}

#[cfg(all(
    feature = "first-party-runtime-plugins",
    feature = "first-party-advanced-render-runtime-plugins",
    feature = "ui"
))]
#[test]
fn render_profile_runtime_plugins_merge_runtime_profile_baseline_with_advanced_providers() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d)
        .with_render_profile(RenderProfileBundle::advanced_render());
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);
    let ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();

    for expected in [
        "sound",
        "rendering",
        "texture",
        "virtual_geometry",
        "hybrid_gi",
    ] {
        assert!(
            ids.contains(&expected),
            "missing first-party registration {expected}"
        );
    }
}

#[cfg(all(
    feature = "first-party-runtime-plugins",
    feature = "first-party-navigation-runtime-plugin",
    feature = "ui"
))]
#[test]
fn runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d)
        .with_optional_runtime_plugins([RuntimePluginId::Navigation]);
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);

    assert!(registrations
        .iter()
        .any(|registration| registration.package_manifest.id == "navigation"));

    let entry =
        BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations(&config)
            .expect("navigation provider feature should contribute the first-party runtime module");

    assert!(entry
        .module_descriptors()
        .iter()
        .any(|descriptor| descriptor.name == "navigation.runtime"));
}

#[cfg(all(feature = "first-party-zr-vm-language-runtime-plugin", feature = "ui"))]
#[test]
fn runtime_profile_bootstrap_can_link_zr_vm_language_when_provider_feature_is_enabled() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d)
        .with_optional_runtime_plugins([RuntimePluginId::ZrVmLanguage]);
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);

    assert!(registrations
        .iter()
        .any(|registration| registration.package_manifest.id == "zr_vm_language"));

    let entry =
        BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations(&config)
            .expect("ZrVM provider feature should contribute the first-party runtime module");

    assert!(entry
        .module_descriptors()
        .iter()
        .any(|descriptor| descriptor.name == "zr_vm_language.runtime"));
}
