const SOURCE: &str = include_str!("../render_scene.rs");

#[test]
fn direct_light_cookie_atlas_is_built_in_the_scene_packet_before_consumers() {
    let source = SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("render scene source should retain a test-module boundary");
    let encoder = source
        .find("let mut encoder = device.create_command_encoder")
        .expect("direct scene encoder");
    let profile_begin = source
        .find("self.mesh_pipelines.light_cookies.begin_profile_frame()")
        .expect("direct light-cookie profile begin");
    let rebuild = source
        .find("self.mesh_pipelines.light_cookies.rebuild(")
        .expect("direct light-cookie atlas producer");
    let timer = source
        .find("timer.begin_pass(&mut encoder, DIRECT_LIGHT_COOKIE_ATLAS_GPU_PASS)")
        .expect("direct light-cookie GPU timer");
    let mesh_build = source
        .find("self.advanced_plugin_resources.build_mesh_draws(")
        .expect("direct mesh consumer preparation");
    let submit = source
        .find(".submit_graphics_command_buffers_with_frame_diagnostics_and_surface(")
        .expect("direct scene submit");
    let profile_emit = source
        .find("self.mesh_pipelines.light_cookies.emit_profile_frame()")
        .expect("accepted direct light-cookie profile publication");

    assert!(source.contains("if !light_cookies.is_empty()"));
    assert!(encoder < profile_begin);
    assert!(profile_begin < timer);
    assert!(timer < rebuild);
    assert!(rebuild < mesh_build);
    assert!(mesh_build < submit);
    assert!(submit < profile_emit);
}
