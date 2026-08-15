use super::support::*;

#[test]
fn host_window_template_bridge_keeps_workbench_reference_out_of_projection() {
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("builtin workbench host template should project");

    assert!(
        bridge
            .host_projection()
            .node_by_control_id(WORKBENCH_REFERENCE_IMAGE_CONTROL_ID)
            .is_none(),
        "host template must not project the full workbench reference PNG"
    );
    assert!(
        bridge
            .host_projection()
            .node_by_control_id(WORKBENCH_REFERENCE_WINDOW_CONTROL_ID)
            .is_none(),
        "window template must not project the full workbench reference PNG"
    );
}

#[test]
fn componentized_workbench_surface_paints_native_preview_pixels_and_interaction_state() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    let move_frame = bridge
        .control_frame("WorkbenchToolMove")
        .expect("move tool should have an arranged frame");

    let initial = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    assert_eq!(
        initial.len(),
        WORKBENCH_REFERENCE_WIDTH as usize * WORKBENCH_REFERENCE_HEIGHT as usize * 4
    );
    assert!(
        contains_at_least_distinct_non_black_pixels(&initial, 3),
        "native preview capture should contain multiple painted colors"
    );
    maybe_write_workbench_preview_png(&initial);

    bridge
        .dispatch_control_state("WorkbenchToolMove", UiEventKind::Click)
        .expect("tool dispatch should update bridge state")
        .expect("move tool should have a binding");

    let updated = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    let changed_pixels = changed_pixel_count_in_frame(&initial, &updated, move_frame);
    let updated_tool_pixel = first_non_black_pixel_in_frame(&updated, move_frame)
        .expect("selected tool frame should contain visible pixels");

    assert!(
        changed_pixels > 0,
        "state dispatch should repaint at least one pixel inside the selected tool frame"
    );
    assert_ne!(
        updated_tool_pixel,
        [0, 0, 0, 255],
        "selected tool background should not be an empty black placeholder"
    );
}
