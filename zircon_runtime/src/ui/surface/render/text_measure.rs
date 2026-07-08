use super::resolve::{resolve_style, resolve_text};
use crate::ui::text::{measure_text_size, UiTextMeasureCache};
use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

pub(crate) fn measure_text_with_cache(
    metadata: Option<&UiTemplateNodeMetadata>,
    text_measure_cache: Option<&mut UiTextMeasureCache>,
) -> UiSize {
    let Some(text) = resolve_text(metadata) else {
        return UiSize::default();
    };
    if text.is_empty() {
        return UiSize::default();
    }

    let style = resolve_style(metadata);
    match text_measure_cache {
        Some(cache) => cache.measure_text_size(&text, &style),
        None => measure_text_size(&text, &style),
    }
}
