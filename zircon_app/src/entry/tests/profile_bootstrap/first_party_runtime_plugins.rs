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
    first_party_runtime_plugin_registrations_for_config, BuiltinEngineEntry, EngineEntry,
    EntryRunner,
};
use crate::entry::{EntryConfig, EntryProfile};

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
fn linked_feature_registrations(
    config: &EntryConfig,
) -> Vec<zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport> {
    let config = config
        .resolve()
        .expect("runtime plugin request should resolve");
    let manifest = crate::entry::builtin_modules::effective_project_plugin_manifest(&config);
    crate::entry::first_party_runtime_plugins::first_party_runtime_catalog_for_manifest(
        config.target_mode(),
        &manifest,
    )
    .runtime_plugin_features
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_bootstrap_uses_linked_first_party_provider_registrations() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d);
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );
    let ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        config
            .resolve()
            .expect("client_2d request should resolve")
            .runtime_profile(),
        Some(RuntimeProfileId::Client2d)
    );
    assert_eq!(
        ids,
        vec!["ui_document_importer", "sound", "rendering", "texture"],
        "actual linked first-party registrations: {ids:?}"
    );
    assert!(registrations
        .iter()
        .all(RuntimePluginRegistrationReport::is_success));

    let entry = BuiltinEngineEntry::for_config(&config)
        .expect("plain client_2d bootstrap should use linked first-party registrations");
    let descriptors = entry.module_descriptors();

    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "sound.runtime"),
        "actual runtime modules: {:?}",
        descriptors
            .iter()
            .map(|descriptor| descriptor.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "rendering.runtime"),
        "actual runtime modules: {:?}",
        descriptors
            .iter()
            .map(|descriptor| descriptor.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "texture.runtime"),
        "actual runtime modules: {:?}",
        descriptors
            .iter()
            .map(|descriptor| descriptor.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "ui_document_importer.runtime"),
        "actual runtime modules: {:?}",
        descriptors
            .iter()
            .map(|descriptor| descriptor.name.as_str())
            .collect::<Vec<_>>()
    );
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_bootstrap_reports_missing_required_ui_document_importer() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d);
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    )
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

    assert!(
        matches!(
            &error,
            zircon_runtime::core::CoreError::Initialization(context, reason)
                if context == "zircon_app explicit runtime plugin selection"
                    && reason.contains("required plugin selection `ui_document_importer` is Unsupported")
        ),
        "actual bootstrap rejection: {error}"
    );
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn first_party_sound_provider_preserves_manifest_maturity_and_capability_status() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d);
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );
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
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );

    let entry = BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
        &config,
        registrations,
        linked_feature_registrations(&config),
    )
    .expect("client_2d profile should use linked providers during feature-aware bootstrap");

    let descriptors = entry.module_descriptors();
    let module_names = descriptors
        .iter()
        .map(|descriptor| descriptor.name.as_str())
        .collect::<Vec<_>>();
    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "sound.runtime"),
        "actual runtime modules: {module_names:?}"
    );
    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "rendering.runtime"),
        "actual runtime modules: {module_names:?}"
    );
    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "texture.runtime"),
        "actual runtime modules: {module_names:?}"
    );
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_bootstrap_can_link_optional_first_party_runtime_plugins() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d)
        .with_runtime_plugin_minimum_maturity(PluginMaturity::Experimental)
        .with_optional_runtime_plugins([
            RuntimePluginId::Animation,
            RuntimePluginId::Net,
            RuntimePluginId::Particles,
        ]);
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );
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
            "missing first-party registration {expected}; actual registration IDs: {ids:?}"
        );
    }

    let entry =
        BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations(&config)
            .unwrap_or_else(|error| {
                panic!("optional first-party registrations failed bootstrap: {error}")
            });
    let descriptors = entry.module_descriptors();

    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "animation.runtime"),
        "actual runtime modules: {:?}",
        descriptors
            .iter()
            .map(|descriptor| descriptor.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "net.runtime"),
        "actual runtime modules: {:?}",
        descriptors
            .iter()
            .map(|descriptor| descriptor.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        descriptors
            .iter()
            .any(|descriptor| descriptor.name == "particles.runtime"),
        "actual runtime modules: {:?}",
        descriptors
            .iter()
            .map(|descriptor| descriptor.name.as_str())
            .collect::<Vec<_>>()
    );

    let feature_aware_entry =
        BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
            &config,
            registrations,
            linked_feature_registrations(&config),
        )
        .unwrap_or_else(|error| {
            panic!("feature-aware bootstrap rejected optional providers: {error}")
        });
    let feature_aware_descriptors = feature_aware_entry.module_descriptors();
    let feature_aware_report = feature_aware_entry.module_selection_report();
    let feature_aware_module_names = feature_aware_descriptors
        .iter()
        .map(|descriptor| descriptor.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        feature_aware_descriptors
            .iter()
            .any(|descriptor| descriptor.name == "animation.runtime"),
        "actual feature-aware runtime modules: {feature_aware_module_names:?}"
    );
    assert!(
        feature_aware_descriptors
            .iter()
            .any(|descriptor| descriptor.name == "net.runtime"),
        "actual feature-aware runtime modules: {feature_aware_module_names:?}"
    );
    assert!(
        feature_aware_descriptors
            .iter()
            .any(|descriptor| descriptor.name == "particles.runtime"),
        "actual feature-aware runtime modules: {feature_aware_module_names:?}"
    );
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
            "feature-aware report should surface linked {expected:?}; actual availability: {:?}",
            feature_aware_report.runtime_plugin_availability
        );
    }
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn ordinary_first_party_bootstrap_admits_selected_net_http_and_websocket_features() {
    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::{
        ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
    };

    let config_file = super::IsolatedConfigFile::new();
    let net = ProjectPluginSelection::runtime_plugin(RuntimePluginId::Net, true, true)
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_feature(ProjectPluginFeatureSelection::new("net.http").required(true))
        .with_feature(ProjectPluginFeatureSelection::new("net.websocket").required(true));
    let config =
        EntryConfig::new(EntryProfile::Runtime).with_project_plugins(ProjectPluginManifest {
            selections: vec![net],
        });

    let entry = BuiltinEngineEntry::for_config(&config)
        .expect("ordinary first-party bootstrap should admit selected net feature providers")
        .with_config_file_path(config_file.path())
        .expect("the host config path should bind to Foundation");
    let module_names = entry
        .module_descriptors()
        .into_iter()
        .map(|descriptor| descriptor.name)
        .collect::<Vec<_>>();

    assert!(
        module_names.iter().any(|name| name == "net.runtime"),
        "actual selected modules: {module_names:?}"
    );
    assert!(
        module_names.iter().any(|name| name == "net.http.runtime"),
        "actual selected modules: {module_names:?}"
    );
    assert!(
        module_names
            .iter()
            .any(|name| name == "net.websocket.runtime"),
        "actual selected modules: {module_names:?}"
    );

    let runtime = zircon_runtime::core::CoreRuntime::new();
    entry
        .bootstrap(&runtime)
        .expect("ordinary bootstrap should activate selected HTTP and WebSocket modules");
    let _core = runtime.handle();
    {}
    drop(_core);
    super::super::product_composition::close_owner::close_runtime(&runtime);
}

#[cfg(feature = "first-party-advanced-render-runtime-plugins")]
#[test]
fn render_profile_runtime_plugins_do_not_link_advanced_providers_for_default_render() {
    let config = EntryConfig::new(EntryProfile::Runtime);
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );

    assert!(registrations
        .iter()
        .all(|registration| registration.package_manifest.id != "virtual_geometry"));
    assert!(registrations
        .iter()
        .all(|registration| registration.package_manifest.id != "hybrid_gi"));
}

#[test]
fn entry_defaults_enable_hybrid_gi_only_for_editor_rendering() {
    let editor = EntryConfig::new(EntryProfile::Editor)
        .resolve()
        .expect("default editor configuration should resolve");
    let runtime = EntryConfig::new(EntryProfile::Runtime)
        .resolve()
        .expect("default runtime configuration should resolve");

    assert!(
        editor
            .render_profile()
            .has_feature(RenderProductFeature::HybridGlobalIllumination),
        "editor rendering should request Hybrid GI by default"
    );
    assert!(
        !editor
            .render_profile()
            .has_feature(RenderProductFeature::VirtualGeometry),
        "editor Hybrid GI defaults should not implicitly opt into virtual geometry"
    );
    assert!(
        !runtime
            .render_profile()
            .has_feature(RenderProductFeature::HybridGlobalIllumination),
        "client runtime rendering must keep Hybrid GI project opt-in"
    );
}

#[cfg(feature = "first-party-advanced-render-runtime-plugins")]
#[test]
fn editor_default_render_profile_links_hybrid_gi_without_virtual_geometry() {
    let config = EntryConfig::new(EntryProfile::Editor)
        .resolve()
        .expect("default editor configuration should resolve");
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);
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
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );
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

    let diagnostics = EntryRunner::module_selection_diagnostics(config)
        .expect("advanced render provider registrations should satisfy diagnostics");

    assert!(diagnostics.contains("module=virtual_geometry.runtime"));
    assert!(diagnostics.contains("module=hybrid_gi.runtime"));
}

#[cfg(feature = "first-party-advanced-render-runtime-plugins")]
#[test]
fn render_profile_runtime_plugins_link_solari_provider_when_solari_experimental_is_selected() {
    let config = EntryConfig::new(EntryProfile::Runtime)
        .with_render_profile(RenderProfileBundle::solari_experimental());
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );
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

    let diagnostics = EntryRunner::module_selection_diagnostics(config)
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
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );
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
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );

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
    let registrations = first_party_runtime_plugin_registrations_for_config(
        &config
            .resolve()
            .expect("runtime plugin request should resolve"),
    );

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

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_default_maturity_blocks_optional_particles_before_module_and_extension_admission(
) {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d)
        .with_optional_runtime_plugins([RuntimePluginId::Particles]);
    let resolved = config.resolve().expect("Client3d request should resolve");
    let registrations = first_party_runtime_plugin_registrations_for_config(&resolved);
    let entries = [
        BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations(&config)
            .expect("optional blocked provider should not reject Client3d"),
        BuiltinEngineEntry::for_config_with_runtime_plugin_and_feature_registrations(
            &config,
            registrations,
            linked_feature_registrations(&config),
        )
        .expect("both constructors must share the same admission"),
    ];
    for entry in entries {
        assert!(!entry
            .module_descriptors()
            .iter()
            .any(|module| module.name == "particles.runtime"));
        assert!(!entry
            .module_selection_report()
            .runtime_plugin_availability
            .linked
            .iter()
            .any(|row| row.runtime_id == RuntimePluginId::Particles));
        assert!(entry
            .runtime_plugin_availability()
            .blocked_by_maturity
            .iter()
            .any(|row| row.runtime_id == RuntimePluginId::Particles));
        let plan = entry
            .compiled_project_plugin_plan()
            .expect("entry must retain its plan");
        assert_eq!(
            plan.runtime_profile_admission(),
            Some((RuntimeProfileId::Client3d, PluginMaturity::Beta))
        );
        assert!(plan
            .completed_manifest()
            .selections
            .iter()
            .any(|selection| selection.id == "particles" && selection.enabled));
        assert!(!plan
            .linked_provider_package_ids()
            .iter()
            .any(|id| id == "particles"));
        assert!(!plan
            .module_proposals()
            .iter()
            .any(|proposal| proposal.provider_package_id() == "particles"));
        assert!(!plan
            .runtime_extensions()
            .registry
            .plugin_options()
            .iter()
            .any(|option| option.key.starts_with("particles.")));
        assert!(!plan
            .runtime_extensions()
            .registry
            .plugin_event_catalogs()
            .iter()
            .any(|catalog| catalog.namespace == "particles.dynamic_events"));
        let lifecycle = entry
            .runtime_plugin_bridge_lifecycle_state()
            .expect("entry must retain its lifecycle snapshot");
        assert!(std::ptr::eq(
            lifecycle.extension_report(),
            plan.runtime_extensions()
        ));
    }
    let allowed = config.with_runtime_plugin_minimum_maturity(PluginMaturity::Experimental);
    let entry =
        BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations(&allowed)
            .expect("explicit Experimental admission should allow the real provider");
    assert!(entry
        .module_descriptors()
        .iter()
        .any(|module| module.name == "particles.runtime"));
    let plan = entry.compiled_project_plugin_plan().expect("allowed plan");
    assert!(plan
        .runtime_extensions()
        .registry
        .plugin_options()
        .iter()
        .any(|option| option.key == "particles.backend"));
    assert!(plan
        .runtime_extensions()
        .registry
        .plugin_event_catalogs()
        .iter()
        .any(|catalog| catalog.namespace == "particles.dynamic_events"));
    assert!(std::ptr::eq(
        entry
            .runtime_plugin_bridge_lifecycle_state()
            .unwrap()
            .extension_report(),
        plan.runtime_extensions()
    ));
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_plan_cache_and_composition_identity_include_applied_maturity() {
    use zircon_runtime::builtin::RuntimeModuleCompositionCompiler;
    use zircon_runtime::plugin::{RuntimePluginCatalog, RuntimeProfileDescriptor};
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d)
        .with_optional_runtime_plugins([RuntimePluginId::Particles]);
    let resolved = config.resolve().expect("profile request");
    let manifest = crate::entry::builtin_modules::effective_project_plugin_manifest(&resolved);
    let catalog = RuntimePluginCatalog::from_registration_reports(
        first_party_runtime_plugin_registrations_for_config(&resolved),
        linked_feature_registrations(&config),
    );
    let beta_profile = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client3d);
    let experimental_profile = beta_profile
        .clone()
        .with_minimum_maturity(PluginMaturity::Experimental);
    let beta = catalog.compiled_project_plan_for_runtime_profile(&manifest, &beta_profile);
    let repeated = catalog.compiled_project_plan_for_runtime_profile(&manifest, &beta_profile);
    let experimental =
        catalog.compiled_project_plan_for_runtime_profile(&manifest, &experimental_profile);
    assert!(std::sync::Arc::ptr_eq(&beta, &repeated));
    assert!(!std::sync::Arc::ptr_eq(&beta, &experimental));
    assert_eq!(
        beta.source_manifest_fingerprint(),
        experimental.source_manifest_fingerprint()
    );
    assert_eq!(beta.completed_manifest(), experimental.completed_manifest());
    assert!(!beta
        .linked_provider_package_ids()
        .iter()
        .any(|id| id == "particles"));
    assert!(experimental
        .linked_provider_package_ids()
        .iter()
        .any(|id| id == "particles"));
    let beta_composition = RuntimeModuleCompositionCompiler::new(&beta)
        .compile()
        .expect("Beta plan");
    let experimental_composition = RuntimeModuleCompositionCompiler::new(&experimental)
        .compile()
        .expect("Experimental plan");
    assert_ne!(
        beta_composition.identity().composition_hash(),
        experimental_composition.identity().composition_hash()
    );
    assert!(RuntimeModuleCompositionCompiler::new(&experimental)
        .for_runtime_profile(RuntimeProfileId::Client3d)
        .compile()
        .is_err());
    assert!(RuntimeModuleCompositionCompiler::new(&beta)
        .for_runtime_profile(RuntimeProfileId::Client2d)
        .compile()
        .is_err());
    let target_only = catalog.compiled_project_plan(&manifest, resolved.target_mode());
    assert_eq!(target_only.runtime_profile_admission(), None);
    assert!(RuntimeModuleCompositionCompiler::new(&target_only)
        .compile()
        .is_ok());
    assert!(RuntimeModuleCompositionCompiler::new(&target_only)
        .for_runtime_profile(RuntimeProfileId::Client3d)
        .compile()
        .is_err());
    // An identical executable graph still has a distinct admitted policy identity.
    let empty = RuntimePluginCatalog::from_registration_reports([], []);
    let empty_manifest = zircon_runtime::core::framework::project::ProjectPluginManifest::default();
    let minimal = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Minimal)
        .with_minimum_maturity(PluginMaturity::Core);
    let one = empty.compiled_project_plan_for_runtime_profile(&empty_manifest, &minimal);
    let two = empty.compiled_project_plan_for_runtime_profile(
        &empty_manifest,
        &minimal.clone().with_minimum_maturity(PluginMaturity::Beta),
    );
    assert_eq!(
        one.runtime_profile_admission(),
        Some((RuntimeProfileId::Minimal, PluginMaturity::Core))
    );
    assert_eq!(
        two.runtime_profile_admission(),
        Some((RuntimeProfileId::Minimal, PluginMaturity::Beta))
    );
    assert!(!std::sync::Arc::ptr_eq(&one, &two));
    let repeated_minimal_beta = empty.compiled_project_plan_for_runtime_profile(
        &empty_manifest,
        &minimal.clone().with_minimum_maturity(PluginMaturity::Beta),
    );
    assert!(std::sync::Arc::ptr_eq(&two, &repeated_minimal_beta));
    let one = RuntimeModuleCompositionCompiler::new(&one)
        .compile()
        .expect("minimal policy one");
    let two = RuntimeModuleCompositionCompiler::new(&two)
        .compile()
        .expect("minimal policy two");
    assert_eq!(
        one.modules()
            .iter()
            .map(|module| module.module_name())
            .collect::<Vec<_>>(),
        two.modules()
            .iter()
            .map(|module| module.module_name())
            .collect::<Vec<_>>()
    );
    assert_ne!(
        one.identity().composition_hash(),
        two.identity().composition_hash()
    );
}

#[cfg(all(feature = "first-party-runtime-plugins", feature = "ui"))]
#[test]
fn runtime_profile_required_experimental_provider_remains_a_fatal_requirement_at_beta() {
    let config = EntryConfig::for_runtime_profile(RuntimeProfileId::Client3d)
        .with_required_runtime_plugins([RuntimePluginId::Particles]);
    let result =
        BuiltinEngineEntry::for_config_with_first_party_runtime_plugin_registrations(&config);
    assert!(
        result.is_err(),
        "a blocked required provider must fail before activation"
    );
}
