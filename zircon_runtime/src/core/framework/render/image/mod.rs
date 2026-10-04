//! 图像资源契约门面：描述符先通过校验映射为视图形状，再由图形资源层创建纹理与采样器。
//! 本模块只约束维度、格式、用途和回退语义，不跨越资源所有权边界。
mod asset_usage;
mod color_space;
mod descriptor;
mod dimension;
mod fallback;
mod metadata;
mod metadata_validation;
mod sampler;
mod shape;
mod usage;

pub use asset_usage::RenderImageAssetUsage;
pub use color_space::RenderImageColorSpace;
pub use descriptor::RenderImageDescriptor;
pub use dimension::RenderImageDimension;
pub use fallback::RenderImageFallbackKind;
pub use metadata::{
    default_color_space_for_texture_usage, default_compression_for_texture_usage,
    default_mip_filter_for_texture_usage, SvtSettings, TextureCompressionTarget, TextureMetadata,
    TextureMipFilter, TextureMipPolicy, TextureNormalConvention, TextureUsageHint,
    TEXTURE_DEFAULT_MAX_ANISOTROPY, TEXTURE_STREAMING_MIN_DIMENSION,
    TEXTURE_SVT_DEFAULT_BORDER_SIZE, TEXTURE_SVT_DEFAULT_PAGE_SIZE,
};
pub use metadata_validation::{
    validate_texture_metadata, TextureMetadataDiagnostic, TextureMetadataDiagnosticSeverity,
};
pub use sampler::{RenderSamplerAddressMode, RenderSamplerDescriptor, RenderSamplerFilter};
pub use shape::{RenderImageShape, RenderImageShapeError, TextureExtent3D, TextureViewKind};
pub use usage::RenderImageUsage;
