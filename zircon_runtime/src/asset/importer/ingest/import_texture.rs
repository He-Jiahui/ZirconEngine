use crate::asset::assets::{ImportedAsset, TextureAsset};
use crate::asset::{
    decode_texture_source_image, AssetImportContext, AssetImportError, AssetImportOutcome,
};
use crate::core::framework::render::TextureMetadataDiagnosticSeverity;
use crate::core::resource::{ResourceDiagnostic, ResourceDiagnosticSeverity};

pub(crate) fn import_texture(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let image = decode_texture_source_image(context)?;
    let texture =
        TextureAsset::new_rgba8(context.uri.clone(), image.width, image.height, image.rgba)
            .apply_import_settings(&context.import_settings)
            .map_err(|error| {
                AssetImportError::Parse(format!(
                    "apply texture import settings {}: {error}",
                    context.source_path.display()
                ))
            })?;
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    for diagnostic in texture
        .texture_descriptor()
        .validate_metadata(&context.uri.to_string())
    {
        match diagnostic.severity {
            TextureMetadataDiagnosticSeverity::Error => errors.push(diagnostic.message),
            TextureMetadataDiagnosticSeverity::Warning => warnings.push(diagnostic.message),
        }
    }
    if !errors.is_empty() {
        return Err(AssetImportError::Parse(format!(
            "validate texture metadata {}: {}",
            context.uri,
            errors.join("; ")
        )));
    }

    let mut outcome = AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Texture(texture));
    for message in warnings {
        outcome = outcome.with_diagnostic(ResourceDiagnostic {
            severity: ResourceDiagnosticSeverity::Warning,
            message,
        });
    }
    Ok(outcome)
}
