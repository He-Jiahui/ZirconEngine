//! Shared runtime text cache owners.

mod frame_dedup;
mod index;
mod layout_cache;
mod measure_cache;
mod shaped_cache;

#[cfg(test)]
mod tests;

pub(crate) use frame_dedup::{TextFrameDedup, TextFrameDedupReport};
pub(crate) use layout_cache::{
    DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY, TextLayoutCache, TextLayoutCacheReport,
    TextLayoutWidthValidity,
};
pub(crate) use measure_cache::{
    DEFAULT_TEXT_MEASURE_CACHE_CAPACITY, TextMeasureCache, TextMeasureCacheReport,
};
pub(crate) use shaped_cache::{
    DEFAULT_SHAPED_RUN_CACHE_CAPACITY, DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES, ShapedRunCache,
    ShapedRunCacheKey, ShapedRunCacheLookupKey, ShapedRunCacheReport,
};
