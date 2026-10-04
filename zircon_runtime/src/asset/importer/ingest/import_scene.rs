use crate::asset::assets::{ImportedAsset, SceneAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

// 场景文档中的项目引用由当前 registry 上下文解析；修复建议随 outcome 传递，
// 让目录更新与导入结果由同一项目事务提交。
pub(crate) fn import_scene(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    let scene = SceneAsset::from_project_toml_str(document, |reference| {
        context.resolve_project_asset_ref(reference)
    })?;
    Ok(
        AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Scene(scene))
            .with_reference_repairs(context.reference_repairs()),
    )
}

#[cfg(test)]
#[path = "tests/import_scene_plugins07_scene_source_tests.rs"]
mod plugins07_scene_source_tests;
