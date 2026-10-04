//! 插件管理与导出流程的发现入口，共享同一权威与最后成功发布的候选集合。
//! 需要交互式刷新时先在准备阶段取得根身份，再提交票据和查询快照；同步入口允许冷启动等待。
use std::path::Path;

use self::authority::discovery_authority;
use super::{
    NativePluginDiscoveryRefreshTicket, NativePluginDiscoveryRoot, NativePluginDiscoverySnapshot,
    NativePluginLoadReport, NativePluginLoader,
};

pub(super) mod authority;

#[cfg(test)]
#[path = "discover/tests/cases.rs"]
mod tests;

impl NativePluginLoader {
    /// Resolves and interns a canonical discovery root for later nonblocking requests.
    ///
    /// Root resolution may query the filesystem and belongs in project-open or other admitted
    /// setup work, not in an interactive UI request handler.
    pub fn resolve_discovery_root(&self, root: impl AsRef<Path>) -> NativePluginDiscoveryRoot {
        discovery_authority().resolve_root(root.as_ref())
    }

    /// Requests a bounded newest-generation refresh without waiting for collector I/O.
    pub fn request_discovery_refresh(
        &self,
        root: &NativePluginDiscoveryRoot,
    ) -> NativePluginDiscoveryRefreshTicket {
        discovery_authority().request_refresh(root)
    }

    /// Returns the immutable last-good root publication without filesystem or collector I/O.
    pub fn latest_discovery_snapshot(
        &self,
        root: &NativePluginDiscoveryRoot,
    ) -> Option<std::sync::Arc<NativePluginDiscoverySnapshot>> {
        discovery_authority().latest_snapshot(root)
    }

    /// Projects the canonical authority's last-good snapshot. A cold root waits for the single
    /// bounded authority refresh; later calls perform no synchronous filesystem traversal.
    /// 发现报告只描述候选与诊断；库加载、制品信任校验及注册属于后续加载入口。
    /// 已有快照不会自行感知磁盘变化，变更来源必须显式请求刷新或提交清单通知。
    pub fn discover(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        discovery_authority().discover(root.as_ref())
    }

    /// Schedules a coalesced, path-scoped manifest refresh after a watcher/editor notification.
    /// 此返回报告的入口会等待通知票据完成，不能当作交互回调中的无等待提交函数。
    /// 通知路径相对于发现根解释；无法归入根内的通知会退回受预算约束的全量扫描。
    pub fn refresh_discovery_manifest(
        &self,
        root: impl AsRef<Path>,
        manifest_path: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        discovery_authority().refresh_manifest(root.as_ref(), manifest_path.as_ref())
    }

    /// Removes a watcher-reported path from the immutable discovery index without rescanning.
    /// 删除通知操作的是已发布索引，文件已不存在时仍可提交；相对路径须相对于发现根。
    /// 调用会等待票据；保留的旧快照仍由持有它的消费者独立使用。
    pub fn remove_discovered_path(
        &self,
        root: impl AsRef<Path>,
        removed_path: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        discovery_authority().remove_path(root.as_ref(), removed_path.as_ref())
    }

    /// Returns the last published generation without polling or touching the filesystem.
    pub fn discovery_generation(&self, root: impl AsRef<Path>) -> Option<u64> {
        discovery_authority().generation(root.as_ref())
    }
}
