//! Shared runtime text cache owners.

#[path = "frame_dedup.rs"]
mod frame_dedup;
#[path = "hard_line_index.rs"]
mod hard_line_index;
#[path = "index.rs"]
mod index;
#[path = "layout_cache.rs"]
mod layout_cache;
#[path = "measure_cache.rs"]
mod measure_cache;
#[path = "rich_cache.rs"]
mod rich_cache;
#[path = "shaped_cache.rs"]
mod shaped_cache;

#[cfg(test)]
#[path = "tests/cases.rs"]
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
pub(crate) use rich_cache::CompiledRichTextCacheOwner;
pub use rich_cache::CompiledRichTextCacheReport;
pub(crate) use shaped_cache::{
    ShapedRunCache, ShapedRunCacheKey, ShapedRunCacheLookupKey, ShapedRunCacheReport,
    DEFAULT_SHAPED_RUN_CACHE_CAPACITY, DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
};
