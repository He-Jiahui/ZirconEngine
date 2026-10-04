//! profile 的拥有型装配意图：既可由生成预设构造，也可由测试或宿主定制。
//! 默认插件可转为项目清单；可选项仅参与可用性展示，不自动启用。

use serde::{Deserialize, Serialize};

use crate::core::framework::project::{
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use crate::plugin::PluginMaturity;
use crate::{
    builtin::{BuiltinRuntimeModuleId, RuntimePluginId},
    core::framework::platform::RuntimeTargetMode,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 单个 profile 默认插件及其是否为装配硬要求。
pub struct RuntimeProfilePluginSelection {
    pub id: RuntimePluginId,
    #[serde(default)]
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 运行时模块范围、默认插件、目标与成熟度门槛的组合声明。
pub struct RuntimeProfileDescriptor {
    pub id: RuntimeProfileId,
    pub name: String,
    pub target_mode: RuntimeTargetMode,
    pub builtin_modules: Vec<BuiltinRuntimeModuleId>,
    #[serde(default)]
    pub default_plugins: Vec<RuntimeProfilePluginSelection>,
    #[serde(default)]
    pub optional_plugins: Vec<RuntimePluginId>,
    #[serde(default)]
    // TODO: [CR-PLUGIN-BOUNDARY-0301] 明确能力要求何时由装配器裁决；
    // 当前仅在生成预设和描述符中流转，既有审查归属 Runtime42 P1-13。
    pub required_capabilities: Vec<String>,
    pub minimum_maturity: PluginMaturity,
    #[serde(default)]
    pub allow_externalized_required_plugins: bool,
}

impl RuntimeProfilePluginSelection {
    pub fn new(id: RuntimePluginId, required: bool) -> Self {
        Self { id, required }
    }
}

impl RuntimeProfileDescriptor {
    /// 宿主或测试构造自定义 profile，不会隐式继承内置默认集合。
    pub fn new(
        id: RuntimeProfileId,
        name: impl Into<String>,
        target_mode: RuntimeTargetMode,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            target_mode,
            builtin_modules: Vec::new(),
            default_plugins: Vec::new(),
            optional_plugins: Vec::new(),
            required_capabilities: Vec::new(),
            minimum_maturity: PluginMaturity::Experimental,
            allow_externalized_required_plugins: false,
        }
    }

    pub fn with_builtin_module(mut self, id: BuiltinRuntimeModuleId) -> Self {
        if !self.builtin_modules.contains(&id) {
            self.builtin_modules.push(id);
        }
        self
    }

    pub fn with_builtin_modules(
        mut self,
        ids: impl IntoIterator<Item = BuiltinRuntimeModuleId>,
    ) -> Self {
        let ids = ids.into_iter();
        let (minimum_ids, _) = ids.size_hint();
        self.builtin_modules.reserve(minimum_ids);
        for id in ids {
            if !self.builtin_modules.contains(&id) {
                self.builtin_modules.push(id);
            }
        }
        self
    }

    /// 声明启动默认选择；该列表随后按原序转换为目标限定项目清单。
    pub fn with_default_plugin(mut self, id: RuntimePluginId, required: bool) -> Self {
        self.default_plugins
            .push(RuntimeProfilePluginSelection::new(id, required));
        self
    }

    /// 仅登记可提示的候选，不让可选插件自动进入启动清单。
    pub fn with_optional_plugin(mut self, id: RuntimePluginId) -> Self {
        if !self.optional_plugins.contains(&id) {
            self.optional_plugins.push(id);
        }
        self
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self
    }

    pub fn with_minimum_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.minimum_maturity = maturity;
        self
    }

    pub fn allow_externalized_required_plugins(mut self, allow: bool) -> Self {
        self.allow_externalized_required_plugins = allow;
        self
    }

    /// 给默认启动装配生成目标限定清单；项目显式清单应由调用方另行提供。
    pub fn project_manifest(&self) -> ProjectPluginManifest {
        ProjectPluginManifest {
            selections: self
                .default_plugins
                .iter()
                .map(|plugin| {
                    ProjectPluginSelection::runtime_plugin(plugin.id.clone(), true, plugin.required)
                        .with_target_modes([self.target_mode])
                })
                .collect(),
        }
    }
}

#[cfg(test)]
#[path = "tests/descriptor_optimization_tests.rs"]
mod optimization_tests;
