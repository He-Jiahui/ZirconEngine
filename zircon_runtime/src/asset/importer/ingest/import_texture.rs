use crate::asset::assets::{normalize_texture_normal_map_convention, ImportedAsset, TextureAsset};
use crate::asset::{
    decode_texture_source_image, AssetImportContext, AssetImportError, AssetImportOutcome,
};
use crate::core::framework::render::TextureMetadataDiagnosticSeverity;
use crate::core::resource::{ResourceDiagnostic, ResourceDiagnosticSeverity};

pub(crate) fn import_texture(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let image = decode_texture_source_image(context)?;
    let texture = normalize_texture_normal_map_convention(
        TextureAsset::new_rgba8(context.uri.clone(), image.width, image.height, image.rgba)
            .apply_import_settings(context.import_settings())
            .map_err(|error| {
                AssetImportError::Parse(format!(
                    "apply texture import settings {}: {error}",
                    context.source_path.display()
                ))
            })?,
    )
    .map_err(|error| AssetImportError::Parse(error.to_string()))?;
    let warnings = texture_metadata_warnings(context, &texture)?;

    let mut outcome = AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Texture(texture));
    if let Some(root) = outcome.entries.first_mut() {
        root.diagnostics = warnings;
    }
    Ok(outcome)
}

fn texture_metadata_warnings(
    context: &AssetImportContext,
    texture: &TextureAsset,
) -> Result<Vec<ResourceDiagnostic>, AssetImportError> {
    let fallback_descriptor;
    let descriptor = match texture.descriptor.as_ref() {
        Some(descriptor) => descriptor,
        None => {
            fallback_descriptor = texture.texture_descriptor();
            &fallback_descriptor
        }
    };
    let diagnostics = descriptor.validate_metadata(&context.uri.to_string());
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error)
    {
        let mut errors = String::new();
        for diagnostic in diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error)
        {
            if !errors.is_empty() {
                errors.push_str("; ");
            }
            errors.push_str(&diagnostic.message);
        }
        return Err(AssetImportError::Parse(format!(
            "validate texture metadata {}: {errors}",
            context.uri
        )));
    }

    Ok(diagnostics
        .into_iter()
        .map(|diagnostic| ResourceDiagnostic {
            severity: ResourceDiagnosticSeverity::Warning,
            message: diagnostic.message,
        })
        .collect())
}

#[cfg(test)]
#[path = "tests/import_texture.rs"]
mod tests;
