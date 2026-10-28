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

#[cfg(feature = "base-runtime-plugins")]
#[test]
fn runtime_catalog_projects_selected_audio_wav_importer() {
    use zircon_runtime::asset::{AssetImportContext, AssetKind, AssetUri, ImportedAsset};
    use zircon_runtime::builtin::RuntimePluginId;
    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

    for target_mode in [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ] {
        let manifest = ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::AudioImporter,
                true,
                true,
            )
            .with_target_modes([target_mode])],
        };
        let registrations =
            crate::first_party_runtime_plugin_registrations_for_manifest(target_mode, &manifest)
                .into_registrations_if_required_resolved()
                .expect("selected required audio importer resolves");

        assert_eq!(registrations.len(), 1);
        let registration = &registrations[0];
        assert_eq!(
            registration.package_manifest.id,
            RuntimePluginId::AudioImporter.key()
        );
        assert!(
            registration.diagnostics.is_empty(),
            "{:?}",
            registration.diagnostics
        );
        assert!(registration
            .package_manifest
            .capabilities
            .iter()
            .any(|capability| capability == "runtime.asset.importer.audio.wav"));

        let importer = registration
            .extensions
            .asset_importers()
            .select(std::path::Path::new("tone.wav"))
            .expect("linked audio importer handles WAV sources");
        let descriptor = importer.descriptor();
        assert_eq!(descriptor.id, "audio_importer.wav");
        assert_eq!(descriptor.plugin_id, RuntimePluginId::AudioImporter.key());
        assert_eq!(descriptor.output_kind, AssetKind::Sound);
        assert_eq!(
            descriptor.required_capabilities,
            ["runtime.asset.importer.audio.wav"]
        );

        // A complete PCM16 mono WAV: two frames at 8 kHz, with no external fixture.
        let wav = b"RIFF\x28\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0\x40\x1f\0\0\x80\x3e\0\0\x02\0\x10\0data\x04\0\0\0\0\0\xff\x7f";
        let context = AssetImportContext::new(
            "tone.wav".into(),
            AssetUri::parse("res://audio/tone.wav").expect("valid WAV asset URI"),
            wav.to_vec(),
            Default::default(),
        );
        let outcome = importer
            .import(&context)
            .expect("linked WAV importer decodes PCM");
        match &outcome.root_entry().expect("root sound asset").asset {
            ImportedAsset::Sound(sound) => {
                assert_eq!(sound.sample_rate_hz, 8_000);
                assert_eq!(sound.channel_count, 1);
                assert_eq!(sound.frame_count(), 2);
            }
            other => panic!("expected imported Sound asset, got {other:?}"),
        }
    }
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
