use super::*;

#[test]
fn sdf_draw_plan_clips_to_text_frame_without_explicit_clip() {
    let text = text_batch("AAAA", UiFrame::new(8.0, 12.0, 24.0, 20.0));
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);

    let vertices = build_sdf_vertices(
        std::slice::from_ref(&text),
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 64),
    );

    let max_x = vertices
        .iter()
        .map(|vertex| vertex.position[0])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(!vertices.is_empty());
    assert!(vertices.len() <= 24);
    assert!(max_x <= pixel_to_ndc_x(text.frame.right(), 128.0) + 0.0001);
}

#[test]
fn sdf_draw_plan_clips_glyph_vertices_and_uvs() {
    let mut text = text_batch("A", UiFrame::new(8.0, 12.0, 64.0, 20.0));
    text.clip_frame = Some(UiFrame::new(12.0, 12.0, 32.0, 20.0));
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);

    let vertices = build_sdf_vertices(
        &[text],
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 64),
    );

    assert_eq!(vertices.len(), 6);
    assert!(vertices[0].position[0] > -0.875);
    assert!(vertices[0].uv[0] > 0.0);
}

#[test]
fn sdf_draw_plan_applies_text_alignment_inside_frame() {
    let mut centered = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    centered.text_align = UiTextAlign::Center;
    let centered_plan = plan_sdf_atlas(std::slice::from_ref(&centered));
    let (mut centered_bake, mut centered_database, centered_assets, centered_atlas_bake) =
        bake_atlas(&centered_plan);
    let centered_width = text_advance(
        &mut centered_bake,
        &mut centered_database,
        &centered_assets,
        &centered,
    );
    let centered_first = centered_bake.measure_glyph(
        'A',
        centered.font.as_deref(),
        centered.font_family.as_deref(),
        centered.font_weight,
        centered.font_size,
        &mut centered_database,
        &centered_assets,
    );

    let centered_vertices = build_sdf_vertices(
        std::slice::from_ref(&centered),
        &centered_plan,
        &centered_atlas_bake,
        &mut centered_bake,
        &mut centered_database,
        &centered_assets,
        UVec2::new(128, 64),
    );

    let expected_centered_x = centered.frame.x
        + (centered.frame.width - centered_width) * 0.5
        + centered_first.bitmap_left;
    assert!(
        (centered_vertices[0].position[0] - pixel_to_ndc_x(expected_centered_x, 128.0)).abs()
            < 0.0001
    );

    let mut right_aligned = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    right_aligned.text_align = UiTextAlign::Right;
    let right_plan = plan_sdf_atlas(std::slice::from_ref(&right_aligned));
    let (mut right_bake, mut right_database, right_assets, right_atlas_bake) =
        bake_atlas(&right_plan);
    let right_width = text_advance(
        &mut right_bake,
        &mut right_database,
        &right_assets,
        &right_aligned,
    );
    let right_first = right_bake.measure_glyph(
        'A',
        right_aligned.font.as_deref(),
        right_aligned.font_family.as_deref(),
        right_aligned.font_weight,
        right_aligned.font_size,
        &mut right_database,
        &right_assets,
    );

    let right_vertices = build_sdf_vertices(
        std::slice::from_ref(&right_aligned),
        &right_plan,
        &right_atlas_bake,
        &mut right_bake,
        &mut right_database,
        &right_assets,
        UVec2::new(128, 64),
    );

    let expected_right_x = right_aligned.frame.right() - right_width + right_first.bitmap_left;
    assert!(
        (right_vertices[0].position[0] - pixel_to_ndc_x(expected_right_x, 128.0)).abs() < 0.0001
    );
}

#[test]
fn sdf_draw_plan_maps_start_end_through_rtl_direction() {
    let mut start_aligned = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    start_aligned.text_align = UiTextAlign::Start;
    start_aligned.text_direction = UiTextDirection::RightToLeft;
    let start_plan = plan_sdf_atlas(std::slice::from_ref(&start_aligned));
    let (mut start_bake, mut start_database, start_assets, start_atlas_bake) =
        bake_atlas(&start_plan);
    let start_width = text_advance(
        &mut start_bake,
        &mut start_database,
        &start_assets,
        &start_aligned,
    );
    assert!(
        (aligned_text_start_x(&start_aligned, start_width)
            - (start_aligned.frame.right() - start_width))
            .abs()
            < 0.0001
    );
    let start_first = start_bake.measure_glyph(
        'A',
        start_aligned.font.as_deref(),
        start_aligned.font_family.as_deref(),
        start_aligned.font_weight,
        start_aligned.font_size,
        &mut start_database,
        &start_assets,
    );

    let start_vertices = build_sdf_vertices(
        std::slice::from_ref(&start_aligned),
        &start_plan,
        &start_atlas_bake,
        &mut start_bake,
        &mut start_database,
        &start_assets,
        UVec2::new(128, 64),
    );

    let expected_start_x = start_aligned.frame.right() - start_width + start_first.bitmap_left;
    assert!(
        (start_vertices[0].position[0] - pixel_to_ndc_x(expected_start_x, 128.0)).abs() < 0.0001
    );

    let mut end_aligned = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    end_aligned.text_align = UiTextAlign::End;
    end_aligned.text_direction = UiTextDirection::RightToLeft;
    let end_plan = plan_sdf_atlas(std::slice::from_ref(&end_aligned));
    let (mut end_bake, mut end_database, end_assets, end_atlas_bake) = bake_atlas(&end_plan);
    let end_width = text_advance(&mut end_bake, &mut end_database, &end_assets, &end_aligned);
    assert!((aligned_text_start_x(&end_aligned, end_width) - end_aligned.frame.x).abs() < 0.0001);
    let end_first = end_bake.measure_glyph(
        'A',
        end_aligned.font.as_deref(),
        end_aligned.font_family.as_deref(),
        end_aligned.font_weight,
        end_aligned.font_size,
        &mut end_database,
        &end_assets,
    );

    let end_vertices = build_sdf_vertices(
        std::slice::from_ref(&end_aligned),
        &end_plan,
        &end_atlas_bake,
        &mut end_bake,
        &mut end_database,
        &end_assets,
        UVec2::new(128, 64),
    );

    let expected_end_x = (end_aligned.frame.x + end_first.bitmap_left).max(end_aligned.frame.x);
    assert!((end_vertices[0].position[0] - pixel_to_ndc_x(expected_end_x, 128.0)).abs() < 0.0001);
}

#[test]
fn sdf_draw_plan_justifies_word_gaps_inside_frame() {
    let mut justified = text_batch("A B", UiFrame::new(8.0, 12.0, 96.0, 20.0));
    justified.text_align = UiTextAlign::Justify;
    let justified_plan = plan_sdf_atlas(std::slice::from_ref(&justified));
    let (mut justified_bake, mut justified_database, justified_assets, justified_atlas_bake) =
        bake_atlas(&justified_plan);
    let a = justified_bake.measure_glyph(
        'A',
        justified.font.as_deref(),
        justified.font_family.as_deref(),
        justified.font_weight,
        justified.font_size,
        &mut justified_database,
        &justified_assets,
    );
    let b = justified_bake.measure_glyph(
        'B',
        justified.font.as_deref(),
        justified.font_family.as_deref(),
        justified.font_weight,
        justified.font_size,
        &mut justified_database,
        &justified_assets,
    );

    let vertices = build_sdf_vertices(
        std::slice::from_ref(&justified),
        &justified_plan,
        &justified_atlas_bake,
        &mut justified_bake,
        &mut justified_database,
        &justified_assets,
        UVec2::new(160, 64),
    );

    let expected_second_glyph_x = justified.frame.right() - b.advance + b.bitmap_left;
    assert_eq!(vertices.len(), 12);
    let expected_first_glyph_x = (justified.frame.x + a.bitmap_left).max(justified.frame.x);
    assert!(
        (vertices[0].position[0] - pixel_to_ndc_x(expected_first_glyph_x, 160.0)).abs() < 0.0001
    );
    assert!(
        (vertices[6].position[0] - pixel_to_ndc_x(expected_second_glyph_x, 160.0)).abs() < 0.0001
    );
}

#[test]
fn sdf_draw_plan_prefers_resolved_layout_advances_for_parity() {
    let mut text = text_batch("ABC", UiFrame::new(4.0, 8.0, 48.0, 20.0));
    text.glyph_advances = vec![5.0, 17.0, 9.0];

    let advances = resolve_sdf_glyph_advances(&text, vec![16.0, 16.0, 16.0], 48.0);

    assert_eq!(advances, vec![5.0, 17.0, 9.0]);
}

#[test]
fn sdf_draw_plan_maps_resolved_grapheme_advances_to_sdf_char_run() {
    let mut text = text_batch("e\u{301}A", UiFrame::new(4.0, 8.0, 48.0, 20.0));
    text.glyph_advances = vec![19.0, 11.0];

    let advances = resolve_sdf_glyph_advances(&text, vec![16.0, 6.0, 16.0], 38.0);

    assert_eq!(advances, vec![0.0, 19.0, 11.0]);
    assert!((advances.iter().sum::<f32>() - text.glyph_advances.iter().sum::<f32>()).abs() < 0.1);
}

#[test]
fn sdf_draw_plan_preserves_subpixel_glyph_advance_spacing() {
    let line_frame = text_frame_device_origin(UiFrame::new(4.49, 8.51, 96.0, 20.0));
    let glyph = RunGlyph {
        slot_index: Some(0),
        metrics: SdfGlyphMetrics {
            bitmap_width: 8,
            bitmap_height: 12,
            bitmap_left: 0.25,
            bitmap_bottom: 1.5,
            advance: 7.5,
            ascent: 11.0,
        },
        atlas_bitmap_width: 8,
        atlas_bitmap_height: 12,
        visible: true,
        screen_px_range: sdf_screen_px_range(12.0, SdfBakeParams::default()),
    };
    let first = horizontal_sdf_glyph_frame(line_frame.x + 0.2, 20.75, &glyph);
    let second = horizontal_sdf_glyph_frame(line_frame.x + 7.7, 20.75, &glyph);

    assert_eq!(line_frame.x, 4.0);
    assert_eq!(line_frame.y, 9.0);
    assert!((first.x - 4.45).abs() < 0.0001);
    assert!((second.x - first.x - glyph.metrics.advance).abs() < 0.0001);
    assert!((first.y - 7.25).abs() < 0.0001);
}

#[test]
fn sdf_draw_plan_trims_edge_spaces_for_justify() {
    let mut justified = text_batch(" A B ", UiFrame::new(8.0, 12.0, 112.0, 20.0));
    justified.text_align = UiTextAlign::Justify;
    let justified_plan = plan_sdf_atlas(std::slice::from_ref(&justified));
    let (mut justified_bake, mut justified_database, justified_assets, justified_atlas_bake) =
        bake_atlas(&justified_plan);
    let leading_space = justified_bake.measure_glyph(
        ' ',
        justified.font.as_deref(),
        justified.font_family.as_deref(),
        justified.font_weight,
        justified.font_size,
        &mut justified_database,
        &justified_assets,
    );
    let a = justified_bake.measure_glyph(
        'A',
        justified.font.as_deref(),
        justified.font_family.as_deref(),
        justified.font_weight,
        justified.font_size,
        &mut justified_database,
        &justified_assets,
    );
    let b = justified_bake.measure_glyph(
        'B',
        justified.font.as_deref(),
        justified.font_family.as_deref(),
        justified.font_weight,
        justified.font_size,
        &mut justified_database,
        &justified_assets,
    );

    let vertices = build_sdf_vertices(
        std::slice::from_ref(&justified),
        &justified_plan,
        &justified_atlas_bake,
        &mut justified_bake,
        &mut justified_database,
        &justified_assets,
        UVec2::new(180, 64),
    );

    let expected_first_glyph_x =
        (justified.frame.x + leading_space.advance + a.bitmap_left).max(justified.frame.x);
    let expected_second_glyph_x =
        justified.frame.right() - leading_space.advance - b.advance + b.bitmap_left;
    assert_eq!(vertices.len(), 12);
    assert!(
        (vertices[0].position[0] - pixel_to_ndc_x(expected_first_glyph_x, 180.0)).abs() < 0.0001
    );
    assert!(
        (vertices[6].position[0] - pixel_to_ndc_x(expected_second_glyph_x, 180.0)).abs() < 0.0001
    );
}

#[test]
fn sdf_draw_plan_expands_arabic_kashida_advances_for_justify() {
    let mut justified = text_batch("سلام", UiFrame::new(8.0, 12.0, 40.0, 20.0));
    justified.text_align = UiTextAlign::Justify;
    let natural_advances = vec![4.0, 4.0, 4.0, 4.0];

    let glyph_advances = resolve_sdf_glyph_advances(&justified, natural_advances.clone(), 16.0);

    assert!((glyph_advances.iter().sum::<f32>() - justified.frame.width).abs() < 0.1);
    assert!(
        glyph_advances[0] > natural_advances[0],
        "Arabic joining pair should receive kashida-like justify advance"
    );
    assert!(
        glyph_advances[1] > natural_advances[1],
        "lam-alef joining pair should receive kashida-like justify advance"
    );
    assert!((glyph_advances[2] - natural_advances[2]).abs() < 0.1);
    assert!((glyph_advances[3] - natural_advances[3]).abs() < 0.1);
}

#[test]
fn sdf_draw_plan_vertical_rl_advances_glyphs_on_y_axis() {
    let mut text = text_batch("AB", UiFrame::new(24.0, 8.0, 32.0, 96.0));
    text.writing_mode = UiTextWritingMode::VerticalRl;
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);
    let a = font_bake.measure_glyph(
        'A',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.font_weight,
        text.font_size,
        &mut font_database,
        &asset_manager,
    );

    let vertices = build_sdf_vertices(
        std::slice::from_ref(&text),
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 128),
    );

    let first_frame = vertical_sdf_glyph_frame(
        &text,
        &RunGlyph {
            slot_index: Some(0),
            metrics: a,
            atlas_bitmap_width: a.bitmap_width,
            atlas_bitmap_height: a.bitmap_height,
            visible: true,
            screen_px_range: sdf_screen_px_range(text.font_size, SdfBakeParams::default()),
        },
        text.frame.y,
        a.advance.max(0.0),
    );
    assert_eq!(vertices.len(), 12);
    assert!((vertices[0].position[0] - pixel_to_ndc_x(first_frame.x, 128.0)).abs() < 0.0001);
    assert!((vertices[0].position[1] - pixel_to_ndc_y(first_frame.y, 128.0)).abs() < 0.0001);
    let first_center_x = (vertices[0].position[0] + vertices[1].position[0]) * 0.5;
    let second_center_x = (vertices[6].position[0] + vertices[7].position[0]) * 0.5;
    assert!(
        (second_center_x - first_center_x).abs() < 0.0001,
        "vertical text should keep glyph centers in the same column"
    );
    assert!(
        vertices[6].position[1] < vertices[0].position[1],
        "NDC y decreases as the second vertical glyph advances downward"
    );
}
