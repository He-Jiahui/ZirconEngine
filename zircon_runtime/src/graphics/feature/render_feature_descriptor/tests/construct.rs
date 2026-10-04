use super::*;
use crate::render_graph::{RenderResourceSchema, RenderTextureSchema};
use crate::rhi::{TextureFormat, TextureUsage};

#[test]
fn pass_resource_extension_preserves_its_explicit_schema() {
    let schema = RenderResourceSchema::texture(RenderTextureSchema::new(
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let descriptor =
        RenderFeatureDescriptor::new("typed-extension", Vec::new(), Vec::new(), Vec::new())
            .with_pass_read_texture_with_schema("existing-pass", "typed-extension-input", schema);

    let extension = descriptor
        .resource_extensions()
        .next()
        .expect("extension declaration");
    assert_eq!(extension.resource.schema, Some(schema));
}
