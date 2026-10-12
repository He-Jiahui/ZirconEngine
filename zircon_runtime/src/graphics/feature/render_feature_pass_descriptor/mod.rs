#[path = "construct.rs"]
mod construct;
#[path = "../../render_feature_pass_desc/external_resources.rs"]
mod external_resources;
#[path = "../../render_feature_pass_desc/render_feature_pass_desc.rs"]
mod render_feature_pass_descriptor;
#[path = "../render_feature_pass_desc/terminal_schema.rs"]
mod terminal_schema;

pub use crate::render_graph::{
    RenderBufferSchema, RenderResourceFallback, RenderResourceSchema, RenderTextureExtentPolicy,
    RenderTextureExtentReference, RenderTextureExtentRounding, RenderTextureSchema,
};
pub use render_feature_pass_descriptor::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind, RenderFeatureResourceVersion, RenderFeatureResourceWriteMode,
    RenderFeatureTextureViewAlias,
};
