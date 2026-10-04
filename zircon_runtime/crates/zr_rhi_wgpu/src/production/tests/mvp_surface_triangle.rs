#[test]
fn direct_surface_triangle_has_no_native_queue_or_offscreen_color_owner() {
    let source = include_str!("../mvp_surface_triangle.rs");

    assert!(source.contains("frame.target()"));
    assert!(source.contains("frame.default_view()"));
    assert!(source.contains("frame.frame().device_id()"));
    assert!(source.contains("device.device_id()"));
    assert!(source.contains("device.submit(command_list)"));
    assert!(source.contains("device.present_surface_frame(frame.clone(), ticket)"));
    assert!(source.contains("device.discard_surface_frame(frame)"));
    assert!(!source.contains(concat!("wgpu", "::")));
    assert!(!source.contains(concat!("TextureUsage", "::COPY_SRC")));
}
