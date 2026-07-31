use zircon_runtime_interface::ui::surface::{UiPaintElement, UiPaintPayload, UiTextPaint};

pub(super) fn command_text_paint(paint_elements: &[UiPaintElement]) -> Option<&UiTextPaint> {
    paint_elements
        .iter()
        .find_map(|element| match &element.payload {
            UiPaintPayload::Text { text } => Some(text),
            _ => None,
        })
}
