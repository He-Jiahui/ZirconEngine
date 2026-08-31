use crate::container::parse_container_info;
use zircon_runtime::asset::{
    decode_texture_source_image, AssetImportContext, AssetImportError, AssetImportOutcome,
    ImportedAsset, TextureAsset, TextureAssetDescriptor, TexturePayload,
};
use zircon_runtime::core::{
    framework::render::{
        TextureMetadataDiagnostic, TextureMetadataDiagnosticSeverity, TextureMipPolicy,
    },
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
    let metadata_diagnostics = texture.texture_descriptor().validate_metadata(&source_path);
    let (errors, warnings) = partition_metadata_diagnostics(metadata_diagnostics);
    if errors.is_empty() {
        let mut diagnostics = warnings
            .into_iter()
            .map(|message| ResourceDiagnostic {
                severity: ResourceDiagnosticSeverity::Warning,
                message,
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

fn partition_metadata_diagnostics(
    diagnostics: Vec<TextureMetadataDiagnostic>,
) -> (Vec<String>, Vec<String>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for diagnostic in diagnostics {
        match diagnostic.severity {
            TextureMetadataDiagnosticSeverity::Error => errors.push(diagnostic.message),
            TextureMetadataDiagnosticSeverity::Warning => warnings.push(diagnostic.message),
        }
    }
    (errors, warnings)
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

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;

    #[test]
    fn import_pipeline_hotpath_metadata_partition_matches_clone_reference() {
        let diagnostics = vec![
            metadata_diagnostic(TextureMetadataDiagnosticSeverity::Warning, "warning-a"),
            metadata_diagnostic(TextureMetadataDiagnosticSeverity::Error, "error-a"),
            metadata_diagnostic(TextureMetadataDiagnosticSeverity::Warning, "warning-b"),
            metadata_diagnostic(TextureMetadataDiagnosticSeverity::Error, "error-b"),
        ];

        let legacy = legacy_partition_metadata_diagnostics(diagnostics.clone());
        let optimized = partition_metadata_diagnostics(diagnostics);

        assert_eq!(optimized, legacy);
        assert_eq!(
            optimized.0,
            vec!["error-a".to_string(), "error-b".to_string()]
        );
        assert_eq!(
            optimized.1,
            vec!["warning-a".to_string(), "warning-b".to_string()]
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn import_pipeline_hotpath_metadata_partition_release_benchmark() {
        const DIAGNOSTIC_COUNT: usize = 8_192;
        const MESSAGE_BYTES: usize = 96;
        const REQUIRED_IMPROVEMENT_PERCENT: u128 = 40;

        let message = "m".repeat(MESSAGE_BYTES);
        let diagnostics = (0..DIAGNOSTIC_COUNT)
            .map(|_| metadata_diagnostic(TextureMetadataDiagnosticSeverity::Warning, &message))
            .collect::<Vec<_>>();
        assert_eq!(
            partition_metadata_diagnostics(diagnostics.clone()),
            legacy_partition_metadata_diagnostics(diagnostics.clone())
        );

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_partition(
                    &diagnostics,
                    legacy_partition_metadata_diagnostics,
                ));
                optimized_samples.push(measure_partition(
                    &diagnostics,
                    partition_metadata_diagnostics,
                ));
            } else {
                optimized_samples.push(measure_partition(
                    &diagnostics,
                    partition_metadata_diagnostics,
                ));
                legacy_samples.push(measure_partition(
                    &diagnostics,
                    legacy_partition_metadata_diagnostics,
                ));
            }
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement = improvement_percent(legacy_p95, optimized_p95);
        println!(
            "PERF_RESULT plugins07_metadata_diagnostic_partition sample_pairs={} order=alternating_legacy_first_even diagnostics_per_sample={} message_bytes={} legacy_message_clones_per_sample={} optimized_message_clones_per_sample=0 legacy_ns={} optimized_ns={} legacy_p95_ns={} optimized_p95_ns={} threshold_percent={} improvement_percent={}",
            SAMPLE_PAIRS,
            DIAGNOSTIC_COUNT,
            MESSAGE_BYTES,
            DIAGNOSTIC_COUNT,
            samples_csv(&legacy_samples),
            samples_csv(&optimized_samples),
            legacy_p95,
            optimized_p95,
            REQUIRED_IMPROVEMENT_PERCENT,
            improvement
        );
        assert!(
            improvement >= REQUIRED_IMPROVEMENT_PERCENT,
            "single-pass diagnostic partition improved {improvement}%, below {REQUIRED_IMPROVEMENT_PERCENT}%"
        );
    }

    fn metadata_diagnostic(
        severity: TextureMetadataDiagnosticSeverity,
        message: &str,
    ) -> TextureMetadataDiagnostic {
        TextureMetadataDiagnostic {
            severity,
            message: message.to_string(),
        }
    }

    fn legacy_partition_metadata_diagnostics(
        diagnostics: Vec<TextureMetadataDiagnostic>,
    ) -> (Vec<String>, Vec<String>) {
        let diagnostics = diagnostics.into_iter();
        let errors = diagnostics
            .clone()
            .filter(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error)
            .map(|diagnostic| diagnostic.message)
            .collect::<Vec<_>>();
        let warnings = diagnostics
            .filter(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning)
            .map(|diagnostic| diagnostic.message)
            .collect::<Vec<_>>();
        (errors, warnings)
    }

    fn measure_partition(
        diagnostics: &[TextureMetadataDiagnostic],
        partition: fn(Vec<TextureMetadataDiagnostic>) -> (Vec<String>, Vec<String>),
    ) -> u128 {
        let owned = diagnostics.to_vec();
        let started = Instant::now();
        black_box(partition(black_box(owned)));
        started.elapsed().as_nanos()
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        assert_eq!(samples.len(), SAMPLE_PAIRS);
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        ordered[(ordered.len() * 95).div_ceil(100) - 1]
    }

    fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
        assert!(legacy > 0);
        legacy.saturating_sub(optimized) * 100 / legacy
    }

    fn samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
