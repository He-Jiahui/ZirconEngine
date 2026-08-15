#[test]
fn depth_prepass_binds_forward_shadow_receiver_layout_slot() {
    let source = include_str!("mesh_recording.rs");

    assert!(source.contains("record_depth_prepass_to_resources"));
    assert!(source.contains("create_forward_shadow_receiver_bind_group"));
    assert!(source.contains("bind_forward_shadow_receiver_if_needed"));
}

#[test]
fn disabled_forward_volumetric_params_buffer_is_cache_owned() {
    let cache_source = include_str!("../../../mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs");
    let construct_source = include_str!("../../../mesh/mesh_pipeline_cache/construct.rs");
    let binding_source =
        include_str!("../../../mesh/mesh_pipeline_cache/forward_shadow_receiver.rs");
    let cache_field = cache_source
        .lines()
        .zip(cache_source.lines().skip(1))
        .any(|(field, ty)| {
            field
                .trim_end()
                .ends_with("forward_volumetric_disabled_params_buffer:")
                && ty.trim() == "wgpu::Buffer,"
        });

    assert!(cache_field);
    assert!(construct_source.contains("let forward_volumetric_disabled_params_buffer ="));
    assert!(construct_source.contains("forward_volumetric_disabled_params_buffer,"));
    assert!(binding_source.contains("&self.forward_volumetric_disabled_params_buffer"));
    assert!(!binding_source.contains("create_disabled_params_buffer("));
}

#[test]
fn taa_reactive_mask_uses_one_clear_and_draw_pass_only_when_commands_exist() {
    let mesh_recording = include_str!("mesh_recording.rs");
    let reactive_mask_start = mesh_recording
        .find("fn record_taa_reactive_mask_mesh_to_resource")
        .expect("reactive-mask mesh recording must exist");
    let reactive_mask_end = mesh_recording[reactive_mask_start..]
        .find("\nfn mesh_stage_attachment_ops")
        .map(|offset| reactive_mask_start + offset)
        .expect("reactive-mask mesh recording must end before mesh-stage helpers");
    let reactive_mask_recording = &mesh_recording[reactive_mask_start..reactive_mask_end];
    let empty_stream = reactive_mask_recording
        .find("if stream.is_empty()")
        .expect("reactive-mask recording must skip empty streams");
    let begin_pass = reactive_mask_recording
        .find("begin_render_pass")
        .expect("non-empty reactive-mask streams must record one pass");

    assert!(empty_stream < begin_pass);
    assert!(reactive_mask_recording.contains("RenderGraphAttachmentOps::clear_store()"));
    assert!(reactive_mask_recording.contains("drop(pass);"));
    assert!(reactive_mask_recording.contains("record_taa_reactive_mask_encoding"));

    let resource_binding = include_str!(
        "../../../core/scene_renderer_core_render_compiled_scene/render/bind_taa_reactive_mask_graph_resource.rs"
    );
    assert!(resource_binding.contains("taa_reactive_mask_stream().is_empty()"));
    assert!(resource_binding.contains("black_texture_view()"));
    assert!(
        !mesh_recording.contains("record_taa_reactive_mask_clear_to_resource"),
        "the mesh writer owns the single clear-and-draw pass"
    );
}
