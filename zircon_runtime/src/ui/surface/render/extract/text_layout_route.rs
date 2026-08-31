use crate::ui::text::{UiTextLayoutRequest, UiTextLayoutResolution, UiTextMeasureCache};

pub(crate) fn resolve_text_layout_with_cache(
    request: &UiTextLayoutRequest<'_>,
    text_measure_cache: &mut UiTextMeasureCache,
) -> UiTextLayoutResolution {
    text_measure_cache.resolve_or_shape(request)
}
