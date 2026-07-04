use std::sync::{OnceLock, RwLock};

use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

// Shared retained-host control metrics keep Slate-like primitive controls on one
// spacing, radius, and text scale before higher-level composites consume them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostControlMetrics {
    pub radius_control: f32,
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
    pub(in crate::ui::retained_host::host_contract) fn line_height(&self, font_size: f32) -> f32 {
        font_size * self.line_height_ratio
    }
}

pub(in crate::ui::retained_host::host_contract) const METRICS: HostControlMetrics =
    HostControlMetrics {
        radius_control: 4.0,
        border_width: 1.0,
        font_small: 8.0,
        font_body: 10.0,
        font_large: 14.0,
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
        row_height: 24.0,
    };

pub(crate) fn apply_host_metrics_from_tokens(tokens: &EditorDesignTokens) {
    let next_metrics = project_host_metrics(tokens);
    match host_metrics().write() {
        Ok(mut metrics) => *metrics = next_metrics,
        Err(poisoned) => *poisoned.into_inner() = next_metrics,
    }
}

pub(in crate::ui::retained_host::host_contract) fn current_host_metrics() -> HostControlMetrics {
    match host_metrics().read() {
        Ok(metrics) => *metrics,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

pub(in crate::ui::retained_host::host_contract) fn project_host_metrics(
    tokens: &EditorDesignTokens,
) -> HostControlMetrics {
    let controls = &tokens.controls;
    let density = &tokens.density;
    let typography = &tokens.typography;
    HostControlMetrics {
        radius_control: controls.small_radius,
        border_width: controls.border_width,
        font_small: typography.caption_size,
        font_body: typography.body_size,
        font_large: typography.title_size,
        line_height_ratio: typography.line_height,
        button_pad_x: density.gap_large,
        button_icon_gap: (density.gap_medium - controls.border_width).max(0.0),
        button_chevron_reserve: (controls.dense_height - density.gap_large
            + controls.border_width * 2.0)
            .max(0.0),
        text_clip_guard: (density.gap_medium - controls.border_width * 2.0).max(0.0),
        button_pressed_offset_y: controls.border_width,
        input_pad: [
            density.gap_medium,
            density.gap_medium,
            (density.gap_small - controls.border_width).max(0.0),
            density.gap_small,
        ],
        segment_text_inset_y: density.gap_small,
        segment_selected_inset: controls.border_width * 2.0,
        tab_underline_height: controls.border_width * 2.0,
        selection_indicator_width: controls.border_width * 2.0,
        scrollbar_thickness: density.gap_medium.max(controls.border_width * 4.0),
        scrollbar_min_thumb_length: density.row_height.max(density.gap_medium * 2.0),
        gap_s: density.gap_small,
        gap_m: density.gap_medium,
        gap_l: density.gap_large,
        row_height: density.row_height,
    }
}

fn host_metrics() -> &'static RwLock<HostControlMetrics> {
    static METRICS_STATE: OnceLock<RwLock<HostControlMetrics>> = OnceLock::new();
    METRICS_STATE.get_or_init(|| RwLock::new(METRICS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_control_metrics_match_unreal_slate_baseline() {
        assert_eq!(METRICS.radius_control, 4.0);
        assert_eq!(METRICS.border_width, 1.0);
        assert_eq!(METRICS.font_small, 8.0);
        assert_eq!(METRICS.font_body, 10.0);
        assert_eq!(METRICS.font_large, 14.0);
        assert_eq!(METRICS.button_pad_x, 12.0);
        assert_eq!(METRICS.text_clip_guard, 6.0);
        assert_eq!(METRICS.button_pressed_offset_y, 1.0);
        assert_eq!(METRICS.input_pad, [8.0, 8.0, 3.0, 4.0]);
        assert_eq!(METRICS.selection_indicator_width, 2.0);
        assert_eq!(METRICS.scrollbar_thickness, 8.0);
        assert_eq!(METRICS.scrollbar_min_thumb_length, 24.0);
        assert_eq!(METRICS.line_height(METRICS.font_body), 12.0);
    }

    #[test]
    fn host_control_metrics_project_from_editor_design_tokens() {
        let mut tokens = EditorDesignTokens::workbench_dark();
        tokens.controls.small_radius = 3.0;
        tokens.controls.border_width = 1.5;
        tokens.typography.body_size = 11.0;
        tokens.density.gap_medium = 7.0;
        tokens.density.row_height = 26.0;

        let metrics = project_host_metrics(&tokens);

        assert_eq!(metrics.radius_control, 3.0);
        assert_eq!(metrics.border_width, 1.5);
        assert_eq!(metrics.font_body, 11.0);
        assert_eq!(metrics.gap_m, 7.0);
        assert_eq!(metrics.row_height, 26.0);
        assert_eq!(metrics.scrollbar_thickness, 7.0);
        assert_eq!(metrics.scrollbar_min_thumb_length, 26.0);
    }
}
