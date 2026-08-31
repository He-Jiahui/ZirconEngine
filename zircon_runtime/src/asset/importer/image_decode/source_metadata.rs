use super::{
    decode_error_value, stable_source_format_identity, texture_source_image_reader,
    AssetImportContext, AssetImportError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextureSourceImageMetadata {
    width: u32,
    height: u32,
    format_identity: u32,
}

impl TextureSourceImageMetadata {
    pub(crate) const fn width(self) -> u32 {
        self.width
    }

    pub(crate) const fn height(self) -> u32 {
        self.height
    }

    pub(crate) const fn format_identity(self) -> u32 {
        self.format_identity
    }
}

pub(crate) fn decode_texture_source_image_metadata(
    context: &AssetImportContext,
) -> Result<TextureSourceImageMetadata, AssetImportError> {
    let reader = texture_source_image_reader(context)?;
    let format_identity = resolved_source_format_identity(context, reader.format())?;
    let (width, height) = reader.into_dimensions().map_err(|error| {
        decode_error_value(
            context,
            format!("read image dimensions without pixels: {error}"),
        )
    })?;
    Ok(TextureSourceImageMetadata {
        width,
        height,
        format_identity,
    })
}

pub(crate) fn texture_source_image_format_identity(
    context: &AssetImportContext,
) -> Result<u32, AssetImportError> {
    let reader = texture_source_image_reader(context)?;
    resolved_source_format_identity(context, reader.format())
}

fn resolved_source_format_identity(
    context: &AssetImportContext,
    format: Option<image::ImageFormat>,
) -> Result<u32, AssetImportError> {
    let format = format.ok_or_else(|| {
        decode_error_value(context, "resolved image reader has no decoder format")
    })?;
    stable_source_format_identity(format).ok_or_else(|| {
        decode_error_value(
            context,
            format!("resolved image decoder format `{format:?}` has no stable identity"),
        )
    })
}
