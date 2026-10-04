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
        .import_settings()
        .get("image_format")
        .map(|value| ("image_format", value))
        .or_else(|| {
            context
                .import_settings()
                .get("decode_format")
                .map(|value| ("decode_format", value))
        })
        .or_else(|| {
            context
                .import_settings()
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
#[path = "tests/image_decode.rs"]
mod tests;
