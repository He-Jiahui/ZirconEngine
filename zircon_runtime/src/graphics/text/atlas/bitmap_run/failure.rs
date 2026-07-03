use super::super::GlyphAtlasFormat;
use super::placeholder::bitmap_placeholder_glyph;
use super::types::{GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasBitmapAllocationFailureReason {
    UnsupportedFormat,
    EmptyContent,
    DataLengthMismatch { expected: usize, actual: usize },
    PageReservationBlocked,
    OversizedGlyph,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapAllocationFailure {
    pub(crate) source_index: usize,
    pub(crate) format: GlyphAtlasFormat,
    pub(crate) reason: GlyphAtlasBitmapAllocationFailureReason,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapQueuedGlyph {
    pub(crate) source_index: usize,
    pub(crate) source: GlyphAtlasBitmapSource,
    pub(crate) retry_frame_index: u64,
}

pub(super) fn record_bitmap_allocation_failure(
    plan: &mut GlyphAtlasBitmapRunPlan,
    source_index: usize,
    source: GlyphAtlasBitmapSource,
    reason: GlyphAtlasBitmapAllocationFailureReason,
    frame_index: u64,
) {
    if reason == GlyphAtlasBitmapAllocationFailureReason::PageReservationBlocked {
        let retry_frame_index = frame_index.saturating_add(1);
        plan.blocked_glyphs.push(GlyphAtlasBitmapQueuedGlyph {
            source_index,
            source,
            retry_frame_index,
        });
        plan.placeholder_glyphs.push(bitmap_placeholder_glyph(
            source_index,
            source,
            retry_frame_index,
        ));
    }
    plan.allocation_failures
        .push(bitmap_allocation_failure(source_index, source, reason));
}

fn bitmap_allocation_failure(
    source_index: usize,
    source: GlyphAtlasBitmapSource,
    reason: GlyphAtlasBitmapAllocationFailureReason,
) -> GlyphAtlasBitmapAllocationFailure {
    GlyphAtlasBitmapAllocationFailure {
        source_index,
        format: source.format,
        reason,
    }
}
