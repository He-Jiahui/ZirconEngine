use std::io::Cursor;

use image::{GenericImageView, ImageFormat};

use super::{AssetImportContext, AssetImportError};

/// RGBA8 image data decoded from a source image before texture descriptor overrides apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedTextureImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Decode image bytes using Bevy-style source format selection.
///
/// The default path trusts the source extension. Import settings can request
/// `image_format = "guess"` or a concrete image crate format such as `jpeg`.
pub fn decode_texture_source_image(
    context: &AssetImportContext,
) -> Result<DecodedTextureImage, AssetImportError> {
    let setting = image_format_setting(context)?;
    let image = match setting {
        ImageFormatSetting::FromExtension { extension, format } => {
            image::load_from_memory_with_format(&context.source_bytes, format).map_err(|error| {
                decode_error_value(
                    context,
                    format!("decode image as `{extension}` from extension: {error}"),
                )
            })?
        }
        ImageFormatSetting::Format { token, format } => {
            image::load_from_memory_with_format(&context.source_bytes, format).map_err(|error| {
                decode_error_value(
                    context,
                    format!("decode image as explicit `{token}`: {error}"),
                )
            })?
        }
        ImageFormatSetting::Guess => {
            let reader = image::ImageReader::new(Cursor::new(&context.source_bytes))
                .with_guessed_format()
                .map_err(|error| {
                    decode_error_value(context, format!("guess image format from bytes: {error}"))
                })?;
            reader.decode().map_err(|error| {
                decode_error_value(context, format!("decode guessed image: {error}"))
            })?
        }
    };
    let (width, height) = image.dimensions();
    Ok(DecodedTextureImage {
        width,
        height,
        rgba: image.to_rgba8().into_raw(),
    })
}

enum ImageFormatSetting {
    FromExtension {
        extension: String,
        format: ImageFormat,
    },
    Format {
        token: String,
        format: ImageFormat,
    },
    Guess,
}

fn image_format_setting(
    context: &AssetImportContext,
) -> Result<ImageFormatSetting, AssetImportError> {
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
    let normalized = token.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "from_extension" | "extension" => image_format_from_extension(context),
        "guess" | "from_bytes" | "bytes" => Ok(ImageFormatSetting::Guess),
        _ => image_format_from_token(normalized.as_str())
            .map(|format| ImageFormatSetting::Format {
                token: token.to_string(),
                format,
            })
            .ok_or_else(|| {
                decode_error_value(
                    context,
                    format!("unsupported image import setting `{key} = {token}`"),
                )
            }),
    }
}

fn image_format_from_token(token: &str) -> Option<ImageFormat> {
    match token {
        "open_exr" | "openexr" => Some(ImageFormat::OpenExr),
        "radiance_hdr" | "radiance" => Some(ImageFormat::Hdr),
        "portable_anymap" | "portable_bitmap" | "portable_graymap" | "portable_pixmap" => {
            Some(ImageFormat::Pnm)
        }
        _ => ImageFormat::from_extension(token),
    }
}

fn image_format_from_extension(
    context: &AssetImportContext,
) -> Result<ImageFormatSetting, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| decode_error_value(context, "image source has no file extension"))?;
    ImageFormat::from_extension(extension)
        .map(|format| ImageFormatSetting::FromExtension {
            extension: extension.to_ascii_lowercase(),
            format,
        })
        .ok_or_else(|| {
            decode_error_value(
                context,
                format!("unsupported image extension `{extension}`"),
            )
        })
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
