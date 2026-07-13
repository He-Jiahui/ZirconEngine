use zircon_runtime_interface::ui::surface::{UiPaintPayload, UiRenderCommand, UiTextPaint};

pub(super) fn command_text_paint(command: &UiRenderCommand) -> Option<UiTextPaint> {
    command
        .to_paint_elements(0)
        .into_iter()
        .find_map(|element| match element.payload {
            UiPaintPayload::Text { text } => Some(text),
            _ => None,
        })
}
