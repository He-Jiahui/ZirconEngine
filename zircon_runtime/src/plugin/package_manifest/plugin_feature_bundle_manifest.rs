//! 可选能力束与独立提供包共享的声明模型；目录按所有者和提供包建立特性索引。
use serde::{Deserialize, Serialize};

use crate::core::framework::project::ExportPackagingStrategy;

use super::{PluginDistributionManifest, PluginFeatureDependency, PluginModuleManifest};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// owner_plugin_id 标识能力归属；provider_package_id 可指向单独交付的包。
/// 依赖必须含恰好一个指向所有者的 primary 项，启用状态还取决于项目选择与目标。
pub struct PluginFeatureBundleManifest {
    pub id: String,
    pub display_name: String,
    pub owner_plugin_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_package_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution: Option<PluginDistributionManifest>,
    #[serde(default)]
    pub dependencies: Vec<PluginFeatureDependency>,
    #[serde(default)]
    pub modules: Vec<PluginModuleManifest>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    // TODO: [CR-PLUGIN-BOUNDARY-0203] 确认清单省略打包策略应视为错误还是沿用构造器默认；
    // 反序列化得到空列表而构造器给出两种策略，注册校验拒绝空列表；下一步补缺字段的清单契约测试。
    pub default_packaging: Vec<ExportPackagingStrategy>,
    #[serde(default)]
    pub enabled_by_default: bool,
}

impl PluginFeatureBundleManifest {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        owner_plugin_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            owner_plugin_id: owner_plugin_id.into(),
            provider_package_id: None,
            distribution: None,
            dependencies: Vec::new(),
            modules: Vec::new(),
            capabilities: Vec::new(),
            default_packaging: vec![
                ExportPackagingStrategy::SourceTemplate,
                ExportPackagingStrategy::LibraryEmbed,
            ],
            enabled_by_default: false,
        }
    }

    pub fn with_dependency(mut self, dependency: PluginFeatureDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// 为普通所有者包声明外部提供者；特性扩展包在目录合并时以自身包 ID 为准。
    pub fn with_provider_package_id(mut self, provider_package_id: impl Into<String>) -> Self {
        self.provider_package_id = Some(provider_package_id.into());
        self
    }

    pub fn with_distribution(mut self, distribution: PluginDistributionManifest) -> Self {
        self.distribution = Some(distribution);
        self
    }

    pub fn with_runtime_module(mut self, module: PluginModuleManifest) -> Self {
        self.modules.push(module);
        self
    }

    pub fn with_editor_module(mut self, module: PluginModuleManifest) -> Self {
        self.modules.push(module);
        self
    }

    pub fn with_native_module(mut self, module: PluginModuleManifest) -> Self {
        self.modules.push(module);
        self
    }

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.default_packaging = packaging.into_iter().collect();
        self
    }

    /// 仅设置项目选择的初始意向，不能跳过依赖、目标、提供者或注册校验。
    pub fn enabled_by_default(mut self, enabled: bool) -> Self {
        self.enabled_by_default = enabled;
        self
    }
}
