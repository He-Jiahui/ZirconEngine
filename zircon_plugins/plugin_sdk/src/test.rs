//! Runtime test fixture helpers for plugin integration tests.
//! builder 在真实 CoreRuntime 上执行模块注册、扩展目录投影及可选激活，便于集成测试走生产路径。

use std::any::Any;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::core::{
    CoreError, CoreHandle, CoreRuntime, ModuleDescriptor, TimePolicyError, TimePolicyTransaction,
};
use zircon_runtime::plugin::{
    RuntimeExtensionCatalogReport, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
use zircon_runtime::{asset, foundation, scene};

const DEFAULT_FIXED_TIMESTEP_NANOS: u64 = 1_000_000_000 / 60;
const DEFAULT_MAX_FIXED_STEPS: u32 = 4;

pub type Result<T> = std::result::Result<T, TestRuntimeError>;

#[derive(Debug)]
/// 测试运行时构建、模块操作、时间策略或关卡 tick 的错误边界。
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
    LevelTick {
        action: &'static str,
        target: String,
        source: scene::LevelTickError,
    },
    RuntimeExtensionRegistry {
        action: &'static str,
        source: RuntimeExtensionRegistryError,
    },
    TimePolicy {
        source: TimePolicyError,
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
            Self::LevelTick {
                action,
                target,
                source,
            } => write!(f, "test runtime {action} failed for {target}: {source}"),
            Self::RuntimeExtensionRegistry { action, source } => {
                write!(f, "test runtime {action} failed: {source}")
            }
            Self::TimePolicy { source } => write!(f, "test runtime time policy failed: {source}"),
        }
    }
}

impl Error for TestRuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core { source, .. } => Some(source),
            Self::LevelTick { source, .. } => Some(source),
            Self::RuntimeExtensionRegistry { source, .. } => Some(source),
            Self::TimePolicy { source } => Some(source),
            Self::RuntimeExtensionCatalog { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 测试夹具可选安装的基础运行时模块。
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
            Self::Scene => SCENE_MODULE_NAME,
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
/// 已构建的 CoreRuntime、扩展目录报告和本次实际激活的模块名。
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
            target: SCENE_MODULE_NAME.to_string(),
            source,
        })
    }

    pub fn advance_time_by(&self, real_delta: Duration) -> zircon_runtime::core::FrameTimeSnapshot {
        self.runtime
            .advance_time_by(real_delta, self.max_fixed_steps)
    }

    pub fn advance_time_by_seconds(&self, seconds: f64) -> zircon_runtime::core::FrameTimeSnapshot {
        self.advance_time_by(duration_from_seconds(seconds))
    }

    /// 先推进运行时帧时间，再用该快照驱动指定关卡一次 tick。
    pub fn tick_level_seconds(&self, level: &scene::LevelSystem, seconds: f64) -> Result<()> {
        let advance = self.advance_time_by_seconds(seconds);
        level
            .tick(&self.handle(), advance)
            .map_err(|source| TestRuntimeError::LevelTick {
                action: "tick level",
                target: SCENE_MODULE_NAME.to_string(),
                source,
            })
    }
}

#[derive(Debug)]
/// 控制基础模块、插件注册报告、扩展计划安装、激活与固定步长的测试夹具 builder。
pub struct TestRuntimeBuilder {
    fixed_timestep: Option<Duration>,
    max_fixed_steps: u32,
    base_modules: Vec<TestRuntimeBaseModule>,
    runtime_registrations: Vec<RuntimePluginRegistrationReport>,
    feature_registrations: Vec<RuntimePluginFeatureRegistrationReport>,
    install_scene_runtime_extension_plan: bool,
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
            install_scene_runtime_extension_plan: true,
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

    /// 替换默认的 Foundation、Asset、Scene 模块栈。
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

    /// 立即从插件对象提取注册报告；builder 不保留该对象的借用。
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

    /// 跳过 world runtime extension plan 的安装，不跳过扩展目录收集与模块注册。
    pub fn without_scene_runtime_extension_plan(mut self) -> Self {
        self.install_scene_runtime_extension_plan = false;
        self
    }

    pub fn without_base_module_activation(mut self) -> Self {
        self.activate_base_modules = false;
        self
    }

    /// 仍收集、注册插件模块并安装扩展计划，只跳过最后的插件模块激活。
    pub fn without_plugin_module_activation(mut self) -> Self {
        self.activate_plugin_modules = false;
        self
    }

    /// 按时间策略、基础模块注册、插件扩展目录、模块注册、计划安装、模块激活的顺序构建运行时。
    pub fn build(self) -> Result<TestRuntime> {
        let runtime = CoreRuntime::new();
        if let Some(fixed_timestep) = self.fixed_timestep {
            runtime
                .apply_time_policy(TimePolicyTransaction::new(
                    runtime.time_policy().with_fixed_timestep(fixed_timestep),
                ))
                .map_err(|source| TestRuntimeError::TimePolicy { source })?;
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
        if self.install_scene_runtime_extension_plan {
            let plan = extension_report
                .registry
                .world_runtime_extension_plan()
                .map_err(|source| TestRuntimeError::RuntimeExtensionRegistry {
                    action: "project world runtime extensions",
                    source,
                })?;
            scene::install_world_runtime_extension_plan(&runtime.handle(), plan).map_err(
                |source| TestRuntimeError::Core {
                    action: "install world runtime extensions",
                    target: scene::WORLD_DRIVER_NAME.to_string(),
                    source,
                },
            )?;
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
#[path = "tests/test.rs"]
mod tests;
