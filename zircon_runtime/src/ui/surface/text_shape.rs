use std::sync::Arc;

use crate::core::framework::text::TextLayoutError;
use crate::text::{text_style, ShapedGlyphRun, SharedTextLayoutSession, TextRange};
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRange};

pub fn shape_text_line(
    text: &str,
    style: &UiResolvedStyle,
) -> Result<ShapedGlyphRun, TextLayoutError> {
    crate::profile_scope!("runtime", "text.surface", "shape_text_line");
    let mut session = SharedTextLayoutSession::new();
    match session.shape_horizontal_range(
        text,
        &text_style(style),
        style.text_direction.into(),
        TextRange {
            start: 0,
            end: text.len(),
        },
    ) {
        crate::text::shaping::TextShapingOutcome::Ready(run) => Ok(Arc::unwrap_or_clone(run)),
        crate::text::shaping::TextShapingOutcome::Deferred(error)
        | crate::text::shaping::TextShapingOutcome::Failed(error) => {
            session.record_layout_error(&error);
            Err(error.into_error())
        }
    }
}

#[cfg(test)]
#[path = "tests/text_shape.rs"]
mod tests;
