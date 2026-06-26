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
        gap_s: 4.0,
        gap_m: 8.0,
        gap_l: 12.0,
        row_height: 24.0,
    };

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
        assert_eq!(METRICS.line_height(METRICS.font_body), 12.0);
    }
}
