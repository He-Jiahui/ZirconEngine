use std::sync::Arc;

use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::engine_module::EngineModule;

use super::{
    DefaultPlugins, HeadlessPlugins, MinimalPlugins, PluginGroup, PluginGroupBuilder,
    PluginGroupError,
};

#[derive(Debug)]
struct TestModule {
    name: &'static str,
    description: &'static str,
}

impl TestModule {
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            description: name,
        }
    }

    const fn described(name: &'static str, description: &'static str) -> Self {
        Self { name, description }
    }
}

impl EngineModule for TestModule {
    fn module_name(&self) -> &'static str {
        self.name
    }

    fn module_description(&self) -> &'static str {
        self.description
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(self.name, self.description)
    }
}

fn module(name: &'static str) -> Arc<dyn EngineModule> {
    Arc::new(TestModule::new(name))
}

fn described_module(name: &'static str, description: &'static str) -> Arc<dyn EngineModule> {
    Arc::new(TestModule::described(name, description))
}

#[test]
fn plugin_group_builder_orders_and_omits_disabled_modules() {
    let group = PluginGroupBuilder::start("TestPlugins")
        .add(module("A"))
        .unwrap()
        .add(module("B"))
        .unwrap()
        .add_after("A", module("C"))
        .unwrap()
        .disable("B")
        .unwrap()
        .finish();

    assert_eq!(group.name(), "TestPlugins");
    assert_eq!(group.module_keys(), vec!["A", "C"]);
}

#[test]
fn plugin_group_builder_set_replaces_without_moving_order() {
    let group = PluginGroupBuilder::start("TestPlugins")
        .add(module("A"))
        .unwrap()
        .add(described_module("B", "old"))
        .unwrap()
        .add(module("C"))
        .unwrap()
        .set(described_module("B", "replacement"))
        .unwrap()
        .finish();
    let descriptors = group.module_descriptors();

    assert_eq!(group.module_keys(), vec!["A", "B", "C"]);
    assert_eq!(descriptors[1].description, "replacement");
}

#[test]
fn plugin_group_builder_reports_duplicate_key_and_missing_anchor() {
    let duplicate = PluginGroupBuilder::start("TestPlugins")
        .add(module("A"))
        .unwrap()
        .add(module("A"))
        .unwrap_err();
    let missing_anchor = PluginGroupBuilder::start("TestPlugins")
        .add(module("A"))
        .unwrap()
        .add_before("Missing", module("B"))
        .unwrap_err();

    assert_eq!(
        duplicate,
        PluginGroupError::DuplicateKey {
            group: "TestPlugins".to_string(),
            key: "A".to_string(),
        }
    );
    assert_eq!(
        missing_anchor,
        PluginGroupError::MissingAnchor {
            group: "TestPlugins".to_string(),
            key: "Missing".to_string(),
        }
    );
}

#[test]
fn plugin_group_builder_reports_missing_keys_for_mutation() {
    let set_error = PluginGroupBuilder::start("TestPlugins")
        .add(module("A"))
        .unwrap()
        .set(module("Missing"))
        .unwrap_err();
    let disable_error = PluginGroupBuilder::start("TestPlugins")
        .disable("Missing")
        .unwrap_err();
    let enable_error = PluginGroupBuilder::start("TestPlugins")
        .enable("Missing")
        .unwrap_err();

    let expected = PluginGroupError::MissingKey {
        group: "TestPlugins".to_string(),
        key: "Missing".to_string(),
    };
    assert_eq!(set_error, expected);
    assert_eq!(disable_error, expected);
    assert_eq!(enable_error, expected);
}

#[test]
fn plugin_group_builder_reports_disabled_anchor_reordering() {
    let error = PluginGroupBuilder::start("TestPlugins")
        .add(module("A"))
        .unwrap()
        .add(module("B"))
        .unwrap()
        .disable("B")
        .unwrap()
        .add_after("B", module("C"))
        .unwrap_err();

    assert_eq!(
        error,
        PluginGroupError::DisabledAnchor {
            group: "TestPlugins".to_string(),
            key: "B".to_string(),
        }
    );
}

#[test]
fn builtin_plugin_groups_resolve_expected_module_sets() {
    let minimal = MinimalPlugins.build().unwrap().finish();
    let default = DefaultPlugins::default().build().unwrap().finish();
    let headless = HeadlessPlugins::default().build().unwrap().finish();

    assert_eq!(
        minimal.module_keys(),
        vec![
            zircon_runtime::foundation::FOUNDATION_MODULE_NAME,
            zircon_runtime::core::modules::TASKS_MODULE_NAME,
            zircon_runtime::core::modules::TIME_MODULE_NAME,
            zircon_runtime::core::modules::FRAME_COUNT_MODULE_NAME,
            zircon_runtime::core::modules::DIAGNOSTICS_CORE_MODULE_NAME,
        ]
    );
    assert!(default
        .module_keys()
        .contains(&zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
    assert!(default
        .module_keys()
        .contains(&zircon_runtime::script::SCRIPT_MODULE_NAME));
    assert!(!headless
        .module_keys()
        .contains(&zircon_runtime::graphics::GRAPHICS_MODULE_NAME));
}
