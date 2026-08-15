use std::path::Path;

mod contract;
mod manifest_index;
mod metrics;
mod service;
mod ticket;
mod work;

pub(crate) use contract::{
    NativePluginDiscoveryInputIdentity, NativePluginDiscoveryRefreshBudget,
    NativePluginDiscoveryRefreshBudgetKind, NativePluginDiscoveryRefreshError,
    NativePluginDiscoveryRefreshInput, NativePluginDiscoveryRoot, NativePluginDiscoverySnapshot,
};
pub(crate) use contract::{
    NativePluginDiscoveryRefreshCandidateReservation,
    NativePluginDiscoveryRefreshDiagnosticReservation, NativePluginDiscoveryRefreshReadReservation,
    NativePluginDiscoveryRefreshRequest, NativePluginDiscoveryRefreshScratchReservation,
    NativePluginDiscoveryRefreshSink,
};
pub(crate) use service::NativePluginDiscoveryRefreshService;
pub(crate) use ticket::{NativePluginDiscoveryRefreshTerminal, NativePluginDiscoveryRefreshTicket};
pub(super) use work::{NativePluginDiscoveryManifestAction, NativePluginDiscoveryRefreshWork};

#[cfg(test)]
pub(crate) use metrics::NativePluginDiscoveryRefreshMetrics;

pub(super) fn native_plugin_discovery_refresh_service(
    capability: super::discover::authority::NativePluginDiscoveryAuthorityCapability,
    budget: NativePluginDiscoveryRefreshBudget,
) -> NativePluginDiscoveryRefreshService {
    NativePluginDiscoveryRefreshService::native_plugin_authority(capability, budget)
}

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
pub(crate) fn test_native_plugin_discovery_root(
    path: impl Into<std::path::PathBuf>,
) -> NativePluginDiscoveryRoot {
    NativePluginDiscoveryRoot::from_canonical_path(path)
}

#[cfg(test)]
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
