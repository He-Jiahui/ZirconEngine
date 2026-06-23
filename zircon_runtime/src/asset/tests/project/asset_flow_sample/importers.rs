use super::*;

pub(super) fn project_manager_with_sample_importers(root: &Path) -> ProjectManager {
    let mut manager = ProjectManager::open(root).unwrap();
    manager
        .importer_mut()
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager
        .register_asset_importer(dds_container_importer())
        .unwrap();
    manager
}

pub(super) fn project_asset_manager_with_sample_importers() -> ProjectAssetManager {
    let manager = ProjectAssetManager::default();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager
        .register_asset_importer(dds_container_importer())
        .unwrap();
    manager
}

fn dds_container_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new(
            "test.texture.dds.container",
            "test.texture",
            AssetKind::Texture,
            1,
        )
        .with_source_extensions(["dds"])
        .with_priority(130),
        import_dds_container_texture,
    )
}

fn import_dds_container_texture(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let texture = TextureAsset::new_container(
        context.uri.clone(),
        4,
        4,
        "dds/DXT1",
        context.source_bytes.clone(),
        1,
        1,
    );
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(texture),
    ))
}
