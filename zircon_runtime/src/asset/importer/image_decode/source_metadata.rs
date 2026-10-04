use super::{
    decode_error_value, stable_source_format_identity, texture_source_image_reader,
    AssetImportContext, AssetImportError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 热缓存探测所需的图像布局和格式身份，不包含像素缓冲。
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

// 环境图入口在昂贵的 HDR 像素解码前取得请求身份；格式须来自实际选用的 reader，
// 以免扩展名与内容不一致时复用错误的 IBL artifact。
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
