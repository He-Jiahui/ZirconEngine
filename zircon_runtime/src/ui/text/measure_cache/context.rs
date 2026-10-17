//! 缓存构造、上下文准入及身份读取入口。
//! 显式字体集合构造不附加会话租约；上下文构造先取得会话，克隆共享其租约家族。

use std::sync::Arc;

use crate::text::font::{shared_font_collection_service, FontCollectionService};
use crate::text::{
    SharedTextLayoutSession, TextRuntimeContext, TextRuntimeContextAccessError,
    TextRuntimeContextId, TextSessionId,
};

use super::{
    RetainedPlainTextDocumentCache, TextFrameDedup, TextLayoutCache, TextMeasureCache,
    TextParallelShapeBatchReport, UiTextMeasureCache, DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY,
    DEFAULT_TEXT_MEASURE_CACHE_CAPACITY,
};

impl Default for UiTextMeasureCache {
    /// 默认入口使用进程字体集合，返回无 Runtime 上下文身份的独立缓存。
    /// Creates a standalone cache backed by the process-owner font collection.
    /// Runtime Core-owned surfaces use `new_with_text_context` instead.
    fn default() -> Self {
        Self::new_with_font_collection(shared_font_collection_service())
    }
}

impl UiTextMeasureCache {
    /// 以显式字体集合构造独立缓存。
    ///
    /// # 参数与返回
    /// 接收并持有 `font_collection` 的 Arc，返回不附加 Runtime 上下文租约的缓存。
    /// 集合可后续发布新代次，结果键使用会话查询到的代次。
    pub(crate) fn new_with_font_collection(font_collection: Arc<FontCollectionService>) -> Self {
        Self::from_layout_session(
            None,
            SharedTextLayoutSession::new_with_font_collection(font_collection),
        )
    }

    /// 为 Runtime 拥有的 Surface 取得关联上下文的缓存。
    ///
    /// # 参数、返回与错误
    /// 借用 `context` 申请会话；不接受新会话时原样返回准入错误。
    /// 成功返回拥有会话租约的缓存，上下文本身不借入返回值。
    ///
    /// # 生命周期
    /// 克隆共享租约家族，最后一份释放时归还；后续测量不逐次检查 Active。
    pub(crate) fn new_with_text_context(
        context: &TextRuntimeContext,
    ) -> Result<Self, TextRuntimeContextAccessError> {
        context
            .create_layout_session()
            .map(|session| Self::from_layout_session(Some(context.id()), session))
    }

    /// 取得已创建会话的拥有权并初始化缓存和报告，不再次执行上下文准入。
    fn from_layout_session(
        text_runtime_context_id: Option<TextRuntimeContextId>,
        text_layout_session: SharedTextLayoutSession,
    ) -> Self {
        Self {
            text_runtime_context_id,
            measure_frame_dedup: TextFrameDedup::default(),
            measure_cache: TextMeasureCache::with_capacity(DEFAULT_TEXT_MEASURE_CACHE_CAPACITY),
            text_layout_session,
            layout_frame_dedup: TextFrameDedup::default(),
            layout_cache: TextLayoutCache::with_capacity(DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY),
            retained_plain_documents: RetainedPlainTextDocumentCache::default(),
            uncached_document_resolve_count: 0,
            shape_prewarm_report: TextParallelShapeBatchReport::default(),
            frame_index: 0,
        }
    }

    /// 返回构造时记录的上下文身份；独立字体集合入口返回 None。
    pub(crate) const fn text_runtime_context_id(&self) -> Option<TextRuntimeContextId> {
        self.text_runtime_context_id
    }

    /// 返回会话的租约身份；克隆保持同一家族，独立会话可能为 None。
    pub(crate) fn text_session_id(&self) -> Option<TextSessionId> {
        self.text_layout_session.text_session_id()
    }
}

#[cfg(test)]
#[path = "tests/context.rs"]
mod tests;
