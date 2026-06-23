use super::*;

#[test]
fn custom_target_late_producer_feeds_previous_frame_not_future_sample() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_late_producer_sample",
        [1.0, 1.0, 1.0, 1.0],
    );
    let target_uri = "res://tests/camera-target/late-producer.texture";
    let target = fixture.insert_srgb_render_target_texture(target_uri, fixture.viewport_size);
    let sampled_material = fixture.insert_texture_sampling_material(
        "res://materials/sample-late-producer-output-target.zmaterial",
        target_uri,
    );

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());

    let mut warmup_extract = fixture.frame_extract(Vec::new());
    let warmup_camera = texture_camera_descriptor(
        501,
        0,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(0.0, 1.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(18),
        warmup_extract.view.camera.clone(),
    );
    warmup_extract.view = warmup_extract.view.with_cameras(vec![warmup_camera]);
    framework
        .submit_frame_extract(viewport, warmup_extract)
        .unwrap();
    let (warmup_size, warmup_rgba) = framework
        .read_output_target_texture_rgba_for_tests(target)
        .unwrap()
        .expect("warmup custom target should be readable before the next frame");

    let mut extract = fixture.frame_extract(vec![sampled_fullscreen_mesh_on_layer(
        601,
        fixture.model,
        sampled_material,
        16,
    )]);
    let early_primary = primary_surface_camera_descriptor(
        602,
        -10,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        RenderLayerSet::layer(16),
        extract.view.camera.clone(),
    );
    let late_texture = texture_camera_descriptor(
        603,
        10,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(17),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![early_primary, late_texture]);

    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface should remain the viewport terminal frame");
    let (final_target_size, final_target_rgba) = framework
        .read_output_target_texture_rgba_for_tests(target)
        .unwrap()
        .expect("late producer custom target should be readable after the frame");
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(warmup_size, fixture.viewport_size);
    assert_eq!(final_target_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );

    let warmup_center = rgba_pixel_at(&warmup_rgba, warmup_size.x, UVec2::new(80, 60));
    let final_target_center =
        rgba_pixel_at(&final_target_rgba, final_target_size.x, UVec2::new(80, 60));
    assert!(
        is_dominant_green(&warmup_center),
        "warmup frame should seed the custom target with green previous-frame content; pixel={warmup_center:?}"
    );
    assert!(
        is_dominant_red(&final_target_center),
        "late producer should still write the custom target red after the early PrimarySurface samples; pixel={final_target_center:?}"
    );

    let center = RenderViewportRegion::new(UVec2::new(48, 28), UVec2::new(64, 64));
    let center_pixels = (center.size.x * center.size.y) as usize;
    let frame_green = dominant_green_pixels_in_region(&frame, center);
    let frame_red = dominant_red_pixels_in_region(&frame, center);
    assert!(
        frame_green > center_pixels / 3 && frame_red < center_pixels / 10,
        "early PrimarySurface should sample the previous-frame custom target content, not the later same-frame red producer; green={frame_green}, red={frame_red}, total={center_pixels}"
    );
}
