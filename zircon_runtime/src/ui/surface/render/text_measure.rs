use super::resolve::{resolve_style, resolve_text};
use crate::ui::text::UiTextMeasureCache;
use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

pub(crate) fn measure_text_with_cache(
    metadata: Option<&UiTemplateNodeMetadata>,
    text_measure_cache: &mut UiTextMeasureCache,
) -> UiSize {
    let Some(text) = resolve_text(metadata) else {
        return UiSize::default();
    };
    if text.is_empty() {
        return UiSize::default();
    }

    let style = resolve_style(metadata);
    text_measure_cache.measure_text_size(&text, &style)
}

/// Exact height-only measure for a leaf whose width is already fixed by its constraint.
/// The caller owns the fixed-width proof; this path only accepts simple horizontal text and
/// otherwise falls back to complete text measurement.
pub(crate) fn measure_text_with_fixed_width_cache(
    metadata: Option<&UiTemplateNodeMetadata>,
    text_measure_cache: &mut UiTextMeasureCache,
    fixed_width: f32,
) -> UiSize {
    let Some(text) = resolve_text(metadata) else {
        return UiSize::default();
    };
    if text.is_empty() || !fixed_width.is_finite() || fixed_width <= 0.0 {
        return measure_text_with_cache(metadata, text_measure_cache);
    }

    let style = resolve_style(metadata);
    if let Some(height) = text_measure_cache.measure_unwrapped_text_height(&text, &style) {
        UiSize::new(fixed_width, height)
    } else {
        text_measure_cache.measure_text_size(&text, &style)
    }
}
