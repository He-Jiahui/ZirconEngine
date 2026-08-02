use std::ops::Range;

use crate::text::TextStyle;
use unicode_segmentation::UnicodeSegmentation;

use crate::text::font::FontDatabase;
use crate::text::{
    BackendShapeRequest, FontFaceId, FontFamilyName, FontQuery, FontStretch, FontStyle, FontWeight,
    InstancedFaceId,
};

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
    request: BackendShapeRequest<'_>,
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
    let mut face_resolver = database.begin_shaping_face_resolution(&query, request.language);
    if let Some(face) = face_resolver
        .as_ref()
        .filter(|resolver| resolver.primary_covers_text(text))
        .map(|resolver| resolver.primary_face())
    {
        let family = database
            .face_family_name(face)
            .map(|family| family.0)
            .or_else(|| default_family.map(str::to_string));
        let instance = database
            .effective_instance_id(
                face,
                TextStyle::normalized_font_weight(request.style.font_weight),
            )
            .ok();
        return vec![FallbackTextSpan {
            range: 0..text.len(),
            family,
            face: Some(face),
            instance,
        }];
    }
    let mut spans = Vec::<FallbackTextSpan>::new();
    let mut cluster_codepoints = Vec::new();
    for (start, cluster) in text.grapheme_indices(true) {
        let end = start + cluster.len();
        cluster_codepoints.clear();
        cluster_codepoints.extend(cluster.chars());
        let face = face_resolver.as_mut().map(|resolver| {
            resolver.resolve(font_script_for_cluster(cluster), &cluster_codepoints)
        });
        let instance = face.and_then(|face| {
            database
                .effective_instance_id(
                    face,
                    TextStyle::normalized_font_weight(request.style.font_weight),
                )
                .ok()
        });
        if let Some(previous) = spans.last_mut() {
            if previous.face == face && previous.instance == instance && previous.range.end == start
            {
                previous.range.end = end;
                continue;
            }
        }
        let family = face
            .and_then(|face| database.face_family_name(face))
            .map(|family| family.0)
            .or_else(|| default_family.map(str::to_string));
        spans.push(FallbackTextSpan {
            range: start..end,
            family,
            face,
            instance,
        });
    }
    spans
}

fn font_query_for_style(style: &TextStyle) -> FontQuery {
    let family = style
        .font_family
        .as_deref()
        .or(style.font.as_deref())
        .unwrap_or_default();
    FontQuery {
        families: vec![FontFamilyName::from(family)],
        weight: FontWeight::clamped(TextStyle::normalized_font_weight(style.font_weight)),
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::fallback_text_spans;
    use crate::core::framework::text::TextDirection;
    use crate::text::font::FontDatabase;
    use crate::text::BackendShapeRequest;
    use crate::text::{TextRange, TextStyle};

    #[test]
    fn fallback_spans_keep_primary_coverage_in_one_contiguous_span() {
        let mut database = FontDatabase::default();
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
        let primary = database
            .register_font_file(source, Some("Layout Primary"), 0)
            .expect("tracked layout font should register");
        let style = TextStyle {
            font_family: Some("Layout Primary".to_string()),
            ..TextStyle::default()
        };
        let text = "Workbench layout label";
        let spans = fallback_text_spans(
            text,
            BackendShapeRequest::horizontal(
                text,
                &style,
                TextDirection::LeftToRight,
                TextRange {
                    start: 0,
                    end: text.len(),
                },
            ),
            &database,
        );

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].range, 0..text.len());
        assert_eq!(spans[0].face, Some(primary));
    }

    #[test]
    fn fallback_itemization_reuses_cluster_codepoint_storage() {
        let source = include_str!("fallback_spans.rs");
        let compact = source
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();

        assert!(!compact.contains("text.chars().collect::<Vec<_>>()"));
        assert!(compact.contains("cluster_codepoints.clear()"));
    }
}
