use super::*;

#[test]
#[ignore]
fn export_normal_aware_temporal_rejection_wgpu_png() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping normal-aware temporal Wgpu product because no adapter is available");
        return;
    };
    let accepted =
        run_temporal_resolve_pixels(&device, &queue, TemporalCase::new(true, [0.0, 0.0], 0.0, 1));
    let mut checker_normals = [DEFAULT_NORMAL_CODE; (TEST_SIZE * TEST_SIZE) as usize];
    for y in 0..TEST_SIZE as usize {
        for x in 0..TEST_SIZE as usize {
            if (x + y) % 2 == 0 {
                checker_normals[y * TEST_SIZE as usize + x] = OPPOSITE_NORMAL_CODE;
            }
        }
    }
    let checker = run_temporal_resolve_pixels(
        &device,
        &queue,
        TemporalCase::new(true, [0.0, 0.0], 0.0, 1).with_current_normal_codes(checker_normals),
    );

    let mut normal_rejected_pixels = 0_usize;
    let mut normal_retained_pixels = 0_usize;
    let mut reprojection_border_pixels = 0_usize;
    for pixel_index in 0..accepted.len() {
        let x = pixel_index as u32 % TEST_SIZE;
        let y = pixel_index as u32 / TEST_SIZE;
        if x + 1 == TEST_SIZE || y + 1 == TEST_SIZE {
            assert_vec4_near(
                checker[pixel_index].lighting,
                accepted[pixel_index].lighting,
                0.01,
            );
            reprojection_border_pixels += 1;
            continue;
        }
        if checker[pixel_index].metadata[3] <= 0.3 {
            assert!(checker[pixel_index].lighting[0] + 0.04 < accepted[pixel_index].lighting[0]);
            normal_rejected_pixels += 1;
        } else {
            assert!(checker[pixel_index].metadata[3] > 0.75);
            assert_vec4_near(
                checker[pixel_index].lighting,
                accepted[pixel_index].lighting,
                0.01,
            );
            normal_retained_pixels += 1;
        }
    }
    assert!(normal_rejected_pixels >= 4);
    assert!(normal_retained_pixels >= 4);
    assert_eq!(normal_rejected_pixels + normal_retained_pixels, 9);
    assert_eq!(reprojection_border_pixels, 7);

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_temporal_normal_matrix_png(
        output_dir.join(NORMAL_REJECTION_PRODUCT_PNG),
        &accepted,
        &checker,
    );
    fs::write(
        output_dir.join(NORMAL_REJECTION_PRODUCT_REPORT),
        format!(
            "png={}\nleft=matching_normals_with_reprojection_border\nright=checkerboard_opposite_normals\nwidth=257\nheight=128\ngpu_output_grid=4x4_temporal_resolve_pixels\nnormal_rejected_interior_pixels={}\nnormal_retained_interior_pixels={}\nreprojection_border_pixels={}\nnormal_encoding=6bit_octahedral\ntemporal_metadata_y=source_times_64_plus_normal_code_exact_r16f_integer\nnormal_dot_threshold=0.75\ntrace_tile_words=8\ntrace_buffer_minimum_bytes=2304\nvalidated_scene_normal_inputs=single_sample_plus_msaa_surface_sample\nvalidated_temporal_behavior=depth_source_support_normal_motion_luma_rejection_plus_confidence\n",
            NORMAL_REJECTION_PRODUCT_PNG,
            normal_rejected_pixels,
            normal_retained_pixels,
            reprojection_border_pixels,
        ),
    )
    .unwrap();
}
