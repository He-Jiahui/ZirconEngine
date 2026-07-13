use crate::asset::assets::{ImportedAsset, MaterialAsset, ZMaterialDocument};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_material(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let material_document = ZMaterialDocument::from_project_toml_str(&document, |reference| {
        context.resolve_project_asset_ref(reference)
    })?;
    let material = MaterialAsset::from_zmaterial_document(material_document);
    let mut outcome = AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Material(material.clone()),
    )
    .with_dependency(material.shader.locator.clone());
    for (_, texture) in material.all_texture_slots() {
        outcome = outcome.with_dependency(texture.locator.clone());
    }
    Ok(outcome.with_reference_repairs(context.reference_repairs()))
}
