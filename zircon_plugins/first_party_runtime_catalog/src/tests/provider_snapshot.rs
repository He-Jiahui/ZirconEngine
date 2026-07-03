use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::plugin::{ProjectPluginManifest, ProjectPluginSelection};

#[test]
fn feature_enabled_first_party_provider_snapshot_reports_compiled_runtime_plugins() {
    let mut expected = Vec::new();
    let mut selections = Vec::new();
    #[cfg(feature = "base-runtime-plugins")]
    push_expected_runtime_provider(&mut expected, &mut selections, RuntimePluginId::Ai, "ai");
    #[cfg(feature = "base-runtime-plugins")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::Sound,
        "sound",
    );
    #[cfg(feature = "base-runtime-plugins")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::Texture,
        "texture",
    );
    #[cfg(feature = "base-runtime-plugins")]
    push_expected_runtime_provider(&mut expected, &mut selections, RuntimePluginId::Net, "net");
    #[cfg(feature = "navigation-runtime-plugin")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::Navigation,
        "navigation",
    );
    #[cfg(feature = "base-runtime-plugins")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::Particles,
        "particles",
    );
    #[cfg(feature = "base-runtime-plugins")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::Animation,
        "animation",
    );
    #[cfg(feature = "base-runtime-plugins")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::Rendering,
        "rendering",
    );
    #[cfg(feature = "base-runtime-plugins")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::GltfImporter,
        "gltf_importer",
    );
    #[cfg(feature = "advanced-render-runtime-plugins")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::VirtualGeometry,
        "virtual_geometry",
    );
    #[cfg(feature = "advanced-render-runtime-plugins")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::HybridGi,
        "hybrid_gi",
    );
    #[cfg(feature = "advanced-render-runtime-plugins")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::Solari,
        "solari",
    );
    #[cfg(feature = "zr-vm-language-runtime-plugin")]
    push_expected_runtime_provider(
        &mut expected,
        &mut selections,
        RuntimePluginId::ZrVmLanguage,
        "zr_vm_language",
    );

    let manifest = ProjectPluginManifest { selections };
    let reports = crate::first_party_runtime_plugin_registrations_for_manifest(
        RuntimeTargetMode::ClientRuntime,
        &manifest,
    );
    let actual = reports
        .iter()
        .map(|report| report.package_manifest.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        actual, expected,
        "first-party provider registration order drift"
    );
    for report in reports {
        assert_eq!(
            report.project_selection.id, report.package_manifest.id,
            "{} provider selection id must match package manifest id",
            report.package_manifest.id
        );
        assert!(
            report.diagnostics.is_empty(),
            "{} provider emitted diagnostics: {:?}",
            report.package_manifest.id,
            report.diagnostics
        );
    }
}

fn push_expected_runtime_provider(
    expected: &mut Vec<&'static str>,
    selections: &mut Vec<ProjectPluginSelection>,
    id: RuntimePluginId,
    package_id: &'static str,
) {
    expected.push(package_id);
    selections.push(
        ProjectPluginSelection::runtime_plugin(id, true, true)
            .with_target_modes([RuntimeTargetMode::ClientRuntime]),
    );
}
