use std::sync::Arc;

use zircon_app::{PluginGroup, PluginGroupBuilder, PluginGroupError};
use zircon_runtime::core::{ModuleDependencySpec, ModuleDescriptor};
use zircon_runtime::engine_module::EngineModule;

#[derive(Debug)]
struct DependentModule;

impl EngineModule for DependentModule {
    fn module_name(&self) -> &'static str {
        "DependentModule"
    }

    fn module_description(&self) -> &'static str {
        "depends on a module omitted by the nested group"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(self.module_name(), self.module_description())
            .with_module_dependency(ModuleDependencySpec::named("MissingModule"))
    }
}

struct InvalidNestedGroup;

impl PluginGroup for InvalidNestedGroup {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError> {
        PluginGroupBuilder::start("InvalidNestedGroup").add_module(Arc::new(DependentModule))
    }
}

#[test]
fn add_group_propagates_nested_module_order_errors() {
    let error = PluginGroupBuilder::start("OuterGroup")
        .add_group(InvalidNestedGroup)
        .expect_err("invalid nested module order must remain a typed builder error");

    let PluginGroupError::ModuleOrder { group, reason } = error else {
        panic!("expected nested module-order error");
    };
    assert_eq!(group, "InvalidNestedGroup");
    assert!(reason.contains("DependentModule"));
    assert!(reason.contains("MissingModule"));
}
