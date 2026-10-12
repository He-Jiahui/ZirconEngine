#[path = "cache.rs"]
mod cache;
#[path = "variant.rs"]
mod variant;

pub(in crate::graphics) use cache::{
    RealtimeIblCompiledGraphCache, RealtimeIblCompiledGraphCacheStats,
};
