use super::model_mesh_subassets::model_outcome_with_mesh_subassets;
use super::primitive_from_indexed_mesh::{
    backfill_mesh_sdf_for_model, backfill_virtual_geometry_for_model,
};
use crate::asset::assets::ModelAsset;
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

// 作者模型 TOML 先解析项目引用，再按请求补充可选 VG/SDF 结果与具名 mesh 子资产；
// 引用修复随 outcome 交给项目事务，未登记 GUID 不得仅凭路径候选静默替换。
pub(crate) fn import_model(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let mut model = ModelAsset::from_project_toml_str(&document, |reference| {
        context.resolve_project_asset_ref(reference)
    })?;
    let virtual_geometry_request = context.virtual_geometry_cook_request()?;
    let mesh_sdf_request = context.mesh_sdf_cook_request()?;
    backfill_virtual_geometry_for_model(&mut model, &virtual_geometry_request);
    backfill_mesh_sdf_for_model(&mut model, &mesh_sdf_request)?;
    Ok(
        model_outcome_with_mesh_subassets(context.uri.clone(), model)
            .with_reference_repairs(context.reference_repairs()),
    )
}

#[cfg(test)]
#[path = "tests/import_model.rs"]
mod tests;
