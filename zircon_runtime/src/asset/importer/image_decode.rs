use std::io::Cursor;

use image::{DynamicImage, GenericImageView, ImageFormat};

use super::{AssetImportContext, AssetImportError};

mod source_format_identity;
mod source_metadata;

use source_format_identity::stable_source_format_identity;
pub(crate) use source_metadata::{
    decode_texture_source_image_metadata, texture_source_image_format_identity,
    TextureSourceImageMetadata,
};

/// RGBA8 image data decoded from a source image before texture descriptor overrides apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedTextureImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Linear RGBA32F image data for import stages that must preserve HDR radiance.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodedTextureImageRgba32F {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<[f32; 4]>,
}

/// Decode image bytes using Bevy-style source format selection.
///
/// The default path trusts the source extension. Import settings can request
/// `image_format = "guess"` or a concrete image crate format such as `jpeg`.
pub fn decode_texture_source_image(
    context: &AssetImportContext,
) -> Result<DecodedTextureImage, AssetImportError> {
    let image = decode_texture_source_dynamic_image(context)?;
    let (width, height) = image.dimensions();
    Ok(DecodedTextureImage {
        width,
        height,
        rgba: image.to_rgba8().into_raw(),
    })
}

/// Decode source image bytes without quantizing HDR/EXR radiance through RGBA8.
pub fn decode_texture_source_image_rgba32f(
    context: &AssetImportContext,
) -> Result<DecodedTextureImageRgba32F, AssetImportError> {
    let image = decode_texture_source_dynamic_image(context)?;
    let (width, height) = image.dimensions();
    let rgba = image.to_rgba32f().pixels().map(|pixel| pixel.0).collect();
    Ok(DecodedTextureImageRgba32F {
        width,
        height,
        rgba,
    })
}

fn decode_texture_source_dynamic_image(
    context: &AssetImportContext,
) -> Result<DynamicImage, AssetImportError> {
    texture_source_image_reader(context)?
        .decode()
        .map_err(|error| decode_error_value(context, format!("decode resolved image: {error}")))
}

fn texture_source_image_reader<'a>(
    context: &'a AssetImportContext,
) -> Result<image::ImageReader<Cursor<&'a [u8]>>, AssetImportError> {
    let setting = image_format_setting(context)?;
    Ok(match setting {
        ImageFormatSetting::FromExtension { format, .. }
        | ImageFormatSetting::Format { format, .. } => {
            image::ImageReader::with_format(Cursor::new(context.source_bytes.as_slice()), format)
        }
        ImageFormatSetting::Guess => {
            image::ImageReader::new(Cursor::new(context.source_bytes.as_slice()))
                .with_guessed_format()
                .map_err(|error| {
                    decode_error_value(context, format!("guess image format from bytes: {error}"))
                })?
        }
    })
}

enum ImageFormatSetting<'a> {
    FromExtension {
        extension: &'a str,
        format: ImageFormat,
    },
    Format {
        token: &'a str,
        format: ImageFormat,
    },
    Guess,
}

fn image_format_setting<'a>(
    context: &'a AssetImportContext,
) -> Result<ImageFormatSetting<'a>, AssetImportError> {
    let Some((key, value)) = context
        .import_settings
        .get("image_format")
        .map(|value| ("image_format", value))
        .or_else(|| {
            context
                .import_settings
                .get("decode_format")
                .map(|value| ("decode_format", value))
        })
        .or_else(|| {
            context
                .import_settings
                .get("source_format")
                .map(|value| ("source_format", value))
        })
    else {
        return image_format_from_extension(context);
    };
    let token = value.as_str().ok_or_else(|| {
        decode_error_value(
            context,
            format!("image import setting `{key}` must be a string"),
        )
    })?;
    if normalized_token_eq(token, "from_extension") || normalized_token_eq(token, "extension") {
        return image_format_from_extension(context);
    }
    if normalized_token_eq(token, "guess")
        || normalized_token_eq(token, "from_bytes")
        || normalized_token_eq(token, "bytes")
    {
        return Ok(ImageFormatSetting::Guess);
    }
    image_format_from_token(token)
        .map(|format| ImageFormatSetting::Format { token, format })
        .ok_or_else(|| {
            decode_error_value(
                context,
                format!("unsupported image import setting `{key} = {token}`"),
            )
        })
}

fn image_format_from_token(token: &str) -> Option<ImageFormat> {
    if normalized_token_eq(token, "open_exr") || normalized_token_eq(token, "openexr") {
        return Some(ImageFormat::OpenExr);
    }
    if normalized_token_eq(token, "radiance_hdr") || normalized_token_eq(token, "radiance") {
        return Some(ImageFormat::Hdr);
    }
    if normalized_token_eq(token, "portable_anymap")
        || normalized_token_eq(token, "portable_bitmap")
        || normalized_token_eq(token, "portable_graymap")
        || normalized_token_eq(token, "portable_pixmap")
    {
        return Some(ImageFormat::Pnm);
    }
    image_format_from_extension_token(token)
}

fn image_format_from_extension<'a>(
    context: &'a AssetImportContext,
) -> Result<ImageFormatSetting<'a>, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| decode_error_value(context, "image source has no file extension"))?;
    image_format_from_extension_token(extension)
        .map(|format| ImageFormatSetting::FromExtension { extension, format })
        .ok_or_else(|| {
            decode_error_value(
                context,
                format!("unsupported image extension `{extension}`"),
            )
        })
}

fn normalized_token_eq(value: &str, expected: &str) -> bool {
    let value = value.trim().as_bytes();
    let expected = expected.as_bytes();
    value.len() == expected.len()
        && value.iter().zip(expected).all(|(value, expected)| {
            let value = if *value == b'-' { b'_' } else { *value };
            value.to_ascii_lowercase() == expected.to_ascii_lowercase()
        })
}

fn image_format_from_extension_token(token: &str) -> Option<ImageFormat> {
    let token = token.trim();
    let first = token.as_bytes().first()?.to_ascii_lowercase();
    match (token.len(), first) {
        (2, b'f') if token.eq_ignore_ascii_case("ff") => Some(ImageFormat::Farbfeld),
        (3, b'b') if token.eq_ignore_ascii_case("bmp") => Some(ImageFormat::Bmp),
        (3, b'd') if token.eq_ignore_ascii_case("dds") => Some(ImageFormat::Dds),
        (3, b'e') if token.eq_ignore_ascii_case("exr") => Some(ImageFormat::OpenExr),
        (3, b'g') if token.eq_ignore_ascii_case("gif") => Some(ImageFormat::Gif),
        (3, b'h') if token.eq_ignore_ascii_case("hdr") => Some(ImageFormat::Hdr),
        (3, b'i') if token.eq_ignore_ascii_case("ico") => Some(ImageFormat::Ico),
        (3, b'j') if token.eq_ignore_ascii_case("jpg") => Some(ImageFormat::Jpeg),
        (3, b'p') if token.eq_ignore_ascii_case("png") => Some(ImageFormat::Png),
        (3, b'p')
            if ["pbm", "pam", "ppm", "pgm", "pnm"]
                .iter()
                .any(|candidate| token.eq_ignore_ascii_case(candidate)) =>
        {
            Some(ImageFormat::Pnm)
        }
        (3, b'q') if token.eq_ignore_ascii_case("qoi") => Some(ImageFormat::Qoi),
        (3, b't') if token.eq_ignore_ascii_case("tga") => Some(ImageFormat::Tga),
        (3, b't') if token.eq_ignore_ascii_case("tif") => Some(ImageFormat::Tiff),
        (4, b'a') if token.eq_ignore_ascii_case("avif") => Some(ImageFormat::Avif),
        (4, b'a') if token.eq_ignore_ascii_case("apng") => Some(ImageFormat::Png),
        (4, b'j') if token.eq_ignore_ascii_case("jpeg") || token.eq_ignore_ascii_case("jfif") => {
            Some(ImageFormat::Jpeg)
        }
        (4, b't') if token.eq_ignore_ascii_case("tiff") => Some(ImageFormat::Tiff),
        (4, b'w') if token.eq_ignore_ascii_case("webp") => Some(ImageFormat::WebP),
        _ => None,
    }
}

fn decode_error_value(
    context: &AssetImportContext,
    message: impl Into<String>,
) -> AssetImportError {
    AssetImportError::Parse(format!(
        "decode image {}: {}",
        context.source_path.display(),
        message.into()
    ))
}

#[cfg(test)]
mod tests {
    use std::{hint::black_box, time::Instant};

    use super::*;
    use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};

    #[test]
    fn default_format_reports_missing_extension() {
        let context = texture_context("checker", tiny_png_bytes(), "");

        let error = decode_texture_source_image(&context)
            .unwrap_err()
            .to_string();

        assert!(
            error.contains("image source has no file extension"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn explicit_source_format_reports_unsupported_token() {
        let context = texture_context(
            "checker.png",
            tiny_png_bytes(),
            r#"source_format = "made_up_format""#,
        );

        let error = decode_texture_source_image(&context)
            .unwrap_err()
            .to_string();

        assert!(
            error.contains("unsupported image import setting `source_format = made_up_format`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn header_metadata_matches_full_decode_without_pixel_materialization() {
        let context = texture_context("checker.png", tiny_png_bytes(), "");

        let metadata = decode_texture_source_image_metadata(&context)
            .expect("header metadata must inspect a supported source");
        let decoded = decode_texture_source_image(&context).expect("full image must decode");

        assert_eq!(
            (metadata.width(), metadata.height()),
            (decoded.width, decoded.height)
        );
        assert_eq!(metadata.format_identity(), 1);
    }

    #[test]
    fn explicit_alias_and_guessed_bytes_converge_on_stable_format_identity() {
        let bytes = tiny_png_bytes();
        let explicit = texture_context("checker.bin", bytes.clone(), r#"source_format = "PNG""#);
        let guessed = texture_context("checker.bin", bytes, r#"source_format = "guess""#);

        let explicit = decode_texture_source_image_metadata(&explicit)
            .expect("explicit PNG metadata must resolve");
        let guessed = decode_texture_source_image_metadata(&guessed)
            .expect("guessed PNG metadata must resolve");

        assert_eq!(explicit, guessed);
    }

    #[test]
    fn plugins07_builtin_import_hotpath_image_format_selection_preserves_aliases_and_borrows() {
        let cases = [
            ("PNG", ImageFormat::Png),
            ("JpEg", ImageFormat::Jpeg),
            ("TIFF", ImageFormat::Tiff),
            ("Open-EXR", ImageFormat::OpenExr),
            ("portable_bitmap", ImageFormat::Pnm),
            ("Radiance-HDR", ImageFormat::Hdr),
        ];
        for (token, expected) in cases {
            assert_eq!(image_format_from_token(token), Some(expected));
        }

        let extension_context = texture_context("checker.PnG", Vec::new(), "");
        let ImageFormatSetting::FromExtension { extension, format } =
            image_format_setting(&extension_context).expect("mixed-case extension is supported")
        else {
            panic!("expected extension-derived image format");
        };
        assert_eq!(extension, "PnG");
        assert_eq!(format, ImageFormat::Png);

        let explicit_context = texture_context(
            "checker.bin",
            Vec::new(),
            r#"source_format = "Portable-Bitmap""#,
        );
        let ImageFormatSetting::Format { token, format } =
            image_format_setting(&explicit_context).expect("explicit alias is supported")
        else {
            panic!("expected explicit image format");
        };
        assert_eq!(token, "Portable-Bitmap");
        assert_eq!(format, ImageFormat::Pnm);
    }

    #[test]
    #[ignore = "release-only allocation-free image format selection benchmark"]
    fn plugins07_builtin_import_hotpath_release_allocation_free_image_format_selection_p95_gate() {
        const SAMPLE_PAIRS: usize = 21;
        const CHECKS_PER_SAMPLE: usize = 100_000;
        const TOKENS: [&str; 12] = [
            "PNG", "JpEg", "GIF", "WebP", "TIFF", "TGA", "DDS", "BMP", "ICO", "HDR", "EXR", "QOI",
        ];

        fn legacy_image_format_from_token(token: &str) -> Option<ImageFormat> {
            let normalized = token.trim().to_ascii_lowercase().replace('-', "_");
            match normalized.as_str() {
                "open_exr" | "openexr" => Some(ImageFormat::OpenExr),
                "radiance_hdr" | "radiance" => Some(ImageFormat::Hdr),
                "portable_anymap" | "portable_bitmap" | "portable_graymap" | "portable_pixmap" => {
                    Some(ImageFormat::Pnm)
                }
                _ => ImageFormat::from_extension(normalized.as_str()),
            }
        }

        fn measure(classify: impl Fn(&str) -> Option<ImageFormat>) -> u128 {
            let started = Instant::now();
            for check in 0..CHECKS_PER_SAMPLE {
                let token = black_box(TOKENS[check % TOKENS.len()]);
                black_box(classify(token));
            }
            started.elapsed().as_nanos().max(1)
        }

        for token in TOKENS {
            assert_eq!(
                image_format_from_token(token),
                legacy_image_format_from_token(token),
            );
        }
        let (legacy_samples, optimized_samples) = alternating_samples(
            SAMPLE_PAIRS,
            || measure(legacy_image_format_from_token),
            || measure(image_format_from_token),
        );
        report_and_assert(
            "plugins07_builtin_image_format_selection",
            SAMPLE_PAIRS,
            CHECKS_PER_SAMPLE,
            TOKENS.len(),
            CHECKS_PER_SAMPLE * 3,
            &legacy_samples,
            &optimized_samples,
        );
    }

    fn alternating_samples(
        sample_pairs: usize,
        mut legacy: impl FnMut() -> u128,
        mut optimized: impl FnMut() -> u128,
    ) -> (Vec<u128>, Vec<u128>) {
        for _ in 0..4 {
            black_box(legacy());
            black_box(optimized());
        }
        let mut legacy_samples = Vec::with_capacity(sample_pairs);
        let mut optimized_samples = Vec::with_capacity(sample_pairs);
        for pair in 0..sample_pairs {
            if pair % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn report_and_assert(
        name: &str,
        sample_pairs: usize,
        checks_per_sample: usize,
        variants: usize,
        legacy_owned_strings_per_sample: usize,
        legacy_samples: &[u128],
        optimized_samples: &[u128],
    ) {
        let legacy_p95_ns = percentile(legacy_samples, 95);
        let optimized_p95_ns = percentile(optimized_samples, 95);
        let improvement_percent = improvement_percent(legacy_p95_ns, optimized_p95_ns);
        println!(
            "PERF_RESULT {name} sample_pairs={sample_pairs} \
checks_per_sample={checks_per_sample} variants={variants} \
order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_owned_strings_per_sample={legacy_owned_strings_per_sample} \
optimized_owned_strings_per_sample=0 legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} \
threshold_percent=50 legacy_ns={} optimized_ns={}",
            raw(legacy_samples),
            raw(optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
            "allocation-free image format selection must reduce P95 by at least 50%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
        if optimized >= legacy {
            0
        } else {
            legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
        }
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn texture_context(
        source_path: &str,
        source_bytes: Vec<u8>,
        settings: &str,
    ) -> AssetImportContext {
        let uri = format!("res://textures/{source_path}");
        AssetImportContext::new(
            source_path.into(),
            crate::asset::AssetUri::parse(&uri).unwrap(),
            source_bytes,
            settings.parse().expect("valid image import settings"),
        )
    }

    fn tiny_png_bytes() -> Vec<u8> {
        let image = ImageBuffer::<Rgba<u8>, _>::from_fn(1, 1, |_x, _y| Rgba([1, 2, 3, 255]));
        let image = DynamicImage::ImageRgba8(image);
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Png).unwrap();
        bytes.into_inner()
    }
}
