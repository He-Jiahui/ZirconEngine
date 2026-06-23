use super::*;

#[test]
fn importer_registry_priority_overrides_duplicate_extension() {
    let root = unique_temp_project_root("registry_priority");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("payload.testdata");
    fs::write(&path, "payload").unwrap();
    let uri = AssetUri::parse("res://data/payload.testdata").unwrap();

    let mut registry = AssetImporterRegistry::default();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.low", "test", crate::asset::AssetKind::Data, 1)
                .with_source_extensions(["testdata"])
                .with_priority(0),
            |context| test_data_outcome(context, "low"),
        ))
        .unwrap();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.high", "test", crate::asset::AssetKind::Data, 1)
                .with_source_extensions(["testdata"])
                .with_priority(10),
            |context| test_data_outcome(context, "high"),
        ))
        .unwrap();

    let imported = AssetImporter::with_registry(registry)
        .import_from_source(&path, &uri)
        .unwrap();

    match imported {
        ImportedAsset::Data(data) => assert_eq!(data.canonical_json["winner"], "high"),
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_registry_prefers_available_extension_importer_over_higher_priority_diagnostic() {
    let root = unique_temp_project_root("registry_available_over_diagnostic_extension");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("payload.profileaudio");
    fs::write(&path, "payload").unwrap();
    let uri = AssetUri::parse("res://data/payload.profileaudio").unwrap();

    let mut registry = AssetImporterRegistry::default();
    registry
        .register(DiagnosticOnlyAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.externalized.profileaudio",
                "test",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_source_extensions(["profileaudio"])
            .with_priority(100),
            "profile audio importer is externalized",
        ))
        .unwrap();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.available.profileaudio",
                "test",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_source_extensions(["profileaudio"])
            .with_priority(10),
            |context| test_data_outcome(context, "available"),
        ))
        .unwrap();

    let report = registry
        .capability_report_for_source(&path)
        .expect("available importer report");
    assert_eq!(report.descriptor.id, "test.available.profileaudio");
    assert!(report.status.is_available());

    let imported = AssetImporter::with_registry(registry)
        .import_from_source(&path, &uri)
        .unwrap();

    match imported {
        ImportedAsset::Data(data) => assert_eq!(data.canonical_json["winner"], "available"),
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_registry_prefers_available_full_suffix_importer_over_higher_priority_diagnostic() {
    let root = unique_temp_project_root("registry_available_over_diagnostic_suffix");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("payload.profile.audio");
    fs::write(&path, "payload").unwrap();
    let uri = AssetUri::parse("res://data/payload.profile.audio").unwrap();

    let mut registry = AssetImporterRegistry::default();
    registry
        .register(DiagnosticOnlyAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.externalized.profile_audio_suffix",
                "test",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_full_suffixes([".profile.audio"])
            .with_priority(100),
            "profile audio suffix importer is externalized",
        ))
        .unwrap();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.available.profile_audio_suffix",
                "test",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_full_suffixes([".profile.audio"])
            .with_priority(10),
            |context| test_data_outcome(context, "available_suffix"),
        ))
        .unwrap();

    let report = registry
        .capability_report_for_source(&path)
        .expect("available suffix importer report");
    assert_eq!(report.descriptor.id, "test.available.profile_audio_suffix");
    assert!(report.status.is_available());

    let imported = AssetImporter::with_registry(registry)
        .import_from_source(&path, &uri)
        .unwrap();

    match imported {
        ImportedAsset::Data(data) => assert_eq!(data.canonical_json["winner"], "available_suffix"),
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_registry_rejects_same_priority_duplicate_matcher() {
    let mut registry = AssetImporterRegistry::default();
    registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.first", "test", crate::asset::AssetKind::Data, 1)
                .with_source_extensions(["dup"]),
            |context| test_data_outcome(context, "first"),
        ))
        .unwrap();

    let error = registry
        .register(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.second", "test", crate::asset::AssetKind::Data, 1)
                .with_source_extensions(["dup"]),
            |context| test_data_outcome(context, "second"),
        ))
        .unwrap_err();

    assert!(error.to_string().contains("duplicate importer matcher"));
}
