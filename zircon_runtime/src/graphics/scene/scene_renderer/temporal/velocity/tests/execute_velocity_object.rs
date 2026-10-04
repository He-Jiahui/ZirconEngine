use crate::core::framework::render::PostProcessGraphResourceNames;

#[test]
fn object_velocity_writes_graph_scene_velocity_resource() {
    assert_eq!(
        PostProcessGraphResourceNames::SCENE_VELOCITY,
        "scene-velocity"
    );
}

#[test]
fn object_velocity_binds_forward_shadow_receiver_group() {
    let source = include_str!("../execute_velocity_object.rs");

    assert!(source.contains(
        "create_forward_shadow_receiver_bind_group(\n                self.device,\n                self.shadow_atlas_resources,\n                None,\n                None,\n                None,\n            )"
    ));
    assert!(source.contains("pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[])"));
}

#[test]
fn object_velocity_skips_empty_load_store_pass_before_recording() {
    let source = include_str!("../execute_velocity_object.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("object velocity implementation");
    let empty_guard = implementation
        .find("attachment_ops.load == RenderGraphAttachmentLoadOp::Load")
        .expect("empty Load+Store guard");
    let begin_pass = implementation
        .find("let mut pass = self.encoder.begin_render_pass")
        .expect("object velocity render pass");

    assert!(empty_guard < begin_pass);
    assert!(implementation.contains("attachment_ops.store == RenderGraphAttachmentStoreOp::Store"));
}
