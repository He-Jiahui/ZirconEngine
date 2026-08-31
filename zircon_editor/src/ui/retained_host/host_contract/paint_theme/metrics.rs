use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorDesignTokens, EditorTypographyTokens,
};

// Shared retained-host control metrics keep Slate-like primitive controls on one
// spacing, radius, and text scale before higher-level composites consume them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HostControlMetrics {
    pub control_default_height: f32,
    pub control_large_height: f32,
    pub radius_small: f32,
    pub radius_control: f32,
    pub radius_panel: f32,
    pub border_width: f32,
    pub font_small: f32,
    pub font_body: f32,
    pub font_large: f32,
    pub line_height_ratio: f32,
    pub button_pad_x: f32,
    pub button_icon_gap: f32,
    pub button_chevron_reserve: f32,
    pub text_clip_guard: f32,
    pub button_pressed_offset_y: f32,
    pub input_pad: [f32; 4],
    pub segment_text_inset_y: f32,
    pub segment_selected_inset: f32,
    pub tab_underline_height: f32,
    pub selection_indicator_width: f32,
    pub scrollbar_thickness: f32,
    pub scrollbar_min_thumb_length: f32,
    pub gap_s: f32,
    pub gap_m: f32,
    pub gap_l: f32,
    pub row_height: f32,
}

impl HostControlMetrics {
    pub(in crate::ui::retained_host) fn line_height(&self, font_size: f32) -> f32 {
        font_size * self.line_height_ratio
    }

    pub(super) fn at_scale(self, scale_factor: f32) -> Self {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        let scaled = |value: f32| {
            let value = value * scale_factor;
            if value.is_finite() {
                value
            } else {
                0.0
            }
        };
        Self {
            control_default_height: scaled(self.control_default_height),
            control_large_height: scaled(self.control_large_height),
            radius_small: scaled(self.radius_small),
            radius_control: scaled(self.radius_control),
            radius_panel: scaled(self.radius_panel),
            border_width: scaled(self.border_width),
            font_small: scaled(self.font_small),
            font_body: scaled(self.font_body),
            font_large: scaled(self.font_large),
            line_height_ratio: self.line_height_ratio,
            button_pad_x: scaled(self.button_pad_x),
            button_icon_gap: scaled(self.button_icon_gap),
            button_chevron_reserve: scaled(self.button_chevron_reserve),
            text_clip_guard: scaled(self.text_clip_guard),
            button_pressed_offset_y: scaled(self.button_pressed_offset_y),
            input_pad: self.input_pad.map(scaled),
            segment_text_inset_y: scaled(self.segment_text_inset_y),
            segment_selected_inset: scaled(self.segment_selected_inset),
            tab_underline_height: scaled(self.tab_underline_height),
            selection_indicator_width: scaled(self.selection_indicator_width),
            scrollbar_thickness: scaled(self.scrollbar_thickness),
            scrollbar_min_thumb_length: scaled(self.scrollbar_min_thumb_length),
            gap_s: scaled(self.gap_s),
            gap_m: scaled(self.gap_m),
            gap_l: scaled(self.gap_l),
            row_height: scaled(self.row_height),
        }
    }
}

pub(crate) const METRICS: HostControlMetrics = HostControlMetrics {
    control_default_height: 32.0,
    control_large_height: 48.0,
    radius_small: 6.0,
    radius_control: 8.0,
    radius_panel: 12.0,
    border_width: 1.0,
    font_small: EditorTypographyTokens::WORKBENCH_CAPTION_SIZE,
    font_body: EditorTypographyTokens::WORKBENCH_BODY_SIZE,
    font_large: EditorTypographyTokens::WORKBENCH_TITLE_SIZE,
    line_height_ratio: 1.2,
    button_pad_x: 12.0,
    button_icon_gap: 7.0,
    button_chevron_reserve: 18.0,
    text_clip_guard: 6.0,
    button_pressed_offset_y: 1.0,
    input_pad: [8.0, 8.0, 3.0, 4.0],
    segment_text_inset_y: 4.0,
    segment_selected_inset: 2.0,
    tab_underline_height: 2.0,
    selection_indicator_width: 2.0,
    scrollbar_thickness: 8.0,
    scrollbar_min_thumb_length: 24.0,
    gap_s: 4.0,
    gap_m: 8.0,
    gap_l: 12.0,
    row_height: EditorDensityTokens::WORKBENCH_ROW_HEIGHT,
};

pub(crate) fn apply_host_metrics_from_tokens(tokens: &EditorDesignTokens) {
    super::replace_host_metrics(project_host_metrics(tokens));
}

pub(crate) fn current_host_metrics() -> HostControlMetrics {
    super::host_metrics_for_read()
}

pub(in crate::ui::retained_host::host_contract) fn project_host_metrics(
    tokens: &EditorDesignTokens,
) -> HostControlMetrics {
    let controls = &tokens.controls;
    let density = &tokens.density;
    let typography = &tokens.typography;
    let control_default_height =
        finite_positive_or(controls.default_height, METRICS.control_default_height);
    let control_large_height =
        finite_positive_or(controls.large_height, METRICS.control_large_height);
    let radius_small = finite_non_negative_or(controls.small_radius, METRICS.radius_small);
    let radius_control = finite_non_negative_or(controls.control_radius, METRICS.radius_control);
    let radius_panel = finite_non_negative_or(controls.panel_radius, METRICS.radius_panel);
    let border_width = finite_non_negative_or(controls.border_width, METRICS.border_width);
    let font_small = finite_positive_or(typography.caption_size, METRICS.font_small);
    let font_body = finite_positive_or(typography.body_size, METRICS.font_body);
    let font_large = finite_positive_or(typography.title_size, METRICS.font_large);
    let line_height_ratio = finite_positive_or(typography.line_height, METRICS.line_height_ratio);
    let gap_small = finite_non_negative_or(density.gap_small, METRICS.gap_s);
    let gap_medium = finite_non_negative_or(density.gap_medium, METRICS.gap_m);
    let gap_large = finite_non_negative_or(density.gap_large, METRICS.gap_l);
    let row_height = finite_positive_or(density.row_height, METRICS.row_height);
    let dense_height = finite_positive_or(
        controls.dense_height,
        EditorControlTokens::workbench_dense().dense_height,
    );
    HostControlMetrics {
        control_default_height,
        control_large_height,
        radius_small,
        radius_control,
        radius_panel,
        border_width,
        font_small,
        font_body,
        font_large,
        line_height_ratio,
        button_pad_x: gap_large,
        button_icon_gap: (gap_medium - border_width).max(0.0),
        button_chevron_reserve: (dense_height - gap_large + border_width * 2.0).max(0.0),
        text_clip_guard: (gap_medium - border_width * 2.0).max(0.0),
        button_pressed_offset_y: border_width,
        input_pad: [
            gap_medium,
            gap_medium,
            (gap_small - border_width).max(0.0),
            gap_small,
        ],
        segment_text_inset_y: gap_small,
        segment_selected_inset: border_width * 2.0,
        tab_underline_height: border_width * 2.0,
        selection_indicator_width: border_width * 2.0,
        scrollbar_thickness: gap_medium.max(border_width * 4.0),
        scrollbar_min_thumb_length: row_height.max(gap_medium * 2.0),
        gap_s: gap_small,
        gap_m: gap_medium,
        gap_l: gap_large,
        row_height,
    }
}

fn finite_non_negative_or(value: f32, fallback: f32) -> f32 {
    (value.is_finite() && value >= 0.0)
        .then_some(value)
        .unwrap_or(fallback)
}

fn finite_positive_or(value: f32, fallback: f32) -> f32 {
    (value.is_finite() && value > 0.0)
        .then_some(value)
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_control_metrics_match_zircon_editor_baseline() {
        let slate_points_to_logical_pixels = 96.0 / 72.0;
        assert_eq!(METRICS.radius_small, 6.0);
        assert_eq!(METRICS.radius_control, 8.0);
        assert_eq!(METRICS.radius_panel, 12.0);
        assert_eq!(METRICS.border_width, 1.0);
        assert_eq!(METRICS.control_default_height, 32.0);
        assert_eq!(METRICS.control_large_height, 48.0);
        assert!((METRICS.font_small - 8.0 * slate_points_to_logical_pixels).abs() < 0.001);
        assert!((METRICS.font_body - 10.0 * slate_points_to_logical_pixels).abs() < 0.001);
        assert!((METRICS.font_large - 14.0 * slate_points_to_logical_pixels).abs() < 0.001);
        assert_eq!(METRICS.button_pad_x, 12.0);
        assert_eq!(METRICS.text_clip_guard, 6.0);
        assert_eq!(METRICS.button_pressed_offset_y, 1.0);
        assert_eq!(METRICS.input_pad, [8.0, 8.0, 3.0, 4.0]);
        assert_eq!(METRICS.selection_indicator_width, 2.0);
        assert_eq!(METRICS.scrollbar_thickness, 8.0);
        assert_eq!(METRICS.scrollbar_min_thumb_length, 24.0);
        assert_eq!(
            METRICS.row_height,
            EditorDensityTokens::WORKBENCH_ROW_HEIGHT
        );
        assert!((METRICS.line_height(METRICS.font_body) - 16.0).abs() < 0.001);
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
}
