//! 导出逻辑资源访问的 ID、范围、版本及意图；builder 编排声明，编译索引与执行绑定按精确访问身份读取。

mod buffer_range;
mod resource_access_id;
mod resource_access_intent;
mod resource_access_metadata;
mod resource_access_range;
mod shader_stages;
mod texture_aspect;
mod texture_subresource_range;
mod versioned_access_key;

pub use buffer_range::RenderGraphBufferRange;
pub use resource_access_id::RenderGraphResourceAccessId;
pub use resource_access_intent::RenderGraphResourceAccessIntent;
pub use resource_access_metadata::RenderGraphResourceAccessMetadata;
pub use resource_access_range::RenderGraphResourceAccessRange;
pub use shader_stages::RenderGraphShaderStages;
pub use texture_aspect::RenderGraphTextureAspect;
pub use texture_subresource_range::RenderGraphTextureSubresourceRange;
pub use versioned_access_key::RenderGraphVersionedAccessKey;
