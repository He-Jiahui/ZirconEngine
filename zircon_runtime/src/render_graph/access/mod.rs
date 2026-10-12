//! 导出逻辑资源访问的 ID、范围、版本及意图；builder 编排声明，编译索引与执行绑定按精确访问身份读取。

#[path = "buffer_range.rs"]
mod buffer_range;
#[path = "resource_access_id.rs"]
mod resource_access_id;
#[path = "resource_access_intent.rs"]
mod resource_access_intent;
#[path = "resource_access_metadata.rs"]
mod resource_access_metadata;
#[path = "resource_access_range.rs"]
mod resource_access_range;
#[path = "shader_stages.rs"]
mod shader_stages;
#[path = "texture_aspect.rs"]
mod texture_aspect;
#[path = "texture_subresource_range.rs"]
mod texture_subresource_range;
#[path = "versioned_access_key.rs"]
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
