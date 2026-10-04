use zircon_runtime_interface::ui::{
    layout::UiFrame, surface::UiRichTextFormat, tree::UiTemplateNodeMetadata,
};

use super::resolve::{resolve_style, resolve_text};
use crate::ui::text::{
    inline_widget_layout_from_compiled, UiInlineWidgetLayout, UiTextLayoutRequest,
    UiTextMeasureCache,
};

pub(crate) fn metadata_has_inline_widget(
    metadata: Option<&UiTemplateNodeMetadata>,
    text_measure_cache: &UiTextMeasureCache,
) -> bool {
    parsed_inline_widget_owner(metadata, text_measure_cache).is_some()
}

pub(crate) fn resolve_inline_widget_layout_with_cache(
    metadata: Option<&UiTemplateNodeMetadata>,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    text_measure_cache: &mut UiTextMeasureCache,
) -> Option<UiInlineWidgetLayout> {
    let (text, style, parsed) = parsed_inline_widget_owner(metadata, text_measure_cache)?;
    let request = UiTextLayoutRequest::new(&text, &style, frame, clip_frame);
    let resolution = text_measure_cache.resolve_or_shape(&request);
    inline_widget_layout_from_compiled(parsed.rich.as_ref(), Some(&resolution.layout))
}

fn parsed_inline_widget_owner(
    metadata: Option<&UiTemplateNodeMetadata>,
    text_measure_cache: &UiTextMeasureCache,
) -> Option<(
    String,
    zircon_runtime_interface::ui::surface::UiResolvedStyle,
    crate::ui::text::UiParsedText,
)> {
    let text = resolve_text(metadata)?;
    let style = resolve_style(metadata);
    if matches!(style.rich_text_format, UiRichTextFormat::Plain) {
        return None;
    }
    let parsed = text_measure_cache
        .parse_source_text(&text, style.rich_text_format.into())
        .ok()?;
    inline_widget_layout_from_compiled(parsed.rich.as_ref(), None)?;
    Some((text, style, parsed))
}
