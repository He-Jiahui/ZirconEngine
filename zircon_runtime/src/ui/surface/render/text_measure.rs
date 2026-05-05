use super::resolve::{resolve_style, resolve_text};
use crate::ui::text::measure_text_size;
use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

pub(crate) fn measure_text(metadata: Option<&UiTemplateNodeMetadata>) -> UiSize {
    let Some(text) = resolve_text(metadata) else {
        return UiSize::default();
    };
    if text.is_empty() {
        return UiSize::default();
    }

    measure_text_size(&text, &resolve_style(metadata))
}
