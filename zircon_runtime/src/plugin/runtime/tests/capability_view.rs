use super::CapabilityView;
use crate::plugin::{CapabilityStatus, CapabilityStatusManifest, PluginPackageManifest};

#[test]
fn disjoint_indexes_preserve_status_only_capability_queries() {
    let manifest = PluginPackageManifest::new("fixture", "Fixture")
        .with_capability("runtime.capability.declared")
        .with_capability_status(CapabilityStatusManifest::new(
            "runtime.capability.declared",
            CapabilityStatus::Complete,
        ))
        .with_capability_status(CapabilityStatusManifest::new(
            "runtime.capability.status_only",
            CapabilityStatus::Partial,
        ));
    let mut view = CapabilityView::default();

    view.extend_package_manifest(&manifest);

    assert!(view.provided.is_empty());
    assert_eq!(view.statuses.len(), 2);
    assert!(view.has("runtime.capability.declared"));
    assert!(view.has("runtime.capability.status_only"));
    assert_eq!(
        view.status("runtime.capability.declared"),
        Some(CapabilityStatus::Complete)
    );
    assert_eq!(
        view.status("runtime.capability.status_only"),
        Some(CapabilityStatus::Partial)
    );
}
