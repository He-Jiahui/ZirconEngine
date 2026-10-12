//! 编译结果的复用边界：内存缓存维护失效快照，磁盘存储按完整键隔离版本；二者都不替宿主解析导入。
#[path = "cache_key.rs"]
mod cache_key;
#[path = "compile_cache.rs"]
mod compile_cache;
#[path = "outcome.rs"]
mod outcome;
#[path = "persistent.rs"]
mod persistent;

pub use cache_key::compile_cache_key_from_compiler;
pub use compile_cache::{UiAssetCompileCache, UiAssetCompileCacheEvictionReport};
pub use outcome::UiCompileCacheOutcome;
pub use persistent::{
    UiCompiledArtifactKey, UiCompiledArtifactStore, UiCompiledArtifactStoreEvictionReport,
};
