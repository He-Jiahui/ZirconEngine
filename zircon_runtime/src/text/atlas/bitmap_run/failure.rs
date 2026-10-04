//! 区分可跨帧恢复的容量阻塞与必须修正输入的永久失败。
//! 提交层只为本帧不能驱逐的页面排入重试并保留占位，其他错误交给报告层诊断。

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
/// 保留一次字形出现的来源与最早重试帧；不是拥有像素的缓存。
/// 跨帧使用者还须保留或重新取得同一光栅键的像素，并在字体失效时丢弃队列。
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
