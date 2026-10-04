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
    /// Creates a standalone cache backed by the process-owner font collection.
    /// Runtime Core-owned surfaces use `new_with_text_context` instead.
    fn default() -> Self {
        Self::new_with_font_collection(shared_font_collection_service())
    }
}

impl UiTextMeasureCache {
    pub(crate) fn new_with_font_collection(font_collection: Arc<FontCollectionService>) -> Self {
        Self::from_layout_session(
            None,
            SharedTextLayoutSession::new_with_font_collection(font_collection),
        )
    }

    pub(crate) fn new_with_text_context(
        context: &TextRuntimeContext,
    ) -> Result<Self, TextRuntimeContextAccessError> {
        context
            .create_layout_session()
            .map(|session| Self::from_layout_session(Some(context.id()), session))
    }

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

    pub(crate) const fn text_runtime_context_id(&self) -> Option<TextRuntimeContextId> {
        self.text_runtime_context_id
    }

    pub(crate) fn text_session_id(&self) -> Option<TextSessionId> {
        self.text_layout_session.text_session_id()
    }
}

#[cfg(test)]
#[path = "tests/context.rs"]
mod tests;
