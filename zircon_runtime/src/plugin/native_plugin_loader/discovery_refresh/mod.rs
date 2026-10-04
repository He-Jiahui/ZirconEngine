//! 把发现权威的文件收集与消费者查询隔离：收集经由受预算约束的 I/O 票据，查询只投影不可变快照。
//! 生产收集能力由发现权威私有凭据创建，测试替身仅在测试配置下开放。
use std::path::Path;

mod contract;
mod manifest_index;
mod metrics;
mod service;
mod ticket;
mod work;

pub use contract::{
    NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshBudgetKind,
    NativePluginDiscoveryRefreshError, NativePluginDiscoveryRoot, NativePluginDiscoverySnapshot,
};
pub(crate) use contract::{NativePluginDiscoveryRefreshBudget, NativePluginDiscoveryRefreshInput};
pub(crate) use contract::{
    NativePluginDiscoveryRefreshCandidateReservation,
    NativePluginDiscoveryRefreshDiagnosticReservation, NativePluginDiscoveryRefreshReadReservation,
    NativePluginDiscoveryRefreshRequest, NativePluginDiscoveryRefreshScratchReservation,
    NativePluginDiscoveryRefreshSink,
};
pub(crate) use service::NativePluginDiscoveryRefreshService;
pub use ticket::{NativePluginDiscoveryRefreshTerminal, NativePluginDiscoveryRefreshTicket};
pub(super) use work::{NativePluginDiscoveryManifestAction, NativePluginDiscoveryRefreshWork};

#[cfg(test)]
pub(crate) use metrics::NativePluginDiscoveryRefreshMetrics;

pub(super) fn native_plugin_discovery_refresh_service(
    capability: super::discover::authority::NativePluginDiscoveryAuthorityCapability,
    budget: NativePluginDiscoveryRefreshBudget,
) -> NativePluginDiscoveryRefreshService {
    NativePluginDiscoveryRefreshService::native_plugin_authority(capability, budget)
}

/// 准备阶段建立词法路径与规范路径的稳定身份；失败时保留词法路径以便报告缺失根。
/// 此步骤可能访问文件系统，交互式提交应复用已经解析的根身份。
pub(super) fn native_plugin_discovery_root(path: &Path) -> NativePluginDiscoveryRoot {
    let lexical = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current| current.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };
    // This is the authority's one root-identity stat, not discovery traversal. It prevents path
    // aliases from bypassing same-root coalescing or consuming separate root admissions.
    let canonical = lexical.canonicalize().unwrap_or(lexical);
    NativePluginDiscoveryRoot::from_canonical_path(canonical)
}

pub(super) fn is_native_plugin_discovery_io_lane() -> bool {
    service::is_native_plugin_discovery_io_lane()
}

#[cfg(test)]
/// 为纯状态机测试构造虚拟根；这些路径不证明真实文件系统的规范化或包含关系。
pub(crate) fn test_native_plugin_discovery_root(
    path: impl Into<std::path::PathBuf>,
) -> NativePluginDiscoveryRoot {
    NativePluginDiscoveryRoot::from_canonical_path(path)
}

#[cfg(test)]
/// 文件预算测试复用生产收集器，但使用独立服务状态与测试预算，避免污染进程级发现权威。
pub(crate) fn test_native_plugin_discovery_refresh_service(
    budget: NativePluginDiscoveryRefreshBudget,
) -> NativePluginDiscoveryRefreshService {
    NativePluginDiscoveryRefreshService::native_plugin_authority(
        super::discover::authority::NativePluginDiscoveryAuthorityCapability::for_test(),
        budget,
    )
}

#[cfg(test)]
pub(crate) use service::NativePluginDiscoveryTestCollector;

#[cfg(test)]
mod tests;
