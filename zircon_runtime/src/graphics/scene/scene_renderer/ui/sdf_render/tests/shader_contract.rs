use super::*;

#[test]
fn text_sdf_screen_px_range_scales_with_font_size() {
    let mut small = text_batch("A", UiFrame::new(8.0, 12.0, 64.0, 20.0));
    small.font_size = 16.0;
    let mut medium = small.clone();
    medium.font_size = 32.0;
    let mut large = small.clone();
    large.font_size = 64.0;

    let small_range = first_sdf_screen_px_range(small);
    let medium_range = first_sdf_screen_px_range(medium);
    let large_range = first_sdf_screen_px_range(large);

    assert!((small_range - 8.0 / 3.0).abs() < 0.0001);
    assert!((medium_range - 16.0 / 3.0).abs() < 0.0001);
    assert!((large_range - 32.0 / 3.0).abs() < 0.0001);
    assert!((medium_range - small_range * 2.0).abs() < 0.0001);
    assert!((large_range - small_range * 4.0).abs() < 0.0001);
}

#[test]
fn sdf_draw_plan_reuses_fixed_bake_slot_while_scaling_display_size() {
    let mut small = text_batch("A", UiFrame::new(8.0, 12.0, 96.0, 32.0));
    small.font_size = 16.0;
    small.line_height = 20.0;
    let mut large = text_batch("A", UiFrame::new(8.0, 56.0, 128.0, 64.0));
    large.font_size = 32.0;
    large.line_height = 40.0;
    let plan = plan_sdf_atlas(&[small.clone(), large.clone()]);

    assert_eq!(plan.slots.len(), 1);
    assert_eq!(plan.runs[0].glyph_slot_indices, vec![Some(0)]);
    assert_eq!(plan.runs[1].glyph_slot_indices, vec![Some(0)]);
    assert_eq!(plan.slots[0].key.bake_params, SdfBakeParams::default());

    let (_, _, asset_manager, atlas_bake) = bake_atlas(&plan);
    let vertices = build_sdf_vertices(
        &[small, large],
        &plan,
        &atlas_bake,
        &asset_manager,
        UVec2::new(256, 160),
    );

    assert_eq!(vertices.len(), 12);
    let small_width = (vertices[1].position[0] - vertices[0].position[0]).abs();
    let large_width = (vertices[7].position[0] - vertices[6].position[0]).abs();
    assert!(large_width > small_width * 1.75);
    assert!((vertices[0].uv[0] - vertices[6].uv[0]).abs() < 0.0001);
    assert!((vertices[2].uv[0] - vertices[8].uv[0]).abs() < 0.0001);
    assert!((vertices[0].screen_px_range * 2.0 - vertices[6].screen_px_range).abs() < 0.0001);
}
#[test]
fn sdf_shader_uses_screen_px_range_instead_of_fixed_smoothstep_thresholds() {
    assert!(SDF_TEXT_SHADER.contains("screen_px_range"));
    assert!(!SDF_TEXT_SHADER.contains("0.42"));
    assert!(!SDF_TEXT_SHADER.contains("0.58"));
}

#[test]
fn sdf_shader_samples_page_indexed_atlas_array() {
    assert!(SDF_TEXT_SHADER.contains("texture_2d_array"));
    assert!(SDF_TEXT_SHADER.contains("page_index"));
    assert!(SDF_TEXT_SHADER.contains("sdf_atlas"));
    assert!(SDF_TEXT_SHADER.contains("msdf_atlas"));
    assert!(SDF_TEXT_SHADER.contains("distance_field_sampler"));
}

#[test]
fn sdf_shader_decodes_explicit_sdf_msdf_and_mtsdf_modes() {
    assert!(SDF_TEXT_SHADER.contains("decode_mode"));
    assert!(SDF_TEXT_SHADER.contains("fn median3"));
    assert!(SDF_TEXT_SHADER.contains("median3(sample.rgb)"));
    assert!(SDF_TEXT_SHADER.contains("input.decode_mode == MTSDF_MODE"));
    assert!(SDF_TEXT_SHADER.contains("sample.a"));
    assert!(!SDF_TEXT_SHADER.contains("let distance = textureSample(sdf_atlas"));
    assert!(
        !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl")
            .exists()
    );
}

#[test]
fn sdf_vertices_propagate_mode_for_horizontal_and_vertical_glyphs() {
    for writing_mode in [
        UiTextWritingMode::HorizontalTb,
        UiTextWritingMode::VerticalRl,
    ] {
        let mut text = text_batch("AMW", UiFrame::new(8.0, 12.0, 120.0, 120.0));
        text.writing_mode = writing_mode;
        let mut plan = plan_sdf_atlas(&[text.clone()]);
        for (slot, mode) in plan
            .slots
            .iter_mut()
            .zip([SdfMode::Sdf, SdfMode::Msdf, SdfMode::Mtsdf])
        {
            slot.key.bake_params.mode = mode;
            slot.page_key = GlyphAtlasPageKey::new(mode.atlas_format(), 0);
        }
        let (_, _, asset_manager, atlas_bake) = bake_atlas(&plan);
        let vertices = build_sdf_vertices(
            &[text],
            &plan,
            &atlas_bake,
            &asset_manager,
            UVec2::new(256, 160),
        );

        assert_eq!(vertices.len(), 18);
        assert_eq!(vertices[0].decode_mode, SdfMode::Sdf.shader_discriminant());
        assert_eq!(vertices[6].decode_mode, SdfMode::Msdf.shader_discriminant());
        assert_eq!(
            vertices[12].decode_mode,
            SdfMode::Mtsdf.shader_discriminant()
        );
    }
}

#[test]
fn sdf_shader_screen_px_range_wgsl_parses() {
    naga::front::wgsl::parse_str(SDF_TEXT_SHADER)
        .expect("screen_px_range SDF text shader should parse");
}
