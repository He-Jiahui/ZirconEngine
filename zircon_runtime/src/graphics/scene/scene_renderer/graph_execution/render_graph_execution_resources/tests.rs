use super::RenderGraphExecutionResources;
use crate::render_graph::{
    QueueLane, RenderGraphBuilder, RenderGraphResource, RenderGraphResourceKind,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

#[test]
fn resource_registry_reports_missing_named_resources() {
    let resources = RenderGraphExecutionResources::new();

    assert_eq!(
        resources.require_texture_view("scene-color").unwrap_err(),
        "render graph execution texture resource `scene-color` is not bound"
    );
    assert_eq!(
        resources
            .require_buffer("particles.gpu.alive-indices")
            .unwrap_err(),
        "render graph execution buffer resource `particles.gpu.alive-indices` is not bound"
    );
}

#[test]
fn resource_registry_validates_declaration_kind_before_name_lookup() {
    let mut builder = RenderGraphBuilder::new("declaration-kind");
    let texture = builder.create_texture(TextureDesc::new(
        "scene-color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let buffer = builder.create_buffer(BufferDesc::new(
        "light-list",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let output = builder.import_external_resource("viewport-output");
    let pass = builder.add_pass("write", QueueLane::Graphics);
    builder.write_texture(pass, texture).unwrap();
    builder.write_buffer(pass, buffer).unwrap();
    builder.write_external(pass, output).unwrap();
    let graph = builder.compile().unwrap();
    let resources = RenderGraphExecutionResources::new();
    let texture_declaration = graph
        .resource_declaration(RenderGraphResource::TransientTexture(texture))
        .unwrap();
    let buffer_declaration = graph
        .resource_declaration(RenderGraphResource::TransientBuffer(buffer))
        .unwrap();

    assert_eq!(
        texture_declaration.kind,
        RenderGraphResourceKind::TransientTexture
    );
    assert_eq!(
        resources
            .require_buffer_for_declaration(texture_declaration)
            .unwrap_err(),
        "render graph execution resource `scene-color` is a texture declaration, not a buffer"
    );
    assert_eq!(
        buffer_declaration.kind,
        RenderGraphResourceKind::TransientBuffer
    );
    assert_eq!(
        resources
            .require_texture_view_for_declaration(buffer_declaration)
            .unwrap_err(),
        "render graph execution resource `light-list` is a buffer declaration, not a texture view"
    );
}

#[test]
fn transient_allocations_and_external_imports_have_separate_physical_owners() {
    let declaration = include_str!("mod.rs");
    let lifecycle = include_str!("lifecycle.rs");

    assert!(declaration.contains("owned_textures: BTreeMap<String, TransientTextureAllocation>"));
    assert!(declaration.contains("owned_buffers: BTreeMap<String, TransientBufferAllocation>"));
    assert!(declaration.contains("imported_textures: BTreeMap<String, wgpu::Texture>"));
    assert!(declaration.contains("buffers: BTreeMap<String, wgpu::Buffer>"));
    assert!(!declaration.contains("owned_texture_descs"));
    assert!(!declaration.contains("owned_texture_identities"));
    assert!(!declaration.contains("owned_buffer_descs"));

    let abort = lifecycle
        .split("fn release_transient_backings_into_pool")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn retire_transient_backings_after_submission")
                .next()
        })
        .expect("abort retirement owner must remain explicit");
    assert!(abort.contains("pool.release_texture(allocation)"));
    assert!(abort.contains("pool.release_buffer(allocation)"));
    let abort_clear = abort
        .find("self.clear_transient_binding_metadata()")
        .expect("abort path must clear frame-scoped access handles");
    let abort_release = abort
        .find("pool.release_texture(allocation)")
        .expect("abort path must return texture allocations");
    assert!(
        abort_clear < abort_release,
        "frame-scoped access handles must drop before abort retirement returns allocations"
    );

    let submitted = lifecycle
        .split("fn retire_transient_backings_after_submission")
        .nth(1)
        .expect("submitted retirement owner must remain explicit");
    assert!(submitted.contains("pool.release_texture_after_submission(allocation, ticket)"));
    assert!(submitted.contains("pool.release_buffer_after_submission(allocation, ticket)"));
    let submitted_clear = submitted
        .find("self.clear_transient_binding_metadata()")
        .expect("submitted path must clear frame-scoped access handles");
    let submitted_release = submitted
        .find("pool.release_texture_after_submission(allocation, ticket)")
        .expect("submitted path must retire texture allocations");
    assert!(
        submitted_clear < submitted_release,
        "frame-scoped access handles must drop before submitted allocations enter the pool"
    );
}

#[test]
fn raw_resource_mutators_are_scoped_to_the_scene_renderer_owner() {
    let binding = include_str!("binding.rs");

    for method in [
        "import_texture_view",
        "insert_buffer",
        "import_texture_alias",
    ] {
        assert!(
            binding.contains(&format!(
                "pub(in crate::graphics::scene::scene_renderer) fn {method}"
            )),
            "{method} must remain inside the scene-renderer resource owner"
        );
        assert!(
            !binding.contains(&format!("pub fn {method}")),
            "{method} must not expose a crate-wide raw WGPU mutator"
        );
    }
}
