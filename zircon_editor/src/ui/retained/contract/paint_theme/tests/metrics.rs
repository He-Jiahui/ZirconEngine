use super::*;

#[test]
fn host_control_metrics_match_zircon_editor_baseline() {
    assert_eq!(METRICS.radius_small, 4.0);
    assert_eq!(METRICS.radius_control, 4.0);
    assert_eq!(METRICS.radius_panel, 0.0);
    assert_eq!(METRICS.border_width, 1.0);
    assert_eq!(METRICS.control_default_height, 32.0);
    assert_eq!(METRICS.control_large_height, 48.0);
    assert_eq!(METRICS.font_small, 12.0);
    assert_eq!(METRICS.font_body, 14.0);
    assert_eq!(METRICS.font_large, 20.0);
    assert_eq!(METRICS.button_pad_x, 12.0);
    assert_eq!(METRICS.text_clip_guard, 6.0);
    assert_eq!(METRICS.button_pressed_offset_y, 1.0);
    assert_eq!(METRICS.input_pad, [8.0, 8.0, 4.0, 4.0]);
    assert_eq!(METRICS.selection_indicator_width, 2.0);
    assert_eq!(METRICS.scrollbar_thickness, 8.0);
    assert_eq!(METRICS.scrollbar_min_thumb_length, 24.0);
    assert_eq!(
        METRICS.row_height,
        EditorDensityTokens::WORKBENCH_ROW_HEIGHT
    );
    assert!((METRICS.line_height(METRICS.font_body) - 19.6).abs() < 0.001);
}

#[test]
fn host_control_metrics_project_from_editor_design_tokens() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.controls.small_radius = 3.0;
    tokens.controls.control_radius = 7.0;
    tokens.controls.panel_radius = 11.0;
    tokens.controls.default_height = 31.0;
    tokens.controls.large_height = 45.0;
    tokens.controls.border_width = 1.5;
    tokens.typography.body_size = 11.0;
    tokens.density.gap_medium = 7.0;
    tokens.density.row_height = 26.0;

    let metrics = project_host_metrics(&tokens);

    assert_eq!(metrics.radius_small, 3.0);
    assert_eq!(metrics.radius_control, 7.0);
    assert_eq!(metrics.radius_panel, 11.0);
    assert_eq!(metrics.control_default_height, 31.0);
    assert_eq!(metrics.control_large_height, 45.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.font_body, 11.0);
    assert_eq!(metrics.gap_m, 7.0);
    assert_eq!(metrics.row_height, 26.0);
    assert_eq!(metrics.scrollbar_thickness, 7.0);
    assert_eq!(metrics.scrollbar_min_thumb_length, 26.0);
}

#[test]
fn host_control_metrics_scale_dimensions_but_keep_ratios_dimensionless() {
    let scaled = METRICS.at_scale(2.0);

    assert_eq!(scaled.control_default_height, 64.0);
    assert_eq!(scaled.radius_small, METRICS.radius_small * 2.0);
    assert_eq!(scaled.radius_control, METRICS.radius_control * 2.0);
    assert_eq!(scaled.radius_panel, METRICS.radius_panel * 2.0);
    assert_eq!(scaled.border_width, 2.0);
    assert_eq!(scaled.font_body, METRICS.font_body * 2.0);
    assert_eq!(scaled.input_pad, [16.0, 16.0, 6.0, 8.0]);
    assert_eq!(scaled.row_height, METRICS.row_height * 2.0);
    assert_eq!(scaled.line_height_ratio, METRICS.line_height_ratio);
}

#[test]
fn host_control_metrics_preserve_fractional_device_scale() {
    let at_125_percent = METRICS.at_scale(1.25);
    let at_150_percent = METRICS.at_scale(1.5);

    assert_eq!(at_125_percent.scale_factor, 1.25);
    assert_eq!(at_150_percent.scale_factor, 1.5);
    assert_eq!(at_125_percent.control_default_height, 40.0);
    assert_eq!(at_125_percent.radius_small, 7.5);
    assert_eq!(at_125_percent.radius_control, 10.0);
    assert_eq!(at_125_percent.radius_panel, 15.0);
    assert_eq!(at_125_percent.border_width, 1.25);
    assert_eq!(at_150_percent.control_default_height, 48.0);
    assert_eq!(at_150_percent.radius_small, 9.0);
    assert_eq!(at_150_percent.radius_control, 12.0);
    assert_eq!(at_150_percent.radius_panel, 18.0);
    assert_eq!(at_150_percent.border_width, 1.5);
}

#[test]
fn host_control_metrics_fail_closed_for_invalid_token_geometry() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.controls.default_height = f32::NAN;
    tokens.controls.large_height = -1.0;
    tokens.controls.dense_height = f32::INFINITY;
    tokens.controls.small_radius = f32::NEG_INFINITY;
    tokens.controls.control_radius = f32::NEG_INFINITY;
    tokens.controls.panel_radius = f32::NEG_INFINITY;
    tokens.controls.border_width = f32::NAN;
    tokens.typography.caption_size = 0.0;
    tokens.typography.body_size = f32::NEG_INFINITY;
    tokens.typography.title_size = f32::INFINITY;
    tokens.typography.line_height = f32::NAN;
    tokens.density.gap_small = -1.0;
    tokens.density.gap_medium = f32::NAN;
    tokens.density.gap_large = f32::INFINITY;
    tokens.density.row_height = 0.0;

    let metrics = project_host_metrics(&tokens);

    assert_eq!(metrics, METRICS);
}

#[test]
fn host_control_metrics_preserve_zero_border_and_spacing_tokens() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.controls.border_width = 0.0;
    tokens.controls.small_radius = 5.0;
    tokens.controls.control_radius = 0.0;
    tokens.controls.panel_radius = 0.0;
    tokens.density.gap_small = 0.0;
    tokens.density.gap_medium = 0.0;
    tokens.density.gap_large = 0.0;

    let metrics = project_host_metrics(&tokens);

    assert_eq!(metrics.border_width, 0.0);
    assert_eq!(metrics.radius_small, 5.0);
    assert_eq!(metrics.radius_control, 0.0);
    assert_eq!(metrics.radius_panel, 0.0);
    assert_eq!(metrics.gap_s, 0.0);
    assert_eq!(metrics.gap_m, 0.0);
    assert_eq!(metrics.gap_l, 0.0);
    assert_eq!(metrics.button_icon_gap, 0.0);
}
