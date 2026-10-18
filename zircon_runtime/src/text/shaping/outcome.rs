//! 内部塑形结果与请求级失败信封；由阶段调用方决定发布、重试或 fallback。
//!
//! 值与诊断分开传递，泛型载荷可带借用；参见 [`TextShapingOutcome`]。

use crate::core::framework::text::TextLayoutError;
use crate::text::model::{TextShapingFailureReceipt, TextShapingRequestDiagnostics};
use crate::text::ShapedGlyphRun;
use std::ops::Deref;
use std::sync::Arc;

/// 请求级失败信封，持有通用错误、可选回执和瞬态诊断。
///
/// # 使用与边界
/// 构造方负责保证错误与回执一致；本类型不执行重试、fallback 或缓存发布。
/// 诊断随请求传递，参见 [`TextShapingCompletion`] 与 [`TextShapingOutcome`]。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextShapingFailure {
    /// 通用布局错误；它本身不表达 Deferred/Failed 的处置区分。
    error: TextLayoutError,
    /// 调用方提供的阶段与依赖回执；一般错误允许没有回执。
    receipt: Option<TextShapingFailureReceipt>,
    /// 随当前请求传递的诊断，不属于可缓存的字形载荷。
    request_diagnostics: TextShapingRequestDiagnostics,
}

impl TextShapingFailure {
    /// 消费 error 与 receipt，构造空请求诊断的信封；不校验二者是否匹配。
    pub(crate) const fn with_receipt(
        error: TextLayoutError,
        receipt: TextShapingFailureReceipt,
    ) -> Self {
        Self {
            error,
            receipt: Some(receipt),
            request_diagnostics: TextShapingRequestDiagnostics::EMPTY,
        }
    }

    /// 保留调用者给定的回执有无状态，诊断初始化为空；处置由外层结果决定。
    pub(crate) const fn with_optional_receipt(
        error: TextLayoutError,
        receipt: Option<TextShapingFailureReceipt>,
    ) -> Self {
        Self {
            error,
            receipt,
            request_diagnostics: TextShapingRequestDiagnostics::EMPTY,
        }
    }

    /// 借用通用错误，引用有效期受当前信封借用约束。
    pub(crate) const fn error(&self) -> &TextLayoutError {
        &self.error
    }

    /// 复制可选回执，不补齐或生成新回执。
    pub(crate) const fn receipt(&self) -> Option<TextShapingFailureReceipt> {
        self.receipt
    }

    /// 复制当前请求诊断快照。
    pub(crate) const fn request_diagnostics(&self) -> TextShapingRequestDiagnostics {
        self.request_diagnostics
    }

    /// 消费信封，按诊断类型的合并规则追加 diagnostics，保留错误与回执。
    pub(crate) fn with_request_diagnostics(
        mut self,
        diagnostics: TextShapingRequestDiagnostics,
    ) -> Self {
        self.request_diagnostics.merge(diagnostics);
        self
    }

    /// 消费信封，以调用方已聚合的 diagnostics 替换原值，避免重复累加。
    pub(crate) fn replace_request_diagnostics(
        mut self,
        diagnostics: TextShapingRequestDiagnostics,
    ) -> Self {
        self.request_diagnostics = diagnostics;
        self
    }

    /// 消费信封，仅返回布局错误；回执与请求诊断随信封丢弃。
    pub(crate) fn into_error(self) -> TextLayoutError {
        self.error
    }

    // 仅为 generation 变化且无回执的失败补标准回执；已有回执与请求诊断保留。
    fn ensure_generation_receipt(self) -> Self {
        if self.error == TextLayoutError::FontGenerationChanged && self.receipt.is_none() {
            return Self::font_generation_changed()
                .with_request_diagnostics(self.request_diagnostics);
        }
        self
    }
}

/// 将通用布局错误装入失败信封，供后续结果转换入口分类。
impl From<TextLayoutError> for TextShapingFailure {
    /// 消费 error，初始化为空诊断与无回执；此处不补字体 generation 回执。
    fn from(error: TextLayoutError) -> Self {
        Self {
            error,
            receipt: None,
            request_diagnostics: TextShapingRequestDiagnostics::EMPTY,
        }
    }
}

/// 阶段值与请求诊断的并列载体，供上层分别处理。
///
/// `T` 可以包含借用，生命周期约束由其类型携带；构造信封不会使借用变成独立数据。
/// 参见 [`Self::into_parts`]，缓存发布仍由调用方负责。
/// A publishable value plus request-owned diagnostics that must not enter the artifact cache.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextShapingCompletion<T> {
    /// 阶段产出的值，其内部的资源与借用由 T 决定。
    value: T,
    /// 与值并列的请求统计，不属于缓存 artifact。
    diagnostics: TextShapingRequestDiagnostics,
}

impl<T> TextShapingCompletion<T> {
    /// 直接保存 value 与 diagnostics，不合并诊断或改变 T 的借用约束。
    pub(crate) const fn new(value: T, diagnostics: TextShapingRequestDiagnostics) -> Self {
        Self { value, diagnostics }
    }

    /// 消费信封，按原字段返回值与诊断；不克隆 value。
    pub(crate) fn into_parts(self) -> (T, TextShapingRequestDiagnostics) {
        (self.value, self.diagnostics)
    }
}

/// 以共享借用投影通用错误，不改变附属回执或诊断。
impl Deref for TextShapingFailure {
    type Target = TextLayoutError;

    /// 返回与 self 同借用期限的布局错误引用。
    fn deref(&self) -> &Self::Target {
        self.error()
    }
}

/// 阶段间传递的三态结果，保留失败处置直到上层选择策略。
///
/// # 使用与生命周期
/// `T` 可以包含借用；本层移动载荷和失败信封，不调度重试或执行发布。
/// 显式构造方法保留调用者的处置选择；两个 Result 转换入口才按错误自动分类。
/// [`Self::into_result`] 会丢失失败处置区分、回执及请求诊断。
///
/// Typed internal handoff from a fallible text stage to its owner.
///
/// Only `Ready` contains a publishable value. Error outcomes retain their disposition until the
/// sole publication owner applies an explicit fallback policy; lower stages must not manufacture
/// zero geometry or empty glyph runs.
#[derive(Clone, Debug, PartialEq)]
/// 由 Result 转换时，字体 generation 变化保留为 Deferred；发布者据此区分重试与失败策略。
pub(crate) enum TextShapingOutcome<T = Arc<ShapedGlyphRun>> {
    /// 可交给发布拥有者的阶段值；本枚举不执行实际发布。
    Ready(T),
    /// 交给上层重试决策的失败信封；本枚举不调度重试。
    Deferred(TextShapingFailure),
    /// 交给上层失败或 fallback 策略的失败信封。
    Failed(TextShapingFailure),
}

impl<T> TextShapingOutcome<T> {
    /// 显式选择 Deferred；仅字体 generation 变化会补标准回执，其余错误也保持此处置。
    pub(crate) fn deferred(error: TextLayoutError) -> Self {
        let failure = TextShapingFailure::from(error).ensure_generation_receipt();
        Self::Deferred(failure)
    }

    /// 显式选择 Failed 并建立无回执信封；字体 generation 变化也不改分类。
    pub(crate) fn failed(error: TextLayoutError) -> Self {
        Self::Failed(error.into())
    }

    /// 显式选择 Failed 并保留 error/receipt，不验证回执与分类是否相符。
    pub(crate) fn failed_with_receipt(
        error: TextLayoutError,
        receipt: TextShapingFailureReceipt,
    ) -> Self {
        Self::Failed(TextShapingFailure::with_receipt(error, receipt))
    }

    /// 消费通用 Result：成功为 Ready，generation 变化为 Deferred，其余错误为 Failed。
    /// generation 分支补标准回执；此处不执行实际重试。
    pub(crate) fn from_result(result: Result<T, TextLayoutError>) -> Self {
        match result {
            Ok(value) => Self::Ready(value),
            Err(error @ TextLayoutError::FontGenerationChanged) => Self::deferred(error),
            Err(error) => Self::failed(error),
        }
    }

    /// 消费带信封的 Result 并按 error 分类，保留已有回执与请求诊断。
    /// generation 分支仅在回执缺失时补齐，其他失败不补造回执。
    pub(crate) fn from_shape_result(result: Result<T, TextShapingFailure>) -> Self {
        match result {
            Ok(value) => Self::Ready(value),
            Err(failure) if failure.error() == &TextLayoutError::FontGenerationChanged => {
                Self::Deferred(failure.ensure_generation_receipt())
            }
            Err(failure) => Self::Failed(failure),
        }
    }

    /// 复制错误分支的可选回执；Ready 返回 None，不改变结果。
    pub(crate) const fn failure_receipt(&self) -> Option<TextShapingFailureReceipt> {
        match self {
            Self::Deferred(failure) | Self::Failed(failure) => failure.receipt(),
            Self::Ready(_) => None,
        }
    }

    /// 消费当前结果，仅对 Ready 的值调用一次 map；错误信封原样转移。
    /// U 可继续携带借用，其生命周期由输入与闭包返回类型约束。
    pub(crate) fn map<U>(self, map: impl FnOnce(T) -> U) -> TextShapingOutcome<U> {
        match self {
            Self::Ready(value) => TextShapingOutcome::Ready(map(value)),
            Self::Deferred(error) => TextShapingOutcome::Deferred(error),
            Self::Failed(error) => TextShapingOutcome::Failed(error),
        }
    }

    /// 消费当前结果，仅 Ready 执行后续阶段，直接采用闭包返回的任一处置。
    /// 其余分支不执行闭包，保留原失败信封；不重新分类错误。
    pub(crate) fn and_then<U>(
        self,
        map: impl FnOnce(T) -> TextShapingOutcome<U>,
    ) -> TextShapingOutcome<U> {
        match self {
            Self::Ready(value) => map(value),
            Self::Deferred(error) => TextShapingOutcome::Deferred(error),
            Self::Failed(error) => TextShapingOutcome::Failed(error),
        }
    }

    /// 消费结果，保留 Ready 值，或只取失败信封的布局错误。
    /// 此边界丢失 Deferred/Failed 区分、回执和请求诊断；需要时应先读取这些信息。
    pub(crate) fn into_result(self) -> Result<T, TextLayoutError> {
        match self {
            Self::Ready(value) => Ok(value),
            Self::Deferred(failure) | Self::Failed(failure) => Err(failure.into_error()),
        }
    }
}

/// 与塑形结果相同的泛型别名，供测量、断行与发布共享处置，没有独立策略。
/// Generic name used by measure, breaking, and publication owners during the M2b hard cut.
pub(crate) type TextLayoutOutcome<T> = TextShapingOutcome<T>;

#[cfg(test)]
#[path = "tests/outcome.rs"]
mod tests;
