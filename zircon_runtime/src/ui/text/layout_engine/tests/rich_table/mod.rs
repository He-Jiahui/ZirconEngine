use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedTextLayout, UiRichTextFormat, UiTextOverflow, UiTextWrap, UiTextWritingMode,
    },
};

use super::{layout_text, test_style};

mod horizontal;
mod vertical_rl;

fn vertical_table(markup: &str, frame: UiFrame) -> UiResolvedTextLayout {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCodeV1;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    layout_text(markup, &style, frame, None)
}

fn find_line<'a>(
    layout: &'a UiResolvedTextLayout,
    text: &str,
) -> &'a zircon_runtime_interface::ui::surface::UiResolvedTextLine {
    layout
        .lines
        .iter()
        .find(|line| line.text.contains(text))
        .unwrap_or_else(|| panic!("expected table line containing {text:?}"))
}
