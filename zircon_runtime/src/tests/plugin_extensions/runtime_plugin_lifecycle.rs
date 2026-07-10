#[path = "runtime_plugin_lifecycle/lifecycle_fixtures.rs"]
mod lifecycle_fixtures;
use lifecycle_fixtures::*;

#[path = "runtime_plugin_lifecycle/capability_projection.rs"]
mod capability_projection;
#[path = "runtime_plugin_lifecycle/kernel_lifecycle.rs"]
mod kernel_lifecycle;
#[path = "runtime_plugin_lifecycle/native_projection.rs"]
mod native_projection;
#[path = "runtime_plugin_lifecycle/registration_order.rs"]
mod registration_order;

#[test]
fn runtime_plugin_lifecycle_hard_cuts_to_kernel_module_lifecycle() {
    let plugin_trait = include_str!("../../plugin/runtime_plugin/runtime_plugin/plugin.rs");
    let feature_trait = include_str!("../../plugin/runtime_plugin/runtime_plugin/feature.rs");
    let catalog_root = include_str!("../../plugin/runtime_plugin/runtime_plugin_catalog.rs");

    for retired_hook in ["fn ready(", "fn finish(", "fn activate(", "fn deactivate("] {
        assert!(!plugin_trait.contains(retired_hook), "{retired_hook}");
        assert!(!feature_trait.contains(retired_hook), "{retired_hook}");
    }
    assert!(plugin_trait.contains("fn lifecycle(&self) -> &dyn ModuleLifecycle"));
    assert!(!catalog_root.contains("mod lifecycle;"));
}
