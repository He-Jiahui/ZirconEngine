//! 将整形错误归为类型化回执，并按调用观测聚合诊断；此处不执行重试或替代后端。
//! 观测窗口由调用方管理，重复记录会再次累计；参考 `cosmic.rs` 与布局会话、并行整形的消费入口。

use crate::text::font::FontDatabaseError;
use crate::text::model::{
    TextFontResolutionReport, TextShapingBudgetKind, TextShapingFailureCode,
    TextShapingFailureDependency, TextShapingFailureDisposition, TextShapingFailurePhase,
    TextShapingFailureReceipt, TextShapingRequestDiagnostics,
};
use crate::text::{compiled_unicode_data_snapshot_id, ShapedGlyphRun, TextOrientation, TextRange};

use super::backend_error::{BackendFontOperation, BackendShapeError};
use super::bidi::BidiInvariantError;
use super::direct_error::{BackendGlyphInvariantKind, DirectShapeError};
use super::fallback_spans::FallbackItemizationError;
use super::itemize::ItemizationError;
use super::outcome::TextShapingFailure;

/// 从内部错误构造诊断值；保留错误所带坐标与字体标识，不另作有效性验证。
impl TextShapingFailureReceipt {
    /// 字体代次复核失败时生成延期回执；不等待变化完成，也不重启请求。
    pub(crate) const fn font_generation_changed() -> Self {
        Self {
            code: TextShapingFailureCode::FontGenerationChanged,
            phase: TextShapingFailurePhase::FontResolution,
            source_range: None,
            face: None,
            dependency: TextShapingFailureDependency::FontDatabase,
            disposition: TextShapingFailureDisposition::Deferred,
            budget: None,
        }
    }

    /// 将主字体不可用映射为字体解析终态；未提供的范围与字体标识保持为空。
    const fn from_fallback_itemization(error: FallbackItemizationError) -> Self {
        match error {
            FallbackItemizationError::PrimaryFaceUnavailable => Self {
                code: TextShapingFailureCode::FontPrimaryUnavailable,
                phase: TextShapingFailurePhase::FontResolution,
                source_range: None,
                face: None,
                dependency: TextShapingFailureDependency::FontDatabase,
                disposition: TextShapingFailureDisposition::Terminal,
                budget: None,
            },
        }
    }

    /// 借用一次直接整形错误，复制诊断值；返回值不保留对错误的借用。
    /// `orientation` 只影响可替代策略：水平后端及字形不变量错误可替代，预算与 Bidi 错误仍为终态。
    /// 此处既不检查范围或错误字段组合，也不实际执行回退；参考 `cosmic.rs` 的策略消费。
    fn from_direct(error: &DirectShapeError, orientation: TextOrientation) -> Self {
        // 这里只改处置标签，真正是否尝试另一后端由消费方决定。
        let alternate_backend = |mut receipt: Self| {
            if orientation == TextOrientation::Horizontal {
                receipt.disposition = TextShapingFailureDisposition::AlternateBackend;
            }
            receipt
        };
        match error {
            DirectShapeError::Itemization(ItemizationError::InvalidSourceRange { range }) => Self {
                code: TextShapingFailureCode::ItemizationInvalidSourceRange,
                phase: TextShapingFailurePhase::Itemization,
                source_range: Some(*range),
                face: None,
                dependency: TextShapingFailureDependency::SourceText,
                disposition: TextShapingFailureDisposition::Terminal,
                budget: None,
            },
            DirectShapeError::Itemization(ItemizationError::MissingFallbackSpan { range }) => {
                Self {
                    code: TextShapingFailureCode::ItemizationMissingFallbackSpan,
                    phase: TextShapingFailurePhase::Itemization,
                    source_range: Some(*range),
                    face: None,
                    dependency: TextShapingFailureDependency::FontDatabase,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                }
            }
            DirectShapeError::Itemization(ItemizationError::BidiInvariant(source))
            | DirectShapeError::BidiInvariant(source) => bidi_receipt(source),
            DirectShapeError::Backend { range, source } => match source {
                // 源预算先于普通字体数据库错误匹配，避免被水平路径改为可替代。
                BackendShapeError::FontDatabase {
                    operation,
                    face,
                    source: FontDatabaseError::SourceBudget { .. },
                } => Self {
                    code: TextShapingFailureCode::FontSourceBudgetExceeded,
                    phase: backend_font_phase(*operation),
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::WorkBudget,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: Some(TextShapingBudgetKind::FontSourceAdmission),
                },
                BackendShapeError::FontDatabase {
                    operation, face, ..
                } => alternate_backend(Self {
                    code: TextShapingFailureCode::BackendFontDatabase,
                    phase: backend_font_phase(*operation),
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::FontDatabase,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                }),
                BackendShapeError::FaceParseFailed { face, .. } => alternate_backend(Self {
                    code: TextShapingFailureCode::BackendFaceParse,
                    phase: TextShapingFailurePhase::FontLoad,
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::FontFace,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                }),
                BackendShapeError::EmptyGlyphOutput { face } => alternate_backend(Self {
                    code: TextShapingFailureCode::BackendEmptyGlyphOutput,
                    phase: TextShapingFailurePhase::BackendShape,
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::ShapingBackend,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                }),
            },
            DirectShapeError::InvalidSourceRange { range } => Self {
                code: TextShapingFailureCode::DirectInvalidSourceRange,
                phase: TextShapingFailurePhase::InputValidation,
                source_range: Some(*range),
                face: None,
                dependency: TextShapingFailureDependency::SourceText,
                disposition: TextShapingFailureDisposition::Terminal,
                budget: None,
            },
            DirectShapeError::BackendGlyphInvariant { face, range, kind } => {
                alternate_backend(Self {
                    code: backend_glyph_code(*kind),
                    phase: TextShapingFailurePhase::BackendValidation,
                    source_range: Some(*range),
                    face: Some(*face),
                    dependency: TextShapingFailureDependency::ShapingBackend,
                    disposition: TextShapingFailureDisposition::Terminal,
                    budget: None,
                })
            }
        }
    }
}

/// 将通用布局错误和能力回执一同交给上层，保留延期或失败的具体原因。
impl TextShapingFailure {
    /// 构造字体代次变化错误，供上层选择延期入口记录。
    pub(crate) const fn font_generation_changed() -> Self {
        Self::with_receipt(
            crate::core::framework::text::TextLayoutError::FontGenerationChanged,
            TextShapingFailureReceipt::font_generation_changed(),
        )
    }
}

/// 主字体不可用同时保留通用错误与字体解析原因，不在转换中解析字体。
impl From<FallbackItemizationError> for TextShapingFailure {
    /// 按值消费错误；没有来源的坐标与字体标识不会在此补填。
    fn from(error: FallbackItemizationError) -> Self {
        Self::with_receipt(
            crate::core::framework::text::TextLayoutError::FontUnavailable,
            TextShapingFailureReceipt::from_fallback_itemization(error),
        )
    }
}

/// 调用方观测窗口内的失败诊断值快照，可按值复制而不借用整形输入。
///
/// 默认值复制编译期 Unicode 数据标识并清零统计；何时重置由持有方决定。
/// 公开字段允许修改，标识与计数之间没有自动一致性校验；参考 `TextShapingDiagnosticsReport` 的记录入口。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TextShapingFailureReport {
    /// 默认构造时复制的 Unicode 数据代次；记录和合并均不刷新。
    pub unicode_data_generation: u64,
    /// 默认构造时复制的数据指纹；合并保留接收方值。
    pub unicode_data_fingerprint: u64,
    /// 已记录回执次数，包含重复记录，饱和累计。
    pub observed_count: u64,
    /// 可替代处置的回执次数，不代表替代后端已执行或成功。
    pub alternate_backend_count: u64,
    /// 延期处置的回执次数。
    pub deferred_count: u64,
    /// 终态处置的回执次数。
    pub terminal_count: u64,
    /// 按稳定错误码索引保存次数，不建立动态标签表。
    counts: [u64; TextShapingFailureCode::COUNT],
    /// 最近一次记录或非空合并输入留下的回执，不按时间戳排序。
    pub last_failure: Option<TextShapingFailureReceipt>,
}

impl TextShapingFailureReport {
    /// 按错误码读取此快照的累计次数。
    ///
    /// `self` 按值复制，不清空原报告；索引来自错误码定义，公开字段关系不在此验证。
    pub const fn count(self, code: TextShapingFailureCode) -> u64 {
        self.counts[code.index()]
    }

    /// 按回执自身的处置饱和累计并替换最后回执；不去重，不校验字段组合。
    pub(crate) fn record(&mut self, receipt: TextShapingFailureReceipt) {
        self.observed_count = self.observed_count.saturating_add(1);
        self.counts[receipt.code.index()] = self.counts[receipt.code.index()].saturating_add(1);
        match receipt.disposition {
            TextShapingFailureDisposition::AlternateBackend => {
                self.alternate_backend_count = self.alternate_backend_count.saturating_add(1);
            }
            TextShapingFailureDisposition::Deferred => {
                self.deferred_count = self.deferred_count.saturating_add(1);
            }
            TextShapingFailureDisposition::Terminal => {
                self.terminal_count = self.terminal_count.saturating_add(1);
            }
        }
        self.last_failure = Some(receipt);
    }

    /// 逐项饱和加总另一份报告，不检测重复来源或 Unicode 标识相容性。
    /// 接收方标识不变；输入的最后回执非空就覆盖，不按时间比较。
    pub(crate) fn merge(&mut self, other: Self) {
        self.observed_count = self.observed_count.saturating_add(other.observed_count);
        self.alternate_backend_count = self
            .alternate_backend_count
            .saturating_add(other.alternate_backend_count);
        self.deferred_count = self.deferred_count.saturating_add(other.deferred_count);
        self.terminal_count = self.terminal_count.saturating_add(other.terminal_count);
        for (count, other_count) in self.counts.iter_mut().zip(other.counts) {
            *count = count.saturating_add(other_count);
        }
        if other.last_failure.is_some() {
            self.last_failure = other.last_failure;
        }
    }
}

/// 以编译期 Unicode 快照作为新报告的标识，不观察字体数据库代次。
impl Default for TextShapingFailureReport {
    /// 构造零计数、无最后回执的值；之后不自动随外部状态刷新。
    fn default() -> Self {
        let unicode_data = compiled_unicode_data_snapshot_id();
        Self {
            unicode_data_generation: unicode_data.generation(),
            unicode_data_fingerprint: unicode_data.fingerprint(),
            observed_count: 0,
            alternate_backend_count: 0,
            deferred_count: 0,
            terminal_count: 0,
            counts: [0; TextShapingFailureCode::COUNT],
            last_failure: None,
        }
    }
}

/// 按消费入口观测到的后端路由计数，不等同于缓存写入或最终发布次数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextShapingBackendRouteReport {
    /// 无水平组合回执的就绪观测次数。
    pub(crate) direct_run_count: u64,
    /// 有组合回执且替代范围为空的整段替代观测次数。
    pub(crate) alternate_run_count: u64,
    /// 有组合回执且替代范围非空的混合路由观测次数。
    pub(crate) hybrid_run_count: u64,
    /// 整段替代每次计一；混合路由按回执中的替代范围数量累计。
    pub(crate) recovered_source_range_count: u64,
    /// 从延期入口记录且携带能力回执的次数。
    pub(crate) deferred_run_count: u64,
    /// 从终态入口记录且携带能力回执的次数。
    pub(crate) terminal_run_count: u64,
}

impl TextShapingBackendRouteReport {
    /// 借用就绪产物，仅按组合回执形状分类；不检查后续字体代次或发布结果。
    fn record_ready_run(&mut self, run: &ShapedGlyphRun) {
        let Some(receipt) = run.horizontal_composition_receipt.as_deref() else {
            self.direct_run_count = self.direct_run_count.saturating_add(1);
            return;
        };

        if receipt.alternate_ranges.is_empty() {
            self.alternate_run_count = self.alternate_run_count.saturating_add(1);
            self.recovered_source_range_count = self.recovered_source_range_count.saturating_add(1);
        } else {
            self.hybrid_run_count = self.hybrid_run_count.saturating_add(1);
            self.recovered_source_range_count = self
                .recovered_source_range_count
                .saturating_add(receipt.alternate_ranges.len() as u64);
        }
    }

    /// 将另一份路由观测逐项饱和相加，不去重或重新分类。
    fn merge(&mut self, other: Self) {
        self.direct_run_count = self.direct_run_count.saturating_add(other.direct_run_count);
        self.alternate_run_count = self
            .alternate_run_count
            .saturating_add(other.alternate_run_count);
        self.hybrid_run_count = self.hybrid_run_count.saturating_add(other.hybrid_run_count);
        self.recovered_source_range_count = self
            .recovered_source_range_count
            .saturating_add(other.recovered_source_range_count);
        self.deferred_run_count = self
            .deferred_run_count
            .saturating_add(other.deferred_run_count);
        self.terminal_run_count = self
            .terminal_run_count
            .saturating_add(other.terminal_run_count);
    }
}

/// 合并请求工作量、失败原因与路由观测的值快照，由会话或批次持有。
/// 请求、错误与产物只在记录调用期间借用；本报告不租用字体或整形资源。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextShapingDiagnosticsReport {
    /// 按能力回执自己的处置分类的失败统计。
    pub(crate) failures: TextShapingFailureReport,
    /// 按所调用记录入口与组合回执形状分类的路由统计。
    pub(crate) backend_routes: TextShapingBackendRouteReport,
    /// 已合并请求诊断中的尝试次数，不由记录调用本身推导。
    pub(crate) shaping_attempt_count: u64,
    /// 已合并请求诊断中的字体代次重启次数。
    pub(crate) font_generation_restart_count: u64,
    /// 字体解析工作量，各字段沿用其报告的饱和合并规则。
    pub(crate) font_resolution: TextFontResolutionReport,
}

impl TextShapingDiagnosticsReport {
    /// 先合并请求工作，再记录组合回执的首个失败及路由；不枚举所有恢复原因。
    /// 上层随后仍可能因字体代次不符而延期，故此处不表示缓存或发布成功。
    pub(crate) fn record_ready_run(
        &mut self,
        run: &ShapedGlyphRun,
        request: TextShapingRequestDiagnostics,
    ) {
        self.record_request(request);
        if let Some(receipt) = run.horizontal_composition_receipt.as_deref() {
            self.failures.record(receipt.first_failure);
        }
        self.backend_routes.record_ready_run(run);
    }

    /// 先合并错误所带请求工作；无能力回执就返回，不增加失败或路由次数。
    /// 有回执时失败按其处置分类，路由按本终态入口分类，不校验二者是否一致。
    pub(crate) fn record_terminal_failure(&mut self, failure: &TextShapingFailure) {
        self.record_request(failure.request_diagnostics());
        let Some(receipt) = failure.receipt() else {
            return;
        };
        self.failures.record(receipt);
        self.backend_routes.terminal_run_count =
            self.backend_routes.terminal_run_count.saturating_add(1);
    }

    /// 先合并错误所带请求工作；无能力回执时不增加失败或路由次数。
    /// 有回执时路由计为延期，失败仍按回执处置分类；重复调用不会去重。
    pub(crate) fn record_deferred_failure(&mut self, failure: &TextShapingFailure) {
        self.record_request(failure.request_diagnostics());
        let Some(receipt) = failure.receipt() else {
            return;
        };
        self.failures.record(receipt);
        self.backend_routes.deferred_run_count =
            self.backend_routes.deferred_run_count.saturating_add(1);
    }

    /// 按各子报告规则合并计数；不核对来源身份、观测窗口或 Unicode 标识。
    pub(crate) fn merge(&mut self, other: Self) {
        self.failures.merge(other.failures);
        self.backend_routes.merge(other.backend_routes);
        self.shaping_attempt_count = self
            .shaping_attempt_count
            .saturating_add(other.shaping_attempt_count);
        self.font_generation_restart_count = self
            .font_generation_restart_count
            .saturating_add(other.font_generation_restart_count);
        self.font_resolution.merge(other.font_resolution);
    }

    /// 消费一份请求诊断并饱和合并工作量，不额外推导命中、重试或失败次数。
    fn record_request(&mut self, request: TextShapingRequestDiagnostics) {
        self.shaping_attempt_count = self
            .shaping_attempt_count
            .saturating_add(request.shaping_attempt_count);
        self.font_generation_restart_count = self
            .font_generation_restart_count
            .saturating_add(request.font_generation_restart_count);
        self.font_resolution.merge(request.font_resolution);
    }
}

/// 为直接整形消费方转交分类结果；只借用错误，不在这里决定后端切换。
pub(super) fn classify_direct_shape_failure(
    error: &DirectShapeError,
    orientation: TextOrientation,
) -> TextShapingFailureReceipt {
    TextShapingFailureReceipt::from_direct(error, orientation)
}

/// 按失败的字体数据库操作选择诊断阶段，不执行该操作。
const fn backend_font_phase(operation: BackendFontOperation) -> TextShapingFailurePhase {
    match operation {
        BackendFontOperation::ResolveVariations => TextShapingFailurePhase::FontResolution,
        BackendFontOperation::LoadFaceBytes | BackendFontOperation::ResolveFaceIndex => {
            TextShapingFailurePhase::FontLoad
        }
    }
}

/// 将字形不变量类别映射为稳定错误码，不在此重新检查字形。
const fn backend_glyph_code(kind: BackendGlyphInvariantKind) -> TextShapingFailureCode {
    match kind {
        BackendGlyphInvariantKind::EmptyOutput => TextShapingFailureCode::BackendGlyphEmptyOutput,
        BackendGlyphInvariantKind::InvalidClusterOffset => {
            TextShapingFailureCode::BackendGlyphInvalidClusterOffset
        }
        BackendGlyphInvariantKind::NonFiniteMetrics => {
            TextShapingFailureCode::BackendGlyphNonFiniteMetrics
        }
        BackendGlyphInvariantKind::NonMonotonicClusterOrder => {
            TextShapingFailureCode::BackendGlyphNonMonotonicClusterOrder
        }
    }
}

/// 构造 Bidi 终态回执，保留可取得的诊断坐标，不附字体标识。
fn bidi_receipt(source: &BidiInvariantError) -> TextShapingFailureReceipt {
    TextShapingFailureReceipt {
        code: TextShapingFailureCode::BidiInvariant,
        phase: TextShapingFailurePhase::BidiAnalysis,
        source_range: bidi_error_range(source),
        face: None,
        dependency: TextShapingFailureDependency::UnicodeBidiData,
        disposition: TextShapingFailureDisposition::Terminal,
        budget: None,
    }
}

/// 尽量投影错误携带的范围或点；倒置、空范围与缺失值均不构成可安全切片的证明。
fn bidi_error_range(error: &BidiInvariantError) -> Option<TextRange> {
    match *error {
        BidiInvariantError::InvalidResolvedRange { start, end }
        | BidiInvariantError::InvalidLineRange { start, end }
        | BidiInvariantError::LineOutsideParagraph { start, end }
        | BidiInvariantError::GlyphOutsideLine { start, end, .. } => Some(TextRange { start, end }),
        BidiInvariantError::MissingResolvedLevel { offset, .. }
        | BidiInvariantError::MissingResolvedRangeLevel { offset }
        | BidiInvariantError::MissingSignatureScalar { offset } => Some(TextRange {
            start: offset,
            end: offset,
        }),
        BidiInvariantError::NonMonotonicGlyphRange { start, .. } => {
            Some(TextRange { start, end: start })
        }
        BidiInvariantError::ProjectionCardinalityMismatch { .. }
        | BidiInvariantError::AdvanceCardinalityMismatch { .. }
        | BidiInvariantError::MissingLogicalCluster { .. } => None,
    }
}

#[cfg(test)]
#[path = "tests/failure_receipt.rs"]
mod tests;
