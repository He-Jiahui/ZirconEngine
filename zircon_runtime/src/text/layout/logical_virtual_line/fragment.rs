use std::sync::Arc;

use crate::text::font::FontCollectionRevision;
use crate::text::shaping::TextShapingOutcome;
use crate::text::{ShapedGlyphRun, TextStyle};

use super::super::{HorizontalLineFragmentGeometry, MeasuredGlyphCluster, TextLineMetrics};

/// One logical display fragment shaped before virtual candidates become physical UI text.
///
/// Generated display clusters cannot share a `CanonicalPhysicalLineFragment`: their zero-width
/// source anchors differ from the logical shaping input. This private owner gives that input the
/// same one-shape contract for line metrics, grapheme advances, and artifact projection.
#[derive(Clone, Debug)]
pub(crate) struct CanonicalLogicalVirtualLineFragment {
    geometry: HorizontalLineFragmentGeometry,
    font_revision: FontCollectionRevision,
}

impl CanonicalLogicalVirtualLineFragment {
    pub(super) fn new(
        shaped: Arc<ShapedGlyphRun>,
        text: &str,
        style: &TextStyle,
        font_revision: FontCollectionRevision,
    ) -> TextShapingOutcome<Self> {
        HorizontalLineFragmentGeometry::from_shaped(shaped, text, style).map(|geometry| Self {
            geometry,
            font_revision,
        })
    }

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
}
