use super::*;

#[test]
fn custom_target_two_viewport_stacks_preserve_independent_composites_before_primary_sample() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_dual_viewport_stack_sample",
        [1.0, 1.0, 1.0, 1.0],
    );
    let target_uri = "res://tests/camera-target/dual-viewport-stack.texture";
    let target = fixture.insert_srgb_render_target_texture(target_uri, fixture.viewport_size);
    let sampled_material = fixture.insert_texture_sampling_material(
        "res://materials/sample-dual-viewport-stack-target.zmaterial",
        target_uri,
    );

    let mut extract = fixture.frame_extract(vec![
        overlay_mesh(
            1301,
            fixture.model,
            fixture.material,
            41,
            Vec4::new(0.0, 1.0, 0.0, 1.0),
        ),
        overlay_mesh(
            1302,
            fixture.model,
            fixture.material,
            42,
            Vec4::new(1.0, 0.0, 0.0, 1.0),
        ),
        sampled_fullscreen_mesh(1303, fixture.model, sampled_material),
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
    let left_base = texture_camera_descriptor(
        1201,
        -40,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(51),
        extract.view.camera.clone(),
    )
    .with_viewport(left_viewport)
    .with_stack([1202]);
    let left_overlay = texture_camera_descriptor(
        1202,
        -39,
        target,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(41),
        extract.view.camera.clone(),
    );
    let right_base = texture_camera_descriptor(
        1203,
        -30,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        true,
        RenderLayerSet::layer(52),
        extract.view.camera.clone(),
    )
    .with_viewport(right_viewport)
    .with_stack([1204]);
    let right_overlay = texture_camera_descriptor(
        1204,
        -29,
        target,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(42),
        extract.view.camera.clone(),
    );
    let primary = primary_surface_camera_descriptor(
        1205,
        10,
        RenderCameraClear::Color(Vec4::new(0.015, 0.015, 0.015, 1.0)),
        RenderLayerSet::layer(0),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![
        left_base,
        left_overlay,
        right_base,
        right_overlay,
        primary,
    ]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());

    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface should sample both custom-target viewport stacks");
    let (target_size, target_rgba) = framework
        .read_output_target_texture_rgba_for_tests(target)
        .unwrap()
        .expect("dual viewport stack target should remain readable");
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
    let right_overlay_region = RenderViewportRegion::new(UVec2::new(96, 28), UVec2::new(40, 64));
    let right_base_edge_region = RenderViewportRegion::new(UVec2::new(140, 20), UVec2::new(16, 80));
    let left_overlay_pixels = (left_overlay_region.size.x * left_overlay_region.size.y) as usize;
    let left_edge_pixels = (left_base_edge_region.size.x * left_base_edge_region.size.y) as usize;
    let right_overlay_pixels = (right_overlay_region.size.x * right_overlay_region.size.y) as usize;
    let right_edge_pixels =
        (right_base_edge_region.size.x * right_base_edge_region.size.y) as usize;

    let target_left_green =
        dominant_green_pixels_in_rgba_region(&target_rgba, target_size, left_overlay_region);
    let target_left_edge_red =
        dominant_red_pixels_in_rgba_region(&target_rgba, target_size, left_base_edge_region);
    let target_left_blue =
        dominant_blue_pixels_in_rgba_region(&target_rgba, target_size, left_overlay_region);
    let target_right_red =
        dominant_red_pixels_in_rgba_region(&target_rgba, target_size, right_overlay_region);
    let target_right_edge_blue =
        dominant_blue_pixels_in_rgba_region(&target_rgba, target_size, right_base_edge_region);
    let target_right_green =
        dominant_green_pixels_in_rgba_region(&target_rgba, target_size, right_overlay_region);
    assert!(
        target_left_green > left_overlay_pixels / 10,
        "left stack should composite the green overlay inside its viewport; green={target_left_green}, total={left_overlay_pixels}"
    );
    assert!(
        target_left_edge_red > left_edge_pixels / 3,
        "left stack should keep its red base clear outside the overlay mesh; red={target_left_edge_red}, total={left_edge_pixels}"
    );
    assert!(
        target_left_blue < left_overlay_pixels / 10,
        "right stack blue clear must not leak into the left viewport composite; blue={target_left_blue}, total={left_overlay_pixels}"
    );
    assert!(
        target_right_red > right_overlay_pixels / 10,
        "right stack should composite the red overlay inside its viewport; red={target_right_red}, total={right_overlay_pixels}"
    );
    assert!(
        target_right_edge_blue > right_edge_pixels / 3,
        "right stack should keep its blue base clear outside the overlay mesh; blue={target_right_edge_blue}, total={right_edge_pixels}"
    );
    assert!(
        target_right_green < right_overlay_pixels / 10,
        "left stack green overlay must not leak into the right viewport composite; green={target_right_green}, total={right_overlay_pixels}"
    );

    let frame_left_green = dominant_green_pixels_in_region(&frame, left_overlay_region);
    let frame_left_edge_red = dominant_red_pixels_in_region(&frame, left_base_edge_region);
    let frame_left_blue = dominant_blue_pixels_in_region(&frame, left_overlay_region);
    let frame_right_red = dominant_red_pixels_in_region(&frame, right_overlay_region);
    let frame_right_edge_blue = dominant_blue_pixels_in_region(&frame, right_base_edge_region);
    let frame_right_green = dominant_green_pixels_in_region(&frame, right_overlay_region);
    assert!(
        frame_left_green > left_overlay_pixels / 10,
        "later PrimarySurface should sample the left stack green overlay; green={frame_left_green}, total={left_overlay_pixels}"
    );
    assert!(
        frame_left_edge_red > left_edge_pixels / 3 && frame_left_blue < left_overlay_pixels / 10,
        "later PrimarySurface should sample the left stack red base without right blue leakage; red={frame_left_edge_red}, blue={frame_left_blue}, edge_total={left_edge_pixels}, overlay_total={left_overlay_pixels}"
    );
    assert!(
        frame_right_red > right_overlay_pixels / 10,
        "later PrimarySurface should sample the right stack red overlay; red={frame_right_red}, total={right_overlay_pixels}"
    );
    assert!(
        frame_right_edge_blue > right_edge_pixels / 3 && frame_right_green < right_overlay_pixels / 10,
        "later PrimarySurface should sample the right stack blue base without left green leakage; blue={frame_right_edge_blue}, green={frame_right_green}, edge_total={right_edge_pixels}, overlay_total={right_overlay_pixels}"
    );
}
