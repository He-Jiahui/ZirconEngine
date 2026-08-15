//! Shared runtime text cache owners.

mod frame_dedup;
mod hard_line_index;
mod index;
mod layout_cache;
mod measure_cache;
mod rich_cache;
mod shaped_cache;

#[cfg(test)]
mod tests;

pub(crate) use frame_dedup::{TextFrameDedup, TextFrameDedupReport};
pub(crate) use hard_line_index::{
    HardLineIndexCache, HardLineIndexCacheReport, TextDocumentKey,
    DEFAULT_HARD_LINE_INDEX_CACHE_CAPACITY, DEFAULT_HARD_LINE_INDEX_CACHE_MAX_BYTES,
};
pub(crate) use index::{IndexedTextCache, IndexedTextCacheEntry};
pub(crate) use layout_cache::{
    TextLayoutCache, TextLayoutCacheReport, TextLayoutWidthValidity,
    DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY,
};
pub(crate) use measure_cache::{
    TextMeasureCache, TextMeasureCacheReport, DEFAULT_TEXT_MEASURE_CACHE_CAPACITY,
};
pub(crate) use rich_cache::{cached_compiled_rich_text, lookup_cached_compiled_rich_text};
pub use rich_cache::{
    shared_compiled_rich_text_cache_report, CompiledRichTextCacheFrameSampler,
    CompiledRichTextCacheReport,
};
pub(crate) use shaped_cache::{
    ShapedRunCache, ShapedRunCacheKey, ShapedRunCacheLookupKey, ShapedRunCacheReport,
    DEFAULT_SHAPED_RUN_CACHE_CAPACITY, DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
};
