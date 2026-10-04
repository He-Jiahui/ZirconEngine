use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::font::FontCollectionRevision;
use crate::text::shaping::{
    analyze_bidi_line, BidiInvariantError, BidiLineOrder, TextLayoutOutcome, TextShapeRunProvider,
    TextShapingOutcome,
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
#[path = "tests/physical_line_fragment.rs"]
mod tests;
