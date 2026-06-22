//! Runtime test fixture helpers for plugin integration tests.

use std::any::Any;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::core::{CoreError, CoreHandle, CoreRuntime, ModuleDescriptor};
use zircon_runtime::plugin::{
    RuntimeExtensionCatalogReport, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
use zircon_runtime::{asset, foundation, scene};

const DEFAULT_FIXED_TIMESTEP_NANOS: u64 = 1_000_000_000 / 60;
const DEFAULT_MAX_FIXED_STEPS: u32 = 4;

pub type Result<T> = std::result::Result<T, TestRuntimeError>;

#[derive(Debug)]
pub enum TestRuntimeError {
    RuntimeExtensionCatalog {
        diagnostics: Vec<String>,
        fatal_diagnostics: Vec<String>,
    },
    Core {
        action: &'static str,
        target: String,
        source: CoreError,
    },
    RuntimeExtensionRegistry {
        action: &'static str,
        source: RuntimeExtensionRegistryError,
    },
}

impl fmt::Display for TestRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuntimeExtensionCatalog {
                diagnostics,
                fatal_diagnostics,
            } => write!(
                f,
                "runtime extension catalog has fatal diagnostics: {:?}; diagnostics: {:?}",
                fatal_diagnostics, diagnostics
            ),
            Self::Core {
                action,
                target,
                source,
            } => write!(f, "test runtime {action} failed for {target}: {source}"),
            Self::RuntimeExtensionRegistry { action, source } => {
                write!(f, "test runtime {action} failed: {source}")
            }
        }
    }
}

impl Error for TestRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core { source, .. } => Some(source),
            Self::RuntimeExtensionRegistry { source, .. } => Some(source),
            Self::RuntimeExtensionCatalog { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestRuntimeBaseModule {
    Foundation,
    Asset,
    Scene,
}

impl TestRuntimeBaseModule {
    pub fn default_stack() -> [Self; 3] {
        [Self::Foundation, Self::Asset, Self::Scene]
    }

    pub fn module_name(self) -> &'static str {
        match self {
            Self::Foundation => foundation::FOUNDATION_MODULE_NAME,
            Self::Asset => asset::ASSET_MODULE_NAME,
            Self::Scene => scene::SCENE_MODULE_NAME,
        }
    }

    fn descriptor(self) -> ModuleDescriptor {
        match self {
            Self::Foundation => foundation::module_descriptor(),
            Self::Asset => asset::module_descriptor(),
            Self::Scene => scene::module_descriptor(),
        }
    }
}

#[derive(Debug)]
pub struct TestRuntime {
    runtime: CoreRuntime,
    extension_report: RuntimeExtensionCatalogReport,
    activated_modules: Vec<String>,
    max_fixed_steps: u32,
}

impl TestRuntime {
    pub fn builder() -> TestRuntimeBuilder {
        TestRuntimeBuilder::default()
    }

    pub fn runtime(&self) -> &CoreRuntime {
        &self.runtime
    }

    pub fn into_runtime(self) -> CoreRuntime {
        self.runtime
    }

    pub fn handle(&self) -> CoreHandle {
        self.runtime.handle()
    }

    pub fn extension_report(&self) -> &RuntimeExtensionCatalogReport {
        &self.extension_report
    }

    pub fn activated_modules(&self) -> &[String] {
        &self.activated_modules
    }

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>> {
        self.runtime
            .resolve_manager(name)
            .map_err(|source| TestRuntimeError::Core {
                action: "resolve manager",
                target: name.to_string(),
                source,
            })
    }

    pub fn create_default_level(&self) -> Result<scene::LevelSystem> {
        scene::create_default_level(&self.handle()).map_err(|source| TestRuntimeError::Core {
            action: "create default level",
            target: scene::SCENE_MODULE_NAME.to_string(),
            source,
        })
    }

    pub fn advance_time_by(
        &self,
        real_delta: Duration,
    ) -> zircon_runtime::core::RuntimeTimeAdvance {
        self.runtime
            .advance_time_by(real_delta, self.max_fixed_steps)
    }

    pub fn advance_time_by_seconds(
        &self,
        seconds: f64,
    ) -> zircon_runtime::core::RuntimeTimeAdvance {
        self.advance_time_by(duration_from_seconds(seconds))
    }

    pub fn tick_level_seconds(&self, level: &scene::LevelSystem, seconds: f64) -> Result<()> {
        let advance = self.advance_time_by_seconds(seconds);
        level
            .tick(&self.handle(), advance)
            .map_err(|source| TestRuntimeError::Core {
                action: "tick level",
                target: scene::SCENE_MODULE_NAME.to_string(),
                source,
            })
    }
}

#[derive(Debug)]
pub struct TestRuntimeBuilder {
    fixed_timestep: Option<Duration>,
    max_fixed_steps: u32,
    base_modules: Vec<TestRuntimeBaseModule>,
    runtime_registrations: Vec<RuntimePluginRegistrationReport>,
    feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
    install_scene_runtime_hooks: bool,
    install_world_runtime_extensions: bool,
    activate_base_modules: bool,
    activate_plugin_modules: bool,
}

impl Default for TestRuntimeBuilder {
    fn default() -> Self {
        Self {
            fixed_timestep: Some(default_fixed_timestep()),
            max_fixed_steps: DEFAULT_MAX_FIXED_STEPS,
            base_modules: TestRuntimeBaseModule::default_stack().to_vec(),
            runtime_registrations: Vec::new(),
            feature_registrations: Vec::new(),
            install_scene_runtime_hooks: true,
            install_world_runtime_extensions: true,
            activate_base_modules: true,
            activate_plugin_modules: true,
        }
    }
}

impl TestRuntimeBuilder {
    pub fn with_fixed_timestep(mut self, timestep: Duration) -> Self {
        self.fixed_timestep = Some(timestep);
        self
    }

    pub fn without_fixed_timestep(mut self) -> Self {
        self.fixed_timestep = None;
        self
    }

    pub fn with_max_fixed_steps(mut self, max_fixed_steps: u32) -> Self {
        self.max_fixed_steps = max_fixed_steps;
        self
    }

    pub fn with_base_modules(
        mut self,
        modules: impl IntoIterator<Item = TestRuntimeBaseModule>,
    ) -> Self {
        self.base_modules = modules.into_iter().collect();
        self
    }

    pub fn without_base_modules(mut self) -> Self {
        self.base_modules.clear();
        self
    }

    pub fn with_runtime_plugin(mut self, plugin: &dyn RuntimePlugin) -> Self {
        self.runtime_registrations
            .push(RuntimePluginRegistrationReport::from_plugin(plugin));
        self
    }

    pub fn with_runtime_plugins<'plugin>(
        mut self,
        plugins: impl IntoIterator<Item = &'plugin dyn RuntimePlugin>,
    ) -> Self {
        for plugin in plugins {
            self.runtime_registrations
                .push(RuntimePluginRegistrationReport::from_plugin(plugin));
        }
        self
    }

    pub fn with_registration_report(mut self, report: RuntimePluginRegistrationReport) -> Self {
        self.runtime_registrations.push(report);
        self
    }

    pub fn with_feature_registration_report(
        mut self,
        report: RuntimePluginFeatureRegistrationReport,
    ) -> Self {
        self.feature_registrations.push(report);
        self
    }

    pub fn without_scene_runtime_hooks(mut self) -> Self {
        self.install_scene_runtime_hooks = false;
        self
    }

    pub fn without_world_runtime_extensions(mut self) -> Self {
        self.install_world_runtime_extensions = false;
        self
    }

    pub fn without_base_module_activation(mut self) -> Self {
        self.activate_base_modules = false;
        self
    }

    pub fn without_plugin_module_activation(mut self) -> Self {
        self.activate_plugin_modules = false;
        self
    }

    pub fn build(self) -> Result<TestRuntime> {
        let runtime = CoreRuntime::new();
        if let Some(fixed_timestep) = self.fixed_timestep {
            runtime.set_fixed_timestep(fixed_timestep);
        }

        for module in &self.base_modules {
            register_module(&runtime, module.descriptor())?;
        }

        let catalog = RuntimePluginCatalog::from_registration_reports(
            self.runtime_registrations,
            self.feature_registrations,
        );
        let extension_report = catalog.runtime_extensions();
        if extension_report.has_fatal_diagnostics() {
            return Err(TestRuntimeError::RuntimeExtensionCatalog {
                diagnostics: extension_report.diagnostics,
                fatal_diagnostics: extension_report.fatal_diagnostics,
            });
        }

        for module in extension_report.registry.modules() {
            register_module(&runtime, module.clone())?;
        }
        if self.install_scene_runtime_hooks {
            runtime
                .install_scene_runtime_hooks(&extension_report.registry)
                .map_err(|source| TestRuntimeError::RuntimeExtensionRegistry {
                    action: "install scene runtime hooks",
                    source,
                })?;
        }
        if self.install_world_runtime_extensions {
            runtime
                .install_world_runtime_extensions(&extension_report.registry)
                .map_err(|source| TestRuntimeError::RuntimeExtensionRegistry {
                    action: "install world runtime extensions",
                    source,
                })?;
        }

        let mut activated_modules = Vec::new();
        if self.activate_base_modules {
            for module in &self.base_modules {
                activate_module(&runtime, module.module_name())?;
                activated_modules.push(module.module_name().to_string());
            }
        }
        if self.activate_plugin_modules {
            for module in extension_report.registry.modules() {
                activate_module(&runtime, &module.name)?;
                activated_modules.push(module.name.clone());
            }
        }

        Ok(TestRuntime {
            runtime,
            extension_report,
            activated_modules,
            max_fixed_steps: self.max_fixed_steps,
        })
    }
}

fn register_module(runtime: &CoreRuntime, descriptor: ModuleDescriptor) -> Result<()> {
    let module_name = descriptor.name.clone();
    runtime
        .register_module(descriptor)
        .map_err(|source| TestRuntimeError::Core {
            action: "register module",
            target: module_name,
            source,
        })
}

fn activate_module(runtime: &CoreRuntime, module_name: &str) -> Result<()> {
    runtime
        .activate_module(module_name)
        .map_err(|source| TestRuntimeError::Core {
            action: "activate module",
            target: module_name.to_string(),
            source,
        })
}

fn default_fixed_timestep() -> Duration {
    Duration::from_nanos(DEFAULT_FIXED_TIMESTEP_NANOS)
}

fn duration_from_seconds(seconds: f64) -> Duration {
    if seconds.is_finite() && seconds > 0.0 {
        Duration::from_secs_f64(seconds)
    } else {
        Duration::ZERO
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
    use zircon_runtime::core::runtime::ServiceObject;
    use zircon_runtime::core::{ManagerDescriptor, ServiceKind, StartupMode};
    use zircon_runtime::engine_module::{factory, qualified_name};
    use zircon_runtime::plugin::{RuntimeExtensionRegistry, RuntimePluginDescriptor};

    use super::*;

    const TEST_PACKAGE_ID: &str = "prefab_tools";
    const TEST_PLUGIN_MODULE_NAME: &str = "prefab_tools.runtime";
    const TEST_RUNTIME_MODULE_NAME: &str = "SdkTestRuntimeModule";
    const TEST_MANAGER_NAME: &str = "SdkTestRuntimeModule.Manager.SdkTestManager";

    #[derive(Debug)]
    struct SdkTestManager;

    #[derive(Clone, Debug)]
    struct SdkTestRuntimePlugin {
        descriptor: RuntimePluginDescriptor,
    }

    impl SdkTestRuntimePlugin {
        fn new() -> Self {
            Self {
                descriptor: RuntimePluginDescriptor::builder(
                    TEST_PACKAGE_ID,
                    "SDK Test Runtime",
                    RuntimePluginId::PrefabTools,
                    "zircon_plugin_sdk_test_runtime",
                )
                .with_category("runtime")
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capability("runtime.plugin.prefab_tools")
                .build(),
            }
        }
    }

    impl RuntimePlugin for SdkTestRuntimePlugin {
        fn descriptor(&self) -> &RuntimePluginDescriptor {
            &self.descriptor
        }

        fn register(
            &self,
            registry: &mut RuntimeExtensionRegistry,
        ) -> std::result::Result<(), RuntimeExtensionRegistryError> {
            let owner = registry.intern_plugin_module(TEST_PLUGIN_MODULE_NAME)?;
            assert_eq!(
                registry.plugin_module_name(owner),
                Some(TEST_PLUGIN_MODULE_NAME)
            );
            registry.register_module(test_runtime_module_descriptor())
        }
    }

    fn test_runtime_module_descriptor() -> ModuleDescriptor {
        ModuleDescriptor::new(TEST_RUNTIME_MODULE_NAME, "SDK test runtime module").with_manager(
            ManagerDescriptor::new(
                qualified_name(
                    TEST_RUNTIME_MODULE_NAME,
                    ServiceKind::Manager,
                    "SdkTestManager",
                ),
                StartupMode::Immediate,
                Vec::new(),
                factory(|_| Ok(Arc::new(SdkTestManager) as ServiceObject)),
            ),
        )
    }

    #[test]
    fn test_runtime_builder_registers_base_and_plugin_modules() {
        let plugin = SdkTestRuntimePlugin::new();
        let runtime = TestRuntime::builder()
            .with_runtime_plugin(&plugin)
            .build()
            .expect("SDK test runtime should build");

        assert!(runtime
            .activated_modules()
            .contains(&foundation::FOUNDATION_MODULE_NAME.to_string()));
        assert!(runtime
            .activated_modules()
            .contains(&asset::ASSET_MODULE_NAME.to_string()));
        assert!(runtime
            .activated_modules()
            .contains(&scene::SCENE_MODULE_NAME.to_string()));
        assert!(runtime
            .activated_modules()
            .contains(&TEST_RUNTIME_MODULE_NAME.to_string()));
        runtime
            .resolve_manager::<SdkTestManager>(TEST_MANAGER_NAME)
            .expect("plugin manager should resolve after module activation");
        assert!(runtime.extension_report().is_success());
    }

    #[test]
    fn test_runtime_builder_can_build_scene_levels_with_extensions_installed() {
        let plugin = SdkTestRuntimePlugin::new();
        let runtime = TestRuntime::builder()
            .with_runtime_plugin(&plugin)
            .build()
            .expect("SDK test runtime should build");

        let level = runtime
            .create_default_level()
            .expect("default level should include runtime world extensions");
        runtime
            .tick_level_seconds(&level, 1.0 / 60.0)
            .expect("level tick should use the SDK runtime clock");
    }
}
