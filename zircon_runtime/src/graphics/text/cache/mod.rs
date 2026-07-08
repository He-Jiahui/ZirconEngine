//! Shared runtime text cache owners.

mod frame_dedup;
mod layout_cache;
mod measure_cache;
mod shaped_cache;

#[cfg(test)]
mod tests;

pub(crate) use frame_dedup::{TextFrameDedup, TextFrameDedupReport};
pub(crate) use layout_cache::{
    TextLayoutCache, TextLayoutCacheReport, TextLayoutWidthValidity,
    DEFAULT_TEXT_LAYOUT_CACHE_CAPACITY,
};
pub(crate) use measure_cache::{
    TextMeasureCache, TextMeasureCacheReport, DEFAULT_TEXT_MEASURE_CACHE_CAPACITY,
};
pub(crate) use shaped_cache::{
    ShapedRunCache, ShapedRunCacheKey, ShapedRunCacheReport, DEFAULT_SHAPED_RUN_CACHE_CAPACITY,
    DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
};
