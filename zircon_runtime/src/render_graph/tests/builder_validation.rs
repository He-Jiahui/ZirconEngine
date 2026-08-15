use crate::render_graph::{
    QueueLane, RenderGraphBuilder, RenderGraphError, RenderGraphResourceKind, RgTextureHandle,
};

#[test]
fn builder_rejects_foreign_resource_handles_before_index_lookup() {
    let mut builder = RenderGraphBuilder::new("invalid-resource-handle");
    let pass = builder.add_pass("write", QueueLane::Graphics);

    let error = builder
        .write_texture(pass, RgTextureHandle::from_index(usize::MAX, 0))
        .unwrap_err();

    assert!(matches!(
        error,
        RenderGraphError::ForeignResource {
            kind: RenderGraphResourceKind::TransientTexture,
            index: usize::MAX,
            ..
        }
    ));
}

#[test]
fn builder_resource_validation_uses_constant_time_handle_bounds() {
    let source = include_str!("../builder.rs");

    assert!(source.contains("if handle_generation != self.generation"));
    assert!(source.contains("handle.index() < self.next_texture"));
    assert!(source.contains("handle.index() < self.next_buffer"));
    assert!(source.contains("handle.index() < self.next_external_resource"));
    assert!(!source.contains("self.resources.iter().any(|node| node.resource == resource)"));
}
