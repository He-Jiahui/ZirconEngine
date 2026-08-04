use crate::container::parse_container_info;
use zircon_runtime::asset::{
    decode_texture_source_image, AssetImportContext, AssetImportError, AssetImportOutcome,
    ImportedAsset, TextureAsset, TextureAssetDescriptor, TexturePayload,
};
use zircon_runtime::core::{
    framework::render::{TextureMetadataDiagnosticSeverity, TextureMipPolicy},
    resource::{ResourceDiagnostic, ResourceDiagnosticSeverity},
};

use crate::mipgen::{generate_offline_mips, prepare_runtime_mips};
use crate::normal_convention::normalize_normal_map_convention;
use crate::transcode::transcode_normal_bc5;

pub fn import_image(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let image = decode_texture_source_image(context)?;
    let (texture, diagnostics) = apply_texture_import_settings(
        context,
        TextureAsset::new_rgba8(context.uri.clone(), image.width, image.height, image.rgba),
    )?;
    Ok(texture_import_outcome(context, texture, diagnostics))
}

pub fn import_psd(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let psd = psd::Psd::from_bytes(&context.source_bytes).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode psd {}: {error}",
            context.source_path.display()
        ))
    })?;
    let width = psd.width();
    let height = psd.height();
    let rgba = psd.rgba();
    let expected_len = width as usize * height as usize * 4;
    if rgba.len() != expected_len {
        return Err(AssetImportError::Parse(format!(
            "decode psd {}: decoded rgba length {} did not match expected {}",
            context.source_path.display(),
            rgba.len(),
            expected_len
        )));
    }

    let (texture, diagnostics) = apply_texture_import_settings(
        context,
        TextureAsset::new_rgba8(context.uri.clone(), width, height, rgba),
    )?;

    Ok(texture_import_outcome(context, texture, diagnostics))
}

pub fn import_texture_container(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let info = parse_container_info(context)?;
    let upload_bytes = info
        .upload_bytes
        .unwrap_or_else(|| context.source_bytes.clone());
    let mut descriptor =
        TextureAssetDescriptor::container(info.format.clone(), info.mip_count, info.array_layers);
    descriptor.dimension = info.dimension;
    descriptor.depth_or_array_layers = info.depth_or_array_layers;
    let (texture, diagnostics) = apply_texture_import_settings(
        context,
        TextureAsset::new_container(
            context.uri.clone(),
            info.width,
            info.height,
            info.format,
            upload_bytes,
            info.mip_count,
            info.array_layers,
        )
        .with_descriptor(descriptor),
    )?;
    Ok(texture_import_outcome(context, texture, diagnostics))
}

pub(crate) fn apply_texture_import_settings(
    context: &AssetImportContext,
    texture: TextureAsset,
) -> Result<(TextureAsset, Vec<ResourceDiagnostic>), AssetImportError> {
    let mut texture = texture
        .apply_import_settings(&context.import_settings)
        .map_err(|error| {
            AssetImportError::Parse(format!(
                "apply texture import settings {}: {error}",
                context.source_path.display()
            ))
        })?;
    let source_mip_count = match &texture.payload {
        TexturePayload::Container { mip_count, .. } => *mip_count,
        TexturePayload::Rgba8 => 1,
    };
    let fallback_to_source_mips = source_mip_count > 1
        && texture.texture_descriptor().metadata.mip_policy == TextureMipPolicy::GenerateOffline;
    if fallback_to_source_mips {
        let mut descriptor = texture.texture_descriptor();
        descriptor.metadata.mip_policy = TextureMipPolicy::FromSource;
        texture.descriptor = Some(descriptor);
    }
    let source_path = context.source_path.display().to_string();
    let metadata_diagnostics = texture
        .texture_descriptor()
        .validate_metadata(&source_path)
        .into_iter();
    let errors = metadata_diagnostics
        .clone()
        .filter(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    if errors.is_empty() {
        let mut diagnostics = metadata_diagnostics
            .filter(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning)
            .map(|diagnostic| ResourceDiagnostic {
                severity: ResourceDiagnosticSeverity::Warning,
                message: diagnostic.message,
            })
            .collect::<Vec<_>>();
        if fallback_to_source_mips {
            diagnostics.push(ResourceDiagnostic {
                severity: ResourceDiagnosticSeverity::Warning,
                message: format!(
                    "'{}' already contains {source_mip_count} mips; falling back to from_source",
                    context.uri
                ),
            });
        }
        let texture = transcode_normal_bc5(generate_offline_mips(prepare_runtime_mips(
            normalize_normal_map_convention(texture)?,
        )?)?)?;
        return Ok((texture, diagnostics));
    }

    Err(AssetImportError::Parse(format!(
        "validate texture metadata {source_path}: {}",
        errors.join("; ")
    )))
}

pub(crate) fn texture_import_outcome(
    context: &AssetImportContext,
    texture: TextureAsset,
    diagnostics: Vec<ResourceDiagnostic>,
) -> AssetImportOutcome {
    diagnostics.into_iter().fold(
        AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Texture(texture)),
        AssetImportOutcome::with_diagnostic,
    )
}
