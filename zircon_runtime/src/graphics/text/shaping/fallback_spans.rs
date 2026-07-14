use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use crate::core::framework::render::{
    FontFaceId, FontFamilyName, FontQuery, FontStretch, FontStyle, FontWeight, InstancedFaceId,
    TextShapeRequest,
};
use crate::graphics::text::font::FontDatabase;

use super::script_segment::font_script_for_cluster;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FallbackTextSpan {
    pub(crate) range: Range<usize>,
    pub(crate) family: Option<String>,
    pub(crate) face: Option<FontFaceId>,
    pub(crate) instance: Option<InstancedFaceId>,
}

pub(crate) fn fallback_text_spans(
    text: &str,
    request: TextShapeRequest<'_>,
    database: &FontDatabase,
) -> Vec<FallbackTextSpan> {
    let query = font_query_for_style(request.style);
    let default_family = request
        .style
        .font_family
        .as_deref()
        .or(request.style.font.as_deref())
        .map(str::trim)
        .filter(|family| !family.is_empty());
    let mut spans = Vec::<FallbackTextSpan>::new();
    for (start, cluster) in text.grapheme_indices(true) {
        let end = start + cluster.len();
        let codepoints = cluster.chars().collect::<Vec<_>>();
        let face = database.resolve_shaping_face_for_cluster(
            font_script_for_cluster(cluster),
            &codepoints,
            &query,
            request.language,
        );
        let family = face
            .and_then(|face| database.face_family_name(face))
            .map(|family| family.0)
            .or_else(|| default_family.map(str::to_string));
        let instance = face.and_then(|face| {
            database
                .effective_instance_id(
                    face,
                    UiResolvedStyle::normalized_font_weight(request.style.font_weight),
                )
                .ok()
        });
        if let Some(previous) = spans.last_mut() {
            if previous.family == family
                && previous.face == face
                && previous.instance == instance
                && previous.range.end == start
            {
                previous.range.end = end;
                continue;
            }
        }
        spans.push(FallbackTextSpan {
            range: start..end,
            family,
            face,
            instance,
        });
    }
    spans
}

fn font_query_for_style(style: &UiResolvedStyle) -> FontQuery {
    let family = style
        .font_family
        .as_deref()
        .or(style.font.as_deref())
        .unwrap_or_default();
    FontQuery {
        families: vec![FontFamilyName::from(family)],
        weight: FontWeight::clamped(UiResolvedStyle::normalized_font_weight(style.font_weight)),
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    }
}
