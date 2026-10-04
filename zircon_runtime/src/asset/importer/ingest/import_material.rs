use crate::asset::assets::{ImportedAsset, MaterialAsset, ZMaterialDocument};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

// 项目材质引用先经 context 解析并记录修复，材质自身再给出直接依赖；
// 项目导入事务依赖这些 locator 发布依赖图，不能只保留 shader 或 parent 的局部列表。
pub(crate) fn import_material(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let material_document = ZMaterialDocument::from_project_toml_str(&document, |reference| {
        context.resolve_project_asset_ref(reference)
    })?;
    let material = MaterialAsset::from_zmaterial_document(material_document);
    let dependencies = material.direct_reference_locators();
    let mut outcome =
        AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Material(material));
    outcome.entries[0].dependencies = dependencies;
    Ok(outcome.with_reference_repairs(context.reference_repairs()))
}

#[cfg(test)]
#[path = "tests/import_material_plugins07_material_dependency_tests.rs"]
mod plugins07_material_dependency_tests;
