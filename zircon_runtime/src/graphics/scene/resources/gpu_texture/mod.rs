#[path = "gpu_texture_resource.rs"]
mod gpu_texture_resource;
#[path = "asset_texture.rs"]
mod gpu_texture_resource_from_asset;
#[path = "sampler_cache.rs"]
mod sampler_cache;

pub(crate) use gpu_texture_resource::GpuTextureResource;
pub(crate) use gpu_texture_resource_from_asset::texture_upload_support_from_device;
pub(in crate::graphics::scene::resources) use gpu_texture_resource_from_asset::GpuTextureUploadWork;
pub(crate) use sampler_cache::TextureSamplerCache;
