use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::{TextDirection, TextGlyph};
use crate::text::font::FontCollectionRevision;
use crate::text::shaping::{
    BidiInvariantError, BidiLineOrder, TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome,
    analyze_bidi_line,
};
use crate::text::{TextRange, TextStyle};

mod fragment;
mod glyph_projection;
mod input_validation;

pub(crate) use fragment::CanonicalLogicalVirtualLineFragment;

/// Private logical input retained for a final line that owns generated display fragments.
///
/// The resolved UI line intentionally keeps physical visual text for consumers. That text must
/// never be used as a fresh RTL shaping input, because it loses the logical context required by
/// Arabic shaping. This sequence preserves the logical display input and its explicit virtual
/// anchors until the renderer artifact has been built.
#[derive(Clone, Debug)]
pub(crate) struct LogicalVirtualLineSequence {
    text: Arc<str>,
    base_direction: TextDirection,
    clusters: Vec<LogicalVirtualLineCluster>,
    visual_to_logical: Vec<usize>,
    fragment: Option<Arc<CanonicalLogicalVirtualLineFragment>>,
    artifact_projection_rejected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LogicalVirtualLineCluster {
    logical_range: TextRange,
    source_range: TextRange,
    style_owner_source_range: Option<TextRange>,
    replaced_source_range: Option<TextRange>,
    virtual_role: Option<LogicalVirtualFragmentRole>,
    external: bool,
    visual_index: usize,
    bidi_level: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LogicalVirtualFragmentRole {
    Ellipsis,
    DiscretionaryHyphen,
    Justification,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LogicalVirtualSourceReceipt {
    pub(crate) style_source_range: TextRange,
    pub(crate) replaced_source_range: Option<TextRange>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LogicalVisualClusterReceipt {
    pub(crate) visual_index: usize,
    pub(crate) source_range: TextRange,
    pub(crate) replaced_source_range: Option<TextRange>,
    pub(crate) external: bool,
    pub(crate) right_to_left: bool,
}

impl LogicalVirtualLineSequence {
    /// Compares the immutable rebuild contract while excluding the regenerable shaped cache.
    pub(crate) fn has_same_artifact_identity(&self, other: &Self) -> bool {
        self.text == other.text
            && self.base_direction == other.base_direction
            && self.clusters == other.clusters
            && self.visual_to_logical == other.visual_to_logical
            && self.artifact_projection_rejected == other.artifact_projection_rejected
    }

    /// Creates a sequence only when at least one display cluster has an explicit zero-width
    /// source anchor. Normal source-congruent lines continue through the retained-fragment path.
    pub(crate) fn new(
        text: Arc<str>,
        base_direction: TextDirection,
        source_ranges: Vec<TextRange>,
    ) -> Option<Self> {
        let style_owner_source_ranges = vec![None; source_ranges.len()];
        Self::new_with_style_owners(
            text,
            base_direction,
            source_ranges,
            style_owner_source_ranges,
        )
    }

    pub(crate) fn new_with_style_owners(
        text: Arc<str>,
        base_direction: TextDirection,
        source_ranges: Vec<TextRange>,
        style_owner_source_ranges: Vec<Option<TextRange>>,
    ) -> Option<Self> {
        let replaced_source_ranges = vec![None; source_ranges.len()];
        Self::new_with_source_receipts(
            text,
            base_direction,
            source_ranges,
            style_owner_source_ranges,
            replaced_source_ranges,
        )
    }

    pub(crate) fn new_with_source_receipts(
        text: Arc<str>,
        base_direction: TextDirection,
        source_ranges: Vec<TextRange>,
        style_owner_source_ranges: Vec<Option<TextRange>>,
        replaced_source_ranges: Vec<Option<TextRange>>,
    ) -> Option<Self> {
        let external_clusters = vec![false; source_ranges.len()];
        Self::new_with_source_receipts_and_external_clusters(
            text,
            base_direction,
            source_ranges,
            style_owner_source_ranges,
            replaced_source_ranges,
            external_clusters,
        )
    }

    pub(crate) fn new_with_source_receipts_and_external_clusters(
        text: Arc<str>,
        base_direction: TextDirection,
        source_ranges: Vec<TextRange>,
        style_owner_source_ranges: Vec<Option<TextRange>>,
        replaced_source_ranges: Vec<Option<TextRange>>,
        external_clusters: Vec<bool>,
    ) -> Option<Self> {
        let virtual_roles = vec![None; source_ranges.len()];
        Self::new_with_source_receipts_external_clusters_and_roles(
            text,
            base_direction,
            source_ranges,
            style_owner_source_ranges,
            replaced_source_ranges,
            external_clusters,
            virtual_roles,
        )
    }

    pub(crate) fn new_with_source_receipts_external_clusters_and_roles(
        text: Arc<str>,
        base_direction: TextDirection,
        source_ranges: Vec<TextRange>,
        style_owner_source_ranges: Vec<Option<TextRange>>,
        replaced_source_ranges: Vec<Option<TextRange>>,
        external_clusters: Vec<bool>,
        virtual_roles: Vec<Option<LogicalVirtualFragmentRole>>,
    ) -> Option<Self> {
        if text.is_empty() || source_ranges.is_empty() {
            return None;
        }
        let logical_ranges = text
            .grapheme_indices(true)
            .map(|(start, grapheme)| TextRange {
                start,
                end: start + grapheme.len(),
            })
            .collect::<Vec<_>>();
        if !input_validation::logical_virtual_sequence_input_is_valid(
            text.as_ref(),
            &logical_ranges,
            &source_ranges,
            &style_owner_source_ranges,
            &replaced_source_ranges,
            &external_clusters,
            &virtual_roles,
        ) {
            return None;
        }
        let cluster_count = logical_ranges.len();
        Some(Self {
            text,
            base_direction,
            visual_to_logical: (0..cluster_count).collect(),
            fragment: None,
            artifact_projection_rejected: false,
            clusters: logical_ranges
                .into_iter()
                .zip(source_ranges)
                .zip(style_owner_source_ranges)
                .zip(replaced_source_ranges)
                .zip(external_clusters)
                .zip(virtual_roles)
                .enumerate()
                .map(
                    |(
                        visual_index,
                        (
                            (
                                (
                                    ((logical_range, source_range), style_owner_source_range),
                                    replaced_source_range,
                                ),
                                external,
                            ),
                            virtual_role,
                        ),
                    )| {
                        LogicalVirtualLineCluster {
                            logical_range,
                            source_range,
                            style_owner_source_range,
                            replaced_source_range,
                            virtual_role,
                            external,
                            visual_index,
                            bidi_level: 0,
                        }
                    },
                )
                .collect(),
        })
    }

    pub(crate) fn text(&self) -> &str {
        self.text.as_ref()
    }

    pub(crate) const fn base_direction(&self) -> TextDirection {
        self.base_direction
    }

    pub(crate) fn logical_cluster_receipts(
        &self,
    ) -> impl Iterator<
        Item = (
            TextRange,
            TextRange,
            Option<TextRange>,
            Option<TextRange>,
            bool,
        ),
    > + '_ {
        self.clusters.iter().map(|cluster| {
            (
                cluster.logical_range,
                cluster.source_range,
                cluster.style_owner_source_range,
                cluster.replaced_source_range,
                cluster.external,
            )
        })
    }

    pub(crate) fn visual_source_receipts(&self) -> Vec<Option<LogicalVirtualSourceReceipt>> {
        let mut receipts = vec![None; self.clusters.len()];
        for cluster in &self.clusters {
            receipts[cluster.visual_index] =
                cluster.style_owner_source_range.map(|style_source_range| {
                    LogicalVirtualSourceReceipt {
                        style_source_range,
                        replaced_source_range: cluster.replaced_source_range,
                    }
                });
        }
        receipts
    }

    pub(crate) const fn visual_cluster_count(&self) -> usize {
        self.clusters.len()
    }

    pub(crate) fn logical_virtual_fragment_roles(
        &self,
    ) -> impl Iterator<
        Item = (
            TextRange,
            TextRange,
            Option<TextRange>,
            Option<LogicalVirtualFragmentRole>,
        ),
    > + '_ {
        self.clusters.iter().map(|cluster| {
            (
                cluster.logical_range,
                cluster.source_range,
                cluster.replaced_source_range,
                cluster.virtual_role,
            )
        })
    }

    pub(crate) fn visual_cluster_receipts(
        &self,
    ) -> impl Iterator<Item = LogicalVisualClusterReceipt> + '_ {
        self.visual_to_logical
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(visual_index, logical_index)| {
                let cluster = self.clusters.get(logical_index)?;
                Some(LogicalVisualClusterReceipt {
                    visual_index,
                    source_range: cluster.source_range,
                    replaced_source_range: cluster.replaced_source_range,
                    external: cluster.external,
                    right_to_left: cluster.bidi_level % 2 == 1,
                })
            })
    }

    /// A display-BiDi invariant rejected this private route. The resolved layout remains valid,
    /// but no artifact path may reinterpret this sequence as a trusted visual order.
    pub(crate) const fn artifact_projection_allowed(&self) -> bool {
        !self.artifact_projection_rejected
    }

    pub(crate) fn reject_artifact_projection(&mut self) {
        self.artifact_projection_rejected = true;
        self.fragment = None;
    }

    /// Shapes the preserved logical display input once for the current font generation.
    ///
    /// This intentionally happens before UAX#9 materializes the candidate into physical display
    /// text. The retained run is therefore safe for Arabic contextual shaping and is the owner of
    /// both final-line metrics and logical grapheme advances.
    pub(crate) fn shape_fragment_with_provider<P>(
        &mut self,
        style: &TextStyle,
        provider: &mut P,
    ) -> TextLayoutOutcome<()>
    where
        P: TextShapeRunProvider + ?Sized,
    {
        let font_revision = provider.font_collection_revision();
        if self.clusters.iter().any(|cluster| cluster.external) {
            return TextShapingOutcome::failed(
                crate::core::framework::text::TextLayoutError::LayoutFailed,
            );
        }
        if self
            .fragment
            .as_ref()
            .is_some_and(|fragment| fragment.font_collection_revision() == font_revision)
        {
            return TextShapingOutcome::Ready(());
        }
        provider
            .shape_horizontal_range_with_kerning(
                self.text(),
                style,
                self.base_direction,
                TextRange {
                    start: 0,
                    end: self.text.len(),
                },
                true,
            )
            .and_then(|shaped| {
                if provider.font_collection_revision() != font_revision {
                    return TextShapingOutcome::deferred(
                        crate::core::framework::text::TextLayoutError::FontGenerationChanged,
                    );
                }
                CanonicalLogicalVirtualLineFragment::new(shaped, self.text(), style, font_revision)
                    .map(|fragment| {
                        self.fragment = Some(Arc::new(fragment));
                    })
            })
    }

    pub(crate) fn fragment_for_revision(
        &self,
        font_revision: FontCollectionRevision,
    ) -> Option<&CanonicalLogicalVirtualLineFragment> {
        if !self.artifact_projection_allowed() {
            return None;
        }
        self.fragment
            .as_deref()
            .filter(|fragment| fragment.font_collection_revision() == font_revision)
    }

    pub(crate) fn resolve_visual_order(&mut self) -> Result<BidiLineOrder, BidiInvariantError> {
        let ranges = self
            .clusters
            .iter()
            .map(|cluster| cluster.logical_range)
            .collect::<Vec<_>>();
        let order = analyze_bidi_line(
            self.text(),
            self.base_direction,
            TextRange {
                start: 0,
                end: self.text.len(),
            },
            &ranges,
        )?;
        self.record_visual_order(&order)?;
        Ok(order)
    }

    pub(crate) fn record_visual_order(
        &mut self,
        order: &BidiLineOrder,
    ) -> Result<(), BidiInvariantError> {
        if order.visual_indices.len() != self.clusters.len()
            || order.logical_levels.len() != self.clusters.len()
        {
            return Err(BidiInvariantError::ProjectionCardinalityMismatch {
                cluster_count: self.clusters.len(),
                visual_index_count: order.visual_indices.len(),
                level_count: order.logical_levels.len(),
            });
        }
        for (visual_index, logical_index) in order.visual_indices.iter().copied().enumerate() {
            let Some(cluster) = self.clusters.get_mut(logical_index) else {
                return Err(BidiInvariantError::MissingLogicalCluster {
                    logical_index,
                    cluster_count: self.clusters.len(),
                });
            };
            cluster.visual_index = visual_index;
            cluster.bidi_level = order.logical_levels[logical_index];
        }
        self.visual_to_logical.clone_from(&order.visual_indices);
        Ok(())
    }

    /// Projects logical-order glyphs into final physical visual order while restoring source
    /// ownership. The two cursors make this O(G + C), where G is glyph count and C cluster count.
    pub(crate) fn project_logical_glyphs(
        &self,
        glyphs: Vec<TextGlyph>,
        visual_advances: &[f32],
    ) -> Option<Vec<TextGlyph>> {
        glyph_projection::project_logical_glyphs(self, glyphs, visual_advances)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::framework::text::{
        TextDirection, TextGlyph, TextGlyphFlags, TextGlyphRotation as ProjectedGlyphRotation,
    };
    use crate::text::shaping::{TextShapeRunProvider, TextShapingOutcome};
    use crate::text::{
        ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun,
        ShapedGlyphScript, ShapedHardLine, TextOrientation, TextRange, TextStyle, VerticalMode,
    };

    use super::{
        LogicalVirtualFragmentRole, LogicalVirtualLineSequence, LogicalVirtualSourceReceipt,
    };

    #[test]
    fn logical_virtual_sequence_projects_ellipsis_to_its_zero_width_anchor() {
        let mut sequence = LogicalVirtualLineSequence::new(
            Arc::from("a…b"),
            TextDirection::LeftToRight,
            vec![range(0, 1), range(1, 1), range(1, 2)],
        )
        .expect("virtual display clusters create a logical sidecar");
        sequence
            .resolve_visual_order()
            .expect("display-owned UAX#9 order");

        let glyphs = sequence
            .project_logical_glyphs(
                vec![glyph(0, 0..1), glyph(1, 1..4), glyph(2, 4..5)],
                &[4.0, 16.0, 4.0],
            )
            .expect("logical glyphs project to the visual sequence");

        assert_eq!(glyphs[0].source_range, 0..1);
        assert_eq!(glyphs[0].advance, 4.0);
        assert_eq!(glyphs[1].source_range, 1..1);
        assert_eq!(glyphs[1].advance, 16.0);
        assert!(glyphs[1].flags.virtual_glyph);
        assert_eq!(glyphs[2].source_range, 1..2);
        assert_eq!(glyphs[2].advance, 4.0);
    }

    #[test]
    fn logical_virtual_sequence_keeps_external_clusters_out_of_glyph_projection() {
        let mut sequence =
            LogicalVirtualLineSequence::new_with_source_receipts_and_external_clusters(
                Arc::from("a\u{fffc}\u{2026}"),
                TextDirection::LeftToRight,
                vec![range(0, 1), range(1, 4), range(6, 6)],
                vec![None, None, Some(range(4, 6))],
                vec![None, None, Some(range(4, 6))],
                vec![false, true, false],
            )
            .expect("external display cluster creates a logical sidecar");
        sequence
            .resolve_visual_order()
            .expect("display-owned UAX#9 order");

        let glyphs = sequence
            .project_logical_glyphs(vec![glyph(1, 0..1), glyph(2, 4..7)], &[4.0, 16.0, 8.0])
            .expect("text glyphs project around the external cluster");

        assert_eq!(glyphs.len(), 2);
        assert_eq!(glyphs[0].source_range, 0..1);
        assert_eq!(glyphs[0].advance, 4.0);
        assert_eq!(glyphs[1].source_range, 6..6);
        assert_eq!(glyphs[1].advance, 8.0);
        assert!(glyphs[1].flags.virtual_glyph);
    }

    #[test]
    fn logical_virtual_sequence_reorders_rtl_tatweel_without_losing_its_anchor() {
        let mut sequence = LogicalVirtualLineSequence::new(
            Arc::from("سـلام"),
            TextDirection::RightToLeft,
            vec![
                range(0, 2),
                range(2, 2),
                range(2, 4),
                range(4, 6),
                range(6, 8),
            ],
        )
        .expect("tatweel creates a virtual display cluster");
        sequence
            .resolve_visual_order()
            .expect("RTL display sequence resolves UAX#9 order");

        let glyphs = sequence
            .project_logical_glyphs(
                vec![
                    glyph(0, 0..2),
                    glyph(1, 2..4),
                    glyph(2, 4..6),
                    glyph(3, 6..8),
                    glyph(4, 8..10),
                ],
                &[1.0, 2.0, 3.0, 4.0, 5.0],
            )
            .expect("logical RTL glyphs project to physical order");

        assert_eq!(glyphs.len(), 5);
        assert!(glyphs.iter().all(|glyph| glyph.flags.right_to_left));
        let tatweel = glyphs
            .iter()
            .find(|glyph| glyph.flags.virtual_glyph)
            .expect("tatweel remains a virtual glyph");
        assert_eq!(tatweel.source_range, 2..2);
        assert!(tatweel.advance > 0.0);
    }

    #[test]
    fn logical_virtual_sequence_identity_tracks_rebuild_input_and_rejection() {
        let first = LogicalVirtualLineSequence::new(
            Arc::from("a\u{2026}b"),
            TextDirection::LeftToRight,
            vec![range(0, 1), range(1, 1), range(1, 2)],
        )
        .expect("virtual display clusters create a logical sidecar");
        let mut same = first.clone();

        assert!(first.has_same_artifact_identity(&same));
        same.reject_artifact_projection();
        assert!(!first.has_same_artifact_identity(&same));
    }

    #[test]
    fn logical_virtual_sequence_identity_tracks_explicit_source_receipts() {
        let first = LogicalVirtualLineSequence::new_with_source_receipts(
            Arc::from("\u{2026}b"),
            TextDirection::LeftToRight,
            vec![range(3, 3), range(3, 4)],
            vec![Some(range(3, 4)), None],
            vec![Some(range(0, 3)), None],
        )
        .expect("virtual display clusters create a logical sidecar");
        let different = LogicalVirtualLineSequence::new_with_source_receipts(
            Arc::from("\u{2026}b"),
            TextDirection::LeftToRight,
            vec![range(3, 3), range(3, 4)],
            vec![Some(range(3, 4)), None],
            vec![Some(range(0, 2)), None],
        )
        .expect("virtual display clusters create a logical sidecar");

        assert!(!first.has_same_artifact_identity(&different));
        assert_eq!(
            first.visual_source_receipts(),
            vec![
                Some(LogicalVirtualSourceReceipt {
                    style_source_range: range(3, 4),
                    replaced_source_range: Some(range(0, 3)),
                }),
                None,
            ]
        );
    }

    #[test]
    fn logical_virtual_sequence_identity_tracks_typed_virtual_role() {
        let soft_hyphen =
            LogicalVirtualLineSequence::new_with_source_receipts_external_clusters_and_roles(
                Arc::from("a-"),
                TextDirection::LeftToRight,
                vec![range(0, 1), range(3, 3)],
                vec![None, Some(range(1, 3))],
                vec![None, Some(range(1, 3))],
                vec![false, false],
                vec![None, Some(LogicalVirtualFragmentRole::DiscretionaryHyphen)],
            )
            .expect("typed virtual sequence");
        let mut ellipsis = soft_hyphen.clone();
        ellipsis.clusters[1].virtual_role = Some(LogicalVirtualFragmentRole::Ellipsis);

        assert!(!soft_hyphen.has_same_artifact_identity(&ellipsis));
    }

    #[test]
    fn logical_virtual_sequence_rejects_role_and_display_mismatch() {
        assert!(
            LogicalVirtualLineSequence::new_with_source_receipts_external_clusters_and_roles(
                Arc::from("a-"),
                TextDirection::LeftToRight,
                vec![range(0, 1), range(3, 3)],
                vec![None, Some(range(1, 3))],
                vec![None, Some(range(1, 3))],
                vec![false, false],
                vec![None, Some(LogicalVirtualFragmentRole::Ellipsis)],
            )
            .is_none()
        );
    }

    #[test]
    fn logical_virtual_sequence_reuses_one_shape_for_metrics_and_advances() {
        let mut sequence = LogicalVirtualLineSequence::new(
            Arc::from("a\u{2026}b"),
            TextDirection::LeftToRight,
            vec![range(0, 1), range(1, 1), range(1, 2)],
        )
        .expect("virtual display clusters create a logical sidecar");
        let shaped = Arc::new(ShapedGlyphRun {
            source_text: Arc::from("a\u{2026}b"),
            source_range: TextRange { start: 0, end: 5 },
            unicode_data_snapshot: crate::text::compiled_unicode_data_snapshot_id(),
            primary_face_id: None,
            direction: TextDirection::LeftToRight,
            orientation: TextOrientation::Horizontal,
            vertical_mode: VerticalMode::Mixed,
            include_kerning: true,
            measured_width: 21.0,
            measured_height: 19.0,
            horizontal_composition_receipt: None,
            horizontal_line_raw_metrics: Vec::new(),
            horizontal_glyph_metric_spans: Vec::new(),
            lines: vec![ShapedHardLine {
                line_index: 0,
                source_range: TextRange { start: 0, end: 5 },
                visual_range: TextRange { start: 0, end: 5 },
                measured_width: 21.0,
                baseline: 14.0,
                line_height: 19.0,
                glyphs: vec![
                    shaped_glyph(0, 0, 1, 4.0),
                    shaped_glyph(1, 1, 4, 13.0),
                    shaped_glyph(2, 4, 5, 4.0),
                ],
            }],
        });
        let mut provider = CountingShapeRunProvider {
            shaped: Arc::clone(&shaped),
            shape_calls: 0,
        };

        sequence
            .shape_fragment_with_provider(&TextStyle::default(), &mut provider)
            .into_result()
            .expect("shape the virtual logical fragment");
        sequence
            .shape_fragment_with_provider(&TextStyle::default(), &mut provider)
            .into_result()
            .expect("reuse the current-generation fragment");

        let fragment = sequence
            .fragment_for_revision(provider.font_collection_revision())
            .expect("current generation retains the logical fragment");
        assert!(Arc::ptr_eq(fragment.shaped(), &shaped));
        assert_eq!(fragment.metrics().baseline, 14.0);
        assert_eq!(fragment.metrics().line_height, 19.0);
        assert_eq!(fragment.grapheme_advances(), &[4.0, 13.0, 4.0]);
        assert_eq!(fragment.glyph_clusters().len(), 3);
        assert_eq!(provider.shape_calls, 1);
    }

    fn range(start: usize, end: usize) -> TextRange {
        TextRange { start, end }
    }

    fn glyph(glyph_id: u32, source_range: std::ops::Range<usize>) -> TextGlyph {
        TextGlyph {
            glyph_id,
            source_range: source_range.clone(),
            visual_range: source_range,
            advance: 1.0,
            position: [0.0, 0.0],
            offset: [0.0, 0.0],
            font_face: None,
            font_instance: None,
            rotation: ProjectedGlyphRotation::None,
            bidi_level: 0,
            flags: TextGlyphFlags {
                cluster_start: true,
                ..TextGlyphFlags::default()
            },
            requires_rasterization: false,
        }
    }

    fn shaped_glyph(glyph_id: u32, start: usize, end: usize, advance: f32) -> ShapedGlyph {
        ShapedGlyph {
            glyph_id,
            font_id: None,
            font_instance_id: None,
            source_range: TextRange { start, end },
            visual_range: TextRange { start, end },
            advance,
            x: 0.0,
            y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            direction: TextDirection::LeftToRight,
            bidi_level: 0,
            cluster_flags: ShapedGlyphClusterFlags::default(),
            rotation: ShapedGlyphRotation::None,
            script: ShapedGlyphScript::default(),
        }
    }

    struct CountingShapeRunProvider {
        shaped: Arc<ShapedGlyphRun>,
        shape_calls: usize,
    }

    impl TextShapeRunProvider for CountingShapeRunProvider {
        fn shape_horizontal_range_with_kerning(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _direction: TextDirection,
            _source_range: TextRange,
            _include_kerning: bool,
        ) -> TextShapingOutcome {
            self.shape_calls = self.shape_calls.saturating_add(1);
            TextShapingOutcome::Ready(Arc::clone(&self.shaped))
        }
    }
}
