use super::{feature_provider_selection, sound_registration};
use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};
use crate::plugin::{
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest,
    PluginPackageManifest, RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

#[test]
fn excludes_modules_from_disabled_plugin_registrations() {
    let mut registration = sound_registration();
    registration.project_selection.enabled = false;
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifest = sound_manifest();

    let plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(plan.module_proposals().is_empty());
    assert!(plan.runtime_extensions().registry.modules().is_empty());
}

#[test]
fn does_not_reenable_an_explicitly_disabled_alias_package() {
    let audio_alias_registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("audio", "Audio Alias").with_runtime_module(
            PluginModuleManifest::runtime("audio.alias.runtime", "audio_alias_runtime")
                .with_target_modes([RuntimeTargetMode::ClientRuntime]),
        ),
    );
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration(), audio_alias_registration],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false),
            feature_provider_selection("audio", false),
        ],
    };

    let plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);
    let module_names = proposal_module_names(&plan);

    assert_eq!(module_names, ["sound.runtime"]);
}

#[test]
fn disabled_plugin_registration_cannot_satisfy_dependent_feature_capabilities() {
    let mut registration = sound_registration();
    registration.project_selection.enabled = false;
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration],
        [sound_spatial_registration("sound.spatial.runtime", None)],
    );
    let manifest = sound_manifest_with_spatial_feature();

    assert_dependent_feature_is_excluded(&catalog, &manifest);
}

#[test]
fn target_incompatible_plugin_registration_cannot_satisfy_dependent_features() {
    let mut registration = sound_registration();
    registration.project_selection.target_modes = vec![RuntimeTargetMode::EditorHost];
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration],
        [sound_spatial_registration("sound.spatial.runtime", None)],
    );
    let manifest = sound_manifest_with_spatial_feature();

    assert_dependent_feature_is_excluded(&catalog, &manifest);
}

#[test]
fn unregistered_package_cannot_borrow_another_plugins_capability() {
    let ghost_dependent_feature = sound_spatial_feature(
        "Sound Spatial",
        "sound.spatial.runtime",
        "sound_spatial_runtime",
    )
    .with_dependency(PluginFeatureDependency::required(
        "ghost",
        "runtime.plugin.sound",
    ));
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        [
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
                ghost_dependent_feature,
                None,
            ),
        ],
    );
    let mut manifest = sound_manifest_with_spatial_feature();
    manifest
        .selections
        .push(feature_provider_selection("ghost", true));

    assert_dependent_feature_is_excluded(&catalog, &manifest);
}

#[test]
fn target_plan_excludes_runtime_modules_for_other_targets() {
    let mut registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("mixed_target", "Mixed Target")
            .with_supported_targets([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::ServerRuntime,
            ])
            .with_runtime_module(
                PluginModuleManifest::runtime("mixed.client.runtime", "mixed_client_runtime")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime]),
            )
            .with_runtime_module(
                PluginModuleManifest::runtime("mixed.server.runtime", "mixed_server_runtime")
                    .with_target_modes([RuntimeTargetMode::ServerRuntime]),
            ),
    );
    registration
        .diagnostics
        .push("mixed target diagnostic".to_string());
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifest = ProjectPluginManifest {
        selections: vec![feature_provider_selection("mixed_target", true)],
    };

    let plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);
    let proposal_modules = proposal_module_names(&plan);
    let registry_modules = plan
        .runtime_extensions()
        .registry
        .modules()
        .iter()
        .map(|descriptor| descriptor.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(proposal_modules, ["mixed.client.runtime"]);
    assert_eq!(registry_modules, ["mixed.client.runtime"]);
    assert_eq!(
        plan.runtime_extensions()
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.contains("mixed target diagnostic"))
            .count(),
        1
    );
}

#[test]
fn proposes_and_merges_only_the_selected_feature_provider_target_modules() {
    let feature_a = sound_spatial_feature(
        "Sound Spatial A",
        "sound.spatial.a.runtime",
        "sound_spatial_a_runtime",
    );
    let feature_b = sound_spatial_feature(
        "Sound Spatial B",
        "sound.spatial.b.runtime",
        "sound_spatial_b_runtime",
    )
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.spatial.b.server.runtime",
            "sound_spatial_b_server_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ServerRuntime]),
    );
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        [
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
                feature_a,
                Some("sound_spatial_a".to_string()),
            ),
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
                feature_b,
                Some("sound_spatial_b".to_string()),
            ),
        ],
    );
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.spatial")
                        .enabled(true)
                        .with_provider_package_id("sound_spatial_b"),
                ),
            feature_provider_selection("sound_spatial_b", true),
        ],
    };

    let plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);
    let feature_modules = plan
        .module_proposals()
        .iter()
        .filter(|proposal| proposal.feature_id().is_some())
        .map(|proposal| {
            (
                proposal.provider_package_id(),
                proposal.feature_id(),
                proposal.descriptor().name.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let registry_feature_modules = plan
        .runtime_extensions()
        .registry
        .modules()
        .iter()
        .filter(|descriptor| descriptor.name.starts_with("sound.spatial."))
        .map(|descriptor| descriptor.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        feature_modules,
        vec![(
            "sound_spatial_b",
            Some("sound.spatial"),
            "sound.spatial.b.runtime",
        )]
    );
    assert_eq!(registry_feature_modules, ["sound.spatial.b.runtime"]);
}

fn sound_manifest() -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    }
}

fn sound_manifest_with_spatial_feature() -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(ProjectPluginFeatureSelection::new("sound.spatial").enabled(true))],
    }
}

fn sound_spatial_registration(
    module_name: &str,
    provider_package_id: Option<String>,
) -> RuntimePluginFeatureRegistrationReport {
    RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        sound_spatial_feature("Sound Spatial", module_name, "sound_spatial_runtime"),
        provider_package_id,
    )
}

fn sound_spatial_feature(
    display_name: &str,
    module_name: &str,
    runtime_crate: &str,
) -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new("sound.spatial", display_name, "sound")
        .with_dependency(PluginFeatureDependency::primary(
            "sound",
            "runtime.plugin.sound",
        ))
        .with_runtime_module(
            PluginModuleManifest::runtime(module_name, runtime_crate)
                .with_target_modes([RuntimeTargetMode::ClientRuntime]),
        )
}

fn proposal_module_names(plan: &crate::plugin::CompiledProjectPluginPlan) -> Vec<&str> {
    plan.module_proposals()
        .iter()
        .map(|proposal| proposal.descriptor().name.as_str())
        .collect()
}

fn assert_dependent_feature_is_excluded(
    catalog: &RuntimePluginCatalog,
    manifest: &ProjectPluginManifest,
) {
    let plan = catalog.compiled_project_plan(manifest, RuntimeTargetMode::ClientRuntime);

    assert!(plan
        .feature_dependency_report()
        .available_features
        .is_empty());
    assert!(plan.module_proposals().is_empty());
    assert!(plan.runtime_extensions().registry.modules().is_empty());
}
