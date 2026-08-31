use std::sync::Arc;

use crate::{
    core::framework::text::TextDirection,
    text::{hard_lines, TextRange},
};
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiEditableTextState, UiRichTextArtifactHandle, UiTextRange,
};

use crate::text::shaping::{
    capture_bidi_line_signature, resolve_bidi_base_direction, BidiInvariantError, BidiLineOrder,
    BidiLineSignature,
};

const MASK_GLYPH: char = '\u{2022}';
const SECURE_TEXT_PRESENTATION_ARTIFACT_IDENTITY: (&str, u8) = ("secure-text-presentation", 1);

/// A display-only secure-text projection.
///
/// The original string is borrowed only while building this value. Once built, no raw text is
/// retained: the renderer receives the mask, while input, selection, and accessibility can use
/// the offset map to retain their original-source semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiSecureTextPresentation {
    display_text: String,
    source_len: usize,
    clusters: Vec<UiSecureTextPresentationCluster>,
    lines: Vec<UiSecureTextPresentationLine>,
}

/// One atomic source/display unit. Text clusters are one mask glyph; hard-line separators retain
/// their delimiter so multiline layout keeps the canonical source segmentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UiSecureTextPresentationCluster {
    pub(crate) source_range: UiTextRange,
    pub(crate) display_range: UiTextRange,
    pub(crate) is_hard_line_separator: bool,
}

/// UAX#9 ordering computed from the original hard line, not from neutral mask glyphs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiSecureTextPresentationBidi {
    pub(crate) resolved_base_direction: TextDirection,
    pub(crate) logical_levels: Vec<u8>,
    pub(crate) visual_indices: Vec<usize>,
    pub(crate) unicode_data_snapshot: crate::text::UnicodeDataSnapshotId,
    signature: Option<BidiLineSignature>,
}

/// A logical hard-line projection. Cluster indexes refer to `UiSecureTextPresentation::clusters`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiSecureTextPresentationLine {
    pub(crate) source_range: UiTextRange,
    pub(crate) display_range: UiTextRange,
    pub(crate) cluster_range: std::ops::Range<usize>,
    pub(crate) bidi: UiSecureTextPresentationBidi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiSecureTextPresentationError {
    Bidi(BidiInvariantError),
    UnsupportedLayoutProjection,
}

/// Process-local marker that tells artifact publication to shape the display mask and remap its
/// glyph ranges through the resolved secure presentation runs. It deliberately retains no source
/// text or offsets beyond the serializable layout's display-to-source run map.
#[derive(Debug)]
pub(crate) struct UiSecureTextPresentationArtifact;

pub(crate) fn register_secure_text_presentation_artifact() -> UiRichTextArtifactHandle {
    UiRichTextArtifactHandle::from_runtime_artifact_with_identity(
        Arc::new(UiSecureTextPresentationArtifact),
        SECURE_TEXT_PRESENTATION_ARTIFACT_IDENTITY,
    )
}

pub(crate) fn is_secure_text_presentation_artifact(handle: &UiRichTextArtifactHandle) -> bool {
    handle
        .downcast_runtime_artifact::<UiSecureTextPresentationArtifact>()
        .is_some()
}

impl UiSecureTextPresentation {
    /// Masks one extended grapheme cluster at a time and preserves original UAX#9 ordering.
    pub(crate) fn new(
        source_text: &str,
        requested_direction: TextDirection,
    ) -> Result<Self, UiSecureTextPresentationError> {
        let mut display_text = String::with_capacity(source_text.len());
        let mut clusters = Vec::new();
        let mut lines = Vec::new();

        for hard_line in hard_lines(source_text) {
            let source_content = &source_text[hard_line.content.clone()];
            let source_start = hard_line.content.start;
            let display_start = display_text.len();
            let cluster_start = clusters.len();
            let mut logical_ranges = Vec::new();

            for (source_offset, grapheme) in source_content.grapheme_indices(true) {
                let source_range = UiTextRange {
                    start: source_start + source_offset,
                    end: source_start + source_offset + grapheme.len(),
                };
                let display_range = UiTextRange {
                    start: display_text.len(),
                    end: display_text.len() + MASK_GLYPH.len_utf8(),
                };
                display_text.push(MASK_GLYPH);
                logical_ranges.push(TextRange {
                    start: source_offset,
                    end: source_offset + grapheme.len(),
                });
                clusters.push(UiSecureTextPresentationCluster {
                    source_range,
                    display_range,
                    is_hard_line_separator: false,
                });
            }

            let bidi = bidi_for_line(source_content, requested_direction, &logical_ranges)?;
            lines.push(UiSecureTextPresentationLine {
                source_range: UiTextRange {
                    start: hard_line.content.start,
                    end: hard_line.content.end,
                },
                display_range: UiTextRange {
                    start: display_start,
                    end: display_text.len(),
                },
                cluster_range: cluster_start..clusters.len(),
                bidi,
            });

            if !hard_line.separator.is_empty() {
                let display_range = UiTextRange {
                    start: display_text.len(),
                    end: display_text.len() + hard_line.separator.len(),
                };
                display_text.push_str(&source_text[hard_line.separator.clone()]);
                clusters.push(UiSecureTextPresentationCluster {
                    source_range: UiTextRange {
                        start: hard_line.separator.start,
                        end: hard_line.separator.end,
                    },
                    display_range,
                    is_hard_line_separator: true,
                });
            }
        }

        Ok(Self {
            display_text,
            source_len: source_text.len(),
            clusters,
            lines,
        })
    }

    pub(crate) fn display_text(&self) -> &str {
        &self.display_text
    }

    /// Returns a display-only mask when UAX#9 signature construction fails before a complete
    /// presentation can be published. The caller must pair it with a fail-closed layout rather
    /// than analyze these neutral mask glyphs as `Auto` text.
    pub(crate) fn mask_display_text(source_text: &str) -> String {
        let mut display_text = String::with_capacity(source_text.len());
        for hard_line in hard_lines(source_text) {
            for (_, _) in source_text[hard_line.content.clone()].grapheme_indices(true) {
                display_text.push(MASK_GLYPH);
            }
            display_text.push_str(&source_text[hard_line.separator]);
        }
        display_text
    }

    pub(crate) fn source_len(&self) -> usize {
        self.source_len
    }

    /// Builds the editable state allowed to cross the render boundary for a secure field.
    ///
    /// Layout keeps source offsets for caret and selection geometry, while text and composition
    /// contents must never leave the input owner. Secure IME is disabled, so dropping composition
    /// is both the security boundary and the intended presentation behavior.
    pub(crate) fn render_editable_state(
        &self,
        source_state: &UiEditableTextState,
    ) -> UiEditableTextState {
        let mut caret = source_state.caret.clone();
        caret.offset = caret.offset.min(self.source_len);
        let selection = source_state.selection.as_ref().map(|selection| {
            let mut selection = selection.clone();
            selection.anchor = selection.anchor.min(self.source_len);
            selection.focus = selection.focus.min(self.source_len);
            selection
        });
        UiEditableTextState {
            text: self.display_text.clone(),
            caret,
            selection,
            composition: None,
            read_only: source_state.read_only,
        }
    }

    pub(crate) fn clusters(&self) -> &[UiSecureTextPresentationCluster] {
        &self.clusters
    }

    pub(crate) fn lines(&self) -> &[UiSecureTextPresentationLine] {
        &self.lines
    }

    /// Returns complete, non-separator mask clusters from one source-owned hard line.
    ///
    /// The construction pass stores both hard-line and cluster display ranges in monotonic order.
    /// Keep the lookup here so every presentation consumer shares the same atomic-boundary
    /// validation instead of scanning the complete hard line once for each wrapped physical row.
    pub(crate) fn clusters_for_display_range(
        &self,
        display_range: UiTextRange,
    ) -> Option<&[UiSecureTextPresentationCluster]> {
        let line = self.display_line_for_range(display_range)?;
        self.clusters_for_line_display_range(line, display_range)
    }

    /// Returns the original/source cluster for a visual position in one masked hard line.
    ///
    /// `display_text` stays in logical order because the layout owner applies UAX#9 later. An
    /// artifact must therefore use this lookup, rather than infer source order from neutral mask
    /// glyphs after the visual line has been materialized.
    pub(crate) fn cluster_for_line_visual_index(
        &self,
        line_index: usize,
        visual_index: usize,
    ) -> Option<UiSecureTextPresentationCluster> {
        let line = self.lines.get(line_index)?;
        let logical_index = *line.bidi.visual_indices.get(visual_index)?;
        line.cluster_range
            .start
            .checked_add(logical_index)
            .and_then(|index| self.clusters.get(index))
            .copied()
            .filter(|cluster| !cluster.is_hard_line_separator)
    }

    /// Reconstructs UAX#9 ordering for a wrapped display subrange without retaining or
    /// reinterpreting the source text. The range must cover consecutive, whole mask clusters from
    /// one hard line. A caller must fail closed when this returns an invariant error rather than
    /// falling back to `Auto` analysis of the neutral bullet string.
    pub(crate) fn bidi_for_display_range(
        &self,
        display_range: UiTextRange,
    ) -> Result<Option<UiSecureTextPresentationBidi>, UiSecureTextPresentationError> {
        let Some(line) = self.display_line_for_range(display_range) else {
            return Ok(None);
        };
        let Some(clusters) = self.clusters_for_line_display_range(line, display_range) else {
            return Ok(None);
        };
        let Some(first_source) = clusters.first().map(|cluster| cluster.source_range.start) else {
            return Ok(None);
        };
        let Some(last_source) = clusters.last().map(|cluster| cluster.source_range.end) else {
            return Ok(None);
        };
        let source_base = line.source_range.start;
        let local_line = TextRange {
            start: first_source.saturating_sub(source_base),
            end: last_source.saturating_sub(source_base),
        };
        let logical_ranges = clusters
            .iter()
            .map(|cluster| TextRange {
                start: cluster.source_range.start.saturating_sub(source_base),
                end: cluster.source_range.end.saturating_sub(source_base),
            })
            .collect::<Vec<_>>();
        let Some(signature) = line.bidi.signature.as_ref() else {
            return Ok(None);
        };
        let order = signature
            .line_order(local_line.start..local_line.end, &logical_ranges)
            .map_err(UiSecureTextPresentationError::Bidi)?;
        Ok(Some(UiSecureTextPresentationBidi {
            resolved_base_direction: order.resolved_base_direction,
            logical_levels: order.logical_levels,
            visual_indices: order.visual_indices,
            unicode_data_snapshot: order.unicode_data_snapshot,
            signature: None,
        }))
    }

    fn display_line_for_range(
        &self,
        display_range: UiTextRange,
    ) -> Option<&UiSecureTextPresentationLine> {
        if display_range.start >= display_range.end {
            return None;
        }
        let line_index = self
            .lines
            .partition_point(|line| line.display_range.end <= display_range.start);
        let line = self.lines.get(line_index)?;
        (line.display_range.start <= display_range.start
            && display_range.end <= line.display_range.end)
            .then_some(line)
    }

    fn clusters_for_line_display_range(
        &self,
        line: &UiSecureTextPresentationLine,
        display_range: UiTextRange,
    ) -> Option<&[UiSecureTextPresentationCluster]> {
        let line_clusters = self.clusters.get(line.cluster_range.clone())?;
        let first = line_clusters
            .partition_point(|cluster| cluster.display_range.end <= display_range.start);
        let after_last = line_clusters
            .partition_point(|cluster| cluster.display_range.start < display_range.end);
        let clusters = line_clusters.get(first..after_last)?;
        (clusters.first()?.display_range.start == display_range.start
            && clusters.last()?.display_range.end == display_range.end
            && !clusters
                .iter()
                .any(|cluster| cluster.is_hard_line_separator))
        .then_some(clusters)
    }

    /// Converts only a canonical source grapheme/separator boundary to its display boundary.
    pub(crate) fn display_offset_for_source_boundary(&self, source_offset: usize) -> Option<usize> {
        if source_offset == 0 {
            return Some(0);
        }
        if source_offset == self.source_len {
            return Some(self.display_text.len());
        }
        let index = self
            .clusters
            .partition_point(|cluster| cluster.source_range.end < source_offset);
        let cluster = self.clusters.get(index)?;
        if cluster.source_range.start == source_offset {
            Some(cluster.display_range.start)
        } else if cluster.source_range.end == source_offset {
            Some(cluster.display_range.end)
        } else {
            None
        }
    }

    /// Converts only a canonical display grapheme/separator boundary to its source boundary.
    pub(crate) fn source_offset_for_display_boundary(
        &self,
        display_offset: usize,
    ) -> Option<usize> {
        if display_offset == 0 {
            return Some(0);
        }
        if display_offset == self.display_text.len() {
            return Some(self.source_len);
        }
        let index = self
            .clusters
            .partition_point(|cluster| cluster.display_range.end < display_offset);
        let cluster = self.clusters.get(index)?;
        if cluster.display_range.start == display_offset {
            Some(cluster.source_range.start)
        } else if cluster.display_range.end == display_offset {
            Some(cluster.source_range.end)
        } else {
            None
        }
    }

    pub(crate) fn display_range_for_source_range(
        &self,
        source_range: UiTextRange,
    ) -> Option<UiTextRange> {
        let start = self.display_offset_for_source_boundary(source_range.start)?;
        let end = self.display_offset_for_source_boundary(source_range.end)?;
        (start <= end).then_some(UiTextRange { start, end })
    }

    pub(crate) fn source_range_for_display_range(
        &self,
        display_range: UiTextRange,
    ) -> Option<UiTextRange> {
        let start = self.source_offset_for_display_boundary(display_range.start)?;
        let end = self.source_offset_for_display_boundary(display_range.end)?;
        (start <= end).then_some(UiTextRange { start, end })
    }
}

fn bidi_for_line(
    source_content: &str,
    requested_direction: TextDirection,
    logical_ranges: &[TextRange],
) -> Result<UiSecureTextPresentationBidi, UiSecureTextPresentationError> {
    let (order, signature) = if logical_ranges.is_empty() {
        (
            BidiLineOrder {
                resolved_base_direction: resolve_bidi_base_direction(
                    source_content,
                    requested_direction,
                ),
                logical_levels: Vec::new(),
                visual_indices: Vec::new(),
                unicode_data_snapshot: crate::text::compiled_unicode_data_snapshot_id(),
            },
            None,
        )
    } else {
        let signature = capture_bidi_line_signature(
            source_content,
            requested_direction,
            TextRange {
                start: 0,
                end: source_content.len(),
            },
        )
        .map_err(UiSecureTextPresentationError::Bidi)?;
        let order = signature
            .line_order(0..source_content.len(), logical_ranges)
            .map_err(UiSecureTextPresentationError::Bidi)?;
        (order, Some(signature))
    };
    Ok(UiSecureTextPresentationBidi {
        resolved_base_direction: order.resolved_base_direction,
        logical_levels: order.logical_levels,
        visual_indices: order.visual_indices,
        unicode_data_snapshot: order.unicode_data_snapshot,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use crate::{core::framework::text::TextDirection, text::shaping::analyze_bidi_line};
    use unicode_segmentation::UnicodeSegmentation;
    use zircon_runtime_interface::ui::surface::UiTextRange;

    use super::{
        is_secure_text_presentation_artifact, register_secure_text_presentation_artifact,
        UiSecureTextPresentation,
    };

    #[test]
    fn secure_presentation_marker_has_stable_runtime_owner_identity() {
        let first = register_secure_text_presentation_artifact();
        let second = register_secure_text_presentation_artifact();

        assert_eq!(first, second);
        assert!(is_secure_text_presentation_artifact(&first));
    }

    #[test]
    fn masks_each_extended_grapheme_and_preserves_hard_line_separators() {
        let source = "a\u{0301}\u{4E2D}\u{1F9D1}\u{200D}\u{1F4BB}\r\nb";

        let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();

        assert_eq!(
            presentation.display_text(),
            "\u{2022}\u{2022}\u{2022}\r\n\u{2022}"
        );
        assert_eq!(presentation.source_len(), source.len());
        assert_eq!(presentation.lines().len(), 2);
        assert_eq!(
            presentation.lines()[0].source_range,
            UiTextRange { start: 0, end: 17 }
        );
        assert_eq!(
            presentation.lines()[0].display_range,
            UiTextRange { start: 0, end: 9 }
        );
        assert_eq!(presentation.lines()[0].cluster_range, 0..3);
        assert_eq!(
            presentation.lines()[1].source_range,
            UiTextRange { start: 19, end: 20 }
        );
        assert_eq!(
            presentation.lines()[1].display_range,
            UiTextRange { start: 11, end: 14 }
        );
        assert_eq!(
            presentation
                .clusters()
                .iter()
                .map(|cluster| (
                    cluster.source_range,
                    cluster.display_range,
                    cluster.is_hard_line_separator,
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    UiTextRange { start: 0, end: 3 },
                    UiTextRange { start: 0, end: 3 },
                    false,
                ),
                (
                    UiTextRange { start: 3, end: 6 },
                    UiTextRange { start: 3, end: 6 },
                    false,
                ),
                (
                    UiTextRange { start: 6, end: 17 },
                    UiTextRange { start: 6, end: 9 },
                    false,
                ),
                (
                    UiTextRange { start: 17, end: 19 },
                    UiTextRange { start: 9, end: 11 },
                    true,
                ),
                (
                    UiTextRange { start: 19, end: 20 },
                    UiTextRange { start: 11, end: 14 },
                    false,
                ),
            ]
        );
    }

    #[test]
    fn source_and_display_ranges_round_trip_at_atomic_boundaries_only() {
        let source = "a\u{0301}\u{4E2D}\u{1F9D1}\u{200D}\u{1F4BB}\r\nb";
        let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();

        let source_range = UiTextRange { start: 3, end: 19 };
        let display_range = UiTextRange { start: 3, end: 11 };
        assert_eq!(
            presentation.display_range_for_source_range(source_range),
            Some(display_range)
        );
        assert_eq!(
            presentation.source_range_for_display_range(display_range),
            Some(source_range)
        );
        assert_eq!(presentation.display_offset_for_source_boundary(1), None);
        assert_eq!(presentation.source_offset_for_display_boundary(1), None);
    }

    #[test]
    fn render_editable_state_contains_only_mask_text_and_source_offsets() {
        let source = "a\u{0301}\u{4E2D}";
        let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();
        let source_state = zircon_runtime_interface::ui::surface::UiEditableTextState {
            text: source.to_string(),
            caret: zircon_runtime_interface::ui::surface::UiTextCaret {
                offset: source.len().saturating_add(4),
                affinity: zircon_runtime_interface::ui::surface::UiTextCaretAffinity::Downstream,
            },
            selection: Some(zircon_runtime_interface::ui::surface::UiTextSelection {
                anchor: 0,
                focus: source.len().saturating_add(2),
            }),
            composition: Some(zircon_runtime_interface::ui::surface::UiTextComposition::default()),
            read_only: true,
        };

        let render_state = presentation.render_editable_state(&source_state);

        assert_eq!(render_state.text, "\u{2022}\u{2022}");
        assert_eq!(render_state.caret.offset, source.len());
        assert_eq!(
            render_state.selection,
            Some(zircon_runtime_interface::ui::surface::UiTextSelection {
                anchor: 0,
                focus: source.len(),
            })
        );
        assert_eq!(render_state.composition, None);
        assert!(render_state.read_only);
    }

    #[test]
    fn preserves_original_rtl_bidi_order_instead_of_reanalyzing_mask_glyphs() {
        let source = "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627} abc";
        let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();
        let source_ranges = source
            .grapheme_indices(true)
            .map(|(start, grapheme)| crate::text::TextRange {
                start,
                end: start + grapheme.len(),
            })
            .collect::<Vec<_>>();
        let expected = analyze_bidi_line(
            source,
            TextDirection::Auto,
            crate::text::TextRange {
                start: 0,
                end: source.len(),
            },
            &source_ranges,
        )
        .unwrap();

        let line = &presentation.lines()[0];
        assert_eq!(
            line.bidi.resolved_base_direction,
            expected.resolved_base_direction
        );
        assert_eq!(line.bidi.logical_levels, expected.logical_levels);
        assert_eq!(line.bidi.visual_indices, expected.visual_indices);
        assert_eq!(
            line.bidi.resolved_base_direction,
            TextDirection::RightToLeft
        );
        assert_ne!(
            line.bidi.visual_indices,
            (0..presentation.clusters().len()).collect::<Vec<_>>()
        );
        let first_visual_logical_index = line.bidi.visual_indices[0];
        assert_eq!(
            presentation.cluster_for_line_visual_index(0, 0),
            presentation
                .clusters()
                .get(first_visual_logical_index)
                .copied()
        );
    }

    #[test]
    fn wrapped_secure_rtl_line_replays_source_l1_instead_of_analyzing_bullets() {
        let source = "\u{05D0}\u{05D1} abc ";
        let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();
        let source_ranges = source
            .grapheme_indices(true)
            .map(|(start, grapheme)| crate::text::TextRange {
                start,
                end: start + grapheme.len(),
            })
            .collect::<Vec<_>>();
        let display_range = UiTextRange {
            start: presentation.clusters()[0].display_range.start,
            end: presentation.clusters()[2].display_range.end,
        };
        let expected = analyze_bidi_line(
            source,
            TextDirection::Auto,
            crate::text::TextRange {
                start: 0,
                end: source_ranges[2].end,
            },
            &source_ranges[..3],
        )
        .unwrap();

        let actual = presentation
            .bidi_for_display_range(display_range)
            .unwrap()
            .unwrap();

        assert_eq!(
            actual.resolved_base_direction,
            expected.resolved_base_direction
        );
        assert_eq!(actual.logical_levels, expected.logical_levels);
        assert_eq!(actual.visual_indices, expected.visual_indices);
        assert_eq!(
            presentation.bidi_for_display_range(UiTextRange { start: 1, end: 3 }),
            Ok(None)
        );
    }

    #[test]
    fn display_range_lookup_reaches_late_hard_lines_without_reinterpreting_mask_text() {
        const HARD_LINE_COUNT: usize = 64;
        let source = (0..HARD_LINE_COUNT)
            .map(|_| "\u{05d0}a")
            .collect::<Vec<_>>()
            .join("\n");
        let presentation = UiSecureTextPresentation::new(&source, TextDirection::Auto).unwrap();
        let line = presentation
            .lines()
            .last()
            .expect("the final hard line must be retained");

        let order = presentation
            .bidi_for_display_range(line.display_range)
            .unwrap()
            .expect("a complete final mask line must replay source-owned bidi");

        assert_eq!(presentation.lines().len(), HARD_LINE_COUNT);
        assert_eq!(order.logical_levels.len(), 2);
        assert_eq!(order.visual_indices.len(), 2);
        assert_eq!(
            presentation.display_offset_for_source_boundary(line.source_range.start),
            Some(line.display_range.start)
        );
        assert_eq!(
            presentation.source_offset_for_display_boundary(line.display_range.end),
            Some(line.source_range.end)
        );
        assert_eq!(
            presentation.bidi_for_display_range(UiTextRange {
                start: line.display_range.start.saturating_add(1),
                end: line.display_range.end,
            }),
            Ok(None),
            "a non-atomic display boundary remains invalid even on a late hard line"
        );
    }

    #[test]
    fn secure_layout_projection_keeps_mask_text_and_original_grapheme_ranges() {
        use crate::ui::text::layout_engine::{apply_secure_text_presentation, layout_text};
        use zircon_runtime_interface::ui::{
            layout::UiFrame,
            surface::{
                UiResolvedStyle, UiTextLineSourceMap, UiTextOverflow, UiTextVisualBoundaryBias,
                UiTextWrap,
            },
        };

        let source = "A \u{0645}\u{0631}\u{062d}\u{0628}\u{0627} \u{4e2d}";
        let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto)
            .expect("a valid source must produce a secure presentation");
        let style = UiResolvedStyle {
            wrap: UiTextWrap::None,
            text_overflow: UiTextOverflow::Clip,
            font_size: 16.0,
            line_height: 20.0,
            ..UiResolvedStyle::default()
        };
        let mut layout = layout_text(
            presentation.display_text(),
            &style,
            UiFrame::new(0.0, 0.0, 160.0, 24.0),
            None,
        );

        apply_secure_text_presentation(&mut layout, &presentation)
            .expect("secure projection must preserve valid source cluster ownership");

        let line = layout.lines.first().expect("one resolved line");
        assert_eq!(line.text, presentation.display_text());
        assert!(line.runs.iter().all(|run| run.text == "\u{2022}"));
        assert_eq!(
            line.runs
                .iter()
                .map(|run| run.source_range)
                .collect::<Vec<_>>(),
            presentation.lines()[0]
                .bidi
                .visual_indices
                .iter()
                .map(|&index| presentation.clusters()[index].source_range)
                .collect::<Vec<_>>()
        );
        let source_map = UiTextLineSourceMap::new(line);
        let caret = source_map.caret_for_visual_boundary(
            0,
            UiTextVisualBoundaryBias::LeadingCurrent,
            usize::MAX,
        );
        assert_ne!(caret.offset, usize::MAX);
        assert!(caret.offset <= source.len());
    }

    #[test]
    fn secure_layout_projection_reanchors_start_for_each_source_owned_hard_line_direction() {
        use crate::ui::text::layout_engine::{apply_secure_text_presentation, layout_text};
        use zircon_runtime_interface::ui::{
            layout::UiFrame,
            surface::{UiResolvedStyle, UiTextAlign, UiTextOverflow, UiTextWrap},
        };

        let source = "abc\n\u{05D0}\u{05D1}";
        let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto)
            .expect("a valid source must produce a secure presentation");
        let style = UiResolvedStyle {
            text_align: UiTextAlign::Start,
            wrap: UiTextWrap::None,
            text_overflow: UiTextOverflow::Clip,
            ..UiResolvedStyle::default()
        };
        let frame = UiFrame::new(10.0, 0.0, 120.0, 48.0);
        let mut layout = layout_text(presentation.display_text(), &style, frame, None);

        apply_secure_text_presentation(&mut layout, &presentation)
            .expect("secure projection must preserve each hard-line direction");

        assert_eq!(layout.lines.len(), 2);
        assert!((layout.lines[0].frame.x - frame.x).abs() < 0.01);
        assert_eq!(
            layout.lines[1].direction,
            zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft
        );
        assert!((layout.lines[1].frame.right() - frame.right()).abs() < 0.01);
    }
}
