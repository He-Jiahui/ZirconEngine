use std::sync::Arc;

use super::super::{PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError};
use super::{
    snapshot::{PreferenceEviction, PreferenceReadSnapshot},
    tickets::{PreferenceFlushTicket, PreferenceMutationSubmission},
    work_deadline::PreferenceWorkDeadline,
};

/// 平台管理端提供的首选项契约；宿主后端原语只允许持有工作权限的 IO 执行者调用。
/// 写入先建立可见代际，再由票据报告持久化终态；调用方不能把提交成功当作落盘。
pub trait PreferenceStorage: Send + Sync + 'static {
    fn backend_kind(&self) -> PreferenceStorageBackendKind;

    /// 返回当前可见视图；首次缺失时可异步发起读取，须结合 durability 区分待定与缺失。
    fn snapshot(
        &self,
        key: &PreferenceKey,
    ) -> Result<PreferenceReadSnapshot, PreferenceStorageError>;

    /// 受理写入并返回票据；必要的落盘保证须等待票据或后续 flush fence。
    fn submit_write(
        &self,
        key: PreferenceKey,
        value: Arc<[u8]>,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError>;

    fn submit_remove(
        &self,
        key: PreferenceKey,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError>;

    /// 只约束此前已受理的工作，之后提交的不同键写入不属于这张栅栏。
    fn flush_fence(
        &self,
        deadline: PreferenceWorkDeadline,
    ) -> Result<Arc<dyn PreferenceFlushTicket>, PreferenceStorageError>;

    /// Explicitly discards one terminal, visible-not-durable generation and its failure state.
    ///
    /// Pending and durable generations are not eligible for lossy eviction.
    fn evict(&self, key: &PreferenceKey) -> Option<PreferenceEviction>;
}
