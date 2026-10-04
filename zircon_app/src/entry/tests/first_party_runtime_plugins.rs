use zircon_runtime::core::framework::project::{
    PluginSelectionResolutionStatus, ProjectPluginManifest, ProjectPluginSelection,
};
use zircon_runtime::core::framework::render::RenderProductFeature;

use super::*;
use crate::entry::builtin_modules::render_profile_runtime_plugin_overlay;

#[test]
fn prepared_product_retains_optional_unsupported_selection_diagnostics() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection {
            id: "optional-unlinked-provider".to_owned(),
            enabled: true,
            required: false,
            target_modes: vec![RuntimeTargetMode::ServerRuntime],
            packaging:
                zircon_runtime::core::framework::project::ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: None,
            editor_crate: None,
            features: Vec::new(),
        }],
    };
    let config = crate::entry::EntryConfig::new(crate::entry::EntryProfile::Headless)
        .with_project_plugins(manifest);
    let prepared = crate::entry::BuiltinEngineEntry::for_config(&config)
        .expect("optional unlinked provider must not block product preparation");
    let report = prepared.module_selection_report();
    assert!(report.plugin_selection_outcomes.iter().any(|outcome| {
        outcome.selection.id == "optional-unlinked-provider"
            && outcome.status == PluginSelectionResolutionStatus::Unsupported
    }));
    assert!(report.format_diagnostics().contains(
        "entry.plugin_selection=optional-unlinked-provider status=Unsupported required=false"
    ));
}

#[test]
fn required_invalid_selection_is_preserved_and_rejected() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection {
            id: String::new(),
            enabled: true,
            required: true,
            target_modes: vec![RuntimeTargetMode::ClientRuntime],
            packaging:
                zircon_runtime::core::framework::project::ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: None,
            editor_crate: None,
            features: Vec::new(),
        }],
    };
    let report = first_party_runtime_plugin_registrations_for_manifest(
        RuntimeTargetMode::ClientRuntime,
        &manifest,
    );

    assert_eq!(
        report.outcomes()[0].status,
        PluginSelectionResolutionStatus::InvalidId
    );
    assert!(report.into_registrations_if_required_resolved().is_err());
}

#[test]
fn render_profile_overlay_omits_a_project_selected_runtime_plugin() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::HybridGi,
            true,
            false,
        )],
    };
    let render_profile = RenderProfileBundle::default_render()
        .with_features([RenderProductFeature::HybridGlobalIllumination]);

    let overlay = render_profile_runtime_plugin_overlay(
        &manifest,
        RuntimeTargetMode::EditorHost,
        &render_profile,
    );

    assert!(overlay.selections.is_empty());
}

#[test]
fn render_profile_overlay_adds_only_the_missing_runtime_plugin() {
    let render_profile = RenderProfileBundle::default_render()
        .with_features([RenderProductFeature::HybridGlobalIllumination]);

    let overlay = render_profile_runtime_plugin_overlay(
        &ProjectPluginManifest::default(),
        RuntimeTargetMode::EditorHost,
        &render_profile,
    );

    assert_eq!(overlay.selections.len(), 1);
    assert_eq!(overlay.selections[0].id, RuntimePluginId::HybridGi.key());
    assert_eq!(
        overlay.selections[0].target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
}

#[cfg(feature = "first-party-ui-document-importer")]
#[test]
fn ui_product_baseline_projects_one_document_importer_provider() {
    let config = crate::entry::EntryConfig::new(crate::entry::EntryProfile::Runtime)
        .resolve()
        .expect("runtime product host config should resolve");
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);

    assert_eq!(
        registrations
            .iter()
            .filter(|registration| {
                registration.package_manifest.id == RuntimePluginId::UiDocumentImporter.key()
            })
            .count(),
        1
    );
}

#[cfg(feature = "first-party-ui-document-importer")]
#[test]
fn project_manifest_can_disable_the_linked_document_importer_provider() {
    let config = crate::entry::EntryConfig::new(crate::entry::EntryProfile::Runtime)
        .with_project_plugins(ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::UiDocumentImporter,
                false,
                false,
            )],
        })
        .resolve()
        .expect("runtime product host config should resolve");
    let registrations = first_party_runtime_plugin_registrations_for_config(&config);

    assert!(registrations.iter().all(|registration| {
        registration.package_manifest.id != RuntimePluginId::UiDocumentImporter.key()
    }));
}
