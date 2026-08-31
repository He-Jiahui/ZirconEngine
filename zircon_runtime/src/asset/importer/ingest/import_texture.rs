use crate::asset::assets::{ImportedAsset, TextureAsset, normalize_texture_normal_map_convention};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, decode_texture_source_image,
};
use crate::core::framework::render::TextureMetadataDiagnosticSeverity;
use crate::core::resource::{ResourceDiagnostic, ResourceDiagnosticSeverity};

pub(crate) fn import_texture(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let image = decode_texture_source_image(context)?;
    let texture = normalize_texture_normal_map_convention(
        TextureAsset::new_rgba8(context.uri.clone(), image.width, image.height, image.rgba)
            .apply_import_settings(&context.import_settings)
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
mod tests {
    use std::hint::black_box;
    use std::io::Cursor;
    use std::time::Instant;

    use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};

    use super::*;
    use crate::asset::AssetUri;
    use crate::core::framework::render::{RenderImageColorSpace, TextureUsageHint};

    const BENCHMARK_CHECKS: usize = 32_768;
    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_THRESHOLD_PERCENT: u128 = 20;

    fn context() -> AssetImportContext {
        AssetImportContext::new(
            "metadata.png".into(),
            AssetUri::parse("res://textures/metadata.png").unwrap(),
            Vec::new(),
            toml::Table::new(),
        )
    }

    fn texture() -> TextureAsset {
        TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/metadata.png").unwrap(),
            1,
            1,
            vec![255, 255, 255, 255],
        )
    }

    fn rgba8_png_bytes(pixel: [u8; 4]) -> Vec<u8> {
        let image = ImageBuffer::<Rgba<u8>, _>::from_pixel(1, 1, Rgba(pixel));
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn builtin_texture_importer_canonicalizes_explicit_dx_normals_to_gl() {
        let settings = r#"
usage_hint = "normal"
normal_convention = "dx"
mip_policy = "from_source"
compression = "uncompressed"
"#
        .parse()
        .unwrap();
        let context = AssetImportContext::new(
            "normal.png".into(),
            AssetUri::parse("res://textures/normal.png").unwrap(),
            rgba8_png_bytes([128, 64, 255, 255]),
            settings,
        );

        let outcome = import_texture(&context).unwrap();
        let ImportedAsset::Texture(texture) = &outcome.root_entry().unwrap().asset else {
            panic!("builtin texture importer must produce a texture");
        };

        assert_eq!(texture.rgba, vec![128, 191, 255, 255]);
        assert_eq!(
            texture.texture_descriptor().metadata.normal_convention,
            crate::core::framework::render::TextureNormalConvention::TangentSpaceGl
        );
    }

    #[test]
    fn builtin_texture_importer_preserves_explicit_gl_normals() {
        let settings = r#"
usage_hint = "normal"
normal_convention = "gl"
mip_policy = "from_source"
compression = "uncompressed"
"#
        .parse()
        .unwrap();
        let context = AssetImportContext::new(
            "normal.png".into(),
            AssetUri::parse("res://textures/normal.png").unwrap(),
            rgba8_png_bytes([128, 64, 255, 255]),
            settings,
        );

        let outcome = import_texture(&context).unwrap();
        let ImportedAsset::Texture(texture) = &outcome.root_entry().unwrap().asset else {
            panic!("builtin texture importer must produce a texture");
        };

        assert_eq!(texture.rgba, vec![128, 64, 255, 255]);
        assert_eq!(
            texture.texture_descriptor().metadata.normal_convention,
            crate::core::framework::render::TextureNormalConvention::TangentSpaceGl
        );
    }

    fn legacy_texture_metadata_warnings(
        context: &AssetImportContext,
        texture: &TextureAsset,
    ) -> Result<Vec<ResourceDiagnostic>, AssetImportError> {
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
        Ok(warnings
            .into_iter()
            .map(|message| ResourceDiagnostic {
                severity: ResourceDiagnosticSeverity::Warning,
                message,
            })
            .collect())
    }

    fn measure_metadata_validation(
        context: &AssetImportContext,
        texture: &TextureAsset,
        mut validate: impl FnMut(
            &AssetImportContext,
            &TextureAsset,
        ) -> Result<Vec<ResourceDiagnostic>, AssetImportError>,
    ) -> u128 {
        let timer = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..BENCHMARK_CHECKS {
            checksum += black_box(validate(black_box(context), black_box(texture)).unwrap()).len();
        }
        black_box(checksum);
        timer.elapsed().as_nanos()
    }

    fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 - 1) / 100]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn borrowed_texture_metadata_validation_preserves_warning() {
        let context = context();
        let mut texture = texture();
        let descriptor = texture.descriptor.as_mut().unwrap();
        descriptor.color_space = RenderImageColorSpace::Linear;
        descriptor.metadata.color_space = RenderImageColorSpace::Linear;
        descriptor.metadata.usage_hint = TextureUsageHint::Albedo;

        let warnings = texture_metadata_warnings(&context, &texture).unwrap();

        assert!(warnings.iter().any(|warning| {
            warning.severity == ResourceDiagnosticSeverity::Warning
                && warning.message.contains("declares linear")
        }));
    }

    #[test]
    fn borrowed_texture_metadata_validation_preserves_error() {
        let context = context();
        let mut texture = texture();
        texture.descriptor.as_mut().unwrap().metadata.usage_hint = TextureUsageHint::Normal;

        let error = texture_metadata_warnings(&context, &texture)
            .expect_err("srgb normal metadata must be rejected");

        assert!(
            error
                .to_string()
                .contains("normal map must use linear color space")
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn benchmark_borrowed_texture_metadata_descriptor() {
        let context = context();
        let texture = texture();
        assert_eq!(
            legacy_texture_metadata_warnings(&context, &texture).unwrap(),
            texture_metadata_warnings(&context, &texture).unwrap()
        );

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_metadata_validation(
                    &context,
                    &texture,
                    legacy_texture_metadata_warnings,
                ));
                optimized_samples.push(measure_metadata_validation(
                    &context,
                    &texture,
                    texture_metadata_warnings,
                ));
            } else {
                optimized_samples.push(measure_metadata_validation(
                    &context,
                    &texture,
                    texture_metadata_warnings,
                ));
                legacy_samples.push(measure_metadata_validation(
                    &context,
                    &texture,
                    legacy_texture_metadata_warnings,
                ));
            }
        }

        let legacy_raw = legacy_samples.clone();
        let optimized_raw = optimized_samples.clone();
        let legacy_p95_ns = nearest_rank_p95(&mut legacy_samples);
        let optimized_p95_ns = nearest_rank_p95(&mut optimized_samples);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);

        println!(
            "PERF_RESULT plugins07_borrowed_texture_metadata_descriptor checks_per_sample={} sample_pairs={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_descriptor_clones_per_sample={} optimized_descriptor_clones_per_sample=0 legacy_extra_partition_vectors_per_sample={} optimized_extra_partition_vectors_per_sample=0 legacy_p95_ns={} optimized_p95_ns={} improvement_percent={} threshold_percent={} legacy_ns={} optimized_ns={}",
            BENCHMARK_CHECKS,
            BENCHMARK_SAMPLE_PAIRS,
            BENCHMARK_CHECKS,
            BENCHMARK_CHECKS * 2,
            legacy_p95_ns,
            optimized_p95_ns,
            improvement_percent,
            BENCHMARK_THRESHOLD_PERCENT,
            sample_csv(&legacy_raw),
            sample_csv(&optimized_raw),
        );

        assert_eq!(BENCHMARK_SAMPLE_PAIRS, legacy_raw.len());
        assert_eq!(BENCHMARK_SAMPLE_PAIRS, optimized_raw.len());
        assert!(
            improvement_percent >= BENCHMARK_THRESHOLD_PERCENT,
            "borrowed texture metadata P95 improvement {improvement_percent}% misses {BENCHMARK_THRESHOLD_PERCENT}% gate"
        );
    }
}
