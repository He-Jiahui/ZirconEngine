#[cfg(test)]
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextAlign, UiTextDirection,
    UiTextRange, UiTextWritingMode,
};

use crate::text::font::FontCollectionSnapshot;
use crate::text::layout::GraphemeAdvanceIndex;
use crate::text::shaping::{FontCollectionTextShapeRunProvider, TextShapingOutcome};
use crate::text::{text_style, TextRange};

#[derive(Clone, Copy)]
pub(super) struct SourceMeasureContext<'a> {
    pub(super) text: &'a str,
    pub(super) style: &'a UiResolvedStyle,
    pub(super) font_collection: &'a FontCollectionSnapshot,
}

pub(super) struct SourceLineGeometry {
    index: GraphemeAdvanceIndex,
    source_start: usize,
    source_end: usize,
}

impl SourceLineGeometry {
    pub(super) fn for_line(
        layout: &UiResolvedTextLayout,
        line: &UiResolvedTextLine,
        measure_context: Option<SourceMeasureContext<'_>>,
    ) -> Option<Self> {
        let measure_context = measure_context?;
        if !line_accepts_source_measure(layout, line, measure_context.text) {
            return None;
        }
        let mut provider = FontCollectionTextShapeRunProvider::new(measure_context.font_collection);
        let index = match GraphemeAdvanceIndex::measured_with_provider(
            &line.text,
            &text_style(measure_context.style),
            &mut provider,
        ) {
            TextShapingOutcome::Ready(index) => index,
            TextShapingOutcome::Deferred(_) | TextShapingOutcome::Failed(_) => return None,
        };
        Some(Self {
            index,
            source_start: line.source_range.start,
            source_end: line.source_range.end,
        })
    }

    pub(super) fn caret_advance(
        &self,
        caret: &zircon_runtime_interface::ui::surface::UiTextCaret,
    ) -> f32 {
        let local_offset = caret
            .offset
            .clamp(self.source_start, self.source_end)
            .saturating_sub(self.source_start);
        if let Some((leading, trailing)) = self.index.ltr_atomic_caret_span(local_offset) {
            return match caret.affinity {
                zircon_runtime_interface::ui::surface::UiTextCaretAffinity::Upstream => leading,
                zircon_runtime_interface::ui::surface::UiTextCaretAffinity::Downstream => trailing,
            };
        }
        self.index.advance(0, local_offset)
    }

    pub(super) fn range_advance_span(&self, range: UiTextRange) -> Option<(f32, f32)> {
        let start = range.start.max(self.source_start);
        let end = range.end.min(self.source_end);
        if start >= end {
            return None;
        }
        let local = self.index.coalesce_atomic_source_range(TextRange {
            start: start.saturating_sub(self.source_start),
            end: end.saturating_sub(self.source_start),
        });
        Some((
            self.index.advance(0, local.start),
            self.index.advance(0, local.end),
        ))
    }

    pub(super) fn caret_at_advance(
        &self,
        visual_advance: f32,
    ) -> Option<(zircon_runtime_interface::ui::surface::UiTextCaret, usize)> {
        let (range, leading_half) = self.index.ltr_caret_hit(visual_advance)?;
        let local_offset = if leading_half { range.start } else { range.end };
        Some((
            zircon_runtime_interface::ui::surface::UiTextCaret {
                offset: self.source_start.saturating_add(local_offset),
                affinity: if leading_half {
                    zircon_runtime_interface::ui::surface::UiTextCaretAffinity::Downstream
                } else {
                    zircon_runtime_interface::ui::surface::UiTextCaretAffinity::Upstream
                },
            },
            self.index.grapheme_boundary_index(local_offset),
        ))
    }
}

#[cfg(test)]
pub(super) fn source_prefix_range_for_visual_offset(
    line: &UiResolvedTextLine,
    visual_offset: usize,
) -> UiTextRange {
    let local_end = grapheme_floor(line.text.as_str(), visual_offset.min(line.text.len()));
    UiTextRange {
        start: line.source_range.start,
        end: line
            .source_range
            .start
            .saturating_add(local_end)
            .min(line.source_range.end),
    }
}

pub(super) fn line_accepts_source_measure(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    source_text: &str,
) -> bool {
    if matches!(layout.text_align, UiTextAlign::Justify)
        || !matches!(layout.writing_mode, UiTextWritingMode::HorizontalTb)
        || !matches!(layout.direction, UiTextDirection::LeftToRight)
        || !matches!(line.direction, UiTextDirection::LeftToRight)
        || line.ellipsized
        || line.text.contains('\t')
    {
        return false;
    }

    let Some(source_slice) = source_text.get(line.source_range.start..line.source_range.end) else {
        return false;
    };
    if source_slice != line.text {
        return false;
    }

    let [run] = line.runs.as_slice() else {
        return false;
    };
    run.source_range == line.source_range
        && run.visual_range == line.visual_range
        && run.text == line.text
        && matches!(run.direction, UiTextDirection::LeftToRight)
}

#[cfg(test)]
fn grapheme_floor(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return start;
        }
        if start >= offset {
            break;
        }
    }
    offset
}
