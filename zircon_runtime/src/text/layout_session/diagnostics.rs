//! 收集文字会话的布局、整形与几何诊断；此模块只归类观测并复制回执数据。
//! 父模块在 `begin_frame` 重置会话报告；错误处置、代次准入与缓存提交由调用方决定。

use crate::core::framework::text::TextLayoutError;
use crate::text::model::TextShapingRequestDiagnostics;
use crate::text::shaping::{TextShapingDiagnosticsReport, TextShapingFailure};
use crate::text::{
    compiled_unicode_data_snapshot_id, ShapedGlyphRun, TextLayoutGeometryOwner,
    TextLayoutGeometryViolation,
};

/// 布局错误的饱和分类计数与编译期 Unicode 数据标识快照。
///
/// # 使用时机与约束
/// `Default` 捕获 Unicode 数据的代次和指纹，不代表字体数据库代次。
/// 字体代次变化只记延期；其余已记录错误计入回退总数和一个分类，计数不表示已成功呈现。
/// 会话持有时由父模块 `SharedTextLayoutSession::begin_frame` 重置；独立报告由持有方管理。
/// 复制报告只复制标量，不能延长会话、字体集合或布局产物的生命周期。
///
/// # 参考
/// 父模块 `SharedTextLayoutSession::diagnostics_report` 和 `record_layout_error` 是会话侧入口。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextLayoutFallbackReport {
    pub unicode_data_generation: u64,
    pub unicode_data_fingerprint: u64,
    pub fallback_count: u64,
    pub generation_deferred_count: u64,
    pub invalid_font_size_count: u64,
    pub invalid_language_count: u64,
    pub bidi_invariant_count: u64,
    pub geometry_too_large_count: u64,
    pub other_error_count: u64,
}

impl Default for TextLayoutFallbackReport {
    fn default() -> Self {
        let unicode_data = compiled_unicode_data_snapshot_id();
        Self {
            unicode_data_generation: unicode_data.generation(),
            unicode_data_fingerprint: unicode_data.fingerprint(),
            fallback_count: 0,
            generation_deferred_count: 0,
            invalid_font_size_count: 0,
            invalid_language_count: 0,
            bidi_invariant_count: 0,
            geometry_too_large_count: 0,
            other_error_count: 0,
        }
    }
}

impl TextLayoutFallbackReport {
    /// 累计一次错误观测；不执行回退，也不改变错误值。
    ///
    /// # 参数
    /// `error` 仅在本次调用期间借用；未单列的错误归入 `other_error_count`。
    pub(crate) fn record(&mut self, error: &TextLayoutError) {
        if matches!(error, TextLayoutError::FontGenerationChanged) {
            self.generation_deferred_count = self.generation_deferred_count.saturating_add(1);
            return;
        }
        self.fallback_count = self.fallback_count.saturating_add(1);
        match error {
            TextLayoutError::InvalidFontSize => {
                self.invalid_font_size_count = self.invalid_font_size_count.saturating_add(1);
            }
            TextLayoutError::InvalidLanguage => {
                self.invalid_language_count = self.invalid_language_count.saturating_add(1);
            }
            TextLayoutError::BidiInvariant => {
                self.bidi_invariant_count = self.bidi_invariant_count.saturating_add(1);
            }
            TextLayoutError::GeometryTooLarge => {
                self.geometry_too_large_count = self.geometry_too_large_count.saturating_add(1);
            }
            _ => {
                self.other_error_count = self.other_error_count.saturating_add(1);
            }
        }
    }
}

/// 几何拒绝时复制的最近一次回执数据，不持有被拒绝的布局结果。
/// 区间、范围数值和工作量来自调用方；本类型不检查其有效性，也不执行裁剪或预算准入。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextLayoutGeometryRejectionReceipt {
    pub(crate) owner: TextLayoutGeometryOwner,
    pub(crate) source_range: Option<(u32, u32)>,
    pub(crate) attempted_extent: f32,
    pub(crate) admitted_extent: f32,
    pub(crate) work_units: usize,
}

/// 累计拒绝次数并保留最后一份回执；不会保存完整拒绝历史。
/// 它与布局回退计数独立，记录几何拒绝不会自动增加 `fallback_count`。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TextLayoutGeometryReport {
    pub(crate) rejection_count: u64,
    pub(crate) last_rejection: Option<TextLayoutGeometryRejectionReceipt>,
}

impl TextLayoutGeometryReport {
    /// 复制调用方提供的几何观测，并以本次回执替换上一份。
    ///
    /// # 参数与约束
    /// `owner`、`source_range` 标识观测位置；`violation` 与 `work_units` 已由调用方产生，此处不再校验。
    fn record_rejection(
        &mut self,
        owner: TextLayoutGeometryOwner,
        violation: TextLayoutGeometryViolation,
        source_range: Option<(u32, u32)>,
        work_units: usize,
    ) {
        self.rejection_count = self.rejection_count.saturating_add(1);
        self.last_rejection = Some(TextLayoutGeometryRejectionReceipt {
            owner,
            source_range,
            attempted_extent: violation.attempted_extent,
            admitted_extent: violation.admitted_extent,
            work_units,
        });
    }
}

/// 按值聚合布局、整形和几何诊断，供父会话读取快照。
/// `begin_frame` 重置此聚合；`clear` 只清缓存，不重置诊断。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TextLayoutSessionDiagnostics {
    pub(crate) layout_fallbacks: TextLayoutFallbackReport,
    pub(crate) shaping: TextShapingDiagnosticsReport,
    pub(crate) geometry: TextLayoutGeometryReport,
}

impl TextLayoutSessionDiagnostics {
    pub(crate) fn record_layout_error(&mut self, error: &TextLayoutError) {
        // 字体 generation 变化单独计为 deferred，避免与已经选择 fallback 的终态错误混计。
        self.layout_fallbacks.record(error);
    }

    pub(crate) fn record_geometry_rejection(
        &mut self,
        owner: TextLayoutGeometryOwner,
        violation: TextLayoutGeometryViolation,
        source_range: Option<(u32, u32)>,
        work_units: usize,
    ) {
        self.geometry
            .record_rejection(owner, violation, source_range, work_units);
    }

    pub(crate) fn record_ready_run(
        &mut self,
        run: &ShapedGlyphRun,
        request: TextShapingRequestDiagnostics,
    ) {
        // 此处只转交观测；父会话随后仍会核对字体代次，可能把结果改为延期。
        self.shaping.record_ready_run(run, request);
    }

    pub(crate) fn record_terminal_failure(&mut self, failure: &TextShapingFailure) {
        self.shaping.record_terminal_failure(failure);
    }

    pub(crate) fn record_deferred_failure(&mut self, failure: &TextShapingFailure) {
        self.shaping.record_deferred_failure(failure);
    }

    pub(crate) fn merge_shaping(&mut self, report: TextShapingDiagnosticsReport) {
        self.shaping.merge(report);
    }
}

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;
