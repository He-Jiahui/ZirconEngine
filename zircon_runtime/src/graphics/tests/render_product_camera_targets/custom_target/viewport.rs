use super::*;

#[test]
fn custom_target_viewport_regions_feed_later_primary_surface_sample() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_split_rt_sample",
        [1.0, 1.0, 1.0, 1.0],
    );
    let target_uri = "res://tests/camera-target/split-stack.texture";
    let target = fixture.insert_srgb_render_target_texture(target_uri, fixture.viewport_size);
    let sampled_material = fixture.insert_texture_sampling_material(
        "res://materials/sample-split-output-target.zmaterial",
        target_uri,
    );

    let mut extract = fixture.frame_extract(vec![sampled_fullscreen_mesh(
        401,
        fixture.model,
        sampled_material,
    )]);
    let half_width = fixture.viewport_size.x / 2;
    let left_region = RenderViewportRect::new(UVec2::ZERO, UVec2::new(half_width, 120));
    let right_region = RenderViewportRect::new(
        UVec2::new(half_width, 0),
        UVec2::new(fixture.viewport_size.x - half_width, 120),
    );
    let texture_left = texture_camera_descriptor(
        301,
        -20,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(31),
        extract.view.camera.clone(),
    )
    .with_viewport(left_region);
    let texture_right = texture_camera_descriptor(
        302,
        -19,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(0.0, 1.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(32),
        extract.view.camera.clone(),
    )
    .with_viewport(right_region);
    let primary = primary_surface_camera_descriptor(
        303,
        10,
        RenderCameraClear::Color(Vec4::new(0.015, 0.015, 0.015, 1.0)),
        RenderLayerSet::layer(0),
        extract.view.camera.clone(),
    );
    extract.view = extract
        .view
        .with_cameras(vec![texture_left, texture_right, primary]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());

    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface should sample the prepared split custom target");
    let (target_size, target_rgba) = framework
        .read_output_target_texture_rgba_for_tests(target)
        .unwrap()
        .expect("split custom target should remain readable");
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(target_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );

    let target_left_pixel = rgba_pixel_at(&target_rgba, target_size.x, UVec2::new(24, 60));
    let target_right_pixel = rgba_pixel_at(&target_rgba, target_size.x, UVec2::new(136, 60));
    assert!(
        is_dominant_red(&target_left_pixel),
        "left custom target viewport center should stay red; pixel={target_left_pixel:?}"
    );
    assert!(
        is_dominant_green(&target_right_pixel),
        "right custom target viewport center should stay green; pixel={target_right_pixel:?}"
    );

    let frame_left = RenderViewportRegion::new(UVec2::new(12, 20), UVec2::new(56, 80));
    let frame_right = RenderViewportRegion::new(UVec2::new(92, 20), UVec2::new(56, 80));
    let frame_region_pixels = (frame_left.size.x * frame_left.size.y) as usize;
    let frame_left_red = dominant_red_pixels_in_region(&frame, frame_left);
    let frame_left_green = dominant_green_pixels_in_region(&frame, frame_left);
    let frame_right_red = dominant_red_pixels_in_region(&frame, frame_right);
    let frame_right_green = dominant_green_pixels_in_region(&frame, frame_right);
    assert!(
        frame_left_red > frame_region_pixels / 3 && frame_left_green < frame_region_pixels / 6,
        "later PrimarySurface should sample the red left half of the custom target; red={frame_left_red}, green={frame_left_green}, total={frame_region_pixels}"
    );
    assert!(
        frame_right_green > frame_region_pixels / 3 && frame_right_red < frame_region_pixels / 6,
        "later PrimarySurface should sample the green right half of the custom target; green={frame_right_green}, red={frame_right_red}, total={frame_region_pixels}"
    );
}

#[test]
fn custom_target_overlay_inherits_base_viewport_region_before_primary_sample() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_overlay_viewport_sample",
        [1.0, 1.0, 1.0, 1.0],
    );
    let target_uri = "res://tests/camera-target/overlay-viewport.texture";
    let target = fixture.insert_srgb_render_target_texture(target_uri, fixture.viewport_size);
    let sampled_material = fixture.insert_texture_sampling_material(
        "res://materials/sample-overlay-viewport-output-target.zmaterial",
        target_uri,
    );

    let mut extract = fixture.frame_extract(vec![
        overlay_mesh(
            701,
            fixture.model,
            fixture.material,
            19,
            Vec4::new(0.0, 1.0, 0.0, 1.0),
        ),
        sampled_fullscreen_mesh(702, fixture.model, sampled_material),
    ]);
    let half_width = fixture.viewport_size.x / 2;
    let left_viewport =
        RenderViewportRect::new(UVec2::ZERO, UVec2::new(half_width, fixture.viewport_size.y));
    let right_viewport = RenderViewportRect::new(
        UVec2::new(half_width, 0),
        UVec2::new(
            fixture.viewport_size.x - half_width,
            fixture.viewport_size.y,
        ),
    );
    let right_base = texture_camera_descriptor(
        801,
        -30,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        true,
        RenderLayerSet::layer(20),
        extract.view.camera.clone(),
    )
    .with_viewport(right_viewport);
    let left_base = texture_camera_descriptor(
        802,
        -20,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(21),
        extract.view.camera.clone(),
    )
    .with_viewport(left_viewport)
    .with_stack([803]);
    let left_overlay = texture_camera_descriptor(
        803,
        -19,
        target,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(19),
        extract.view.camera.clone(),
    );
    let primary = primary_surface_camera_descriptor(
        804,
        10,
        RenderCameraClear::Color(Vec4::new(0.015, 0.015, 0.015, 1.0)),
        RenderLayerSet::layer(0),
        extract.view.camera.clone(),
    );
    extract.view = extract
        .view
        .with_cameras(vec![right_base, left_base, left_overlay, primary]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());

    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface should sample the overlay viewport-limited target");
    let (target_size, target_rgba) = framework
        .read_output_target_texture_rgba_for_tests(target)
        .unwrap()
        .expect("overlay viewport custom target should remain readable");
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(target_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );

    let left_overlay_region = RenderViewportRegion::new(UVec2::new(24, 28), UVec2::new(40, 64));
    let left_base_edge_region = RenderViewportRegion::new(UVec2::new(4, 20), UVec2::new(18, 80));
    let right_near_region = RenderViewportRegion::new(UVec2::new(84, 28), UVec2::new(48, 64));
    let left_overlay_pixels = (left_overlay_region.size.x * left_overlay_region.size.y) as usize;
    let left_edge_pixels = (left_base_edge_region.size.x * left_base_edge_region.size.y) as usize;
    let right_pixels = (right_near_region.size.x * right_near_region.size.y) as usize;

    let target_left_green =
        dominant_green_pixels_in_rgba_region(&target_rgba, target_size, left_overlay_region);
    let target_left_edge_red =
        dominant_red_pixels_in_rgba_region(&target_rgba, target_size, left_base_edge_region);
    let target_right_blue =
        dominant_blue_pixels_in_rgba_region(&target_rgba, target_size, right_near_region);
    let target_right_green =
        dominant_green_pixels_in_rgba_region(&target_rgba, target_size, right_near_region);
    assert!(
        target_left_green > left_overlay_pixels / 10,
        "left overlay should draw green inside the Base viewport; green={target_left_green}, total={left_overlay_pixels}"
    );
    assert!(
        target_left_edge_red > left_edge_pixels / 3,
        "left Base red clear should remain visible outside the overlay mesh; red={target_left_edge_red}, total={left_edge_pixels}"
    );
    assert!(
        target_right_blue > right_pixels / 2 && target_right_green < right_pixels / 10,
        "Overlay must inherit the left Base viewport and not contaminate the right custom-target half; blue={target_right_blue}, green={target_right_green}, total={right_pixels}"
    );

    let frame_left_green = dominant_green_pixels_in_region(&frame, left_overlay_region);
    let frame_right_blue = dominant_blue_pixels_in_region(&frame, right_near_region);
    let frame_right_green = dominant_green_pixels_in_region(&frame, right_near_region);
    assert!(
        frame_left_green > left_overlay_pixels / 10,
        "later PrimarySurface should sample the left viewport-limited overlay from the prepared target; green={frame_left_green}, total={left_overlay_pixels}"
    );
    assert!(
        frame_right_blue > right_pixels / 3 && frame_right_green < right_pixels / 10,
        "later PrimarySurface should keep sampling the blue right half when the left overlay inherits its Base viewport; blue={frame_right_blue}, green={frame_right_green}, total={right_pixels}"
    );
}
