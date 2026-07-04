use super::super::super::super::data::TemplatePaneNodeData;
use super::super::metrics::dialog_metrics;
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;

pub(super) fn action_label(node: &TemplatePaneNodeData, index: usize) -> Option<String> {
    node.actions
        .row_data(index)
        .and_then(|action| non_empty(action.label.as_str()).map(str::to_string))
}

pub(super) fn action_width(text: &str) -> f32 {
    let metrics = dialog_metrics();
    (measure_runtime_text_width(text, metrics.action_font_size)
        + metrics.action_text_padding_x * 2.0
        + metrics.action_text_clip_guard)
        .max(metrics.action_min_width)
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_width_uses_runtime_text_measurement() {
        let text = "WWWiiiWWWiii";
        let metrics = dialog_metrics();
        let expected = (measure_runtime_text_width(text, metrics.action_font_size)
            + metrics.action_text_padding_x * 2.0
            + metrics.action_text_clip_guard)
            .max(metrics.action_min_width);
        let legacy_half_em_width = text.chars().count() as f32 * (metrics.action_font_size * 0.5)
            + metrics.action_text_padding_x * 2.0;

        assert!((action_width(text) - expected).abs() <= 0.01);
        assert_ne!(action_width(text).round(), legacy_half_em_width.round());
    }

    #[test]
    fn action_width_keeps_minimum_for_short_labels() {
        assert_eq!(action_width("OK"), dialog_metrics().action_min_width);
    }
}
