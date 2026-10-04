//! 给两种项目资产管理入口注册同一测试导入器集合，确保样例比较的是导入和运行时加载链，而非注册差异。

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

// 测试专用 DDS 入口只建立容器资产；运行时的上传能力仍由消费者单独判定。
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
