#[test]
fn runtime_catalog_preallocates_manifest_projection_storage() {
    let source = include_str!("../lib.rs");
    let projection = source
        .split("pub fn first_party_runtime_plugin_registrations_for_manifest")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn first_party_registration_for_runtime_plugin")
                .next()
        })
        .expect("read runtime catalog manifest projection");

    assert!(projection.contains("resolve_plugin_selections(target_mode, manifest"));
}

#[cfg(feature = "ui-document-importer")]
#[test]
fn runtime_catalog_projects_the_selected_ui_document_importer_provider() {
    use zircon_runtime::builtin::RuntimePluginId;
    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::UiDocumentImporter,
            true,
            true,
        )],
    };
    let registrations = crate::first_party_runtime_plugin_registrations_for_manifest(
        RuntimeTargetMode::ClientRuntime,
        &manifest,
    )
    .into_registrations_if_required_resolved()
    .expect("selected required UI document importer resolves");

    assert_eq!(registrations.len(), 1);
    assert_eq!(
        registrations[0].package_manifest.id,
        RuntimePluginId::UiDocumentImporter.key()
    );
}
