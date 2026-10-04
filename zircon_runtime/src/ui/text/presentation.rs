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
        let hard_lines = hard_lines(source_text);
        let mut display_text = String::with_capacity(secure_display_capacity(source_text.len()));
        let mut clusters = Vec::with_capacity(source_text.len());
        let mut lines = Vec::with_capacity(hard_lines.len());

        for hard_line in hard_lines {
            let source_content = &source_text[hard_line.content.clone()];
            let source_start = hard_line.content.start;
            let display_start = display_text.len();
            let cluster_start = clusters.len();
            let graphemes = source_content.grapheme_indices(true);
            let grapheme_capacity = graphemes.size_hint().1.unwrap_or(source_content.len());
            let mut logical_ranges = Vec::with_capacity(grapheme_capacity);

            for (source_offset, grapheme) in graphemes {
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
        let mut display_text = String::with_capacity(secure_display_capacity(source_text.len()));
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

// 按源字节数的三倍预留遮罩容量；超出可分配范围时退回源长度，遮罩与分行逻辑不变。
fn secure_display_capacity(source_len: usize) -> usize {
    source_len
        .checked_mul(MASK_GLYPH.len_utf8())
        .filter(|capacity| *capacity <= isize::MAX as usize)
        .unwrap_or(source_len)
}

#[cfg(test)]
#[path = "tests/presentation.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/presentation_capacity_tests.rs"]
mod capacity_tests;
