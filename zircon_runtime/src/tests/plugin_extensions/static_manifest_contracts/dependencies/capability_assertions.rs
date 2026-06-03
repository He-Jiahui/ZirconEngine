use std::collections::BTreeSet;
use std::path::Path;

use super::capabilities::StaticPackageCapabilities;

pub(super) fn assert_declared_dependency_capability(
    relative_path: &Path,
    context: &str,
    capability: &str,
    target: &StaticPackageCapabilities,
) {
    assert!(
        target.capabilities.contains(capability),
        "plugin manifest {relative_path:?} {context} capability `{capability}` should be declared by the referenced static plugin package or one of its feature rows"
    );
}

pub(super) fn assert_host_dependency_capability(
    relative_path: &Path,
    context: &str,
    capability: &str,
) {
    assert!(
        capability.starts_with("runtime.module.") || capability.starts_with("runtime.capability."),
        "plugin manifest {relative_path:?} {context} capability `{capability}` references no static plugin package and should use a runtime.module.* or runtime.capability.* host namespace"
    );
}

pub(super) fn assert_declared_or_host_capability(
    relative_path: &Path,
    context: &str,
    capability: &str,
    declared_capabilities: &BTreeSet<String>,
) {
    assert!(
        declared_capabilities.contains(capability) || is_host_required_capability(capability),
        "plugin manifest {relative_path:?} {context} `{capability}` should reference a declared static package/feature capability or an explicitly host-owned capability"
    );
}

fn is_host_required_capability(capability: &str) -> bool {
    capability.starts_with("runtime.capability.") || capability == "runtime.asset.importer.native"
}
