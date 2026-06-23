use super::*;

#[test]
fn render_product_dual_camera_rt_then_main() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_dual_rt_then_main",
        [1.0, 1.0, 1.0, 1.0],
    );
    let target_uri = "res://tests/camera-target/dual-camera-source.texture";
    let target = fixture.insert_srgb_render_target_texture(target_uri, fixture.viewport_size);
    let sampled_material = fixture.insert_texture_sampling_material(
        "res://materials/sample-dual-camera-source.zmaterial",
        target_uri,
    );

    let mut extract = fixture.frame_extract(vec![sampled_fullscreen_mesh_on_layer(
        901,
        fixture.model,
        sampled_material,
        3,
    )]);
    let texture_camera = texture_camera_descriptor(
        902,
        -1,
        target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(7),
        extract.view.camera.clone(),
    );
    let primary = primary_surface_camera_descriptor(
        903,
        1,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        RenderLayerSet::layer(3),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![texture_camera, primary]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface terminal frame should be capturable");
    let (target_size, target_rgba) = framework
        .read_output_target_texture_rgba_for_tests(target)
        .unwrap()
        .expect("source texture target should remain prepared after the primary surface submit");
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

    let target_pixels = (fixture.viewport_size.x * fixture.viewport_size.y) as usize;
    let target_red = dominant_red_pixels(&target_rgba);
    assert!(
        target_red > target_pixels * 3 / 4,
        "source texture target should be red before the later PrimarySurface samples it; red={target_red}, total={target_pixels}"
    );

    let center = RenderViewportRegion::new(UVec2::new(48, 28), UVec2::new(64, 64));
    let center_pixels = (center.size.x * center.size.y) as usize;
    let frame_red = dominant_red_pixels_in_region(&frame, center);
    let frame_blue = dominant_blue_pixels_in_region(&frame, center);
    assert!(
        frame_red > center_pixels / 3 && frame_blue < center_pixels / 10,
        "later PrimarySurface should sample the earlier texture target over its blue clear; red={frame_red}, blue={frame_blue}, total={center_pixels}"
    );
}

#[test]
fn custom_target_stacks_feed_later_primary_surface_materials_independently() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_multi_rt_sample",
        [1.0, 1.0, 1.0, 1.0],
    );
    let target_a_uri = "res://tests/camera-target/multi-stack-a.texture";
    let target_b_uri = "res://tests/camera-target/multi-stack-b.texture";
    let target_a = fixture.insert_srgb_render_target_texture(target_a_uri, fixture.viewport_size);
    let target_b = fixture.insert_srgb_render_target_texture(target_b_uri, fixture.viewport_size);
    let material_a = fixture.insert_texture_sampling_material(
        "res://materials/sample-output-target-a.zmaterial",
        target_a_uri,
    );
    let material_b = fixture.insert_texture_sampling_material(
        "res://materials/sample-output-target-b.zmaterial",
        target_b_uri,
    );

    let mut extract = fixture.frame_extract(vec![
        overlay_mesh(
            201,
            fixture.model,
            fixture.material,
            11,
            Vec4::new(0.0, 1.0, 0.0, 1.0),
        ),
        overlay_mesh(
            202,
            fixture.model,
            fixture.material,
            12,
            Vec4::new(1.0, 0.0, 0.0, 1.0),
        ),
        sampled_mesh(301, fixture.model, material_a, Vec3::new(-0.62, 0.0, 0.0)),
        sampled_mesh(302, fixture.model, material_b, Vec3::new(0.62, 0.0, 0.0)),
    ]);
    let texture_a_base = texture_camera_descriptor(
        101,
        -20,
        target_a,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(21),
        extract.view.camera.clone(),
    )
    .with_stack([102]);
    let texture_a_overlay = texture_camera_descriptor(
        102,
        -19,
        target_a,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(11),
        extract.view.camera.clone(),
    );
    let texture_b_base = texture_camera_descriptor(
        103,
        -10,
        target_b,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        true,
        RenderLayerSet::layer(22),
        extract.view.camera.clone(),
    )
    .with_stack([104]);
    let texture_b_overlay = texture_camera_descriptor(
        104,
        -9,
        target_b,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(12),
        extract.view.camera.clone(),
    );
    let primary = primary_surface_camera_descriptor(
        105,
        10,
        RenderCameraClear::Color(Vec4::new(0.015, 0.015, 0.015, 1.0)),
        RenderLayerSet::layer(0),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![
        texture_a_base,
        texture_a_overlay,
        texture_b_base,
        texture_b_overlay,
        primary,
    ]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());

    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface terminal frame should be capturable");
    let (target_a_size, target_a_rgba) = framework
        .read_output_target_texture_rgba_for_tests(target_a)
        .unwrap()
        .expect("first custom target stack should remain readable");
    let (target_b_size, target_b_rgba) = framework
        .read_output_target_texture_rgba_for_tests(target_b)
        .unwrap()
        .expect("second custom target stack should remain readable");
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(target_a_size, fixture.viewport_size);
    assert_eq!(target_b_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );

    let target_pixels = (fixture.viewport_size.x * fixture.viewport_size.y) as usize;
    let target_a_red = dominant_red_pixels(&target_a_rgba);
    let target_a_green = dominant_green_pixels(&target_a_rgba);
    let target_a_blue = dominant_blue_pixels(&target_a_rgba);
    assert!(
        target_a_red > target_pixels / 3 && target_a_green > target_pixels / 20,
        "first custom target should keep red base plus green overlay; red={target_a_red}, green={target_a_green}, blue={target_a_blue}, total={target_pixels}"
    );
    assert!(
        target_a_blue < target_pixels / 10,
        "second custom target blue clear must not leak into the first target; blue={target_a_blue}, total={target_pixels}"
    );

    let target_b_red = dominant_red_pixels(&target_b_rgba);
    let target_b_green = dominant_green_pixels(&target_b_rgba);
    let target_b_blue = dominant_blue_pixels(&target_b_rgba);
    assert!(
        target_b_blue > target_pixels / 3 && target_b_red > target_pixels / 20,
        "second custom target should keep blue base plus red overlay; red={target_b_red}, blue={target_b_blue}, green={target_b_green}, total={target_pixels}"
    );
    assert!(
        target_b_green < target_pixels / 10,
        "first custom target green overlay must not leak into the second target; green={target_b_green}, total={target_pixels}"
    );

    let left = RenderViewportRegion::new(UVec2::new(8, 18), UVec2::new(64, 84));
    let right = RenderViewportRegion::new(UVec2::new(88, 18), UVec2::new(64, 84));
    let left_pixels = (left.size.x * left.size.y) as usize;
    let right_pixels = (right.size.x * right.size.y) as usize;
    let left_red = dominant_red_pixels_in_region(&frame, left);
    let left_green = dominant_green_pixels_in_region(&frame, left);
    let left_blue = dominant_blue_pixels_in_region(&frame, left);
    let right_red = dominant_red_pixels_in_region(&frame, right);
    let right_green = dominant_green_pixels_in_region(&frame, right);
    let right_blue = dominant_blue_pixels_in_region(&frame, right);

    assert!(
        left_red > left_pixels / 16 && left_green > left_pixels / 32 && left_blue < left_pixels / 8,
        "left primary mesh should sample only the first red/green custom target; red={left_red}, green={left_green}, blue={left_blue}, total={left_pixels}"
    );
    assert!(
        right_blue > right_pixels / 16
            && right_red > right_pixels / 32
            && right_green < right_pixels / 8,
        "right primary mesh should sample only the second blue/red custom target; red={right_red}, green={right_green}, blue={right_blue}, total={right_pixels}"
    );
}

#[test]
fn custom_target_chain_feeds_later_texture_and_primary_surface_samples() {
    let fixture = RenderFixture::new(
        "graphics_camera_targets_chained_rt_sample",
        [1.0, 1.0, 1.0, 1.0],
    );
    let source_uri = "res://tests/camera-target/chain-source.texture";
    let intermediate_uri = "res://tests/camera-target/chain-intermediate.texture";
    let source_target =
        fixture.insert_srgb_render_target_texture(source_uri, fixture.viewport_size);
    let intermediate_target =
        fixture.insert_srgb_render_target_texture(intermediate_uri, fixture.viewport_size);
    let material_from_source = fixture.insert_texture_sampling_material(
        "res://materials/sample-chain-source.zmaterial",
        source_uri,
    );
    let material_from_intermediate = fixture.insert_texture_sampling_material(
        "res://materials/sample-chain-intermediate.zmaterial",
        intermediate_uri,
    );

    let mut extract = fixture.frame_extract(vec![
        sampled_fullscreen_mesh_on_layer(501, fixture.model, material_from_source, 13),
        sampled_fullscreen_mesh_on_layer(502, fixture.model, material_from_intermediate, 14),
    ]);
    let source_camera = texture_camera_descriptor(
        401,
        -30,
        source_target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(15),
        extract.view.camera.clone(),
    );
    let intermediate_camera = texture_camera_descriptor(
        402,
        -20,
        intermediate_target,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        true,
        RenderLayerSet::layer(13),
        extract.view.camera.clone(),
    );
    let primary = primary_surface_camera_descriptor(
        403,
        10,
        RenderCameraClear::Color(Vec4::new(0.015, 0.015, 0.015, 1.0)),
        RenderLayerSet::layer(14),
        extract.view.camera.clone(),
    );
    extract.view = extract
        .view
        .with_cameras(vec![source_camera, intermediate_camera, primary]);

    let (framework, viewport) = fixture.configured_framework(camera_target_product_profile());

    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface should sample the chained custom target");
    let (source_size, source_rgba) = framework
        .read_output_target_texture_rgba_for_tests(source_target)
        .unwrap()
        .expect("source custom target should remain readable");
    let (intermediate_size, intermediate_rgba) = framework
        .read_output_target_texture_rgba_for_tests(intermediate_target)
        .unwrap()
        .expect("intermediate custom target should remain readable");
    framework.destroy_viewport(viewport).unwrap();

    assert_eq!(source_size, fixture.viewport_size);
    assert_eq!(intermediate_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );

    let source_center = rgba_pixel_at(&source_rgba, source_size.x, UVec2::new(80, 60));
    let intermediate_center =
        rgba_pixel_at(&intermediate_rgba, intermediate_size.x, UVec2::new(80, 60));
    assert!(
        is_dominant_red(&source_center),
        "source custom target should keep the first red camera output; pixel={source_center:?}"
    );
    assert!(
        is_dominant_red(&intermediate_center),
        "intermediate custom target should sample the source target, not keep its blue clear at center; pixel={intermediate_center:?}"
    );

    let center = RenderViewportRegion::new(UVec2::new(48, 28), UVec2::new(64, 64));
    let center_pixels = (center.size.x * center.size.y) as usize;
    let frame_red = dominant_red_pixels_in_region(&frame, center);
    let frame_blue = dominant_blue_pixels_in_region(&frame, center);
    assert!(
        frame_red > center_pixels / 3 && frame_blue < center_pixels / 10,
        "later PrimarySurface should sample the intermediate custom target that already sampled the source; red={frame_red}, blue={frame_blue}, total={center_pixels}"
    );
}
