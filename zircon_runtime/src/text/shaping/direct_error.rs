//! 直接成形错误的阶段信息及横竖排共用的 glyph 前置检查。
//!
//! 错误按值交给失败回执分类器；本模块不执行字体后端调用或回退。
//! 检查只同步借用 glyph 与分段文本；度量字段的选择和 cluster 顺序恢复由调用方负责。

use crate::text::{FontFaceId, TextRange};

use super::backend_error::BackendShapeError;
use super::bidi::BidiInvariantError;
use super::itemize::ItemizationError;

/// 后端输出被拒绝时保留的稳定类别，供 [`DirectShapeError`] 携带并映射为失败回执。
///
/// 前三类由 [`validate_backend_glyphs`] 产生；顺序类别由调用方的独立顺序恢复步骤产生。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::text::shaping) enum BackendGlyphInvariantKind {
    /// 输出中没有可供后续处理的 glyph。
    EmptyOutput,
    /// 至少一个 cluster 起点不是文本内部的 UTF-8 字符边界。
    InvalidClusterOffset,
    /// 调用方提供的有限性判定拒绝了至少一个 glyph。
    NonFiniteMetrics,
    /// 调用方无法恢复所需的 cluster 逻辑顺序；通用检查函数不产生此类错误。
    NonMonotonicClusterOrder,
}

#[derive(Debug, thiserror::Error)]
/// 直接后端错误保留 itemization/bidi、字体加载和 glyph 校验的阶段信息；回退分类器依赖这些变体而非错误字符串。
///
/// # 使用与所有权
/// 这些变体保存诊断阶段与调用方提交的字段，不认证范围、字体及错误之间的对应关系。
/// 错误被按值转交；`Itemization` 的转换来源与 `Backend` 的 `source` 保留错误链，
/// 不能据此推定每个变体都有 `Error::source`，也不持有字体资源租约。
pub(in crate::text::shaping) enum DirectShapeError {
    /// 保存分段阶段的错误；派生的 `From<ItemizationError>` 同时保留其错误来源。
    #[error("text itemization failed: {0}")]
    Itemization(#[from] ItemizationError),
    /// 保存直接转换而来的双向文本不变量错误，分类时仍能区分该阶段。
    #[error("text bidi invariant failed: {0:?}")]
    BidiInvariant(BidiInvariantError),
    /// 保存后端调用失败及其诊断范围；是否尝试替代后端由后续分类器决定。
    #[error("direct shaping backend failed at {range:?}: {source}")]
    Backend {
        /// 调用者提供的诊断范围；构造时不检查它与后端错误是否对应。
        range: TextRange,
        /// 按值保存后端错误；错误链返回的借用受当前错误值的借用期约束。
        #[source]
        source: BackendShapeError,
    },
    /// 保存调用者报告的无效范围；该变体的构造本身不执行文本切片。
    #[error("direct shaping source range is invalid: {range:?}")]
    InvalidSourceRange { range: TextRange },
    /// 保存 glyph 校验失败时的字体标识、分段范围和具体类别。
    #[error("direct backend glyph invariant {kind:?} failed for {face:?} at {range:?}")]
    BackendGlyphInvariant {
        /// 诊断所用的字体标识，不建立字体资源租约。
        face: FontFaceId,
        /// 出错分段的诊断范围，按调用者提供的值保留。
        range: TextRange,
        /// 供回执分类使用的类别；错误封装不会重新执行该检查。
        kind: BackendGlyphInvariantKind,
    },
}

impl DirectShapeError {
    /// 为后端错误附上调用者提供的诊断范围。
    ///
    /// # 参数与返回
    /// `range` 按原值保留，`source` 被移入返回的 [`Self::Backend`] 变体。
    ///
    /// # 约束
    /// 不校验二者的对应关系，不执行后端调用或回退；后续使用见失败回执分类逻辑。
    pub(in crate::text::shaping) const fn backend(
        range: TextRange,
        source: BackendShapeError,
    ) -> Self {
        Self::Backend { range, source }
    }

    /// 封装已由调用者判定的 glyph 校验失败。
    ///
    /// # 参数
    /// - `face`：诊断所用的字体标识。
    /// - `range`：该次失败关联的分段范围。
    /// - `kind`：调用者判定的类别，包括独立顺序恢复步骤产生的类别。
    ///
    /// # 返回与生命周期
    /// 返回 [`Self::BackendGlyphInvariant`]，按值保留三个字段，不保留文本借用或资源租约。
    /// 本函数不复核字段之间的对应关系，也不重新执行校验。
    pub(in crate::text::shaping) const fn backend_glyph_invariant(
        face: FontFaceId,
        range: TextRange,
        kind: BackendGlyphInvariantKind,
    ) -> Self {
        Self::BackendGlyphInvariant { face, range, kind }
    }
}

impl From<BidiInvariantError> for DirectShapeError {
    /// 将 `error` 移入 [`Self::BidiInvariant`]；不增加校验或改变错误类别。
    fn from(error: BidiInvariantError) -> Self {
        Self::BidiInvariant(error)
    }
}

/// 验证后端 glyph 非空、cluster 起点位于文本内且为 UTF-8 字符边界、度量有限；失败保留具体类别供回执分类。
///
/// # 参数
/// - `glyphs`：本次后端输出的借用切片；`T` 不要求实现特定 glyph trait。
/// - `text`：这些 glyph 对应的分段文本，偏移相对于此字符串的 UTF-8 字节起点。
/// - `source_offset`：提供 cluster 起点；必须严格小于 `text.len()` 且落在字符边界。
/// - `metrics_are_finite`：定义本次所需的度量有限性；当前横竖排调用只判定 advance 和两个偏移。
///
/// # 返回
/// 空切片先返回 `EmptyOutput`；随后返回首类无效起点错误，再检查度量并可能返回 `NonFiniteMetrics`。
/// 全部通过才返回 `Ok(())`。两个 `any` 均短路；发现无效起点时不会调用度量判定。
///
/// # 约束与生命周期
/// 不检查 cluster 终点、字素边界、完整源范围或顺序，也不认证所有输出几何字段。
/// 回调按值传入并仅同步调用；返回值不保留 glyph、文本或回调的借用。
/// 不要求回调无副作用，不回滚回调副作用，也不捕获其 panic。
/// 类别的后续用途见 [`BackendGlyphInvariantKind`]；顺序恢复由直接后端调用方另行处理。
pub(in crate::text::shaping) fn validate_backend_glyphs<T>(
    glyphs: &[T],
    text: &str,
    source_offset: impl Fn(&T) -> usize,
    metrics_are_finite: impl Fn(&T) -> bool,
) -> Result<(), BackendGlyphInvariantKind> {
    if glyphs.is_empty() {
        return Err(BackendGlyphInvariantKind::EmptyOutput);
    }
    if glyphs.iter().any(|glyph| {
        let offset = source_offset(glyph);
        offset >= text.len() || !text.is_char_boundary(offset)
    }) {
        return Err(BackendGlyphInvariantKind::InvalidClusterOffset);
    }
    if glyphs.iter().any(|glyph| !metrics_are_finite(glyph)) {
        return Err(BackendGlyphInvariantKind::NonFiniteMetrics);
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/direct_error.rs"]
mod tests;
