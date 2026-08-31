use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::font::FontCollectionRevision;
use crate::text::shaping::{
    BidiInvariantError, BidiLineOrder, TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome,
    analyze_bidi_line,
};
use crate::text::{ShapedGlyphRun, TextRange, TextStyle};

use super::{HorizontalLineFragmentGeometry, MeasuredGlyphCluster, TextLineMetrics};

/// One final physical source slice shaped once for layout metrics, advances, and later artifact
/// projection.
///
/// This owner retains the absolute source range selected by the caller. It does not infer that
/// range from visual text, so synthetic ellipsis and tatweel remain distinct requests.
#[derive(Clone, Debug)]
pub(crate) struct CanonicalPhysicalLineFragment {
    geometry: HorizontalLineFragmentGeometry,
    font_revision: FontCollectionRevision,
    visual_order: Option<BidiLineOrder>,
}

impl CanonicalPhysicalLineFragment {
    pub(crate) fn shaped(&self) -> &Arc<ShapedGlyphRun> {
        self.geometry.shaped()
    }

    pub(crate) const fn font_generation(&self) -> u64 {
        self.font_revision.generation()
    }

    pub(crate) const fn font_collection_revision(&self) -> FontCollectionRevision {
        self.font_revision
    }

    pub(crate) const fn metrics(&self) -> TextLineMetrics {
        self.geometry.metrics()
    }

    pub(crate) fn grapheme_advances(&self) -> &[f32] {
        self.geometry.grapheme_advances()
    }

    pub(crate) fn glyph_clusters(&self) -> &[MeasuredGlyphCluster] {
        self.geometry.glyph_clusters()
    }

    pub(crate) fn visual_order(&self) -> Option<&BidiLineOrder> {
        self.visual_order.as_ref()
    }
}

pub(crate) fn shape_horizontal_physical_line_fragment_with_provider<P>(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
    provider: &mut P,
) -> TextLayoutOutcome<CanonicalPhysicalLineFragment>
where
    P: TextShapeRunProvider + ?Sized,
{
    let font_revision = provider.font_collection_revision();
    provider
        .shape_horizontal_range_with_kerning(text, style, direction, source_range, true)
        .and_then(|shaped| {
            if provider.font_collection_revision() != font_revision {
                return TextShapingOutcome::deferred(
                    crate::core::framework::text::TextLayoutError::FontGenerationChanged,
                );
            }
            let visual_order = if text.is_empty() {
                None
            } else {
                match resolve_physical_line_visual_order(text, direction) {
                    Ok(order) => Some(order),
                    Err(_) => {
                        return TextShapingOutcome::failed(
                            crate::core::framework::text::TextLayoutError::BidiInvariant,
                        );
                    }
                }
            };
            HorizontalLineFragmentGeometry::from_shaped(shaped, text, style).map(|geometry| {
                CanonicalPhysicalLineFragment {
                    font_revision,
                    geometry,
                    visual_order,
                }
            })
        })
}

fn resolve_physical_line_visual_order(
    text: &str,
    direction: TextDirection,
) -> Result<BidiLineOrder, BidiInvariantError> {
    crate::profile_scope!("runtime", "text.layout", "resolve_physical_line_bidi_order");
    let logical_ranges = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| TextRange {
            start,
            end: start.saturating_add(grapheme.len()),
        })
        .collect::<Vec<_>>();
    analyze_bidi_line(
        text,
        direction,
        TextRange {
            start: 0,
            end: text.len(),
        },
        &logical_ranges,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::framework::text::TextDirection;
    use crate::text::shaping::DirectTextShapeRunProvider;
    use crate::text::{TextRange, TextStyle};

    use super::super::horizontal_line_fragment::HorizontalLineFragmentGeometry;
    use super::{
        resolve_physical_line_visual_order, shape_horizontal_physical_line_fragment_with_provider,
    };

    #[test]
    fn physical_fragment_resolves_one_post_wrap_visual_cluster_order() {
        let text = "abc אבג";

        let order = resolve_physical_line_visual_order(text, TextDirection::LeftToRight)
            .expect("mixed physical line has one canonical visual order");

        assert_eq!(order.logical_levels, vec![0, 0, 0, 0, 1, 1, 1]);
        assert_eq!(order.visual_indices, vec![0, 1, 2, 3, 6, 5, 4]);
    }

    #[test]
    fn physical_fragment_keeps_the_callers_absolute_source_range() {
        let mut provider = DirectTextShapeRunProvider::default();
        let fragment = shape_horizontal_physical_line_fragment_with_provider(
            "world",
            &TextStyle::default(),
            TextDirection::LeftToRight,
            TextRange { start: 11, end: 16 },
            &mut provider,
        )
        .into_result()
        .expect("shape final physical fragment");

        assert_eq!(
            fragment.shaped().source_range,
            TextRange { start: 11, end: 16 }
        );
        assert_eq!(fragment.grapheme_advances().len(), 5);
        assert_eq!(fragment.glyph_clusters().len(), 5);
        assert_eq!(
            fragment
                .visual_order()
                .expect("non-empty physical line stores its visual-order receipt")
                .visual_indices,
            vec![0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn horizontal_geometry_keeps_one_shaped_run_for_metrics_and_advances() {
        let mut provider = DirectTextShapeRunProvider::default();
        let style = TextStyle::default();
        let fragment = shape_horizontal_physical_line_fragment_with_provider(
            "world",
            &style,
            TextDirection::LeftToRight,
            TextRange { start: 11, end: 16 },
            &mut provider,
        )
        .into_result()
        .expect("shape final physical fragment");

        let geometry = HorizontalLineFragmentGeometry::from_shaped(
            Arc::clone(fragment.shaped()),
            "world",
            &style,
        )
        .into_result()
        .expect("valid horizontal geometry");

        assert!(Arc::ptr_eq(geometry.shaped(), fragment.shaped()));
        assert_eq!(geometry.metrics(), fragment.metrics());
        assert_eq!(geometry.grapheme_advances(), fragment.grapheme_advances());
        assert_eq!(geometry.glyph_clusters(), fragment.glyph_clusters());
    }
}
