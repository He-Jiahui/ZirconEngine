use crate::asset::assets::{ImportedAsset, SceneAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_scene(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let scene = SceneAsset::from_project_toml_str(&document, |reference| {
        context.resolve_project_asset_ref(reference)
    })?;
    Ok(
        AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Scene(scene))
            .with_reference_repairs(context.reference_repairs()),
    )
}
