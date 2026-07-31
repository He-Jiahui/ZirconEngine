use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use zircon_runtime::core::{ModuleDescriptor, sort_module_activation_order};
use zircon_runtime::engine_module::EngineModule;

use super::{
    DefaultPlugins, DevPlugins, HeadlessPlugins, MinimalPlugins, PluginGroup, PluginGroupBuilder,
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

#[derive(Debug)]
struct CountingModule {
    name: &'static str,
    descriptor_calls: Arc<AtomicUsize>,
}

impl CountingModule {
    fn new(name: &'static str, descriptor_calls: Arc<AtomicUsize>) -> Self {
        Self {
            name,
            descriptor_calls,
        }
    }
}

impl EngineModule for CountingModule {
    fn module_name(&self) -> &'static str {
        self.name
    }

    fn module_description(&self) -> &'static str {
        "descriptor generation counter"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        let generation = self.descriptor_calls.fetch_add(1, AtomicOrdering::SeqCst) + 1;
        ModuleDescriptor::new(self.name, format!("descriptor generation {generation}"))
    }
}

struct CountingPluginGroup {
    descriptor_calls: Arc<AtomicUsize>,
}

impl PluginGroup for CountingPluginGroup {
    fn build(self) -> Result<PluginGroupBuilder, PluginGroupError> {
        PluginGroupBuilder::start("NestedCountingPlugins").add_module(Arc::new(
            CountingModule::new("NestedCounting", self.descriptor_calls),
        ))
    }
}

fn module(name: &'static str) -> Arc<dyn EngineModule> {
    Arc::new(TestModule::new(name))
}

fn described_module(name: &'static str, description: &'static str) -> Arc<dyn EngineModule> {
    Arc::new(TestModule::described(name, description))
}

#[test]
fn resolved_plugin_group_freezes_each_enabled_module_descriptor_once() {
    let descriptor_calls = Arc::new(AtomicUsize::new(0));
    let group = PluginGroupBuilder::start("CountingPlugins")
        .add_module(Arc::new(CountingModule::new(
            "Counting",
            Arc::clone(&descriptor_calls),
        )))
        .unwrap()
        .finish();

    let first_snapshot = group.module_descriptors();
    let second_snapshot = group.module_descriptors();

    assert_eq!(descriptor_calls.load(AtomicOrdering::SeqCst), 1);
    assert_eq!(first_snapshot[0].description, "descriptor generation 1");
    assert_eq!(
        second_snapshot[0].description,
        first_snapshot[0].description
    );
}

#[test]
fn resolved_plugin_group_does_not_generate_disabled_module_descriptors() {
    let descriptor_calls = Arc::new(AtomicUsize::new(0));
    let group = PluginGroupBuilder::start("CountingPlugins")
        .add_module(Arc::new(CountingModule::new(
            "DisabledCounting",
            Arc::clone(&descriptor_calls),
        )))
        .unwrap()
        .disable("DisabledCounting")
        .unwrap()
        .finish();

    assert!(group.module_descriptors().is_empty());
    assert_eq!(descriptor_calls.load(AtomicOrdering::SeqCst), 0);
}

#[test]
fn resolved_plugin_group_preserves_the_nested_descriptor_snapshot() {
    let descriptor_calls = Arc::new(AtomicUsize::new(0));
    let group = PluginGroupBuilder::start("OuterCountingPlugins")
        .add_group(CountingPluginGroup {
            descriptor_calls: Arc::clone(&descriptor_calls),
        })
        .unwrap()
        .finish();

    assert_eq!(descriptor_calls.load(AtomicOrdering::SeqCst), 1);
    assert_eq!(
        group.module_descriptors()[0].description,
        "descriptor generation 1"
    );
}

#[test]
fn resolved_plugin_group_does_not_regenerate_a_nested_descriptor_when_disabled() {
    let descriptor_calls = Arc::new(AtomicUsize::new(0));
    let group = PluginGroupBuilder::start("OuterCountingPlugins")
        .add_group(CountingPluginGroup {
            descriptor_calls: Arc::clone(&descriptor_calls),
        })
        .unwrap()
        .disable("NestedCounting")
        .unwrap()
        .finish();

    assert!(group.module_descriptors().is_empty());
    assert_eq!(
        descriptor_calls.load(AtomicOrdering::SeqCst),
        1,
        "nested validation owns generation 1; the disabled outer generation must not regenerate"
    );
}

#[test]
fn plugin_group_builder_orders_and_omits_disabled_modules() {
    let group = PluginGroupBuilder::start("TestPlugins")
        .add_module(module("A"))
        .unwrap()
        .add_module(module("B"))
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
        .add_module(module("A"))
        .unwrap()
        .add_module(described_module("B", "old"))
        .unwrap()
        .add_module(module("C"))
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
        .add_module(module("A"))
        .unwrap()
        .add_module(module("A"))
        .unwrap_err();
    let missing_anchor = PluginGroupBuilder::start("TestPlugins")
        .add_module(module("A"))
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
        .add_module(module("A"))
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
        .add_module(module("A"))
        .unwrap()
        .add_module(module("B"))
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
    let default = DefaultPlugins.build().unwrap().finish();
    let dev = DevPlugins.build().unwrap().finish();
    let headless = HeadlessPlugins.build().unwrap().finish();

    assert_eq!(
        minimal.module_keys(),
        vec![
            zircon_runtime::foundation::FOUNDATION_MODULE_NAME,
            zircon_runtime::core::runtime::modules::TASKS_MODULE_NAME,
            zircon_runtime::core::runtime::modules::TIME_MODULE_NAME,
            zircon_runtime::core::runtime::modules::FRAME_COUNT_MODULE_NAME,
            zircon_runtime::core::runtime::modules::DIAGNOSTICS_CORE_MODULE_NAME,
        ]
    );
    assert!(
        default
            .module_keys()
            .contains(&zircon_runtime::platform::PLATFORM_MODULE_NAME)
    );
    assert!(
        default
            .module_keys()
            .contains(&zircon_runtime::input::INPUT_MODULE_NAME)
    );
    assert!(
        dev.module_keys()
            .contains(&zircon_runtime::platform::PLATFORM_MODULE_NAME)
    );
    assert!(
        dev.module_keys()
            .contains(&zircon_runtime::input::INPUT_MODULE_NAME)
    );
    assert!(
        headless
            .module_keys()
            .contains(&zircon_runtime::platform::PLATFORM_MODULE_NAME)
    );
    assert!(
        headless
            .module_keys()
            .contains(&zircon_runtime::input::INPUT_MODULE_NAME)
    );
    assert!(
        !minimal
            .module_keys()
            .contains(&zircon_runtime::platform::PLATFORM_MODULE_NAME)
    );
    assert!(
        !minimal
            .module_keys()
            .contains(&zircon_runtime::input::INPUT_MODULE_NAME)
    );
    assert!(
        default
            .module_keys()
            .contains(&zircon_runtime::core::runtime::modules::LOG_MODULE_NAME)
    );
    assert!(
        !default
            .module_keys()
            .contains(&zircon_runtime::core::runtime::modules::LOG_DIAGNOSTICS_MODULE_NAME)
    );
    assert!(
        dev.module_keys()
            .contains(&zircon_runtime::core::runtime::modules::LOG_DIAGNOSTICS_MODULE_NAME)
    );
    assert!(
        headless
            .module_keys()
            .contains(&zircon_runtime::core::runtime::modules::LOG_MODULE_NAME)
    );
    assert!(
        default
            .module_keys()
            .contains(&zircon_runtime::core::framework::render::GRAPHICS_MODULE_NAME)
    );
    assert!(
        default
            .module_keys()
            .contains(&zircon_runtime::script::SCRIPT_MODULE_NAME)
    );
    assert!(
        !headless
            .module_keys()
            .contains(&zircon_runtime::core::framework::render::GRAPHICS_MODULE_NAME)
    );
}

#[test]
fn builtin_plugin_groups_finish_in_descriptor_activation_order() {
    for group in [
        MinimalPlugins.build().unwrap().finish(),
        DefaultPlugins.build().unwrap().finish(),
        DevPlugins.build().unwrap().finish(),
        HeadlessPlugins.build().unwrap().finish(),
    ] {
        let descriptors = group.module_descriptors();
        let sorted_names = sort_module_activation_order(&descriptors).unwrap();
        assert_eq!(
            group.module_keys(),
            sorted_names.iter().map(String::as_str).collect::<Vec<_>>(),
            "{} should be ordered by module descriptors",
            group.name()
        );
    }
}
