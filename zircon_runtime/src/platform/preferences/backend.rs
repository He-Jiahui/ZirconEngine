use std::io::Read;
use std::time::Duration;

use crate::core::framework::platform::{
    PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError,
};

use super::PreferenceBackendWorkAuthority;

/// 后端累计的路径缓存与持久化阶段统计；适配器读取快照，不重置后端计数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreferenceStorageBackendDiagnostics {
    pub path_build_wall: Duration,
    pub path_cache_hits: u64,
    pub path_cache_misses: u64,
    pub path_builds: u64,
    pub path_cache_evictions: u64,
    pub path_cache_entries: u64,
    pub staged_write_wall: Duration,
    pub fsync_wall: Duration,
    pub reads: u64,
    pub writes: u64,
    pub removes: u64,
    pub flushes: u64,
}

/// Host-owned persistence implementation. Primitive access requires worker authority.
/// 宿主在驱动安装阶段提供实现；所有原始读写都由持有 worker capability 的持久化任务调用。
pub trait PreferenceStorageBackend: Send + Sync + 'static {
    fn backend_kind(&self) -> PreferenceStorageBackendKind;

    /// 打开一个键的读取流；`None` 表示宿主没有持久化值，读取上限由 worker 侧执行。
    fn open_read(
        &self,
        authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<Option<Box<dyn Read + Send>>, PreferenceStorageError>;

    /// 将完整值交给宿主后端写入；调用者已在适配器层完成大小配额与排队。
    fn write(
        &self,
        authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
        value: &[u8],
    ) -> Result<(), PreferenceStorageError>;

    fn remove(
        &self,
        authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<(), PreferenceStorageError>;

    /// 在全局 fence 到达时同步后端可见的先前写入。
    fn flush(
        &self,
        authority: &PreferenceBackendWorkAuthority,
    ) -> Result<(), PreferenceStorageError>;

    fn diagnostics(&self) -> PreferenceStorageBackendDiagnostics {
        PreferenceStorageBackendDiagnostics::default()
    }
}
