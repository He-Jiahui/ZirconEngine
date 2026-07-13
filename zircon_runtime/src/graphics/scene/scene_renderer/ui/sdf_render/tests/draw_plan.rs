use super::*;

#[test]
fn sdf_draw_plan_creates_one_textured_quad_per_glyph() {
    let text = text_batch("AB", UiFrame::new(8.0, 12.0, 64.0, 20.0));
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

    assert_eq!(vertices.len(), 12);
    assert_eq!(vertices[0].color, [0.2, 0.3, 0.4, 0.5]);
    assert!(
        (vertices[0].screen_px_range
            - sdf_screen_px_range(text.font_size, SdfBakeParams::default()))
        .abs()
            < 0.0001
    );
    assert!(vertices[0].uv[0] >= 0.0);
    assert!(vertices[0].uv[1] >= 0.0);
    assert!(vertices[0].uv[0] < vertices[2].uv[0]);
    assert!(vertices[6].uv[0] > vertices[0].uv[0]);
}
#[test]
fn sdf_draw_plan_snaps_text_origin_but_preserves_glyph_subpixel_phase() {
    let text = text_batch("A", UiFrame::new(8.375, 12.51, 64.0, 20.0));
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);
    // Measurements used for placement must be the same baked metrics that the
    // draw plan consumes, including distance-field padding.
    let a = font_bake.measure_glyph(
        'A',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.language.as_deref(),
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
        UVec2::new(128, 64),
    );

    let positioned_frame = text_frame_device_origin(text.frame);
    let requested_x = positioned_frame.x + a.bitmap_left;
    let placement = GlyphRasterPlacement::from_raster_input(
        GlyphAtlasFormat::Sdf,
        GlyphSmoothingMode::None,
        false,
        requested_x,
    );
    assert_eq!(placement.requested_x, requested_x);
    assert_eq!(placement.subpixel_bin, 0);
    assert_eq!(vertices.len(), 6);
    let baked_metrics = scale_sdf_metrics_for_display(
        atlas_bake.glyphs[0].metrics,
        text.font_size,
        plan.slots[0].key.bake_params,
    );
    assert_eq!(a, baked_metrics);
    let baseline = positioned_frame.y
        + (text.line_height.max(text.font_size) - text.font_size.max(1.0)).max(0.0) * 0.5
        + baked_metrics.ascent.max(text.font_size.max(1.0));
    let first_frame = horizontal_sdf_glyph_frame(
        positioned_frame.x,
        baseline,
        &RunGlyph {
            slot_index: Some(0),
            metrics: baked_metrics,
            atlas_bitmap_width: atlas_bake.glyphs[0].metrics.bitmap_width,
            atlas_bitmap_height: atlas_bake.glyphs[0].metrics.bitmap_height,
            visible: true,
            screen_px_range: sdf_screen_px_range(text.font_size, SdfBakeParams::default()),
            atlas_px_range: SdfBakeParams::default().spread_px_f32(),
        },
    );
    assert!((first_frame.x - placement.snapped_x).abs() < 0.0001);
    let viewport = UiFrame::new(0.0, 0.0, 128.0, 64.0);
    let clip = text
        .frame
        .intersection(viewport)
        .expect("text frame should overlap the viewport");
    let clipped_first_frame = first_frame
        .intersection(clip)
        .and_then(|frame| frame.intersection(viewport))
        .expect("baked glyph frame should remain visible after production clipping");
    assert!(
        (vertices[0].position[0] - pixel_to_ndc_x(clipped_first_frame.x, 128.0)).abs() < 0.0001
    );
    assert!((vertices[0].position[1] - pixel_to_ndc_y(clipped_first_frame.y, 64.0)).abs() < 0.0001);
}

#[test]
fn sdf_glyph_frames_preserve_fractional_bitmap_origins_without_changing_size() {
    let glyph = RunGlyph {
        slot_index: Some(0),
        metrics: SdfGlyphMetrics {
            bitmap_width: 11,
            bitmap_height: 17,
            bitmap_left: 0.58,
            bitmap_bottom: 2.25,
            advance: 12.5,
            ascent: 14.25,
        },
        atlas_bitmap_width: 11,
        atlas_bitmap_height: 17,
        visible: true,
        screen_px_range: 4.0,
        atlas_px_range: 8.0,
    };

    let frame = horizontal_sdf_glyph_frame(20.0, 30.0, &glyph);

    assert_eq!(frame.x, 20.58);
    assert_eq!(frame.y, 10.75);
    assert_eq!(frame.width, 11.0);
    assert_eq!(frame.height, 17.0);
}

#[test]
fn sdf_draw_plan_carries_atlas_page_index_for_array_texture_sampling() {
    let text = text_batch("A", UiFrame::new(8.0, 12.0, 64.0, 20.0));
    let plan = synthetic_layered_plan(1);
    let atlas_bake = synthetic_layered_bake(&plan);
    let mut font_bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();

    let vertices = build_sdf_vertices(
        std::slice::from_ref(&text),
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 64),
    );

    assert_eq!(vertices.len(), 6);
    assert!(vertices.iter().all(|vertex| vertex.page_index == 1));
}

#[test]

fn sdf_draw_plan_skips_whitespace_quads_but_preserves_advance() {
    let text = text_batch("A B", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);
    let a = font_bake.measure_glyph(
        'A',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.language.as_deref(),
        text.font_weight,
        text.font_size,
        &mut font_database,
        &asset_manager,
    );
    let space = font_bake.measure_glyph(
        ' ',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.language.as_deref(),
        text.font_weight,
        text.font_size,
        &mut font_database,
        &asset_manager,
    );
    let b = font_bake.measure_glyph(
        'B',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.language.as_deref(),
        text.font_weight,
        text.font_size,
        &mut font_database,
        &asset_manager,
    );

    let vertices = build_sdf_vertices(
        &[text],
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 64),
    );

    let expected_second_glyph_x = 8.0 + a.advance + space.advance + b.bitmap_left;
    assert_eq!(vertices.len(), 12);
    assert!(
        (vertices[6].position[0] - pixel_to_ndc_x(expected_second_glyph_x, 128.0)).abs() < 0.0001
    );
}
#[test]
fn sdf_draw_plan_zeroes_format_control_advances_without_slots() {
    let text = text_batch("A\u{200D}\u{FE0F}B", UiFrame::new(8.0, 12.0, 80.0, 20.0));
    let plan = plan_sdf_atlas(std::slice::from_ref(&text));
    let (mut font_bake, mut font_database, asset_manager, atlas_bake) = bake_atlas(&plan);
    let a = font_bake.measure_glyph(
        'A',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.language.as_deref(),
        text.font_weight,
        text.font_size,
        &mut font_database,
        &asset_manager,
    );
    let b = font_bake.measure_glyph(
        'B',
        text.font.as_deref(),
        text.font_family.as_deref(),
        text.language.as_deref(),
        text.font_weight,
        text.font_size,
        &mut font_database,
        &asset_manager,
    );

    let vertices = build_sdf_vertices(
        &[text],
        &plan,
        &atlas_bake,
        &mut font_bake,
        &mut font_database,
        &asset_manager,
        UVec2::new(128, 64),
    );

    let requested_second_glyph_x = 8.0 + a.advance + b.bitmap_left;
    let expected_second_glyph_x = GlyphRasterPlacement::from_raster_input(
        GlyphAtlasFormat::Sdf,
        GlyphSmoothingMode::None,
        false,
        requested_second_glyph_x,
    )
    .snapped_x;
    assert_eq!(
        plan.runs[0].glyph_slot_indices,
        vec![Some(0), None, None, Some(1)]
    );
    assert_eq!(vertices.len(), 12);
    assert!(
        (vertices[6].position[0] - pixel_to_ndc_x(expected_second_glyph_x, 128.0)).abs() < 0.0001
    );
}
