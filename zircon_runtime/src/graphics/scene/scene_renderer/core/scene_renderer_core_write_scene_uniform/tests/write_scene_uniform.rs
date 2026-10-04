#[test]
fn static_cubemap_upload_prepares_frame_bindings_without_publishing_them() {
    let uniform_source = include_str!("../write_scene_uniform.rs");
    let upload = uniform_source
        .find("self.scene_environment_cubemap.ensure_uploaded(")
        .expect("source cubemap uploads must be encoded through the scene uniform path");

    let prepare = uniform_source
        .find("self.prepare_static_environment_bindings(device)")
        .unwrap();
    assert!(upload < prepare);
    assert!(
        !uniform_source[..uniform_source.find("#[cfg(test)]").unwrap()]
            .contains("self.scene_bind_group =")
    );
    assert!(uniform_source[upload..].contains("encoder,"));
    assert!(uniform_source[upload..].contains("&mut batch,"));
}

#[test]
fn provider_upgrade_downgrades_the_environment_variant_before_draw_construction() {
    let uniform_source = include_str!("../write_scene_uniform.rs");
    let prepare = uniform_source
        .find(".reflection_probes.prepare(")
        .expect("scene uniform update must prepare reflection providers");
    let provider_fallback = uniform_source
        .find(".requires_generic_environment_pbr()")
        .expect("scene uniform update must observe the provider upgrade");
    let deferred_binding_publication = uniform_source
        .find(".set_reflection_probe_bindings(")
        .expect("deferred lighting must receive the final provider binding lease");
    let variant_downgrade = uniform_source
        .find(".disable_environment_only_pbr_base_profile()")
        .expect("scene uniform update must select the generic environment variant");

    assert!(prepare < deferred_binding_publication);
    assert!(deferred_binding_publication < provider_fallback);
    assert!(provider_fallback < variant_downgrade);

    let render_source = include_str!("../../scene_renderer_core_render_scene/render_scene.rs");
    let write_uniform = render_source
        .find("self.write_scene_uniform(")
        .expect("direct rendering must update scene state before drawing");
    let build_draws = render_source
        .find(".build_mesh_draws(")
        .expect("direct rendering must build mesh draw commands");

    assert!(
        write_uniform < build_draws,
        "provider fallback must resolve before mesh variants are selected"
    );
}

#[test]
fn incomplete_realtime_ticket_keeps_procedural_environment_bindings() {
    let uniform_source = include_str!("../write_scene_uniform.rs");

    assert!(uniform_source.contains("prepared.uses_realtime_resources()"));
    assert!(uniform_source
        .contains("!realtime_ibl.is_some_and(RealtimeIblPreparedFrame::uses_realtime_resources)"));
}

#[test]
fn scene_constants_prepare_one_packed_frame_upload_batch() {
    let production = include_str!("../write_scene_uniform.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("scene uniform test boundary");

    assert!(production.contains("let payload: Arc<[u8]> = Arc::from(payload);"));
    assert!(production.contains("WgpuBufferUploadBatch::new()"));
    assert!(production.contains("Ok(batch)"));
    assert!(production.contains("GraphicsError::InvalidBufferUploadRange"));
    assert!(!production.contains(".expect(\"HitProxy scene uniform upload"));
    assert!(!production.contains(".expect(\"scene environment SH9 upload range"));
    assert!(!production.contains(".expect(\"scene uniform upload range"));
    assert!(!production.contains("enqueue_copy_buffer_upload_batch"));
    assert!(!production.contains("queue.write_buffer("));
}
